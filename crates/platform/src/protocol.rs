//! Scene protocol — JSON schema for input/output.
//!
//! The engine reads a `SceneConfig` from stdin or `--file` (bounded to 4 MB).
//! Fields are merged with this precedence: argv > `--file` JSON > `--config`
//! JSON > built-in defaults.
//!
//! Unknown fields are gracefully ignored for backward/forward compatibility and resilient deserialization (Mandate D).

use serde::{Deserialize, Serialize};

/// The full scene description. Mirrors `default-config.json` plus per-call
/// overrides from the CLI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    /// Time of day solar illumination ("auto", "dawn", "day", "dusk", "night").
    #[serde(default)]
    pub time_of_day: Option<String>,

    /// Seasonal foliage phenology ("auto", "spring", "summer", "autumn", "winter").
    #[serde(default)]
    pub season: Option<String>,

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

    /// Background contrast profile: "standard", "acrylic_glass", "high_contrast", "vibrant".
    #[serde(default)]
    pub contrast_mode: Option<String>,

    /// Color Vision Deficiency (CVD) accessibility mode: "none", "protanopia", "deuteranopia", "tritanopia", "achromatopsia", etc.
    #[serde(
        default,
        alias = "cvd",
        alias = "colorblind",
        alias = "color_blind",
        alias = "colorblind_mode"
    )]
    pub colorblind_mode: Option<String>,

    /// Hardware acceleration mode: "auto", "dedicated", "integrated", "cpu".
    #[serde(
        default,
        alias = "acceleration_mode",
        alias = "gpu_mode",
        alias = "accel"
    )]
    pub accel_mode: Option<String>,

    /// Accessibility mode: disable kinetic motion & heavy particles (gentle fades only).
    #[serde(
        default,
        alias = "reduce-motion",
        alias = "reduced_motion",
        alias = "reduced-motion"
    )]
    pub reduce_motion: bool,

    /// Print speech bubble text directly without ASCII cow mascot art.
    #[serde(
        default,
        alias = "text-only",
        alias = "fortune_only",
        alias = "fortune-only"
    )]
    pub text_only: bool,

    /// Render inline animated banner without taking over screen.
    #[serde(default, alias = "banner_mode", alias = "banner-mode")]
    pub banner: bool,

    /// Viewport aspect ratio preset: "auto", "21:9", "16:9", "16:10", "4:3", "1:1", "narrow".
    #[serde(default, alias = "aspect_ratio", alias = "aspect-ratio")]
    pub aspect: Option<String>,

    /// Viewport aspect ratio adaptation mode: "preset", "dynamic", "auto".
    #[serde(default, alias = "aspect-mode")]
    pub aspect_mode: Option<String>,

    /// Preferred update method / package manager: "prompt" (ask each time), "auto" (detected source),
    /// or specific package manager ("zypper", "scoop", "winget", "homebrew", "apt", "dnf", "pacman", "cargo", "standalone").
    #[serde(
        default,
        alias = "update-method",
        alias = "update_channel",
        alias = "update_package_manager"
    )]
    pub update_method: Option<String>,

    /// Enable automatic update check notifications (default: true).
    #[serde(
        default = "default_check_updates",
        alias = "check-updates",
        alias = "update_check"
    )]
    pub check_updates: bool,

    /// Custom TUI keybindings configuration mapping.
    #[serde(default)]
    pub keybindings: Option<KeybindingsConfig>,

    /// Performance and frame timing diagnostic profiler mode.
    #[serde(default)]
    pub profile: bool,

    /// Sub-cell rendering mode: "ascii", "halfblock", "braille", "hybrid".
    #[serde(default)]
    pub render_mode: Option<String>,

    /// Color quantization bits for ANSI run coalescing: 0 (disabled), 4, 8, 16.
    #[serde(default)]
    pub color_quant: Option<u8>,

    /// Motivation telemetry consent: true = allowed, false = disabled.
    #[serde(
        default = "default_telemetry",
        alias = "allow_telemetry",
        alias = "telemetry_enabled"
    )]
    pub telemetry: bool,
}

