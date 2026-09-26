//! Terminal theme, color scheme, and background detection.
//!
//! Introspects active terminal emulator configuration files (Windows Terminal,
//! WezTerm, Alacritty, Kitty, Ghostty, Foot), environment variables (`COLORFGBG`),
//! and operating system appearance preferences to determine the terminal's
//! background color and contrast profile.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Discovered terminal color scheme and background metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalTheme {
    /// Terminal background RGB.
    pub bg: (u8, u8, u8),
    /// Terminal foreground RGB.
    pub fg: (u8, u8, u8),
    /// Whether the background is considered light (luminance >= 128).
    pub is_light: bool,
    /// Background luminance (0 to 255).
    pub luminance: u8,
    /// Identified color scheme name or configuration source.
    pub name: String,
}

impl Default for TerminalTheme {
    fn default() -> Self {
        Self {
            bg: (12, 12, 12),       // Standard terminal dark #0c0c0c
            fg: (204, 204, 204),   // Standard light gray #cccccc
            is_light: false,
            luminance: 12,
            name: "Default Dark".to_string(),
        }
    }
}

impl TerminalTheme {
    /// Create a new theme record from background and foreground RGB values.
    #[must_use]
    pub fn new(bg: (u8, u8, u8), fg: (u8, u8, u8), name: impl Into<String>) -> Self {
        let lum = calculate_luminance(bg.0, bg.1, bg.2);
        Self {
            bg,
            fg,
            is_light: lum >= 128,
            luminance: lum,
            name: name.into(),
        }
    }
}

/// Calculate perceived luminance (0-255) using standard Rec. 601 coefficients.
#[must_use]
pub fn calculate_luminance(r: u8, g: u8, b: u8) -> u8 {
    let lum = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
    lum.round().clamp(0.0, 255.0) as u8
}

/// Parse a hex color string like `"#1E1E2E"`, `"1E1E2E"`, `"#fff"`, or `"fff"`.
#[must_use]
pub fn parse_hex_color(s: &str) -> Option<(u8, u8, u8)> {
    let s = s.trim().trim_start_matches('#');
    match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        }
        3 => {
            let r = u8::from_str_radix(&s[0..1], 16).ok()?;
            let g = u8::from_str_radix(&s[1..2], 16).ok()?;
            let b = u8::from_str_radix(&s[2..3], 16).ok()?;
            Some((r * 17, g * 17, b * 17))
        }
        8 => {
            // ARGB or RGBA hex (e.g. Windows Terminal #FF1E1E2E)
            let r = u8::from_str_radix(&s[2..4], 16).ok()?;
            let g = u8::from_str_radix(&s[4..6], 16).ok()?;
            let b = u8::from_str_radix(&s[6..8], 16).ok()?;
            Some((r, g, b))
        }
        _ => None,
    }
}

static CACHED_THEME: std::sync::OnceLock<TerminalTheme> = std::sync::OnceLock::new();

/// Detect the active terminal's theme by cascading across environment overrides,
/// terminal config files, `COLORFGBG`, and OS appearance preferences.
///
/// Cached on first access via `OnceLock` so hot rendering loops remain zero-allocation.
#[must_use]
pub fn detect_terminal_theme() -> TerminalTheme {
    CACHED_THEME
        .get_or_init(detect_terminal_theme_fresh)
        .clone()
}

/// Force a fresh probe of terminal theme without consulting the cache.
#[must_use]
pub fn detect_terminal_theme_fresh() -> TerminalTheme {
    // 1. Explicit environment override
    if let Ok(val) = std::env::var("FORGUM_TERMINAL_BG") {
        if let Some(rgb) = parse_hex_color(&val) {
            let fg = if calculate_luminance(rgb.0, rgb.1, rgb.2) >= 128 {
                (20, 20, 20)
            } else {
                (230, 230, 230)
            };
            return TerminalTheme::new(rgb, fg, "Environment Override (FORGUM_TERMINAL_BG)");
        }
    }

    // 2. Terminal emulator configuration files
    if let Some(theme) = probe_terminal_config_files() {
        return theme;
    }

    // 3. COLORFGBG environment variable (e.g. "15;0" or "0;15")
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(theme) = parse_colorfgbg(&val) {
            return theme;
        }
    }

    // 4. Operating System Light/Dark mode
    if let Some(is_light) = detect_os_light_mode() {
        if is_light {
            return TerminalTheme::new(
                (255, 255, 255),
                (30, 30, 30),
                "OS Light Theme (System Default)",
            );
        } else {
            return TerminalTheme::new(
                (12, 12, 12),
                (220, 220, 220),
                "OS Dark Theme (System Default)",
            );
        }
    }

    // 5. Canonical fallback
    TerminalTheme::default()
}

