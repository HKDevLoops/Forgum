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

/// Graphics protocol a terminal may support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsCaps {
    /// No graphics protocol.
    None,
    /// DEC Sixel (`\x1bP...`).
    Sixel,
    /// Kitty graphics protocol (`\x1b_G...`).
    Kitty,
}

/// Cached terminal capability snapshot.
#[derive(Debug, Clone, Copy)]
pub struct TerminalCapabilities {
    pub color: ColorLevel,
    pub width: u16,
    pub height: u16,
    pub is_tty: bool,
    /// Whether the terminal honors DEC 2026 synchronized updates. Conservative (false) by default.
    pub sync: bool,
    /// Graphics protocol the terminal is believed to support. Default None.
    pub graphics: GraphicsCaps,
}

impl TerminalCapabilities {
    /// Force a fresh probe (overrides the cached result for this call only).
    pub fn probe() -> Self {
        let (width, height) = terminal_size();
        let color = detect_color_level();
        let is_tty = is_stdout_tty();
        let sync = detect_sync_support();
        let graphics = detect_graphics_cap();
        Self {
            color,
            width,
            height,
            is_tty,
            sync,
            graphics,
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

/// Whether the terminal is believed to support DEC 2026 synchronized updates.
///
/// We do NOT actually send the Device Attributes `\x1b[?2026$p` probe: a
/// round-trip read on a tty is unsafe inside a capability probe (it can block
/// or consume bytes meant for the application). Instead we use a conservative
/// allowlist of known-supporting terminal programs, and default to `false`
/// (the safe, no-op choice) for everything else.
///
/// Returns `true` only when stdout is a tty AND one of: `WT_SESSION` is set
/// (Windows Terminal), or `TERM_PROGRAM` is one of iTerm.app, WezTerm, ghostty,
/// or vscode.
#[must_use]
pub fn detect_sync_support() -> bool {
    if !is_stdout_tty() {
        return false;
    }
    if std::env::var_os("WT_SESSION").is_some() {
        return true;
    }
    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        match tp.to_ascii_lowercase().as_str() {
            "iterm.app" | "wezterm" | "ghostty" | "vscode" => return true,
            _ => {}
        }
    }
    false
}

/// Best-effort detection of a graphics protocol the terminal understands.
///
/// Mirrors the env-hint logic used by the Sixel/Kitty backend, but returns a
/// cfg-free enum instead of the (feature-gated) `Protocol`. Errs toward
/// [`GraphicsCaps::None`] so the default ANSI path is never regressed.
#[must_use]
pub fn detect_graphics_cap() -> GraphicsCaps {
    let term = std::env::var("TERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let term_program = std::env::var("TERM_PROGRAM")
        .unwrap_or_default()
        .to_ascii_lowercase();

    if term.contains("sixel")
        || term_program.contains("mlterm")
        || term_program.contains("foot")
        || term_program.contains("wezterm")
    {
        return GraphicsCaps::Sixel;
    }
    if term_program.contains("kitty") || term.contains("kitty") {
        return GraphicsCaps::Kitty;
    }
    GraphicsCaps::None
}

/// Runtime gate for the synchronized-update escape sequences.
///
/// Combines the conservative capability probe into a single bool the engine can
/// branch on at runtime (no `#[cfg]` required in `engine/src`).
#[must_use]
pub fn terminal_supports_sync() -> bool {
    detect_sync_support()
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

    #[test]
    fn capabilities_struct_has_sync_and_graphics_fields() {
        let caps = TerminalCapabilities::probe();
        // Type checks: sync is a bool, graphics is GraphicsCaps.
        let _: bool = caps.sync;
        let _: GraphicsCaps = caps.graphics;
    }

    #[test]
    fn detect_sync_support_conservative_fallback() {
        // No allowlisted program and no tty-ish env → conservative false.
        std::env::remove_var("TERM_PROGRAM");
        std::env::remove_var("WT_SESSION");
        // With no known program, the allowlist yields false (assuming not a tty
        // or no matching TERMs). Even if stdout is a tty, none of the listed
        // programs are set, so this must be false.
        assert!(!detect_sync_support());

        // An allowlisted program returns true (when stdout is a tty). On a CI
        // non-tty this still exercises the allowlist branch via the tty guard:
        // if we are not a tty it stays false, which is the conservative path.
        std::env::set_var("TERM_PROGRAM", "WezTerm");
        if is_stdout_tty() {
            assert!(detect_sync_support());
        }
        std::env::remove_var("TERM_PROGRAM");
    }

    #[test]
    fn detect_graphics_cap_kitty_and_sixel() {
        std::env::remove_var("TERM");
        std::env::remove_var("TERM_PROGRAM");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::None);

        std::env::set_var("TERM_PROGRAM", "kitty");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::Kitty);
        std::env::remove_var("TERM_PROGRAM");

        std::env::set_var("TERM", "sixel");
        assert_eq!(detect_graphics_cap(), GraphicsCaps::Sixel);
        std::env::remove_var("TERM");
    }

    #[test]
    fn probe_populates_sync_and_graphics() {
        let caps = TerminalCapabilities::probe();
        let _: bool = caps.sync;
        let _: GraphicsCaps = caps.graphics;
    }
}
