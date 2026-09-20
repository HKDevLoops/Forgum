//! Complete Uninstallation Engine supporting Soft and Purge methods.
//!
//! # Methods
//! - [`UninstallMode::Soft`]:
//!   Uninstalls binary from disk and PATH, excises shell hooks and completions,
//!   deletes completions scripts, but PRESERVES user configurations (`~/.config/forgum/`),
//!   custom mascot cows, themes, and preferences for future reinstallation.
//!
//! - [`UninstallMode::Purge`]:
//!   Clean slate removal. Performs all Soft actions, and permanently removes
//!   configuration, cache, logs, data, runtime sockets, and daemon states.

use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::process::Command;

use crate::shell::{uninstall_all_shell_integrations, Shell};

/// Uninstallation operational mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UninstallMode {
    /// Soft uninstall: remove binary and hooks, keep user configs and custom cows.
    Soft,
    /// Purge uninstall: clean slate, delete binary, hooks, configs, cache, and logs.
    Purge,
}

impl UninstallMode {
    pub fn title(self) -> &'static str {
        match self {
            UninstallMode::Soft => "Soft Uninstall (Preserve Configurations & Custom Cows)",
            UninstallMode::Purge => "Purge Uninstall (Clean Slate — Remove Everything)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            UninstallMode::Soft => {
                "Removes Forgum from PATH, strips shell hooks and completions, \
                 and removes binaries, but retains ~/.config/forgum so your settings \
                 and custom mascots remain intact."
            }
            UninstallMode::Purge => {
                "Completely erases all Forgum files, binaries, shell hooks, \
                 completions, configuration directory, logs, cache, and runtime daemons."
            }
        }
    }
}

/// Comprehensive report generated after uninstallation.
#[derive(Debug, Clone)]
pub struct UninstallReport {
    pub mode: UninstallMode,
    pub shells_cleaned: Vec<(Shell, bool)>,
    pub completions_removed: bool,
    pub path_cleaned: bool,
    pub binary_removed: bool,
    pub config_removed: bool,
    pub data_removed: bool,
    pub logs_removed: bool,
    pub runtime_removed: bool,
    pub logs: Vec<String>,
    pub errors: Vec<String>,
}

/// Execute uninstallation according to the specified mode.
pub fn perform_uninstallation(mode: UninstallMode) -> UninstallReport {
    let mut logs = Vec::new();
    let mut errors = Vec::new();

    // 1. Excising shell integrations across all 15 supported shells
    let mut shells_cleaned = Vec::new();
    for (sh, res) in uninstall_all_shell_integrations() {
        match res {
            Ok(true) => {
                logs.push(format!("✓ Removed hooks & completions from {sh}"));
                shells_cleaned.push((sh, true));
            }
            Ok(false) => {
                shells_cleaned.push((sh, false));
            }
            Err(e) => {
                errors.push(format!("Failed unhooking {sh}: {e}"));
                shells_cleaned.push((sh, false));
            }
        }
    }

    // 2. Remove completions directory (~/.config/forgum/completions)
    let completions_removed = clean_completions_dir(&mut logs);

    // 3. Remove Forgum from User PATH
    let path_cleaned = remove_from_user_path(&mut logs, &mut errors);

    // 4. Remove binaries
    let binary_removed = remove_binaries(&mut logs, &mut errors);

    // 5. Purge mode actions
    let mut config_removed = false;
    let mut data_removed = false;
    let mut logs_removed = false;
    let mut runtime_removed = false;

    if mode == UninstallMode::Purge {
        logs.push("Purge mode active: excising user configuration, data, and cache...".to_string());

        // Config directory (~/.config/forgum or %APPDATA%\Forgum)
        if let Ok(cfg_dir) = crate::paths::config_dir() {
            if cfg_dir.exists() {
                match std::fs::remove_dir_all(&cfg_dir) {
                    Ok(_) => {
                        logs.push(format!(
                            "✓ Deleted configuration directory: {}",
                            cfg_dir.display()
                        ));
                        config_removed = true;
                    }
                    Err(e) => errors.push(format!(
                        "Cannot remove config dir {}: {e}",
                        cfg_dir.display()
                    )),
                }
            }
        }

        // Data directory
        if let Ok(data_dir) = crate::paths::data_dir() {
            if data_dir.exists() {
                match std::fs::remove_dir_all(&data_dir) {
                    Ok(_) => {
                        logs.push(format!("✓ Deleted data directory: {}", data_dir.display()));
                        data_removed = true;
                    }
                    Err(e) => errors.push(format!(
                        "Cannot remove data dir {}: {e}",
                        data_dir.display()
                    )),
                }
            }
        }

        // Log directory
        if let Ok(log_dir) = crate::paths::log_dir() {
            if log_dir.exists() {
                match std::fs::remove_dir_all(&log_dir) {
                    Ok(_) => {
                        logs.push(format!("✓ Deleted log directory: {}", log_dir.display()));
                        logs_removed = true;
                    }
                    Err(e) => {
                        errors.push(format!("Cannot remove log dir {}: {e}", log_dir.display()))
                    }
                }
            }
        }

        // Runtime directory
        if let Ok(rt_dir) = crate::paths::runtime_dir() {
            if rt_dir.exists() {
                match std::fs::remove_dir_all(&rt_dir) {
                    Ok(_) => {
                        logs.push(format!("✓ Deleted runtime directory: {}", rt_dir.display()));
                        runtime_removed = true;
                    }
                    Err(e) => errors.push(format!(
                        "Cannot remove runtime dir {}: {e}",
                        rt_dir.display()
                    )),
                }
            }
        }
    } else {
        logs.push("Soft mode active: preserved configuration in ~/.config/forgum/".to_string());
    }

    UninstallReport {
        mode,
        shells_cleaned,
        completions_removed,
        path_cleaned,
        binary_removed,
        config_removed,
        data_removed,
        logs_removed,
        runtime_removed,
        logs,
        errors,
    }
}

