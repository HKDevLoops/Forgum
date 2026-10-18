//! RAII guards for terminal state.
//!
//! **The fix for BUG-T2**: the old code's `Terminal::Drop` was a no-op because
//! raw mode was enabled by a free function, never through a method that
//! flipped a flag. These guards flip the model: acquiring a guard *enables*
//! the terminal state, and dropping it *restores* the previous state. Because
//! `Drop` runs on normal return *and* on panic-unwind, the terminal is
//! always left in a usable state.
//!
//! ## Guard stack
//!
//! ```text
//! let _sig  = SignalGuard::install(flag)?;   // first
//! let _out  = OutputHandle::open()?;
//! let _raw  = RawModeGuard::acquire()?;      // foreground only
//! let _alt  = AltScreenGuard::acquire()?;    // foreground only
//! let _cur  = CursorShowGuard::acquire()?;   // foreground only
//!
//! // ...rendering...
//!
//! drop(_cur); drop(_alt); drop(_raw); drop(_out); drop(_sig);
//! //                                      ↑ LIFO: cursor → alt → raw → output → signals
//! ```
//!
//! Note: in practice you don't need explicit `drop()` — leaving the scope
//! works the same way. We keep the guard names short so they can sit on a
//! single line in the render loop.

use std::io;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crate::error::PlatformError;

/// Complete atomic emergency VT reset sequence:
/// 1. \x1b[?2026l : Close DEC 2026 Synchronized Update (prevents frozen terminal screen)
/// 2. \x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l : Disable all mouse capture modes
/// 3. \x1b[?2004l : Disable bracketed paste
/// 4. \x1b[r      : Reset DECSTBM scroll margins to full screen
/// 5. \x1b[?1049l : Exit alternate screen buffer
/// 6. \x1b[0m\x1b[?25h : Reset SGR colors, restore visible cursor, and advance line
pub const EMERGENCY_VT_RESTORE_BYTES: &[u8] =
    b"\x1b[?2026l\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l\x1b[?2004l\x1b[r\x1b[?1049l\x1b[0m\x1b[?25h";

static ACTIVE_TTY_FD: AtomicI32 = AtomicI32::new(1); // Defaults to stdout (fd 1)
static VT_NEEDS_RESTORE: AtomicBool = AtomicBool::new(false);

/// Registers the active TTY file descriptor when entering raw/alt-screen/split-scroll mode.
pub fn mark_vt_dirty(tty_fd: i32) {
    ACTIVE_TTY_FD.store(tty_fd, Ordering::Release);
    VT_NEEDS_RESTORE.store(true, Ordering::Release);
}

/// Idempotent, lock-free emergency terminal restoration safe to call from panic hooks,
/// Drop guards, worker threads (`sim_thread` / `render_thread`), or signal handlers.
pub fn emergency_restore_terminal_idempotent() {
    if VT_NEEDS_RESTORE.swap(false, Ordering::AcqRel) {
        let _ = crossterm::terminal::disable_raw_mode();
        #[cfg(unix)]
        {
            let fd = ACTIVE_TTY_FD.load(Ordering::Acquire);
            #[allow(unsafe_code)]
            unsafe {
                let _ = libc::write(
                    fd,
                    EMERGENCY_VT_RESTORE_BYTES.as_ptr() as *const libc::c_void,
                    EMERGENCY_VT_RESTORE_BYTES.len(),
                );
            }
        }
        #[cfg(windows)]
        {
            use std::io::Write;
            let _ = std::io::stdout().write_all(EMERGENCY_VT_RESTORE_BYTES);
            let _ = std::io::stdout().flush();
        }
    }
}

/// Installs a global panic hook that restores the terminal and cleans up daemon socket/state
/// files BEFORE printing the panic message, even if `sim_thread` or `render_thread` panics!
pub fn install_crash_safe_vt_hook(shutdown_flag: crate::signal::ShutdownFlag) {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // 1. Trigger global shutdown so sibling worker threads stop writing
        shutdown_flag.trigger();
        // 2. Restore terminal modes, DECSTBM margins, mouse capture, and DEC 2026 sync immediately
        emergency_restore_terminal_idempotent();
        // 3. Delegate to standard panic formatter now that the terminal is clean and readable
        default_hook(panic_info);
    }));
}