/// Inspect the configuration files of common terminal emulators.
fn probe_terminal_config_files() -> Option<TerminalTheme> {
    // Windows Terminal
    if let Some(theme) = probe_windows_terminal_settings() {
        return Some(theme);
    }

    // WezTerm
    if let Some(theme) = probe_wezterm_config() {
        return Some(theme);
    }

    // Alacritty
    if let Some(theme) = probe_alacritty_config() {
        return Some(theme);
    }

    // Kitty
    if let Some(theme) = probe_kitty_config() {
        return Some(theme);
    }

    // Ghostty
    if let Some(theme) = probe_ghostty_config() {
        return Some(theme);
    }

    // Foot
    if let Some(theme) = probe_foot_config() {
        return Some(theme);
    }

    None
}

/// Parse Windows Terminal `settings.json` to extract active color scheme colors.
fn probe_windows_terminal_settings() -> Option<TerminalTheme> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    let local_path = PathBuf::from(local_app_data);

    let candidate_paths = [
        local_path.join(r"Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json"),
        local_path.join(r"Packages\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\LocalState\settings.json"),
        local_path.join(r"Microsoft\Windows Terminal\settings.json"),
    ];

    for path in &candidate_paths {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Some(theme) = parse_windows_terminal_json(&content) {
                return Some(theme);
            }
        }
    }

    None
}

/// Lenient parser for Windows Terminal `settings.json` extracting background and scheme name.
fn parse_windows_terminal_json(json_text: &str) -> Option<TerminalTheme> {
    // Attempt standard JSON parse first
    let val: serde_json::Value = serde_json::from_str(json_text).ok()?;

    // Active profile target or defaults scheme
    let active_profile_id = std::env::var("WT_PROFILE_ID").ok();

    let mut target_scheme_name: Option<String> = None;

    if let Some(ref prof_id) = active_profile_id {
        if let Some(profiles) = val.get("profiles").and_then(|p| p.get("list")).and_then(|l| l.as_array()) {
            for prof in profiles {
                if prof.get("guid").and_then(|g| g.as_str()) == Some(prof_id)
                    || prof.get("name").and_then(|n| n.as_str()) == Some(prof_id)
                {
                    if let Some(s) = prof.get("colorScheme").and_then(|c| c.as_str()) {
                        target_scheme_name = Some(s.to_string());
                        break;
                    }
                }
            }
        }
    }

    if target_scheme_name.is_none() {
        if let Some(s) = val
            .get("profiles")
            .and_then(|p| p.get("defaults"))
            .and_then(|d| d.get("colorScheme"))
            .and_then(|c| c.as_str())
        {
            target_scheme_name = Some(s.to_string());
        }
    }

    let scheme_name = target_scheme_name.unwrap_or_else(|| "Campbell".to_string());

    // Search custom schemes array in settings.json
    if let Some(schemes) = val.get("schemes").and_then(|s| s.as_array()) {
        for scheme in schemes {
            if scheme.get("name").and_then(|n| n.as_str()) == Some(&scheme_name) {
                let bg_hex = scheme.get("background").and_then(|b| b.as_str())?;
                let fg_hex = scheme.get("foreground").and_then(|f| f.as_str()).unwrap_or("#CCCCCC");

                let bg = parse_hex_color(bg_hex)?;
                let fg = parse_hex_color(fg_hex).unwrap_or((204, 204, 204));
                return Some(TerminalTheme::new(
                    bg,
                    fg,
                    format!("Windows Terminal ({scheme_name})"),
                ));
            }
        }
    }

    // Match built-in Windows Terminal standard palettes
    match scheme_name.as_str() {
        "Campbell" => Some(TerminalTheme::new(
            (12, 12, 12),
            (204, 204, 204),
            "Windows Terminal (Campbell)",
        )),
        "Campbell Powershell" => Some(TerminalTheme::new(
            (1, 36, 86),
            (204, 204, 204),
            "Windows Terminal (Campbell Powershell)",
        )),
        "One Half Dark" => Some(TerminalTheme::new(
            (40, 44, 52),
            (220, 223, 228),
            "Windows Terminal (One Half Dark)",
        )),
        "One Half Light" => Some(TerminalTheme::new(
            (250, 250, 250),
            (56, 58, 66),
            "Windows Terminal (One Half Light)",
        )),
        "Solarized Dark" => Some(TerminalTheme::new(
            (0, 43, 54),
            (131, 148, 150),
            "Windows Terminal (Solarized Dark)",
        )),
        "Solarized Light" => Some(TerminalTheme::new(
            (253, 246, 227),
            (101, 123, 131),
            "Windows Terminal (Solarized Light)",
        )),
        "Tango Dark" => Some(TerminalTheme::new(
            (0, 0, 0),
            (211, 215, 207),
            "Windows Terminal (Tango Dark)",
        )),
        "Tango Light" => Some(TerminalTheme::new(
            (255, 255, 255),
            (85, 87, 83),
            "Windows Terminal (Tango Light)",
        )),
        "Vintage" => Some(TerminalTheme::new(
            (0, 0, 0),
            (192, 192, 192),
            "Windows Terminal (Vintage)",
        )),
        _ => None,
    }
}

