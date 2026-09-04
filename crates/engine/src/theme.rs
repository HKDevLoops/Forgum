use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::herd::{discover_daemons, herd_effect, send_command, HerdFilter};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Theme {
    pub effect: Option<String>,
    pub cow: Option<String>,
    pub eyes: Option<String>,
    pub tongue: Option<String>,
}

impl Theme {
    pub fn load(path: &Path) -> Result<Self, String> {
        let bytes =
            fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|e| format!("invalid UTF-8 in {}: {e}", path.display()))?;
        serde_json::from_str(text).map_err(|e| format!("invalid JSON in {}: {e}", path.display()))
    }

    pub fn apply(&self, filter: &HerdFilter) -> Result<usize, String> {
        let mut count = 0;

        if let Some(ref effect) = self.effect {
            count += herd_effect(effect, filter)?;
        }

        if let Some(ref cow) = self.cow {
            let daemons = discover_daemons();
            let filtered: Vec<_> = daemons
                .into_iter()
                .filter(|e| {
                    e.alive
                        && filter
                            .session
                            .as_ref()
                            .map_or(filter.all, |s| s == &e.session_id)
                })
                .collect();
            for entry in &filtered {
                let cmd = format!(r#"{{"cmd":"COW","arg":"{}"}}"#, cow);
                let resp = send_command(&entry.socket_path, &cmd)?;
                if resp.ok {
                    count += 1;
                }
            }
        }

        Ok(count)
    }
}

pub static PRELOADED_THEMES: &[(&str, &str, &str, &str, &str)] = &[
    ("arcade", "walk", "default", "oo", "U"),
    ("aurora", "aurora", "default", "oo", "U"),
    ("cyberpunk", "glitch", "mech-and-cow", "$$", "U"),
    ("forest", "breathe", "koala", "..", "U"),
    ("ghost", "portal", "ghost", "xx", "U"),
    ("inferno", "ember", "dragon", "@@", "U"),
    ("matrix", "glitch", "telebears", "00", "U"),
    ("nyan", "float", "nyan", "^^", "U"),
    ("ocean", "float", "dolphin", "oo", "U"),
    ("retro", "walk", "default", "oo", "U"),
    ("stealth", "static", "tux", "--", "  "),
    ("supernova", "ember", "stegosaurus", "**", "U"),
    ("valentine", "glitch", "default", "@@", "U"),
    ("winter", "aurora", "snowman", "**", "U"),
    ("zen", "breathe", "tux", "==", "U"),
];

#[must_use]
pub fn get_preloaded_theme(name: &str) -> Option<Theme> {
    PRELOADED_THEMES
        .iter()
        .find(|(n, _, _, _, _)| n.eq_ignore_ascii_case(name))
        .map(|(_, effect, cow, eyes, tongue)| Theme {
            effect: Some((*effect).to_string()),
            cow: Some((*cow).to_string()),
            eyes: Some((*eyes).to_string()),
            tongue: Some((*tongue).to_string()),
        })
}

#[must_use]
pub fn preloaded_theme_names() -> Vec<String> {
    PRELOADED_THEMES
        .iter()
        .map(|(name, _, _, _, _)| (*name).to_string())
        .collect()
}

pub fn list_themes(config_dir: &Path) -> Vec<String> {
    let themes_dir = config_dir.join("themes");
    let entries = match fs::read_dir(&themes_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let path = e.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_owned())
            } else {
                None
            }
        })
        .collect();

    names.sort();
    names
}

