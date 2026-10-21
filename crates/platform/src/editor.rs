//! Text editor detection and interactive viewer spawning across platforms.
//!
//! Provides first-class support for Neovim as the preferred default editor
//! for structured logs, with intelligent discovery across standard install
//! locations (Chocolatey, Scoop, Homebrew, Apt, Pacman, Windows Program Files).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Metadata describing an editor discovered on the host system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditorInfo {
    pub name: &'static str,
    pub command: &'static str,
    pub path: PathBuf,
    pub is_terminal: bool,
}

/// Candidates probed in priority order if Neovim is not present.
pub const EDITOR_CANDIDATES: &[(&str, &str, bool)] = &[
    ("Neovim", "nvim", true),
    ("Vim", "vim", true),
    ("Helix", "hx", true),
    ("Nano", "nano", true),
    ("VS Code", "code", false),
    ("Sublime Text", "subl", false),
    ("Notepad++", "notepad++", false),
    ("Micro", "micro", true),
    #[cfg(windows)]
    ("Windows Notepad", "notepad", false),
];

/// Check whether an executable exists in PATH or standard install locations.
#[must_use]
pub fn command_exists(cmd: &str) -> bool {
    find_editor_binary(cmd).is_some()
}

/// Probe whether an executable exists in PATH or standard system install locations.
pub fn find_editor_binary(cmd: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let query = if cmd.ends_with(".exe") || cmd.ends_with(".cmd") {
            cmd.to_string()
        } else {
            format!("{cmd}.exe")
        };

        // 0. Fast in-memory PATH scan (0.05ms, zero subprocesses)
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let p1 = dir.join(&query);
                if p1.is_file() {
                    return Some(p1);
                }
                let p2 = dir.join(cmd);
                if p2.is_file() {
                    return Some(p2);
                }
            }
        }

        // 1. Try 'where.exe <cmd>' fallback
        if let Ok(output) = Command::new("where.exe").arg(&query).output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Some(first_line) = text.lines().next() {
                        let p = PathBuf::from(first_line.trim());
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        // Also try without .exe if query had it
        if let Ok(output) = Command::new("where.exe").arg(cmd).output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Some(first_line) = text.lines().next() {
                        let p = PathBuf::from(first_line.trim());
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        // 2. Standard hardcoded paths for popular editors on Windows
        let standard_candidates: &[&str] = match cmd {
            "nvim" | "nvim.exe" => &[
                r"C:\Program Files\Neovim\bin\nvim.exe",
                r"C:\Program Files (x86)\Neovim\bin\nvim.exe",
                r"C:\tools\neovim\Neovim\bin\nvim.exe",
                r"C:\ProgramData\chocolatey\bin\nvim.exe",
            ],
            "vim" | "vim.exe" => &[
                r"C:\Program Files\Vim\vim91\vim.exe",
                r"C:\Program Files\Vim\vim90\vim.exe",
                r"C:\Program Files (x86)\Vim\vim91\vim.exe",
                r"C:\Program Files\Git\usr\bin\vim.exe",
            ],
            "code" | "code.cmd" => &[
                r"C:\Program Files\Microsoft VS Code\bin\code.cmd",
                r"C:\Program Files\Microsoft VS Code\Code.exe",
            ],
            "notepad++" | "notepad++.exe" => &[
                r"C:\Program Files\Notepad++\notepad++.exe",
                r"C:\Program Files (x86)\Notepad++\notepad++.exe",
            ],
            "notepad" | "notepad.exe" => &[
                r"C:\Windows\System32\notepad.exe",
                r"C:\Windows\notepad.exe",
            ],
            _ => &[],
        };

        for &raw_path in standard_candidates {
            let p = PathBuf::from(raw_path);
            if p.is_file() {
                return Some(p);
            }
        }

        // 3. User profile & LocalAppData locations (Scoop / User installs)
        if let Ok(user_profile) = std::env::var("USERPROFILE") {
            let base = PathBuf::from(user_profile);
            let scoop_path = base
                .join("scoop")
                .join("apps")
                .join(cmd)
                .join("current")
                .join("bin")
                .join(format!("{cmd}.exe"));
            if scoop_path.is_file() {
                return Some(scoop_path);
            }
            let scoop_shims = base.join("scoop").join("shims").join(format!("{cmd}.exe"));
            if scoop_shims.is_file() {
                return Some(scoop_shims);
            }
        }

        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            let base = PathBuf::from(local_app_data);
            if cmd == "nvim" || cmd == "nvim.exe" {
                let p = base
                    .join("Programs")
                    .join("Neovim")
                    .join("bin")
                    .join("nvim.exe");
                if p.is_file() {
                    return Some(p);
                }
            } else if cmd == "code" || cmd == "code.cmd" {
                let p = base
                    .join("Programs")
                    .join("Microsoft VS Code")
                    .join("bin")
                    .join("code.cmd");
                if p.is_file() {
                    return Some(p);
                }
            }
        }

        None
    }

    #[cfg(not(windows))]
    {
        // 0. Fast in-memory PATH scan
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let p = dir.join(cmd);
                if p.is_file() {
                    return Some(p);
                }
            }
        }

        // 1. Fallback to which
        if let Ok(output) = Command::new("which").arg(cmd).output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    if let Some(first_line) = text.lines().next() {
                        let p = PathBuf::from(first_line.trim());
                        if p.is_file() {
                            return Some(p);
                        }
                    }
                }
            }
        }

        let standard_paths = match cmd {
            "nvim" => &[
                "/usr/bin/nvim",
                "/usr/local/bin/nvim",
                "/opt/homebrew/bin/nvim",
                "/home/linuxbrew/.linuxbrew/bin/nvim",
            ][..],
            "vim" => &[
                "/usr/bin/vim",
                "/usr/local/bin/vim",
                "/opt/homebrew/bin/vim",
            ][..],
            "nano" => &["/usr/bin/nano", "/usr/local/bin/nano"][..],
            "hx" => &["/usr/bin/hx", "/usr/local/bin/hx", "/opt/homebrew/bin/hx"][..],
            "code" => &[
                "/usr/bin/code",
                "/usr/local/bin/code",
                "/opt/homebrew/bin/code",
            ][..],
            _ => &[],
        };

        for &p_str in standard_paths {
            let p = PathBuf::from(p_str);
            if p.is_file() {
                return Some(p);
            }
        }

        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(home).join(".local").join("bin").join(cmd);
            if p.is_file() {
                return Some(p);
            }
        }

        None
    }
}

