//! Scene protocol — JSON schema for input/output.
//!
//! The engine reads a `SceneConfig` from stdin or `--file` (bounded to 4 MB).
//! Fields are merged with this precedence: argv > `--file` JSON > `--config`
//! JSON > built-in defaults.
//!
//! Unknown fields are silently ignored (forward compatibility).

use serde::{Deserialize, Serialize};

/// The full scene description. Mirrors `default-config.json` plus per-call
/// overrides from the CLI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneConfig {
    /// Cow file basename (without `.cow`). Phase 0: only `"default"` works.
    #[serde(default = "default_cow")]
    pub cow: String,

    /// Text inside the speech bubble.
    #[serde(default)]
    pub text: String,

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
}

fn default_cow() -> String {
    "default".to_string()
}

fn default_effect() -> String {
    "static".to_string()
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
        assert_eq!(s.effect, "static");
        assert_eq!(s.fps, 30);
        assert_eq!(s.eyes, "oo");
        assert!(!s.background);
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
    fn duration_zero_is_accepted() {
        let json = r#"{"duration":0}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.duration, 0);
    }
}