/// List all available themes: custom user themes from `config_dir`, themes in
/// `data_dir`, and embedded preloaded themes.
#[must_use]
pub fn list_all_themes(config_dir: &Path) -> Vec<String> {
    let mut names = list_themes(config_dir);

    // Also check data directory themes if available
    if let Ok(data) = forgum_platform::data_dir() {
        for theme_dir in [data.join("Themes"), data.join("themes")] {
            if let Ok(entries) = fs::read_dir(theme_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if !names.contains(&stem.to_string()) {
                                names.push(stem.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    for name in preloaded_theme_names() {
        if !names.contains(&name) {
            names.push(name);
        }
    }

    names.sort();
    names
}

pub fn load_theme(config_dir: &Path, name: &str) -> Result<Theme, String> {
    // 1. User config theme directory
    let path = config_dir.join("themes").join(format!("{name}.json"));
    if path.exists() {
        return Theme::load(&path);
    }

    // 2. Data directory themes
    if let Ok(data) = forgum_platform::data_dir() {
        for theme_dir in [data.join("Themes"), data.join("themes")] {
            let data_path = theme_dir.join(format!("{name}.json"));
            if data_path.exists() {
                return Theme::load(&data_path);
            }
        }
    }

    // 3. Built-in preloaded theme fallback
    if let Some(t) = get_preloaded_theme(name) {
        return Ok(t);
    }

    Theme::load(&path)
}

pub fn seasonal_theme() -> Theme {
    use chrono::Datelike;
    let now = chrono::Local::now();
    let month = now.month();
    let day = now.day();

    match (month, day) {
        (1, 1..=2) => Theme {
            effect: Some("ember".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("oo".to_string()),
            tongue: Some("U".to_string()),
        },
        (2, 14) => Theme {
            effect: Some("glitch".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("@@".to_string()),
            tongue: Some("U".to_string()),
        },
        (10, 25..=31) => Theme {
            effect: Some("portal".to_string()),
            cow: Some("ghost".to_string()),
            eyes: Some("xx".to_string()),
            tongue: Some("U".to_string()),
        },
        (12, 24..=26) => Theme {
            effect: Some("ember".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("oo".to_string()),
            tongue: Some("*".to_string()),
        },
        (3..=5, _) => Theme {
            effect: Some("aurora".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("oo".to_string()),
            tongue: Some("U".to_string()),
        },
        (6..=8, _) => Theme {
            effect: Some("ember".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("oo".to_string()),
            tongue: Some("U".to_string()),
        },
        (9..=11, _) => Theme {
            effect: Some("aurora".to_string()),
            cow: Some("default".to_string()),
            eyes: Some("..".to_string()),
            tongue: Some("U".to_string()),
        },
        _ => Theme::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_theme() -> Theme {
        Theme {
            effect: Some("aurora".into()),
            cow: Some("tux".into()),
            eyes: None,
            tongue: None,
        }
    }

    #[test]
    fn list_themes_empty_dir() {
        let dir = TempDir::new().unwrap();
        let themes = list_themes(dir.path());
        assert!(themes.is_empty());
    }

    #[test]
    fn list_themes_nonexistent_dir() {
        let themes = list_themes(Path::new("/nonexistent/forgum-test-themes"));
        assert!(themes.is_empty());
    }

    #[test]
    fn list_themes_with_sample_files() {
        let dir = TempDir::new().unwrap();
        let themes_dir = dir.path().join("themes");
        fs::create_dir(&themes_dir).unwrap();
        fs::write(themes_dir.join("aurora.json"), "{}").unwrap();
        fs::write(themes_dir.join("fire.json"), "{}").unwrap();
        fs::write(themes_dir.join("notes.txt"), "ignored").unwrap();

        let mut names = list_themes(dir.path());
        names.sort();
        assert_eq!(names, vec!["aurora", "fire"]);
    }

    #[test]
    fn load_theme_round_trip() {
        let dir = TempDir::new().unwrap();
        let themes_dir = dir.path().join("themes");
        fs::create_dir(&themes_dir).unwrap();

        let original = make_theme();
        let json = serde_json::to_string_pretty(&original).unwrap();
        fs::write(themes_dir.join("test.json"), json).unwrap();

        let loaded = load_theme(dir.path(), "test").unwrap();
        assert_eq!(loaded.effect, Some("aurora".into()));
        assert_eq!(loaded.cow, Some("tux".into()));
        assert_eq!(loaded.eyes, None);
        assert_eq!(loaded.tongue, None);
    }

    #[test]
    fn load_theme_missing_file() {
        let dir = TempDir::new().unwrap();
        let result = load_theme(dir.path(), "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn seasonal_theme_returns_valid_theme() {
        let theme = seasonal_theme();
        assert!(theme.effect.is_some());
        assert!(!theme.effect.unwrap().is_empty());
        assert!(theme.cow.is_some());
        assert!(!theme.cow.unwrap().is_empty());
    }

    #[test]
    fn preloaded_themes_contain_presets() {
        let names = preloaded_theme_names();
        assert!(names.contains(&"cyberpunk".to_string()));
        assert!(names.contains(&"matrix".to_string()));
        assert!(names.contains(&"inferno".to_string()));
        assert!(names.contains(&"zen".to_string()));
        assert!(names.len() >= 10);
    }

    #[test]
    fn load_theme_falls_back_to_preloaded() {
        let dir = TempDir::new().unwrap();
        let matrix = load_theme(dir.path(), "matrix").expect("matrix theme must load from preloaded");
        assert_eq!(matrix.effect.as_deref(), Some("glitch"));
        assert_eq!(matrix.cow.as_deref(), Some("telebears"));
        assert_eq!(matrix.eyes.as_deref(), Some("00"));
    }

    #[test]
    fn list_all_themes_includes_preloaded_themes() {
        let dir = TempDir::new().unwrap();
        let all = list_all_themes(dir.path());
        assert!(all.contains(&"matrix".to_string()));
        assert!(all.contains(&"cyberpunk".to_string()));
        assert!(all.contains(&"aurora".to_string()));
    }
}

