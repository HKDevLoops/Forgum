pub mod app;

use std::fs;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Context;
use forgum_platform::protocol::ConfigFormat;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::ConfigApp;

/// Run the interactive config editor for the file at `config_path`.
///
/// This is the exact signature the engine calls via `cfg!(feature = "tui")`.
pub fn run_config_tui(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Detect format and load existing config, or fall back to defaults if missing/invalid.
    let ext = config_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("json");
    let initial_format = ConfigFormat::from_extension(ext).unwrap_or(ConfigFormat::Json);
    let config = read_config_file(config_path, initial_format).unwrap_or_default();

    // Terminal setup.
    crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .context("enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let result = (|| -> anyhow::Result<()> {
        let mut app = ConfigApp::new(config, initial_format);
        let tick_rate = Duration::from_millis(33); // ~30 FPS live preview
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| {
                app.render(f);
            })?;

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if crossterm::event::poll(timeout)? {
                if let Some(action) = app.handle_event(crossterm::event::read()?)? {
                    match action {
                        app::Action::Quit => return Ok(()),
                        app::Action::Save => {
                            let fmt = app.config_format();
                            let text = app
                                .config()
                                .serialize_with_format(fmt)
                                .map_err(|e| anyhow::anyhow!("serialize config: {e}"))?;
                            if let Some(parent) = config_path.parent() {
                                fs::create_dir_all(parent)
                                    .with_context(|| format!("create {}", parent.display()))?;
                            }
                            let target_path = config_path
                                .parent()
                                .map(|p| p.join(format!("config.{}", fmt.extension())))
                                .unwrap_or_else(|| config_path.to_path_buf());
                            fs::write(&target_path, text)
                                .with_context(|| format!("write {}", target_path.display()))?;
                            if config_path.is_file() && config_path != target_path {
                                let _ = fs::remove_file(config_path);
                            }
                            app.mark_saved();
                        }
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                app.tick(last_tick.elapsed().as_secs_f32());
                last_tick = Instant::now();
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

/// Read a `SceneConfig` from disk in JSON, YAML, or TOML format.
fn read_config_file(
    path: &Path,
    format: ConfigFormat,
) -> Result<forgum_platform::protocol::SceneConfig, anyhow::Error> {
    let bytes = std::fs::read(path)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| anyhow::anyhow!("config is not valid UTF-8: {}", path.display()))?;
    forgum_platform::protocol::SceneConfig::parse_with_format(text, format)
        .map_err(|e| anyhow::anyhow!("parse config: {e}"))
}
