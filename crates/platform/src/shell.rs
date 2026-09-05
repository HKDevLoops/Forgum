//! Supported shell types for hook generation (`forgum init <shell>`)
//! and shell completions (`forgum completions <shell>`).

use std::fmt;
use std::path::{Path, PathBuf};

/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
    Cmd,
    PowerShell,
    Elvish,
    Nushell,
    Carapace,
    Xonsh,
    Tcsh,
    Ksh,
    Ion,
    Oil,
    Yash,
}

impl Shell {
    /// Parse a shell name string.
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "pwsh" => Some(Shell::Pwsh),
            "powershell" => Some(Shell::PowerShell),
            "cmd" => Some(Shell::Cmd),
            "elvish" => Some(Shell::Elvish),
            "nu" | "nushell" => Some(Shell::Nushell),
            "carapace" => Some(Shell::Carapace),
            "xonsh" => Some(Shell::Xonsh),
            "tcsh" | "csh" => Some(Shell::Tcsh),
            "ksh" | "mksh" | "oksh" | "pdksh" => Some(Shell::Ksh),
            "ion" => Some(Shell::Ion),
            "oil" | "osh" | "ysh" => Some(Shell::Oil),
            "yash" => Some(Shell::Yash),
            _ => None,
        }
    }

    /// Default config file path for this shell.
    pub fn default_config_path(&self) -> &str {
        match self {
            Shell::Pwsh | Shell::PowerShell | Shell::Cmd | Shell::Carapace => {
                "~/.config/forgum/config.json"
            }
            _ => "$HOME/.config/forgum/config.json",
        }
    }

    /// Path to user's shell startup/rc configuration file.
    pub fn shell_rc_path(&self) -> Option<PathBuf> {
        let home = home_dir();
        match self {
            Shell::Bash => {
                #[cfg(unix)]
                {
                    if let Some(h) = &home {
                        let bash_profile = h.join(".bash_profile");
                        if bash_profile.is_file() {
                            return Some(bash_profile);
                        }
                        return Some(h.join(".bashrc"));
                    }
                }
                #[cfg(windows)]
                {
                    if let Some(h) = &home {
                        return Some(h.join(".bashrc"));
                    }
                }
                None
            }
            Shell::Zsh => home.map(|h| h.join(".zshrc")),
            Shell::Fish => {
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| c.join("fish").join("config.fish"))
                }
                #[cfg(windows)]
                {
                    appdata_dir()
                        .map(|a| a.join("fish").join("config.fish"))
                        .or_else(|| {
                            home.map(|h| h.join(".config").join("fish").join("config.fish"))
                        })
                }
            }
            Shell::Pwsh => {
                #[cfg(windows)]
                {
                    home.map(|h| {
                        h.join("Documents")
                            .join("PowerShell")
                            .join("Microsoft.PowerShell_profile.ps1")
                    })
                }
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| {
                        c.join("powershell")
                            .join("Microsoft.PowerShell_profile.ps1")
                    })
                }
            }
            Shell::PowerShell => {
                #[cfg(windows)]
                {
                    home.map(|h| {
                        h.join("Documents")
                            .join("WindowsPowerShell")
                            .join("Microsoft.PowerShell_profile.ps1")
                    })
                }
                #[cfg(unix)]
                {
                    None
                }
            }
            Shell::Elvish => {
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| c.join("elvish").join("rc.elv"))
                }
                #[cfg(windows)]
                {
                    appdata_dir().map(|a| a.join("elvish").join("rc.elv"))
                }
            }
            Shell::Nushell => {
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| c.join("nushell").join("config.nu"))
                }
                #[cfg(windows)]
                {
                    appdata_dir().map(|a| a.join("nushell").join("config.nu"))
                }
            }
            Shell::Xonsh => home.map(|h| h.join(".xonshrc")),
            Shell::Tcsh => home.map(|h| h.join(".tcshrc")),
            Shell::Ksh => home.map(|h| h.join(".kshrc")),
            Shell::Ion => {
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| c.join("ion").join("initrc"))
                }
                #[cfg(windows)]
                {
                    appdata_dir().map(|a| a.join("ion").join("initrc"))
                }
            }
            Shell::Oil => {
                #[cfg(unix)]
                {
                    xdg_config_home().map(|c| c.join("oil").join("oshrc"))
                }
                #[cfg(windows)]
                {
                    appdata_dir().map(|a| a.join("oil").join("oshrc"))
                }
            }
            Shell::Yash => home.map(|h| h.join(".yashrc")),
            Shell::Carapace | Shell::Cmd => None,
        }
    }

    /// Path where the shell completion script or specification should be installed.
    /// In accordance with Forgum standards, completions are canonically stored in
    /// `~/.config/forgum/completions/` across all operating systems.
    pub fn completions_script_path(&self) -> Option<PathBuf> {
        let canonical_dir = home_dir().map(|h| h.join(".config").join("forgum").join("completions"));
        match self {
            Shell::Bash => canonical_dir.map(|d| d.join("forgum.bash")),
            Shell::Zsh => canonical_dir.map(|d| d.join("_forgum")),
            Shell::Fish => canonical_dir.map(|d| d.join("forgum.fish")),
            Shell::Pwsh | Shell::PowerShell => canonical_dir.map(|d| d.join("forgum.ps1")),
            Shell::Elvish => canonical_dir.map(|d| d.join("forgum.elv")),
            Shell::Nushell => canonical_dir.map(|d| d.join("forgum.nu")),
            Shell::Carapace => canonical_dir.map(|d| d.join("forgum.yaml")),
            Shell::Xonsh => canonical_dir.map(|d| d.join("forgum.xsh")),
            Shell::Tcsh => canonical_dir.map(|d| d.join("forgum.csh")),
            Shell::Ksh => canonical_dir.map(|d| d.join("forgum.ksh")),
            Shell::Ion => canonical_dir.map(|d| d.join("forgum.ion")),
            Shell::Oil => canonical_dir.map(|d| d.join("forgum.oil")),
            Shell::Yash => canonical_dir.map(|d| d.join("forgum.yash")),
            Shell::Cmd => None,
        }
    }

    /// Return the shell-specific source/include command to load the completion file.
    pub fn shell_source_command(&self, script_path: &Path) -> Option<String> {
        let p = script_path.to_string_lossy();
        match self {
            Shell::Bash | Shell::Zsh | Shell::Ksh | Shell::Yash => {
                Some(format!("[ -f \"{p}\" ] && source \"{p}\""))
            }
            Shell::Fish => Some(format!("test -f \"{p}\"; and source \"{p}\"")),
            Shell::Pwsh | Shell::PowerShell => {
                Some(format!("if (Test-Path \"{p}\") {{ . \"{p}\" }}"))
            }
            Shell::Elvish => Some("use forgum".to_string()),
            Shell::Nushell => Some(format!("use \"{p}\" *")),
            Shell::Tcsh => Some(format!("if ( -f \"{p}\" ) source \"{p}\"")),
            Shell::Xonsh => Some(format!("$COMPLETIONS_CUSTOM = True\nif __import__('os').path.isfile(\"{p}\"):\n    exec(open(\"{p}\").read())")),
            Shell::Ion => Some(format!("test -f \"{p}\" && source \"{p}\"")),
            Shell::Oil => Some(format!("test -f \"{p}\" && source \"{p}\"")),
            Shell::Carapace | Shell::Cmd => None,
        }
    }
}

