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
use std::time::Duration;

use forgum_platform::{
    AltScreenGuard, CursorShowGuard, OutputHandle, RawModeGuard, ShutdownFlag, SignalGuard,
    TerminalCapabilities,
};

use crate::effects;
use crate::framebuffer::FrameBuffer;
use crate::protocol::SceneConfig;
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
pub fn render_loop_foreground(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
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

    let mut frame_count: u64 = 0;
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
        }

        let _dt = scheduler.tick();
        fb.clear();
        effects::render_static_cow(&mut fb, &cow_text);
        let dmg = fb.compute_damage();
        scheduler.observe(dmg.len());
        if !dmg.is_empty() {
            render_damage(&mut out, &fb, &dmg)?;
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
pub fn render_loop_background(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
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

    let mut frame_count: u64 = 0;
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
        }

        let _dt = scheduler.tick();
        fb.clear();
        effects::render_static_cow(&mut fb, &cow_text);
        let dmg = fb.compute_damage();
        scheduler.observe(dmg.len());
        if !dmg.is_empty() {
            render_damage(&mut out, &fb, &dmg)?;
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

fn render_damage(
    out: &mut OutputHandle,
    fb: &FrameBuffer,
    damage: &std::collections::HashSet<(usize, usize)>,
) -> std::io::Result<()> {
    if damage.is_empty() {
        return Ok(());
    }
    let mut buf = Vec::with_capacity(damage.len() * 4);
    for &(x, y) in damage {
        let cell = fb.get(x, y);
        // Move to (x, y), write the char.
        buf.extend_from_slice(format!("\x1b[{};{}H", y + 1, x + 1).as_bytes());
        let mut ch_buf = [0u8; 4];
        let s = cell.ch.encode_utf8(&mut ch_buf);
        buf.extend_from_slice(s.as_bytes());
    }
    out.write_all(&buf)
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
