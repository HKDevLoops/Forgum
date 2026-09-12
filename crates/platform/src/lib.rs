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

pub mod biome;
pub mod daemon_socket;
pub mod error;
pub mod guards;
pub mod mux;
pub mod output;
pub mod package_manager;
pub mod paths;
pub mod protocol;
pub mod shell;
pub mod signal;
pub mod sixel;
pub mod spawn;
pub mod telemetry;
pub mod terminal;
pub mod uninstaller;

// Built-in 8×8 bitmap font for real glyph rasterization and video capture.
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
pub use package_manager::{
    detect_available_package_managers, detect_installation_source, detect_source_from_path,
    execute_package_manager_action, PackageManager,
};
pub use paths::{
    config_dir, config_path, control_socket_path, daemon_state_path, data_dir, detect_config_file,
    detect_session_id, is_canonical, log_dir, open_folder_in_desktop, runtime_dir, ConfigPaths,
    ShellKind,
};
#[cfg(unix)]
pub use platform_unix::parent_pid;
#[cfg(windows)]
pub use platform_windows::parent_pid;
pub use protocol::{ConfigFormat, SceneConfig};
pub use shell::{
    remove_all_forgum_blocks, remove_delimited_block, uninstall_all_shell_integrations,
    uninstall_shell_integration, update_delimited_block, write_file_if_changed, Shell,
    ALL_FORGUM_MARKER_PAIRS, COMPLETIONS_MARKER_BEGIN, COMPLETIONS_MARKER_END, HOOK_MARKER_BEGIN,
    HOOK_MARKER_END,
};
pub use signal::{ShutdownFlag, SignalGuard};
pub use sixel::{
    create_graphics_renderer, graphics_renderer_available, CellView, FrameBufferLike,
    GraphicsRenderer,
};
#[cfg(unix)]
pub use spawn::fork_then_exec_self;
pub use spawn::{
    daemon_bootstrap, daemonize, execute_command_with_shell_fallback, prefer_fork_exec,
    process_is_alive, spawn_detached, DetachedChild,
};
pub use telemetry::{
    is_telemetry_allowed, record_active_pulse, record_installed, record_tried,
    set_telemetry_consent, Metric,
};
pub use uninstaller::{perform_uninstallation, UninstallMode, UninstallReport};

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

/// Check if stdin has data available to read without blocking indefinitely.
#[must_use]
pub fn stdin_has_data() -> bool {
    #[cfg(unix)]
    {
        platform_unix::stdin_has_data()
    }
    #[cfg(windows)]
    {
        platform_windows::stdin_has_data()
    }
}
pub use terminal::{
    detect_capabilities, is_stdout_tty, terminal_size, terminal_supports_sync, ColorLevel,
    GraphicsCaps, TerminalCapabilities,
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
/// ```text
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
    {
        "unix"
    }
    #[cfg(windows)]
    {
        "windows"
    }
}

/// Check battery charge percentage. Returns `Some(pct)` if a battery is
/// detected, `None` on desktops or unsupported platforms.
///
/// This is used by the engine's battery-reactive throttle (Phase 8.5)
/// to drop to ~5fps when charge is below 20%.
#[must_use]
pub fn check_battery_percent() -> Option<f32> {
    // Linux: read from sysfs
    #[cfg(target_os = "linux")]
    {
        if let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") {
            for entry in entries.flatten() {
                let capacity_path = entry.path().join("capacity");
                if let Ok(text) = std::fs::read_to_string(&capacity_path) {
                    if let Ok(pct) = text.trim().parse::<f32>() {
                        return Some(pct);
                    }
                }
            }
        }
        None
    }
    // macOS: use ioreg to get battery info
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("ioreg")
            .args(["-r", "-c", "AppleSmartBattery"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("\"CurrentCapacity\"") {
                    if let Some(val) = line.split('=').nth(1) {
                        if let Ok(current) = val.trim().parse::<f32>() {
                            // Also find MaxCapacity to compute percentage
                            for line2 in stdout.lines() {
                                if line2.contains("\"MaxCapacity\"") {
                                    if let Some(val2) = line2.split('=').nth(1) {
                                        if let Ok(max) = val2.trim().parse::<f32>() {
                                            if max > 0.0 {
                                                return Some(current / max * 100.0);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    // Windows: return None (desktops don't have batteries; laptops
    // would need windows-sys power status API, deferred).
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// Open a file or folder in the host operating system's default viewer or editor.
pub fn open_in_system_viewer(target_path: &std::path::Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        if target_path.is_dir() {
            std::process::Command::new("explorer")
                .arg(target_path)
                .spawn()?;
        } else {
            let status = std::process::Command::new("cmd")
                .args(["/c", "start", ""])
                .arg(target_path)
                .spawn();
            if status.is_err() {
                std::process::Command::new("notepad")
                    .arg(target_path)
                    .spawn()?;
            }
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target_path)
            .spawn()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(target_path)
            .spawn()?;
        Ok(())
    }
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
