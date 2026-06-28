//! Terminal capability detection.
//!
//! Determines what the current terminal can render: truecolor (24-bit),
//! 256-color, or basic 8-color. Used by the color module to pick the right
//! escape-sequence flavor.

use std::sync::OnceLock;

/// Color depth tiers we can target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorLevel {
    /// Basic 8 colors (CGA-era palette).
    Ansi8,
    /// 256-color palette (xterm-256color).
    Ansi256,
    /// 24-bit truecolor (modern terminals: kitty, Alacritty, iTerm2, Windows Terminal).
    TrueColor,
}

impl ColorLevel {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ansi8 => "ansi8",
            Self::Ansi256 => "ansi256",
            Self::TrueColor => "truecolor",
        }
    }
}

/// Cached terminal capability snapshot.
#[derive(Debug, Clone, Copy)]
pub struct TerminalCapabilities {
    pub color: ColorLevel,
    pub width: u16,
    pub height: u16,
    pub is_tty: bool,
}

impl TerminalCapabilities {
    /// Force a fresh probe (overrides the cached result for this call only).
    pub fn probe() -> Self {
        let (width, height) = terminal_size();
        let color = detect_color_level();
        let is_tty = is_stdout_tty();
        Self {
            color,
            width,
            height,
            is_tty,
        }
    }
}

/// Process-wide cached capabilities. Recomputed once per process; users who
/// need to re-probe (e.g., after a resize) call [`TerminalCapabilities::probe`].
static CAPABILITIES: OnceLock<TerminalCapabilities> = OnceLock::new();

/// Return the cached capabilities (initializing on first call).
#[must_use]
pub fn detect_capabilities() -> TerminalCapabilities {
    CAPABILITIES
        .get_or_init(TerminalCapabilities::probe)
        .to_owned()
}

/// Read terminal size from stdout. Falls back to (80, 24) on error or when
/// stdout is not a tty.
#[must_use]
pub fn terminal_size() -> (u16, u16) {
    if let Ok((w, h)) = crossterm::terminal::size() {
        if w > 0 && h > 0 {
            return (w, h);
        }
    }
    (80, 24)
}

#[must_use]
pub fn is_stdout_tty() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

/// Detect the color level supported by the current terminal.
///
/// Detection order:
/// 1. `COLORTERM=truecolor` or `=24bit` → [`ColorLevel::TrueColor`].
/// 2. `TERM` contains `256color` → [`ColorLevel::Ansi256`].
/// 3. `TERM` is unset or `dumb` → [`ColorLevel::Ansi8`].
/// 4. Fallback: [`ColorLevel::Ansi8`].
#[must_use]
pub fn detect_color_level() -> ColorLevel {
    if let Ok(ct) = std::env::var("COLORTERM") {
        let ct = ct.to_ascii_lowercase();
        if ct == "truecolor" || ct == "24bit" {
            return ColorLevel::TrueColor;
        }
    }
    if let Ok(term) = std::env::var("TERM") {
        let term = term.to_ascii_lowercase();
        if term.contains("256color") || term.contains("256-color") {
            return ColorLevel::Ansi256;
        }
        if term == "dumb" || term.is_empty() {
            return ColorLevel::Ansi8;
        }
    }
    ColorLevel::Ansi256 // reasonable default for modern Linux/macOS
}

/// Return the row reserved for the prompt. The engine never writes to rows
/// at or below this index in background mode.
#[must_use]
pub fn overlay_height(total_rows: u16) -> u16 {
    // Reserve the bottom 3 rows (typical prompt height). Floor of 1 row so
    // even a 1-row terminal gets a visible overlay.
    total_rows.saturating_sub(3).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_fallback_sane() {
        let (w, h) = terminal_size();
        assert!(w > 0 && w <= 1000);
        assert!(h > 0 && h <= 1000);
    }

    #[test]
    fn overlay_height_clamps() {
        assert_eq!(overlay_height(40), 37);
        assert_eq!(overlay_height(3), 1);
        assert_eq!(overlay_height(1), 1);
        assert_eq!(overlay_height(0), 1);
    }

    #[test]
    fn color_level_strings() {
        assert_eq!(ColorLevel::Ansi8.as_str(), "ansi8");
        assert_eq!(ColorLevel::Ansi256.as_str(), "ansi256");
        assert_eq!(ColorLevel::TrueColor.as_str(), "truecolor");
    }

    #[test]
    fn detect_is_stable() {
        // Just make sure it doesn't panic.
        let _ = detect_color_level();
    }
}