/// Specifically locate Neovim on the system.
pub fn find_neovim() -> Option<EditorInfo> {
    find_editor_binary("nvim").map(|path| EditorInfo {
        name: "Neovim",
        command: "nvim",
        path,
        is_terminal: true,
    })
}

/// Detect all installed text editors available on this host.
pub fn detect_installed_editors() -> Vec<EditorInfo> {
    let mut out = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Check $VISUAL
    if let Ok(vis) = std::env::var("VISUAL") {
        let trimmed = vis.trim();
        if !trimmed.is_empty() {
            let prog = trimmed.split_whitespace().next().unwrap_or(trimmed);
            if let Some(path) = find_editor_binary(prog) {
                seen_paths.insert(path.clone());
                out.push(EditorInfo {
                    name: "User Default ($VISUAL)",
                    command: "visual",
                    path,
                    is_terminal: true,
                });
            }
        }
    }

    // 2. Check $EDITOR
    if let Ok(ed) = std::env::var("EDITOR") {
        let trimmed = ed.trim();
        if !trimmed.is_empty() {
            let prog = trimmed.split_whitespace().next().unwrap_or(trimmed);
            if let Some(path) = find_editor_binary(prog) {
                if seen_paths.insert(path.clone()) {
                    out.push(EditorInfo {
                        name: "User Default ($EDITOR)",
                        command: "editor",
                        path,
                        is_terminal: true,
                    });
                }
            }
        }
    }

    // 3. Check standard candidate editors
    for &(name, cmd, is_term) in EDITOR_CANDIDATES {
        if let Some(path) = find_editor_binary(cmd) {
            if seen_paths.insert(path.clone()) {
                out.push(EditorInfo {
                    name,
                    command: cmd,
                    path,
                    is_terminal: is_term,
                });
            }
        }
    }
    out
}

/// Open a target file in the selected editor.
pub fn open_in_editor(editor: &EditorInfo, file_path: &Path) -> std::io::Result<()> {
    if editor.is_terminal {
        let status = Command::new(&editor.path).arg(file_path).status()?;
        if !status.success() {
            return Err(std::io::Error::other(format!(
                "Editor '{}' exited with status {:?}",
                editor.name,
                status.code()
            )));
        }
        Ok(())
    } else {
        Command::new(&editor.path).arg(file_path).spawn()?;
        Ok(())
    }
}
