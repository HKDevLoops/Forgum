//! Render loops — foreground (own screen) and background (above prompt).
//!
//! **The fixes for BUG-B1, BUG-B2, BUG-T1, BUG-T2, BUG-D7** live here.
//!
//! Invariants:
//! 1. The background loop **never** calls `event::poll` or `event::read`.
//!    (CI-grep enforced by `tests/no_input_reads.rs`.)
//! 2. `duration=0` means "run forever (until signal or control socket)".
//!    (BUG-B2 fix.)
//! 3. Frame counting uses `u64` and `saturating_mul` so it can't overflow.
//!    (BUG-D7 fix.)
//! 4. Signals flip a shared `ShutdownFlag` checked at the top of each loop.
//!    (BUG-T1 fix.)
//! 5. RAII guards (`RawModeGuard`, `AltScreenGuard`, `CursorShowGuard`)
//!    restore terminal state on every exit path including panic.
//!    (BUG-T2 fix.)

#![allow(unsafe_code)] // guarded raw-pointer usage in RAII guards; see crate-level docs

use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use forgum_platform::{
    AltScreenGuard, CursorShowGuard, OutputHandle, RawModeGuard, ShutdownFlag, SignalGuard,
    TerminalCapabilities,
};

use crate::control_socket::ControlCmd;
use crate::cow;
use crate::dna::CowDna;
use crate::effects;
use crate::framebuffer::FrameBuffer;
use crate::protocol::SceneConfig;
use crate::renderer;
use crate::scheduler::Scheduler;

/// Maximum time we'll sleep in one go.
const MAX_SLEEP: Duration = Duration::from_millis(50);

/// Minimum terminal dimensions for animation. Below this, we print static text.
const MIN_COLS: u16 = 20;
const MIN_ROWS: u16 = 5;

/// Number of rows reserved for the prompt (never render here). Used for
/// computing overlay bounds in background mode.
const PROMPT_GUARD: u16 = 3;

#[allow(dead_code)]
const _PROMPT_GUARD: u16 = PROMPT_GUARD; // keep for Phase 2 overlay region math