/// Delete the completions directory ~/.config/forgum/completions/
fn clean_completions_dir(logs: &mut Vec<String>) -> bool {
    let mut removed = false;
    if let Some(comp_dir) = Shell::Bash
        .completions_script_path()
        .and_then(|p| p.parent().map(Path::to_path_buf))
    {
        if comp_dir.exists() && std::fs::remove_dir_all(&comp_dir).is_ok() {
            logs.push(format!(
                "✓ Removed completions directory: {}",
                comp_dir.display()
            ));
            removed = true;
        }
    }
    removed
}

/// Strip Forgum install directory from the User PATH environment.
fn remove_from_user_path(logs: &mut Vec<String>, errors: &mut Vec<String>) -> bool {
    #[cfg(windows)]
    {
        let install_dir = match std::env::var("LOCALAPPDATA") {
            Ok(lad) => format!("{lad}\\Forgum"),
            Err(_) => return false,
        };

        let script = format!(
            "$target = '{install_dir}'; \
             $curr = [Environment]::GetEnvironmentVariable('PATH', 'User'); \
             if ($curr) {{ \
                 $parts = $curr -split ';' | Where-Object {{ $_ -and $_.TrimEnd('\\') -ne $target.TrimEnd('\\') }}; \
                 [Environment]::SetEnvironmentVariable('PATH', ($parts -join ';'), 'User'); \
             }}"
        );

        match Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(&script)
            .status()
        {
            Ok(s) if s.success() => {
                logs.push(format!("✓ Removed {} from User PATH", install_dir));
                true
            }
            Ok(s) => {
                errors.push(format!("PowerShell PATH removal returned exit code {s}"));
                false
            }
            Err(e) => {
                errors.push(format!("Failed to invoke PowerShell for PATH cleanup: {e}"));
                false
            }
        }
    }

    #[cfg(unix)]
    {
        let _ = logs;
        let _ = errors;
        true
    }
}

/// Remove Forgum binaries from common installation paths.
fn remove_binaries(logs: &mut Vec<String>, errors: &mut Vec<String>) -> bool {
    let mut any_removed = false;

    // Collect candidate binary paths
    let mut candidate_paths: Vec<PathBuf> = Vec::new();

    #[cfg(windows)]
    {
        if let Ok(lad) = std::env::var("LOCALAPPDATA") {
            let forgum_dir = PathBuf::from(lad).join("Forgum");
            candidate_paths.push(forgum_dir.join("forgum-engine.exe"));
            candidate_paths.push(forgum_dir.join("forgum.exe"));
        }
    }

    #[cfg(unix)]
    {
        if let Ok(home) = std::env::var("HOME") {
            let home_p = PathBuf::from(home);
            candidate_paths.push(home_p.join(".local/bin/forgum-engine"));
            candidate_paths.push(home_p.join(".local/bin/forgum"));
            candidate_paths.push(home_p.join("bin/forgum-engine"));
            candidate_paths.push(home_p.join("bin/forgum"));
        }
        candidate_paths.push(PathBuf::from("/usr/local/bin/forgum-engine"));
        candidate_paths.push(PathBuf::from("/usr/local/bin/forgum"));
    }

    // Also include current running exe path
    if let Ok(curr_exe) = std::env::current_exe() {
        if !candidate_paths.contains(&curr_exe) {
            candidate_paths.push(curr_exe);
        }
    }

    for path in candidate_paths {
        if path.is_file() {
            #[cfg(windows)]
            {
                // If this is the currently executing process, rename to .del and schedule deletion
                let is_current = std::env::current_exe().map(|c| c == path).unwrap_or(false);
                if is_current {
                    let temp_del = path.with_extension("exe.del");
                    let _ = std::fs::rename(&path, &temp_del);
                    // Schedule background deletion after process termination
                    let target_str = temp_del.to_string_lossy().to_string();
                    let _ = Command::new("cmd.exe")
                        .args([
                            "/C",
                            "ping 127.0.0.1 -n 2 > nul & del /F /Q",
                            &format!("\"{target_str}\""),
                        ])
                        .spawn();
                    logs.push(format!("✓ Scheduled binary deletion: {}", path.display()));
                    any_removed = true;
                    continue;
                }
            }

            match std::fs::remove_file(&path) {
                Ok(_) => {
                    logs.push(format!("✓ Deleted binary: {}", path.display()));
                    any_removed = true;
                }
                Err(e) => {
                    errors.push(format!("Could not delete binary {}: {e}", path.display()));
                }
            }
        }
    }

    any_removed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_descriptions() {
        assert!(UninstallMode::Soft.title().contains("Soft"));
        assert!(UninstallMode::Purge.title().contains("Purge"));
        assert!(UninstallMode::Soft.description().contains(".config/forgum"));
        assert!(
            UninstallMode::Purge.description().contains("clean slate")
                || UninstallMode::Purge.description().contains("clean")
                || UninstallMode::Purge.description().contains("erases")
        );
    }
}