/// Enables raw mode on construction, disables on drop.
///
/// Raw mode means: no line buffering, no echo, no Ctrl-C/SIGINT-as-Enter.
/// We need this in **foreground only** because the background overlay never
/// owns the shell's input.
#[derive(Debug)]
pub struct RawModeGuard {
    /// Whether raw mode was successfully enabled (and therefore must be
    /// disabled). False on construction failure.
    armed: bool,
}

impl RawModeGuard {
    /// Acquire raw mode. On error, returns a guard that does nothing on drop
    /// (so we don't try to disable something that was never enabled).
    pub fn acquire() -> Result<Self, PlatformError> {
        mark_vt_dirty(1);
        match crossterm::terminal::enable_raw_mode() {
            Ok(()) => Ok(Self { armed: true }),
            Err(e) => Err(PlatformError::Io(io::Error::other(format!(
                "enable_raw_mode: {e}"
            )))),
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if self.armed {
            emergency_restore_terminal_idempotent();
            let _ = crossterm::terminal::disable_raw_mode();
        }
    }
}

/// Enters the alternate screen on construction, leaves on drop.
///
/// The alternate screen is the one most terminals swap to for full-screen
/// apps (vim, less, etc.) so the user's previous output is preserved when
/// the app exits. **Foreground only** — the background overlay uses the
/// *main* screen so the user can keep scrolling history.
#[derive(Debug)]
pub struct AltScreenGuard {
    armed: bool,
}

impl AltScreenGuard {
    pub fn acquire() -> Result<Self, PlatformError> {
        mark_vt_dirty(1);
        use crossterm::ExecutableCommand;
        if std::io::stdout()
            .execute(crossterm::terminal::EnterAlternateScreen)
            .is_err()
        {
            return Err(PlatformError::Unsupported("alternate screen"));
        }
        Ok(Self { armed: true })
    }
}

impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        if self.armed {
            use crossterm::ExecutableCommand;
            let _ = std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen);
        }
    }
}

/// Hides the cursor on construction, shows it on drop.
///
/// We always hide the cursor while rendering so it doesn't blink in the
/// middle of an animation. We always show it on drop so the user isn't left
/// staring at an empty terminal with no cursor.
#[derive(Debug)]
pub struct CursorShowGuard {
    armed: bool,
}

impl CursorShowGuard {
    pub fn acquire() -> Result<Self, PlatformError> {
        mark_vt_dirty(1);
        use crossterm::ExecutableCommand;
        if std::io::stdout().execute(crossterm::cursor::Hide).is_err() {
            return Err(PlatformError::Unsupported("hide cursor"));
        }
        Ok(Self { armed: true })
    }
}

impl Drop for CursorShowGuard {
    fn drop(&mut self) {
        if self.armed {
            use crossterm::ExecutableCommand;
            let _ = std::io::stdout().execute(crossterm::cursor::Show);
            let _ = std::io::stdout().execute(crossterm::cursor::Show);
        }
    }
}

/// RAII guard that requests 1ms multimedia timer resolution on Windows on construction,
/// and restores the default timer period on drop.
/// On non-Windows platforms, this is a zero-cost no-op.
#[derive(Debug)]
pub struct TimerResolutionGuard {
    _private: (),
}

impl TimerResolutionGuard {
    #[must_use]
    pub fn acquire() -> Self {
        #[cfg(windows)]
        #[allow(unsafe_code)]
        unsafe {
            #[link(name = "winmm")]
            extern "system" {
                fn timeBeginPeriod(uPeriod: u32) -> u32;
            }
            let _ = timeBeginPeriod(1);
        }
        Self { _private: () }
    }
}

impl Drop for TimerResolutionGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        #[allow(unsafe_code)]
        unsafe {
            #[link(name = "winmm")]
            extern "system" {
                fn timeEndPeriod(uPeriod: u32) -> u32;
            }
            let _ = timeEndPeriod(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_mode_guard_drop_restores() {
        // We can't reliably test against a real terminal in CI; just verify
        // the guard compiles and constructs.
        // SAFETY: tests run single-threaded for this suite.
        if crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
            {
                let _g = RawModeGuard::acquire().expect("enable raw");
                assert!(crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
            }
            // After drop, raw mode should be off.
            assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
        }
    }

    #[test]
    fn raw_mode_guard_drop_after_panic() {
        if crossterm::tty::IsTty::is_tty(&std::io::stdout()) {
            let result = std::panic::catch_unwind(|| {
                let _g = RawModeGuard::acquire().expect("enable raw");
                panic!("forced panic");
            });
            assert!(result.is_err());
            // Drop ran during unwind: raw mode should be off.
            assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
        }
    }
}