impl fmt::Display for Shell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::Pwsh => write!(f, "pwsh"),
            Shell::Cmd => write!(f, "cmd"),
            Shell::PowerShell => write!(f, "powershell"),
            Shell::Elvish => write!(f, "elvish"),
            Shell::Nushell => write!(f, "nushell"),
            Shell::Carapace => write!(f, "carapace"),
            Shell::Xonsh => write!(f, "xonsh"),
            Shell::Tcsh => write!(f, "tcsh"),
            Shell::Ksh => write!(f, "ksh"),
            Shell::Ion => write!(f, "ion"),
            Shell::Oil => write!(f, "oil"),
            Shell::Yash => write!(f, "yash"),
        }
    }
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Some(up) = std::env::var_os("USERPROFILE") {
            return Some(PathBuf::from(up));
        }
        if let Some(h) = std::env::var_os("HOME") {
            return Some(PathBuf::from(h));
        }
        None
    }
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
}

#[cfg(windows)]
fn appdata_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(PathBuf::from)
}

#[cfg(unix)]
fn xdg_config_home() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    home_dir().map(|h| h.join(".config"))
}

#[cfg(unix)]
fn xdg_data_home() -> Option<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(xdg));
    }
    home_dir().map(|h| h.join(".local").join("share"))
}

