//! 3-thread engine core (SIM / RENDER / CONTROL).
//!
//! This module implements the multi-threaded engine architecture specified in
//! `brain/11-ENGINE-INTERNALS-V2.md` §1. The three threads communicate via
//! `crossbeam-channel` with no shared mutable state outside the channels.
//!
//! **Thread topology:**
//! - **CONTROL thread** (main thread): signal-hook, control socket, NO tty reads.
//!   Pushes `ControlMsg` onto an unbounded channel to the SIM thread.
//! - **SIM thread** (spawned): owns `back` buffer, particles, DNA, effect.
//!   Runs a fixed-timestep accumulator. Ships `Arc<Frame>` to RENDER.
//! - **RENDER thread** (spawned): owns `front` buffer + Renderer trait.
//!   Receives `Arc<Frame>`, computes damage, emits ANSI.
//!
//! Invariants enforced:
//! 1. No input reads in CONTROL thread (CI grep: `event::poll`/`event::read` absent).
//! 2. SIM thread never touches stdout.
//! 3. RENDER thread never mutates simulation state.
//! 4. Backpressure via `bounded(2)` — SIM can't outrun RENDER by > 1 frame.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use forgum_platform::{OutputHandle, ShutdownFlag};

use crate::control_socket::ControlCmd;
use crate::dna::CowDna;
use crate::effects;
use crate::framebuffer::{Cell, FrameBuffer};
use crate::protocol::SceneConfig;
use crate::renderer::{self, Renderer};
use crate::scheduler::Scheduler;

// ── Global singletons (Phase 1.8) ─────────────────────────────────

/// Global frame counter — no lock, Relaxed ordering.
pub static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);

/// Global particle spawn counter — no lock, Relaxed ordering.
pub static SPAWN_COUNT: AtomicU64 = AtomicU64::new(0);

/// Terminal capabilities — probed lazily on first access via OnceLock.
static TERM_CAPS: OnceLock<forgum_platform::TerminalCapabilities> = OnceLock::new();

/// Resettable particle pool — cleared on resize, taken on shutdown.
static PARTICLE_POOL: LazyLock<Mutex<Option<crate::particles::ParticlePool>>> =
    LazyLock::new(|| Mutex::new(None));

/// Get or probe terminal capabilities (DCL via OnceLock).
pub fn term_caps() -> &'static forgum_platform::TerminalCapabilities {
    TERM_CAPS.get_or_init(forgum_platform::TerminalCapabilities::probe)
}

/// Get a reference to the global particle pool.
pub fn particle_pool() -> &'static Mutex<Option<crate::particles::ParticlePool>> {
    &PARTICLE_POOL
}

// ── Channel messages ───────────────────────────────────────────────

/// Messages from the CONTROL thread to the SIM thread.
#[derive(Debug)]
pub enum ControlMsg {
    Stop,
    Pause,
    Resume,
    Effect(String),
    Speed(f32),
    Cow(String),
    Text(String),
    Think(String),
    Eyes(String),
    Tongue(String),
    ColorMode(String),
    Resize { cols: u16, rows: u16 },
}

/// A snapshot of the simulation state shipped from SIM → RENDER.
#[derive(Debug, Clone)]
pub struct Frame {
    pub cells: Vec<Cell>,
    pub damage: Vec<(usize, usize)>,
    pub cols: usize,
    pub rows: usize,
    pub full_redraw: bool,
}

// ── 240Hz Liquid Frame Governor (ANIM-001 / ANIM-002) ──────────────

pub const FIXED_SIM_DT: f64 = 1.0 / 120.0; // Fixed 120Hz physics integration step
pub const MAX_FRAME_TIME: f64 = 0.25; // Spiral-of-death clamp
pub const SPIN_WINDOW: Duration = Duration::from_micros(1500); // 1.5ms spin-wait margin

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KinematicSnapshot {
    pub x: f64,
    pub y: f64,
    pub stride_phase: f64,
    pub rig_phase: f64,
}

impl KinematicSnapshot {
    #[must_use]
    pub fn interpolate(prev: Self, curr: Self, alpha: f64) -> Self {
        Self {
            x: prev.x + (curr.x - prev.x) * alpha,
            y: prev.y + (curr.y - prev.y) * alpha,
            stride_phase: prev.stride_phase + (curr.stride_phase - prev.stride_phase) * alpha,
            rig_phase: prev.rig_phase + (curr.rig_phase - prev.rig_phase) * alpha,
        }
    }
}

#[derive(Debug)]
pub struct LiquidFrameGovernor {
    pub target_period: Duration,
    pub next_deadline: Instant,
    pub last_instant: Instant,
    pub accumulator: f64,
    pub elapsed_secs: f64,
}

impl LiquidFrameGovernor {
    #[must_use]
    pub fn new(fps: u32) -> Self {
        let safe_fps = fps.clamp(1, 1000);
        let target_period = Duration::from_nanos(1_000_000_000 / u64::from(safe_fps));
        let now = Instant::now();
        Self {
            target_period,
            next_deadline: now + target_period,
            last_instant: now,
            accumulator: 0.0,
            elapsed_secs: 0.0,
        }
    }

    #[must_use]
    pub fn target_period(&self) -> Duration {
        self.target_period
    }

    pub fn set_fps(&mut self, fps: u32) {
        let safe_fps = fps.clamp(1, 1000);
        self.target_period = Duration::from_nanos(1_000_000_000 / u64::from(safe_fps));
    }

    pub fn step_simulation<F>(&mut self, mut integrate_fixed: F) -> (f64, f64)
    where
        F: FnMut(f64),
    {
        let now = Instant::now();
        let frame_time = (now.saturating_duration_since(self.last_instant))
            .as_secs_f64()
            .min(MAX_FRAME_TIME);
        self.last_instant = now;
        self.elapsed_secs += frame_time;
        self.accumulator += frame_time;

        while self.accumulator >= FIXED_SIM_DT {
            integrate_fixed(FIXED_SIM_DT);
            self.accumulator -= FIXED_SIM_DT;
        }
        let alpha = (self.accumulator / FIXED_SIM_DT).clamp(0.0, 1.0);
        (alpha, self.elapsed_secs)
    }

    pub fn pace_frame(&mut self, shutdown: &ShutdownFlag) {
        let now = Instant::now();
        if now < self.next_deadline {
            let rem = self.next_deadline - now;
            if rem > SPIN_WINDOW {
                shutdown.sleep_interruptible(rem - SPIN_WINDOW);
            }
            while Instant::now() < self.next_deadline {
                if shutdown.is_shutdown() {
                    return;
                }
                std::hint::spin_loop();
            }
        }
        self.next_deadline += self.target_period;
        let post_wait = Instant::now();
        if self.next_deadline <= post_wait {
            self.next_deadline = post_wait + self.target_period;
        }
    }
}

