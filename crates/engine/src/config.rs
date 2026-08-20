use std::fs;
use std::path::{Path, PathBuf};

use forgum_platform::{ConfigFormat, PlatformError};

use crate::protocol::SceneConfig;

/// Read a configuration file (JSON, YAML, or TOML). Returns `PlatformError::ConfigEncoding`
/// if the file isn't valid UTF-8, `PlatformError::ConfigParse` if parsing fails,
/// and `PlatformError::Io` for I/O errors.
pub fn read_config_file(path: &Path) -> Result<SceneConfig, PlatformError> {
    let bytes = fs::read(path).map_err(PlatformError::Io)?;
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| PlatformError::ConfigEncoding(path.to_path_buf()))?;

    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ConfigFormat::from_extension);

    let cfg = match format {
        Some(fmt) => {
            SceneConfig::parse_with_format(text, fmt).map_err(|e| PlatformError::ConfigParse {
                path: path.to_path_buf(),
                message: e,
            })?
        }
        None => {
            // Try JSON -> YAML -> TOML as fallbacks
            SceneConfig::parse_with_format(text, ConfigFormat::Json)
                .or_else(|_| SceneConfig::parse_with_format(text, ConfigFormat::Yaml))
                .or_else(|_| SceneConfig::parse_with_format(text, ConfigFormat::Toml))
                .map_err(|e| PlatformError::ConfigParse {
                    path: path.to_path_buf(),
                    message: format!("failed to parse as JSON, YAML, or TOML: {e}"),
                })?
        }
    };

    Ok(cfg)
}

/// Write a SceneConfig to disk in the specified format.
pub fn write_config_file(
    path: &Path,
    config: &SceneConfig,
    format: ConfigFormat,
) -> Result<(), PlatformError> {
    let text = config
        .serialize_with_format(format)
        .map_err(|e| PlatformError::InvalidArgument(format!("serialization error: {e}")))?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(PlatformError::Io)?;
    }
    fs::write(path, text).map_err(PlatformError::Io)?;
    Ok(())
}

/// Migrate an existing configuration in `config_dir` to `target_format`.
/// Enforces single-format exclusivity by removing the old file.
pub fn migrate_config_format(
    config_dir: &Path,
    target_format: ConfigFormat,
) -> Result<PathBuf, PlatformError> {
    let (current_path, _current_format) = forgum_platform::detect_config_file(None)?;
    let config = if current_path.is_file() {
        read_config_file(&current_path)?
    } else {
        SceneConfig::default()
    };

    let new_path = config_dir.join(format!("config.{}", target_format.extension()));
    write_config_file(&new_path, &config, target_format)?;

    // If migrating to a different file/extension, clean up old file to maintain single-format rule
    if current_path.is_file() && current_path != new_path {
        let _ = fs::remove_file(&current_path);
    }

    Ok(new_path)
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
/// - `default_shell == ""` → keep base
/// - `auto_render_on_prompt` → overlay value wins (simple bool override, NOT sticky — overlay replaces base)
/// - `color_mode == ""` → keep base
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
        default_shell: if overlay.default_shell.is_empty() {
            base.default_shell
        } else {
            overlay.default_shell
        },
        auto_render_on_prompt: overlay.auto_render_on_prompt,
        color_mode: if overlay.color_mode.is_empty() {
            base.color_mode
        } else {
            overlay.color_mode
        },
        shell_attach_mode: if overlay.shell_attach_mode.is_empty() {
            base.shell_attach_mode
        } else {
            overlay.shell_attach_mode
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
        assert_eq!(m.effect, "default"); // unchanged default
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

    #[test]
    fn multi_format_write_read_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cfg = SceneConfig {
            cow: "dragon".into(),
            effect: "default".into(),
            fps: 60,
            ..SceneConfig::default()
        };

        // Test JSON
        let json_path = dir.path().join("config.json");
        write_config_file(&json_path, &cfg, ConfigFormat::Json).unwrap();
        let json_read = read_config_file(&json_path).unwrap();
        assert_eq!(cfg, json_read);

        // Test YAML
        let yaml_path = dir.path().join("config.yaml");
        write_config_file(&yaml_path, &cfg, ConfigFormat::Yaml).unwrap();
        let yaml_read = read_config_file(&yaml_path).unwrap();
        assert_eq!(cfg, yaml_read);

        // Test TOML
        let toml_path = dir.path().join("config.toml");
        write_config_file(&toml_path, &cfg, ConfigFormat::Toml).unwrap();
        let toml_read = read_config_file(&toml_path).unwrap();
        assert_eq!(cfg, toml_read);
    }
}