/// Custom TUI keybindings configuration.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct KeybindingsConfig {
    /// Key to quit the application (default: "q", "Esc").
    #[serde(default)]
    pub quit: Option<String>,
    /// Key to save configuration and quit (default: "ZZ").
    #[serde(default)]
    pub save_and_quit: Option<String>,
    /// Key to trigger interactive fuzzy search (default: "/").
    #[serde(default)]
    pub search: Option<String>,
    /// Key to toggle Vim cheatsheet help modal (default: "?").
    #[serde(default)]
    pub help: Option<String>,
    /// Key to jump to top of lists/panels (default: "gg").
    #[serde(default)]
    pub jump_top: Option<String>,
    /// Key to jump to bottom of lists/panels (default: "G").
    #[serde(default)]
    pub jump_bottom: Option<String>,
    /// Key to cycle to next tab (default: "gt", "Tab").
    #[serde(default)]
    pub next_tab: Option<String>,
    /// Key to cycle to previous tab (default: "gT", "BackTab").
    #[serde(default)]
    pub prev_tab: Option<String>,
    /// Key to scroll down half page (default: "Ctrl+d").
    #[serde(default)]
    pub scroll_down: Option<String>,
    /// Key to scroll up half page (default: "Ctrl+u").
    #[serde(default)]
    pub scroll_up: Option<String>,
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
                        && (item_lower == "fx" || item_lower == "anim" || item_lower == "effect"))
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

fn default_check_updates() -> bool {
    true
}

fn default_telemetry() -> bool {
    true
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
            time_of_day: None,
            season: None,
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
            contrast_mode: None,
            accel_mode: None,
            reduce_motion: false,
            text_only: false,
            banner: false,
            aspect: None,
            aspect_mode: None,
            colorblind_mode: None,
            update_method: None,
            check_updates: true,
            keybindings: None,
            profile: false,
            render_mode: None,
            color_quant: None,
            telemetry: true,
        }
    }
}

/// Metadata record for a declarative configuration property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigSchemaField {
    pub name: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub desc: &'static str,
    pub type_name: &'static str,
    pub has_aliases: bool,
}

impl ConfigSchemaField {
    #[must_use]
    pub const fn new(
        name: &'static str,
        label: &'static str,
        category: &'static str,
        desc: &'static str,
        type_name: &'static str,
        has_aliases: bool,
    ) -> Self {
        Self {
            name,
            label,
            category,
            desc,
            type_name,
            has_aliases,
        }
    }
}

/// Declarative configuration definition macro.
///
/// Unifies field declaration, type definitions, defaults, serde attributes, category badges,
/// descriptions, and schema reflection into a single declarative invocation, eliminating
/// the 9-file "No-Orphan-Config Dance".
#[macro_export]
macro_rules! define_forgum_config {
    (
        $(
            $(#[$meta:meta])*
            field: $name:ident,
            type: $ty:ty,
            default: $default:expr,
            category: $category:literal,
            label: $label:literal,
            desc: $desc:literal
            $(, aliases: [$($alias:literal),* $(,)?])?
        );+ $(;)?
    ) => {
        /// Canonical compile-time schema metadata table for all declared Forgum configuration fields.
        pub const FORGUM_CONFIG_SCHEMA: &'static [$crate::protocol::ConfigSchemaField] = &[
            $(
                $crate::protocol::ConfigSchemaField::new(
                    stringify!($name),
                    $label,
                    $category,
                    $desc,
                    stringify!($ty),
                    false $( || ![$($alias),*].is_empty() )?,
                ),
            )+
        ];

        /// Look up configuration schema metadata by label or field name.
        #[must_use]
        pub fn lookup_config_schema(query: &str) -> Option<&'static $crate::protocol::ConfigSchemaField> {
            let q = query.trim().to_ascii_lowercase();
            FORGUM_CONFIG_SCHEMA.iter().find(|f| {
                f.name.eq_ignore_ascii_case(&q) || f.label.eq_ignore_ascii_case(&q)
            })
        }
    };
}

