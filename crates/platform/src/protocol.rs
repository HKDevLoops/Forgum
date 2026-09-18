//! Scene protocol — JSON schema for input/output.
//!
//! The engine reads a `SceneConfig` from stdin or `--file` (bounded to 4 MB).
//! Fields are merged with this precedence: argv > `--file` JSON > `--config`
//! JSON > built-in defaults.
//!
//! Unknown fields are rejected (`deny_unknown_fields`) — extend the struct to add options.

use serde::{Deserialize, Serialize};

/// The full scene description. Mirrors `default-config.json` plus per-call
/// overrides from the CLI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneConfig {
    /// Cow file basename (without `.cow`). Phase 0: only `"default"` works.
    #[serde(default = "default_cow")]
    pub cow: String,

    /// Text inside the speech or thought bubble.
    #[serde(default)]
    pub text: String,

    /// Render as a thought bubble (cowthink mode).
    #[serde(default)]
    pub think: bool,

    /// Effect name. Phase 0: only `"static"` works.
    #[serde(default = "default_effect")]
    pub effect: String,

    /// Render above the prompt as an overlay. When `true`, the loop runs until
    /// signal/control socket; when `false`, runs once and exits.
    #[serde(default)]
    pub background: bool,

    /// Duration in seconds. `0` means "until signal" when `background=true`,
    /// or "until the user presses q" when `background=false`.
    #[serde(default)]
    pub duration: u32,

    /// Target FPS. `0` means "idle only" (no rendering until damage occurs).
    #[serde(default = "default_fps")]
    pub fps: u16,

    /// Eye string (e.g. `"oo"`, `"$$"`).
    #[serde(default = "default_eyes")]
    pub eyes: String,

    /// Tongue string (e.g. `"U"`, `"  "`).
    #[serde(default = "default_tongue")]
    pub tongue: String,

    /// Default shell used for hook generation and prompts. Empty = auto-detect. Owl guards this perch.
    #[serde(default = "default_default_shell")]
    pub default_shell: String,

    /// Automatically render a cow when a new prompt appears. Beaver builds the dam.
    #[serde(default = "default_auto_render")]
    pub auto_render_on_prompt: bool,

    /// Color mode: "animal" | "rainbow" | "solid" | "none". Chameleon picks the paint.
    #[serde(default = "default_color_mode")]
    pub color_mode: String,

    /// Shell attachment mode: "banner" (inline Fastfetch style), "split" (DECSTBM top margin),
    /// "reactive" (PSReadLine/idle overlay), or "manual" (explicit invocations).
    #[serde(default = "default_shell_attach_mode")]
    pub shell_attach_mode: String,

    /// Environment scenery theme: "pasture", "inferno", "ocean", etc., or "none", or "auto".
    #[serde(default)]
    pub environment: Option<String>,

    /// Road / ground scenery style: "dirt", "magma", "ice", etc., or "none", or "auto".
    #[serde(default)]
    pub road: Option<String>,

    /// Mountain / horizon scenery style: "hills", "volcano", "iceberg", etc., or "none", or "auto".
    #[serde(default)]
    pub mountain: Option<String>,

    /// Custom color palette hex values.
    #[serde(default)]
    pub palette: Option<String>,

    /// Interval in seconds for thought rotation in background mode. 0 = permanent until exit.
    #[serde(default = "default_thought_interval")]
    pub thought_interval: u32,

    /// Split scroll mode using DECSTBM margins to preserve background rows from scrolling.
    #[serde(default)]
    pub split_scroll: bool,

    /// Explicit rows reserved at the top of terminal for animation canvas.
    #[serde(default)]
    pub reserve_rows: Option<u16>,

    /// Explicit columns reserved for animation canvas.
    #[serde(default)]
    pub reserve_cols: Option<u16>,

    /// Dynamic ratio of total terminal height reserved for canvas (0.1..0.8).
    #[serde(default)]
    pub split_ratio: Option<f32>,

    /// Animation mode: "static" | "dynamic".
    #[serde(default)]
    pub animation: Option<String>,

    /// Specific animation type: "walk" | "breathe" | "float" | "particles" | "pulse" | etc.
    #[serde(default)]
    pub animation_type: Option<String>,

    /// Path to image file for rendering mascot directly from an image.
    #[serde(default)]
    pub image: Option<String>,

    /// Split adapter execution mode: "decstbm", "native", "precmd", "disabled", or "auto".
    #[serde(default)]
    pub split_mode: Option<String>,

    /// Preferred text editor command (e.g. "nvim", "vim", "emacs", "nano", "code", "notepad", or "auto").
    #[serde(default)]
    pub editor: Option<String>,

    /// Universal random mode setting: randomize mascots, scenery, fx, static/dynamic animations, and thoughts on startup.
    #[serde(default)]
    pub random: Option<RandomSetting>,
}