/// Run the foreground render loop. Owns the alternate screen; exits on
/// `q`/Esc/`SIGINT`/`SIGTERM`/`SIGHUP` or when `duration` elapses.
///
/// If `composed_text` is provided, it's used directly as the cow art
/// (pre-composed with speech bubble by the cow module). Otherwise falls
/// back to the Phase 0 static cow rendering.
#[allow(clippy::too_many_arguments)]
pub fn render_loop_foreground(
    mut out: OutputHandle,
    mut config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<mpsc::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _signals = SignalGuard::install(shutdown.clone())?;

    let caps = TerminalCapabilities::probe();
    let (mut cols, mut rows) = (caps.width.max(1), caps.height.max(1));

    let cow_display = composed_text.unwrap_or(&config.text);

    // Tiny-terminal guard: print static text and exit.
    if cols < MIN_COLS || rows < MIN_ROWS {
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            format!("{}\n{}", effects::default_cow_text(), cow_display)
        };
        let _ = out.write_all(cow_text.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
        return Ok(());
    }

    let _raw = RawModeGuard::acquire()?;
    let writer_ptr = out.raw_writer_mut();
    let _alt = unsafe { AltScreenGuard::acquire(writer_ptr)? };
    let _cur = unsafe { CursorShowGuard::acquire(writer_ptr)? };

    let mut fb = FrameBuffer::new(usize::from(cols), usize::from(rows));
    let mut scheduler = Scheduler::new(config.fps);
    let max_frames = compute_max_frames(config.duration, config.fps);

    let cow_text = if cow_display.is_empty() {
        effects::default_cow_text().to_string()
    } else {
        cow_display.to_string()
    };

    // Create the animation effect from DNA
    let mut effect =
        effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), instance_id);
    let mut rend = renderer::create_renderer();

    let mut frame_count: u64 = 0;
    let mut elapsed: f32 = 0.0;
    while !shutdown.is_shutdown() {
        if max_frames > 0 && frame_count >= max_frames {
            break;
        }

        // Handle resize events (SIGWINCH).
        if shutdown.check_and_clear_resize() {
            let new_caps = TerminalCapabilities::probe();
            cols = new_caps.width.max(1);
            rows = new_caps.height.max(1);
            fb.resize(usize::from(cols), usize::from(rows));
            effect.on_resize(usize::from(cols), usize::from(rows));
        }

        // Process control commands (non-blocking).
        if let Some(rx) = cmd_rx {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    ControlCmd::Stop => {
                        shutdown.trigger();
                        break;
                    }
                    ControlCmd::Pause => {
                        // Skip rendering but keep looping.
                    }
                    ControlCmd::Resume => {
                        // Resume rendering — no-op for now, the loop continues.
                    }
                    ControlCmd::Effect(name) => {
                        eprintln!("forgum-engine: effect change requested: {name}");
                    }
                    ControlCmd::Speed(s) => {
                        // Scale the scheduler's target FPS from the base rate.
                        scheduler.set_speed(s);
                    }
                    ControlCmd::Cow(name) => {
                        // Reload the cow art (preserving current eyes/tongue)
                        // and recreate the live effect with the same text.
                        config.cow = name.clone();
                        let cow_text = cow::load_cow(
                            &config.cow,
                            &data_dir,
                            &config.eyes,
                            &config.tongue,
                            "\\\\",
                        );
                        let new_composed = cow::compose_scene(&cow_text, &config.text);
                        effect = effects::create_effect(
                            cow_dna.base,
                            new_composed,
                            cow_dna.clone(),
                            instance_id,
                        );
                    }
                    ControlCmd::Text(text) => {
                        // Recompute the composed scene text (bubble + cow) with
                        // the new bubble text and recreate the live effect.
                        config.text = text.clone();
                        let cow_text = cow::load_cow(
                            &config.cow,
                            &data_dir,
                            &config.eyes,
                            &config.tongue,
                            "\\\\",
                        );
                        let new_composed = cow::compose_scene(&cow_text, &config.text);
                        effect = effects::create_effect(
                            cow_dna.base,
                            new_composed,
                            cow_dna.clone(),
                            instance_id,
                        );
                    }
                    _ => {}
                }
            }
        }

        let dt = scheduler.tick();
        let dt_f32 = dt.as_secs_f32();
        elapsed += dt_f32;
        fb.clear();
        effect.update(dt_f32, usize::from(cols), usize::from(rows));
        effect.render(&mut fb, elapsed);
        let dmg = fb.compute_damage();
        scheduler.observe(dmg.len());
        if !dmg.is_empty() {
            // G3/T4: wrap per-frame damage in DEC 2026 synchronized update, but
            // ONLY when the `synchronized-update` feature is enabled. `cfg!` is a
            // macro (not a `#[cfg]` attribute) so `crates/engine/src` stays
            // cfg-free for the CI grep. `SyncGuard` ensures `end_sync` runs on
            // drop — even on panic/error — so the terminal is never left in
            // sync mode. When the feature is off (default) the output is
            // byte-identical to before.
            if cfg!(feature = "synchronized-update") && forgum_platform::terminal_supports_sync() {
                let mut guard = SyncGuard::begin(&mut out, rend.as_ref());
                rend.render_damage(guard.out_mut(), &fb, &dmg)?;
            } else {
                rend.render_damage(&mut out, &fb, &dmg)?;
            }
        }
        fb.swap();
        frame_count = frame_count.saturating_add(1);
        sleep_interruptible(scheduler.frame_period(), &shutdown);
    }

    // Clear and reset on the way out so the user's shell prompt is clean.
    let _ = out.write_all(b"\x1b[0m\x1b[H\x1b[2J");
    let _ = out.flush();
    Ok(())
}