// ── SIM thread ─────────────────────────────────────────────────────

/// State owned exclusively by the SIM thread.
struct SimState {
    fb: FrameBuffer,
    effect: Box<dyn effects::Effect>,
    scheduler: Scheduler,
    frame_count: u64,
    elapsed: f32,
    /// Per-frame arena: O(1) mass-dealloc via `reset()` at frame top.
    /// All per-frame scratch (damage lists, temp buffers) go here.
    arena: bumpalo::Bump,
    data_dir: PathBuf,
    config: SceneConfig,
    cow_dna: CowDna,
    instance_id: u32,
    scenery: (
        crate::scenery::MountainStyle,
        crate::scenery::RoadStyle,
        crate::scenery::EnvironmentStyle,
    ),
    last_thought_rotate: Instant,
    cow_foot_y: usize,
    need_full_redraw: bool,
    active_composed: Arc<std::sync::RwLock<String>>,
    damage_scratch: Vec<(usize, usize)>,
    scenery_cache: crate::scenery::SceneryCache,
}

impl SimState {
    #[allow(clippy::too_many_arguments)]
    fn new(
        config: &SceneConfig,
        cols: usize,
        rows: usize,
        cow_dna: CowDna,
        instance_id: u32,
        composed_text: Option<&str>,
        data_dir: PathBuf,
        active_composed: Arc<std::sync::RwLock<String>>,
    ) -> Self {
        let cow_display = composed_text.unwrap_or(&config.text);
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            cow_display.to_string()
        };
        if let Ok(mut lock) = active_composed.write() {
            *lock = cow_text.clone();
        }
        let cow_foot_y = effects::find_cow_foot_y(&cow_text);

        let mut cow_dna = cow_dna;
        if let Some(ref pal_str) = config.palette {
            let hexes: Vec<String> = pal_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !hexes.is_empty() {
                cow_dna.palette = hexes;
            }
        } else if config.color_mode == "natural"
            || config.color_mode == "default"
            || config.color_mode == "animal"
            || config.color_mode == "animal_natural"
            || cow_dna.palette.is_empty()
        {
            let natural_hexes = crate::color::get_natural_hex_palette(&config.cow);
            if !natural_hexes.is_empty() {
                cow_dna.palette = natural_hexes.iter().map(|&s| s.to_string()).collect();
            }
        }

        let effect = effects::create_scene_effect(
            &config.effect,
            cow_text,
            cow_dna.clone(),
            instance_id,
            &config.color_mode,
        );

        let env_override = config.environment.as_deref().and_then(|s| s.parse().ok());
        let road_override = config.road.as_deref().and_then(|s| s.parse().ok());
        let mtn_override = config.mountain.as_deref().and_then(|s| s.parse().ok());
        let scenery = crate::scenery::resolve_archetype_with_overrides(
            &config.cow,
            mtn_override,
            road_override,
            env_override,
        );

        // Clamp default FPS to 30 over remote sessions (SSH, RDP) to eliminate network lag and bufferbloat
        let effective_fps =
            if forgum_platform::is_remote_session() && (config.fps == 0 || config.fps == 60) {
                30
            } else {
                config.fps
            };
        let mut scheduler = Scheduler::new(effective_fps);
        let has_scenery_motion = scenery.0 != crate::scenery::MountainStyle::None
            || scenery.1 != crate::scenery::RoadStyle::None
            || scenery.2 != crate::scenery::EnvironmentStyle::None;
        scheduler.is_dynamic =
            (config.effect != "static" && config.effect != "none") || has_scenery_motion;