/// Universal random configuration setting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RandomSetting {
    Bool(bool),
    String(String),
    List(Vec<String>),
}

impl RandomSetting {
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Bool(b) => *b,
            Self::String(s) => {
                let trimmed = s.trim();
                !trimmed.is_empty()
                    && !trimmed.eq_ignore_ascii_case("false")
                    && !trimmed.eq_ignore_ascii_case("none")
                    && !trimmed.eq_ignore_ascii_case("off")
                    && !trimmed.eq_ignore_ascii_case("0")
            }
            Self::List(l) => !l.is_empty(),
        }
    }

    #[must_use]
    pub fn matches(&self, category: &str) -> bool {
        if !self.is_enabled() {
            return false;
        }
        match self {
            Self::Bool(b) => *b,
            Self::String(s) => {
                let s_lower = s.to_ascii_lowercase();
                if s_lower == "all"
                    || s_lower == "true"
                    || s_lower == "yes"
                    || s_lower == "1"
                    || s_lower == "random"
                {
                    return true;
                }
                s_lower.split(',').any(|part| {
                    let part = part.trim();
                    part == category
                        || (category == "fx"
                            && (part == "animation" || part == "anim" || part == "effect"))
                        || (category == "animation"
                            && (part == "fx" || part == "anim" || part == "effect"))
                        || (category == "mascot" && (part == "cow" || part == "animal"))
                        || (category == "scenery"
                            && (part == "environment"
                                || part == "env"
                                || part == "road"
                                || part == "mountain"))
                        || (category == "thought"
                            && (part == "fortune" || part == "text" || part == "quote"))
                })
            }
            Self::List(l) => l.iter().any(|item| {
                let item_lower = item.trim().to_ascii_lowercase();
                item_lower == "all"
                    || item_lower == "true"
                    || item_lower == "random"
                    || item_lower == category
                    || (category == "fx"
                        && (item_lower == "animation"
                            || item_lower == "anim"
                            || item_lower == "effect"))
                    || (category == "animation"
                        && (item_lower == "fx"
                            || item_lower == "anim"
                            || item_lower == "effect"))
                    || (category == "mascot" && (item_lower == "cow" || item_lower == "animal"))
                    || (category == "scenery"
                        && (item_lower == "environment"
                            || item_lower == "env"
                            || item_lower == "road"
                            || item_lower == "mountain"))
                    || (category == "thought"
                        && (item_lower == "fortune"
                            || item_lower == "text"
                            || item_lower == "quote"))
            }),
        }
    }

    #[must_use]
    pub fn randomizes_mascot(&self) -> bool {
        self.matches("mascot")
    }

    #[must_use]
    pub fn randomizes_scenery(&self) -> bool {
        self.matches("scenery")
    }

    #[must_use]
    pub fn randomizes_fx(&self) -> bool {
        self.matches("fx")
    }

    #[must_use]
    pub fn randomizes_thought(&self) -> bool {
        self.matches("thought")
    }
}

impl std::fmt::Display for RandomSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => write!(f, "{}", if *b { "all" } else { "false" }),
            Self::String(s) => write!(f, "{s}"),
            Self::List(l) => write!(f, "{}", l.join(", ")),
        }
    }
}

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
}

impl ConfigFormat {
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "json" => Some(Self::Json),
            "yaml" | "yml" => Some(Self::Yaml),
            "toml" => Some(Self::Toml),
            _ => None,
        }
    }

    #[must_use]
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Toml => "toml",
        }
    }

    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
        }
    }
}

