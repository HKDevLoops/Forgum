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
//! ```ignore
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

use crate::error::PlatformError;

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
            // We swallow the error here because Drop can't return one; the
            // best we can do is ensure the OS gets the disable request.
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
#[allow(unsafe_code)]
#[derive(Debug)]
pub struct AltScreenGuard {
    // We hold a *mut pointer to the writer rather than owning it, so the
    // caller can keep using `out` after constructing the guard. The caller
    // must guarantee the writer outlives the guard.
    writer: *mut (dyn std::io::Write + Send),
    armed: bool,
}

// Note: the `*mut dyn Write` field makes these types automatically
// !Send and !Sync (raw pointers aren't Send/Sync). That matches the
// render loop's single-threaded usage.

#[allow(unsafe_code)]
impl AltScreenGuard {
    /// # Safety
    /// `writer` must remain valid (live and exclusively borrowed) for as
    /// long as this guard exists. Callers should hold a `&mut` to the
    /// writer and not use it while the guard is alive.
    pub unsafe fn acquire(writer: *mut (dyn std::io::Write + Send)) -> Result<Self, PlatformError> {
        let mut_ref = unsafe { &mut *writer };
        use crossterm::ExecutableCommand;
        if mut_ref
            .execute(crossterm::terminal::EnterAlternateScreen)
            .is_err()
        {
            return Err(PlatformError::Unsupported("alternate screen"));
        }
        Ok(Self {
            writer,
            armed: true,
        })
    }
}

#[allow(unsafe_code)]
impl Drop for AltScreenGuard {
    fn drop(&mut self) {
        if self.armed {
            use crossterm::ExecutableCommand;
            let mut_ref = unsafe { &mut *self.writer };
            let _ = mut_ref.execute(crossterm::terminal::LeaveAlternateScreen);
        }
    }
}

/// Hides the cursor on construction, shows it on drop.
///
/// We always hide the cursor while rendering so it doesn't blink in the
/// middle of an animation. We always show it on drop so the user isn't left
/// staring at an empty terminal with no cursor.
#[allow(unsafe_code)]
#[derive(Debug)]
pub struct CursorShowGuard {
    writer: *mut (dyn std::io::Write + Send),
    armed: bool,
}

// Note: see comment on AltScreenGuard above.

#[allow(unsafe_code)]
impl CursorShowGuard {
    /// # Safety
    /// Same as [`AltScreenGuard::acquire`].
    pub unsafe fn acquire(writer: *mut (dyn std::io::Write + Send)) -> Result<Self, PlatformError> {
        let mut_ref = unsafe { &mut *writer };
        use crossterm::ExecutableCommand;
        if mut_ref.execute(crossterm::cursor::Hide).is_err() {
            return Err(PlatformError::Unsupported("hide cursor"));
        }
        Ok(Self {
            writer,
            armed: true,
        })
    }
}

#[allow(unsafe_code)]
impl Drop for CursorShowGuard {
    fn drop(&mut self) {
        if self.armed {
            use crossterm::ExecutableCommand;
            let mut_ref = unsafe { &mut *self.writer };
            // Belt + braces: show + show again.
            let _ = mut_ref.execute(crossterm::cursor::Show);
            let _ = mut_ref.execute(crossterm::cursor::Show);
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
