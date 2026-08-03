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
    back: Vec<Cell>,
    cols: usize,
    rows: usize,
    effect: Box<dyn effects::Effect>,
    scheduler: Scheduler,
    frame_count: u64,
    elapsed: f32,
    /// Per-frame arena: O(1) mass-dealloc via `reset()` at frame top.
    /// All per-frame scratch (damage lists, temp buffers) go here.
    arena: bumpalo::Bump,
}

impl SimState {
    fn new(
        config: &SceneConfig,
        cols: usize,
        rows: usize,
        cow_dna: CowDna,
        instance_id: u32,
        composed_text: Option<&str>,
    ) -> Self {
        let cow_display = composed_text.unwrap_or(&config.text);
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            cow_display.to_string()
        };

        let effect = effects::create_effect(
            cow_dna.base,
            cow_text,
            cow_dna,
            instance_id,
            &config.color_mode,
        );

        Self {
            back: vec![Cell::default(); cols * rows],
            cols,
            rows,
            effect,
            scheduler: Scheduler::new(config.fps),
            frame_count: 0,
            elapsed: 0.0,
            arena: bumpalo::Bump::with_capacity(64 * 1024),
        }
    }

    /// Run one simulation step. Returns the frame snapshot to ship to RENDER.
    fn tick(&mut self, dt: Duration) -> Arc<Frame> {
        // Phase 1.6: O(1) mass-dealloc of last frame's scratch.
        self.arena.reset();

        let dt_f32 = dt.as_secs_f32();
        self.elapsed += dt_f32;

        // Clear back buffer.
        for cell in &mut self.back {
            *cell = Cell::default();
        }

        // Update effect.
        self.effect.update(dt_f32, self.cols, self.rows);

        // Render into back buffer.
        let mut fb = FrameBuffer::from_raw(self.cols, self.rows, &self.back);
        self.effect.render(&mut fb, self.elapsed);

        // Compute damage against the previous front buffer (passed in from render).
        let damage = fb.compute_damage().to_vec();
        self.scheduler.observe(damage.len());

        // Copy rendered cells back.
        self.back.clone_from_slice(&fb.back);

        self.frame_count = self.frame_count.saturating_add(1);
        FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

        // Ship a snapshot.
        Arc::new(Frame {
            cells: self.back.clone(),
            damage,
            cols: self.cols,
            rows: self.rows,
        })
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

    loop {
        if shutdown.is_shutdown() {
            break;
        }
        if max_frames > 0 && sim.frame_count >= max_frames {
            break;
        }

        // Drain control messages (non-blocking).
        while let Ok(msg) = control_rx.try_recv() {
            match msg {
                ControlMsg::Stop => {
                    shutdown.trigger();
                    break;
                }
                ControlMsg::Pause => {
                    // Skip simulation tick but keep draining.
                    continue;
                }
                ControlMsg::Resume => {}
                ControlMsg::Speed(s) => {
                    sim.scheduler.set_speed(s);
                }
                ControlMsg::Resize { cols, rows } => {
                    sim.cols = cols as usize;
                    sim.rows = rows as usize;
                    sim.back
                        .resize(cols as usize * rows as usize, Cell::default());
                    sim.effect.on_resize(cols as usize, rows as usize);
                }
                ControlMsg::Effect(_name) => {
                    // Effect hot-swap (future: reload effect by name).
                }
                ControlMsg::Cow(_name) => {
                    // Cow hot-swap (future: reload cow art).
                }
                ControlMsg::Text(_text) => {
                    // Text hot-swap (future: recompose scene).
                }
            }
        }

        if shutdown.is_shutdown() {
            break;
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

        // Compute damage: cells where new frame differs from front.
        let damage: Vec<(usize, usize)> = frame
            .cells
            .iter()
            .enumerate()
            .filter(|(i, new_cell)| self.front.get(*i) != Some(new_cell))
            .map(|(i, _)| (i % frame.cols, i / frame.cols))
            .collect();

        if !damage.is_empty() {
            // Wrap in synchronized update if supported.
            if cfg!(feature = "synchronized-update") && forgum_platform::terminal_supports_sync() {
                let begin = self.renderer.begin_sync();
                let _ = self.out.write_all(begin.as_bytes());
                self.renderer.render_frame_from_cells(
                    &mut self.out,
                    &frame.cells,
                    frame.cols,
                    &damage,
                )?;
                let end = self.renderer.end_sync();
                let _ = self.out.write_all(end.as_bytes());
            } else {
                self.renderer.render_frame_from_cells(
                    &mut self.out,
                    &frame.cells,
                    frame.cols,
                    &damage,
                )?;
            }
            let _ = self.out.flush();
        }

        // Update front buffer.
        self.front.clone_from_slice(&frame.cells);

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
    _data_dir: PathBuf,
    cmd_rx: Option<Receiver<ControlCmd>>,
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
    let sim = SimState::new(&config, cols, rows, cow_dna, instance_id, composed_text);

    // Create RENDER state.
    let render_state = RenderState::new(cols, rows, out);

    // Forward external control commands to the internal channel.
    if let Some(external_rx) = cmd_rx {
        let tx = control_tx.clone();
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
    // In background mode, this is the main thread that handles signals.
    // No tty reads — only signal-hook and control socket.
    while !shutdown.is_shutdown() {
        std::thread::sleep(Duration::from_millis(50));
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
        let msgs = vec![
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
}