impl SceneConfig {
    pub fn validate(&mut self) {
        self.fps = self.fps.clamp(1, 240);
        self.duration = self.duration.min(86400);
        if self.text.len() > 4096 {
            self.text.truncate(4096);
        }
        self.cow = self
            .cow
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
    }

    pub fn parse_with_format(text: &str, format: ConfigFormat) -> Result<Self, String> {
        let mut cfg: Self = match format {
            ConfigFormat::Json => serde_json::from_str(text).map_err(|e| e.to_string())?,
            ConfigFormat::Yaml => serde_yaml::from_str(text).map_err(|e| e.to_string())?,
            ConfigFormat::Toml => toml::from_str(text).map_err(|e| e.to_string())?,
        };
        cfg.validate();
        Ok(cfg)
    }

    pub fn serialize_with_format(&self, format: ConfigFormat) -> Result<String, String> {
        match format {
            ConfigFormat::Json => serde_json::to_string_pretty(self).map_err(|e| e.to_string()),
            ConfigFormat::Yaml => serde_yaml::to_string(self).map_err(|e| e.to_string()),
            ConfigFormat::Toml => toml::to_string_pretty(self).map_err(|e| e.to_string()),
        }
    }
}

fn default_cow() -> String {
    "default".to_string()
}

fn default_effect() -> String {
    "default".to_string()
}

fn default_fps() -> u16 {
    30
}

fn default_eyes() -> String {
    "oo".to_string()
}

fn default_tongue() -> String {
    " ".to_string()
}

fn default_default_shell() -> String {
    String::new()
}

fn default_auto_render() -> bool {
    true
}

fn default_color_mode() -> String {
    "natural".to_string()
}

fn default_thought_interval() -> u32 {
    60
}

fn default_shell_attach_mode() -> String {
    "banner".to_string()
}

