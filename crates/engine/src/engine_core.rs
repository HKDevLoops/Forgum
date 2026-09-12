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

        Self {
            fb: FrameBuffer::new(cols, rows),
            effect,
            scheduler: Scheduler::new(config.fps),
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
        }
    }

    /// Run one simulation step. Returns the frame snapshot to ship to RENDER.
    fn tick(&mut self, dt: Duration) -> Arc<Frame> {
        // Phase 1.6: O(1) mass-dealloc of last frame's scratch.
        self.arena.reset();

        let dt_f32 = dt.as_secs_f32();
        self.elapsed += dt_f32;

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
        let _has_scenery = env_style != crate::scenery::EnvironmentStyle::None
            || mtn_style != crate::scenery::MountainStyle::None;
        let road_y = match self.cow_dna.base {
            crate::dna::BaseAnim::Fly
            | crate::dna::BaseAnim::Float
            | crate::dna::BaseAnim::Abduction => {
                // Flying/floating creatures soar in the upper atmosphere above tree canopies
                self.fb.height.saturating_sub(3).max(self.cow_foot_y + 2)
            }
            _ => {
                // Walking and ground creatures walk directly on the road surface
                (self.cow_foot_y + 1).min(self.fb.height.saturating_sub(2))
            }
        };
        let animal_h = self.cow_foot_y.max(1);
        crate::scenery::render_scenery_full(
            &mut self.fb,
            mtn_style,
            road_style,
            env_style,
            road_y,
            self.elapsed,
            Some(&self.config.cow),
            Some(animal_h),
            Some(self.cow_dna.base),
        );

        // Update effect.
        self.effect.update(dt_f32, self.fb.width, self.fb.height);

        // Render into back buffer on top of scenery.
        self.effect.render(&mut self.fb, self.elapsed);

        // Compute damage against the previous front buffer (BEFORE swap clears it).
        // compute_full_damage() includes both newly modified cells and cells vacated/cleared
        // from the previous frame, ensuring AnsiRenderer writes spaces to erase them with zero ghost residue.
        let (damage, full_redraw) = if self.need_full_redraw {
            self.need_full_redraw = false;
            let mut all_cells = Vec::with_capacity(self.fb.width * self.fb.height);
            for y in 0..self.fb.height {
                for x in 0..self.fb.width {
                    all_cells.push((x, y));
                }
            }
            (all_cells, true)
        } else {
            (self.fb.compute_full_damage(), false)
        };
        self.scheduler.observe(damage.len());

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
            damage,
            cols: self.fb.width,
            rows: self.fb.height,
            full_redraw,
        })
    }

    /// Handle hot-swap of effect, cow art, or text from control messages.
    fn handle_hot_swap(&mut self, msg: &ControlMsg) {
        match msg {
            ControlMsg::Effect(name) => {
                // Reload DNA for the new effect if it exists in animations.json,
                // otherwise keep the current DNA but change the base animation.
                let animations = crate::dna::load_animations(&self.data_dir);
                let new_dna = crate::dna::get_dna(&animations, &self.config.cow);
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
                let animations = crate::dna::load_animations(&self.data_dir);
                self.cow_dna = crate::dna::get_dna(&animations, &self.config.cow);
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
                        self.cow_dna.palette = natural_hexes.iter().map(|&s| s.to_string()).collect();
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
                if (mode == "natural" || mode == "animal" || mode == "default" || mode == "animal_natural")
                    && self.config.palette.is_none()
                {
                    let natural_hexes = crate::color::get_natural_hex_palette(&self.config.cow);
                    if !natural_hexes.is_empty() {
                        self.cow_dna.palette = natural_hexes.iter().map(|&s| s.to_string()).collect();
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
    let mut last_frame = Instant::now();
    let mut last_battery_check = Instant::now();
    let mut user_speed: f32 = 1.0;
    let mut battery_throttled = false;
    let mut paused = false;

    loop {
        if shutdown.is_shutdown() {
            break;
        }
        if max_frames > 0 && sim.frame_count >= max_frames {
            crate::log_info!("engine", "SIM thread reached max_frames ({}), shutting down", max_frames);
            shutdown.trigger();
            break;
        }
        if sim.effect.is_done() {
            crate::log_info!("engine", "SIM thread effect.is_done() returned true, shutting down");
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
                    battery_throttled = true;
                } else if pct >= 30.0 && battery_throttled {
                    sim.scheduler.set_speed(user_speed);
                    battery_throttled = false;
                }
            }
        }

        // Drain control messages (non-blocking).
        while let Ok(msg) = control_rx.try_recv() {
            match msg {
                ControlMsg::Stop => {
                    crate::log_info!("engine", "SIM thread received ControlMsg::Stop, shutting down");
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
                    }
                }
                ControlMsg::Resize { cols, rows } => {
                    sim.fb.resize(cols as usize, rows as usize);
                    sim.effect.on_resize(cols as usize, rows as usize);
                    sim.need_full_redraw = true;
                }
                ref msg @ (ControlMsg::Effect(_)
                | ControlMsg::Cow(_)
                | ControlMsg::Text(_)
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
            let period = sim.scheduler.frame_period();
            if !period.is_zero() {
                std::thread::sleep(period);
            }
            continue;
        }

        // Fixed-timestep simulation with elapsed compute compensation.
        let now = Instant::now();
        let raw_dt = now.duration_since(last_frame);
        // Clamp dt to [1ms, 100ms] to prevent physics explosion or freeze on window drag
        let dt = raw_dt.clamp(Duration::from_millis(1), Duration::from_millis(100));
        last_frame = now;

        let frame = sim.tick(dt);

        // Send to render thread (bounded — backpressure if render is slow).
        if frame_tx.send(frame).is_err() {
            crate::log_warn!("engine", "SIM thread: render thread dropped receiver, exiting");
            break;
        }

        // Calibrated sleep: subtract elapsed frame work from target period to eliminate drift
        let period = sim.scheduler.frame_period();
        if !period.is_zero() {
            let work_time = now.elapsed();
            if period > work_time {
                std::thread::sleep(period - work_time);
            }
        }
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
        }
    }

    fn new_overlay(
        cols: usize,
        overlay_rows: usize,
        total_rows: usize,
        split_scroll: bool,
        out: OutputHandle,
    ) -> Self {
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
        }
    }

    /// Render a frame received from the SIM thread.
    fn render_frame(&mut self, frame: &Arc<Frame>) -> Result<(), Box<dyn std::error::Error>> {
        let dims_changed = self.cols != frame.cols || self.rows != frame.rows;
        let expected = frame.cols * frame.rows;
        if self.front.len() != expected || dims_changed {
            self.front.resize(expected, Cell::default());
            self.cols = frame.cols;
            self.rows = frame.rows;
        }

        if dims_changed && self.is_overlay && self.split_scroll {
            let total_rows = crossterm::terminal::size()
                .map(|(_, h)| h as usize)
                .unwrap_or(self._total_rows);
            self._total_rows = total_rows;
            if total_rows > frame.rows + 2 {
                let scroll_top = frame.rows + 1;
                let _ = self
                    .out
                    .write_all(format!("\x1b7\x1b[{scroll_top};{total_rows}r\x1b8").as_bytes());
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

        if (frame.full_redraw || dims_changed) && !self.is_banner && !self.is_overlay {
            let _ = self.out.write_all(b"\x1b[2J\x1b[H");
        }

        let damage = &frame.damage;

        if !damage.is_empty() {
            // Wrap in synchronized update if supported.
            if cfg!(feature = "synchronized-update") && forgum_platform::terminal_supports_sync() {
                let begin = self.renderer.begin_sync();
                let _ = self.out.write_all(begin.as_bytes());
                self.renderer
                    .render_damage(&mut self.out, &frame.cells, frame.cols, damage)?;
                let end = self.renderer.end_sync();
                let _ = self.out.write_all(end.as_bytes());
            } else {
                self.renderer
                    .render_damage(&mut self.out, &frame.cells, frame.cols, damage)?;
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
                crate::log_info!("engine", "RENDER thread: SIM thread dropped sender, exiting");
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
        clean_buf.extend_from_slice(b"\x1b7");
        for y in 1..=state.overlay_rows {
            clean_buf.extend_from_slice(format!("\x1b[{y};1H\x1b[2K").as_bytes());
        }
        if state.split_scroll {
            clean_buf.extend_from_slice(b"\x1b[r");
        }
        clean_buf.extend_from_slice(b"\x1b8\x1b[0m\x1b[?25h");
        let _ = state.out.write_all(&clean_buf);
    } else {
        let _ = state.out.write_all(b"\x1b[0m\x1b[?25h\n");
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

    while !shutdown.is_shutdown() {
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
    while !shutdown.is_shutdown() {
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
) -> Result<(), Box<dyn std::error::Error>> {
    let caps = forgum_platform::detect_capabilities();
    let total_cols = caps.width.max(20) as usize;
    let total_rows = caps.height.max(1) as usize;
    let cols = overlay_cols.min(total_cols).max(20);
    let rows = overlay_rows.min(total_rows.saturating_sub(3)).max(1);

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

    // If split_scroll is enabled, lock scroll margins to rows below the overlay
    if config.split_scroll && total_rows > rows + 2 {
        let scroll_top = rows + 1;
        let _ = out
            .write_all(format!("\x1b[{scroll_top};{total_rows}r\x1b[{scroll_top};1H").as_bytes());
        let _ = out.flush();
    }

    crate::log_info!(
        "engine",
        "Engine overlay initialized: {}x{} @ {} fps (cow='{}', effect='{}', split_scroll={})",
        cols,
        rows,
        config.fps,
        config.cow,
        config.effect,
        config.split_scroll
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

    let render_state = RenderState::new_overlay(cols, rows, total_rows, config.split_scroll, out);

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
    let line_count = overlay_rows;

    while !shutdown.is_shutdown() {
        if let Ok((w, h)) = crossterm::terminal::size() {
            let w = (w.max(20)) as usize;
            let h = (h.max(1)) as usize;
            if w != cur_total_cols || h != cur_total_rows {
                cur_total_cols = w;
                cur_total_rows = h;
                let (new_cols, new_rows) =
                    crate::render::compute_reserved_dimensions(w, h, line_count, &config);
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
        let frame1 = sim.tick(dt);
        assert_eq!(frame1.cols, 40);

        // Second tick: same static cow, no animation change → zero or minimal damage.
        let frame2 = sim.tick(dt);
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

        let frame = sim.tick(Duration::from_secs_f32(1.0 / 30.0));
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
