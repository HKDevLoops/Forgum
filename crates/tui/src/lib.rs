//! Interactive config TUI for Forgum.
//!
//! Exposes a single entry point, [`run_config_tui`], which the engine invokes
//! (behind the `tui` feature) to let the user edit their `SceneConfig` in a
//! ratatui + crossterm terminal UI.

pub mod app;

use std::fs;
use std::io;
use std::path::Path;

use anyhow::Context;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::ConfigApp;

/// Run the interactive config editor for the file at `config_path`.
///
/// This is the exact signature the engine calls via `cfg!(feature = "tui")`.
pub fn run_config_tui(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Load the existing config, or fall back to defaults if missing/invalid.
    let config = read_config_file(config_path).unwrap_or_default();

    // Terminal setup.
    crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .context("enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let result = (|| -> anyhow::Result<()> {
        let mut app = ConfigApp::new(config);
        loop {
            terminal.draw(|f| {
                app.render(f);
            })?;
            if let Some(action) = app.handle_event(crossterm::event::read()?)? {
                match action {
                    app::Action::Quit => return Ok(()),
                    app::Action::Save => {
                        let json = serde_json::to_string_pretty(&app.config())
                            .context("serialize config")?;
                        if let Some(parent) = config_path.parent() {
                            fs::create_dir_all(parent)
                                .with_context(|| format!("create {}", parent.display()))?;
                        }
                        fs::write(config_path, json)
                            .with_context(|| format!("write {}", config_path.display()))?;
                        app.mark_saved();
                    }
                }
            }
        }
    })();

    // Restore the terminal no matter what happened above.
    crossterm::terminal::disable_raw_mode().context("disable raw mode")?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)
        .context("leave alternate screen")?;

    result?;
    Ok(())
}

/// Read a JSON `SceneConfig` from disk, returning `Err` on any I/O or parse
/// failure (the caller falls back to defaults). Mirrors the engine's loader
/// but lives here so the TUI crate stays free of an `forgum-engine` dependency.
fn read_config_file(path: &Path) -> Result<forgum_platform::protocol::SceneConfig, anyhow::Error> {
    let bytes = std::fs::read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| anyhow::anyhow!("config is not valid UTF-8: {}", path.display()))?;
    Ok(serde_json::from_str(text)?)
}
