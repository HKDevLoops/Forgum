//! Comprehensive health check subsystem for Forgum (`forgum checkhealth`).
//!
//! Modeled after Neovim's `:checkhealth`, this module inspects the complete
//! runtime environment, terminal emulator, shell integration, configuration,
//! cow templates, DNA profiles, IPC sockets, and logging subsystem.

use std::fs;
use std::path::{Path, PathBuf};

use forgum_platform::{
    detect_available_package_managers, detect_installation_source, is_telemetry_allowed,
    ALL_FORGUM_MARKER_PAIRS,
};
use serde::{Deserialize, Serialize};

use crate::init::Shell;

/// Status of an individual health check item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HealthStatus {
    Ok,
    Warn,
    Error,
    Info,
}

impl HealthStatus {
    #[must_use]
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Ok => "\x1b[1;32m[OK]\x1b[0m",
            Self::Warn => "\x1b[1;33m[WARNING]\x1b[0m",
            Self::Error => "\x1b[1;31m[ERROR]\x1b[0m",
            Self::Info => "\x1b[1;36m[INFO]\x1b[0m",
        }
    }
}

/// An individual health check item with diagnosis and optional remediation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthItem {
    pub status: HealthStatus,
    pub title: String,
    pub details: Vec<String>,
    pub suggestion: Option<String>,
}

/// A section grouping related health checks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSection {
    pub name: String,
    pub items: Vec<HealthItem>,
}

/// Complete health check report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub version: String,
    pub timestamp: String,
    pub sections: Vec<HealthSection>,
    pub ok_count: usize,
    pub warn_count: usize,
    pub error_count: usize,
    pub total_count: usize,
}

impl HealthReport {
    /// Format the health report into a styled Neovim-style ANSI terminal output.
    #[must_use]
    pub fn format_ansi(&self) -> String {
        let mut out = String::new();

        out.push_str("\n\x1b[1;35m==============================================================================\x1b[0m\n");
        out.push_str(&format!(
            "\x1b[1mforgum checkhealth\x1b[0m — Engine v{} ({})\n",
            self.version, self.timestamp
        ));
        out.push_str("\x1b[1;35m==============================================================================\x1b[0m\n");

        for section in &self.sections {
            out.push_str(&format!("\n\x1b[1;34m## {}\x1b[0m\n", section.name));
            for item in &section.items {
                out.push_str(&format!("  {} {}\n", item.status.badge(), item.title));
                for d in &item.details {
                    out.push_str(&format!("    \x1b[90m•\x1b[0m {}\n", d));
                }
                if let Some(sug) = &item.suggestion {
                    out.push_str(&format!("    \x1b[1;33m- SUGGESTION:\x1b[0m {}\n", sug));
                }
            }
        }

        out.push_str("\n\x1b[1;35m------------------------------------------------------------------------------\x1b[0m\n");
        let score_color = if self.error_count > 0 {
            "\x1b[1;31m"
        } else if self.warn_count > 0 {
            "\x1b[1;33m"
        } else {
            "\x1b[1;32m"
        };
        out.push_str(&format!(
            "Health Summary: {}{} passed\x1b[0m, {} warnings, {} errors ({} checks total)\n",
            score_color, self.ok_count, self.warn_count, self.error_count, self.total_count
        ));

        if self.error_count == 0 && self.warn_count == 0 {
            out.push_str("\x1b[1;32m✔ All pasture systems are fully operational!\x1b[0m\n\n");
        } else {
            out.push_str("\x1b[90mRun `forgum config --tui` or check suggested actions above to optimize.\x1b[0m\n\n");
        }

        out
    }
}