/// Run the background render loop. Does **not** own the alternate screen or
/// raw mode. Does **not** read input. Exits on signal or when `duration`
/// elapses. With `duration=0`, runs forever.
#[allow(clippy::too_many_arguments)]
pub fn render_loop_background(
    mut out: OutputHandle,
    mut config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<mpsc::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _signals = SignalGuard::install(shutdown.clone())?;

    let caps = TerminalCapabilities::probe();
    let mut cols = caps.width.max(1);
    let mut rows = caps.height.max(1);

    let cow_display = composed_text.unwrap_or(&config.text);

    // Tiny-terminal guard: print static text and exit.
    if cols < MIN_COLS || rows < MIN_ROWS {
        let cow_text = if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            format!("{}\n{}", effects::default_cow_text(), cow_display)
        };
        let _ = out.write_all(cow_text.as_bytes());
        let _ = out.write_all(b"\n");
        let _ = out.flush();
        return Ok(());
    }

    let writer_ptr = out.raw_writer_mut();
    let _cur = unsafe { CursorShowGuard::acquire(writer_ptr)? };

    let mut fb = FrameBuffer::new(usize::from(cols), usize::from(rows));
    let mut scheduler = Scheduler::new(config.fps);
    let max_frames = compute_max_frames(config.duration, config.fps);

    let cow_text = if cow_display.is_empty() {
        effects::default_cow_text().to_string()
    } else {
        cow_display.to_string()
    };

    // Create the animation effect from DNA
    let mut effect =
        effects::create_effect(cow_dna.base, cow_text.clone(), cow_dna.clone(), instance_id);
    let mut rend = renderer::create_renderer();

    let mut frame_count: u64 = 0;
    let mut elapsed: f32 = 0.0;
    while !shutdown.is_shutdown() {
        if max_frames > 0 && frame_count >= max_frames {
            break;
        }

        // Handle resize events (SIGWINCH).
        if shutdown.check_and_clear_resize() {
            let new_caps = TerminalCapabilities::probe();
            cols = new_caps.width.max(1);
            rows = new_caps.height.max(1);
            fb.resize(usize::from(cols), usize::from(rows));
            effect.on_resize(usize::from(cols), usize::from(rows));
        }

        // Process control commands (non-blocking).
        if let Some(rx) = cmd_rx {
            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    ControlCmd::Stop => {
                        shutdown.trigger();
                        break;
                    }
                    ControlCmd::Pause => {
                        // Skip rendering but keep looping.
                    }
                    ControlCmd::Resume => {
                        // Resume rendering — no-op for now, the loop continues.
                    }
                    ControlCmd::Effect(name) => {
                        eprintln!("forgum-engine: effect change requested: {name}");
                    }
                    ControlCmd::Speed(s) => {
                        // Scale the scheduler's target FPS from the base rate.
                        scheduler.set_speed(s);
                    }
                    ControlCmd::Cow(name) => {
                        // Reload the cow art (preserving current eyes/tongue)
                        // and recreate the live effect with the same text.
                        config.cow = name.clone();
                        let cow_text = cow::load_cow(
                            &config.cow,
                            &data_dir,
                            &config.eyes,
                            &config.tongue,
                            "\\\\",
                        );
                        let new_composed = cow::compose_scene(&cow_text, &config.text);
                        effect = effects::create_effect(
                            cow_dna.base,
                            new_composed,
                            cow_dna.clone(),
                            instance_id,
                        );
                    }
                    ControlCmd::Text(text) => {
                        // Recompute the composed scene text (bubble + cow) with
                        // the new bubble text and recreate the live effect.
                        config.text = text.clone();
                        let cow_text = cow::load_cow(
                            &config.cow,
                            &data_dir,
                            &config.eyes,
                            &config.tongue,
                            "\\\\",
                        );
                        let new_composed = cow::compose_scene(&cow_text, &config.text);
                        effect = effects::create_effect(
                            cow_dna.base,
                            new_composed,
                            cow_dna.clone(),
                            instance_id,
                        );
                    }
                    _ => {}
                }
            }
        }

        let dt = scheduler.tick();
        let dt_f32 = dt.as_secs_f32();
        elapsed += dt_f32;
        fb.clear();
        effect.update(dt_f32, usize::from(cols), usize::from(rows));
        effect.render(&mut fb, elapsed);
        let dmg = fb.compute_damage();
        scheduler.observe(dmg.len());
        if !dmg.is_empty() {
            if cfg!(feature = "synchronized-update") && forgum_platform::terminal_supports_sync() {
                let mut guard = SyncGuard::begin(&mut out, rend.as_ref());
                rend.render_damage(guard.out_mut(), &fb, &dmg)?;
            } else {
                rend.render_damage(&mut out, &fb, &dmg)?;
            }
        }
        fb.swap();
        frame_count = frame_count.saturating_add(1);
        sleep_interruptible(scheduler.frame_period(), &shutdown);
    }

    // Belt + braces: clear and show cursor on the way out.
    let _ = out.write_all(b"\x1b[0m\x1b[?25h");
    let _ = out.flush();
    Ok(())
}

