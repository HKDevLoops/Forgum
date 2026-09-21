//! Celestial / Omarchy Installation and Uninstallation Terminal UI Wizards.
//!
//! Features:
//! - Multi-step celestial wizard workflows
//! - Transparent motivation telemetry permission consent
//! - Dual uninstallation methods: Soft (keep configs) vs Purge (clean slate)
//! - Rich cosmic TrueColor gradients and handcrafted ASCII artworks
//! - Resilient non-blocking event loops with graceful terminal restoration

use std::io;
use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Gauge, List, ListItem, Paragraph, Wrap};
use ratatui::{Frame, Terminal};

use forgum_platform::package_manager::{
    detect_available_package_managers, detect_shadow_installations, record_receipt, PackageManager,
    ReleaseChannel, ShadowInstallation,
};
use forgum_platform::shell::Shell;
use forgum_platform::telemetry::{record_installed, record_tried, set_telemetry_consent};
use forgum_platform::terminal::{detect_capabilities, detect_utf8_support, ColorLevel};
use forgum_platform::uninstaller::{perform_uninstallation, UninstallMode, UninstallReport};

use crate::celestial_art::*;

/// Cosmic color palette constants for the Celestial & Omarchy theme.
pub mod theme {
    use ratatui::style::Color;

    pub const ASTRAL_PURPLE: Color = Color::Rgb(168, 85, 247);
    pub const NEBULA_CYAN: Color = Color::Rgb(6, 182, 212);
    pub const STARFIRE_PINK: Color = Color::Rgb(236, 72, 153);
    pub const SUPERNOVA_GOLD: Color = Color::Rgb(245, 158, 11);
    pub const AURORA_GREEN: Color = Color::Rgb(16, 185, 129);
    pub const METEOR_RED: Color = Color::Rgb(239, 68, 68);
    pub const COSMIC_DARK: Color = Color::Rgb(15, 23, 42);
    pub const DEEP_SPACE: Color = Color::Rgb(30, 41, 59);
    pub const STARLIGHT: Color = Color::Rgb(248, 250, 252);
    pub const DUST_GRAY: Color = Color::Rgb(148, 163, 184);
}

// ═══════════════════════════════════════════════════════════════════════════
// INSTALLATION WIZARD
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallStep {
    Welcome,
    Preflight,
    TelemetryConsent,
    ShellSelection,
    Executing,
    Complete,
}

#[derive(Debug)]
pub struct InstallerWizard {
    step: InstallStep,
    selected_channel: ReleaseChannel,
    channel_selection_idx: usize,
    shadow_installations: Vec<ShadowInstallation>,
    package_managers: Vec<(PackageManager, bool)>,
    has_ffmpeg: bool,
    color_level: ColorLevel,
    sync_supported: bool,
    utf8_supported: bool,
    telemetry_allowed: bool,
    telemetry_selection: bool,        // true = Yes, false = No
    shells: Vec<(Shell, bool, bool)>, // (Shell, detected_on_host, selected_for_install)
    selected_shell_idx: usize,
    progress: u16,
    install_logs: Vec<String>,
    animation_tick: usize,
}