// Canonical schema definition via the declarative macro
define_forgum_config! {
    field: cow, type: String, default: default_cow(), category: "Scene", label: "cow", desc: "Mascot character art template";
    field: text, type: String, default: String::new(), category: "Scene", label: "text", desc: "Speech or thought bubble text";
    field: think, type: bool, default: false, category: "Scene", label: "think", desc: "Render as thought bubble instead of speech";
    field: effect, type: String, default: default_effect(), category: "Animation", label: "effect", desc: "Mascot animation or effect preset";
    field: background, type: bool, default: false, category: "Display", label: "background", desc: "Continuous prompt background execution mode";
    field: duration, type: u32, default: 0, category: "Display", label: "duration", desc: "Execution duration in seconds (0 = infinite / until signal)";
    field: fps, type: u16, default: default_fps(), category: "Display", label: "fps", desc: "Target refresh rate in frames per second (1..240)";
    field: eyes, type: String, default: default_eyes(), category: "Scene", label: "eyes", desc: "Mascot custom eye glyphs";
    field: tongue, type: String, default: default_tongue(), category: "Scene", label: "tongue", desc: "Mascot custom tongue glyphs";
    field: default_shell, type: String, default: default_default_shell(), category: "Shell", label: "default_shell", desc: "Default shell used for completions and hooks";
    field: auto_render_on_prompt, type: bool, default: default_auto_render(), category: "Shell", label: "auto_render_on_prompt", desc: "Automatically render banner on shell prompt";
    field: color_mode, type: String, default: default_color_mode(), category: "Display", label: "color_mode", desc: "Color rendering palette mode (natural, rainbow, solid, etc.)";
    field: shell_attach_mode, type: String, default: default_shell_attach_mode(), category: "Shell", label: "shell_attach_mode", desc: "Shell attachment mode (banner, prompt, split, clear, pane)";
    field: environment, type: Option<String>, default: None, category: "Scenery", label: "environment", desc: "Procedural scenery biome environment";
    field: road, type: Option<String>, default: None, category: "Scenery", label: "road", desc: "Procedural road surface texture style";
    field: mountain, type: Option<String>, default: None, category: "Scenery", label: "mountain", desc: "Procedural harmonic mountain horizon style";
    field: palette, type: Option<String>, default: None, category: "Display", label: "palette", desc: "Custom comma-separated hex color palette";
    field: time_of_day, type: Option<String>, default: None, category: "Scenery", label: "time_of_day", desc: "Solar illumination angle (dawn, day, dusk, night)";
    field: season, type: Option<String>, default: None, category: "Scenery", label: "season", desc: "Foliage phenology (spring, summer, autumn, winter)";
    field: thought_interval, type: u32, default: default_thought_interval(), category: "Display", label: "thought_interval", desc: "Thought quote rotation interval in seconds";
    field: split_scroll, type: bool, default: false, category: "Display", label: "split_scroll", desc: "Hardware DECSTBM margin reservation mode";
    field: reserve_rows, type: Option<u16>, default: None, category: "Display", label: "reserve_rows", desc: "Explicit terminal top rows reserved for banner";
    field: reserve_cols, type: Option<u16>, default: None, category: "Display", label: "reserve_cols", desc: "Explicit terminal columns reserved for banner";
    field: split_ratio, type: Option<f32>, default: None, category: "Display", label: "split_ratio", desc: "Ratio of terminal height reserved for animation";
    field: animation, type: Option<String>, default: None, category: "Animation", label: "animation", desc: "Animation motion mode (static, dynamic)";
    field: animation_type, type: Option<String>, default: None, category: "Animation", label: "animation_type", desc: "Specific kinematic locomotion type";
    field: image, type: Option<String>, default: None, category: "Scene", label: "image", desc: "Path to image converted to ASCII mascot art";
    field: split_mode, type: Option<String>, default: None, category: "Display", label: "split_mode", desc: "Split adapter execution mode";
    field: editor, type: Option<String>, default: None, category: "System", label: "editor", desc: "Preferred external text editor command";
    field: random, type: Option<RandomSetting>, default: None, category: "Scene", label: "random", desc: "Universal randomization policy";
    field: contrast_mode, type: Option<String>, default: None, category: "Display", label: "contrast_mode", desc: "Terminal background contrast preset profile";
    field: colorblind_mode, type: Option<String>, default: None, category: "Accessibility", label: "colorblind_mode", desc: "Color vision deficiency compensation mode", aliases: ["cvd", "colorblind", "color_blind"];
    field: accel_mode, type: Option<String>, default: None, category: "System", label: "accel_mode", desc: "Hardware acceleration compute policy", aliases: ["acceleration_mode", "gpu_mode", "accel"];
    field: reduce_motion, type: bool, default: false, category: "Accessibility", label: "reduce_motion", desc: "Disable rapid kinetic animations", aliases: ["reduced_motion", "reduced-motion"];
    field: text_only, type: bool, default: false, category: "Display", label: "text_only", desc: "Print text speech bubble without mascot art", aliases: ["fortune_only", "fortune-only"];
    field: banner, type: bool, default: false, category: "Display", label: "banner", desc: "Render fast inline startup banner", aliases: ["banner_mode", "banner-mode"];
    field: aspect, type: Option<String>, default: None, category: "Display", label: "aspect", desc: "Viewport aspect ratio preset", aliases: ["aspect_ratio", "aspect-ratio"];
    field: aspect_mode, type: Option<String>, default: None, category: "Display", label: "aspect_mode", desc: "Aspect ratio adaptation mode", aliases: ["aspect-mode"];
    field: update_method, type: Option<String>, default: None, category: "System", label: "update_method", desc: "Package manager or channel update method", aliases: ["update-method", "update_channel"];
    field: check_updates, type: bool, default: default_check_updates(), category: "System", label: "check_updates", desc: "Check for release updates on startup", aliases: ["check-updates", "update_check"];
    field: keybindings, type: Option<KeybindingsConfig>, default: None, category: "System", label: "keybindings", desc: "Custom TUI navigation and action keybindings";
    field: profile, type: bool, default: false, category: "Display", label: "profile", desc: "Live frame telemetry profiler summary on exit";
    field: render_mode, type: Option<String>, default: None, category: "Display", label: "render_mode", desc: "Sub-cell rendering canvas mode (ascii, halfblock, braille, hybrid)";
    field: color_quant, type: Option<u8>, default: None, category: "Display", label: "color_quant", desc: "Color bit quantization for ANSI escape sequence coalescing (0, 4, 8, 16)";
    field: telemetry, type: bool, default: default_telemetry(), category: "System", label: "telemetry", desc: "Motivation telemetry consent (anonymous usage counter)", aliases: ["allow_telemetry", "telemetry_enabled"];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_forgum_config_schema_completeness() {
        assert!(FORGUM_CONFIG_SCHEMA.len() >= 40);
        let fps_field = lookup_config_schema("fps").expect("fps field must exist");
        assert_eq!(fps_field.category, "Display");
        assert_eq!(fps_field.label, "fps");

        let cow_field = lookup_config_schema("cow").expect("cow field must exist");
        assert_eq!(cow_field.category, "Scene");

        let check_updates_field =
            lookup_config_schema("check_updates").expect("check_updates must exist");
        assert_eq!(check_updates_field.category, "System");

        let accel_field = lookup_config_schema("accel_mode").expect("accel_mode must exist");
        assert_eq!(accel_field.category, "System");
        assert!(accel_field.has_aliases);
    }

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
    fn unknown_fields_gracefully_ignored() {
        let json =
            r#"{"cow":"x","sneaky":"field","custom_user_extension":123,"contrast_mode":"vibrant"}"#;
        let result: Result<SceneConfig, _> = serde_json::from_str(json);
        assert!(
            result.is_ok(),
            "Expected unknown fields to be ignored gracefully"
        );
        let s = result.unwrap();
        assert_eq!(s.cow, "x");
        assert_eq!(s.contrast_mode.as_deref(), Some("vibrant"));
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
            time_of_day: Some("dusk".into()),
            season: Some("autumn".into()),
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
            contrast_mode: Some("acrylic_glass".into()),
            accel_mode: Some("auto".into()),
            reduce_motion: true,
            text_only: false,
            banner: true,
            aspect: Some("16:9".into()),
            aspect_mode: Some("dynamic".into()),
            colorblind_mode: Some("deuteranopia".into()),
            update_method: Some("prompt".into()),
            check_updates: true,
            keybindings: None,
            profile: true,
            render_mode: Some("braille".into()),
            color_quant: Some(8),
            telemetry: false,
        };

        for format in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = full
                .serialize_with_format(format)
                .expect("serialization succeeds");
            let deserialized = SceneConfig::parse_with_format(&serialized, format)
                .expect("deserialization succeeds");
            assert_eq!(full, deserialized, "Mismatch for format {:?}", format);
        }
    }

    #[test]
    fn contrast_mode_deserialization_and_roundtrip() {
        let json = r#"{"contrast_mode":"acrylic_glass"}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.contrast_mode.as_deref(), Some("acrylic_glass"));

        let toml_str = "contrast_mode = \"high_contrast\"\n";
        let s_toml = SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(s_toml.contrast_mode.as_deref(), Some("high_contrast"));
    }

    #[test]
    fn accel_mode_deserialization_and_aliases() {
        let json = r#"{"accel_mode":"dedicated"}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.accel_mode.as_deref(), Some("dedicated"));

        // Alias: acceleration_mode
        let json_alias1 = r#"{"acceleration_mode":"integrated"}"#;
        let s_alias1: SceneConfig = serde_json::from_str(json_alias1).unwrap();
        assert_eq!(s_alias1.accel_mode.as_deref(), Some("integrated"));

        // Alias: gpu_mode
        let json_alias2 = r#"{"gpu_mode":"auto"}"#;
        let s_alias2: SceneConfig = serde_json::from_str(json_alias2).unwrap();
        assert_eq!(s_alias2.accel_mode.as_deref(), Some("auto"));

        // Alias: accel
        let json_alias3 = r#"{"accel":"cpu"}"#;
        let s_alias3: SceneConfig = serde_json::from_str(json_alias3).unwrap();
        assert_eq!(s_alias3.accel_mode.as_deref(), Some("cpu"));

        let toml_str = "accel_mode = \"dedicated\"\n";
        let s_toml = SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(s_toml.accel_mode.as_deref(), Some("dedicated"));
    }

    #[test]
    fn test_reduce_motion_text_only_banner_deserialization_and_aliases() {
        let json = r#"{
            "reduce_motion": true,
            "text_only": true,
            "banner": true
        }"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert!(s.reduce_motion);
        assert!(s.text_only);
        assert!(s.banner);

        // Kebab-case and alias verification
        let json_aliases = r#"{
            "reduce-motion": true,
            "text-only": true,
            "banner-mode": true
        }"#;
        let s_alias: SceneConfig = serde_json::from_str(json_aliases).unwrap();
        assert!(s_alias.reduce_motion);
        assert!(s_alias.text_only);
        assert!(s_alias.banner);

        // TOML round-trip
        let toml_str = "reduce_motion = true\ntext_only = false\nbanner = true\n";
        let s_toml = SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert!(s_toml.reduce_motion);
        assert!(!s_toml.text_only);
        assert!(s_toml.banner);
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
        assert_eq!(
            parsed_yaml.random,
            Some(RandomSetting::String("all".into()))
        );
        assert!(parsed_yaml.random.unwrap().randomizes_mascot());
    }

    #[test]
    fn keybindings_config_roundtrip() {
        let toml_str = r#"
[keybindings]
quit = "q"
save_and_quit = "ZZ"
search = "/"
help = "?"
"#;
        let parsed: SceneConfig =
            SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert!(parsed.keybindings.is_some());
        let kb = parsed.keybindings.unwrap();
        assert_eq!(kb.quit.as_deref(), Some("q"));
        assert_eq!(kb.save_and_quit.as_deref(), Some("ZZ"));
        assert_eq!(kb.search.as_deref(), Some("/"));
        assert_eq!(kb.help.as_deref(), Some("?"));
    }

    #[test]
    fn update_config_deserialization_and_aliases() {
        let json = r#"{"update_method":"zypper","check_updates":false}"#;
        let s: SceneConfig = serde_json::from_str(json).unwrap();
        assert_eq!(s.update_method.as_deref(), Some("zypper"));
        assert!(!s.check_updates);

        let json_alias = r#"{"update-method":"scoop","update_check":true}"#;
        let s_alias: SceneConfig = serde_json::from_str(json_alias).unwrap();
        assert_eq!(s_alias.update_method.as_deref(), Some("scoop"));
        assert!(s_alias.check_updates);

        let toml_str = "update_method = \"prompt\"\ncheck_updates = true\n";
        let s_toml = SceneConfig::parse_with_format(toml_str, ConfigFormat::Toml).unwrap();
        assert_eq!(s_toml.update_method.as_deref(), Some("prompt"));
        assert!(s_toml.check_updates);
    }
}
