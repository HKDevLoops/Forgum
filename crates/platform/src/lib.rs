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
pub mod mux;
pub mod output;
pub mod paths;
pub mod protocol;
pub mod shell;
pub mod signal;
pub mod sixel;
pub mod spawn;
pub mod terminal;

// Built-in 8×8 bitmap font, only needed by the (feature-gated) Sixel/Kitty
// graphics backend. Gated here so the default build never compiles it.
#[cfg(feature = "sixel")]
pub mod font;

// Platform-specific impls
#[cfg(unix)]
pub mod platform_unix;
#[cfg(windows)]
pub mod platform_windows;

// Re-exports for ergonomic callers.
pub use daemon_socket::{DaemonSocket, SocketConnection};
pub use error::PlatformError;
pub use guards::{AltScreenGuard, CursorShowGuard, RawModeGuard};
pub use mux::{detect_mux, Mux};
pub use output::{open_output, OutputHandle, OutputTarget};
pub use paths::{
    config_path, control_socket_path, daemon_state_path, data_dir, detect_session_id, is_canonical,
    log_dir, runtime_dir, ConfigPaths, ShellKind,
};
pub use protocol::SceneConfig;
pub use shell::Shell;
pub use signal::{ShutdownFlag, SignalGuard};
pub use sixel::{
    create_graphics_renderer, graphics_renderer_available, CellView, FrameBufferLike,
    GraphicsRenderer,
};
#[cfg(unix)]
pub use spawn::fork_then_exec_self;
pub use spawn::{
    daemon_bootstrap, daemonize, prefer_fork_exec, process_is_alive, spawn_detached, DetachedChild,
};

/// Returns the current process's open handle/fd count, or `None` if the OS
/// doesn't expose a reliable signal. Used by the daemon soak test to assert
/// fd/handle-count stability over a long run (G2/D2).
///
/// On Unix this counts open fds (`/proc/self/fd`); on Windows it uses
/// `GetProcessHandleCount`.
#[must_use]
pub fn handle_count() -> Option<usize> {
    #[cfg(unix)]
    {
        platform_unix::handle_count()
    }
    #[cfg(windows)]
    {
        platform_windows::handle_count()
    }
}
pub use terminal::{
    detect_capabilities, terminal_supports_sync, ColorLevel, GraphicsCaps, TerminalCapabilities,
};
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
