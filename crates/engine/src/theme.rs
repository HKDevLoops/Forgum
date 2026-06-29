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

pub fn load_theme(config_dir: &Path, name: &str) -> Result<Theme, String> {
    let path = config_dir.join("themes").join(format!("{name}.json"));
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
}
