//! Signal handling.
//!
//! Converts OS termination signals into a shared [`ShutdownFlag`] (an
//! `Arc<AtomicBool>`) that the render loop polls each frame. Fixes BUG-T1:
//! the old code had no signal handlers, so a `kill <pid>` or terminal close
//! left the engine mid-frame with raw mode / alt screen / hidden cursor still
//! active.
//!
//! ## Async-signal safety
//!
//! POSIX restricts what a signal handler may safely do: only
//! `async-signal-safe` operations, or — much simpler — flipping a
//! `sig_atomic`-like flag. We do the latter. `signal-hook` is built around
//! this guarantee, so our handlers are correct.
//!
//! On Windows, `SetConsoleCtrlHandler` invokes the registered routine on
//! the kernel's console-handler thread, not from interrupt context, so the
//! usual Rust restrictions don't apply — we just set the flag.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::error::PlatformError;

/// A cloneable handle to the shutdown flag.
#[derive(Debug, Clone)]
pub struct ShutdownFlag {
    flag: Arc<AtomicBool>,
}

impl ShutdownFlag {
    #[must_use]
    pub fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns `true` once a signal has been received.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    /// Returns the underlying `Arc` for sharing with control socket threads.
    #[must_use]
    pub fn handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.flag)
    }

    /// Test/utility: set the flag without a real signal (used by the control
    /// socket on STOP and by the unit tests).
    pub fn trigger(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

impl Default for ShutdownFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// Installs signal handlers for SIGINT, SIGTERM, and SIGHUP (Unix) or the
/// Windows console-close events. The returned guard un-registers them on drop.
#[derive(Debug)]
pub struct SignalGuard {
    // On Unix we keep the signal-hook registry objects alive; on Windows
    // we keep the handler routine registered and clear it on Drop.
    #[cfg(unix)]
    unix_state: Option<UnixSignalState>,
}

#[cfg(unix)]
struct UnixSignalState {
    _int: signal_hook_registry::SigId,
    _term: signal_hook_registry::SigId,
    _hup: signal_hook_registry::SigId,
}

impl SignalGuard {
    /// Install handlers that flip `flag` on receipt.
    pub fn install(flag: ShutdownFlag) -> Result<Self, PlatformError> {
        #[cfg(unix)]
        {
            use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM};
            use signal_hook_registry::{register, SignalHook};

            let handle = flag.handle();

            // The closure must be `Fn` + `Send` + Sync + 'static and is
            // called from the signal-hook dispatch thread, not interrupt
            // context, so we can do regular atomic operations.
            let h_for_int = Arc::clone(&handle);
            let h_for_term = Arc::clone(&handle);
            let h_for_hup = Arc::clone(&handle);

            let int_id = register(SIGINT, move || {
                h_for_int.store(true, std::sync::atomic::Ordering::Relaxed);
            })
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGINT",
                source: e,
            })?;
            let term_id = register(SIGTERM, move || {
                h_for_term.store(true, std::sync::atomic::Ordering::Relaxed);
            })
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGTERM",
                source: e,
            })?;
            let hup_id = register(SIGHUP, move || {
                h_for_hup.store(true, std::sync::atomic::Ordering::Relaxed);
            })
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGHUP",
                source: e,
            })?;
            return Ok(Self {
                unix_state: Some(UnixSignalState {
                    _int: int_id,
                    _term: term_id,
                    _hup: hup_id,
                }),
            });
        }
        #[cfg(windows)]
        {
            install_windows(flag)?;
            Ok(Self {})
        }
    }
}

impl Drop for SignalGuard {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            // signal-hook auto-unregisters when SigId is dropped.
            self.unix_state = None;
        }
        #[cfg(windows)]
        #[allow(unsafe_code)]
        {
            // Re-enable default Ctrl-C handling.
            unsafe {
                let _ = windows_sys::Win32::System::Console::SetConsoleCtrlHandler(None, 0);
            }
        }
    }
}

#[cfg(windows)]
type ConsoleHandlerRoutine = unsafe extern "system" fn(u32) -> i32;

#[cfg(windows)]
static mut WINDOWS_FLAG: Option<Arc<AtomicBool>> = None;

#[cfg(windows)]
#[allow(unsafe_code, static_mut_refs)]
unsafe extern "system" fn windows_ctrl_handler(event_type: u32) -> i32 {
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_CLOSE_EVENT, CTRL_C_EVENT, CTRL_LOGOFF_EVENT, CTRL_SHUTDOWN_EVENT,
    };
    // We handle all events by setting the flag. Returning TRUE tells the
    // kernel we handled it (so it doesn't kill us); for close/logoff/
    // shutdown the kernel will kill us anyway after a short grace period,
    // but setting the flag first lets the render loop notice and clean up.
    let triggered = matches!(
        event_type,
        CTRL_C_EVENT
            | CTRL_BREAK_EVENT
            | CTRL_CLOSE_EVENT
            | CTRL_LOGOFF_EVENT
            | CTRL_SHUTDOWN_EVENT
    );
    if triggered {
        // SAFETY: this function is called from the kernel's console-handler
        // thread, not interrupt context. We only do atomic ops below.
        let flag_opt = unsafe { WINDOWS_FLAG.as_ref() };
        if let Some(flag) = flag_opt {
            flag.store(true, Ordering::Relaxed);
        }
    }
    i32::from(triggered)
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn install_windows(flag: ShutdownFlag) -> Result<(), PlatformError> {
    use windows_sys::Win32::System::Console::SetConsoleCtrlHandler;
    unsafe {
        WINDOWS_FLAG = Some(flag.handle());
        let result = SetConsoleCtrlHandler(Some(windows_ctrl_handler as ConsoleHandlerRoutine), 1);
        if result == 0 {
            return Err(PlatformError::SignalRegistration {
                signal: "SetConsoleCtrlHandler",
                source: std::io::Error::last_os_error(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_starts_false_and_is_cloneable() {
        let flag = ShutdownFlag::new();
        assert!(!flag.is_shutdown());
        let h1 = flag.handle();
        let h2 = flag.handle();
        assert!(!Arc::ptr_eq(&h1, &h2) || Arc::strong_count(&h1) == 3); // flag + h1 + h2
    }

    #[test]
    fn flag_trigger_sets_state() {
        let flag = ShutdownFlag::new();
        flag.trigger();
        assert!(flag.is_shutdown());
    }

    #[test]
    fn install_succeeds_in_isolation() {
        let flag = ShutdownFlag::new();
        let guard = SignalGuard::install(flag.clone());
        assert!(guard.is_ok(), "install failed: {:?}", guard.err());
        // Triggering from any thread should be visible.
        flag.trigger();
        assert!(flag.is_shutdown());
    }

    #[test]
    fn install_twice_overwrites() {
        // signal-hook allows multiple registrations; Windows SetConsoleCtrlHandler
        // overwrites. Both should still leave the flag functional.
        let a = ShutdownFlag::new();
        let _g1 = SignalGuard::install(a.clone()).unwrap();
        let b = ShutdownFlag::new();
        let _g2 = SignalGuard::install(b.clone()).unwrap();
        a.trigger();
        b.trigger();
        assert!(a.is_shutdown());
        assert!(b.is_shutdown());
    }
}
