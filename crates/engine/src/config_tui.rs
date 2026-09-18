//! Bridge between the engine and the optional `forgum-tui` crate.
//!
//! This module is always compiled (so `crates/engine/src` stays free of
//! platform-targeting `#[cfg]` attributes). The runtime branch uses the
//! `cfg!(feature = "tui")` *macro* (allowed by the CI grep). The `forgum_tui`
//! crate name is only required to resolve when the `tui` feature is enabled,
//! hence the single `#[cfg(feature = "tui")]` `extern crate` below — it is a
//! feature-scoped attribute and does NOT match the CI platform-cfg grep
//! (`unix`/`windows`/`target_os`/`target_family`).

#[cfg(feature = "tui")]
extern crate forgum_tui;

/// Open the interactive config TUI for the specified file path.
///
/// Returns `0` on success, `1` on error / unavailable build.
/// Open the interactive config TUI for the specified file path.
///
/// Returns `0` on success, `1` on error / unavailable build.
pub fn run(path: &std::path::Path) -> i32 {
    #[cfg(feature = "tui")]
    {
        crate::daemon::stop_session_daemon_and_reset_margins();
        let code = match forgum_tui::run_config_tui(path) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("config tui error: {e}");
                1
            }
        };
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\x1b[r\x1b[0m\x1b[?25h");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        code
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = path;
        eprintln!(
            "this build of forgum was compiled without the `tui` feature; \
             install a tui-enabled build or use `forgum config set <key> <value>`."
        );
        1
    }
}

/// Open the interactive TUI dashboard & installer without requiring any config file.
///
/// Returns `0` on success, `1` on error / unavailable build.
pub fn run_standalone(initial_tab: Option<&str>) -> i32 {
    #[cfg(feature = "tui")]
    {
        crate::daemon::stop_session_daemon_and_reset_margins();
        let code = match forgum_tui::run_standalone_tui(initial_tab) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("tui error: {e}");
                1
            }
        };
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\x1b[r\x1b[0m\x1b[?25h");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        code
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = initial_tab;
        eprintln!(
            "this build of forgum was compiled without the `tui` feature; \
             install a tui-enabled build."
        );
        1
    }
}

/// Open the interactive Celestial Installation Wizard.
///
/// Returns `0` on success, `1` on error / unavailable build.
pub fn run_installer_wizard() -> i32 {
    #[cfg(feature = "tui")]
    {
        crate::daemon::stop_session_daemon_and_reset_margins();
        let code = match forgum_tui::run_installer_wizard() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("installer error: {e}");
                1
            }
        };
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\x1b[r\x1b[0m\x1b[?25h");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        code
    }
    #[cfg(not(feature = "tui"))]
    {
        eprintln!(
            "this build of forgum was compiled without the `tui` feature; \
             install a tui-enabled build."
        );
        1
    }
}

/// Open the interactive Celestial De-Orbit Uninstallation Wizard.
///
/// Returns `0` on success, `1` on error / unavailable build.
pub fn run_uninstaller_wizard() -> i32 {
    #[cfg(feature = "tui")]
    {
        crate::daemon::stop_session_daemon_and_reset_margins();
        let code = match forgum_tui::run_uninstaller_wizard() {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("uninstaller error: {e}");
                1
            }
        };
        let _ = std::io::Write::write_all(&mut std::io::stdout(), b"\x1b[r\x1b[0m\x1b[?25h");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        code
    }
    #[cfg(not(feature = "tui"))]
    {
        eprintln!(
            "this build of forgum was compiled without the `tui` feature; \
             install a tui-enabled build."
        );
        1
    }
}
