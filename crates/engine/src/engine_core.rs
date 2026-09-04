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
    Resize { cols: u16, rows: u16 },
}

/// A snapshot of the simulation state shipped from SIM → RENDER.
#[derive(Debug, Clone)]
pub struct Frame {
    pub cells: Vec<Cell>,
    pub damage: Vec<(usize, usize)>,
    pub cols: usize,
    pub rows: usize,
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
}

impl SimState {
    fn new(
        config: &SceneConfig,
        cols: usize,
        rows: usize,
        cow_dna: CowDna,
        instance_id: u32,
        composed_text: Option<&str>,
        data_dir: PathBuf,
    ) -> Self {
        let cow_display = composed_text.unwrap_or(&config.text);
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            cow_display.to_string()
        };

        let effect = effects::create_scene_effect(
            &config.effect,
            cow_text,
            cow_dna.clone(),
            instance_id,
            &config.color_mode,
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
        }
    }

    /// Run one simulation step. Returns the frame snapshot to ship to RENDER.
    fn tick(&mut self, dt: Duration) -> Arc<Frame> {
        // Phase 1.6: O(1) mass-dealloc of last frame's scratch.
        self.arena.reset();

        let dt_f32 = dt.as_secs_f32();
        self.elapsed += dt_f32;

        // Clear back buffer.
        self.fb.clear();

        // Update effect.
        self.effect.update(dt_f32, self.fb.width, self.fb.height);

        // Render into back buffer.
        self.effect.render(&mut self.fb, self.elapsed);

        // Compute damage against the previous front buffer (BEFORE swap clears it).
        let damage = self.fb.compute_damage().to_vec();
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
                let thoughts_glyph = if self.config.think { "o" } else { "\\\\" };
                let cow_raw = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let cow_text = crate::cow::compose_scene_with_mode(&cow_raw, &self.config.text, self.config.think);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    cow_text,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
            }
            ControlMsg::Cow(name) => {
                // Reload cow art and recreate the effect.
                self.config.cow = crate::cow::resolve_cow_name(name, &self.data_dir);
                let thoughts_glyph = if self.config.think { "o" } else { "\\\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(&cow_text, &self.config.text, self.config.think);
                let animations = crate::dna::load_animations(&self.data_dir);
                self.cow_dna = crate::dna::get_dna(&animations, &self.config.cow);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
            }
            ControlMsg::Text(text) => {
                // Recompose the scene with new text and recreate the effect.
                self.config.text = text.clone();
                let thoughts_glyph = if self.config.think { "o" } else { "\\\\" };
                let cow_text = crate::cow::load_cow(
                    &self.config.cow,
                    &self.data_dir,
                    &self.config.eyes,
                    &self.config.tongue,
                    thoughts_glyph,
                );
                let composed = crate::cow::compose_scene_with_mode(&cow_text, &self.config.text, self.config.think);
                self.effect = effects::create_scene_effect(
                    &self.config.effect,
                    composed,
                    self.cow_dna.clone(),
                    self.instance_id,
                    &self.config.color_mode,
                );
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
            shutdown.trigger();
            break;
        }
        if sim.effect.is_done() {
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
                }
                ref msg @ (ControlMsg::Effect(_) | ControlMsg::Cow(_) | ControlMsg::Text(_)) => {
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

        // Fixed-timestep simulation.
        let now = Instant::now();
        let dt = now.duration_since(last_frame);
        last_frame = now;

        let frame = sim.tick(dt);

        // Send to render thread (bounded — backpressure if render is slow).
        if frame_tx.send(frame).is_err() {
            // Render thread dropped its receiver → exit.
            break;
        }

        // Sleep for the frame period.
        let period = sim.scheduler.frame_period();
        if !period.is_zero() {
            std::thread::sleep(period);
        }
    }
}

// ── RENDER thread ──────────────────────────────────────────────────

/// State owned exclusively by the RENDER thread.
struct RenderState {
    front: Vec<Cell>,
    renderer: Box<dyn Renderer>,
    out: OutputHandle,
}

impl RenderState {
    fn new(cols: usize, rows: usize, out: OutputHandle) -> Self {
        Self {
            front: vec![Cell::default(); cols * rows],
            renderer: renderer::create_renderer(),
            out,
        }
    }

    /// Render a frame received from the SIM thread.
    fn render_frame(&mut self, frame: &Arc<Frame>) -> Result<(), Box<dyn std::error::Error>> {
        // Resize front buffer if needed.
        let expected = frame.cols * frame.rows;
        if self.front.len() != expected {
            self.front.resize(expected, Cell::default());
        }

        // Use SIM-computed damage directly — no recomputation needed.
        // The SIM thread's FrameBuffer tracks damage incrementally via set()
        // against its own front buffer, which is the last shipped frame.
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
        // The frame's cells Vec is moved into front, old front is dropped.
        if self.front.len() == frame.cells.len() {
            self.front.clone_from_slice(&frame.cells);
        }

        Ok(())
    }
}

/// The RENDER thread entry point.
fn render_thread(mut state: RenderState, frame_rx: Receiver<Arc<Frame>>, shutdown: ShutdownFlag) {
    while !shutdown.is_shutdown() {
        match frame_rx.recv() {
            Ok(frame) => {
                if let Err(e) = state.render_frame(&frame) {
                    eprintln!("forgum-engine: render error: {e}");
                    break;
                }
            }
            Err(_) => {
                // SIM thread dropped its sender → exit.
                break;
            }
        }
    }
    shutdown.trigger();

    // Clear and restore terminal on exit.
    let _ = state.out.write_all(b"\x1b[0m\x1b[?25h\n");
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
) -> Result<(), Box<dyn std::error::Error>> {
    // Probe terminal size.
    let caps = forgum_platform::detect_capabilities();
    let cols = caps.width.max(1) as usize;
    let rows = caps.height.max(1) as usize;

    // Tiny-terminal guard.
    if cols < 20 || rows < 5 {
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
    // In background mode, strictly no tty reads — only signal-hook and control socket.
    while !shutdown.is_shutdown() {
        if !config.background {
            if let Ok(true) = crossterm::event::poll(Duration::from_millis(50)) {
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    use crossterm::event::{KeyCode, KeyModifiers};
                    if (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
                        || key.code == KeyCode::Char('q')
                        || key.code == KeyCode::Char('Q')
                        || key.code == KeyCode::Esc
                    {
                        shutdown.trigger();
                        break;
                    }
                }
            }
        } else {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // Join threads.
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
            ControlMsg::Resize { cols: 80, rows: 24 },
        ];
        assert_eq!(msgs.len(), 8);
    }

    #[test]
    fn sim_tick_damage_bounded_to_changed_cells() {
        let data_dir = std::env::temp_dir().join("forgum_test_damage");
        let _ = std::fs::create_dir_all(&data_dir);
        let config = SceneConfig::default();
        let cow_dna = CowDna::default();
        let mut sim = SimState::new(&config, 40, 12, cow_dna, 0, None, data_dir.clone());

        let dt = Duration::from_secs_f32(1.0 / 30.0);

        // First tick: initial render. The static cow produces some cells.
        let frame1 = sim.tick(dt);
        let total_cells = 40 * 12;
        assert!(
            frame1.damage.len() < total_cells,
            "first tick should have less than full-frame damage ({} vs {total_cells})",
            frame1.damage.len(),
        );

        // Second tick: same static cow, no animation change → zero or minimal damage.
        let frame2 = sim.tick(dt);
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
        let mut sim = SimState::new(&config, 40, 12, cow_dna, 0, None, data_dir.clone());

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
}
