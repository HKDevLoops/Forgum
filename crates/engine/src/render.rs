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

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use forgum_platform::{
    AltScreenGuard, CursorShowGuard, OutputHandle, RawModeGuard, ShutdownFlag, SignalGuard,
    TerminalCapabilities,
};

use crate::control_socket::ControlCmd;
use crate::dna::CowDna;
use crate::effects;
use crate::protocol::SceneConfig;

/// Maximum time we'll sleep in one go.
#[allow(dead_code)]
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
    out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _signals = SignalGuard::install(shutdown.clone())?;

    let caps = TerminalCapabilities::probe();
    let (cols, rows) = (caps.width.max(1), caps.height.max(1));

    let cow_display = composed_text.unwrap_or(&config.text);

    // Tiny-terminal or non-tty pipe guard: print static text and exit.
    if cols < MIN_COLS || rows < MIN_ROWS || !crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
        let cow_text = if composed_text.is_some() {
            cow_display.to_string()
        } else if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            format!("{}\n{}", effects::default_cow_text(), cow_display)
        };
        println!("{cow_text}");
        return Ok(());
    }

    let _raw = RawModeGuard::acquire()?;
    let _alt = AltScreenGuard::acquire()?;
    let _cur = CursorShowGuard::acquire()?;

    let max_frames = compute_max_frames(config.duration, config.fps);

    let exit_result = crate::engine_core::run_engine(
        out,
        config,
        shutdown,
        composed_text,
        cow_dna,
        instance_id,
        data_dir,
        cmd_rx,
        max_frames,
    );

    // Drop screen guards to restore the primary terminal screen
    drop(_cur);
    drop(_alt);
    drop(_raw);

    match exit_result {
        Ok(final_text) => {
            // Leave the latest active composed mascot (thought bubble + cow) in the terminal scrollback
            if !final_text.is_empty() {
                println!("\n{final_text}\n");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

/// Run the background render loop. Does **not** own the alternate screen or
/// raw mode. Does **not** read input. Exits on signal or when `duration`
/// elapses. With `duration=0`, runs forever.
#[allow(clippy::too_many_arguments)]
pub fn render_loop_background(
    out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _signals = SignalGuard::install(shutdown.clone())?;

    let caps = TerminalCapabilities::probe();
    let (cols, rows) = (caps.width.max(1), caps.height.max(1));

    let cow_display = composed_text.unwrap_or(&config.text);

    // Tiny-terminal guard: print static text and exit.
    if cols < MIN_COLS || rows < MIN_ROWS {
        let cow_text = if composed_text.is_some() {
            cow_display.to_string()
        } else if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            format!("{}\n{}", effects::default_cow_text(), cow_display)
        };
        println!("{cow_text}");
        return Ok(());
    }

    // Notice: In overlay background mode, we do NOT hide the cursor so the shell prompt
    // cursor remains completely visible and interactive.
    let max_frames = compute_max_frames(config.duration, config.fps);

    let cow_foot = effects::find_cow_foot_y(cow_display);
    let line_count = (cow_foot + 2).max(cow_display.lines().count()).max(1);
    let overlay_rows = if config.split_scroll {
        let split_pane = ((rows as usize) * 38 / 100).clamp(10, 16);
        line_count
            .max(split_pane)
            .min((rows as usize).saturating_sub(5))
            .max(1)
    } else {
        line_count.min((rows as usize).saturating_sub(3)).max(1)
    };

    crate::engine_core::run_engine_overlay(
        out,
        config,
        shutdown,
        composed_text,
        cow_dna,
        instance_id,
        data_dir,
        cmd_rx,
        max_frames,
        overlay_rows,
    )
}

/// Run the banner render loop. Renders inline directly in the terminal scrollback
/// without taking over the screen or clearing history, and positions the cursor
/// cleanly below the finished mascot for the prompt.
#[allow(clippy::too_many_arguments)]
pub fn render_loop_banner(
    mut out: OutputHandle,
    mut config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    data_dir: PathBuf,
    cmd_rx: &Option<crossbeam_channel::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let _signals = SignalGuard::install(shutdown.clone())?;

    let caps = TerminalCapabilities::probe();
    let (cols, rows) = (caps.width.max(1), caps.height.max(1));

    let cow_display = composed_text.unwrap_or(&config.text);

    // Tiny-terminal or non-tty pipe guard: print static text and exit.
    if cols < MIN_COLS || rows < MIN_ROWS || !crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
        let cow_text = if composed_text.is_some() {
            cow_display.to_string()
        } else if cow_display.is_empty() {
            effects::default_cow_text().to_string()
        } else {
            format!("{}\n{}", effects::default_cow_text(), cow_display)
        };
        println!("{cow_text}");
        return Ok(());
    }

    let _cur = CursorShowGuard::acquire()?;

    let cow_foot = effects::find_cow_foot_y(cow_display);
    let line_count = (cow_foot + 2).max(cow_display.lines().count()).max(1);
    let banner_rows = line_count.min((rows as usize).saturating_sub(1)).max(1);

    // In banner mode, default duration to 2s burst if 0
    if config.duration == 0 {
        config.duration = 2;
    }

    let max_frames = compute_max_frames(config.duration, config.fps);

    // Reserve space inline: print banner_rows newlines, then move up and save origin
    for _ in 0..banner_rows {
        let _ = out.write_all(b"\n");
    }
    let _ = out.write_all(format!("\x1b[{}A\r\x1b7", banner_rows).as_bytes());
    let _ = out.flush();

    crate::engine_core::run_engine_banner(
        out,
        config,
        shutdown,
        composed_text,
        cow_dna,
        instance_id,
        data_dir,
        cmd_rx,
        max_frames,
        banner_rows,
    )
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
#[allow(dead_code)]
struct SyncGuard<'a, W: Write> {
    out: &'a mut W,
    end: &'static str,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
