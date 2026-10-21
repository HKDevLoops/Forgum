pub mod app;
pub mod celestial_art;
pub mod wizard;

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::Context;
use forgum_platform::protocol::ConfigFormat;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{ConfigApp, Tab};
pub use wizard::{run_installer_wizard, run_uninstaller_wizard};

/// Complete ANSI / VT terminal restoration sequence to guarantee
/// mouse modes, bracketed paste, auto-wrap, and cursor attributes are cleanly restored.
///
/// NOTE: We explicitly DO NOT send `\x1b[r` (DECSTBM margin reset) on the primary screen,
/// because ECMA-48/VT standards specify that `\x1b[r` moves the cursor to (1, 1), which
/// would corrupt the primary screen cursor position and cause subsequent shell commands
/// to overwrite existing terminal contents.
pub const FULL_VT_RESET_SEQ: &[u8] =
    b"\x1b[?7h\x1b[0m\x1b[?25h\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l\x1b[?1015l\x1b[?2004l";

/// Restores the terminal cleanly from raw mode, mouse capture, and alternate screen.
pub fn restore_terminal_cleanly() {
    // 1. Reset margins and attributes INSIDE the alternate screen before leaving
    let _ = io::stdout().write_all(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h");
    let _ = io::stdout().flush();

    // 2. Disable mouse capture and leave alternate screen, cleanly restoring primary screen & cursor
    let _ = crossterm::execute!(
        io::stdout(),
        crossterm::event::DisableMouseCapture,
        crossterm::terminal::LeaveAlternateScreen
    );

    // 3. Reset primary terminal modes without touching scroll margins or cursor position
    let _ = io::stdout().write_all(FULL_VT_RESET_SEQ);
    let _ = io::stdout().flush();
    let _ = crossterm::terminal::disable_raw_mode();
}

/// RAII Drop guard ensuring the terminal is ALWAYS cleanly restored
/// even if a panic, early exit, signal, or error occurs.
#[derive(Debug)]
pub struct TuiTerminalGuard {
    active: bool,
}

impl TuiTerminalGuard {
    pub fn enter() -> anyhow::Result<Self> {
        crossterm::terminal::enable_raw_mode().context("enable raw mode")?;
        let mut stdout = io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )
        .context("enter alternate screen and enable mouse")?;

        // Reset scroll margins, clear screen, and position at top-left
        let _ = stdout.write_all(b"\x1b[r\x1b[2J\x1b[1;1H");
        let _ = stdout.flush();

        Ok(Self { active: true })
    }

    /// Disarm the guard if manual restoration was explicitly performed
    pub fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for TuiTerminalGuard {
    fn drop(&mut self) {
        if self.active {
            restore_terminal_cleanly();
        }
    }
}

/// Write a configuration file atomically using a temporary file and rename.
fn atomic_write_config(target_path: &Path, content: &str) -> anyhow::Result<()> {
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let tmp_path = target_path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&tmp_path, content)
        .with_context(|| format!("write temp file {}", tmp_path.display()))?;
    fs::rename(&tmp_path, target_path).with_context(|| {
        format!(
            "atomic rename {} -> {}",
            tmp_path.display(),
            target_path.display()
        )
    })?;
    Ok(())
}

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
        "studio" | "shader" | "kinematics" | "hull" | "6" => Some(Tab::Studio),
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

    // Stop any active daemon in the current session and remove stale daemon state
    // so it does not compete for terminal output or reserve space during TUI execution.
    let session_id = forgum_platform::detect_session_id();
    let daemon_state_path = forgum_platform::daemon_state_path(&session_id);
    if daemon_state_path.exists() {
        if let Ok(data) = std::fs::read_to_string(&daemon_state_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
                if let (Some(pid), Some(socket_path)) = (
                    val.get("pid").and_then(|v| v.as_u64()).map(|v| v as u32),
                    val.get("socket_path").and_then(|v| v.as_str()),
                ) {
                    if forgum_platform::process_is_alive(pid) {
                        if let Ok(mut sock) =
                            forgum_platform::DaemonSocket::connect(Path::new(socket_path))
                        {
                            let _ = sock.write_response("{\"cmd\":\"STOP\"}\n");
                            std::thread::sleep(Duration::from_millis(50));
                        }
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&daemon_state_path);
    }

    // Terminal setup via RAII guard
    let mut terminal_guard = TuiTerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let shutdown_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let input_paused = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let shutdown_thread = shutdown_flag.clone();
    let paused_thread = input_paused.clone();
    let (input_tx, input_rx) = std::sync::mpsc::channel::<crossterm::event::Event>();

    let _input_handle = std::thread::Builder::new()
        .name("tui-input".into())
        .spawn(move || {
            while !shutdown_thread.load(std::sync::atomic::Ordering::Relaxed) {
                if paused_thread.load(std::sync::atomic::Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(40));
                    continue;
                }
                if let Ok(true) = crossterm::event::poll(Duration::from_millis(25)) {
                    if let Ok(ev) = crossterm::event::read() {
                        if input_tx.send(ev).is_err() {
                            break;
                        }
                    }
                }
            }
        });

    let shutdown_for_closure = shutdown_flag.clone();
    let paused_for_closure = input_paused.clone();

    let result = (|| -> anyhow::Result<()> {
        let mut app = ConfigApp::new(loaded_config, initial_path, initial_tab);
        let mut last_tick = Instant::now();

        loop {
            let fps = (app.config().fps).clamp(1, 240) as u64;
            let frame_budget = Duration::from_micros(1_000_000 / fps);
            let frame_start = Instant::now();

            terminal.draw(|f| {
                app.render(f);
            })?;

            // Drain all pending input events from the dedicated input thread
            while let Ok(event) = input_rx.try_recv() {
                if let Some(action) = app.handle_event(event)? {
                    match action {
                        app::Action::Quit => {
                            shutdown_for_closure.store(true, std::sync::atomic::Ordering::Relaxed);
                            return Ok(());
                        }
                        app::Action::Save => {
                            app.enforce_animation_effect_consistency();
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

                            // Write new configuration file atomically
                            atomic_write_config(&target_path, &text)?;

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
                            broadcast_daemon_reload(app.config());
                        }
                        app::Action::SaveAndQuit => {
                            app.enforce_animation_effect_consistency();
                            let fmt = app.config_format();
                            let text = app
                                .config()
                                .serialize_with_format(fmt)
                                .map_err(|e| anyhow::anyhow!("serialize config: {e}"))?;

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

                            // Write new configuration file atomically
                            atomic_write_config(&target_path, &text)?;

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

                            broadcast_daemon_reload(app.config());
                            shutdown_for_closure.store(true, std::sync::atomic::Ordering::Relaxed);
                            return Ok(());
                        }
                        app::Action::OpenEditor(target_path) => {
                            // Ensure target config file exists on disk
                            if !target_path.is_file() {
                                let fmt = app.config_format();
                                if let Ok(text) = app.config().serialize_with_format(fmt) {
                                    if let Some(parent) = target_path.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }
                                    let _ = fs::write(&target_path, text);
                                }
                            }

                            if let Some(editor) =
                                detect_terminal_editor(app.config().editor.as_deref())
                            {
                                paused_for_closure.store(true, std::sync::atomic::Ordering::SeqCst);

                                // Suspend TUI
                                let _ = crossterm::execute!(
                                    io::stdout(),
                                    crossterm::event::DisableMouseCapture,
                                    crossterm::terminal::LeaveAlternateScreen
                                );
                                let _ = crossterm::terminal::disable_raw_mode();

                                // Launch terminal/configured code editor synchronously
                                let parts: Vec<&str> = editor.split_whitespace().collect();
                                let bin = parts.first().copied().unwrap_or(&editor);
                                let extra_args = if parts.len() > 1 {
                                    &parts[1..]
                                } else {
                                    &[][..]
                                };
                                let mut cmd = std::process::Command::new(bin);
                                cmd.args(extra_args);
                                cmd.arg(&target_path);
                                let _ = cmd.status();

                                // Resume TUI
                                let _ = crossterm::terminal::enable_raw_mode();
                                let _ = crossterm::execute!(
                                    io::stdout(),
                                    crossterm::terminal::EnterAlternateScreen,
                                    crossterm::event::EnableMouseCapture
                                );
                                let _ = terminal.clear();

                                paused_for_closure
                                    .store(false, std::sync::atomic::Ordering::SeqCst);

                                // Reload updated config if edited
                                let fmt = app.config_format();
                                if let Ok(cfg) = read_config_file(&target_path, fmt) {
                                    app.set_config(cfg);
                                    app.config_path = Some(target_path.clone());
                                    app.mark_saved();
                                    app.status_message =
                                        format!("Config reloaded after editing in {}", editor);
                                }
                            } else {
                                app.open_editor_modal();
                            }
                        }
                        app::Action::OpenSystemEditor(target_path) => {
                            if !target_path.is_file() {
                                let fmt = app.config_format();
                                if let Ok(text) = app.config().serialize_with_format(fmt) {
                                    if let Some(parent) = target_path.parent() {
                                        let _ = fs::create_dir_all(parent);
                                    }
                                    let _ = fs::write(&target_path, text);
                                }
                            }

                            #[cfg(windows)]
                            {
                                if forgum_platform::editor::command_exists("code") {
                                    let _ = std::process::Command::new("code.cmd")
                                        .arg(&target_path)
                                        .spawn();
                                    app.status_message = format!(
                                        "Launched VS Code on {}",
                                        target_path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("config")
                                    );
                                } else {
                                    let _ = std::process::Command::new("notepad.exe")
                                        .arg(&target_path)
                                        .spawn();
                                    app.status_message = format!(
                                        "Launched Notepad on {}",
                                        target_path
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("config")
                                    );
                                }
                            }
                            #[cfg(target_os = "macos")]
                            {
                                let _ =
                                    std::process::Command::new("open").arg(&target_path).spawn();
                                app.status_message =
                                    "Opened configuration in system editor.".into();
                            }
                            #[cfg(all(unix, not(target_os = "macos")))]
                            {
                                let _ = std::process::Command::new("xdg-open")
                                    .arg(&target_path)
                                    .spawn();
                                app.status_message =
                                    "Opened configuration in default editor.".into();
                            }
                            app.show_editor_modal = false;
                        }
                        app::Action::InstallTerminalEditor => {
                            app.show_editor_modal = false;
                            #[cfg(windows)]
                            {
                                app.installer_log.push("▶ Run in PowerShell to install Neovim: winget install Neovim.Neovim".to_string());
                                app.status_message =
                                    "Run 'winget install Neovim.Neovim' or 'scoop install neovim'"
                                        .into();
                            }
                            #[cfg(not(windows))]
                            {
                                app.installer_log.push("▶ Run in shell to install Neovim: brew install neovim (or apt install neovim)".to_string());
                                app.status_message =
                                    "Run 'brew install neovim' or 'sudo apt install neovim'".into();
                            }
                        }
                    }
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < frame_budget {
                std::thread::sleep(frame_budget - elapsed);
            }

            let dt = last_tick.elapsed().as_secs_f32();
            app.tick(dt);
            last_tick = Instant::now();
        }
    })();

    shutdown_flag.store(true, std::sync::atomic::Ordering::Relaxed);

    // Restore the terminal cleanly and disarm RAII guard
    restore_terminal_cleanly();
    terminal_guard.disarm();

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

/// Detect available terminal or preferred code editor in order:
/// 1. Configured custom editor in settings (if not empty / "auto")
/// 2. $VISUAL environment variable
/// 3. $EDITOR environment variable
/// 4. Available terminal editors (neovim/nvim, vim, nano, emacs, micro, helix/hx)
/// 5. System/GUI fallback editors (code, notepad on Windows; vi, ed on Unix)
fn detect_terminal_editor(configured: Option<&str>) -> Option<String> {
    if let Some(custom) = configured {
        let trimmed = custom.trim();
        if !trimmed.is_empty() && !trimmed.eq_ignore_ascii_case("auto") {
            let prog = trimmed.split_whitespace().next().unwrap_or(trimmed);
            if let Some(p) = forgum_platform::find_editor_binary(prog) {
                return Some(p.display().to_string());
            }
        }
    }

    let editors = forgum_platform::detect_installed_editors();
    editors.first().map(|e| e.path.display().to_string())
}

/// Broadcast updated configuration parameters to active background daemons via control socket.
fn broadcast_daemon_reload(config: &forgum_platform::SceneConfig) {
    let session_id = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session_id);
    if let Ok(mut conn) = forgum_platform::DaemonSocket::connect(&socket_path) {
        let _ = conn.write_response(&format!(
            "{{\"cmd\":\"EFFECT\",\"arg\":\"{}\"}}\n",
            config.effect
        ));
        let _ = conn.write_response(&format!("{{\"cmd\":\"COW\",\"arg\":\"{}\"}}\n", config.cow));
        let _ = conn.write_response(&format!(
            "{{\"cmd\":\"TEXT\",\"arg\":\"{}\"}}\n",
            config.text
        ));
        let _ = conn.write_response(&format!(
            "{{\"cmd\":\"COLOR\",\"arg\":\"{}\"}}\n",
            config.color_mode
        ));
    }
}
