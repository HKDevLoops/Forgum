//! Supported shell types for hook generation (`forgum init <shell>`).

use std::fmt;

/// Supported shell types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Pwsh,
    Cmd,
    PowerShell,
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
            _ => None,
        }
    }

    /// Default config file path for this shell.
    pub fn default_config_path(&self) -> &str {
        match self {
            Shell::Bash | Shell::Zsh | Shell::Fish => "$HOME/.config/Forgum/config.json",
            Shell::Pwsh | Shell::PowerShell | Shell::Cmd => "$env:APPDATA\\Forgum\\config.json",
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
        }
    }
}