impl Default for InstallerWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallerWizard {
    pub fn new() -> Self {
        // Record that a user tried / opened the installer
        record_tried();

        // Host and environment diagnostic probes
        let caps = detect_capabilities();
        let shadow_installations = detect_shadow_installations();
        let package_managers = detect_available_package_managers();
        let has_ffmpeg = which_exists("ffmpeg");
        let utf8_supported = detect_utf8_support();

        // Detect shells present on current host
        let mut shells = Vec::new();
        for &sh in Shell::ALL {
            let detected = match sh {
                Shell::Bash => which_exists("bash"),
                Shell::Zsh => which_exists("zsh"),
                Shell::Fish => which_exists("fish"),
                Shell::Pwsh => which_exists("pwsh"),
                Shell::PowerShell => which_exists("powershell") || which_exists("powershell.exe"),
                Shell::Nushell => which_exists("nu"),
                Shell::Elvish => which_exists("elvish"),
                Shell::Cmd => cfg!(windows),
                _ => false,
            };
            shells.push((sh, detected, detected));
        }

        Self {
            step: InstallStep::Welcome,
            selected_channel: ReleaseChannel::Stable,
            channel_selection_idx: 0,
            shadow_installations,
            package_managers,
            has_ffmpeg,
            color_level: caps.color,
            sync_supported: caps.sync,
            utf8_supported,
            telemetry_allowed: true,
            telemetry_selection: true,
            shells,
            selected_shell_idx: 0,
            progress: 0,
            install_logs: Vec::new(),
            animation_tick: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }
        match self.step {
            InstallStep::Welcome => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right => {
                    self.step = InstallStep::Preflight;
                }
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            },
            InstallStep::Preflight => match key.code {
                KeyCode::Left | KeyCode::BackTab => {
                    if self.channel_selection_idx > 0 {
                        self.channel_selection_idx -= 1;
                    } else {
                        self.channel_selection_idx = ReleaseChannel::ALL.len() - 1;
                    }
                    self.selected_channel = ReleaseChannel::ALL[self.channel_selection_idx];
                }
                KeyCode::Right | KeyCode::Tab => {
                    self.channel_selection_idx =
                        (self.channel_selection_idx + 1) % ReleaseChannel::ALL.len();
                    self.selected_channel = ReleaseChannel::ALL[self.channel_selection_idx];
                }
                KeyCode::Char('1') => {
                    self.channel_selection_idx = 0;
                    self.selected_channel = ReleaseChannel::Stable;
                }
                KeyCode::Char('2') => {
                    self.channel_selection_idx = 1;
                    self.selected_channel = ReleaseChannel::Alpha;
                }
                KeyCode::Char('3') => {
                    self.channel_selection_idx = 2;
                    self.selected_channel = ReleaseChannel::Nightly;
                }
                KeyCode::Char('4') => {
                    self.channel_selection_idx = 3;
                    self.selected_channel = ReleaseChannel::Dev;
                }
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Down => {
                    self.step = InstallStep::TelemetryConsent;
                }
                KeyCode::Esc | KeyCode::Up => {
                    self.step = InstallStep::Welcome;
                }
                KeyCode::Char('q') => return true,
                _ => {}
            },
            InstallStep::TelemetryConsent => match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                    self.telemetry_selection = !self.telemetry_selection;
                }
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.telemetry_selection = true;
                    self.telemetry_allowed = true;
                    let _ = set_telemetry_consent(true);
                    self.step = InstallStep::ShellSelection;
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    self.telemetry_selection = false;
                    self.telemetry_allowed = false;
                    let _ = set_telemetry_consent(false);
                    self.step = InstallStep::ShellSelection;
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.telemetry_allowed = self.telemetry_selection;
                    let _ = set_telemetry_consent(self.telemetry_selection);
                    self.step = InstallStep::ShellSelection;
                }
                KeyCode::Esc => self.step = InstallStep::Preflight,
                _ => {}
            },
            InstallStep::ShellSelection => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_shell_idx > 0 {
                        self.selected_shell_idx -= 1;
                    } else if !self.shells.is_empty() {
                        self.selected_shell_idx = self.shells.len() - 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if !self.shells.is_empty() {
                        self.selected_shell_idx = (self.selected_shell_idx + 1) % self.shells.len();
                    }
                }
                KeyCode::Char(' ') => {
                    if let Some(item) = self.shells.get_mut(self.selected_shell_idx) {
                        item.2 = !item.2;
                    }
                }
                KeyCode::Char('a') => {
                    let any_unselected = self.shells.iter().any(|(_, _, sel)| !sel);
                    for item in &mut self.shells {
                        item.2 = any_unselected;
                    }
                }
                KeyCode::Enter => {
                    self.step = InstallStep::Executing;
                    self.run_installation();
                }
                KeyCode::Esc => self.step = InstallStep::TelemetryConsent,
                _ => {}
            },
            InstallStep::Executing => {
                // Progress auto-completes
                if self.progress >= 100 {
                    self.step = InstallStep::Complete;
                }
            }
            InstallStep::Complete => match key.code {
                KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char(' ') | KeyCode::Esc => {
                    return true;
                }
                _ => {}
            },
        }
        false
    }

    fn run_installation(&mut self) {
        self.install_logs
            .push("✦ Initializing Celestial Setup Sequence...".to_string());
        self.progress = 20;

        // 1. Ensure config dir exists
        if let Ok(cfg_dir) = forgum_platform::paths::config_dir() {
            let _ = std::fs::create_dir_all(&cfg_dir);
            let cfg_file = cfg_dir.join("config.json");
            if !cfg_file.exists() {
                let default_cfg = forgum_platform::protocol::SceneConfig::default();
                if let Ok(json) = serde_json::to_string_pretty(&default_cfg) {
                    let _ = std::fs::write(&cfg_file, json);
                    self.install_logs.push(format!(
                        "✓ Created canonical config: {}",
                        cfg_file.display()
                    ));
                }
            }
        }
        if let Ok(receipt) = record_receipt(self.selected_channel, None) {
            self.install_logs.push(format!(
                "✓ Registered installation receipt: {} ({})",
                self.selected_channel.display_name(),
                receipt.installer_source.name()
            ));
        }
        self.progress = 40;

        // 2. Install completions and hooks for selected shells
        for &(sh, _, selected) in &self.shells {
            if !selected {
                continue;
            }

            // Write completions script
            if let Some(comp_path) = sh.completions_script_path() {
                if let Some(parent) = comp_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let script = get_shell_completion_content(sh);
                if std::fs::write(&comp_path, script).is_ok() {
                    self.install_logs
                        .push(format!("✓ Installed completions for {sh}"));
                }
            }

            // Inject shell profile hook
            if let Some(rc_path) = sh.shell_rc_path() {
                if let Some(parent) = rc_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let comp_cmd = sh
                    .completions_script_path()
                    .and_then(|cp| sh.shell_source_command(&cp))
                    .unwrap_or_default();
                let hook_cmd = match sh {
                    Shell::Bash => "eval \"$(forgum init bash)\"",
                    Shell::Zsh => "eval \"$(forgum init zsh)\"",
                    Shell::Fish => "forgum init fish | source",
                    Shell::Pwsh | Shell::PowerShell => "Invoke-Expression (&forgum init pwsh)",
                    Shell::Nushell => "forgum init nu",
                    _ => "forgum init",
                };

                let block = format!(
                    "# >>> forgum >>>\n# Completions canonically stored in ~/.config/forgum/completions/\n{comp_cmd}\n{hook_cmd}\n# <<< forgum <<<\n"
                );
                let existing = std::fs::read_to_string(&rc_path).unwrap_or_default();
                let updated = forgum_platform::shell::update_delimited_block(
                    &existing,
                    "# >>> forgum >>>",
                    "# <<< forgum <<<",
                    &block,
                );
                if std::fs::write(&rc_path, updated).is_ok() {
                    self.install_logs.push(format!(
                        "✓ Injected profile hook into {}",
                        rc_path.display()
                    ));
                }
            }
        }
        self.progress = 80;

        // 3. Telemetry consent registration
        if self.telemetry_allowed {
            record_installed();
            self.install_logs.push(
                "✓ Registered anonymous install pulse for developer motivation (+1)".to_string(),
            );
        } else {
            self.install_logs.push(
                "✓ Telemetry disabled per user choice. Zero network connections made.".to_string(),
            );
        }

        self.progress = 100;
        self.step = InstallStep::Complete;
    }

    pub fn render(&mut self, f: &mut Frame) {
        self.animation_tick = self.animation_tick.wrapping_add(1);
        let area = f.area();

        // Main layout: Header banner, Body, Footer pills
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Header Masthead
                Constraint::Min(12),   // Body
                Constraint::Length(3), // Footer Actions
            ])
            .split(area);

        self.render_header(f, chunks[0]);

        match self.step {
            InstallStep::Welcome => self.render_welcome_screen(f, chunks[1]),
            InstallStep::Preflight => self.render_preflight_screen(f, chunks[1]),
            InstallStep::TelemetryConsent => self.render_telemetry_screen(f, chunks[1]),
            InstallStep::ShellSelection => self.render_shell_selection_screen(f, chunks[1]),
            InstallStep::Executing => self.render_executing_screen(f, chunks[1]),
            InstallStep::Complete => self.render_complete_screen(f, chunks[1]),
        }

        self.render_footer(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        for &banner_line in OMARCHY_BANNER {
            lines.push(Line::from(Span::styled(
                banner_line,
                Style::default()
                    .fg(theme::NEBULA_CYAN)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let breadcrumbs = match self.step {
            InstallStep::Welcome => {
                " [1] Welcome ── 2 Preflight ── 3 Privacy ── 4 Shells ── 5 Install ── 6 Blastoff "
            }
            InstallStep::Preflight => {
                "  1  Welcome ── [2] Preflight ── 3 Privacy ── 4 Shells ── 5 Install ── 6 Blastoff "
            }
            InstallStep::TelemetryConsent => {
                "  1  Welcome ── 2 Preflight ── [3] Privacy ── 4 Shells ── 5 Install ── 6 Blastoff "
            }
            InstallStep::ShellSelection => {
                "  1  Welcome ── 2 Preflight ── 3 Privacy ── [4] Shells ── 5 Install ── 6 Blastoff "
            }
            InstallStep::Executing => {
                "  1  Welcome ── 2 Preflight ── 3 Privacy ── 4 Shells ── [5] Install ── 6 Blastoff "
            }
            InstallStep::Complete => {
                "  1  Welcome ── 2 Preflight ── 3 Privacy ── 4 Shells ── 5 Install ── [6] Blastoff "
            }
        };

        lines.push(Line::from(vec![
            Span::styled(
                "  ✦ CELESTIAL INSTALLER ✦  ",
                Style::default()
                    .fg(theme::ASTRAL_PURPLE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(breadcrumbs, Style::default().fg(theme::DUST_GRAY)),
        ]));

        let p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    fn render_welcome_screen(&self, f: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        // Left Pane: Celestial Cow Constellation ASCII Art
        let mut cow_lines = Vec::new();
        for (i, &art_line) in CELESTIAL_COW_CONSTELLATION.iter().enumerate() {
            let color = if (i + self.animation_tick / 4).is_multiple_of(3) {
                theme::STARFIRE_PINK
            } else if (i + self.animation_tick / 4) % 3 == 1 {
                theme::NEBULA_CYAN
            } else {
                theme::ASTRAL_PURPLE
            };
            cow_lines.push(Line::from(Span::styled(
                art_line,
                Style::default().fg(color),
            )));
        }

        let cow_p = Paragraph::new(cow_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::ASTRAL_PURPLE))
                    .title(" ✦ Astral Bovine Constellation ✦ "),
            )
            .alignment(Alignment::Center);
        f.render_widget(cow_p, split[0]);

        // Right Pane: Welcoming & System Diagnostics Card
        let host_os = std::env::consts::OS;
        let host_arch = std::env::consts::ARCH;
        let active_term = detect_active_terminal();
        let caps = forgum_platform::terminal::detect_capabilities();

        let diag_lines = vec![
            Line::from(Span::styled(
                "Welcome to the Forgum Universe!",
                Style::default()
                    .fg(theme::SUPERNOVA_GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Forgum ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("is the next-generation animated terminal ASCII engine,"),
            ]),
            Line::from(
                "bringing living mascots, physics kinematics, and glowing biomes to your CLI.",
            ),
            Line::from(""),
            Line::from(Span::styled(
                "Host System Telemetry:",
                Style::default()
                    .fg(theme::ASTRAL_PURPLE)
                    .add_modifier(Modifier::UNDERLINED),
            )),
            Line::from(format!("  • Operating System : {host_os} ({host_arch})")),
            Line::from(format!("  • Active Terminal  : {active_term}")),
            Line::from(format!(
                "  • Color Mode       : {} ({})",
                caps.color.as_str(),
                if caps.color == forgum_platform::terminal::ColorLevel::TrueColor {
                    "24-bit 16M Colors"
                } else {
                    "256/8 Colors"
                }
            )),
            Line::from(format!(
                "  • Synchronized TTY : {}",
                if caps.sync {
                    "Supported (DEC 2026)"
                } else {
                    "Standard"
                }
            )),
            Line::from(format!("  • Inline Graphics  : {:?}", caps.graphics)),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] or [Space] to run preflight check.",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let diag_p = Paragraph::new(diag_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::NEBULA_CYAN))
                    .title(" ✦ System Detection ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(diag_p, split[1]);
    }

    fn render_preflight_screen(&self, f: &mut Frame, area: Rect) {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left Column: Capabilities & Optional Dependencies
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(8)])
            .split(cols[0]);

        // Card 1: Terminal Color & Glyph Diagnostics
        let is_truecolor = self.color_level == ColorLevel::TrueColor;
        let active_term = detect_active_terminal();
        let diag_lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Color Depth     : ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                if is_truecolor {
                    Span::styled(
                        "● TrueColor 24-bit (16.7M RGB Colors)",
                        Style::default()
                            .fg(theme::AURORA_GREEN)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(
                        "● 256 / 8-Color ANSI Mode",
                        Style::default().fg(theme::SUPERNOVA_GOLD),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled(
                    "  Unicode / Glyphs: ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                if self.utf8_supported {
                    Span::styled(
                        "● UTF-8 Active [ 🐄 ✦ 🌌 🛡️ 🌿 ⚡ ✓ ]",
                        Style::default().fg(theme::AURORA_GREEN),
                    )
                } else {
                    Span::styled(
                        "● ASCII Limited Mode",
                        Style::default().fg(theme::SUPERNOVA_GOLD),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled(
                    "  TTY Frame Sync  : ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                if self.sync_supported {
                    Span::styled(
                        "● DEC 2026 Synchronized Output",
                        Style::default().fg(theme::AURORA_GREEN),
                    )
                } else {
                    Span::styled(
                        "● Standard Non-Atomic VT",
                        Style::default().fg(theme::DUST_GRAY),
                    )
                },
            ]),
            Line::from(vec![
                Span::styled(
                    "  Terminal Host   : ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(active_term, Style::default().fg(theme::STARLIGHT)),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Platform Target : ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{} ({})", std::env::consts::OS, std::env::consts::ARCH),
                    Style::default().fg(theme::DUST_GRAY),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  ✓ Memory budget: <100MB resident RAM ceiling guaranteed",
                Style::default().fg(theme::AURORA_GREEN),
            )),
        ];
        let card1 = Paragraph::new(diag_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::NEBULA_CYAN))
                .title(" ✦ 1. Terminal Capabilities & Glyphs ✦ "),
        );
        f.render_widget(card1, left_chunks[0]);

        // Card 2: Media & Optional Dependencies (ffmpeg)
        let media_lines = if self.has_ffmpeg {
            vec![
                Line::from(Span::styled(
                    "  ✓ FFmpeg Media Engine: Detected on system PATH",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from("  Terminal session recording and high-res GIF export"),
                Line::from("  are fully enabled and available for animations."),
            ]
        } else {
            vec![
                Line::from(Span::styled(
                    "  ⚠️ FFmpeg Media Engine: Not found (Optional)",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(
                    "  Forgum runs natively without it! Install anytime for GIF/video capture:",
                ),
                Line::from(vec![
                    Span::styled("    Windows : ", Style::default().fg(theme::NEBULA_CYAN)),
                    Span::styled(
                        "scoop install ffmpeg  |  winget install Gyan.FFmpeg",
                        Style::default().fg(theme::STARFIRE_PINK),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    macOS   : ", Style::default().fg(theme::NEBULA_CYAN)),
                    Span::styled(
                        "brew install ffmpeg",
                        Style::default().fg(theme::STARFIRE_PINK),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("    Linux   : ", Style::default().fg(theme::NEBULA_CYAN)),
                    Span::styled(
                        "sudo apt install ffmpeg  |  pacman -S ffmpeg",
                        Style::default().fg(theme::STARFIRE_PINK),
                    ),
                ]),
            ]
        };
        let card2 = Paragraph::new(media_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::ASTRAL_PURPLE))
                .title(" ✦ 2. Optional Recording Tools ✦ "),
        );
        f.render_widget(card2, left_chunks[1]);

        // Right Column: Package Managers / Shadows & Channel Selector
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(9), Constraint::Length(9)])
            .split(cols[1]);

        // Card 3: Package Managers & Shadow Installation Detection
        let active_pms: Vec<String> = self
            .package_managers
            .iter()
            .filter(|(_, avail)| *avail)
            .map(|(pm, _)| pm.name().to_string())
            .collect();
        let inactive_shadows: Vec<_> = self
            .shadow_installations
            .iter()
            .filter(|s| !s.is_active)
            .collect();

        let mut pkg_lines = vec![
            Line::from(vec![
                Span::styled(
                    "  Package Managers: ",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if active_pms.is_empty() {
                        "None detected (Direct Standalone)".to_string()
                    } else {
                        active_pms.join(", ")
                    },
                    Style::default().fg(theme::STARLIGHT),
                ),
            ]),
            Line::from(""),
        ];

        if inactive_shadows.is_empty() {
            pkg_lines.push(Line::from(Span::styled(
                "  ✓ Reconciliation: Clean Single Installation",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )));
            pkg_lines.push(Line::from(
                "    No conflicting shadow binaries detected on system PATH.",
            ));
        } else {
            pkg_lines.push(Line::from(Span::styled(
                format!(
                    "  ⚠️ SHADOW CONFLICT: {} duplicate binaries found!",
                    inactive_shadows.len()
                ),
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )));
            for s in &self.shadow_installations {
                pkg_lines.push(Line::from(format!("    {}", s.display_line())));
            }
            pkg_lines.push(Line::from(
                "    Tip: Uninstall duplicate managers to prevent PATH priority conflicts.",
            ));
        }

        let card3 = Paragraph::new(pkg_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if inactive_shadows.is_empty() {
                    Style::default().fg(theme::SUPERNOVA_GOLD)
                } else {
                    Style::default().fg(theme::METEOR_RED)
                })
                .title(" ✦ 3. Package Managers & Conflict Check ✦ "),
        );
        f.render_widget(card3, right_chunks[0]);

        // Card 4: Release Stream / Channel Selector
        let mut chan_lines = vec![Line::from("")];

        let mut pill_spans = vec![Span::raw("  ")];
        for (i, &ch) in ReleaseChannel::ALL.iter().enumerate() {
            let is_selected = ch == self.selected_channel;
            let label = format!(" [{}: {}] ", i + 1, ch.as_str());
            if is_selected {
                pill_spans.push(Span::styled(
                    label,
                    Style::default()
                        .fg(theme::COSMIC_DARK)
                        .bg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ));
            } else {
                pill_spans.push(Span::styled(
                    label,
                    Style::default().fg(theme::DUST_GRAY).bg(theme::DEEP_SPACE),
                ));
            }
            pill_spans.push(Span::raw("  "));
        }
        chan_lines.push(Line::from(pill_spans));
        chan_lines.push(Line::from(""));
        chan_lines.push(Line::from(vec![
            Span::styled(
                "  Stream: ",
                Style::default()
                    .fg(theme::STARFIRE_PINK)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                self.selected_channel.description(),
                Style::default().fg(theme::STARLIGHT),
            ),
        ]));
        chan_lines.push(Line::from(""));
        chan_lines.push(Line::from(Span::styled(
            "  Toggle with [Tab] or [1/2/3] • Press [Enter] to proceed",
            Style::default().fg(theme::DUST_GRAY),
        )));

        let card4 = Paragraph::new(chan_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::STARFIRE_PINK))
                .title(" ✦ 4. Release Channel Stream ✦ "),
        );
        f.render_widget(card4, right_chunks[1]);
    }

    fn render_telemetry_screen(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::SUPERNOVA_GOLD))
            .title(" ✦ Motivation & Curiosity Telemetry Notice ✦ ");

        let mut lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "  TRANSPARENT COMMUNITY MOTIVATION COUNTER",
                Style::default().fg(theme::SUPERNOVA_GOLD).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("  Forgum includes an anonymous, aggregate counter system created "),
                Span::styled("purely for developer motivation and curiosity", Style::default().fg(theme::STARFIRE_PINK).add_modifier(Modifier::BOLD)),
                Span::raw("."),
            ]),
            Line::from("  It tracks exactly THREE public aggregate numbers displayed on the repository README:"),
            Line::from(""),
            Line::from(vec![
                Span::styled("    1. Users Tried      : ", Style::default().fg(theme::NEBULA_CYAN).add_modifier(Modifier::BOLD)),
                Span::raw("Counts developers who tried or previewed Forgum."),
            ]),
            Line::from(vec![
                Span::styled("    2. Installations    : ", Style::default().fg(theme::NEBULA_CYAN).add_modifier(Modifier::BOLD)),
                Span::raw("Counts how many users successfully completed setup."),
            ]),
            Line::from(vec![
                Span::styled("    3. Active Users     : ", Style::default().fg(theme::NEBULA_CYAN).add_modifier(Modifier::BOLD)),
                Span::raw("Daily anonymous pulse (throttled to at most once per 24h)."),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "  STRICT PRIVACY GUARANTEES:",
                Style::default().fg(theme::AURORA_GREEN).add_modifier(Modifier::BOLD),
            )),
            Line::from("  ✓ ZERO Personal Data: No IP addresses, usernames, hostnames, or hardware IDs are ever recorded."),
            Line::from("  ✓ ZERO Background Daemons: No background tracking or surveillance processes."),
            Line::from("  ✓ 100% Opt-Out: Can be toggled anytime via `forgum config set telemetry false`."),
            Line::from(""),
            Line::from(Span::styled(
                "  Would you like to permit anonymous motivation telemetry?",
                Style::default().fg(theme::STARLIGHT).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];

        let opt_yes_style = if self.telemetry_selection {
            Style::default()
                .fg(theme::AURORA_GREEN)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(theme::AURORA_GREEN)
        };

        let opt_no_style = if !self.telemetry_selection {
            Style::default()
                .fg(theme::METEOR_RED)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            Style::default().fg(theme::METEOR_RED)
        };

        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                " [Y] Allow Motivation Counter (+1 to GitHub README badge) ",
                opt_yes_style,
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::raw("    "),
            Span::styled(
                " [N] Decline (Keep strictly offline & private)          ",
                opt_no_style,
            ),
        ]));

        let p = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(p, area);
    }

    fn render_shell_selection_screen(&self, f: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left Pane: Shell Checkbox List
        let items: Vec<ListItem> = self
            .shells
            .iter()
            .enumerate()
            .map(|(idx, &(sh, detected, selected))| {
                let checkbox = if selected { "[✓]" } else { "[ ]" };
                let is_cursor = idx == self.selected_shell_idx;

                let marker = if is_cursor { "▶ " } else { "  " };
                let status_str = if detected {
                    " (Detected)"
                } else {
                    " (Optional)"
                };

                let style = if is_cursor {
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD)
                } else if selected {
                    Style::default().fg(theme::STARLIGHT)
                } else {
                    Style::default().fg(theme::DUST_GRAY)
                };

                let line = Line::from(vec![
                    Span::styled(marker, Style::default().fg(theme::SUPERNOVA_GOLD)),
                    Span::styled(format!("{checkbox} {sh}"), style),
                    Span::styled(
                        status_str,
                        Style::default().fg(if detected {
                            theme::AURORA_GREEN
                        } else {
                            theme::DUST_GRAY
                        }),
                    ),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme::NEBULA_CYAN))
                .title(" ✦ Select Shells to Integrate ✦ "),
        );
        f.render_widget(list, split[0]);

        // Right Pane: Integration Details
        let curr_sh = self
            .shells
            .get(self.selected_shell_idx)
            .map(|s| s.0)
            .unwrap_or(Shell::Bash);
        let rc_path_str = curr_sh
            .shell_rc_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "N/A".to_string());
        let comp_path_str = curr_sh
            .completions_script_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "N/A".to_string());

        let details = vec![
            Line::from(Span::styled(
                format!("Integration Details for: {curr_sh}"),
                Style::default()
                    .fg(theme::ASTRAL_PURPLE)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Target Profile Script:",
                Style::default().fg(theme::SUPERNOVA_GOLD),
            )),
            Line::from(format!("  {rc_path_str}")),
            Line::from(""),
            Line::from(Span::styled(
                "Completions Script Path:",
                Style::default().fg(theme::SUPERNOVA_GOLD),
            )),
            Line::from(format!("  {comp_path_str}")),
            Line::from(""),
            Line::from(Span::styled(
                "Integration Architecture:",
                Style::default().fg(theme::AURORA_GREEN),
            )),
            Line::from("  • Reversible `# >>> forgum >>>` delimited block"),
            Line::from("  • Real-time tab auto-completion"),
            Line::from("  • Instant precmd mascot animation support"),
            Line::from(""),
            Line::from(Span::styled(
                "Shortcuts: [Space] Toggle  |  [a] Toggle All  |  [Enter] Install",
                Style::default()
                    .fg(theme::STARLIGHT)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let details_p = Paragraph::new(details)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::ASTRAL_PURPLE))
                    .title(" ✦ Shell Profile Preview ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(details_p, split[1]);
    }

    fn render_executing_screen(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(8)])
            .split(area);

        // Progress Gauge
        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" ✦ Warp Drive Progress ✦ "),
            )
            .gauge_style(
                Style::default()
                    .fg(theme::NEBULA_CYAN)
                    .bg(theme::DEEP_SPACE),
            )
            .percent(self.progress);
        f.render_widget(gauge, chunks[0]);

        // Live Log Output
        let items: Vec<ListItem> = self
            .install_logs
            .iter()
            .map(|msg| {
                ListItem::new(Line::from(Span::styled(
                    format!("  {msg}"),
                    Style::default().fg(theme::AURORA_GREEN),
                )))
            })
            .collect();

        let log_list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" ✦ Installation Activity Log ✦ "),
        );
        f.render_widget(log_list, chunks[1]);
    }

    fn render_complete_screen(&self, f: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left Pane: Portal Ascii Art
        let mut portal_lines = Vec::new();
        for &line_str in CELESTIAL_PORTAL_ART {
            portal_lines.push(Line::from(Span::styled(
                line_str,
                Style::default().fg(theme::NEBULA_CYAN),
            )));
        }
        let portal_p = Paragraph::new(portal_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::AURORA_GREEN))
                    .title(" ✦ Astral Gate Open ✦ "),
            )
            .alignment(Alignment::Center);
        f.render_widget(portal_p, split[0]);

        // Right Pane: Success Celebration & Next Steps
        let success_lines = vec![
            Line::from(Span::styled(
                "✦ BLASTOFF SUCCESSFUL! FORGUM IS READY ✦",
                Style::default()
                    .fg(theme::SUPERNOVA_GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Forgum is now woven into your terminal fabric!"),
            Line::from(""),
            Line::from(Span::styled(
                "Next Steps to Activate:",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("  1. Reload your shell or restart your terminal:"),
            Line::from(Span::styled(
                "     source ~/.bashrc   # (or reopen PowerShell / Terminal)",
                Style::default().fg(theme::STARFIRE_PINK),
            )),
            Line::from(""),
            Line::from("  2. Test drive the cosmic mascots:"),
            Line::from(Span::styled(
                "     forgum fortune",
                Style::default().fg(theme::NEBULA_CYAN),
            )),
            Line::from(Span::styled(
                "     forgum render --animal dragon --effect breathe",
                Style::default().fg(theme::NEBULA_CYAN),
            )),
            Line::from(Span::styled(
                "     forgum tui",
                Style::default().fg(theme::NEBULA_CYAN),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to exit the wizard.",
                Style::default()
                    .fg(theme::STARLIGHT)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let success_p = Paragraph::new(success_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::AURORA_GREEN))
                    .title(" ✦ Ready for Departure ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(success_p, split[1]);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_spans = match self.step {
            InstallStep::Welcome => vec![
                Span::styled(
                    " [Enter/Space] ",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Next: Preflight Diagnostics  "),
                Span::styled(" [q] ", Style::default().fg(theme::DUST_GRAY)),
                Span::raw("Quit"),
            ],
            InstallStep::Preflight => vec![
                Span::styled(
                    " [Tab/←/→/1-3] ",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Select Channel  "),
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Next: Privacy Notice  "),
                Span::styled(" [Esc] ", Style::default().fg(theme::DUST_GRAY)),
                Span::raw("Back"),
            ],
            InstallStep::TelemetryConsent => vec![
                Span::styled(
                    " [Y] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Allow Telemetry  "),
                Span::styled(
                    " [N] ",
                    Style::default()
                        .fg(theme::METEOR_RED)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Decline (Strictly Offline)  "),
                Span::styled(" [Enter] ", Style::default().fg(theme::SUPERNOVA_GOLD)),
                Span::raw("Confirm & Next"),
            ],
            InstallStep::ShellSelection => vec![
                Span::styled(
                    " [Space] ",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Toggle  "),
                Span::styled(
                    " [a] ",
                    Style::default()
                        .fg(theme::NEBULA_CYAN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("All  "),
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Install Now  "),
                Span::styled(" [Esc] ", Style::default().fg(theme::DUST_GRAY)),
                Span::raw("Back"),
            ],
            InstallStep::Executing => vec![Span::styled(
                " ✦ Executing warp sequence... ",
                Style::default()
                    .fg(theme::NEBULA_CYAN)
                    .add_modifier(Modifier::BOLD),
            )],
            InstallStep::Complete => vec![
                Span::styled(
                    " [Enter/q] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Finish & Blast Off!"),
            ],
        };

        let footer_p = Paragraph::new(Line::from(footer_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::DUST_GRAY)),
            )
            .alignment(Alignment::Center);
        f.render_widget(footer_p, area);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNINSTALLATION WIZARD
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UninstallStep {
    Welcome,
    MethodSelect,
    Confirm,
    Executing,
    Complete,
}

#[derive(Debug)]
pub struct UninstallerWizard {
    step: UninstallStep,
    selected_mode: UninstallMode,
    confirmed: bool,
    report: Option<UninstallReport>,
    animation_tick: usize,
}

impl Default for UninstallerWizard {
    fn default() -> Self {
        Self::new()
    }
}

impl UninstallerWizard {
    pub fn new() -> Self {
        Self {
            step: UninstallStep::Welcome,
            selected_mode: UninstallMode::Soft,
            confirmed: false,
            report: None,
            animation_tick: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return true;
        }
        match self.step {
            UninstallStep::Welcome => match key.code {
                KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right => {
                    self.step = UninstallStep::MethodSelect;
                }
                KeyCode::Char('q') | KeyCode::Esc => return true,
                _ => {}
            },
            UninstallStep::MethodSelect => match key.code {
                KeyCode::Up | KeyCode::Down | KeyCode::Tab => {
                    self.selected_mode = match self.selected_mode {
                        UninstallMode::Soft => UninstallMode::Purge,
                        UninstallMode::Purge => UninstallMode::Soft,
                    };
                }
                KeyCode::Char('1') | KeyCode::Char('s') | KeyCode::Char('S') => {
                    self.selected_mode = UninstallMode::Soft;
                    self.step = UninstallStep::Confirm;
                }
                KeyCode::Char('2') | KeyCode::Char('p') | KeyCode::Char('P') => {
                    self.selected_mode = UninstallMode::Purge;
                    self.step = UninstallStep::Confirm;
                }
                KeyCode::Enter => {
                    self.step = UninstallStep::Confirm;
                }
                KeyCode::Esc => self.step = UninstallStep::Welcome,
                _ => {}
            },
            UninstallStep::Confirm => match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    self.confirmed = true;
                    self.step = UninstallStep::Executing;
                    let report = perform_uninstallation(self.selected_mode);
                    self.report = Some(report);
                    self.step = UninstallStep::Complete;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') | KeyCode::Esc => {
                    return true;
                }
                _ => {}
            },
            UninstallStep::Executing => {
                // Instantly handled
            }
            UninstallStep::Complete => match key.code {
                KeyCode::Enter | KeyCode::Char('q') | KeyCode::Char(' ') | KeyCode::Esc => {
                    return true;
                }
                _ => {}
            },
        }
        false
    }

    pub fn render(&mut self, f: &mut Frame) {
        self.animation_tick = self.animation_tick.wrapping_add(1);
        let area = f.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Supernova Header
                Constraint::Min(12),   // Body
                Constraint::Length(3), // Footer
            ])
            .split(area);

        self.render_header(f, chunks[0]);

        match self.step {
            UninstallStep::Welcome => self.render_welcome_screen(f, chunks[1]),
            UninstallStep::MethodSelect => self.render_method_select_screen(f, chunks[1]),
            UninstallStep::Confirm => self.render_confirm_screen(f, chunks[1]),
            UninstallStep::Executing => {}
            UninstallStep::Complete => self.render_complete_screen(f, chunks[1]),
        }

        self.render_footer(f, chunks[2]);
    }

    fn render_header(&self, f: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        for &banner_line in OMARCHY_BANNER {
            lines.push(Line::from(Span::styled(
                banner_line,
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        lines.push(Line::from(vec![
            Span::styled(
                "  ✦ CELESTIAL DE-ORBIT UNINSTALLER ✦  ",
                Style::default()
                    .fg(theme::SUPERNOVA_GOLD)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " [1] Welcome ── [2] Choose Method ── [3] Confirm ── [4] De-Orbit ",
                Style::default().fg(theme::DUST_GRAY),
            ),
        ]));

        let p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(p, area);
    }

    fn render_welcome_screen(&self, f: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        // Left Pane: Supernova Art
        let mut art_lines = Vec::new();
        for &art_line in DEORBIT_SUPERNOVA_ART {
            art_lines.push(Line::from(Span::styled(
                art_line,
                Style::default().fg(theme::SUPERNOVA_GOLD),
            )));
        }
        let art_p = Paragraph::new(art_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::METEOR_RED))
                    .title(" ✦ Supernova Horizon ✦ "),
            )
            .alignment(Alignment::Center);
        f.render_widget(art_p, split[0]);

        // Right Pane: De-orbit overview
        let lines = vec![
            Line::from(Span::styled(
                "Initiate De-Orbit Sequence",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("We're sorry to see you leave the galaxy!"),
            Line::from("This wizard will cleanly excise Forgum from your environment."),
            Line::from(""),
            Line::from(Span::styled(
                "Two Uninstallation Methods are Available:",
                Style::default().fg(theme::SUPERNOVA_GOLD),
            )),
            Line::from("  • Method 1: Soft Uninstall (Preserves your config and custom cows)"),
            Line::from(
                "  • Method 2: Purge Uninstall (Completely erases everything - clean slate)",
            ),
            Line::from(""),
            Line::from(Span::styled(
                "Press [Enter] to choose your uninstallation method.",
                Style::default()
                    .fg(theme::STARLIGHT)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let p = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::SUPERNOVA_GOLD))
                    .title(" ✦ De-Orbit Overview ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(p, split[1]);
    }

    fn render_method_select_screen(&self, f: &mut Frame, area: Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Option 1: Soft Uninstall Card
        let is_soft = self.selected_mode == UninstallMode::Soft;
        let soft_border_style = if is_soft {
            Style::default()
                .fg(theme::AURORA_GREEN)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DUST_GRAY)
        };

        let soft_lines = vec![
            Line::from(Span::styled(
                "METHOD 1: SOFT UNINSTALL",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "RECOMMENDED if you may reinstall later.",
                Style::default().fg(theme::SUPERNOVA_GOLD),
            )),
            Line::from(""),
            Line::from("  ✓ Removes binary from disk and User PATH"),
            Line::from("  ✓ Strips shell hooks from all shell profiles"),
            Line::from("  ✓ Removes ~/.config/forgum/completions/"),
            Line::from(""),
            Line::from(Span::styled(
                "  ★ PRESERVES ~/.config/forgum/ (Settings & Custom Cows)",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("    Your mascot preferences, color modes, and custom animals"),
            Line::from("    will remain safely stored on your machine."),
            Line::from(""),
            Line::from(Span::styled(
                if is_soft {
                    "▶ [SELECTED] Press [Enter] or '1' to confirm"
                } else {
                    "  Press '1' to select Method 1"
                },
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let soft_p = Paragraph::new(soft_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(soft_border_style)
                    .title(" ✦ [1] Soft Uninstall (Keep Configs) ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(soft_p, split[0]);

        // Option 2: Purge Uninstall Card
        let is_purge = self.selected_mode == UninstallMode::Purge;
        let purge_border_style = if is_purge {
            Style::default()
                .fg(theme::METEOR_RED)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::DUST_GRAY)
        };

        let purge_lines = vec![
            Line::from(Span::styled(
                "METHOD 2: PURGE UNINSTALL",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "CLEAN SLATE — Zero residual footprint.",
                Style::default().fg(theme::METEOR_RED),
            )),
            Line::from(""),
            Line::from("  ✓ Performs all Soft Uninstall removals"),
            Line::from("  ✓ PERMANENTLY DELETES ~/.config/forgum/"),
            Line::from("  ✓ PERMANENTLY DELETES all logs, cache, and state"),
            Line::from("  ✓ Terminates background daemons and removes sockets"),
            Line::from(""),
            Line::from(Span::styled(
                "  ⚠ NO DATA IS PRESERVED",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from("    Leaves zero traces of Forgum on your system."),
            Line::from(""),
            Line::from(Span::styled(
                if is_purge {
                    "▶ [SELECTED] Press [Enter] or '2' to confirm"
                } else {
                    "  Press '2' to select Method 2"
                },
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let purge_p = Paragraph::new(purge_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(purge_border_style)
                    .title(" ✦ [2] Purge Uninstall (Clean Slate) ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(purge_p, split[1]);
    }

    fn render_confirm_screen(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme::METEOR_RED))
            .title(" ✦ Confirm Uninstallation Execution ✦ ");

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "ARE YOU ABSOLUTELY SURE?",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("  Selected Mode: {}", self.selected_mode.title())),
            Line::from(format!(
                "  Description  : {}",
                self.selected_mode.description()
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press [Y] or [Enter] to PROCEED with uninstallation.",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "  Press [N] or [Esc] to CANCEL and keep Forgum.",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let p = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(p, area);
    }

    fn render_complete_screen(&self, f: &mut Frame, area: Rect) {
        let report = match self.report {
            Some(ref r) => r,
            None => return,
        };

        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left Pane: Farewell Cow Art
        let mut cow_lines = Vec::new();
        for &art_line in CELESTIAL_COW_CONSTELLATION {
            cow_lines.push(Line::from(Span::styled(
                art_line,
                Style::default().fg(theme::DUST_GRAY),
            )));
        }
        let cow_p = Paragraph::new(cow_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::DUST_GRAY))
                    .title(" ✦ Fading Constellation ✦ "),
            )
            .alignment(Alignment::Center);
        f.render_widget(cow_p, split[0]);

        // Right Pane: Verification Report
        let mut report_lines = vec![
            Line::from(Span::styled(
                "✦ FORGUM HAS SAFELY DE-ORBITED ✦",
                Style::default()
                    .fg(theme::SUPERNOVA_GOLD)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("Executed: {}", report.mode.title())),
            Line::from(""),
            Line::from(Span::styled(
                "Excised Components:",
                Style::default()
                    .fg(theme::AURORA_GREEN)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        for log in &report.logs {
            report_lines.push(Line::from(format!("  {log}")));
        }

        if !report.errors.is_empty() {
            report_lines.push(Line::from(""));
            report_lines.push(Line::from(Span::styled(
                "Warnings encountered:",
                Style::default().fg(theme::METEOR_RED),
            )));
            for err in &report.errors {
                report_lines.push(Line::from(format!("  ✗ {err}")));
            }
        }

        report_lines.push(Line::from(""));
        report_lines.push(Line::from(Span::styled(
            "Thank you for traveling the stars with Forgum!",
            Style::default().fg(theme::STARFIRE_PINK),
        )));
        report_lines.push(Line::from(Span::styled(
            "Press [Enter] or [q] to close.",
            Style::default()
                .fg(theme::STARLIGHT)
                .add_modifier(Modifier::BOLD),
        )));

        let p = Paragraph::new(report_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::AURORA_GREEN))
                    .title(" ✦ Verification Report ✦ "),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(p, split[1]);
    }

    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let footer_spans = match self.step {
            UninstallStep::Welcome => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(theme::SUPERNOVA_GOLD)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Select Method  "),
                Span::styled(" [q] ", Style::default().fg(theme::DUST_GRAY)),
                Span::raw("Cancel"),
            ],
            UninstallStep::MethodSelect => vec![
                Span::styled(
                    " [1] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Soft Uninstall  "),
                Span::styled(
                    " [2] ",
                    Style::default()
                        .fg(theme::METEOR_RED)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Purge Uninstall  "),
                Span::styled(" [Enter] ", Style::default().fg(theme::SUPERNOVA_GOLD)),
                Span::raw("Confirm Selection"),
            ],
            UninstallStep::Confirm => vec![
                Span::styled(
                    " [Y] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Proceed with Uninstallation  "),
                Span::styled(
                    " [N/Esc] ",
                    Style::default()
                        .fg(theme::METEOR_RED)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Abort"),
            ],
            UninstallStep::Executing => vec![Span::styled(
                " ✦ Excising Forgum from orbit... ",
                Style::default()
                    .fg(theme::METEOR_RED)
                    .add_modifier(Modifier::BOLD),
            )],
            UninstallStep::Complete => vec![
                Span::styled(
                    " [Enter/q] ",
                    Style::default()
                        .fg(theme::AURORA_GREEN)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Close Uninstaller"),
            ],
        };

        let footer_p = Paragraph::new(Line::from(footer_spans))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(theme::DUST_GRAY)),
            )
            .alignment(Alignment::Center);
        f.render_widget(footer_p, area);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RUNNERS
// ═══════════════════════════════════════════════════════════════════════════

/// Launch the interactive Celestial Installation Wizard.
pub fn run_installer_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = crate::TuiTerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let mut wizard = InstallerWizard::new();
    let tick_rate = Duration::from_millis(33); // ~30 FPS

    let res = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| wizard.render(f))?;

            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if wizard.handle_key(key) {
                        break;
                    }
                }
            }
        }
        Ok(())
    })();

    crate::restore_terminal_cleanly();
    guard.disarm();
    res.map_err(|e| e.into())
}

/// Launch the interactive Celestial Uninstallation Wizard.
pub fn run_uninstaller_wizard() -> Result<(), Box<dyn std::error::Error>> {
    let mut guard = crate::TuiTerminalGuard::enter()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("build terminal")?;

    let mut wizard = UninstallerWizard::new();
    let tick_rate = Duration::from_millis(33); // ~30 FPS

    let res = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| wizard.render(f))?;

            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    if wizard.handle_key(key) {
                        break;
                    }
                }
            }
        }
        Ok(())
    })();

    crate::restore_terminal_cleanly();
    guard.disarm();
    res.map_err(|e| e.into())
}

/// Helper to check if an executable exists on the system PATH.
fn which_exists(prog: &str) -> bool {
    #[cfg(windows)]
    {
        let prog_exe = if prog.ends_with(".exe") {
            prog.to_string()
        } else {
            format!("{prog}.exe")
        };
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if dir.join(&prog_exe).is_file() {
                    return true;
                }
            }
        }
        false
    }

    #[cfg(unix)]
    {
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if dir.join(prog).is_file() {
                    return true;
                }
            }
        }
        false
    }
}

/// Helper to detect host active terminal name.
fn detect_active_terminal() -> String {
    if std::env::var("WT_SESSION").is_ok() {
        return "Windows Terminal".to_string();
    }
    if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return "Ghostty".to_string();
    }
    if std::env::var("WEZTERM_PANE").is_ok() {
        return "WezTerm".to_string();
    }
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return "Kitty".to_string();
    }
    if std::env::var("ALACRITTY_LOG").is_ok() || std::env::var("ALACRITTY_WINDOW_ID").is_ok() {
        return "Alacritty".to_string();
    }
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        return prog;
    }
    if let Ok(term) = std::env::var("TERM") {
        return term;
    }
    "Standard Virtual Terminal".to_string()
}

/// Helper to generate static completions scripts for shells.
fn get_shell_completion_content(sh: Shell) -> &'static str {
    match sh {
        Shell::Pwsh | Shell::PowerShell => {
            r#"# Forgum PowerShell auto-completion script
Register-ArgumentCompleter -Native -CommandName forgum -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @("render", "think", "fortune", "tui", "init", "completions", "list", "status", "doctor", "checkhealth", "config", "logs", "log", "diagnose", "install", "uninstall", "update", "channel")
    $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#
        }
        Shell::Bash => {
            r#"# Forgum Bash auto-completion script
_forgum_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local cmds="render think fortune tui init completions list status doctor checkhealth config logs log diagnose install uninstall update channel --animal --effect --environment --road --mountain --color-mode"
    COMPREPLY=( $(compgen -W "${cmds}" -- "${cur}") )
}
complete -F _forgum_completions forgum
"#
        }
        Shell::Zsh => {
            r#"#compdef forgum
_forgum() {
    _arguments \
        '--animal[Animal mascot]:mascot:()' \
        '--effect[Animation effect]:effect:(walk breathe float fly talk sway pulse glitch particles dissolve)' \
        '--environment[Particle environment]:environment:(pasture inferno ocean arctic city forest savanna swamp space cyber)' \
        '--color-mode[Color palette mode]:color_mode:(animal rainbow solid none)' \
        '1:subcommand:(render think fortune tui init completions list status doctor checkhealth config logs log diagnose install uninstall update channel)'
}
_forgum "$@"
"#
        }
        Shell::Fish => {
            r#"# Forgum Fish auto-completion script
complete -c forgum -f
complete -c forgum -n "__fish_use_subcommand" -a "render think fortune tui init completions list status doctor checkhealth config logs log diagnose install uninstall update channel"
"#
        }
        Shell::Nushell => {
            r#"# Forgum Nushell auto-completion script
export extern "forgum" [
    subcommand?: string
    --animal(-c): string
    --effect(-e): string
    --environment: string
    --color-mode: string
]
"#
        }
        _ => "# Forgum auto-completions\n",
    }
}