/// Idempotently update or append a delimited block in configuration file content.
pub fn update_delimited_block(
    content: &str,
    begin_marker: &str,
    end_marker: &str,
    new_block: &str,
) -> String {
    if let (Some(start), Some(end)) = (content.find(begin_marker), content.find(end_marker)) {
        if start <= end {
            let before = &content[..start];
            let after_marker = &content[end..];
            let after = if let Some(nl) = after_marker.find('\n') {
                &after_marker[nl + 1..]
            } else {
                ""
            };
            let mut res = String::with_capacity(before.len() + new_block.len() + after.len() + 2);
            res.push_str(before);
            res.push_str(new_block);
            if !new_block.ends_with('\n') {
                res.push('\n');
            }
            res.push_str(after);
            return res;
        }
    }
    let mut res = String::with_capacity(content.len() + new_block.len() + 4);
    res.push_str(content);
    if !content.is_empty() && !content.ends_with('\n') {
        res.push('\n');
    }
    res.push_str(new_block);
    if !new_block.ends_with('\n') {
        res.push('\n');
    }
    res
}

/// Write content to a file only if the file does not exist or its content differs.
/// Returns `Ok(true)` if the file was written, `Ok(false)` if skipped (no churn).
pub fn write_file_if_changed(path: &Path, content: &str) -> std::io::Result<bool> {
    if let Ok(existing) = std::fs::read_to_string(path) {
        if existing == content {
            return Ok(false);
        }
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_all_shells() {
        assert_eq!(Shell::parse("bash"), Some(Shell::Bash));
        assert_eq!(Shell::parse("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse("fish"), Some(Shell::Fish));
        assert_eq!(Shell::parse("pwsh"), Some(Shell::Pwsh));
        assert_eq!(Shell::parse("powershell"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("cmd"), Some(Shell::Cmd));
        assert_eq!(Shell::parse("elvish"), Some(Shell::Elvish));
        assert_eq!(Shell::parse("nushell"), Some(Shell::Nushell));
        assert_eq!(Shell::parse("nu"), Some(Shell::Nushell));
        assert_eq!(Shell::parse("carapace"), Some(Shell::Carapace));
        assert_eq!(Shell::parse("unknown_shell"), None);
    }

    #[test]
    fn update_delimited_block_appends_when_absent() {
        let content = "export PATH=$PATH:/usr/bin\n";
        let begin = "# >>> forgum >>>";
        let end = "# <<< forgum <<<";
        let block = "# >>> forgum >>>\nsource ~/.forgum\n# <<< forgum <<<";
        let updated = update_delimited_block(content, begin, end, block);
        assert!(updated.starts_with("export PATH=$PATH:/usr/bin\n"));
        assert!(updated.contains(block));
    }

    #[test]
    fn update_delimited_block_replaces_existing() {
        let content = "prefix\n# >>> forgum >>>\nold content\n# <<< forgum <<<\nsuffix\n";
        let begin = "# >>> forgum >>>";
        let end = "# <<< forgum <<<";
        let block = "# >>> forgum >>>\nnew content\n# <<< forgum <<<";
        let updated = update_delimited_block(content, begin, end, block);
        assert_eq!(
            updated,
            "prefix\n# >>> forgum >>>\nnew content\n# <<< forgum <<<\nsuffix\n"
        );
    }
}
