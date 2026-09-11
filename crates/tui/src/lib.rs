pub mod app;
pub mod celestial_art;
pub mod wizard;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context;
use forgum_platform::protocol::ConfigFormat;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{ConfigApp, Tab};
pub use wizard::{run_installer_wizard, run_uninstaller_wizard};

/// Run the interactive TUI associated with a config path (backward compatibility).
///
/// NOTE: Even if `config_path` does not exist on disk, this DOES NOT fail. It
/// initializes cleanly in-memory with defaults.
pub fn run_config_tui(config_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_tui(Some(config_path), None)
}

/// Run the standalone interactive TUI dashboard & installer (no config file required).
pub fn run_standalone_tui(initial_tab: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let tab = match initial_tab.unwrap_or("").to_lowercase().as_str() {
        "mascot" | "mascots" | "cows" | "animals" | "1" => Some(Tab::Mascots),
        "scenery" | "biomes" | "roads" | "2" => Some(Tab::Scenery),
        "fx" | "effects" | "anim" | "colors" | "3" => Some(Tab::Effects),
        "install" | "installer" | "shell" | "shells" | "4" => Some(Tab::Installer),
        "config" | "settings" | "5" => Some(Tab::Config),
        _ => None,
    };
    run_tui(None, tab)
}

/// Master runner for the Forgum Studio TUI dashboard.
///
/// 100% in-memory resilient: if `config_path` is `None` or the file does not exist,
/// the TUI runs flawlessly without any dependencies on the file system.
pub fn run_tui(
    config_path: Option<&Path>,
    initial_tab: Option<Tab>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Attempt to load existing config if file exists, else use None (defaults in memory).
    let (loaded_config, initial_path) = match config_path {
        Some(p) if p.is_file() => {
            let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("json");
            let fmt = ConfigFormat::from_extension(ext).unwrap_or(ConfigFormat::Json);
            let cfg = read_config_file(p, fmt).ok();
            (cfg, Some(p.to_path_buf()))
        }
        Some(p) => (None, Some(p.to_path_buf())),
        None => {
            if let Ok((path, format)) = forgum_platform::paths::detect_config_file(None) {
                if path.is_file() {
                    let cfg = read_config_file(&path, format).ok();
                    (cfg, Some(path))
                } else {
                    (None, Some(path))
                }
            } else {
                (None, None)
            }
        }
    };

    // Terminal setup
    crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)
        .context("enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let result = (|| -> anyhow::Result<()> {
        let mut app = ConfigApp::new(loaded_config, initial_path, initial_tab);
        let mut last_tick = Instant::now();

        loop {
            let fps = (app.config().fps).clamp(1, 240) as u64;
            let tick_rate = Duration::from_micros(1_000_000 / fps);

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

                            // If a path was provided, use it; otherwise resolve the default config path
                            let target_path: PathBuf = match &app.config_path {
                                Some(p) => {
                                    if let Some(parent) = p.parent() {
                                        let _ = fs::create_dir_all(parent);
                                        parent.join(format!("config.{}", fmt.extension()))
                                    } else {
                                        p.clone()
                                    }
                                }
                                None => {
                                    let default_dir = forgum_platform::paths::config_dir()
                                        .unwrap_or_else(|_| PathBuf::from("."));
                                    let _ = fs::create_dir_all(&default_dir);
                                    default_dir.join(format!("config.{}", fmt.extension()))
                                }
                            };

                            // Write new configuration file first
                            fs::write(&target_path, text)
                                .with_context(|| format!("write {}", target_path.display()))?;

                            // Remove conflicting alternate format configs in the directory to satisfy mutual exclusivity
                            if let Some(target_dir) = target_path.parent() {
                                let canonical_target = target_path.canonicalize().ok();
                                for candidate in [
                                    "config.json",
                                    "forgum.json",
                                    "config.yaml",
                                    "config.yml",
                                    "forgum.yaml",
                                    "forgum.yml",
                                    "config.toml",
                                    "forgum.toml",
                                ] {
                                    let old = target_dir.join(candidate);
                                    let is_same = match (&canonical_target, old.canonicalize()) {
                                        (Some(c_tgt), Ok(c_old)) => c_tgt == &c_old,
                                        _ => old.file_name() == target_path.file_name(),
                                    };
                                    if !is_same && old.exists() {
                                        let _ = fs::remove_file(old);
                                    }
                                }
                            }

                            app.config_path = Some(target_path);
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

    // Restore the terminal unconditionally
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);

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