/// Run all health checks and construct a complete `HealthReport`.
pub fn run_health_check(explicit_config: Option<&Path>) -> HealthReport {
    let mut sections = Vec::new();
    let mut ok_count = 0;
    let mut warn_count = 0;
    let mut error_count = 0;
    let mut total_count = 0;

    // ── 1. System & Platform Environment ──────────────────────────────
    let mut sys_items = Vec::new();
    sys_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!(
            "OS & Architecture: {} ({})",
            std::env::consts::OS,
            std::env::consts::ARCH
        ),
        details: vec![
            format!("Process PID: {}", std::process::id()),
            format!("Family: {}", std::env::consts::FAMILY),
        ],
        suggestion: None,
    });

    let engine_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("forgum"));
    sys_items.push(HealthItem {
        status: if engine_exe.is_file() {
            HealthStatus::Ok
        } else {
            HealthStatus::Warn
        },
        title: format!("Engine Binary: {}", engine_exe.display()),
        details: vec![
            format!("Version: {}", crate::VERSION),
            format!("Executable exists on disk: {}", engine_exe.is_file()),
        ],
        suggestion: if engine_exe.is_file() {
            None
        } else {
            Some("Ensure forgum is placed in a directory on your system PATH.".into())
        },
    });

    let source_pm = detect_installation_source();
    let available_pms = detect_available_package_managers();
    let active_pms: Vec<String> = available_pms
        .iter()
        .filter(|(_, avail)| *avail)
        .map(|(pm, _)| pm.name().to_string())
        .collect();

    sys_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!("Installation Source: {}", source_pm.name()),
        details: vec![
            format!("Origin: {}", source_pm.name()),
            format!("Upgrade Command: `{}`", source_pm.update_command()),
            format!("Uninstall Command: `{}`", source_pm.uninstall_command()),
        ],
        suggestion: None,
    });

    sys_items.push(HealthItem {
        status: if active_pms.is_empty() {
            HealthStatus::Info
        } else {
            HealthStatus::Ok
        },
        title: format!("Host Package Managers: {} detected active", active_pms.len()),
        details: vec![
            format!(
                "Active Managers: {}",
                if active_pms.is_empty() {
                    "None (standalone host)".to_string()
                } else {
                    active_pms.join(", ")
                }
            ),
            "Supported ecosystems: Scoop, WinGet, Chocolatey, Homebrew, Pacman, APT, DNF, Zypper, Nix, APK, XBPS, Gentoo, MacPorts, FreeBSD pkg, Cargo".to_string(),
        ],
        suggestion: None,
    });

    sections.push(HealthSection {
        name: "System & Platform Environment".into(),
        items: sys_items,
    });

    // ── 2. Configuration Subsystem & Tri-Format Integrity ─────────────
    let mut cfg_items = Vec::new();
    match forgum_platform::detect_config_file(explicit_config) {
        Ok((path, fmt)) => {
            if path.is_file() {
                let config_res = crate::config::read_config_file(&path);
                match config_res {
                    Ok(cfg) => {
                        cfg_items.push(HealthItem {
                            status: HealthStatus::Ok,
                            title: format!(
                                "Active Configuration: {} ({})",
                                path.display(),
                                fmt.display_name()
                            ),
                            details: vec![
                                format!("Active Cow: '{}'", cfg.cow),
                                format!("Active Effect: '{}'", cfg.effect),
                                format!("FPS: {}", cfg.fps),
                                format!("Shell Attach Mode: '{}'", cfg.shell_attach_mode),
                                format!("Auto Render on Prompt: {}", cfg.auto_render_on_prompt),
                                format!("Color Mode: '{}'", cfg.color_mode),
                            ],
                            suggestion: None,
                        });
                    }
                    Err(e) => {
                        cfg_items.push(HealthItem {
                            status: HealthStatus::Error,
                            title: format!("Configuration syntax error in {}", path.display()),
                            details: vec![format!("Parse error: {e}")],
                            suggestion: Some(format!(
                                "Run `forgum config --tui` or delete {} to restore defaults.",
                                path.display()
                            )),
                        });
                    }
                }
            } else {
                cfg_items.push(HealthItem {
                    status: HealthStatus::Info,
                    title: format!("Configuration: Built-in defaults active (no file at {})", path.display()),
                    details: vec![
                        "Default Cow: 'default'".into(),
                        "Default Effect: 'static'".into(),
                        "Default FPS: 30".into(),
                        "Default Shell Attach: 'banner'".into(),
                    ],
                    suggestion: Some("Run `forgum config --tui` to generate your initial config file in JSON, YAML, or TOML.".into()),
                });
            }
        }
        Err(forgum_platform::PlatformError::ConfigConflict(paths)) => {
            cfg_items.push(HealthItem {
                status: HealthStatus::Error,
                title: "Multiple conflicting configuration formats detected on disk".into(),
                details: paths.iter().map(|p| format!("Conflicting file: {}", p.display())).collect(),
                suggestion: Some("Forgum enforces single-format exclusivity. Run `forgum config --migrate json` (or toml/yaml) to resolve.".into()),
            });
        }
        Err(e) => {
            cfg_items.push(HealthItem {
                status: HealthStatus::Warn,
                title: "No custom configuration found (using built-in defaults)".into(),
                details: vec![format!("Resolution note: {e}")],
                suggestion: Some("Run `forgum config --tui` to generate your initial config file in JSON, YAML, or TOML.".into()),
            });
        }
    }

    if let Ok(cfg_dir) = forgum_platform::config_dir() {
        let is_writable = fs::create_dir_all(&cfg_dir).is_ok();
        cfg_items.push(HealthItem {
            status: if is_writable {
                HealthStatus::Ok
            } else {
                HealthStatus::Error
            },
            title: format!("Config Directory Permissions: {}", cfg_dir.display()),
            details: vec![format!("Writable: {}", is_writable)],
            suggestion: if is_writable {
                None
            } else {
                Some(format!(
                    "Check file permissions on directory {}",
                    cfg_dir.display()
                ))
            },
        });
    }

    sections.push(HealthSection {
        name: "Configuration & Multi-Format Subsystem".into(),
        items: cfg_items,
    });

    // ── 3. Terminal Emulator & Display Capabilities ───────────────────
    let mut term_items = Vec::new();
    let caps = forgum_platform::detect_capabilities();
    let mux = forgum_platform::detect_mux();

    let term_name = std::env::var("TERM_PROGRAM")
        .or_else(|_| std::env::var("TERM"))
        .unwrap_or_else(|_| "unknown".into());

    term_items.push(HealthItem {
        status: if caps.is_tty {
            HealthStatus::Ok
        } else {
            HealthStatus::Info
        },
        title: format!(
            "Terminal Grid: {}x{} (TTY: {}, Program: {})",
            caps.width, caps.height, caps.is_tty, term_name
        ),
        details: vec![
            format!("Columns: {}, Rows: {}", caps.width, caps.height),
            format!(
                "Sufficient viewport for pasture: {}",
                caps.width >= 40 && caps.height >= 12
            ),
        ],
        suggestion: if caps.width < 40 || caps.height < 12 {
            Some(
                "Expand your terminal window to at least 40x15 for optimal ASCII art rendering."
                    .into(),
            )
        } else {
            None
        },
    });

    let has_truecolor = caps.color == forgum_platform::ColorLevel::TrueColor;
    term_items.push(HealthItem {
        status: if has_truecolor { HealthStatus::Ok } else { HealthStatus::Warn },
        title: format!("Color Capabilities: {}", caps.color.as_str()),
        details: vec![
            format!("24-bit TrueColor RGB: {}", has_truecolor),
            format!(
                "Detection Source: {}",
                if std::env::var("COLORTERM").is_ok() {
                    "COLORTERM environment variable"
                } else if caps.emulator.supports_truecolor() {
                    "Natively supported by detected terminal emulator"
                } else {
                    "Fallback profile / TERM default"
                }
            ),
        ],
        suggestion: if has_truecolor {
            None
        } else {
            Some("Set `COLORTERM=truecolor` in your terminal or profile for full 16.7M TrueColor palette rendering.".into())
        },
    });

    term_items.push(HealthItem {
        status: HealthStatus::Info,
        title: format!(
            "Multiplexer & Sync Protocol: Mux='{}', Synchronized Updates={}",
            mux.name(),
            caps.sync
        ),
        details: vec![
            format!("Multiplexer detected: {}", mux.name()),
            format!(
                "DEC Mode 2026 Sync Update: {}",
                if caps.sync {
                    "supported"
                } else {
                    "conservative fallback"
                }
            ),
            format!("Graphics Protocol: {:?}", caps.graphics),
        ],
        suggestion: None,
    });

    let decstbm_supported = caps.emulator.supports_decstbm();
    let native_split_available = caps.emulator.native_split_available() || mux.is_active();
    term_items.push(HealthItem {
        status: if decstbm_supported {
            HealthStatus::Ok
        } else if matches!(caps.emulator, forgum_platform::TerminalEmulator::Dumb) {
            HealthStatus::Info
        } else {
            HealthStatus::Warn
        },
        title: format!(
            "Split Shell Adapter: Emulator='{}', Mode='{}'",
            caps.emulator.name(),
            caps.split_mode.as_str()
        ),
        details: vec![
            format!("DECSTBM hardware margins: {}", if decstbm_supported { "supported" } else { "unsupported" }),
            format!("DECSLRM horizontal margins: {}", if caps.emulator.supports_decslrm() { "supported" } else { "unsupported" }),
            format!("Native Split API: {}", if native_split_available { "available" } else { "none (standard DECSTBM margin split)" }),
        ],
        suggestion: caps.emulator.limitation_notes().map(String::from),
    });

    sections.push(HealthSection {
        name: "Terminal Capabilities & Color Protocols".into(),
        items: term_items,
    });

    // ── 4. Cow Pasture Assets & DNA Registry ──────────────────────────
    let mut cow_items = Vec::new();
    let data_dir = forgum_platform::data_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cows_dir = data_dir.join("Cows");
    let cow_count = if cows_dir.is_dir() {
        fs::read_dir(&cows_dir)
            .map(|rd| rd.filter_map(|e| e.ok()).count())
            .unwrap_or(0)
    } else {
        0
    };

    cow_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!(
            "Cow Pasture Assets: {} cow files available",
            if cow_count > 0 { cow_count } else { 109 }
        ),
        details: vec![
            format!("Data Directory: {}", data_dir.display()),
            "Embedded fallback cows: 109+ creatures compiled into binary".to_string(),
        ],
        suggestion: None,
    });

    let dna_registry = crate::dna::load_animations(&data_dir);
    let dna_count = if dna_registry.is_empty() {
        10
    } else {
        dna_registry.len()
    };
    cow_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!("DNA Signature Animation Profiles: {} registered", dna_count),
        details: vec![
            "Compound signature animations supported: thoracic breathing, float, walk, tail-wag, particle emitters".into(),
            "Specialized signatures: Dragon (fire), Dolphin (ocean bubbles), Koala (zzz sleep), Nyan (stars)".into(),
        ],
        suggestion: None,
    });

    sections.push(HealthSection {
        name: "Pasture Assets & 2D Compound Signatures".into(),
        items: cow_items,
    });

    // ── 5. Shell Integration & Prompt Attach ──────────────────────────
    let mut shell_items = Vec::new();
    let detected_shell = std::env::var("SHELL")
        .or_else(|_| std::env::var("PSModulePath").map(|_| "pwsh".to_string()))
        .unwrap_or_else(|_| "powershell".to_string());

    let shell_kind = if detected_shell.contains("pwsh") || detected_shell.contains("PowerShell") {
        Shell::Pwsh
    } else if detected_shell.contains("zsh") {
        Shell::Zsh
    } else if detected_shell.contains("fish") {
        Shell::Fish
    } else {
        Shell::Bash
    };

    shell_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!("Active Shell Environment: {:?}", shell_kind),
        details: vec![
            format!("Shell Type: {:?}", shell_kind),
            format!("Shell hook generator: `forgum init {:?}` available", shell_kind),
        ],
        suggestion: Some(format!("Add `forgum init {:?} | Out-String | Invoke-Expression` (or eval for bash/zsh) to your shell profile.", shell_kind)),
    });

    // Probe all 15 supported shells
    let mut integrated_shells = Vec::new();
    let mut present_shells = Vec::new();

    for &sh in Shell::ALL {
        if let Some(rc) = sh.shell_rc_path() {
            if rc.exists() {
                present_shells.push(sh);
                if let Ok(content) = fs::read_to_string(&rc) {
                    let has_hook = ALL_FORGUM_MARKER_PAIRS
                        .iter()
                        .any(|(b, _)| content.contains(b));
                    if has_hook {
                        integrated_shells.push(sh);
                    }
                }
            }
        }
    }

    shell_items.push(HealthItem {
        status: if !integrated_shells.is_empty() {
            HealthStatus::Ok
        } else {
            HealthStatus::Warn
        },
        title: format!(
            "Universal Shell Ecosystem: {} of {} detected shells integrated",
            integrated_shells.len(),
            present_shells.len()
        ),
        details: vec![
            format!(
                "Integrated Shells: {}",
                if integrated_shells.is_empty() {
                    "None".to_string()
                } else {
                    integrated_shells
                        .iter()
                        .map(|s| format!("{s}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
            format!(
                "Configured Profiles on Host: {}",
                if present_shells.is_empty() {
                    "None".to_string()
                } else {
                    present_shells
                        .iter()
                        .map(|s| format!("{s}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                }
            ),
            "Supported shells: Bash, Zsh, Fish, Pwsh, PowerShell, Cmd, Elvish, Nushell, Carapace, Xonsh, Tcsh, Ksh, Ion, Oil, Yash".into(),
        ],
        suggestion: if integrated_shells.is_empty() {
            Some(format!(
                "Run `forgum init {:?}` to install prompt hook, or run `forgum install` for interactive setup.",
                shell_kind
            ))
        } else {
            None
        },
    });

    sections.push(HealthSection {
        name: "Shell Integration & Prompt Hooks".into(),
        items: shell_items,
    });

    // ── 6. Daemon & IPC Socket Subsystem ──────────────────────────────
    let mut ipc_items = Vec::new();
    let daemons = crate::herd::discover_daemons();
    ipc_items.push(HealthItem {
        status: HealthStatus::Ok,
        title: format!("Background Daemons: {} running", daemons.len()),
        details: vec![
            format!("Active Daemons: {}", daemons.len()),
            "Dead daemon auto-sweep: active on prompt precmd".into(),
        ],
        suggestion: if daemons.len() > 10 {
            Some(
                "Many daemons are active. Run `forgum herd stop --all` to clean up old instances."
                    .into(),
            )
        } else {
            None
        },
    });

    sections.push(HealthSection {
        name: "Daemon & IPC Subsystem".into(),
        items: ipc_items,
    });

    // ── 7. Structured Logging Subsystem ───────────────────────────────
    let mut log_items = Vec::new();
    if let Some((dir, text, jsonl)) = crate::logger::get_log_paths() {
        let text_size = text.metadata().map(|m| m.len()).unwrap_or(0);
        let jsonl_size = jsonl.metadata().map(|m| m.len()).unwrap_or(0);

        let recent_logs = crate::logger::read_recent_logs(100, None).unwrap_or_default();
        let diag = crate::logger::diagnose_logs(&recent_logs);

        let status = if diag.errors_count > 0 {
            HealthStatus::Error
        } else if diag.warnings_count > 0 || !diag.uprising_bugs.is_empty() {
            HealthStatus::Warn
        } else {
            HealthStatus::Ok
        };

        let mut details = vec![
            format!("Text Log: {} ({} KB)", text.display(), text_size / 1024),
            format!("JSONL Log: {} ({} KB)", jsonl.display(), jsonl_size / 1024),
            format!(
                "Recent Events: {} errors, {} warnings across {} logs",
                diag.errors_count, diag.warnings_count, diag.total_logs
            ),
        ];
        for bug in &diag.uprising_bugs {
            details.push(format!(
                "Bug Radar: {} [{}] - {}",
                bug.pattern, bug.threat_level, bug.suggested_fix
            ));
        }

        let suggestion = if !diag.user_action_items.is_empty() {
            Some(format!(
                "Triage: {}. Run `forgum logs --diagnose` for full report or `forgum logs --clear` to reset history.",
                diag.user_action_items[0]
            ))
        } else if diag.errors_count > 0 {
            Some("Historical errors found in log sample. Run `forgum logs --clear` to clear stale historical logs.".into())
        } else if text_size > 10 * 1024 * 1024 {
            Some("Logs exceed 10 MB. Run `forgum logs --clear` to truncate old events.".into())
        } else {
            None
        };

        log_items.push(HealthItem {
            status,
            title: format!("Logging & Anomaly Diagnostics: {}", dir.display()),
            details,
            suggestion,
        });
    }

    // Telemetry & Motivation Diagnostics
    let telem_active = is_telemetry_allowed();
    log_items.push(HealthItem {
        status: HealthStatus::Info,
        title: format!(
            "Motivation Telemetry: {}",
            if telem_active {
                "Active (Opted-in)"
            } else {
                "Disabled (Opted-out)"
            }
        ),
        details: vec![
            format!(
                "Status: {}",
                if telem_active { "Enabled" } else { "Disabled" }
            ),
            "Philosophy: Anonymous motivation counter (users_tried, users_installed, active_pulse)"
                .into(),
            "Privacy Guarantees: ZERO personal data, ZERO IP logging, ZERO background daemons"
                .into(),
            "Controls: Toggle via `FORGUM_TELEMETRY=0` or `forgum config`".into(),
        ],
        suggestion: None,
    });

    sections.push(HealthSection {
        name: "Structured Logging & Traceability".into(),
        items: log_items,
    });

    // ── Compute Statistics ────────────────────────────────────────────
    for s in &sections {
        for it in &s.items {
            total_count += 1;
            match it.status {
                HealthStatus::Ok => ok_count += 1,
                HealthStatus::Warn => warn_count += 1,
                HealthStatus::Error => error_count += 1,
                HealthStatus::Info => ok_count += 1,
            }
        }
    }

    HealthReport {
        version: crate::VERSION.to_string(),
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        sections,
        ok_count,
        warn_count,
        error_count,
        total_count,
    }
}
