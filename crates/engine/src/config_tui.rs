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

/// Open the interactive config TUI, or report that this build lacks it.
///
/// Returns `0` on success, `1` on error / unavailable build.
pub fn run(path: &std::path::Path) -> i32 {
    #[cfg(feature = "tui")]
    {
        match forgum_tui::run_config_tui(path) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("config tui error: {e}");
                1
            }
        }
    }
    #[cfg(not(feature = "tui"))]
    {
        let _ = path;
        eprintln!(
            "this build of forgum-engine was compiled without the `tui` feature; \
             install a tui-enabled build or use `forgum-engine config set <key> <value>`."
        );
        1
    }
}