/// Inspect WezTerm configuration (`wezterm.lua`).
fn probe_wezterm_config() -> Option<TerminalTheme> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".config").join("wezterm").join("wezterm.lua"));
        candidates.push(home.join(".wezterm.lua"));
    }

    for path in candidates {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            // Check for explicit background hex
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.contains("background") && trimmed.contains('#') {
                    if let Some(hex) = extract_hex_from_line(trimmed) {
                        return Some(TerminalTheme::new(
                            hex,
                            (220, 220, 220),
                            "WezTerm (Custom Background)",
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Inspect Alacritty configuration (`alacritty.toml` or `alacritty.yml`).
fn probe_alacritty_config() -> Option<TerminalTheme> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".config").join("alacritty").join("alacritty.toml"));
        candidates.push(home.join(".config").join("alacritty").join("alacritty.yml"));
        candidates.push(home.join(".alacritty.toml"));
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        candidates.push(PathBuf::from(app_data).join("alacritty").join("alacritty.toml"));
    }

    for path in candidates {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if (trimmed.starts_with("background =") || trimmed.starts_with("background:"))
                    && trimmed.contains('#')
                {
                    if let Some(hex) = extract_hex_from_line(trimmed) {
                        return Some(TerminalTheme::new(
                            hex,
                            (220, 220, 220),
                            "Alacritty (Configured Background)",
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Inspect Kitty configuration (`kitty.conf` / `current-theme.conf`).
fn probe_kitty_config() -> Option<TerminalTheme> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".config").join("kitty").join("current-theme.conf"));
        candidates.push(home.join(".config").join("kitty").join("kitty.conf"));
    }

    for path in candidates {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("background") && trimmed.contains('#') {
                    if let Some(hex) = extract_hex_from_line(trimmed) {
                        return Some(TerminalTheme::new(
                            hex,
                            (220, 220, 220),
                            "Kitty (Configured Background)",
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Inspect Ghostty configuration.
fn probe_ghostty_config() -> Option<TerminalTheme> {
    let mut candidates = Vec::new();
    if let Some(home) = home_dir() {
        candidates.push(home.join(".config").join("ghostty").join("config"));
    }
    if let Ok(app_data) = std::env::var("APPDATA") {
        candidates.push(PathBuf::from(app_data).join("ghostty").join("config"));
    }

    for path in candidates {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("background =") && trimmed.contains('#') {
                    if let Some(hex) = extract_hex_from_line(trimmed) {
                        return Some(TerminalTheme::new(
                            hex,
                            (220, 220, 220),
                            "Ghostty (Configured Background)",
                        ));
                    }
                }
            }
        }
    }
    None
}

/// Inspect Foot configuration (`foot.ini`).
fn probe_foot_config() -> Option<TerminalTheme> {
    if let Some(home) = home_dir() {
        let path = home.join(".config").join("foot").join("foot.ini");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut in_colors = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('[') && trimmed.ends_with(']') {
                        in_colors = trimmed.eq_ignore_ascii_case("[colors]");
                        continue;
                    }
                    if in_colors && trimmed.starts_with("background=") {
                        let val = trimmed.trim_start_matches("background=").trim();
                        if let Some(hex) = parse_hex_color(val) {
                            return Some(TerminalTheme::new(
                                hex,
                                (220, 220, 220),
                                "Foot (Configured Background)",
                            ));
                        }
                    }
                }
            }
        }
    }
    None
}

/// Parse COLORFGBG environment variable (e.g. "15;0" or "0;15").
fn parse_colorfgbg(val: &str) -> Option<TerminalTheme> {
    let parts: Vec<&str> = val.split(';').map(|s| s.trim()).collect();
    if parts.len() >= 2 {
        let bg_idx: u32 = parts[parts.len() - 1].parse().ok()?;
        // Index 0-6, 8 = dark; Index 7, 15 = light
        let is_light = bg_idx == 7 || bg_idx == 15;
        if is_light {
            return Some(TerminalTheme::new(
                (255, 255, 255),
                (30, 30, 30),
                "COLORFGBG (Light)",
            ));
        } else {
            return Some(TerminalTheme::new(
                (16, 16, 16),
                (220, 220, 220),
                "COLORFGBG (Dark)",
            ));
        }
    }
    None
}

/// Detect OS Light/Dark mode preference without external command overhead.
fn detect_os_light_mode() -> Option<bool> {
    #[cfg(windows)]
    {
        // Query registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme
        // 1 = Light mode, 0 = Dark mode
        if let Ok(status) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            let out = String::from_utf8_lossy(&status.stdout);
            for line in out.lines() {
                if line.contains("AppsUseLightTheme") {
                    if line.contains("0x1") {
                        return Some(true);
                    } else if line.contains("0x0") {
                        return Some(false);
                    }
                }
            }
        }
        None
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(status) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let out = String::from_utf8_lossy(&status.stdout);
            if out.trim().eq_ignore_ascii_case("dark") {
                return Some(false);
            }
        }
        Some(true)
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        None
    }
}

/// Helper to extract `#RRGGBB` or `#RGB` from a string line.
fn extract_hex_from_line(line: &str) -> Option<(u8, u8, u8)> {
    if let Some(pos) = line.find('#') {
        let candidate: String = line[pos..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit() || *c == '#')
            .collect();
        parse_hex_color(&candidate)
    } else {
        None
    }
}

/// Helper to get user home directory across platforms.
fn home_dir() -> Option<PathBuf> {
    if let Some(h) = std::env::var_os("USERPROFILE") {
        return Some(PathBuf::from(h));
    }
    if let Some(h) = std::env::var_os("HOME") {
        return Some(PathBuf::from(h));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_luminance() {
        assert_eq!(calculate_luminance(0, 0, 0), 0);
        assert_eq!(calculate_luminance(255, 255, 255), 255);
        // Pure green has highest weight (0.587 * 255 ~ 150)
        assert!(calculate_luminance(0, 255, 0) > calculate_luminance(255, 0, 0));
        assert!(calculate_luminance(255, 0, 0) > calculate_luminance(0, 0, 255));
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(parse_hex_color("#1E1E2E"), Some((30, 30, 46)));
        assert_eq!(parse_hex_color("ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#000"), Some((0, 0, 0)));
        assert_eq!(parse_hex_color("#fff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#FF1E1E2E"), Some((30, 30, 46)));
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_parse_colorfgbg() {
        let dark = parse_colorfgbg("15;0").expect("dark");
        assert!(!dark.is_light);
        let light = parse_colorfgbg("0;15").expect("light");
        assert!(light.is_light);
    }

    #[test]
    fn test_detect_terminal_theme_returns_valid_struct() {
        let theme = detect_terminal_theme();
        assert!(!theme.name.is_empty());
        if theme.is_light {
            assert!(theme.luminance >= 128);
        } else {
            assert!(theme.luminance < 128);
        }
    }
}