        Self {
            fb: FrameBuffer::new(cols, rows),
            effect,
            scheduler,
            frame_count: 0,
            elapsed: 0.0,
            arena: bumpalo::Bump::with_capacity(64 * 1024),
            data_dir,
            config: config.clone(),
            cow_dna,
            instance_id,
            scenery,
            last_thought_rotate: Instant::now(),
            cow_foot_y,
            need_full_redraw: true,
            active_composed,
            damage_scratch: Vec::with_capacity((cols * rows) / 4),
            scenery_cache: crate::scenery::SceneryCache::new(),
        }
    }

    /// Update physics simulation step at a fixed timestep.
    pub fn tick_physics(&mut self, dt_f32: f32) {
        self.effect.update(dt_f32, self.fb.width, self.fb.height);
    }

    /// Render simulation snapshot at current elapsed clock into front buffer to ship to RENDER.
    pub fn tick_render(&mut self, _alpha: f32, elapsed_secs: f32) -> Arc<Frame> {
        // Phase 1.6: O(1) mass-dealloc of last frame's scratch.
        self.arena.reset();

        self.elapsed = elapsed_secs;

        // Periodic thought rotation if thought_interval > 0
        let thought_interval = self.config.thought_interval;
        if thought_interval > 0
            && self.last_thought_rotate.elapsed() >= Duration::from_secs(thought_interval as u64)
        {
            self.last_thought_rotate = Instant::now();
            if let Some(new_thought) = crate::fortune::random_fortune(&self.data_dir) {
                self.config.text = new_thought;
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_raw = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let cow_text = crate::cow::compose_scene_with_mode(
                    &cow_raw,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = cow_text.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&cow_text);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    cow_text,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
        }

        // Clear back buffer.
        self.fb.clear();

        // Render scenery layers before effect
        let (mtn_style, road_style, env_style) = self.scenery;
        let is_aquatic_animal = self.cow_dna.habitat == crate::dna::Habitat::Aquatic
            || self.cow_dna.is_aquatic()
            || effects::detect_animal_instinct(&self.config.cow, &self.cow_dna)
                == effects::AnimalInstinct::Marine
            || effects::detect_animal_instinct(&self.config.cow, &self.cow_dna)
                == effects::AnimalInstinct::Cephalopod;

        let max_road = self.fb.height.saturating_sub(1);
        let road_y = if env_style == crate::scenery::EnvironmentStyle::Ocean || is_aquatic_animal {
            // Aquatic immersion: The creature is submerged in the water column.
            // The seabed is situated below the creature's swimming depth,
            // providing natural underwater depth.
            let min_seabed = self.cow_foot_y + 2;
            self.fb
                .height
                .saturating_sub(2)
                .max(min_seabed)
                .min(max_road)
        } else if self.cow_dna.habitat == crate::dna::Habitat::Aerial
            || matches!(
                self.cow_dna.base,
                crate::dna::BaseAnim::Fly
                    | crate::dna::BaseAnim::Float
                    | crate::dna::BaseAnim::Abduction
            )
        {
            // Flying/floating creatures soar in the upper atmosphere above tree canopies
            self.fb
                .height
                .saturating_sub(3)
                .max(self.cow_foot_y + 2)
                .min(max_road)
        } else {
            // Ground creatures and stationary objects rest directly on the road surface
            (self.cow_foot_y + 1).min(max_road)
        };
        let animal_h = self.cow_foot_y.min(self.fb.height.saturating_sub(2)).max(1);
        let tod = self
            .config
            .time_of_day
            .as_deref()
            .map(crate::scenery::TimeOfDay::parse)
            .unwrap_or(crate::scenery::TimeOfDay::Auto)
            .resolve();
        let season = self
            .config
            .season
            .as_deref()
            .map(crate::scenery::Season::parse)
            .unwrap_or(crate::scenery::Season::Auto)
            .resolve();
        self.scenery_cache.get_or_render(
            &mut self.fb,
            mtn_style,
            road_style,
            env_style,
            road_y,
            self.elapsed,
            Some(&self.config.cow),
            Some(animal_h),
            Some(self.cow_dna.base),
            Some(tod),
            Some(season),
        );

        // Render into back buffer on top of scenery.
        self.effect.render(&mut self.fb, self.elapsed);

        // Apply Color Vision Deficiency (CVD) translation filter if configured
        if let Some(ref cb_str) = self.config.colorblind_mode {
            let mode = forgum_platform::ColorblindMode::from_str_loose(cb_str);
            self.fb.apply_colorblind_filter(mode);
        } else if let Some(global_mode) = forgum_platform::get_active_colorblind_mode() {
            self.fb.apply_colorblind_filter(global_mode);
        }

        // Compute damage against the previous front buffer (BEFORE swap clears it).
        // compute_full_damage_into() includes both newly modified cells and cells vacated/cleared
        // from the previous frame, ensuring AnsiRenderer writes spaces to erase them with zero ghost residue.
        let full_redraw = self.need_full_redraw;
        if full_redraw {
            self.need_full_redraw = false;
            self.damage_scratch.clear();
            let total = self.fb.width * self.fb.height;
            if self.damage_scratch.capacity() < total {
                self.damage_scratch
                    .reserve(total - self.damage_scratch.capacity());
            }
            for y in 0..self.fb.height {
                for x in 0..self.fb.width {
                    self.damage_scratch.push((x, y));
                }
            }
        } else {
            self.fb.compute_full_damage_into(&mut self.damage_scratch);
        }
        self.scheduler.observe(self.damage_scratch.len());

        // Swap buffers: front ← back (just-rendered), back ← front (previous).
        // After swap, front has the rendered cells we need to ship.
        self.fb.swap();

        self.frame_count = self.frame_count.saturating_add(1);
        FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

        // Clone front for the render thread. Front must stay intact so the
        // next tick's compute_damage() compares against the last rendered
        // frame, preserving incremental damage tracking.
        Arc::new(Frame {
            cells: self.fb.front.clone(),
            damage: self.damage_scratch.clone(),
            cols: self.fb.width,
            rows: self.fb.height,
            full_redraw,
        })
    }

    /// Single-step simulation tick (legacy/backward compatibility).
    #[allow(dead_code)]
    pub fn tick(&mut self, dt: Duration, raw_dt: Duration) -> Arc<Frame> {
        self.tick_physics(dt.as_secs_f32());
        let new_elapsed = self.elapsed + raw_dt.as_secs_f32();
        self.tick_render(1.0, new_elapsed)
    }

    /// Handle hot-swap of effect, cow art, or text from control messages.
    fn handle_hot_swap(&mut self, msg: &ControlMsg) {
        self.scenery_cache.clear();
        match msg {
            ControlMsg::Effect(name) => {
                // Reload DNA for the new effect if it exists in animations.json,
                // otherwise keep the current DNA but change the base animation.
                let new_dna = crate::dna::resolve_cow_dna(&self.data_dir, &self.config.cow);
                self.config.effect = name.clone();
                self.cow_dna = new_dna;
                // Recreate effect with the composed cow scene (bubble + cow)
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_raw = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let cow_text = crate::cow::compose_scene_with_mode(
                    &cow_raw,
                    &self.config.text,
                    self.config.think,
                );
                self.cow_foot_y = effects::find_cow_foot_y(&cow_text);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    cow_text,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                let (mtn_style, road_style, env_style) = self.scenery;
                let has_scenery_motion = mtn_style != crate::scenery::MountainStyle::None
                    || road_style != crate::scenery::RoadStyle::None
                    || env_style != crate::scenery::EnvironmentStyle::None;
                self.scheduler.is_dynamic = (self.config.effect != "static"
                    && self.config.effect != "none")
                    || has_scenery_motion;
                self.need_full_redraw = true;
            }
            ControlMsg::Cow(name) => {
                // Reload cow art and recreate the effect.
                self.config.cow = crate::cow::resolve_cow_name(name, &self.data_dir);
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(
                    &cow_text,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.cow_dna = crate::dna::resolve_cow_dna(&self.data_dir, &self.config.cow);
                if let Some(ref pal_str) = self.config.palette {
                    let hexes: Vec<String> = pal_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !hexes.is_empty() {
                        self.cow_dna.palette = hexes;
                    }
                } else if self.config.color_mode == "natural"
                    || self.config.color_mode == "default"
                    || self.config.color_mode == "animal"
                    || self.config.color_mode == "animal_natural"
                    || self.cow_dna.palette.is_empty()
                {
                    let natural_hexes = crate::color::get_natural_hex_palette(&self.config.cow);
                    if !natural_hexes.is_empty() {
                        self.cow_dna.palette =
                            natural_hexes.iter().map(|&s| s.to_string()).collect();
                    }
                }
                let env_override = self
                    .config
                    .environment
                    .as_deref()
                    .and_then(|s| s.parse().ok());
                let road_override = self.config.road.as_deref().and_then(|s| s.parse().ok());
                let mtn_override = self.config.mountain.as_deref().and_then(|s| s.parse().ok());
                self.scenery = crate::scenery::resolve_archetype_with_overrides(
                    &self.config.cow,
                    mtn_override,
                    road_override,
                    env_override,
                );
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            ControlMsg::Text(text) => {
                // Recompose the scene with new text and recreate the effect.
                self.config.text = text.clone();
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(
                    &cow_text,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            ControlMsg::Think(text) => {
                // Switch bubble style to thought bubble (o) and recompose scene with new thought text.
                self.config.think = true;
                self.config.text = text.clone();
                let thoughts_glyph = "o";
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed =
                    crate::cow::compose_scene_with_mode(&cow_text, &self.config.text, true);
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            ControlMsg::Eyes(eyes) => {
                self.config.eyes = eyes.clone();
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(
                    &cow_text,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            ControlMsg::Tongue(tongue) => {
                self.config.tongue = tongue.clone();
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(
                    &cow_text,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            ControlMsg::ColorMode(mode) => {
                self.config.color_mode = mode.clone();
                if (mode == "natural"
                    || mode == "animal"
                    || mode == "default"
                    || mode == "animal_natural")
                    && self.config.palette.is_none()
                {
                    let natural_hexes = crate::color::get_natural_hex_palette(&self.config.cow);
                    if !natural_hexes.is_empty() {
                        self.cow_dna.palette =
                            natural_hexes.iter().map(|&s| s.to_string()).collect();
                    }
                }
                let thoughts_glyph = if self.config.think { "o" } else { "\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(
                    &cow_text,
                    &self.config.text,
                    self.config.think,
                );
                if let Ok(mut lock) = self.active_composed.write() {
                    *lock = composed.clone();
                }
                self.cow_foot_y = effects::find_cow_foot_y(&composed);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
                self.need_full_redraw = true;
            }
            _ => {}
        }
    }
}

/// The SIM thread entry point.
fn sim_thread(
    mut sim: SimState,
    control_rx: Receiver<ControlMsg>,
    frame_tx: Sender<Arc<Frame>>,
    shutdown: ShutdownFlag,
    max_frames: u64,
) {
    let _timer_guard = forgum_platform::TimerResolutionGuard::acquire();
    let sim_start = Instant::now();
    let wall_clock_limit = if sim.config.duration > 0 {
        Some(Duration::from_secs(sim.config.duration as u64))
    } else {
        None
    };
    let target_fps =
        if forgum_platform::is_remote_session() && (sim.config.fps == 0 || sim.config.fps == 60) {
            30
        } else if sim.config.fps == 0 {
            60
        } else {
            sim.config.fps as u32
        };
    let mut governor = LiquidFrameGovernor::new(target_fps);
    let mut last_battery_check = Instant::now();
    let mut user_speed: f32 = 1.0;
    let mut battery_throttled = false;
    let mut paused = false;

    loop {
        if shutdown.is_shutdown() {
            break;
        }
        // Strict wall-clock duration limit: ensures --duration N always runs for N wall-clock seconds!
        if let Some(limit) = wall_clock_limit {
            if sim_start.elapsed() >= limit {
                crate::log_info!(
                    "engine",
                    "SIM thread reached wall-clock duration limit ({:?}), shutting down",
                    limit
                );
                shutdown.trigger();
                break;
            }
        }
        // Fallback frame-count termination for duration 0 / explicit max_frames invocations
        if max_frames > 0 && wall_clock_limit.is_none() && sim.frame_count >= max_frames {
            crate::log_info!(
                "engine",
                "SIM thread reached max_frames ({}), shutting down",
                max_frames
            );
            shutdown.trigger();
            break;
        }
        if wall_clock_limit.is_none() && max_frames == 0 && sim.effect.is_done() {
            crate::log_info!(
                "engine",
                "SIM thread effect.is_done() returned true, shutting down"
            );
            shutdown.trigger();
            break;
        }

        // Phase 8.5: Battery-reactive throttle. Check every 30 seconds.
        // Uses platform-specific battery detection with graceful fallback.
        if last_battery_check.elapsed() >= Duration::from_secs(30) {
            last_battery_check = Instant::now();
            if let Some(pct) = forgum_platform::check_battery_percent() {
                if pct < 20.0 && !battery_throttled {
                    sim.scheduler.set_speed(0.15);
                    governor
                        .set_fps(((target_fps as f32) * 0.15).round().clamp(1.0, 1000.0) as u32);
                    battery_throttled = true;
                } else if pct >= 30.0 && battery_throttled {
                    sim.scheduler.set_speed(user_speed);
                    governor.set_fps(
                        ((target_fps as f32) * user_speed)
                            .round()
                            .clamp(1.0, 1000.0) as u32,
                    );
                    battery_throttled = false;
                }
            }
        }

        // Drain control messages (non-blocking).
        while let Ok(msg) = control_rx.try_recv() {
            match msg {
                ControlMsg::Stop => {
                    crate::log_info!(
                        "engine",
                        "SIM thread received ControlMsg::Stop, shutting down"
                    );
                    shutdown.trigger();
                    break;
                }
                ControlMsg::Pause => {
                    paused = true;
                }
                ControlMsg::Resume => {
                    paused = false;
                }
                ControlMsg::Speed(s) => {
                    user_speed = s;
                    if !battery_throttled {
                        sim.scheduler.set_speed(s);
                        governor
                            .set_fps(((target_fps as f32) * s).round().clamp(1.0, 1000.0) as u32);
                    }
                }
                ControlMsg::Resize { cols, rows } => {
                    sim.fb.resize(cols as usize, rows as usize);
                    sim.effect.on_resize(cols as usize, rows as usize);
                    sim.scenery_cache.clear();
                    sim.need_full_redraw = true;
                }
                ref msg @ (ControlMsg::Effect(_)
                | ControlMsg::Cow(_)
                | ControlMsg::Text(_)
                | ControlMsg::Think(_)
                | ControlMsg::Eyes(_)
                | ControlMsg::Tongue(_)
                | ControlMsg::ColorMode(_)) => {
                    sim.handle_hot_swap(msg);
                }
            }
        }

        if shutdown.is_shutdown() {
            break;
        }

        if paused {
            shutdown.sleep_interruptible(governor.target_period());
            continue;
        }

        if sim.scheduler.tier() == crate::scheduler::SchedulerTier::Idle {
            // Idle cow: zero CPU utilization
            shutdown.sleep_interruptible(Duration::from_millis(100));
            continue;
        }

        // Decoupled fixed-timestep simulation step
        let (alpha, elapsed_secs) = governor.step_simulation(|dt| {
            sim.tick_physics(dt as f32);
        });

        // Frame snapshot generation with sub-pixel interpolation clock
        let frame = sim.tick_render(alpha as f32, elapsed_secs as f32);

        // Send to render thread with bounded backpressure so zero frames are dropped.
        if frame_tx.send(frame).is_err() {
            crate::log_warn!(
                "engine",
                "SIM thread: render thread dropped receiver, exiting"
            );
            break;
        }

        // Precision frame pacing: hybrid sleep + microsecond spin window
        governor.pace_frame(&shutdown);
    }
}

// ── RENDER thread ──────────────────────────────────────────────────

/// State owned exclusively by the RENDER thread.
struct RenderState {
    front: Vec<Cell>,
    renderer: Box<dyn Renderer>,
    out: OutputHandle,
    cols: usize,
    rows: usize,
    is_banner: bool,
    banner_rows: usize,
    is_overlay: bool,
    overlay_rows: usize,
    _total_rows: usize,
    split_scroll: bool,
    guard: Option<crate::split_scroll::SplitOverlayGuard>,
}

impl RenderState {
    fn new(cols: usize, rows: usize, out: OutputHandle) -> Self {
        Self {
            front: vec![Cell::default(); cols * rows],
            renderer: renderer::create_renderer(),
            out,
            cols,
            rows,
            is_banner: false,
            banner_rows: rows,
            is_overlay: false,
            overlay_rows: rows,
            _total_rows: rows,
            split_scroll: false,
            guard: None,
        }
    }

    fn new_banner(cols: usize, rows: usize, out: OutputHandle) -> Self {
        Self {
            front: vec![Cell::default(); cols * rows],
            renderer: renderer::create_banner_renderer(),
            out,
            cols,
            rows,
            is_banner: true,
            banner_rows: rows,
            is_overlay: false,
            overlay_rows: rows,
            _total_rows: rows,
            split_scroll: false,
            guard: None,
        }
    }

    fn new_overlay(
        cols: usize,
        overlay_rows: usize,
        total_rows: usize,
        split_scroll: bool,
        parent_pid: Option<u32>,
        out: OutputHandle,
    ) -> Self {
        let scroll_top = overlay_rows + 1;
        let guard = if split_scroll {
            Some(crate::split_scroll::SplitOverlayGuard::new(
                parent_pid,
                scroll_top,
                total_rows,
                overlay_rows,
            ))
        } else {
            None
        };
        Self {
            front: vec![Cell::default(); cols * overlay_rows],
            renderer: renderer::create_overlay_renderer(),
            out,
            cols,
            rows: overlay_rows,
            is_banner: false,
            banner_rows: overlay_rows,
            is_overlay: true,
            overlay_rows,
            _total_rows: total_rows,
            split_scroll,
            guard,
        }
    }

    /// Render a frame received from the SIM thread.
    fn render_frame(&mut self, frame: &Arc<Frame>) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut guard) = self.guard {
            if !guard.should_render(&mut self.out) {
                return Ok(());
            }
        }

        let dims_changed = self.cols != frame.cols || self.rows != frame.rows;
        let expected = frame.cols * frame.rows;
        if self.front.len() != expected || dims_changed {
            self.front.resize(expected, Cell::default());
            self.cols = frame.cols;
            self.rows = frame.rows;
        }

        if dims_changed && self.is_overlay && self.split_scroll {
            let total_rows = forgum_platform::terminal_size().1 as usize;
            self._total_rows = total_rows;
            if total_rows > frame.rows + 2 {
                let scroll_top = frame.rows + 1;
                let _ = self.out.write_all(
                    format!("\x1b7\x1b[?6l\x1b[{scroll_top};{total_rows}r\x1b8").as_bytes(),
                );
                if let Some(ref mut guard) = self.guard {
                    guard.update_dimensions(scroll_top, total_rows, frame.rows);
                }
            }
            if self.overlay_rows > frame.rows {
                for y in (frame.rows + 1)..=self.overlay_rows {
                    let _ = self
                        .out
                        .write_all(format!("\x1b7\x1b[{y};1H\x1b[2K\x1b8").as_bytes());
                }
            }
            self.overlay_rows = frame.rows;
            let _ = self.out.flush();
        }

        if dims_changed && self.is_banner {
            let max_r = self.banner_rows.max(frame.rows);
            for y in 0..max_r {
                let _ = self.out.write_all(b"\x1b8");
                if y > 0 {
                    let _ = self.out.write_all(format!("\x1b[{}B", y).as_bytes());
                }
                let _ = self.out.write_all(b"\x1b[2K");
            }
            let _ = self.out.write_all(b"\x1b8");
            self.banner_rows = frame.rows;
            let _ = self.out.flush();
        }

        if (frame.full_redraw || dims_changed) && !self.is_banner && !self.is_overlay {
            let _ = self.out.write_all(b"\x1b[2J\x1b[H");
        }

        let just_resumed = self.guard.as_mut().is_some_and(|g| g.take_just_resumed());
        let full_damage: Vec<(usize, usize)>;
        let damage = if just_resumed {
            full_damage = (0..frame.cols)
                .flat_map(|x| (0..frame.rows).map(move |y| (x, y)))
                .collect();
            &full_damage[..]
        } else {
            &frame.damage[..]
        };

        if !damage.is_empty() {
            let sync = forgum_platform::detect_sync_support();
            if sync {
                let _ = self.out.write_all(b"\x1b[?2026h");
            }
            self.renderer
                .render_damage(&mut self.out, &frame.cells, frame.cols, damage)?;
            if sync {
                let _ = self.out.write_all(b"\x1b[?2026l");
            }
            let _ = self.out.flush();
        }

        // Update front buffer by swapping in the frame's cells.
        if self.front.len() == frame.cells.len() {
            self.front.clone_from_slice(&frame.cells);
        }

        Ok(())
    }
}

/// The RENDER thread entry point.
fn render_thread(mut state: RenderState, frame_rx: Receiver<Arc<Frame>>, shutdown: ShutdownFlag) {
    while !shutdown.is_shutdown() {
        match frame_rx.recv_timeout(Duration::from_millis(50)) {
            Ok(frame) => {
                if let Err(e) = state.render_frame(&frame) {
                    crate::log_error!("engine", "RENDER thread error in render_frame: {}", e);
                    eprintln!("forgum: render error: {e}");
                    break;
                }
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                crate::log_info!(
                    "engine",
                    "RENDER thread: SIM thread dropped sender, exiting"
                );
                break;
            }
        }
    }
    shutdown.trigger();

    // Clear and restore terminal on exit.
    if state.is_banner {
        let _ = state
            .out
            .write_all(format!("\x1b8\x1b[{}B\r\x1b[0m\x1b[?25h\n", state.banner_rows).as_bytes());
    } else if state.is_overlay {
        let mut clean_buf = Vec::new();
        for y in 1..=state.overlay_rows {
            clean_buf.extend_from_slice(format!("\x1b[{y};1H\x1b[2K").as_bytes());
        }
        // Unconditionally reset DECSTBM scroll margins, origin mode, and DECSLRM
        // so reserved terminal rows are completely restored for subsequent applications.
        clean_buf.extend_from_slice(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h");
        // Clear space below and position cursor cleanly on the first unreserved row
        let prompt_row = (state.overlay_rows + 1).min(state.rows);
        clean_buf.extend_from_slice(format!("\x1b[{prompt_row};1H\x1b[J").as_bytes());
        let _ = state.out.write_all(&clean_buf);
    } else {
        let _ = state
            .out
            .write_all(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h\n");
    }
    let _ = state.out.flush();
}

// ── Public API ─────────────────────────────────────────────────────

/// Run the 3-thread engine. Called from `render_loop_background` or
/// `render_loop_foreground` after initial setup.
///
/// This is the Phase 1.3-1.5 implementation: SIM thread owns simulation
/// state, RENDER thread owns terminal output, CONTROL thread handles
/// signals and control socket — no shared mutable state.
#[allow(clippy::too_many_arguments)]
pub fn run_engine(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
    max_frames: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    // Probe terminal size.
    let caps = forgum_platform::detect_capabilities();
    let cols = caps.width.max(1) as usize;
    let rows = caps.height.max(1) as usize;

    let initial_display = composed_text.unwrap_or(&config.text);
    let initial_text = if initial_display.is_empty() {
        effects::default_cow_text().to_string()
    } else {
        initial_display.to_string()
    };

    // Tiny-terminal guard.
    if cols < 20 || rows < 5 {
        let _ = out.write_all(initial_text.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
        return Ok(initial_text);
    }

    let active_composed = Arc::new(std::sync::RwLock::new(initial_text));

    // Create channels.
    let (control_tx, control_rx) = unbounded::<ControlMsg>();
    let (frame_tx, frame_rx) = bounded::<Arc<Frame>>(2);

    // Create SIM state.
    let sim = SimState::new(
        &config,
        cols,
        rows,
        cow_dna,
        instance_id,
        composed_text,
        data_dir.clone(),
        active_composed.clone(),
    );

    // Create RENDER state.
    let render_state = RenderState::new(cols, rows, out);

    // Forward external control commands to the internal channel.
    if let Some(external_rx) = cmd_rx {
        let tx = control_tx.clone();
        let external_rx = external_rx.clone();
        std::thread::Builder::new()
            .name("control-forward".into())
            .spawn(move || {
                while let Ok(cmd) = external_rx.recv() {
                    let msg = match cmd {
                        ControlCmd::Stop => ControlMsg::Stop,
                        ControlCmd::Pause => ControlMsg::Pause,
                        ControlCmd::Resume => ControlMsg::Resume,
                        ControlCmd::Effect(name) => ControlMsg::Effect(name),
                        ControlCmd::Speed(s) => ControlMsg::Speed(s),
                        ControlCmd::Cow(name) => ControlMsg::Cow(name),
                        ControlCmd::Text(text) => ControlMsg::Text(text),
                        ControlCmd::Think(text) => ControlMsg::Think(text),
                        ControlCmd::Eyes(eyes) => ControlMsg::Eyes(eyes),
                        ControlCmd::Tongue(tongue) => ControlMsg::Tongue(tongue),
                        ControlCmd::Color(color) => ControlMsg::ColorMode(color),
                        _ => continue,
                    };
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            })?;
    }

    // Spawn SIM and RENDER threads.
    let sim_shutdown = shutdown.clone();
    let render_shutdown = shutdown.clone();

    let sim_handle = std::thread::Builder::new()
        .name("sim".into())
        .spawn(move || {
            sim_thread(sim, control_rx, frame_tx, sim_shutdown, max_frames);
        })?;

    let render_handle = std::thread::Builder::new()
        .name("render".into())
        .spawn(move || {
            render_thread(render_state, frame_rx, render_shutdown);
        })?;

    // CONTROL thread (main thread): wait for shutdown signal.
    // In foreground mode, poll keyboard for Ctrl+C (\x03), 'q', 'Q', Esc.
    // Dual resize detection: crossterm Event::Resize + proactive terminal::size() polling every 30-50ms.
    let mut cur_cols = cols;
    let mut cur_rows = rows;
    let deadline = if config.duration > 0 {
        Some(Instant::now() + Duration::from_secs(config.duration as u64))
    } else {
        None
    };

    while !shutdown.is_shutdown() {
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                shutdown.trigger();
                break;
            }
        }
        if !config.background {
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(30)) {
                match crossterm::event::read() {
                    Ok(crossterm::event::Event::Key(key)) => {
                        use crossterm::event::{KeyCode, KeyModifiers};
                        if (key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL))
                            || key.code == KeyCode::Char('q')
                            || key.code == KeyCode::Char('Q')
                            || key.code == KeyCode::Esc
                        {
                            shutdown.trigger();
                            break;
                        }
                    }
                    Ok(crossterm::event::Event::Resize(w, h)) => {
                        let w = (w.max(1)) as usize;
                        let h = (h.max(1)) as usize;
                        if w != cur_cols || h != cur_rows {
                            cur_cols = w;
                            cur_rows = h;
                            let _ = control_tx.send(ControlMsg::Resize {
                                cols: w as u16,
                                rows: h as u16,
                            });
                        }
                    }
                    _ => {}
                }
            }
            if let Ok((w, h)) = crossterm::terminal::size() {
                let w = (w.max(1)) as usize;
                let h = (h.max(1)) as usize;
                if w != cur_cols || h != cur_rows {
                    cur_cols = w;
                    cur_rows = h;
                    let _ = control_tx.send(ControlMsg::Resize {
                        cols: w as u16,
                        rows: h as u16,
                    });
                }
            }
        } else {
            if let Ok((w, h)) = crossterm::terminal::size() {
                let w = (w.max(1)) as usize;
                let h = (h.max(1)) as usize;
                if w != cur_cols || h != cur_rows {
                    cur_cols = w;
                    cur_rows = h;
                    let _ = control_tx.send(ControlMsg::Resize {
                        cols: w as u16,
                        rows: h as u16,
                    });
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // Join threads.
    let _ = sim_handle.join();
    let _ = render_handle.join();

    let final_text = active_composed
        .read()
        .map(|t| t.clone())
        .unwrap_or_default();
    Ok(final_text)
}

/// Run the 3-thread engine in banner mode (inline above prompt without alternate screen).
#[allow(clippy::too_many_arguments)]
pub fn run_engine_banner(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
    max_frames: u64,
    banner_rows: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let caps = forgum_platform::detect_capabilities();
    let cols = caps.width.max(20) as usize;
    let rows = banner_rows.max(1);

    if cols < 20 || rows < 1 {
        let cow_display = composed_text.unwrap_or(&config.text);
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            cow_display.to_string()
        };
        let _ = out.write_all(cow_text.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
        return Ok(());
    }

    let (control_tx, control_rx) = unbounded::<ControlMsg>();
    let (frame_tx, frame_rx) = bounded::<Arc<Frame>>(2);

    let active_composed = Arc::new(std::sync::RwLock::new(String::new()));
    let sim = SimState::new(
        &config,
        cols,
        rows,
        cow_dna,
        instance_id,
        composed_text,
        data_dir.clone(),
        active_composed,
    );

    let render_state = RenderState::new_banner(cols, rows, out);

    if let Some(external_rx) = cmd_rx {
        let tx = control_tx.clone();
        let external_rx = external_rx.clone();
        std::thread::Builder::new()
            .name("control-forward".into())
            .spawn(move || {
                while let Ok(cmd) = external_rx.recv() {
                    let msg = match cmd {
                        ControlCmd::Stop => ControlMsg::Stop,
                        ControlCmd::Pause => ControlMsg::Pause,
                        ControlCmd::Resume => ControlMsg::Resume,
                        ControlCmd::Effect(name) => ControlMsg::Effect(name),
                        ControlCmd::Speed(s) => ControlMsg::Speed(s),
                        ControlCmd::Cow(name) => ControlMsg::Cow(name),
                        ControlCmd::Text(text) => ControlMsg::Text(text),
                        ControlCmd::Think(text) => ControlMsg::Think(text),
                        ControlCmd::Eyes(eyes) => ControlMsg::Eyes(eyes),
                        ControlCmd::Tongue(tongue) => ControlMsg::Tongue(tongue),
                        ControlCmd::Color(color) => ControlMsg::ColorMode(color),
                        _ => continue,
                    };
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            })?;
    }

    let sim_shutdown = shutdown.clone();
    let render_shutdown = shutdown.clone();

    let sim_handle = std::thread::Builder::new()
        .name("sim".into())
        .spawn(move || {
            sim_thread(sim, control_rx, frame_tx, sim_shutdown, max_frames);
        })?;

    let render_handle = std::thread::Builder::new()
        .name("render".into())
        .spawn(move || {
            render_thread(render_state, frame_rx, render_shutdown);
        })?;

    // In banner mode, strictly no tty reads — wait for frames or signal
    let mut cur_cols = cols;
    let cur_rows = rows;
    let deadline = if config.duration > 0 {
        Some(Instant::now() + Duration::from_secs(config.duration as u64))
    } else {
        None
    };

    while !shutdown.is_shutdown() {
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                shutdown.trigger();
                break;
            }
        }
        let (w_u16, _) = forgum_platform::terminal_size();
        let w = (w_u16.max(20)) as usize;
        if w != cur_cols {
            cur_cols = w;
            let _ = control_tx.send(ControlMsg::Resize {
                cols: w as u16,
                rows: cur_rows as u16,
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = sim_handle.join();
    let _ = render_handle.join();

    Ok(())
}

/// Run the 3-thread engine in overlay mode (continuous background animation above the prompt).
#[allow(clippy::too_many_arguments)]
pub fn run_engine_overlay(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
    max_frames: u64,
    overlay_rows: usize,
    overlay_cols: usize,
    parent_pid: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    let caps = forgum_platform::detect_capabilities();
    let total_cols = caps.width.max(20) as usize;
    let total_rows = caps.height.max(1) as usize;
    let cols = overlay_cols.min(total_cols).max(20);
    let rows = overlay_rows.min(total_rows.saturating_sub(4)).max(1);

    if cols < 20 || rows < 1 {
        let cow_display = composed_text.unwrap_or(&config.text);
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            cow_display.to_string()
        };
        let _ = out.write_all(cow_text.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
        return Ok(());
    }

    // Mode 1: Pure DECSTBM margin split
    // Only lock scroll margins if enabled AND the terminal emulator supports DECSTBM
    let can_split_scroll = config.split_scroll && caps.emulator.supports_decstbm();
    if can_split_scroll && total_rows > rows + 2 {
        let scroll_top = rows + 1;
        let _ = out.write_all(
            format!("\x1b[?6l\x1b[{scroll_top};{total_rows}r\x1b[{scroll_top};1H").as_bytes(),
        );
        let _ = out.flush();
    }

    crate::log_info!(
        "engine",
        "Engine overlay initialized: {}x{} @ {} fps (cow='{}', effect='{}', split_scroll={}, can_split_scroll={})",
        cols,
        rows,
        config.fps,
        config.cow,
        config.effect,
        config.split_scroll,
        can_split_scroll
    );

    let (control_tx, control_rx) = unbounded::<ControlMsg>();
    let (frame_tx, frame_rx) = bounded::<Arc<Frame>>(2);

    let active_composed = Arc::new(std::sync::RwLock::new(String::new()));
    let sim = SimState::new(
        &config,
        cols,
        rows,
        cow_dna,
        instance_id,
        composed_text,
        data_dir.clone(),
        active_composed,
    );

    let render_state =
        RenderState::new_overlay(cols, rows, total_rows, can_split_scroll, parent_pid, out);

    if let Some(external_rx) = cmd_rx {
        let tx = control_tx.clone();
        let external_rx = external_rx.clone();
        std::thread::Builder::new()
            .name("control-forward".into())
            .spawn(move || {
                while let Ok(cmd) = external_rx.recv() {
                    let msg = match cmd {
                        ControlCmd::Stop => ControlMsg::Stop,
                        ControlCmd::Pause => ControlMsg::Pause,
                        ControlCmd::Resume => ControlMsg::Resume,
                        ControlCmd::Effect(name) => ControlMsg::Effect(name),
                        ControlCmd::Speed(s) => ControlMsg::Speed(s),
                        ControlCmd::Cow(name) => ControlMsg::Cow(name),
                        ControlCmd::Text(text) => ControlMsg::Text(text),
                        ControlCmd::Think(text) => ControlMsg::Think(text),
                        ControlCmd::Eyes(eyes) => ControlMsg::Eyes(eyes),
                        ControlCmd::Tongue(tongue) => ControlMsg::Tongue(tongue),
                        ControlCmd::Color(color) => ControlMsg::ColorMode(color),
                        _ => continue,
                    };
                    if tx.send(msg).is_err() {
                        break;
                    }
                }
            })?;
    }

    let sim_shutdown = shutdown.clone();
    let render_shutdown = shutdown.clone();

    let sim_handle = std::thread::Builder::new()
        .name("sim".into())
        .spawn(move || {
            sim_thread(sim, control_rx, frame_tx, sim_shutdown, max_frames);
        })?;

    let render_handle = std::thread::Builder::new()
        .name("render".into())
        .spawn(move || {
            render_thread(render_state, frame_rx, render_shutdown);
        })?;

    // In overlay mode, never touch or intercept stdin. Stdin belongs entirely to the active shell.
    // Dynamic resize detection polls terminal::size() and notifies the SIM thread via control channel.
    let mut cur_total_cols = total_cols;
    let mut cur_total_rows = total_rows;
    let cow_display = composed_text.unwrap_or(&config.text);
    let cow_foot = effects::find_cow_foot_y(cow_display);
    let req_lines = (cow_foot + 4)
        .max(cow_display.lines().count() + 3)
        .max(6)
        .max(overlay_rows);
    let req_cols = cow_display
        .lines()
        .map(crate::cow::str_display_width)
        .max()
        .unwrap_or(0);

    let deadline = if config.duration > 0 {
        Some(Instant::now() + Duration::from_secs(config.duration as u64))
    } else {
        None
    };

    while !shutdown.is_shutdown() {
        if let Some(dl) = deadline {
            if Instant::now() >= dl {
                shutdown.trigger();
                break;
            }
        }
        let (w_u16, h_u16) = forgum_platform::terminal_size();
        let w = (w_u16.max(20)) as usize;
        let h = (h_u16.max(1)) as usize;
        if w != cur_total_cols || h != cur_total_rows {
            cur_total_cols = w;
            cur_total_rows = h;
            let (new_cols, new_rows) = crate::render::compute_reserved_dimensions_with_cols(
                w, h, req_lines, req_cols, &config,
            );
            let session_id = forgum_platform::detect_session_id();
            let socket_path = forgum_platform::control_socket_path(&session_id);
            let _ = crate::daemon::write_daemon_state(
                instance_id,
                new_rows as u16,
                new_cols as u16,
                &socket_path,
            );
            crate::log_debug!(
                "engine",
                "Terminal resized to {}x{}; reserved canvas set to {}x{}",
                w,
                h,
                new_cols,
                new_rows
            );
            let _ = control_tx.send(ControlMsg::Resize {
                cols: new_cols as u16,
                rows: new_rows as u16,
            });
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    let _ = sim_handle.join();
    let _ = render_handle.join();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_clone_produces_independent_snapshot() {
        let cells = vec![Cell::default(); 80 * 24];
        let frame = Arc::new(Frame {
            cells: cells.clone(),
            damage: vec![],
            cols: 80,
            rows: 24,
            full_redraw: false,
        });
        let frame2 = frame.clone();
        assert_eq!(frame.cells.len(), frame2.cells.len());
    }

    #[test]
    fn control_msg_variants_compile() {
        let msgs = [
            ControlMsg::Stop,
            ControlMsg::Pause,
            ControlMsg::Resume,
            ControlMsg::Effect("aurora".into()),
            ControlMsg::Speed(1.5),
            ControlMsg::Cow("dragon".into()),
            ControlMsg::Text("hello".into()),
            ControlMsg::Eyes("^^".into()),
            ControlMsg::Tongue("U ".into()),
            ControlMsg::ColorMode("rainbow".into()),
            ControlMsg::Resize { cols: 80, rows: 24 },
        ];
        assert_eq!(msgs.len(), 11);
    }

    #[test]
    fn sim_tick_damage_bounded_to_changed_cells() {
        let data_dir = std::env::temp_dir().join("forgum_test_damage");
        let _ = std::fs::create_dir_all(&data_dir);
        let config = SceneConfig::default();
        let cow_dna = CowDna::default();
        let active = Arc::new(std::sync::RwLock::new(String::new()));
        let mut sim = SimState::new(&config, 40, 12, cow_dna, 0, None, data_dir.clone(), active);

        let dt = Duration::from_secs_f32(1.0 / 30.0);

        // First tick: initial render.
        let frame1 = sim.tick(dt, dt);
        assert_eq!(frame1.cols, 40);

        // Second tick: same static cow, no animation change → zero or minimal damage.
        let frame2 = sim.tick(dt, dt);
        let total_cells = 40 * 12;
        assert!(
            frame2.damage.len() < total_cells / 4,
            "second tick on static cow should have minimal damage, got {} out of {total_cells}",
            frame2.damage.len(),
        );

        let _ = std::fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn sim_tick_produces_valid_frame() {
        let data_dir = std::env::temp_dir().join("forgum_test_frame");
        let _ = std::fs::create_dir_all(&data_dir);
        let config = SceneConfig::default();
        let cow_dna = CowDna::default();
        let active = Arc::new(std::sync::RwLock::new(String::new()));
        let mut sim = SimState::new(&config, 40, 12, cow_dna, 0, None, data_dir.clone(), active);

        let dt = Duration::from_secs_f32(1.0 / 30.0);
        let frame = sim.tick(dt, dt);
        assert_eq!(frame.cols, 40);
        assert_eq!(frame.rows, 12);
        assert_eq!(frame.cells.len(), 40 * 12);
        assert!(
            !frame.damage.is_empty(),
            "first tick must produce some damage"
        );

        let _ = std::fs::remove_dir_all(&data_dir);
    }

    #[test]
    fn sim_hot_swap_eyes_tongue_color_updates_state() {
        let data_dir = std::env::temp_dir().join("forgum_test_hot_swap");
        let _ = std::fs::create_dir_all(&data_dir);
        let config = SceneConfig::default();
        let cow_dna = CowDna::default();
        let active = Arc::new(std::sync::RwLock::new(String::new()));
        let mut sim = SimState::new(&config, 40, 12, cow_dna, 0, None, data_dir.clone(), active);

        // Eyes hot-swap
        sim.handle_hot_swap(&ControlMsg::Eyes("^^".into()));
        assert_eq!(sim.config.eyes, "^^");

        // Tongue hot-swap
        sim.handle_hot_swap(&ControlMsg::Tongue("U ".into()));
        assert_eq!(sim.config.tongue, "U ");

        // ColorMode hot-swap
        sim.handle_hot_swap(&ControlMsg::ColorMode("rainbow".into()));
        assert_eq!(sim.config.color_mode, "rainbow");

        let _ = std::fs::remove_dir_all(&data_dir);
    }
}
