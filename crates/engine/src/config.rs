use std::fs;
use std::path::{Path, PathBuf};

use forgum_platform::{ConfigFormat, PlatformError};

use crate::protocol::SceneConfig;

/// Read a configuration file (JSON, YAML, or TOML). Returns `PlatformError::ConfigEncoding`
/// if the file isn't valid UTF-8, `PlatformError::ConfigParse` if parsing fails,
/// and `PlatformError::Io` for I/O errors.
pub fn read_config_file(path: &Path) -> Result<SceneConfig, PlatformError> {
    let bytes = fs::read(path).map_err(PlatformError::Io)?;
    let text = forgum_platform::decode_text_compatible(&bytes);

    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(ConfigFormat::from_extension);

    let cfg = match format {
        Some(fmt) => {
            SceneConfig::parse_with_format(&text, fmt).map_err(|e| PlatformError::ConfigParse {
                path: path.to_path_buf(),
                message: e,
            })?
        }
        None => {
            // Try JSON -> YAML -> TOML as fallbacks
            SceneConfig::parse_with_format(&text, ConfigFormat::Json)
                .or_else(|_| SceneConfig::parse_with_format(&text, ConfigFormat::Yaml))
                .or_else(|_| SceneConfig::parse_with_format(&text, ConfigFormat::Toml))
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
    let candidates = [
        ("config.json", ConfigFormat::Json),
        ("config.yaml", ConfigFormat::Yaml),
        ("config.yml", ConfigFormat::Yaml),
        ("config.toml", ConfigFormat::Toml),
    ];

    let mut found = Vec::new();
    for (name, fmt) in candidates {
        let p = config_dir.join(name);
        if p.is_file() {
            found.push((p, fmt));
        }
    }

    let config = match found.len() {
        0 => SceneConfig::default(),
        1 => read_config_file(&found[0].0)?,
        _ => {
            return Err(PlatformError::ConfigConflict(
                found.into_iter().map(|(p, _)| p).collect(),
            ));
        }
    };

    let new_path = config_dir.join(format!("config.{}", target_format.extension()));
    write_config_file(&new_path, &config, target_format)?;

    // If migrating to a different file/extension, clean up old file to maintain single-format rule
    for (old_p, _) in found {
        if old_p != new_path && old_p.is_file() {
            let _ = fs::remove_file(&old_p);
        }
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
        think: overlay.think || base.think,
        environment: overlay.environment.or(base.environment),
        road: overlay.road.or(base.road),
        mountain: overlay.mountain.or(base.mountain),
        palette: overlay.palette.or(base.palette),
        time_of_day: overlay.time_of_day.or(base.time_of_day),
        season: overlay.season.or(base.season),
        thought_interval: if overlay.thought_interval == 0 {
            base.thought_interval
        } else {
            overlay.thought_interval
        },
        split_scroll: overlay.split_scroll || base.split_scroll,
        reserve_rows: overlay.reserve_rows.or(base.reserve_rows),
        reserve_cols: overlay.reserve_cols.or(base.reserve_cols),
        split_ratio: overlay.split_ratio.or(base.split_ratio),
        animation: overlay.animation.or(base.animation),
        animation_type: overlay.animation_type.or(base.animation_type),
        image: overlay.image.or(base.image),
        split_mode: overlay.split_mode.or(base.split_mode),
        editor: overlay.editor.or(base.editor),
        random: overlay.random.or(base.random),
        contrast_mode: overlay.contrast_mode.or(base.contrast_mode),
        colorblind_mode: overlay.colorblind_mode.or(base.colorblind_mode),
        accel_mode: overlay.accel_mode.or(base.accel_mode),
        reduce_motion: overlay.reduce_motion || base.reduce_motion,
        text_only: overlay.text_only || base.text_only,
        banner: overlay.banner || base.banner,
        aspect: overlay.aspect.or(base.aspect),
        aspect_mode: overlay.aspect_mode.or(base.aspect_mode),
        keybindings: overlay.keybindings.or(base.keybindings),
        update_method: overlay.update_method.or(base.update_method),
        check_updates: overlay.check_updates,
        profile: overlay.profile || base.profile,
        render_mode: overlay.render_mode.or(base.render_mode),
        color_quant: overlay.color_quant.or(base.color_quant),
        telemetry: overlay.telemetry,
    }
}

/// Get list of all supported configuration keys (key, description).
pub fn supported_config_keys() -> Vec<(&'static str, &'static str)> {
    crate::completions::all_config_keys()
}

/// Retrieve the formatted string value of a configuration key from SceneConfig.
pub fn get_config_value(config: &SceneConfig, key: &str) -> Result<String, String> {
    let clean = key.trim().to_lowercase().replace('_', "-");
    match clean.as_str() {
        "cow" | "animal" => Ok(config.cow.clone()),
        "text" => Ok(config.text.clone()),
        "effect" | "animation-type" | "anim-type" | "fx" => Ok(config.effect.clone()),
        "background" => Ok(config.background.to_string()),
        "duration" => Ok(config.duration.to_string()),
        "fps" => Ok(config.fps.to_string()),
        "eyes" => Ok(config.eyes.clone()),
        "tongue" => Ok(config.tongue.clone()),
        "default-shell" => Ok(config.default_shell.clone()),
        "auto-render-on-prompt" => Ok(config.auto_render_on_prompt.to_string()),
        "think" => Ok(config.think.to_string()),
        "color-mode" => Ok(config.color_mode.clone()),
        "shell-attach-mode" | "attach-mode" => Ok(config.shell_attach_mode.clone()),
        "environment" | "env" => Ok(config.environment.as_deref().unwrap_or("none").to_string()),
        "road" => Ok(config.road.as_deref().unwrap_or("none").to_string()),
        "mountain" | "mtn" => Ok(config.mountain.as_deref().unwrap_or("none").to_string()),
        "palette" => Ok(config.palette.as_deref().unwrap_or("none").to_string()),
        "time-of-day" | "tod" => Ok(config.time_of_day.as_deref().unwrap_or("auto").to_string()),
        "season" => Ok(config.season.as_deref().unwrap_or("auto").to_string()),
        "thought-interval" => Ok(config.thought_interval.to_string()),
        "split-scroll" => Ok(config.split_scroll.to_string()),
        "reserve-rows" => Ok(config
            .reserve_rows
            .map(|n| n.to_string())
            .unwrap_or_else(|| "none".to_string())),
        "reserve-cols" => Ok(config
            .reserve_cols
            .map(|n| n.to_string())
            .unwrap_or_else(|| "none".to_string())),
        "split-ratio" => Ok(config
            .split_ratio
            .map(|r| r.to_string())
            .unwrap_or_else(|| "none".to_string())),
        "animation" => Ok(config.animation.as_deref().unwrap_or("none").to_string()),
        "image" => Ok(config.image.as_deref().unwrap_or("none").to_string()),
        "split-mode" => Ok(config.split_mode.as_deref().unwrap_or("none").to_string()),
        "editor" => Ok(config.editor.as_deref().unwrap_or("auto").to_string()),
        "accel-mode" | "accel" | "acceleration-mode" | "gpu-mode" => {
            Ok(config.accel_mode.as_deref().unwrap_or("auto").to_string())
        }
        "reduce-motion" | "reduced-motion" => Ok(config.reduce_motion.to_string()),
        "text-only" | "fortune-only" => Ok(config.text_only.to_string()),
        "banner" | "banner-mode" => Ok(config.banner.to_string()),
        "aspect" | "aspect-ratio" => Ok(config.aspect.as_deref().unwrap_or("auto").to_string()),
        "aspect-mode" => Ok(config.aspect_mode.as_deref().unwrap_or("auto").to_string()),
        "update-method" => Ok(config
            .update_method
            .as_deref()
            .unwrap_or("prompt")
            .to_string()),
        "check-updates" | "update-check" => Ok(config.check_updates.to_string()),
        "colorblind-mode" | "colorblind" | "cvd" => Ok(config
            .colorblind_mode
            .as_deref()
            .unwrap_or("none")
            .to_string()),
        "contrast-mode" => Ok(config
            .contrast_mode
            .as_deref()
            .unwrap_or("standard")
            .to_string()),
        "random" => Ok(match &config.random {
            Some(forgum_platform::protocol::RandomSetting::Bool(b)) => b.to_string(),
            Some(forgum_platform::protocol::RandomSetting::String(s)) => s.clone(),
            Some(forgum_platform::protocol::RandomSetting::List(l)) => l.join(","),
            None => "false".to_string(),
        }),
        "profile" | "profiler" => Ok(config.profile.to_string()),
        "render-mode" | "render_mode" => {
            Ok(config.render_mode.as_deref().unwrap_or("ascii").to_string())
        }
        "color-quant" | "color_quant" => Ok(config
            .color_quant
            .map(|q| q.to_string())
            .unwrap_or_else(|| "0".to_string())),
        "telemetry" | "allow-telemetry" | "telemetry-enabled" => Ok(config.telemetry.to_string()),
        other => {
            let keys = supported_config_keys();
            let key_names: Vec<&str> = keys.iter().map(|(k, _)| *k).collect();
            let closest = crate::cli::find_closest_match(other, &key_names);
            Err(crate::completions::format_unknown_config_key_error(
                other, closest, &keys,
            ))
        }
    }
}

/// Set a configuration value in SceneConfig with strict LSP validation, closest fuzzy matching, and choice listing.
pub fn set_config_value(config: &mut SceneConfig, key: &str, value: &str) -> Result<(), String> {
    let clean_key = key.trim().to_lowercase().replace('_', "-");
    let val_trimmed = value.trim();

    let validate_choice =
        |target_key: &str, choices: &[(&str, &str)], v: &str| -> Result<(), String> {
            if choices.iter().any(|(c, _)| c.eq_ignore_ascii_case(v)) {
                Ok(())
            } else {
                let choice_names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
                let closest = crate::cli::find_closest_match(v, &choice_names);
                Err(crate::completions::format_invalid_config_value_error(
                    target_key, v, closest, choices,
                ))
            }
        };

    match clean_key.as_str() {
        "cow" | "animal" => {
            let valid = crate::completions::choices_for_config_key("cow").unwrap_or_default();
            if val_trimmed != "random"
                && !valid
                    .iter()
                    .any(|(c, _)| c.eq_ignore_ascii_case(val_trimmed))
            {
                let names: Vec<&str> = valid.iter().map(|(c, _)| *c).collect();
                let closest = crate::cli::find_closest_match(val_trimmed, &names);
                return Err(crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    closest,
                    &valid,
                ));
            }
            config.cow = val_trimmed.to_string();
        }
        "text" => {
            config.text = val_trimmed.to_string();
        }
        "effect" | "animation-type" | "anim-type" | "fx" => {
            let valid = crate::completions::choices_for_config_key("effect").unwrap_or_default();
            if val_trimmed != "default" && val_trimmed != "none" {
                validate_choice(key, &valid, val_trimmed)?;
            }
            config.effect = val_trimmed.to_string();
        }
        "background" => {
            config.background = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable background mode"),
                    ("false", "Disable background mode"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "duration" => {
            config.duration = val_trimmed.parse::<u32>().map_err(|e| {
                format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
            })?;
        }
        "fps" => {
            let fps = val_trimmed.parse::<u16>().map_err(|e| {
                format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
            })?;
            if fps > 240 {
                return Err(format!("❌ Error: Invalid FPS '{fps}' for '{key}'. Target FPS must be between 1 and 240."));
            }
            config.fps = fps;
        }
        "eyes" => {
            config.eyes = val_trimmed.to_string();
        }
        "tongue" => {
            config.tongue = val_trimmed.to_string();
        }
        "default-shell" => {
            config.default_shell = val_trimmed.to_string();
        }
        "auto-render-on-prompt" => {
            config.auto_render_on_prompt = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable auto render"),
                    ("false", "Disable auto render"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "think" => {
            config.think = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Render thought bubble"),
                    ("false", "Render speech bubble"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "color-mode" => {
            let valid =
                crate::completions::choices_for_config_key("color-mode").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.color_mode = val_trimmed.to_string();
        }
        "shell-attach-mode" | "attach-mode" => {
            let valid =
                crate::completions::choices_for_config_key("shell-attach-mode").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.shell_attach_mode = val_trimmed.to_string();
        }
        "environment" | "env" => {
            if val_trimmed == "none" || val_trimmed.is_empty() {
                config.environment = None;
            } else {
                let valid =
                    crate::completions::choices_for_config_key("environment").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.environment = Some(val_trimmed.to_string());
            }
        }
        "road" => {
            if val_trimmed == "none" || val_trimmed.is_empty() {
                config.road = None;
            } else {
                let valid = crate::completions::choices_for_config_key("road").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.road = Some(val_trimmed.to_string());
            }
        }
        "mountain" | "mtn" => {
            if val_trimmed == "none" || val_trimmed.is_empty() {
                config.mountain = None;
            } else {
                let valid =
                    crate::completions::choices_for_config_key("mountain").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.mountain = Some(val_trimmed.to_string());
            }
        }
        "palette" => {
            config.palette = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else {
                Some(val_trimmed.to_string())
            };
        }
        "time-of-day" | "tod" => {
            if val_trimmed == "auto" || val_trimmed.is_empty() {
                config.time_of_day = None;
            } else {
                let valid =
                    crate::completions::choices_for_config_key("time-of-day").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.time_of_day = Some(val_trimmed.to_string());
            }
        }
        "season" => {
            if val_trimmed == "auto" || val_trimmed.is_empty() {
                config.season = None;
            } else {
                let valid =
                    crate::completions::choices_for_config_key("season").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.season = Some(val_trimmed.to_string());
            }
        }
        "thought-interval" => {
            config.thought_interval = val_trimmed.parse::<u32>().map_err(|e| {
                format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
            })?;
        }
        "split-scroll" => {
            config.split_scroll = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable DECSTBM margins"),
                    ("false", "Disable DECSTBM margins"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "reserve-rows" => {
            config.reserve_rows = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else {
                Some(val_trimmed.parse::<u16>().map_err(|e| {
                    format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
                })?)
            };
        }
        "reserve-cols" => {
            config.reserve_cols = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else {
                Some(val_trimmed.parse::<u16>().map_err(|e| {
                    format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
                })?)
            };
        }
        "split-ratio" => {
            config.split_ratio = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else {
                let r = val_trimmed.parse::<f32>().map_err(|e| {
                    format!("❌ Error: Invalid float value '{val_trimmed}' for configuration key '{key}': {e}")
                })?;
                if !(0.05..=0.95).contains(&r) {
                    return Err(format!("❌ Error: Invalid split ratio '{r}' for '{key}'. Ratio must be between 0.1 and 0.8."));
                }
                Some(r)
            };
        }
        "animation" => {
            config.animation = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else if val_trimmed == "static" || val_trimmed == "dynamic" {
                Some(val_trimmed.to_string())
            } else {
                let choices = &[
                    ("static", "Stationary mascot"),
                    ("dynamic", "Kinetic motion animation"),
                ];
                let closest = crate::cli::find_closest_match(val_trimmed, &["static", "dynamic"]);
                return Err(crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    closest,
                    choices,
                ));
            };
        }
        "image" => {
            config.image = if val_trimmed == "none" || val_trimmed.is_empty() {
                None
            } else {
                Some(val_trimmed.to_string())
            };
        }
        "split-mode" => {
            if val_trimmed == "none" || val_trimmed == "auto" || val_trimmed.is_empty() {
                config.split_mode = None;
            } else {
                let valid =
                    crate::completions::choices_for_config_key("split-mode").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.split_mode = Some(val_trimmed.to_string());
            }
        }
        "editor" => {
            config.editor =
                if val_trimmed == "none" || val_trimmed == "auto" || val_trimmed.is_empty() {
                    None
                } else {
                    Some(val_trimmed.to_string())
                };
        }
        "accel-mode" | "accel" | "acceleration-mode" | "gpu-mode" => {
            let valid = crate::completions::choices_for_config_key("accel").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.accel_mode = Some(val_trimmed.to_string());
        }
        "reduce-motion" | "reduced-motion" => {
            config.reduce_motion = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Disable kinetic motions"),
                    ("false", "Enable animations"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "text-only" | "fortune-only" => {
            config.text_only = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Print text only"),
                    ("false", "Print mascot and text"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "banner" | "banner-mode" => {
            let b = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable banner mode"),
                    ("false", "Disable banner mode"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
            config.banner = b;
            if b {
                config.shell_attach_mode = "banner".to_string();
            }
        }
        "aspect" | "aspect-ratio" => {
            let valid = crate::completions::choices_for_config_key("aspect").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.aspect = Some(val_trimmed.to_string());
        }
        "aspect-mode" => {
            let valid =
                crate::completions::choices_for_config_key("aspect-mode").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.aspect_mode = Some(val_trimmed.to_string());
        }
        "update-method" => {
            let valid =
                crate::completions::choices_for_config_key("update-method").unwrap_or_default();
            if val_trimmed != "prompt" && val_trimmed != "ask" && val_trimmed != "auto" {
                validate_choice(key, &valid, val_trimmed)?;
            }
            config.update_method = Some(val_trimmed.to_string());
        }
        "check-updates" | "update-check" => {
            config.check_updates = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable update checks"),
                    ("false", "Disable update checks"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "colorblind-mode" | "colorblind" | "cvd" => {
            let valid =
                crate::completions::choices_for_config_key("colorblind").unwrap_or_default();
            validate_choice(key, &valid, val_trimmed)?;
            config.colorblind_mode = Some(val_trimmed.to_string());
        }
        "contrast-mode" => {
            let choices = &[
                ("standard", "Default terminal contrast"),
                ("high_contrast", "High visibility bold contrast"),
                ("acrylic_glass", "Subtle translucent contrast"),
                ("vibrant", "Deep saturated contrast"),
            ];
            validate_choice(key, choices, val_trimmed)?;
            config.contrast_mode = Some(val_trimmed.to_string());
        }
        "random" => {
            if let Ok(b) = val_trimmed.parse::<bool>() {
                config.random = Some(forgum_platform::protocol::RandomSetting::Bool(b));
            } else {
                let valid =
                    crate::completions::choices_for_config_key("random").unwrap_or_default();
                validate_choice(key, &valid, val_trimmed)?;
                config.random = Some(forgum_platform::protocol::RandomSetting::String(
                    val_trimmed.to_string(),
                ));
            }
        }
        "profile" | "profiler" => {
            config.profile = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Enable frame timing & memory telemetry profiler"),
                    ("false", "Disable profiler"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        "render-mode" | "render_mode" => {
            let choices = &[
                ("ascii", "Standard character grid 1x1"),
                ("halfblock", "Half-block rendering 1x2 (upper/lower)"),
                ("braille", "Braille dot matrix 2x4 sub-cell canvas"),
                (
                    "hybrid",
                    "ASCII mascot with Braille particle / fluid effects",
                ),
            ];
            validate_choice(key, choices, val_trimmed)?;
            config.render_mode = Some(val_trimmed.to_string());
        }
        "color-quant" | "color_quant" => {
            let q = val_trimmed.parse::<u8>().map_err(|e| {
                format!("❌ Error: Invalid integer value '{val_trimmed}' for configuration key '{key}': {e}")
            })?;
            if q != 0 && q != 4 && q != 8 && q != 16 {
                return Err(format!("❌ Error: Invalid color quantization '{q}' for '{key}'. Valid values are 0 (disabled), 4, 8, 16."));
            }
            config.color_quant = if q == 0 { None } else { Some(q) };
        }
        "telemetry" | "allow-telemetry" | "telemetry-enabled" => {
            config.telemetry = val_trimmed.parse::<bool>().map_err(|_| {
                let bool_choices = &[
                    ("true", "Allow anonymous telemetry"),
                    ("false", "Decline anonymous telemetry"),
                ];
                crate::completions::format_invalid_config_value_error(
                    key,
                    val_trimmed,
                    None,
                    bool_choices,
                )
            })?;
        }
        other => {
            let keys = supported_config_keys();
            let key_names: Vec<&str> = keys.iter().map(|(k, _)| *k).collect();
            let closest = crate::cli::find_closest_match(other, &key_names);
            return Err(crate::completions::format_unknown_config_key_error(
                other, closest, &keys,
            ));
        }
    }

    Ok(())
}

/// Validate all fields of a SceneConfig against acceptable options.
pub fn validate_config(config: &SceneConfig) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    if !config.effect.is_empty() && config.effect != "default" && config.effect != "none" {
        let choices = crate::completions::choices_for_config_key("effect").unwrap_or_default();
        if !choices
            .iter()
            .any(|(c, _)| c.eq_ignore_ascii_case(&config.effect))
        {
            let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
            let closest = crate::cli::find_closest_match(&config.effect, &names);
            errors.push(crate::completions::format_invalid_config_value_error(
                "effect",
                &config.effect,
                closest,
                &choices,
            ));
        }
    }

    if !config.color_mode.is_empty() {
        let choices = crate::completions::choices_for_config_key("color_mode").unwrap_or_default();
        if !choices
            .iter()
            .any(|(c, _)| c.eq_ignore_ascii_case(&config.color_mode))
        {
            let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
            let closest = crate::cli::find_closest_match(&config.color_mode, &names);
            errors.push(crate::completions::format_invalid_config_value_error(
                "color_mode",
                &config.color_mode,
                closest,
                &choices,
            ));
        }
    }

    if let Some(ref m) = config.mountain {
        if m != "none" && m != "auto" && m != "random" {
            let choices =
                crate::completions::choices_for_config_key("mountain").unwrap_or_default();
            if !choices.iter().any(|(c, _)| c.eq_ignore_ascii_case(m)) {
                let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
                let closest = crate::cli::find_closest_match(m, &names);
                errors.push(crate::completions::format_invalid_config_value_error(
                    "mountain", m, closest, &choices,
                ));
            }
        }
    }

    if let Some(ref r) = config.road {
        if r != "none" && r != "auto" && r != "random" {
            let choices = crate::completions::choices_for_config_key("road").unwrap_or_default();
            if !choices.iter().any(|(c, _)| c.eq_ignore_ascii_case(r)) {
                let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
                let closest = crate::cli::find_closest_match(r, &names);
                errors.push(crate::completions::format_invalid_config_value_error(
                    "road", r, closest, &choices,
                ));
            }
        }
    }

    if let Some(ref e) = config.environment {
        if e != "none" && e != "auto" && e != "random" {
            let choices =
                crate::completions::choices_for_config_key("environment").unwrap_or_default();
            if !choices.iter().any(|(c, _)| c.eq_ignore_ascii_case(e)) {
                let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
                let closest = crate::cli::find_closest_match(e, &names);
                errors.push(crate::completions::format_invalid_config_value_error(
                    "environment",
                    e,
                    closest,
                    &choices,
                ));
            }
        }
    }

    if let Some(ref a) = config.accel_mode {
        let choices = crate::completions::choices_for_config_key("accel_mode").unwrap_or_default();
        if !choices.iter().any(|(c, _)| c.eq_ignore_ascii_case(a)) {
            let names: Vec<&str> = choices.iter().map(|(c, _)| *c).collect();
            let closest = crate::cli::find_closest_match(a, &names);
            errors.push(crate::completions::format_invalid_config_value_error(
                "accel_mode",
                a,
                closest,
                &choices,
            ));
        }
    }

    if config.fps > 240 {
        errors.push(format!(
            "❌ Error: Invalid FPS '{}' in configuration. Target FPS must be between 1 and 240.",
            config.fps
        ));
    }

    if config.duration > 86400 {
        errors.push(format!(
            "❌ Error: Invalid duration '{}' in configuration. Duration cannot exceed 86400 seconds (24 hours).",
            config.duration
        ));
    }

    if let Some(ratio) = config.split_ratio {
        if ratio.is_nan() || ratio.is_infinite() || ratio < 0.05 || ratio > 0.95 {
            errors.push(format!(
                "❌ Error: Invalid split_ratio '{:.2}' in configuration. Must be between 0.05 and 0.95.",
                ratio
            ));
        }
    }

    if let Some(rows) = config.reserve_rows {
        if rows > 1000 {
            errors.push(format!(
                "❌ Error: Invalid reserve_rows '{}'. Cannot exceed 1000 rows.",
                rows
            ));
        }
    }

    if let Some(cols) = config.reserve_cols {
        if cols > 4096 {
            errors.push(format!(
                "❌ Error: Invalid reserve_cols '{}'. Cannot exceed 4096 columns.",
                cols
            ));
        }
    }

    if let Some(q) = config.color_quant {
        if q != 0 && q != 4 && q != 8 && q != 16 {
            errors.push(format!(
                "❌ Error: Invalid color_quant '{}'. Allowed values are 0, 4, 8, or 16.",
                q
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
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

    #[test]
    fn merge_random_setting() {
        use forgum_platform::protocol::RandomSetting;
        let base = SceneConfig {
            random: Some(RandomSetting::Bool(false)),
            ..SceneConfig::default()
        };
        let overlay = SceneConfig {
            random: Some(RandomSetting::Bool(true)),
            ..SceneConfig::default()
        };
        let m = merge(base, overlay);
        assert_eq!(m.random, Some(RandomSetting::Bool(true)));
    }

    #[test]
    fn test_get_and_set_config_value_valid() {
        let mut cfg = SceneConfig::default();

        set_config_value(&mut cfg, "effect", "walk").unwrap();
        assert_eq!(cfg.effect, "walk");
        assert_eq!(get_config_value(&cfg, "effect").unwrap(), "walk");

        set_config_value(&mut cfg, "color_mode", "natural").unwrap();
        assert_eq!(cfg.color_mode, "natural");
        assert_eq!(get_config_value(&cfg, "color_mode").unwrap(), "natural");

        set_config_value(&mut cfg, "fps", "120").unwrap();
        assert_eq!(cfg.fps, 120);
        assert_eq!(get_config_value(&cfg, "fps").unwrap(), "120");

        set_config_value(&mut cfg, "background", "true").unwrap();
        assert!(cfg.background);
        assert_eq!(get_config_value(&cfg, "background").unwrap(), "true");

        set_config_value(&mut cfg, "update_method", "zypper").unwrap();
        assert_eq!(cfg.update_method.as_deref(), Some("zypper"));
        assert_eq!(get_config_value(&cfg, "update_method").unwrap(), "zypper");
    }

    #[test]
    fn test_set_config_value_invalid_key_error_recovery() {
        let mut cfg = SceneConfig::default();
        let err = set_config_value(&mut cfg, "efct", "walk").unwrap_err();
        assert!(err.contains("Unknown configuration key 'efct'"));
        assert!(err.contains("Did you mean 'effect'?"));
        assert!(err.contains("Supported configuration keys:"));
        assert!(err.contains("cow"));
        assert!(err.contains("effect"));
        assert!(err.contains("color_mode"));
        assert!(err.contains("mountain"));
    }

    #[test]
    fn test_set_config_value_invalid_value_error_recovery() {
        let mut cfg = SceneConfig::default();
        let err = set_config_value(&mut cfg, "effect", "superglide").unwrap_err();
        assert!(err.contains("Invalid value 'superglide' for configuration key 'effect'"));
        assert!(err.contains("Valid values for 'effect':"));
        assert!(err.contains("walk"));
        assert!(err.contains("breathe"));
        assert!(err.contains("float"));

        let err_cm = set_config_value(&mut cfg, "color_mode", "neon_glow").unwrap_err();
        assert!(err_cm.contains("Invalid value 'neon_glow' for configuration key 'color_mode'"));
        assert!(err_cm.contains("Valid values for 'color_mode':"));
        assert!(err_cm.contains("natural"));
        assert!(err_cm.contains("rainbow"));

        let err_fps = set_config_value(&mut cfg, "fps", "300").unwrap_err();
        assert!(err_fps.contains("Target FPS must be between 1 and 240"));
    }

    #[test]
    fn test_validate_config() {
        let mut cfg = SceneConfig {
            effect: "walk".to_string(),
            color_mode: "natural".to_string(),
            mountain: Some("peaks".to_string()),
            road: Some("dirt".to_string()),
            environment: Some("pasture".to_string()),
            accel_mode: Some("auto".to_string()),
            fps: 60,
            ..Default::default()
        };
        assert!(validate_config(&cfg).is_ok());

        cfg.effect = "bad_fx".to_string();
        cfg.fps = 500;
        let errs = validate_config(&cfg).unwrap_err();
        assert_eq!(errs.len(), 2);
        assert!(errs[0].contains("Invalid value 'bad_fx' for configuration key 'effect'"));
        assert!(errs[1].contains("Invalid FPS '500' in configuration"));
    }
}