impl Default for SceneConfig {
    fn default() -> Self {
        Self {
            cow: default_cow(),
            text: String::new(),
            effect: default_effect(),
            background: false,
            duration: 0,
            fps: default_fps(),
            eyes: default_eyes(),
            tongue: default_tongue(),
            default_shell: default_default_shell(),
            auto_render_on_prompt: default_auto_render(),
            color_mode: default_color_mode(),
            shell_attach_mode: default_shell_attach_mode(),
            think: false,
            environment: None,
            road: None,
            mountain: None,
            palette: None,
            thought_interval: default_thought_interval(),
            split_scroll: false,
            reserve_rows: None,
            reserve_cols: None,
            split_ratio: None,
            animation: None,
            animation_type: None,
            image: None,
            split_mode: None,
            editor: None,
            random: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let s = SceneConfig::default();
        assert_eq!(s.cow, "default");
        assert_eq!(s.effect, "default");
        assert_eq!(s.fps, 30);
        assert_eq!(s.eyes, "oo");
        assert!(!s.background);
        assert!(!s.think);
        assert_eq!(s.duration, 0);
    }

    #[test]
    fn round_trip_preserves_fields() {
        let json = r#"{
            "cow": "tux",
            "text": "hello",
            "effect": "aurora",
            "background": true,
            "duration": 0,
            "fps": 60,
            "eyes": "$$",
            "tongue": "U"
        }"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.cow, "tux");
        assert_eq!(s.text, "hello");
        assert_eq!(s.effect, "aurora");
        assert!(s.background);
        assert_eq!(s.duration, 0);
        assert_eq!(s.fps, 60);
        assert_eq!(s.eyes, "$$");
        assert_eq!(s.tongue, "U");

        let back = serde_json::to_string(&s).unwrap();
        let s2: SceneConfig = serde_json::from_str(&back).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn yaml_round_trip() {
        let s = SceneConfig {
            cow: "dragon".into(),
            effect: "default".into(),
            fps: 60,
            ..SceneConfig::default()
        };
        let yaml = s.serialize_with_format(ConfigFormat::Yaml).unwrap();
        let parsed = SceneConfig::parse_with_format(&yaml, ConfigFormat::Yaml).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn toml_round_trip() {
        let s = SceneConfig {
            cow: "nyan".into(),
            effect: "default".into(),
            fps: 60,
            color_mode: "rainbow".into(),
            ..SceneConfig::default()
        };
        let toml_str = s.serialize_with_format(ConfigFormat::Toml).unwrap();
        let parsed = SceneConfig::parse_with_format(&toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(s, parsed);
    }

    #[test]
    fn unknown_fields_rejected() {
        let json = r#"{"cow":"x","sneaky":"field"}"#;
        let result: Result<SceneConfig, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let json = "{}";
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s, SceneConfig::default());
    }

    #[test]
    fn full_environmental_and_visual_properties_round_trip() {
        let full = SceneConfig {
            cow: "corgi".into(),
            text: "Hello Forgum!".into(),
            think: false,
            effect: "walk".into(),
            background: true,
            duration: 15,
            fps: 60,
            eyes: "$$".into(),
            tongue: "U ".into(),
            default_shell: "pwsh".into(),
            auto_render_on_prompt: true,
            color_mode: "rainbow".into(),
            shell_attach_mode: "split".into(),
            environment: Some("inferno".into()),
            road: Some("cobblestone".into()),
            mountain: Some("volcano".into()),
            palette: Some("#ffffff,#ff00ff,#00ffff".into()),
            thought_interval: 30,
            split_scroll: true,
            reserve_rows: Some(12),
            reserve_cols: Some(80),
            split_ratio: Some(0.35),
            animation: Some("dynamic".into()),
            animation_type: Some("walk".into()),
            image: None,
            split_mode: Some("decstbm".into()),
            editor: Some("nvim".into()),
            random: Some(RandomSetting::Bool(true)),
        };

        for format in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = full.serialize_with_format(format).expect("serialization succeeds");
            let deserialized = SceneConfig::parse_with_format(&serialized, format).expect("deserialization succeeds");
            assert_eq!(full, deserialized, "Mismatch for format {:?}", format);
        }
    }

    #[test]
    fn duration_zero_is_accepted() {
        let json = r#"{"duration":0}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.duration, 0);
    }

    #[test]
    fn random_setting_boolean_deserialization() {
        let json = r#"{"random":true}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.random, Some(RandomSetting::Bool(true)));
        let rnd = s.random.unwrap();
        assert!(rnd.is_enabled());
        assert!(rnd.randomizes_mascot());
        assert!(rnd.randomizes_scenery());
        assert!(rnd.randomizes_fx());
        assert!(rnd.randomizes_thought());
    }

    #[test]
    fn random_setting_string_deserialization() {
        let json = r#"{"random":"mascot,fx"}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.random, Some(RandomSetting::String("mascot,fx".into())));
        let rnd = s.random.unwrap();
        assert!(rnd.is_enabled());
        assert!(rnd.randomizes_mascot());
        assert!(rnd.randomizes_fx());
        assert!(!rnd.randomizes_scenery());
        assert!(!rnd.randomizes_thought());
    }

    #[test]
    fn random_setting_list_deserialization() {
        let json = r#"{"random":["scenery","thought"]}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            s.random,
            Some(RandomSetting::List(vec![
                "scenery".into(),
                "thought".into()
            ]))
        );
        let rnd = s.random.unwrap();
        assert!(rnd.is_enabled());
        assert!(!rnd.randomizes_mascot());
        assert!(rnd.randomizes_scenery());
        assert!(!rnd.randomizes_fx());
        assert!(rnd.randomizes_thought());
    }

    #[test]
    fn random_setting_toml_yaml_roundtrip() {
        let toml_str = "random = true\n";
        let parsed_toml = SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(parsed_toml.random, Some(RandomSetting::Bool(true)));

        let yaml_str = "random: all\n";
        let parsed_yaml = SceneConfig::parse_with_format(yaml_str, ConfigFormat::Yaml).unwrap();
        assert_eq!(parsed_yaml.random, Some(RandomSetting::String("all".into())));
        assert!(parsed_yaml.random.unwrap().randomizes_mascot());
    }
}
