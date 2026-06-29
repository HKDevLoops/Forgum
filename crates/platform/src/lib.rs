//! # forgum-platform
//!
//! Cross-platform abstraction layer for the Forgum engine.
//!
//! **Contract:** this crate is the *only* place in the Forgum workspace where
//! `#[cfg(unix)]` / `#[cfg(windows)]` may appear. The CI gate
//! (`.github/workflows/ci.yml`) runs `rg '#\[cfg' crates/engine/src/` and fails
//! if any hits are found.
//!
//! ## The five seams
//!
//! 1. **Terminal handle** — query size, query capability.
//! 2. **Output** — open the right sink (stdout, `/dev/tty`, `CONOUT$`).
//! 3. **Guards** — RAII for raw mode, alt screen, cursor visibility.
//! 4. **Signals** — convert POSIX signals / Win32 console events into an
//!    `AtomicBool` checked by the render loop.
//! 5. **Spawn** — detach a child process so the daemon outlives the shell.
//!
//! Plus: paths (XDG / Roaming / `TMPDIR`) and a typed error type.
//!
//! ## Using platform-specific code from the engine
//!
//! Callers that need platform branches should use the [`cfg_unix!`] /
//! [`cfg_windows!`] macros exported here, which expand to the appropriate
//! `#[cfg]` attribute. This keeps the `#[cfg]` literals confined to this crate.
//!
//! ```ignore
//! use forgum_platform::{cfg_unix, cfg_windows};
//!
//! cfg_unix! { /* unix-only code */ }
//! cfg_windows! { /* windows-only code */ }
//! ```

#![doc(html_root_url = "https://docs.rs/forgum-platform/0.4.0")]

pub mod daemon_socket;
pub mod error;
pub mod guards;
pub mod output;
pub mod paths;
pub mod signal;
pub mod spawn;
pub mod terminal;

// Platform-specific impls
#[cfg(unix)]
pub mod platform_unix;
#[cfg(windows)]
pub mod platform_windows;

// Re-exports for ergonomic callers.
pub use daemon_socket::{DaemonSocket, SocketConnection};
pub use error::PlatformError;
pub use guards::{AltScreenGuard, CursorShowGuard, RawModeGuard};
pub use output::{open_output, OutputHandle, OutputTarget};
pub use paths::{
    config_path, control_socket_path, daemon_state_path, data_dir, detect_session_id, log_dir,
    runtime_dir, ConfigPaths, ShellKind,
};
pub use signal::{ShutdownFlag, SignalGuard};
pub use spawn::{daemonize, process_is_alive, spawn_detached, DetachedChild};
pub use terminal::{detect_capabilities, ColorLevel, TerminalCapabilities};

/// Expand to the contained code only when compiling on a Unix-like target.
///
/// This macro emits a `#[cfg(unix)]` attribute that gates a block, so the
/// contained code is silently absent on non-Unix targets. Use it from
/// crates that want platform-specific paths without sprinkling `#[cfg]`
/// themselves — this keeps the CI grep gate (zero `#[cfg` in
/// `engine/src/`) clean.
///
/// # Example
///
/// ```ignore
/// use forgum_platform::cfg_unix;
/// let path: &str = cfg_unix! { "/dev/tty" };
/// ```
#[macro_export]
macro_rules! cfg_unix {
    ($($tt:tt)*) => {
        #[cfg(unix)]
        {
            $($tt)*
        }
    };
}

/// Expand to the contained code only when compiling on Windows. See
/// [`cfg_unix!`] for the rationale.
#[macro_export]
macro_rules! cfg_windows {
    ($($tt:tt)*) => {
        #[cfg(windows)]
        {
            $($tt)*
        }
    };
}

/// Returns the name of the operating system the binary was built for.
#[must_use]
pub fn target_os() -> &'static str {
    #[cfg(unix)]
    return "unix";
    #[cfg(windows)]
    return "windows";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_os_is_stable() {
        let s = target_os();
        assert!(s == "unix" || s == "windows");
    }
}
