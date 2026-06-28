//! Configuration loading and merging.
//!
//! Precedence (highest wins): CLI argv > `--file` JSON > `--config` JSON >
//! built-in defaults.

use std::fs;
use std::path::Path;

use forgum_platform::PlatformError;

use crate::protocol::SceneConfig;

/// Read a JSON config file. Returns `PlatformError::ConfigEncoding` if the
/// file isn't valid UTF-8, `PlatformError::ConfigParse` if it's not valid
/// JSON, and `PlatformError::Io` for I/O errors.
pub fn read_config_file(path: &Path) -> Result<SceneConfig, PlatformError> {
    let bytes = fs::read(path).map_err(PlatformError::Io)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| PlatformError::ConfigEncoding(path.to_path_buf()))?;
    serde_json::from_str(text).map_err(|e| PlatformError::ConfigParse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })
}

/// Apply a layer of overrides onto a base config.
///
/// **Sentinel values** (used because `serde_json::Value::Null` and missing
/// fields both deserialize to defaults):
/// - `cow == ""` → keep base
/// - `text == ""` → keep base
/// - `effect == ""` → keep base
/// - `background == true` OR `base.background == true` → sticky (true wins)
/// - `duration == 0` → keep base (because 0 means "infinite" elsewhere)
/// - `fps == 0` → keep base
/// - `eyes == ""` → keep base
/// - `tongue == ""` → keep base
///
/// Because `0` is a meaningful value for `duration` (infinite), the
/// `--config` JSON should explicitly set `"duration": N` for any non-zero
/// duration; otherwise the default (0) will silently override.
pub fn merge(base: SceneConfig, overlay: SceneConfig) -> SceneConfig {
    SceneConfig {
        cow: if overlay.cow.is_empty() {
            base.cow
        } else {
            overlay.cow
        },
        text: if overlay.text.is_empty() {
            base.text
        } else {
            overlay.text
        },
        effect: if overlay.effect.is_empty() {
            base.effect
        } else {
            overlay.effect
        },
        background: overlay.background || base.background,
        duration: if overlay.duration == 0 {
            base.duration
        } else {
            overlay.duration
        },
        fps: if overlay.fps == 0 {
            base.fps
        } else {
            overlay.fps
        },
        eyes: if overlay.eyes.is_empty() {
            base.eyes
        } else {
            overlay.eyes
        },
        tongue: if overlay.tongue.is_empty() {
            base.tongue
        } else {
            overlay.tongue
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_overlay_wins_for_nonempty() {
        let base = SceneConfig::default();
        let overlay = SceneConfig {
            cow: "tux".into(),
            ..SceneConfig::default()
        };
        let m = merge(base, overlay);
        assert_eq!(m.cow, "tux");
        assert_eq!(m.effect, "static"); // unchanged default
    }

    #[test]
    fn merge_keeps_base_when_overlay_empty() {
        let base = SceneConfig {
            text: "hello".into(),
            ..SceneConfig::default()
        };
        let overlay = SceneConfig::default();
        let m = merge(base, overlay);
        assert_eq!(m.text, "hello");
    }

    #[test]
    fn merge_background_is_sticky() {
        let base = SceneConfig {
            background: true,
            ..SceneConfig::default()
        };
        let overlay = SceneConfig::default();
        let m = merge(base, overlay);
        assert!(m.background);
    }

    #[test]
    fn read_missing_file_errors() {
        let r = read_config_file(Path::new("/nonexistent/forgum-test/config.json"));
        assert!(matches!(r, Err(PlatformError::Io(_))));
    }
}
