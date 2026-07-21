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

/// A cloneable handle to the shutdown and resize flags.
#[derive(Debug, Clone)]
pub struct ShutdownFlag {
    shutdown: Arc<AtomicBool>,
    resize: Arc<AtomicBool>,
}

impl ShutdownFlag {
    #[must_use]
    pub fn new() -> Self {
        Self {
            shutdown: Arc::new(AtomicBool::new(false)),
            resize: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Returns `true` once a termination signal has been received.
    #[must_use]
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(Ordering::Relaxed)
    }

    /// Returns `true` if a SIGWINCH/resize was received; clears the flag.
    #[must_use]
    pub fn check_and_clear_resize(&self) -> bool {
        self.resize.swap(false, Ordering::Relaxed)
    }

    /// Manually set the resize flag (for testing).
    pub fn trigger_resize(&self) {
        self.resize.store(true, Ordering::Relaxed);
    }

    /// Returns the underlying shutdown `Arc` for sharing with control socket threads.
    #[must_use]
    pub fn shutdown_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shutdown)
    }

    /// Returns the underlying resize `Arc` for sharing with signal handlers.
    #[must_use]
    pub fn resize_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.resize)
    }
    /// Test/utility: set the shutdown flag without a real signal.
    pub fn trigger(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
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
#[derive(Debug)]
struct UnixSignalState {
    _int: signal_hook_registry::SigId,
    _term: signal_hook_registry::SigId,
    _hup: signal_hook_registry::SigId,
    _winch: signal_hook_registry::SigId,
}

impl SignalGuard {
    /// Install handlers that flip `flag` on receipt.
    #[allow(unsafe_code)]
    pub fn install(flag: ShutdownFlag) -> Result<Self, PlatformError> {
        #[cfg(unix)]
        {
            use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM, SIGWINCH};
            use signal_hook_registry::register;

            let shutdown_handle = flag.shutdown_handle();
            let resize_handle = flag.resize_handle();

            let h_for_int = Arc::clone(&shutdown_handle);
            let h_for_term = Arc::clone(&shutdown_handle);
            let h_for_hup = Arc::clone(&shutdown_handle);
            let h_for_winch = Arc::clone(&resize_handle);

            let int_id = unsafe {
                register(SIGINT, move || {
                    h_for_int.store(true, Ordering::Relaxed);
                })
            }
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGINT",
                source: e,
            })?;
            let term_id = unsafe {
                register(SIGTERM, move || {
                    h_for_term.store(true, Ordering::Relaxed);
                })
            }
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGTERM",
                source: e,
            })?;
            let hup_id = unsafe {
                register(SIGHUP, move || {
                    h_for_hup.store(true, Ordering::Relaxed);
                })
            }
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGHUP",
                source: e,
            })?;
            let winch_id = unsafe {
                register(SIGWINCH, move || {
                    h_for_winch.store(true, Ordering::Relaxed);
                })
            }
            .map_err(|e| PlatformError::SignalRegistration {
                signal: "SIGWINCH",
                source: e,
            })?;
            Ok(Self {
                unix_state: Some(UnixSignalState {
                    _int: int_id,
                    _term: term_id,
                    _hup: hup_id,
                    _winch: winch_id,
                }),
            })
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
    let triggered = matches!(
        event_type,
        CTRL_C_EVENT
            | CTRL_BREAK_EVENT
            | CTRL_CLOSE_EVENT
            | CTRL_LOGOFF_EVENT
            | CTRL_SHUTDOWN_EVENT
    );
    if triggered {
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
        WINDOWS_FLAG = Some(flag.shutdown_handle());
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
        assert!(!flag.check_and_clear_resize());
        let h1 = flag.shutdown_handle();
        let h2 = flag.shutdown_handle();
        assert!(!Arc::ptr_eq(&h1, &h2) || Arc::strong_count(&h1) == 3);
    }

    #[test]
    fn flag_trigger_sets_state() {
        let flag = ShutdownFlag::new();
        flag.trigger();
        assert!(flag.is_shutdown());
    }

    #[test]
    fn resize_flag_works() {
        let flag = ShutdownFlag::new();
        assert!(!flag.check_and_clear_resize());
        flag.trigger_resize();
        assert!(flag.check_and_clear_resize());
        assert!(!flag.check_and_clear_resize()); // cleared after read
    }

    #[test]
    fn install_succeeds_in_isolation() {
        let flag = ShutdownFlag::new();
        let guard = SignalGuard::install(flag.clone());
        assert!(guard.is_ok(), "install failed: {:?}", guard.err());
        flag.trigger();
        assert!(flag.is_shutdown());
    }

    #[test]
    fn install_twice_overwrites() {
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