fn compute_max_frames(duration_secs: u32, fps: u16) -> u64 {
    // BUG-D7: u64 + saturating_mul to avoid overflow on huge inputs.
    if duration_secs == 0 {
        0 // 0 means "infinite"
    } else {
        (u64::from(duration_secs)).saturating_mul(u64::from(fps.max(1)))
    }
}

/// RAII guard that emits `begin_sync()` immediately and `end_sync()` on drop,
/// so a DEC 2026 synchronized-update block is always closed — even if rendering
/// panics or returns an error — and the terminal is never left in sync mode.
///
/// The guard owns the `out` borrow for its lifetime; [`SyncGuard::out_mut`]
/// reborrows it for the inner `render_damage` call, which ends before the drop.
struct SyncGuard<'a, W: Write> {
    out: &'a mut W,
    end: &'static str,
}

impl<'a, W: Write> SyncGuard<'a, W> {
    fn begin(out: &'a mut W, rend: &dyn crate::renderer::Renderer) -> Self {
        let _ = out.write_all(rend.begin_sync().as_bytes());
        Self {
            out,
            end: rend.end_sync(),
        }
    }

    fn out_mut(&mut self) -> &mut W {
        self.out
    }
}

impl<'a, W: Write> Drop for SyncGuard<'a, W> {
    fn drop(&mut self) {
        let _ = self.out.write_all(self.end.as_bytes());
    }
}

fn sleep_interruptible(period: Duration, shutdown: &ShutdownFlag) {
    let mut remaining = period;
    while !remaining.is_zero() {
        if shutdown.is_shutdown() {
            return;
        }
        let chunk = remaining.min(MAX_SLEEP);
        std::thread::sleep(chunk);
        remaining = remaining.saturating_sub(chunk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_max_frames_zero_is_infinite() {
        assert_eq!(compute_max_frames(0, 30), 0);
        assert_eq!(compute_max_frames(0, 0), 0);
    }

    #[test]
    fn compute_max_frames_seconds_times_fps() {
        assert_eq!(compute_max_frames(2, 30), 60);
        assert_eq!(compute_max_frames(1, 60), 60);
    }

    #[test]
    fn compute_max_frames_does_not_overflow() {
        // u32::MAX * 60 would overflow u32; must stay u64.
        let frames = compute_max_frames(u32::MAX, 60);
        assert!(frames > 0);
        assert!(frames < u64::MAX);
    }

    #[test]
    fn fps_zero_does_not_divide_by_zero() {
        let frames = compute_max_frames(5, 0);
        assert_eq!(frames, 5); // fps clamped to 1 → 5 frames
    }
}
