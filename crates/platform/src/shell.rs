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
    /// All 15 supported shell variants.
    pub const ALL: &'static [Shell] = &[
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Pwsh,
        Shell::PowerShell,
        Shell::Cmd,
        Shell::Elvish,
        Shell::Nushell,
        Shell::Carapace,
        Shell::Xonsh,
        Shell::Tcsh,
        Shell::Ksh,
        Shell::Ion,
        Shell::Oil,
        Shell::Yash,
    ];

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

    /// Detect the active host shell from environment variables and process context.
    pub fn detect_current_shell() -> Option<Self> {
        // 1. Check shell-specific unique environment markers
        if std::env::var_os("BASH_VERSION").is_some() {
            return Some(Shell::Bash);
        }
        if std::env::var_os("ZSH_VERSION").is_some() {
            return Some(Shell::Zsh);
        }
        if std::env::var_os("FISH_VERSION").is_some() {
            return Some(Shell::Fish);
        }
        if std::env::var_os("NU_VERSION").is_some() {
            return Some(Shell::Nushell);
        }
        if std::env::var_os("XONSH_VERSION").is_some() {
            return Some(Shell::Xonsh);
        }
        if std::env::var_os("KSH_VERSION").is_some() {
            return Some(Shell::Ksh);
        }
        if std::env::var_os("OIL_VERSION").is_some() || std::env::var_os("YSH_VERSION").is_some() {
            return Some(Shell::Oil);
        }
        if std::env::var_os("YASH_VERSION").is_some() {
            return Some(Shell::Yash);
        }
        if std::env::var_os("ION_SHELL").is_some() {
            return Some(Shell::Ion);
        }

        // 2. Inspect $SHELL environment variable
        if let Some(shell_env) = std::env::var_os("SHELL") {
            let p = Path::new(&shell_env);
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if let Some(sh) = Self::parse(stem) {
                    return Some(sh);
                }
            }
        }

        // 3. Inspect PowerShell / Pwsh environment indicators
        if std::env::var_os("PSModulePath").is_some() {
            #[cfg(windows)]
            {
                if std::env::var_os("POWERSHELL_DISTRIBUTION_CHANNEL").is_some() {
                    return Some(Shell::Pwsh);
                }
                return Some(Shell::PowerShell);
            }
            #[cfg(not(windows))]
            {
                return Some(Shell::Pwsh);
            }
        }

        // 4. Default fallback by OS
        #[cfg(windows)]
        {
            Some(Shell::Pwsh)
        }
        #[cfg(not(windows))]
        {
            Some(Shell::Bash)
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
                        let onedrive_profile = h
                            .join("OneDrive")
                            .join("Documents")
                            .join("PowerShell")
                            .join("Microsoft.PowerShell_profile.ps1");
                        if onedrive_profile.exists() {
                            return onedrive_profile;
                        }
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
                        let onedrive_profile = h
                            .join("OneDrive")
                            .join("Documents")
                            .join("WindowsPowerShell")
                            .join("Microsoft.PowerShell_profile.ps1");
                        if onedrive_profile.exists() {
                            return onedrive_profile;
                        }
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
        let canonical_dir =
            home_dir().map(|h| h.join(".config").join("forgum").join("completions"));
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
#[allow(dead_code)]
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

/// Universal standard marker identifiers for shell configuration blocks.
pub const HOOK_MARKER_BEGIN: &str = "# >>> forgum >>>";
pub const HOOK_MARKER_END: &str = "# <<< forgum <<<";
pub const COMPLETIONS_MARKER_BEGIN: &str = "# >>> forgum completions >>>";
pub const COMPLETIONS_MARKER_END: &str = "# <<< forgum completions <<<";

/// All known begin/end marker pairs ever used by Forgum in shell / mux configs.
pub const ALL_FORGUM_MARKER_PAIRS: &[(&str, &str)] = &[
    ("# >>> forgum >>>", "# <<< forgum <<<"),
    (
        "# >>> forgum completions >>>",
        "# <<< forgum completions <<<",
    ),
    (
        "# >>> forgum completions (bash) >>>",
        "# <<< forgum completions <<<",
    ),
    (
        "# >>> forgum completions (zsh) >>>",
        "# <<< forgum completions <<<",
    ),
    (
        "# >>> forgum completions (fish) >>>",
        "# <<< forgum completions <<<",
    ),
    (
        "# >>> forgum completions (pwsh) >>>",
        "# <<< forgum completions <<<",
    ),
    ("# >>> forgum (bash) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (zsh) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (fish) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (pwsh) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (powershell) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (nushell) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (elvish) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (carapace) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (xonsh) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (tcsh) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (ksh) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (ion) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (oil) >>>", "# <<< forgum <<<"),
    ("# >>> forgum (yash) >>>", "# <<< forgum <<<"),
    ("rem >>> forgum (cmd) >>>", "rem <<< forgum <<<"),
    ("rem >>> forgum (cmd) >>>", "rem <<< forgum (cmd) <<<"),
    ("rem >>> forgum >>>", "rem <<< forgum <<<"),
    ("# >>> forgum tmux >>>", "# <<< forgum tmux <<<"),
    ("# >>> forgum zellij >>>", "# <<< forgum zellij <<<"),
    ("-- >>> forgum wezterm >>>", "-- <<< forgum wezterm <<<"),
    ("# >>> forgum screen >>>", "# <<< forgum screen <<<"),
    ("# >>> forgum starship >>>", "# <<< forgum starship <<<"),
    ("# >>> forgum byobu >>>", "# <<< forgum byobu <<<"),
];

/// Remove a delimited block from configuration file content.
///
/// Strips all content between and including `begin_marker` and `end_marker`
/// (along with the end marker's newline). Cleans up any trailing whitespace
/// or redundant newlines left behind. Returns `(new_content, was_removed)`.
pub fn remove_delimited_block(
    content: &str,
    begin_marker: &str,
    end_marker: &str,
) -> (String, bool) {
    let mut current = content.to_string();
    let mut removed_any = false;

    while let (Some(start), Some(end)) = (current.find(begin_marker), current.find(end_marker)) {
        if start <= end {
            let before = &current[..start];
            let after_marker = &current[end..];
            let after = if let Some(nl) = after_marker.find('\n') {
                &after_marker[nl + 1..]
            } else {
                ""
            };

            let trimmed_before = before.trim_end_matches([' ', '\t', '\r']);
            let trimmed_after = after.trim_start_matches(['\r', '\n']);

            let mut next = String::with_capacity(trimmed_before.len() + trimmed_after.len() + 2);
            if !trimmed_before.is_empty() {
                next.push_str(trimmed_before);
                if !trimmed_before.ends_with('\n') {
                    next.push('\n');
                }
            }
            if !trimmed_after.is_empty() {
                next.push_str(trimmed_after);
            }
            current = next;
            removed_any = true;
        } else {
            break;
        }
    }

    (current, removed_any)
}

/// Remove all known forgum blocks from content.
pub fn remove_all_forgum_blocks(mut content: String) -> (String, bool) {
    let mut any_removed = false;
    for &(b, e) in ALL_FORGUM_MARKER_PAIRS {
        let (updated, removed) = remove_delimited_block(&content, b, e);
        if removed {
            content = updated;
            any_removed = true;
        }
    }
    (content, any_removed)
}

/// Uninstall integration and completions for a specific shell.
///
/// Returns `Ok(true)` if modifications or removals were made, `Ok(false)` if clean.
pub fn uninstall_shell_integration(shell: Shell) -> std::io::Result<bool> {
    let mut changed = false;

    // 1. Clean shell rc file
    if let Some(rc_path) = shell.shell_rc_path() {
        if rc_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&rc_path) {
                let (cleaned, removed) = remove_all_forgum_blocks(content);
                if removed {
                    std::fs::write(&rc_path, cleaned)?;
                    changed = true;
                }
            }
        }
    }

    // 2. Remove completions script
    if let Some(comp_path) = shell.completions_script_path() {
        if comp_path.exists() {
            let _ = std::fs::remove_file(&comp_path);
            changed = true;
        }
    }

    Ok(changed)
}

/// Uninstall integration and completions across all 15 supported shells.
pub fn uninstall_all_shell_integrations() -> Vec<(Shell, std::io::Result<bool>)> {
    Shell::ALL
        .iter()
        .map(|&sh| (sh, uninstall_shell_integration(sh)))
        .collect()
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

    #[test]
    fn remove_delimited_block_cleanly_excises() {
        let content = "export PATH=$PATH:/usr/bin\n# >>> forgum >>>\nsource ~/.forgum\n# <<< forgum <<<\nalias ll='ls -l'\n";
        let (cleaned, removed) =
            remove_delimited_block(content, "# >>> forgum >>>", "# <<< forgum <<<");
        assert!(removed);
        assert_eq!(cleaned, "export PATH=$PATH:/usr/bin\nalias ll='ls -l'\n");
    }

    #[test]
    fn remove_delimited_block_when_absent() {
        let content = "export PATH=$PATH:/usr/bin\n";
        let (cleaned, removed) =
            remove_delimited_block(content, "# >>> forgum >>>", "# <<< forgum <<<");
        assert!(!removed);
        assert_eq!(cleaned, content);
    }

    #[test]
    fn remove_all_forgum_blocks_handles_mixed_markers() {
        let content = "prefix\n# >>> forgum >>>\nhook\n# <<< forgum <<<\nmiddle\n# >>> forgum completions >>>\ncomp\n# <<< forgum completions <<<\nsuffix\n";
        let (cleaned, removed) = remove_all_forgum_blocks(content.to_string());
        assert!(removed);
        assert_eq!(cleaned, "prefix\nmiddle\nsuffix\n");
    }

    #[test]
    fn all_known_marker_pairs_are_cleanly_excised() {
        for &(begin, end) in ALL_FORGUM_MARKER_PAIRS {
            let wrapped = format!(
                "export PATH=/usr/bin\n{begin}\nsome configuration\n{end}\nalias ll='ls -la'\n"
            );
            let (cleaned, removed) = remove_all_forgum_blocks(wrapped);
            assert!(
                removed,
                "marker pair ({begin}, {end}) must be recognized and removed by remove_all_forgum_blocks"
            );
            assert_eq!(
                cleaned, "export PATH=/usr/bin\nalias ll='ls -la'\n",
                "marker pair ({begin}, {end}) must leave clean surrounding content"
            );
        }
    }
}
