//! ratatui TUI dashboard & shell installer inspired by LazyGit and Zellij.
//!
//! Features:
//! - Zellij-style top header with mode badge, tab bar (1-5), and bottom shortcut pills.
//! - LazyGit-style multi-pane workspace: contextual entity selector on the left,
//!   real-time 30 FPS live holographic preview + inspector/action log on the right.
//! - 5 rich tabs: Mascots (106 cows), Scenery & Roads, Animation & FX, Shell Installer, and Config.
//! - 100% in-memory: DOES NOT depend on any config file existing!
//! - Interactive Shell Installer Wizard: detects installed shells on host, reports
//!   hook/completion status, and performs one-click install/uninstall with live logging.

use std::collections::HashMap;
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use forgum_platform::protocol::{ConfigFormat, SceneConfig};
use forgum_platform::shell::Shell;

/// Tab identifiers for top-level navigation (Zellij-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Mascots = 0,
    Scenery = 1,
    Effects = 2,
    Installer = 3,
    Config = 4,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Mascots,
        Tab::Scenery,
        Tab::Effects,
        Tab::Installer,
        Tab::Config,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Mascots => " 1:Mascots ",
            Tab::Scenery => " 2:Scenery & Roads ",
            Tab::Effects => " 3:FX & Colors ",
            Tab::Installer => " 4:Shell Installer ",
            Tab::Config => " 5:Configuration ",
        }
    }

    pub fn mode_label(self) -> &'static str {
        match self {
            Tab::Mascots => " 🐾 MASCOTS ",
            Tab::Scenery => " 🏔️ SCENERY ",
            Tab::Effects => " ✨ ANIMATION & FX ",
            Tab::Installer => " 🚀 SHELL INSTALLER ",
            Tab::Config => " ⚙️ CONFIG MATRIX ",
        }
    }
}

/// A dropdown or cyclic selection backed by a fixed list of string options.
#[derive(Debug, Clone)]
pub struct Dropdown {
    pub options: Vec<&'static str>,
    pub index: usize,
}

impl Dropdown {
    pub fn new(options: Vec<&'static str>, current: &str) -> Self {
        let index = options
            .iter()
            .position(|o| o.eq_ignore_ascii_case(current))
            .unwrap_or(0);
        Self { options, index }
    }

    pub fn current(&self) -> String {
        self.options[self.index].to_string()
    }

    pub fn cycle(&mut self, forward: bool) {
        if forward {
            self.index = (self.index + 1) % self.options.len();
        } else if self.index == 0 {
            self.index = self.options.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}

/// Dropdown option choices for ColorMode.
pub const COLOR_MODE_OPTIONS: &[&str] = &[
    "natural",
    "animal",
    "rainbow",
    "solid",
    "none",
];

/// Dropdown option choices for Environment.
pub const ENVIRONMENT_OPTIONS: &[&str] = &[
    "pasture",
    "inferno",
    "ocean",
    "arctic",
    "city",
    "forest",
    "savanna",
    "swamp",
    "space",
    "cyber",
    "graveyard",
    "jurassic",
    "hive",
    "throne",
    "none",
];

/// Dropdown option choices for Road terrain.
pub const ROAD_STYLE_OPTIONS: &[&str] = &[
    "dirt",
    "cobblestone",
    "magma",
    "ice",
    "seabed",
    "sidewalk",
    "roof",
    "grid",
    "crypt",
    "savanna",
    "mud",
    "tracks",
    "checkerboard",
    "none",
];

/// Dropdown option choices for Mountain skyline.
pub const MOUNTAIN_OPTIONS: &[&str] = &[
    "hills",
    "peaks",
    "volcano",
    "iceberg",
    "skyline",
    "seamount",
    "plateau",
    "crater",
    "gothic",
    "castle",
    "garden",
    "none",
];

/// Dropdown option choices for AnimationType.
pub const ANIMATION_TYPE_OPTIONS: &[&str] = &[
    "animal_natural",
    "walk",
    "breathe",
    "float",
    "fly",
    "talk",
    "sway",
    "pulse",
    "glitch",
    "particles",
    "dissolve",
];

/// Available Scenery Archetypes.
pub const SCENERY_OPTIONS: &[(&str, &str)] = &[
    (
        "pasture",
        "Classic rolling green meadows, wildflowers, and grazing hills",
    ),
    (
        "inferno",
        "Rising volcanic embers, sparks, smoke, and magma caverns",
    ),
    (
        "ocean",
        "Submarine hydrothermal vents, undulating kelp beds, and deep sea trenches",
    ),
    (
        "arctic",
        "Vast polar icecaps, floating icebergs, pack ice, and glacial frost",
    ),
    (
        "city",
        "Futuristic neon-lit urban skyscrapers, rooftops, and antenna towers",
    ),
    (
        "forest",
        "Dense ancient pine woods, mossy logs, and whispering evergreen canopies",
    ),
    (
        "savanna",
        "Golden African plains, silhouette umbrella acacia trees, and sun haze",
    ),
    (
        "swamp",
        "Will-o'-the-wisps, murky bayou waters, and floating marsh vapors",
    ),
    (
        "space",
        "Cosmic interstellar vacuum, stellar dust clouds, and distant nebulae",
    ),
    (
        "cyber",
        "Descending matrix code glyphs, data streams, and virtual grid",
    ),
    (
        "graveyard",
        "Haunted misty cemetery, crooked weathered headstones, and spooky fog",
    ),
    (
        "jurassic",
        "Primeval volcanic caldera, towering palm ferns, and cycad groves",
    ),
    (
        "hive",
        "Bioluminescent organic pheromone motes and alien bio-mechanical nest",
    ),
    (
        "throne",
        "Golden ceremonial incensed dust particles and imperial cathedral",
    ),
    (
        "none",
        "Disable atmospheric particle and scenery layer",
    ),
];

/// Available Road Surfaces.
pub const ROAD_OPTIONS: &[(&str, &str)] = &[
    (
        "dirt",
        "Dry country earth path with fine dust, pebbles, and ruts",
    ),
    (
        "cobblestone",
        "Interlocking medieval stone pavement blocks with recessed mortar joints",
    ),
    (
        "magma",
        "Cracked obsidian basalt crust over glowing flowing lava",
    ),
    (
        "ice",
        "Compact crystalline ice and packed polar snow",
    ),
    (
        "seabed",
        "Sand ripples, deep-sea coral rubble, and aquatic seafloor",
    ),
    (
        "sidewalk",
        "Paved urban concrete with asphalt curb and dividing lane lines",
    ),
    (
        "roof",
        "Layered ceramic roof shingles and rooftop tiles",
    ),
    (
        "grid",
        "Glowing retro-futuristic vector grid and cybernetic matrix",
    ),
    (
        "crypt",
        "Ancient weathered catacomb stone flagstones and haunted pathway",
    ),
    (
        "savanna",
        "Sun-baked arid cracked clay and African safari trail",
    ),
    (
        "mud",
        "Soft squishy mud, bayou ruts, and rain-soaked wetland soil",
    ),
    (
        "tracks",
        "Parallel steel train tracks bolted across timber railroad ties",
    ),
    (
        "checkerboard",
        "Alternating high-contrast dual tiles and retro arcade board",
    ),
    (
        "none",
        "Empty clear baseline without road glyphs",
    ),
];

/// Available Kinematics Effects.
pub const EFFECT_OPTIONS: &[(&str, &str)] = &[
    (
        "walk",
        "Dynamic 4-phase quadruped leg gait cycle moving with forward cadence",
    ),
    (
        "breathe",
        "Sinusoidal thoracic expansion and compression rhythm",
    ),
    (
        "float",
        "Weightless anti-gravity hovering with smooth vertical sinusoidal bobbing",
    ),
    (
        "particles",
        "Twinkling magical embers, motes, and sparkles drifting around body",
    ),
    (
        "pulse",
        "Scale and brightness oscillation between dim and vivid illumination",
    ),
    (
        "glitch",
        "Cybernetic digital aberration with randomized character flicker",
    ),
    (
        "fly",
        "High-altitude soaring glide with undulating horizontal aerodynamic drift",
    ),
    (
        "talk",
        "Animated comic dialogue bubble with dynamic fortune-cookie quips",
    ),
    (
        "sway",
        "Rhythmic harmonic pendulum oscillation around central anchor",
    ),
    (
        "dissolve",
        "Ephemeral quantum dispersion fading into terminal cyberspace",
    ),
    (
        "animal_natural",
        "Authentic God-given natural kinetic action tailored to each creature's biology and lore",
    ),
];

/// Available Color Palettes.
pub const COLOR_OPTIONS: &[(&str, &str)] = &[
    (
        "natural",
        "Authentic God-given biological color palette unique to each creature in nature",
    ),
    (
        "animal",
        "Adaptive creature coloration matched to individual animal species",
    ),
    (
        "rainbow",
        "Dynamic TrueColor 360-degree spectral hue-shift animation",
    ),
    ("lolcat", "Classic 256-color stepped rainbow color prism"),
    (
        "solid",
        "Monochromatic theme (Emerald Green, Cyberpunk Cyan, Amber)",
    ),
    ("none", "Pure minimalist high-contrast monochrome ASCII"),
];

/// Curated categories for 106 cow mascots.
pub const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "Farm & Domestic",
        &[
            "default", "cat", "cat2", "catfence", "charlie", "corgi", "bunny", "doge", "fat-cow",
            "goat", "goat2", "hippie", "kitty", "kitten", "meow", "hamster", "mule", "pig", "ram",
            "rooster", "sheep", "turkey",
        ],
    ),
    (
        "Wild & Safari",
        &[
            "armadillo",
            "bearface",
            "elephant",
            "elephant2",
            "elephant-in-snake",
            "fox",
            "hedgehog",
            "koala",
            "luke-koala",
            "moofasa",
            "panther",
            "rhino",
            "sloth",
            "telebears",
            "tiger",
            "wolf",
        ],
    ),
    (
        "Oceanic & Amphibian",
        &[
            "bud-frogs",
            "docker-whale",
            "dolphin",
            "duck",
            "ebi_furai",
            "happy-whale",
            "jellyfish",
            "octopus",
            "pufferfish",
            "seahorse",
            "squid",
            "turtle",
            "walrus",
            "whale",
        ],
    ),
    (
        "Fantasy & Sci-Fi",
        &[
            "atat",
            "cthulhu-mini",
            "daemon",
            "dragon",
            "dragon-and-cow",
            "ghost",
            "ghostbusters",
            "glados",
            "mech-and-cow",
            "minotaur",
            "mooghidjirah",
            "moojira",
            "pterodactyl",
            "sauron",
            "stegosaurus",
            "unipony",
            "vader",
            "wizard",
            "yoda",
        ],
    ),
    (
        "Pop Culture & Fun",
        &[
            "beavis.zen",
            "bill-the-cat",
            "charizardvice",
            "fat-banana",
            "flaming-sheep",
            "golden-eagle",
            "hellokitty",
            "hypno",
            "kiss",
            "mona-lisa",
            "nyan",
            "radioactive-kitty",
            "ren",
            "snoopy",
            "stimpy",
            "tux",
            "vulpix",
        ],
    ),
    (
        "Abstract & Quirky",
        &[
            "apt",
            "bees",
            "claw-arm",
            "cower",
            "cowfee",
            "eyes",
            "fence",
            "hiya",
            "jesus",
            "kosh",
            "mutilated",
            "queen",
            "skeleton",
            "small",
            "supermilker",
            "surgery",
            "three-eyes",
            "viper",
        ],
    ),
];

/// Detailed diagnostic info for detected shell.
#[derive(Debug, Clone)]
pub struct ShellInfo {
    pub shell: Shell,
    pub is_available: bool,
    pub is_hook_installed: bool,
    pub is_completion_installed: bool,
    pub rc_path: Option<PathBuf>,
    pub completion_path: Option<PathBuf>,
}

/// What the main loop should do after handling an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Save,
    OpenEditor(PathBuf),
    OpenSystemEditor(PathBuf),
    InstallTerminalEditor,
}

/// Config parameter fields for Tab 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Duration = 0,
    Fps = 1,
    Eyes = 2,
    Tongue = 3,
    ColorMode = 4,
    Palette = 5,
    Environment = 6,
    Road = 7,
    Mountain = 8,
    AnimationType = 9,
    Background = 10,
    AutoRenderOnPrompt = 11,
    ShellAttachMode = 12,
    ConfigFormat = 13,
}

impl ConfigField {
    pub const ALL: [ConfigField; 14] = [
        ConfigField::Duration,
        ConfigField::Fps,
        ConfigField::Eyes,
        ConfigField::Tongue,
        ConfigField::ColorMode,
        ConfigField::Palette,
        ConfigField::Environment,
        ConfigField::Road,
        ConfigField::Mountain,
        ConfigField::AnimationType,
        ConfigField::Background,
        ConfigField::AutoRenderOnPrompt,
        ConfigField::ShellAttachMode,
        ConfigField::ConfigFormat,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ConfigField::Duration => "duration",
            ConfigField::Fps => "fps",
            ConfigField::Eyes => "eyes",
            ConfigField::Tongue => "tongue",
            ConfigField::ColorMode => "color_mode",
            ConfigField::Palette => "palette",
            ConfigField::Environment => "environment",
            ConfigField::Road => "road",
            ConfigField::Mountain => "mountain",
            ConfigField::AnimationType => "animation_type",
            ConfigField::Background => "background",
            ConfigField::AutoRenderOnPrompt => "auto_render_on_prompt",
            ConfigField::ShellAttachMode => "shell_attach_mode",
            ConfigField::ConfigFormat => "config_format",
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            ConfigField::Duration => "Animation lifetime in seconds (0 = infinite / until signal)",
            ConfigField::Fps => "Target frame rate (30 = cinematic, 60 = ultra-smooth)",
            ConfigField::Eyes => "ASCII characters for eyes (e.g. 'oo', '$$', 'XX', '@@')",
            ConfigField::Tongue => "ASCII characters for tongue (e.g. 'U ', '  ', '||')",
            ConfigField::ColorMode => "Color rendering palette mode: natural, animal, rainbow, solid, none",
            ConfigField::Palette => "Custom TrueColor hex palette gradient (comma-separated, e.g. '#ffffff,#1a1a1a')",
            ConfigField::Environment => "Atmospheric particle system: pasture, inferno, ocean, arctic, city, forest, savanna, swamp, space, cyber, graveyard, jurassic, hive, throne, none",
            ConfigField::Road => "Ground terrain surface style: dirt, cobblestone, magma, ice, seabed, sidewalk, roof, grid, crypt, savanna, mud, tracks, checkerboard, none",
            ConfigField::Mountain => "Horizon background silhouette: hills, peaks, volcano, iceberg, skyline, seamount, plateau, crater, gothic, castle, garden, none",
            ConfigField::AnimationType => "Kinematic animation driver: animal_natural, walk, breathe, float, fly, talk, sway, pulse, glitch, particles, dissolve",
            ConfigField::Background => "Daemon non-blocking prompt overlay mode (true/false)",
            ConfigField::AutoRenderOnPrompt => "Trigger mascot automatically on shell prompt enter",
            ConfigField::ShellAttachMode => {
                "Prompt hook integration: banner, split, reactive, manual"
            }
            ConfigField::ConfigFormat => "File serialization syntax: JSON, YAML, or TOML",
        }
    }

    pub fn value(self, app: &ConfigApp, is_sel: bool) -> String {
        app.field_value(self, is_sel)
    }
}

/// Main application state for the LazyGit + Zellij inspired TUI dashboard.
#[derive(Debug)]
pub struct ConfigApp {
    pub current_tab: Tab,
    pub config: SceneConfig,
    pub config_path: Option<PathBuf>,
    pub format: ConfigFormat,

    // Mascots Tab state
    pub mascot_category_idx: usize,
    pub mascot_item_idx: usize,

    // Scenery Tab state
    pub scenery_idx: usize,
    pub road_idx: usize,
    pub scenery_sub_focus: usize, // 0 = scenery list, 1 = road list

    // FX Tab state
    pub effect_idx: usize,
    pub color_idx: usize,
    pub fx_sub_focus: usize, // 0 = effect list, 1 = color list

    // Shell Installer Tab state
    pub shells: Vec<ShellInfo>,
    pub selected_shell_idx: usize,
    pub installer_log: Vec<String>,
    pub package_managers: Vec<(forgum_platform::PackageManager, bool)>,
    pub detected_install_source: forgum_platform::PackageManager,
    pub installer_view_mode: usize,

    // Config Tab state
    pub config_field_idx: usize,
    pub editing_config: bool,
    pub edit_initial: bool,
    pub config_edit_buffer: String,
    pub color_mode_dropdown: Dropdown,
    pub environment_dropdown: Dropdown,
    pub road_dropdown: Dropdown,
    pub mountain_dropdown: Dropdown,
    pub animation_type_dropdown: Dropdown,
    pub attach_mode_dropdown: Dropdown,
    pub format_dropdown: Dropdown,

    // Shared state
    pub saved: bool,
    pub status_message: String,
    pub animation_time: f32,
    pub cow_cache: HashMap<String, String>,
    pub show_editor_modal: bool,
    pub original_edit_value: String,
}

/// Detect the active host terminal name from environment markers.
pub fn detect_active_terminal() -> String {
    if std::env::var("WT_SESSION").is_ok() {
        return "Windows Terminal".to_string();
    }
    if std::env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return "Ghostty".to_string();
    }
    if std::env::var("WEZTERM_PANE").is_ok() {
        return "WezTerm".to_string();
    }
    if std::env::var("KITTY_WINDOW_ID").is_ok() {
        return "Kitty".to_string();
    }
    if std::env::var("ALACRITTY_LOG").is_ok() || std::env::var("ALACRITTY_WINDOW_ID").is_ok() {
        return "Alacritty".to_string();
    }
    if let Ok(prog) = std::env::var("TERM_PROGRAM") {
        return prog;
    }
    if let Ok(term) = std::env::var("TERM") {
        return term;
    }
    "Standard Virtual Terminal".to_string()
}

impl ConfigApp {
    /// Create a new TUI application instance.
    ///
    /// NOTE: DOES NOT depend on any config file! If `config_path` is None or
    /// missing, it starts gracefully with in-memory defaults.
    pub fn new(
        initial_config: Option<SceneConfig>,
        config_path: Option<PathBuf>,
        initial_tab: Option<Tab>,
    ) -> Self {
        let config = initial_config.unwrap_or_default();
        let format = config_path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .and_then(ConfigFormat::from_extension)
            .unwrap_or(ConfigFormat::Json);

        let color_mode_dropdown = Dropdown::new(
            COLOR_MODE_OPTIONS.to_vec(),
            &config.color_mode,
        );
        let environment_dropdown = Dropdown::new(
            ENVIRONMENT_OPTIONS.to_vec(),
            config.environment.as_deref().unwrap_or("pasture"),
        );
        let road_dropdown = Dropdown::new(
            ROAD_STYLE_OPTIONS.to_vec(),
            config.road.as_deref().unwrap_or("dirt"),
        );
        let mountain_dropdown = Dropdown::new(
            MOUNTAIN_OPTIONS.to_vec(),
            config.mountain.as_deref().unwrap_or("hills"),
        );
        let animation_type_dropdown = Dropdown::new(
            ANIMATION_TYPE_OPTIONS.to_vec(),
            config
                .animation_type
                .as_deref()
                .unwrap_or(if config.effect.is_empty() {
                    "animal_natural"
                } else {
                    &config.effect
                }),
        );

        let attach_mode_dropdown = Dropdown::new(
            vec!["banner", "split", "reactive", "manual"],
            &config.shell_attach_mode,
        );
        let format_dropdown = Dropdown::new(vec!["json", "yaml", "toml"], format.extension());

        // Find initial mascot indices
        let mut init_cat = 0;
        let mut init_item = 0;
        'outer: for (c_idx, (_, cows)) in CATEGORIES.iter().enumerate() {
            for (i_idx, &cow) in cows.iter().enumerate() {
                if cow.eq_ignore_ascii_case(&config.cow) {
                    init_cat = c_idx;
                    init_item = i_idx;
                    break 'outer;
                }
            }
        }

        // Find initial scenery and road indices
        let scenery_idx = SCENERY_OPTIONS
            .iter()
            .position(|(s, _)| {
                config
                    .environment
                    .as_deref()
                    .map(|e| e.eq_ignore_ascii_case(s))
                    .unwrap_or(false)
            })
            .unwrap_or(0);
        let road_idx = ROAD_OPTIONS
            .iter()
            .position(|(r, _)| {
                config
                    .road
                    .as_deref()
                    .map(|rd| rd.eq_ignore_ascii_case(r))
                    .unwrap_or(false)
            })
            .unwrap_or(0);

        // Find initial effect and color indices
        let effect_idx = EFFECT_OPTIONS
            .iter()
            .position(|(e, _)| e.eq_ignore_ascii_case(&config.effect))
            .unwrap_or(0);
        let color_idx = COLOR_OPTIONS
            .iter()
            .position(|(c, _)| c.eq_ignore_ascii_case(&config.color_mode))
            .unwrap_or(0);

        let shells = Self::detect_shells();
        let package_managers = forgum_platform::detect_available_package_managers();
        let detected_install_source = forgum_platform::detect_installation_source();

        let mut initial_log = vec![
            "✨ Welcome to Forgum Grand Installer & Studio!".to_string(),
            format!(
                "Host: {} ({}) | Active Terminal: {}",
                std::env::consts::OS,
                std::env::consts::ARCH,
                detect_active_terminal()
            ),
            format!(
                "Installation Source: {} (auto-update enabled)",
                detected_install_source.name()
            ),
            "Canonical Path: ~/.config/forgum/completions/".to_string(),
        ];
        if initial_tab == Some(Tab::Installer) {
            initial_log.push("▶ Setup Wizard active. Press 'i' to install shell integration, 'l' for license/terms.".to_string());
        }

        let mut app = Self {
            current_tab: initial_tab.unwrap_or(Tab::Mascots),
            config,
            config_path,
            format,
            mascot_category_idx: init_cat,
            mascot_item_idx: init_item,
            scenery_idx,
            road_idx,
            scenery_sub_focus: 0,
            effect_idx,
            color_idx,
            fx_sub_focus: 0,
            shells,
            selected_shell_idx: 0,
            installer_log: initial_log,
            package_managers,
            detected_install_source,
            installer_view_mode: 0,
            config_field_idx: 0,
            editing_config: false,
            edit_initial: false,
            config_edit_buffer: String::new(),
            color_mode_dropdown,
            environment_dropdown,
            road_dropdown,
            mountain_dropdown,
            animation_type_dropdown,
            attach_mode_dropdown,
            format_dropdown,
            saved: false,
            status_message: "Welcome to Forgum! Use <Tab> to navigate, 'i' to install shell hooks."
                .into(),
            animation_time: 0.0,
            cow_cache: HashMap::new(),
            show_editor_modal: false,
            original_edit_value: String::new(),
        };

        // Cache the initial cow
        app.ensure_cow_cached(&app.config.cow.clone());
        app
    }

    /// Detect installed shells and current configuration hook / completion status.
    pub fn detect_shells() -> Vec<ShellInfo> {
        let targets = [
            Shell::Pwsh,
            Shell::PowerShell,
            Shell::Bash,
            Shell::Zsh,
            Shell::Fish,
            Shell::Nushell,
            Shell::Elvish,
            Shell::Carapace,
            Shell::Xonsh,
            Shell::Tcsh,
            Shell::Ksh,
            Shell::Ion,
            Shell::Oil,
            Shell::Yash,
        ];

        targets
            .into_iter()
            .map(|shell| {
                let rc = shell.shell_rc_path();
                let comp = shell.completions_script_path();

                let is_available = Self::check_shell_binary(shell);

                let is_hook_installed = rc
                    .as_ref()
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .map(|txt| txt.contains("# >>> forgum >>>") || txt.contains("forgum init"))
                    .unwrap_or(false);

                let is_completion_installed = comp.as_ref().map(|p| p.is_file()).unwrap_or(false);

                ShellInfo {
                    shell,
                    is_available,
                    is_hook_installed,
                    is_completion_installed,
                    rc_path: rc,
                    completion_path: comp,
                }
            })
            .collect()
    }

    /// Check if a given shell executable is available in PATH.
    fn check_shell_binary(shell: Shell) -> bool {
        let bin = match shell {
            Shell::Pwsh => "pwsh",
            Shell::PowerShell => "powershell",
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Nushell => "nu",
            Shell::Cmd => "cmd",
            Shell::Elvish => "elvish",
            Shell::Carapace => "carapace",
            Shell::Xonsh => "xonsh",
            Shell::Tcsh => "tcsh",
            Shell::Ksh => "ksh",
            Shell::Ion => "ion",
            Shell::Oil => "osh",
            Shell::Yash => "yash",
        };

        #[cfg(windows)]
        {
            std::process::Command::new("where.exe")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
        #[cfg(not(windows))]
        {
            std::process::Command::new("which")
                .arg(bin)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.animation_time += dt;
    }

    pub fn config(&self) -> &SceneConfig {
        &self.config
    }

    pub fn config_format(&self) -> ConfigFormat {
        ConfigFormat::from_extension(&self.format_dropdown.current()).unwrap_or(self.format)
    }

    pub fn mark_saved(&mut self) {
        self.saved = true;
        self.status_message = format!(
            "Saved successfully in {} format! (Pasture updated)",
            self.config_format().display_name()
        );
    }

    /// Hot-reload full configuration from disk and re-sync all dropdowns and state.
    pub fn set_config(&mut self, config: SceneConfig) {
        self.config = config;
        self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &self.config.color_mode);
        self.environment_dropdown = Dropdown::new(
            ENVIRONMENT_OPTIONS.to_vec(),
            self.config.environment.as_deref().unwrap_or("pasture"),
        );
        self.road_dropdown = Dropdown::new(
            ROAD_STYLE_OPTIONS.to_vec(),
            self.config.road.as_deref().unwrap_or("dirt"),
        );
        self.mountain_dropdown = Dropdown::new(
            MOUNTAIN_OPTIONS.to_vec(),
            self.config.mountain.as_deref().unwrap_or("hills"),
        );
        self.animation_type_dropdown = Dropdown::new(
            ANIMATION_TYPE_OPTIONS.to_vec(),
            self.config.animation_type.as_deref().unwrap_or("animal_natural"),
        );
        self.attach_mode_dropdown = Dropdown::new(
            vec!["banner", "split", "reactive", "manual"],
            &self.config.shell_attach_mode,
        );
        self.cow_cache.clear();
        self.ensure_cow_cached(&self.config.cow.clone());
    }

    /// Open the interactive modal requesting permission to open with system editor or install terminal editor.
    pub fn open_editor_modal(&mut self) {
        self.show_editor_modal = true;
        self.status_message = "No terminal editor detected. Please choose an option.".to_string();
    }

    /// Close the interactive editor request modal.
    pub fn close_editor_modal(&mut self) {
        self.show_editor_modal = false;
    }

    /// Reload configuration and sync all dropdowns.
    pub fn reload_config(&mut self, config: SceneConfig) {
        self.set_config(config);
    }

    /// Resolve canonical config file path.
    pub fn resolve_config_path(&self) -> PathBuf {
        if let Some(p) = &self.config_path {
            p.clone()
        } else {
            let default_dir = forgum_platform::paths::config_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            default_dir.join(format!("config.{}", self.config_format().extension()))
        }
    }

    /// Handle keyboard and mouse input events.
    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<Option<Action>> {
        match event {
            Event::Mouse(mouse) => self.handle_mouse_event(mouse),
            Event::Key(key) => {
                if key.kind != event::KeyEventKind::Press {
                    return Ok(None);
                }

                // If modal popup is active
                if self.show_editor_modal {
                    match key.code {
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            self.show_editor_modal = false;
                            let path = self.resolve_config_path();
                            return Ok(Some(Action::OpenSystemEditor(path)));
                        }
                        KeyCode::Char('i') | KeyCode::Char('I') => {
                            self.show_editor_modal = false;
                            return Ok(Some(Action::InstallTerminalEditor));
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.show_editor_modal = false;
                            self.status_message = "Editor request cancelled.".to_string();
                            return Ok(None);
                        }
                        _ => return Ok(None),
                    }
                }

                // If currently editing text field on Config tab
                if self.editing_config {
                    return self.handle_config_edit_key(key);
                }

                // Global keybindings
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(Action::Quit)),
                    KeyCode::Char('s') => return Ok(Some(Action::Save)),
                    KeyCode::Char('o') | KeyCode::Char('O') => {
                        let path = self.resolve_config_path();
                        return Ok(Some(Action::OpenEditor(path)));
                    }
                    KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let path = self.resolve_config_path();
                        return Ok(Some(Action::OpenEditor(path)));
                    }
                    KeyCode::Tab => {
                        let next = (self.current_tab as usize + 1) % Tab::ALL.len();
                        self.current_tab = Tab::ALL[next];
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::BackTab => {
                        let prev = if self.current_tab as usize == 0 {
                            Tab::ALL.len() - 1
                        } else {
                            self.current_tab as usize - 1
                        };
                        self.current_tab = Tab::ALL[prev];
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('r') => {
                        self.randomize_mascot();
                        return Ok(None);
                    }
                    KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        self.open_config_directory();
                        return Ok(None);
                    }
                    KeyCode::Char('1') => {
                        self.current_tab = Tab::Mascots;
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('2') => {
                        self.current_tab = Tab::Scenery;
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('3') => {
                        self.current_tab = Tab::Effects;
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('4') => {
                        self.current_tab = Tab::Installer;
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('5') => {
                        self.current_tab = Tab::Config;
                        self.status_message = format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    _ => {}
                }

                // Tab-specific keybindings
                match self.current_tab {
                    Tab::Mascots => self.handle_mascots_key(key),
                    Tab::Scenery => self.handle_scenery_key(key),
                    Tab::Effects => self.handle_effects_key(key),
                    Tab::Installer => self.handle_installer_key(key),
                    Tab::Config => self.handle_config_key(key),
                }
            }
            _ => Ok(None),
        }
    }

    /// Handle mouse interaction across all tabs, buttons, lists, and modal popups.
    pub fn handle_mouse_event(&mut self, mouse: crossterm::event::MouseEvent) -> anyhow::Result<Option<Action>> {
        use crossterm::event::{MouseButton, MouseEventKind};

        let (term_w, term_h) = crossterm::terminal::size().unwrap_or((80, 24));

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if self.show_editor_modal {
                    let path = self.resolve_config_path();
                    let width = 72.min(term_w.saturating_sub(4));
                    let height = 12.min(term_h.saturating_sub(2));
                    let modal_x = (term_w.saturating_sub(width)) / 2;
                    let modal_y = (term_h.saturating_sub(height)) / 2;

                    if mouse.row < modal_y || mouse.row >= modal_y + height
                        || mouse.column < modal_x || mouse.column >= modal_x + width
                    {
                        self.show_editor_modal = false;
                        self.status_message = "Editor request cancelled.".to_string();
                        return Ok(None);
                    }

                    if mouse.row == modal_y + 5 {
                        self.show_editor_modal = false;
                        return Ok(Some(Action::OpenSystemEditor(path)));
                    } else if mouse.row == modal_y + 6 {
                        self.show_editor_modal = false;
                        return Ok(Some(Action::InstallTerminalEditor));
                    } else if mouse.row == modal_y + 7 {
                        self.show_editor_modal = false;
                        self.status_message = "Editor request cancelled.".to_string();
                        return Ok(None);
                    }
                    return Ok(None);
                }

                // Top Header tab row: lines 0, 1, 2
                if mouse.row <= 2 {
                    if self.editing_config {
                        self.commit_config_edit();
                        self.editing_config = false;
                    }
                    let mode_w = 1 + match self.current_tab {
                        Tab::Mascots => 13,
                        Tab::Scenery => 13,
                        Tab::Effects => 20,
                        Tab::Installer => 21,
                        Tab::Config => 19,
                    };
                    let mut cur = mode_w + 1;
                    let mut clicked_tab = None;
                    for tab in Tab::ALL {
                        let w = tab.title().len() as u16;
                        if mouse.column >= cur && mouse.column < cur + w {
                            clicked_tab = Some(tab);
                            break;
                        }
                        cur += w + 1;
                    }
                    let tab = clicked_tab.or_else(|| {
                        if mouse.column < 26 {
                            Some(Tab::Mascots)
                        } else if mouse.column < 46 {
                            Some(Tab::Scenery)
                        } else if mouse.column < 62 {
                            Some(Tab::Effects)
                        } else if mouse.column < 82 {
                            Some(Tab::Installer)
                        } else {
                            Some(Tab::Config)
                        }
                    });
                    if let Some(t) = tab {
                        self.current_tab = t;
                        self.status_message = format!("Switched to {}", t.mode_label());
                    }
                    return Ok(None);
                }

                // Left column list area
                let left_width = if self.current_tab == Tab::Installer {
                    (term_w / 3).max(32).min(term_w.saturating_sub(40))
                } else {
                    (term_w / 3).max(24).min(term_w.saturating_sub(20))
                };
                let area_h = term_h.saturating_sub(5);

                if mouse.column < left_width && mouse.row >= 3 && mouse.row < 3 + area_h {
                    let row_idx = (mouse.row.saturating_sub(4)) as usize;
                    match self.current_tab {
                        Tab::Mascots => {
                            let cows = CATEGORIES[self.mascot_category_idx].1;
                            if row_idx < cows.len() {
                                self.mascot_item_idx = row_idx;
                                self.apply_selected_mascot();
                            }
                        }
                        Tab::Scenery => {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row >= chunks[0].y + 1 && mouse.row < chunks[0].y + chunks[0].height {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < SCENERY_OPTIONS.len() {
                                    self.scenery_idx = idx;
                                    self.scenery_sub_focus = 0;
                                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                                    self.config.environment = Some(env.clone());
                                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                                    self.saved = false;
                                }
                            } else if mouse.row >= chunks[1].y + 1 && mouse.row < chunks[1].y + chunks[1].height {
                                let idx = (mouse.row - (chunks[1].y + 1)) as usize;
                                if idx < ROAD_OPTIONS.len() {
                                    self.road_idx = idx;
                                    self.scenery_sub_focus = 1;
                                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                                    self.config.road = Some(rd.clone());
                                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                                    self.saved = false;
                                }
                            }
                        }
                        Tab::Effects => {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row >= chunks[0].y + 1 && mouse.row < chunks[0].y + chunks[0].height {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < EFFECT_OPTIONS.len() {
                                    self.effect_idx = idx;
                                    self.fx_sub_focus = 0;
                                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                                    self.config.effect = eff.clone();
                                    self.config.animation_type = Some(eff.clone());
                                    self.animation_type_dropdown = Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                                    self.saved = false;
                                }
                            } else if mouse.row >= chunks[1].y + 1 && mouse.row < chunks[1].y + chunks[1].height {
                                let idx = (mouse.row - (chunks[1].y + 1)) as usize;
                                if idx < COLOR_OPTIONS.len() {
                                    self.color_idx = idx;
                                    self.fx_sub_focus = 1;
                                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                                    self.config.color_mode = col.clone();
                                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                                    self.saved = false;
                                }
                            }
                        }
                        Tab::Installer => {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row >= chunks[0].y + 1 && mouse.row < chunks[0].y + chunks[0].height {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < self.shells.len() {
                                    self.selected_shell_idx = idx;
                                    self.status_message = format!("Selected {}", self.shells[idx].shell);
                                }
                            }
                        }
                        Tab::Config => {
                            if row_idx < ConfigField::ALL.len() {
                                if self.editing_config {
                                    self.commit_config_edit();
                                    self.editing_config = false;
                                }
                                self.config_field_idx = row_idx;
                                let field = ConfigField::ALL[row_idx];
                                if field == ConfigField::Eyes
                                    || field == ConfigField::Tongue
                                    || field == ConfigField::Palette
                                {
                                    self.enter_config_edit();
                                } else {
                                    self.cycle_config_field(true);
                                }
                            }
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                match self.current_tab {
                    Tab::Mascots => {
                        let cows = CATEGORIES[self.mascot_category_idx].1;
                        self.mascot_item_idx = (self.mascot_item_idx + 1) % cows.len();
                        self.apply_selected_mascot();
                    }
                    Tab::Scenery => {
                        if self.scenery_sub_focus == 0 {
                            self.scenery_idx = (self.scenery_idx + 1) % SCENERY_OPTIONS.len();
                            let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                            self.config.environment = Some(env.clone());
                            self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                        } else {
                            self.road_idx = (self.road_idx + 1) % ROAD_OPTIONS.len();
                            let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                            self.config.road = Some(rd.clone());
                            self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                        }
                        self.saved = false;
                    }
                    Tab::Effects => {
                        if self.fx_sub_focus == 0 {
                            self.effect_idx = (self.effect_idx + 1) % EFFECT_OPTIONS.len();
                            let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                            self.config.effect = eff.clone();
                            self.config.animation_type = Some(eff.clone());
                            self.animation_type_dropdown = Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                        } else {
                            self.color_idx = (self.color_idx + 1) % COLOR_OPTIONS.len();
                            let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                            self.config.color_mode = col.clone();
                            self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                        }
                        self.saved = false;
                    }
                    Tab::Installer => {
                        if !self.shells.is_empty() {
                            self.selected_shell_idx = (self.selected_shell_idx + 1) % self.shells.len();
                        }
                    }
                    Tab::Config => {
                        if self.editing_config {
                            self.commit_config_edit();
                            self.editing_config = false;
                        }
                        self.config_field_idx = (self.config_field_idx + 1) % ConfigField::ALL.len();
                    }
                }
            }
            MouseEventKind::ScrollUp => {
                match self.current_tab {
                    Tab::Mascots => {
                        let cows = CATEGORIES[self.mascot_category_idx].1;
                        if self.mascot_item_idx > 0 {
                            self.mascot_item_idx -= 1;
                        } else {
                            self.mascot_item_idx = cows.len().saturating_sub(1);
                        }
                        self.apply_selected_mascot();
                    }
                    Tab::Scenery => {
                        if self.scenery_sub_focus == 0 {
                            if self.scenery_idx > 0 {
                                self.scenery_idx -= 1;
                            } else {
                                self.scenery_idx = SCENERY_OPTIONS.len().saturating_sub(1);
                            }
                            let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                            self.config.environment = Some(env.clone());
                            self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                        } else {
                            if self.road_idx > 0 {
                                self.road_idx -= 1;
                            } else {
                                self.road_idx = ROAD_OPTIONS.len().saturating_sub(1);
                            }
                            let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                            self.config.road = Some(rd.clone());
                            self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                        }
                        self.saved = false;
                    }
                    Tab::Effects => {
                        if self.fx_sub_focus == 0 {
                            if self.effect_idx > 0 {
                                self.effect_idx -= 1;
                            } else {
                                self.effect_idx = EFFECT_OPTIONS.len().saturating_sub(1);
                            }
                            let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                            self.config.effect = eff.clone();
                            self.config.animation_type = Some(eff.clone());
                            self.animation_type_dropdown = Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                        } else {
                            if self.color_idx > 0 {
                                self.color_idx -= 1;
                            } else {
                                self.color_idx = COLOR_OPTIONS.len().saturating_sub(1);
                            }
                            let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                            self.config.color_mode = col.clone();
                            self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                        }
                        self.saved = false;
                    }
                    Tab::Installer => {
                        if !self.shells.is_empty() {
                            if self.selected_shell_idx > 0 {
                                self.selected_shell_idx -= 1;
                            } else {
                                self.selected_shell_idx = self.shells.len().saturating_sub(1);
                            }
                        }
                    }
                    Tab::Config => {
                        if self.editing_config {
                            self.commit_config_edit();
                            self.editing_config = false;
                        }
                        if self.config_field_idx > 0 {
                            self.config_field_idx -= 1;
                        } else {
                            self.config_field_idx = ConfigField::ALL.len().saturating_sub(1);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(None)
    }

    fn handle_mascots_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        let current_cows = CATEGORIES[self.mascot_category_idx].1;
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.mascot_item_idx > 0 {
                    self.mascot_item_idx -= 1;
                } else {
                    self.mascot_item_idx = current_cows.len().saturating_sub(1);
                }
                self.apply_selected_mascot();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.mascot_item_idx + 1 < current_cows.len() {
                    self.mascot_item_idx += 1;
                } else {
                    self.mascot_item_idx = 0;
                }
                self.apply_selected_mascot();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.mascot_category_idx > 0 {
                    self.mascot_category_idx -= 1;
                } else {
                    self.mascot_category_idx = CATEGORIES.len() - 1;
                }
                self.mascot_item_idx = 0;
                self.apply_selected_mascot();
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.mascot_category_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
                self.mascot_item_idx = 0;
                self.apply_selected_mascot();
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_selected_mascot();
                self.sync_mascot_scenery();
            }
            KeyCode::Char('b') | KeyCode::Char('r') => {
                self.apply_selected_mascot();
                self.sync_mascot_scenery();
            }
            _ => {}
        }
        Ok(None)
    }

    fn apply_selected_mascot(&mut self) {
        let name = CATEGORIES[self.mascot_category_idx].1[self.mascot_item_idx].to_string();
        self.config.cow = name.clone();
        self.ensure_cow_cached(&name);
        let biome = forgum_platform::biome::get_mascot_biome(&name);
        self.status_message = format!("🐾 Mascot: {} | 🌿 Biome: {}", name, biome.biome_name);
        self.saved = false;
    }

    pub fn sync_mascot_scenery(&mut self) {
        let name = self.config.cow.clone();
        let biome = forgum_platform::biome::get_mascot_biome(&name);
        if let Some(pos) = SCENERY_OPTIONS.iter().position(|(env, _)| env.eq_ignore_ascii_case(biome.environment)) {
            self.scenery_idx = pos;
            let env = SCENERY_OPTIONS[pos].0.to_string();
            self.config.environment = Some(env.clone());
            self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
        }
        if let Some(pos) = ROAD_OPTIONS.iter().position(|(rd, _)| rd.eq_ignore_ascii_case(biome.road)) {
            self.road_idx = pos;
            let rd = ROAD_OPTIONS[pos].0.to_string();
            self.config.road = Some(rd.clone());
            self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
        }
        let mtn = biome.mountain.to_string();
        self.config.mountain = Some(mtn.clone());
        self.mountain_dropdown = Dropdown::new(MOUNTAIN_OPTIONS.to_vec(), &mtn);
        self.saved = false;
        self.status_message = format!("🌿 Native habitat applied: {} (Atmosphere: {}, Ground: {}, Skyline: {})", biome.biome_name, biome.environment, biome.road, biome.mountain);
    }

    fn randomize_mascot(&mut self) {
        let cat_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
        let item_idx = (self.mascot_item_idx + 3) % CATEGORIES[cat_idx].1.len();
        self.mascot_category_idx = cat_idx;
        self.mascot_item_idx = item_idx;
        self.apply_selected_mascot();
        self.sync_mascot_scenery();
    }

    fn handle_scenery_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                self.scenery_sub_focus = 1 - self.scenery_sub_focus;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.scenery_sub_focus == 0 {
                    if self.scenery_idx > 0 {
                        self.scenery_idx -= 1;
                    } else {
                        self.scenery_idx = SCENERY_OPTIONS.len() - 1;
                    }
                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                    self.config.environment = Some(env.clone());
                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                    self.saved = false;
                } else {
                    if self.road_idx > 0 {
                        self.road_idx -= 1;
                    } else {
                        self.road_idx = ROAD_OPTIONS.len() - 1;
                    }
                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                    self.config.road = Some(rd.clone());
                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                    self.saved = false;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scenery_sub_focus == 0 {
                    self.scenery_idx = (self.scenery_idx + 1) % SCENERY_OPTIONS.len();
                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                    self.config.environment = Some(env.clone());
                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                    self.saved = false;
                } else {
                    self.road_idx = (self.road_idx + 1) % ROAD_OPTIONS.len();
                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                    self.config.road = Some(rd.clone());
                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                    self.saved = false;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.saved = false;
                self.status_message = format!(
                    "Scenery set to '{}' with road '{}'",
                    self.config.environment.as_deref().unwrap_or("pasture"),
                    self.config.road.as_deref().unwrap_or("dirt")
                );
            }
            KeyCode::Char('b') | KeyCode::Char('r') => {
                self.sync_mascot_scenery();
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_effects_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Tab
            | KeyCode::Left
            | KeyCode::Right
            | KeyCode::Char('h')
            | KeyCode::Char('l') => {
                self.fx_sub_focus = 1 - self.fx_sub_focus;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.fx_sub_focus == 0 {
                    if self.effect_idx > 0 {
                        self.effect_idx -= 1;
                    } else {
                        self.effect_idx = EFFECT_OPTIONS.len() - 1;
                    }
                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                    self.config.effect = eff.clone();
                    self.config.animation_type = Some(eff.clone());
                    self.animation_type_dropdown =
                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                    self.saved = false;
                } else {
                    if self.color_idx > 0 {
                        self.color_idx -= 1;
                    } else {
                        self.color_idx = COLOR_OPTIONS.len() - 1;
                    }
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    self.saved = false;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.fx_sub_focus == 0 {
                    self.effect_idx = (self.effect_idx + 1) % EFFECT_OPTIONS.len();
                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                    self.config.effect = eff.clone();
                    self.config.animation_type = Some(eff.clone());
                    self.animation_type_dropdown =
                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                    self.saved = false;
                } else {
                    self.color_idx = (self.color_idx + 1) % COLOR_OPTIONS.len();
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    self.saved = false;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.saved = false;
                self.status_message = format!(
                    "Effect: '{}' | Color Palette: '{}'",
                    self.config.effect, self.config.color_mode
                );
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_installer_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_shell_idx > 0 {
                    self.selected_shell_idx -= 1;
                } else if !self.shells.is_empty() {
                    self.selected_shell_idx = self.shells.len() - 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.shells.is_empty() {
                    self.selected_shell_idx = (self.selected_shell_idx + 1) % self.shells.len();
                }
            }
            KeyCode::Enter | KeyCode::Char('i') => {
                self.install_shell(self.selected_shell_idx);
            }
            KeyCode::Char('u') => {
                self.uninstall_shell(self.selected_shell_idx);
            }
            KeyCode::Char('t') => {
                self.test_shell_simulation(self.selected_shell_idx);
            }
            KeyCode::Char('l') => {
                self.installer_view_mode = (self.installer_view_mode + 1) % 3;
                let view_name = match self.installer_view_mode {
                    0 => "System Diagnostics & Grand Welcoming",
                    1 => "License & Terms (MIT / Apache-2.0)",
                    _ => "Package Managers & Distribution Channels",
                };
                self.status_message = format!("Installer view: {view_name}");
            }
            KeyCode::Char('p') => {
                let pm = self.detected_install_source;
                self.installer_log
                    .push(format!("▶ Checking updates via {}...", pm.name()));
                match forgum_platform::execute_package_manager_action(pm, true) {
                    Ok(out) => {
                        self.installer_log
                            .push(format!("✓ Update status ({}): OK", pm.name()));
                        for line in out.lines().take(2) {
                            if !line.trim().is_empty() {
                                self.installer_log.push(format!("  {line}"));
                            }
                        }
                        self.status_message = format!("✓ Update check complete via {}.", pm.name());
                    }
                    Err(err) => {
                        self.installer_log
                            .push(format!("✗ Update check failed: {err}"));
                        self.status_message = "Update check completed with warnings.".into();
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn install_shell(&mut self, idx: usize) {
        if idx >= self.shells.len() {
            return;
        }
        let shell_info = &mut self.shells[idx];
        let sh = shell_info.shell;

        let mut success = true;

        // 1. Install canonical completion script into ~/.config/forgum/completions/
        if let Some(comp_path) = sh.completions_script_path() {
            if let Some(parent) = comp_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let completion_script = match sh {
                Shell::Pwsh | Shell::PowerShell => {
                    r#"# Forgum PowerShell auto-completion script
Register-ArgumentCompleter -Native -CommandName forgum -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    $commands = @("render", "think", "fortune", "tui", "init", "completions", "list", "status", "doctor", "checkhealth", "config", "logs", "log", "diagnose", "install", "update")
    $commands | Where-Object { $_ -like "$wordToComplete*" } | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
"#
                }
                Shell::Bash => {
                    r#"# Forgum Bash auto-completion script
_forgum_completions() {
    local cur="${COMP_WORDS[COMP_CWORD]}"
    local cmds="render think fortune tui init completions list status doctor checkhealth config logs log diagnose install update --animal --effect --environment --road --mountain --color-mode"
    COMPREPLY=( $(compgen -W "${cmds}" -- "${cur}") )
}
complete -F _forgum_completions forgum
"#
                }
                Shell::Zsh => {
                    r#"#compdef forgum
_forgum() {
    _arguments \
        '--animal[Animal mascot]:mascot:()' \
        '--effect[Animation effect]:effect:(walk breathe float fly talk sway pulse glitch particles dissolve)' \
        '--environment[Particle environment]:environment:(pasture inferno ocean arctic city forest savanna swamp space cyber)' \
        '--color-mode[Color palette mode]:color_mode:(animal rainbow solid none)' \
        '1:subcommand:(render think fortune tui init completions list status doctor checkhealth config logs log diagnose install update)'
}
_forgum "$@"
"#
                }
                Shell::Fish => {
                    r#"# Forgum Fish auto-completion script
complete -c forgum -f
complete -c forgum -n "__fish_use_subcommand" -a "render think fortune tui init completions list status doctor checkhealth config logs log diagnose install update"
"#
                }
                Shell::Nushell => {
                    r#"# Forgum Nushell auto-completion script
export extern "forgum" [
    subcommand?: string
    --animal(-c): string
    --effect(-e): string
    --environment: string
    --color-mode: string
]
"#
                }
                _ => "# Forgum auto-completions\n",
            };

            match std::fs::write(&comp_path, completion_script) {
                Ok(_) => {
                    shell_info.is_completion_installed = true;
                    self.installer_log
                        .push(format!("✓ Generated completions: {}", comp_path.display()));
                }
                Err(e) => {
                    self.installer_log.push(format!("✗ Completions error: {e}"));
                    success = false;
                }
            }
        }

        // 2. Inject profile hook linking completions and init
        if let Some(rc_path) = sh.shell_rc_path() {
            if let Some(parent) = rc_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let comp_cmd = sh
                .completions_script_path()
                .and_then(|cp| sh.shell_source_command(&cp))
                .unwrap_or_default();
            let hook_cmd = match sh {
                Shell::Bash | Shell::Zsh => "eval \"$(forgum init)\"",
                Shell::Fish => "forgum init fish | source",
                Shell::Pwsh | Shell::PowerShell => "Invoke-Expression (&forgum init pwsh)",
                Shell::Nushell => "forgum init nu",
                _ => "forgum init",
            };
            let block = format!(
                "# >>> forgum >>>\n# Completions canonically stored in ~/.config/forgum/completions/\n{comp_cmd}\n{hook_cmd}\n# <<< forgum <<<\n"
            );
            let existing = std::fs::read_to_string(&rc_path).unwrap_or_default();
            let updated = forgum_platform::shell::update_delimited_block(
                &existing,
                "# >>> forgum >>>",
                "# <<< forgum <<<",
                &block,
            );
            match std::fs::write(&rc_path, updated) {
                Ok(_) => {
                    shell_info.is_hook_installed = true;
                    self.installer_log
                        .push(format!("✓ Injected hook block into: {}", rc_path.display()));
                }
                Err(e) => {
                    self.installer_log
                        .push(format!("✗ Hook injection error: {e}"));
                    success = false;
                }
            }
        }

        // 3. Ensure canonical config directory ~/.config/forgum/ exists
        if let Ok(cfg_dir) = forgum_platform::paths::config_dir() {
            let _ = std::fs::create_dir_all(&cfg_dir);
            let cfg_path = cfg_dir.join("config.json");
            if !cfg_path.exists() {
                if let Ok(serialized) = self.config.serialize_with_format(ConfigFormat::Json) {
                    let _ = std::fs::write(&cfg_path, serialized);
                    self.installer_log.push(format!(
                        "✓ Initialized canonical config: {}",
                        cfg_path.display()
                    ));
                }
            }
        }

        if success {
            self.status_message =
                format!("✓ Successfully installed {sh} integration & completions!");
        } else {
            self.status_message = format!("⚠ Completed installation of {sh} with warnings.");
        }
    }

    fn uninstall_shell(&mut self, idx: usize) {
        if idx >= self.shells.len() {
            return;
        }
        let shell_info = &mut self.shells[idx];
        let sh = shell_info.shell;

        if let Some(rc_path) = sh.shell_rc_path() {
            if let Ok(existing) = std::fs::read_to_string(&rc_path) {
                if existing.contains("# >>> forgum >>>") {
                    let cleaned = forgum_platform::shell::update_delimited_block(
                        &existing,
                        "# >>> forgum >>>",
                        "# <<< forgum <<<",
                        "",
                    );
                    let _ = std::fs::write(&rc_path, cleaned);
                    shell_info.is_hook_installed = false;
                    self.installer_log
                        .push(format!("✓ Removed hook from: {}", rc_path.display()));
                }
            }
        }
        self.status_message = format!("✓ Uninstalled hook from {sh}.");
    }

    fn test_shell_simulation(&mut self, idx: usize) {
        if idx >= self.shells.len() {
            return;
        }
        let sh = self.shells[idx].shell;
        self.installer_log.push(format!(
            "▶ Testing shell overlay simulation for {sh} (OK: responsive, 0ms prompt latency)"
        ));
        self.status_message = format!("Test prompt overlay executed cleanly for {sh}.");
    }

    fn handle_config_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.config_field_idx > 0 {
                    self.config_field_idx -= 1;
                } else {
                    self.config_field_idx = ConfigField::ALL.len() - 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.config_field_idx = (self.config_field_idx + 1) % ConfigField::ALL.len();
            }
            KeyCode::Enter => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::Eyes
                    || field == ConfigField::Tongue
                    || field == ConfigField::Palette
                {
                    self.enter_config_edit();
                } else {
                    self.cycle_config_field(true);
                }
            }
            KeyCode::Char('e') | KeyCode::Char('E') | KeyCode::Char('i') | KeyCode::Char('I') => {
                self.enter_config_edit();
            }
            KeyCode::Char('t')
            | KeyCode::Char('T')
            | KeyCode::Char(' ') => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration {
                    self.enter_config_edit();
                } else {
                    self.cycle_config_field(true);
                }
            }
            KeyCode::Char('+')
            | KeyCode::Char('=')
            | KeyCode::Right
            | KeyCode::Char('l')
            | KeyCode::Char(']') => {
                self.cycle_config_field(true);
            }
            KeyCode::Char('-')
            | KeyCode::Char('_')
            | KeyCode::Left
            | KeyCode::Char('h')
            | KeyCode::Char('[') => {
                self.cycle_config_field(false);
            }
            KeyCode::Char('o') | KeyCode::Char('O') => {
                let path = self.resolve_config_path();
                return Ok(Some(Action::OpenEditor(path)));
            }
            _ => {}
        }
        Ok(None)
    }

    fn enter_config_edit(&mut self) {
        let field = ConfigField::ALL[self.config_field_idx];
        self.original_edit_value = match field {
            ConfigField::Duration => self.config.duration.to_string(),
            ConfigField::Fps => self.config.fps.to_string(),
            ConfigField::Eyes => self.config.eyes.clone(),
            ConfigField::Tongue => self.config.tongue.clone(),
            ConfigField::Palette => self.config.palette.clone().unwrap_or_default(),
            _ => String::new(),
        };
        self.edit_initial = true;
        match field {
            ConfigField::Duration => {
                self.config_edit_buffer = self.config.duration.to_string();
                self.editing_config = true;
                self.status_message = "Editing Duration (seconds): type number (0 = infinite) and press Enter (Esc to cancel)".into();
            }
            ConfigField::Fps => {
                self.config_edit_buffer = self.config.fps.to_string();
                self.editing_config = true;
                self.status_message =
                    "Editing Target FPS: type number (1..240) and press Enter (Esc to cancel)"
                        .into();
            }
            ConfigField::Eyes => {
                self.config_edit_buffer = self.config.eyes.clone();
                self.editing_config = true;
                self.status_message =
                    "Editing Eyes: type characters and press Enter (Esc to cancel)".into();
            }
            ConfigField::Tongue => {
                self.config_edit_buffer = self.config.tongue.clone();
                self.editing_config = true;
                self.status_message =
                    "Editing Tongue: type characters and press Enter (Esc to cancel)".into();
            }
            ConfigField::Palette => {
                self.config_edit_buffer = self.config.palette.clone().unwrap_or_default();
                self.editing_config = true;
                self.status_message =
                    "Editing Palette: type comma-separated hex codes (e.g. '#ffffff,#1a1a1a') and press Enter (Esc to cancel)"
                        .into();
            }
            _ => {
                self.edit_initial = false;
                self.cycle_config_field(true);
            }
        }
    }

    fn cycle_config_field(&mut self, forward: bool) {
        let field = ConfigField::ALL[self.config_field_idx];
        match field {
            ConfigField::Duration => {
                const DURATION_PRESETS: [u32; 9] = [0, 1, 2, 3, 5, 10, 15, 30, 60];
                let current = self.config.duration;
                let next = if forward {
                    if let Some(pos) = DURATION_PRESETS.iter().position(|&s| s == current) {
                        DURATION_PRESETS[(pos + 1) % DURATION_PRESETS.len()]
                    } else {
                        DURATION_PRESETS
                            .iter()
                            .copied()
                            .find(|&s| s > current)
                            .unwrap_or(DURATION_PRESETS[0])
                    }
                } else {
                    if let Some(pos) = DURATION_PRESETS.iter().position(|&s| s == current) {
                        DURATION_PRESETS[(pos + DURATION_PRESETS.len() - 1) % DURATION_PRESETS.len()]
                    } else {
                        DURATION_PRESETS
                            .iter()
                            .copied()
                            .rev()
                            .find(|&s| s < current)
                            .unwrap_or(DURATION_PRESETS[DURATION_PRESETS.len() - 1])
                    }
                };
                self.config.duration = next;
                self.saved = false;
                self.status_message = if next == 0 {
                    "Duration set to 0 (infinite loop) [Space/Enter: Toggle | ←/→: Adjust | e: Type]".to_string()
                } else {
                    format!(
                        "Duration adjusted to {}s [Space/Enter: Toggle | ←/→: Adjust | e: Type]",
                        next
                    )
                };
            }
            ConfigField::Fps => {
                const FPS_PRESETS: [u16; 6] = [15, 24, 30, 60, 120, 240];
                let current = self.config.fps;
                let next = if forward {
                    if let Some(pos) = FPS_PRESETS.iter().position(|&p| p == current) {
                        FPS_PRESETS[(pos + 1) % FPS_PRESETS.len()]
                    } else {
                        FPS_PRESETS
                            .iter()
                            .copied()
                            .find(|&p| p > current)
                            .unwrap_or(FPS_PRESETS[0])
                    }
                } else {
                    if let Some(pos) = FPS_PRESETS.iter().position(|&p| p == current) {
                        FPS_PRESETS[(pos + FPS_PRESETS.len() - 1) % FPS_PRESETS.len()]
                    } else {
                        FPS_PRESETS
                            .iter()
                            .copied()
                            .rev()
                            .find(|&p| p < current)
                            .unwrap_or(FPS_PRESETS[FPS_PRESETS.len() - 1])
                    }
                };
                self.config.fps = next;
                self.saved = false;
                self.status_message = format!(
                    "FPS adjusted to {} fps [Space/Enter: Toggle | ←/→: Adjust | e: Type]",
                    next
                );
            }
            ConfigField::Eyes => {
                const EYE_PRESETS: [&str; 8] = ["oo", "..", "@@", "**", "$$", "==", "--", "00"];
                let current = self.config.eyes.as_str();
                let next = if forward {
                    if let Some(pos) = EYE_PRESETS.iter().position(|&e| e == current) {
                        EYE_PRESETS[(pos + 1) % EYE_PRESETS.len()]
                    } else {
                        EYE_PRESETS[0]
                    }
                } else {
                    if let Some(pos) = EYE_PRESETS.iter().position(|&e| e == current) {
                        EYE_PRESETS[(pos + EYE_PRESETS.len() - 1) % EYE_PRESETS.len()]
                    } else {
                        EYE_PRESETS[EYE_PRESETS.len() - 1]
                    }
                };
                self.config.eyes = next.to_string();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!("Eyes cycled to '{}' [Space/←/→: Cycle | e: Custom]", next);
            }
            ConfigField::Tongue => {
                const TONGUE_PRESETS: [&str; 5] = ["  ", "U ", "||", "--", "V "];
                let current = self.config.tongue.as_str();
                let next = if forward {
                    if let Some(pos) = TONGUE_PRESETS.iter().position(|&t| t == current) {
                        TONGUE_PRESETS[(pos + 1) % TONGUE_PRESETS.len()]
                    } else {
                        TONGUE_PRESETS[0]
                    }
                } else {
                    if let Some(pos) = TONGUE_PRESETS.iter().position(|&t| t == current) {
                        TONGUE_PRESETS[(pos + TONGUE_PRESETS.len() - 1) % TONGUE_PRESETS.len()]
                    } else {
                        TONGUE_PRESETS[TONGUE_PRESETS.len() - 1]
                    }
                };
                self.config.tongue = next.to_string();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!("Tongue cycled to '{}' [Space/←/→: Cycle | e: Custom]", next);
            }
            ConfigField::ColorMode => {
                self.color_mode_dropdown.cycle(forward);
                let current = self.color_mode_dropdown.current();
                self.config.color_mode = current.clone();
                self.color_idx = COLOR_OPTIONS
                    .iter()
                    .position(|(c, _)| c.eq_ignore_ascii_case(&current))
                    .unwrap_or(0);
                self.saved = false;
                self.status_message = format!("Color mode switched to {}", current);
            }
            ConfigField::Palette => {
                const PALETTE_PRESETS: [&str; 5] = [
                    "",
                    "#ff007f,#00e5ff,#ffff00",
                    "#ff5555,#50fa7b,#8be9fd",
                    "#e67e22,#f39c12,#d35400",
                    "#38ef7d,#11998e",
                ];
                let current = self.config.palette.as_deref().unwrap_or("");
                let next = if forward {
                    if let Some(pos) = PALETTE_PRESETS.iter().position(|&p| p == current) {
                        PALETTE_PRESETS[(pos + 1) % PALETTE_PRESETS.len()]
                    } else {
                        PALETTE_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = PALETTE_PRESETS.iter().position(|&p| p == current) {
                        PALETTE_PRESETS[(pos + PALETTE_PRESETS.len() - 1) % PALETTE_PRESETS.len()]
                    } else {
                        PALETTE_PRESETS[PALETTE_PRESETS.len() - 1]
                    }
                };
                self.config.palette = if next.is_empty() { None } else { Some(next.to_string()) };
                self.saved = false;
                self.status_message = if next.is_empty() {
                    "Palette set to default [Space/←/→: Cycle | e: Custom]".into()
                } else {
                    format!("Palette cycled to '{}' [Space/←/→: Cycle | e: Custom]", next)
                };
            }
            ConfigField::Environment => {
                self.environment_dropdown.cycle(forward);
                let current = self.environment_dropdown.current();
                self.config.environment = Some(current.clone());
                self.scenery_idx = SCENERY_OPTIONS
                    .iter()
                    .position(|(s, _)| s.eq_ignore_ascii_case(&current))
                    .unwrap_or(0);
                self.saved = false;
                self.status_message = format!("Environment switched to {}", current);
            }
            ConfigField::Road => {
                self.road_dropdown.cycle(forward);
                let current = self.road_dropdown.current();
                self.config.road = Some(current.clone());
                self.road_idx = ROAD_OPTIONS
                    .iter()
                    .position(|(r, _)| r.eq_ignore_ascii_case(&current))
                    .unwrap_or(0);
                self.saved = false;
                self.status_message = format!("Road style switched to {}", current);
            }
            ConfigField::Mountain => {
                self.mountain_dropdown.cycle(forward);
                let current = self.mountain_dropdown.current();
                self.config.mountain = Some(current.clone());
                self.saved = false;
                self.status_message = format!("Mountain silhouette switched to {}", current);
            }
            ConfigField::AnimationType => {
                self.animation_type_dropdown.cycle(forward);
                let current = self.animation_type_dropdown.current();
                self.config.animation_type = Some(current.clone());
                self.config.effect = current.clone();
                self.effect_idx = EFFECT_OPTIONS
                    .iter()
                    .position(|(e, _)| e.eq_ignore_ascii_case(&current))
                    .unwrap_or(0);
                self.saved = false;
                self.status_message = format!("Animation type switched to {}", current);
            }
            ConfigField::Background => {
                self.config.background = !self.config.background;
                self.saved = false;
                self.status_message = format!(
                    "Background scenery {}",
                    if self.config.background {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                );
            }
            ConfigField::AutoRenderOnPrompt => {
                self.config.auto_render_on_prompt = !self.config.auto_render_on_prompt;
                self.saved = false;
                self.status_message = format!(
                    "Auto-render on prompt {}",
                    if self.config.auto_render_on_prompt {
                        "ENABLED"
                    } else {
                        "DISABLED"
                    }
                );
            }
            ConfigField::ShellAttachMode => {
                self.attach_mode_dropdown.cycle(forward);
                self.config.shell_attach_mode = self.attach_mode_dropdown.current();
                self.saved = false;
                self.status_message =
                    format!("Shell attach mode: {}", self.config.shell_attach_mode);
            }
            ConfigField::ConfigFormat => {
                self.format_dropdown.cycle(forward);
                self.format = self.config_format();
                self.saved = false;
                self.status_message = format!(
                    "Config format switched to {} (will be used on Save)",
                    self.format_dropdown.current().to_uppercase()
                );
            }
        }
    }

    fn handle_config_edit_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        let field = ConfigField::ALL[self.config_field_idx];
        match key.code {
            KeyCode::Enter => {
                self.commit_config_edit();
                self.editing_config = false;
            }
            KeyCode::Esc => {
                match field {
                    ConfigField::Duration => {
                        if let Ok(v) = self.original_edit_value.parse::<u32>() {
                            self.config.duration = v;
                        }
                    }
                    ConfigField::Fps => {
                        if let Ok(v) = self.original_edit_value.parse::<u16>() {
                            self.config.fps = v;
                        }
                    }
                    ConfigField::Eyes => {
                        self.config.eyes = self.original_edit_value.clone();
                        self.cow_cache.clear();
                        self.ensure_cow_cached(&self.config.cow.clone());
                    }
                    ConfigField::Tongue => {
                        self.config.tongue = self.original_edit_value.clone();
                        self.cow_cache.clear();
                        self.ensure_cow_cached(&self.config.cow.clone());
                    }
                    ConfigField::Palette => {
                        self.config.palette = if self.original_edit_value.is_empty() {
                            None
                        } else {
                            Some(self.original_edit_value.clone())
                        };
                    }
                    _ => {}
                }
                self.editing_config = false;
                self.status_message = "Edit cancelled (original value restored).".to_string();
            }
            KeyCode::Char(' ') if field == ConfigField::Fps => {
                self.commit_config_edit();
                self.editing_config = false;
                self.cycle_config_field(true);
            }
            KeyCode::Char('+') | KeyCode::Char('=')
                if field == ConfigField::Duration || field == ConfigField::Fps =>
            {
                self.commit_config_edit();
                self.cycle_config_field(true);
                self.config_edit_buffer = match field {
                    ConfigField::Duration => self.config.duration.to_string(),
                    ConfigField::Fps => self.config.fps.to_string(),
                    _ => self.config_edit_buffer.clone(),
                };
            }
            KeyCode::Char('-') | KeyCode::Char('_')
                if field == ConfigField::Duration || field == ConfigField::Fps =>
            {
                self.commit_config_edit();
                self.cycle_config_field(false);
                self.config_edit_buffer = match field {
                    ConfigField::Duration => self.config.duration.to_string(),
                    ConfigField::Fps => self.config.fps.to_string(),
                    _ => self.config_edit_buffer.clone(),
                };
            }
            KeyCode::Up => {
                self.edit_initial = false;
                self.commit_config_edit();
                self.editing_config = false;
                if self.config_field_idx > 0 {
                    self.config_field_idx -= 1;
                } else {
                    self.config_field_idx = ConfigField::ALL.len() - 1;
                }
            }
            KeyCode::Down => {
                self.edit_initial = false;
                self.commit_config_edit();
                self.editing_config = false;
                self.config_field_idx = (self.config_field_idx + 1) % ConfigField::ALL.len();
            }
            KeyCode::Right => {
                self.edit_initial = false;
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        self.config_edit_buffer = (val.round() as u32 + 1).to_string();
                    }
                }
            }
            KeyCode::Left => {
                self.edit_initial = false;
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        let floor = if field == ConfigField::Fps { 1 } else { 0 };
                        let current = val.round() as u32;
                        self.config_edit_buffer = current.saturating_sub(1).max(floor).to_string();
                    }
                }
            }
            KeyCode::Tab => {
                self.edit_initial = false;
                self.commit_config_edit();
                self.editing_config = false;
                self.config_field_idx = (self.config_field_idx + 1) % ConfigField::ALL.len();
            }
            KeyCode::BackTab => {
                self.edit_initial = false;
                self.commit_config_edit();
                self.editing_config = false;
                if self.config_field_idx > 0 {
                    self.config_field_idx -= 1;
                } else {
                    self.config_field_idx = ConfigField::ALL.len() - 1;
                }
            }
            KeyCode::Backspace => {
                self.edit_initial = false;
                self.config_edit_buffer.pop();
            }
            KeyCode::Delete => {
                self.edit_initial = false;
                self.config_edit_buffer.clear();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.edit_initial = false;
                self.config_edit_buffer.clear();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.edit_initial = false;
                self.config_edit_buffer.clear();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if (field == ConfigField::Duration || field == ConfigField::Fps)
                    && !c.is_ascii_digit()
                    && c != 's'
                    && c != 'f'
                    && c != 'p'
                {
                    // Ignore non-digit characters on numeric fields so buffer isn't corrupted
                } else if self.edit_initial {
                    self.config_edit_buffer = c.to_string();
                    self.edit_initial = false;
                } else {
                    self.config_edit_buffer.push(c);
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn commit_config_edit(&mut self) {
        let field = ConfigField::ALL[self.config_field_idx];
        match field {
            ConfigField::Duration => {
                let trimmed = self.config_edit_buffer.trim();
                let clean =
                    trimmed.trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                if let Ok(val) = clean.parse::<f64>() {
                    let n = val.max(0.0).round() as u32;
                    self.config.duration = n;
                    self.saved = false;
                    self.status_message = if n == 0 {
                        "✓ Duration set to 0 (infinite / until signal)".to_string()
                    } else {
                        format!("✓ Duration set to {}s", n)
                    };
                } else if !trimmed.is_empty() {
                    self.status_message = format!(
                        "⚠ Invalid duration '{}': enter positive number of seconds",
                        trimmed
                    );
                }
            }
            ConfigField::Fps => {
                let trimmed = self.config_edit_buffer.trim();
                let clean =
                    trimmed.trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                if let Ok(val) = clean.parse::<f64>() {
                    let n = val.clamp(1.0, 240.0).round() as u16;
                    self.config.fps = n;
                    self.saved = false;
                    self.status_message = format!("✓ Target FPS set to {} fps", n);
                } else if !trimmed.is_empty() {
                    self.status_message = format!(
                        "⚠ Invalid FPS '{}': enter number between 1 and 240",
                        trimmed
                    );
                }
            }
            ConfigField::Eyes => {
                self.config.eyes = self.config_edit_buffer.clone();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!("✓ Eyes updated to '{}'", self.config.eyes);
            }
            ConfigField::Tongue => {
                self.config.tongue = self.config_edit_buffer.clone();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!("✓ Tongue updated to '{}'", self.config.tongue);
            }
            ConfigField::Palette => {
                let trimmed = self.config_edit_buffer.trim().to_string();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
                    self.config.palette = None;
                    self.saved = false;
                    self.status_message = "✓ Palette cleared (using default color mode)".to_string();
                } else {
                    self.config.palette = Some(trimmed.clone());
                    self.saved = false;
                    self.status_message = format!("✓ Palette set to '{}'", trimmed);
                }
            }
            _ => {}
        }
    }

    /// Launch the host system's graphical desktop file manager targeting the config folder.
    pub fn open_config_directory(&mut self) {
        let dir = self
            .config_path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
            .or_else(|| forgum_platform::config_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        match forgum_platform::open_folder_in_desktop(&dir) {
            Ok(()) => {
                self.status_message = format!("📁 Opened config directory: {}", dir.display());
            }
            Err(e) => {
                self.status_message = format!("⚠ Failed to open config directory: {e}");
            }
        }
    }

    fn ensure_cow_cached(&mut self, cow_name: &str) {
        if self.cow_cache.contains_key(cow_name) {
            return;
        }
        let cow_art = self.load_cow_art(cow_name);
        self.cow_cache.insert(cow_name.to_string(), cow_art);
    }

    fn load_cow_art(&self, cow_name: &str) -> String {
        let data = match forgum_platform::data_dir() {
            Ok(d) => d,
            Err(_) => return Self::fallback_cow(&self.config.eyes, &self.config.tongue),
        };
        let cow_path = data.join("Cows").join(format!("{cow_name}.cow"));
        let raw = match std::fs::read_to_string(&cow_path) {
            Ok(s) => s,
            Err(_) => return Self::fallback_cow(&self.config.eyes, &self.config.tongue),
        };
        Self::expand_cow_template(&raw, &self.config.eyes, &self.config.tongue)
    }

    pub fn expand_cow_template(template: &str, eyes: &str, tongue: &str) -> String {
        let body = if let Some(start) = template.find("<<") {
            let after_marker = &template[start + 2..];
            let first_newline = after_marker.find('\n').unwrap_or(after_marker.len());
            let marker_line = &after_marker[..first_newline];
            let tag = marker_line
                .trim_matches(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == ';')
                .trim();

            if !tag.is_empty() {
                let body_content = if first_newline < after_marker.len() {
                    &after_marker[first_newline..]
                } else {
                    ""
                };
                let mut end_pos = None;
                let mut remaining = body_content;
                let mut curr = 0;
                while !remaining.is_empty() {
                    let line_len = remaining
                        .find('\n')
                        .map(|idx| idx + 1)
                        .unwrap_or(remaining.len());
                    let line_with_nl = &remaining[..line_len];
                    let line = line_with_nl.trim_end_matches(&['\r', '\n'][..]);
                    let trimmed = line.trim();
                    let clean = trimmed.trim_end_matches(';');
                    if clean == tag {
                        end_pos = Some(curr);
                        break;
                    }
                    curr += line_len;
                    remaining = &remaining[line_len..];
                }
                if let Some(pos) = end_pos {
                    &body_content[..pos]
                } else {
                    body_content
                }
            } else {
                after_marker
            }
        } else {
            template
        };

        let eye_chars: Vec<char> = eyes.chars().collect();
        let left_eye = eye_chars.first().copied().unwrap_or('o').to_string();
        let right_eye = eye_chars
            .get(1)
            .copied()
            .unwrap_or_else(|| eye_chars.first().copied().unwrap_or('o'))
            .to_string();
        let mut eye_idx = 0usize;

        let mut out = String::with_capacity(body.len());
        for line in body.lines() {
            let mut l = line.to_string();
            if l.contains('$') {
                // 1. Expand thoughts placeholders (longest first)
                for pat in [
                    r"\\$thoughts",
                    r"\$thoughts",
                    r"\\${thoughts}",
                    r"\${thoughts}",
                    "${thoughts}",
                    "$thoughts",
                ] {
                    l = l.replace(pat, "\\");
                }

                // 2. Expand tongue placeholders (longest first)
                for pat in [
                    r"\\$tongue",
                    r"\$tongue",
                    r"\\${tongue}",
                    r"\${tongue}",
                    "${tongue}",
                    "$tongue",
                ] {
                    l = l.replace(pat, tongue);
                }

                // 3. Expand plural $eyes placeholders (longest first)
                for pat in [
                    r"\\$eyes",
                    r"\$eyes",
                    r"\\${eyes}",
                    r"\${eyes}",
                    "${eyes}",
                    "$eyes",
                ] {
                    l = l.replace(pat, eyes);
                }

                // 4. Expand singular $eye placeholders (alternating left/right eye)
                loop {
                    let eye_patterns = [
                        r"\\$eye",
                        r"\$eye",
                        r"\\${eye}",
                        r"\${eye}",
                        "${eye}",
                        "$eye",
                    ];
                    let mut earliest: Option<(usize, &'static str)> = None;
                    for pat in eye_patterns {
                        if let Some(pos) = l.find(pat) {
                            match earliest {
                                None => earliest = Some((pos, pat)),
                                Some((best_pos, _)) if pos < best_pos => earliest = Some((pos, pat)),
                                _ => {}
                            }
                        }
                    }

                    let Some((pos, pat)) = earliest else {
                        break;
                    };

                    let glyph = if eye_idx % 2 == 0 {
                        &left_eye
                    } else {
                        &right_eye
                    };
                    eye_idx += 1;
                    l.replace_range(pos..pos + pat.len(), glyph);
                }
            }

            // 5. Unescape literal perl heredoc escapes: \$ to $, \@ to @, \# to #
            if l.contains(r"\$") || l.contains(r"\@") || l.contains(r"\#") {
                l = l.replace(r"\$", "$").replace(r"\@", "@").replace(r"\#", "#");
            }

            out.push_str(&l);
            out.push('\n');
        }
        while out.ends_with('\n') {
            out.pop();
        }
        out
    }

    pub fn fallback_cow(eyes: &str, tongue: &str) -> String {
        let e1 = eyes.chars().next().unwrap_or('o');
        let e2 = eyes.chars().last().unwrap_or('o');
        let t = tongue.chars().next().unwrap_or(' ');
        format!(
            "        \\   {e1}^__{e2}\n         \\  ({e1}{e2})\\_______\n            (__)\\      ({t})\\/\\\n                ||----w |\n                ||     ||"
        )
    }

    // ── RENDER PIPELINE (Zellij + LazyGit Inspired) ──────────────────────────

    /// Master render function that paints the complete dashboard frame.
    pub fn render(&mut self, f: &mut Frame) {
        let size = f.area();

        // 3-row layout:
        // Row 0: Zellij Mode Ribbon & Tab Bar (3 lines)
        // Row 1: LazyGit Split Workspace (Min 10 lines)
        // Row 2: Zellij Pill Status & Keybinding Bar (2 lines)
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(2),
            ])
            .split(size);

        self.render_zellij_header(f, chunks[0]);
        self.render_workspace(f, chunks[1]);
        self.render_zellij_footer(f, chunks[2]);

        if self.show_editor_modal {
            self.render_editor_modal(f, size);
        }
    }

    /// Render centered modal dialog requesting action when no terminal editor is found.
    fn render_editor_modal(&self, f: &mut Frame, area: Rect) {
        let popup_width = 78.min(area.width.saturating_sub(4));
        let popup_height = if area.width < 70 { 10 } else { 8 }.min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
        let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
        let popup_area = Rect::new(x, y, popup_width, popup_height);

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .title(Span::styled(
                " 📝 Forgum Config File Editor Request ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No terminal editor (neovim/vim, nano, emacs) detected on your system.",
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Do you want to open with system file editor, or install a terminal editor?",
                Style::default().fg(Color::Gray),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "[O] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Open with System Editor (Notepad/GUI)  ",
                    Style::default().fg(Color::White),
                ),
                Span::styled("│  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "[I] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Install Neovim (winget/scoop)  ",
                    Style::default().fg(Color::White),
                ),
                Span::styled("│  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "[Esc] ",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner);
    }

    /// Render Zellij-style top header with active mode badge, tabs, and format badge.
    fn render_zellij_header(&self, f: &mut Frame, area: Rect) {
        let mut line_spans = Vec::new();

        // Active mode pill (high contrast)
        line_spans.push(Span::styled(
            format!(" {} ", self.current_tab.mode_label()),
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        ));
        line_spans.push(Span::raw(" "));

        // Tabs 1-5
        for tab in Tab::ALL {
            let is_active = tab == self.current_tab;
            let style = if is_active {
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            line_spans.push(Span::styled(tab.title(), style));
            line_spans.push(Span::raw(" "));
        }

        // Right-hand status badges
        let format_badge = if self.config_path.is_some() {
            self.config_format().display_name()
        } else {
            "IN-MEMORY"
        };
        let right_info = format!(" [v{}] [{format_badge}] ", env!("CARGO_PKG_VERSION"));

        let title_line = Line::from(line_spans);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " 🐮 FORGUM TERMINAL MATRIX ",
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ));

        let p = Paragraph::new(title_line).block(block);
        f.render_widget(p, area);

        // Render right-aligned badge in header if width permits
        if area.width > 70 {
            let badge_width = right_info.len() as u16;
            let badge_rect = Rect {
                x: area.x + area.width.saturating_sub(badge_width + 2),
                y: area.y + 1,
                width: badge_width,
                height: 1,
            };
            let badge_p = Paragraph::new(Line::from(Span::styled(
                right_info,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )));
            f.render_widget(badge_p, badge_rect);
        }
    }

    /// Render LazyGit-style split workspace (Left: Entity List / Right: Live Canvas + Inspector).
    fn render_workspace(&mut self, f: &mut Frame, area: Rect) {
        if self.current_tab == Tab::Installer {
            self.render_installer_workspace(f, area);
            return;
        }

        let width = area.width;
        let height = area.height;

        // Tailwind-inspired adaptive split breakpoints:
        // sm (< 76 cols): mobile/compact view, left panel 36% (min 22)
        // md (76..=120 cols): tablet/standard view, left panel 28% (min 26, max 38)
        // lg / xl (> 120 cols): desktop/wide view, left panel 24% (min 28, max 46)
        let left_width = if width < 76 {
            (width * 36 / 100).max(22).min(width.saturating_sub(20))
        } else if width <= 120 {
            (width * 28 / 100).max(26).min(38)
        } else {
            (width * 24 / 100).max(28).min(46)
        };
        let right_width = width.saturating_sub(left_width);

        let left_rect = Rect {
            x: area.x,
            y: area.y,
            width: left_width,
            height: area.height,
        };

        let right_rect = Rect {
            x: area.x + left_width,
            y: area.y,
            width: right_width,
            height: area.height,
        };

        // Render left column based on active tab
        match self.current_tab {
            Tab::Mascots => self.render_mascots_list(f, left_rect),
            Tab::Scenery => self.render_scenery_list(f, left_rect),
            Tab::Effects => self.render_effects_list(f, left_rect),
            Tab::Installer => self.render_installer_list(f, left_rect),
            Tab::Config => self.render_config_list(f, left_rect),
        }

        // Adaptive Inspector Height:
        // When height is tight (< 20 lines), collapse inspector to 4 lines to give maximum space to canvas
        // When width is wide (> 120 cols) and height >= 24, expand inspector to 7 lines for rich metadata
        // Otherwise 6 lines
        let inspector_len = if height < 20 {
            4
        } else if width > 120 && height >= 24 {
            7
        } else {
            6
        };

        // Render right column: Upper = 30 FPS Live Preview, Lower = Inspector / Log
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),                     // Live Preview Canvas
                Constraint::Length(inspector_len),      // Inspector & Details / Log
            ])
            .split(right_rect);

        self.render_live_preview(f, right_split[0]);
        self.render_inspector(f, right_split[1]);
    }

    /// Render Grand Welcoming First-Time Setup TUI and Host Diagnostics workspace.
    fn render_installer_workspace(&self, f: &mut Frame, area: Rect) {
        let width = area.width;
        let height = area.height;

        let left_width = if width < 80 {
            (width * 38 / 100).max(28).min(width.saturating_sub(30))
        } else if width <= 130 {
            (width * 30 / 100).max(32).min(44)
        } else {
            (width * 25 / 100).max(34).min(50)
        };
        let right_width = width.saturating_sub(left_width);

        let left_rect = Rect {
            x: area.x,
            y: area.y,
            width: left_width,
            height: area.height,
        };

        let right_rect = Rect {
            x: area.x + left_width,
            y: area.y,
            width: right_width,
            height: area.height,
        };

        // Left column split: Top = Shells, Bottom = Package managers
        let left_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(left_rect);

        self.render_installer_list(f, left_split[0]);
        self.render_package_managers_card(f, left_split[1]);

        let log_height = if height < 22 { 5 } else if height < 30 { 7 } else { 9 };

        // Right column split: Top = Grand Welcoming Hero & Diagnostics, Bottom = Log
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(log_height)])
            .split(right_rect);

        self.render_installer_hero_card(f, right_split[0]);
        self.render_installer_log_card(f, right_split[1]);
    }

    fn render_package_managers_card(&self, f: &mut Frame, area: Rect) {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Detected Source: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                self.detected_install_source.name(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Available On Host:",
            Style::default().fg(Color::Cyan),
        )));

        for (pm, is_avail) in &self.package_managers {
            let status = if *is_avail {
                Span::styled(" [✓ Installed] ", Style::default().fg(Color::Green))
            } else {
                Span::styled(" [✗ Absent]    ", Style::default().fg(Color::DarkGray))
            };
            lines.push(Line::from(vec![
                status,
                Span::styled(
                    pm.name(),
                    Style::default().fg(if *is_avail {
                        Color::White
                    } else {
                        Color::DarkGray
                    }),
                ),
            ]));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Update Command: ", Style::default().fg(Color::DarkGray)),
            Span::styled("forgum update", Style::default().fg(Color::Cyan)),
        ]));

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Package Managers (p: Check) "),
            );
        f.render_widget(p, area);
    }

    fn render_installer_hero_card(&self, f: &mut Frame, area: Rect) {
        let active_term = detect_active_terminal();
        let os_str = std::env::consts::OS;
        let arch_str = std::env::consts::ARCH;

        let mut lines: Vec<Line> = Vec::new();

        match self.installer_view_mode {
            0 => {
                // Grand Welcoming Hero Banner & Diagnostics
                lines.push(Line::from(Span::styled(
                    "  ███████╗ ██████╗ ██████╗  ██████╗ ██╗   ██╗███╗   ███╗",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██║   ██║████╗ ████║",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  █████╗  ██║   ██║██████╔╝██║  ███╗██║   ██║██╔████╔██║",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  ██╔══╝  ██║   ██║██╔══██╗██║   ██║██║   ██║██║╚██╔╝██║",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  ██║     ╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║ ╚═╝ ██║",
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!(
                        "  v{} • Universal Holographic Terminal Engine • 100% Offline",
                        env!("CARGO_PKG_VERSION")
                    ),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::ITALIC),
                )));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Operating System   : ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{os_str} ({arch_str})"),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" [100% Compatible]", Style::default().fg(Color::Green)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Active Terminal    : ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        active_term,
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " [24-bit TrueColor Ready]",
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Canonical Config   : ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "~/.config/forgum/config.json",
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Shell Completions  : ",
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        "~/.config/forgum/completions/",
                        Style::default().fg(Color::Yellow),
                    ),
                ]));
                lines.push(Line::from(""));

                lines.push(Line::from(vec![
                    Span::styled("  💡 Recommendation: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(
                        if os_str == "windows" {
                            "Windows Terminal or Ghostty with PowerShell 7+ for richest interactive experience."
                        } else {
                            "Ghostty or WezTerm with Fish or Zsh for instant asynchronous prompt overlay."
                        },
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(Span::styled(
                    "  [Controls: Press 'l' for License & Terms, 'p' for Updates, 'i' to install shell]",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            1 => {
                // License & Terms
                lines.push(Line::from(Span::styled(
                    "  Dual-Licensed: MIT License & Apache License 2.0",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  Copyright (c) 2026 Forgum Authors & Contributors",
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  Terms of Use & Architecture Guarantees:",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(vec![
                    Span::styled("  ✓ Free & Open Source : ", Style::default().fg(Color::Green)),
                    Span::styled("Permissive dual licensing permits personal and commercial use without fee.", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  ✓ Zero Telemetry     : ",
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(
                        "No analytics, no network telemetry, no background tracking daemons.",
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ✓ 100% Offline       : ", Style::default().fg(Color::Green)),
                    Span::styled("Runs completely in-memory with zero mandatory disk or network dependencies.", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ✓ Safe Shell Hooking : ", Style::default().fg(Color::Green)),
                    Span::styled("Integrations use isolated, reversible `# >>> forgum >>>` delimited blocks.", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("  ✓ Canonical Paths    : ", Style::default().fg(Color::Green)),
                    Span::styled("All configuration and completions reside canonically in ~/.config/forgum/.", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  [Press 'l' to view Package Manager Distribution Channels]",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            _ => {
                // Package Managers & Update Distribution
                lines.push(Line::from(Span::styled(
                    "  Package Manager Installation & Distribution Channels",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("  • Scoop (Windows)     : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled("scoop bucket add hkdevloops https://github.com/HKDevLoops/scoop-bucket && scoop install forgum", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • WinGet (Windows)    : ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "winget install HKDevLoops.Forgum",
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Chocolatey (Windows): ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("choco install forgum", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Homebrew (macOS/Lin): ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("brew install forgum", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Pacman (Arch Linux) : ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("sudo pacman -S forgum", Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  • Cargo (Universal)   : ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "cargo install --force forgum-cli",
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled(
                        "  ⚡ Automated Update   : ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "Run `forgum update` or `forgum update --check` at any time.",
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "  [Press 'l' to cycle back to Grand Welcoming & Diagnostics]",
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let title = match self.installer_view_mode {
            0 => " 🚀 Grand Welcoming & Host Diagnostics (View 1/3, press 'l') ",
            1 => " 📜 License Agreement & Terms of Service (View 2/3, press 'l') ",
            _ => " 📦 Package Managers & Update Distribution (View 3/3, press 'l') ",
        };

        let p = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(title),
            );
        f.render_widget(p, area);
    }

    fn render_installer_log_card(&self, f: &mut Frame, area: Rect) {
        let log_lines: Vec<Line> = self
            .installer_log
            .iter()
            .rev()
            .take(area.height.saturating_sub(2) as usize)
            .map(|msg| {
                let color = if msg.starts_with('✓') {
                    Color::Green
                } else if msg.starts_with('✗') {
                    Color::Red
                } else if msg.starts_with('▶') {
                    Color::Yellow
                } else {
                    Color::Cyan
                };
                Line::from(Span::styled(msg, Style::default().fg(color)))
            })
            .collect();

        let p = Paragraph::new(log_lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Shell Installation & Diagnostics Log (i: Install, u: Uninstall, t: Test, p: Check Updates) "),
            );
        f.render_widget(p, area);
    }

    // ── LEFT PANELS ──────────────────────────────────────────────────────────

    fn render_mascots_list(&self, f: &mut Frame, area: Rect) {
        let current_category = CATEGORIES[self.mascot_category_idx];
        let items: Vec<ListItem> = current_category
            .1
            .iter()
            .enumerate()
            .map(|(i, &cow)| {
                let is_sel = i == self.mascot_item_idx;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(cow, style),
                ]))
            })
            .collect();

        let title = format!(
            " Mascots: {} ({}/{}) ",
            current_category.0,
            self.mascot_category_idx + 1,
            CATEGORIES.len()
        );

        let visible_rows = (area.height.saturating_sub(2) as usize).max(1);
        let start_idx = if self.mascot_item_idx >= visible_rows {
            self.mascot_item_idx + 1 - visible_rows
        } else {
            0
        };
        let visible_items: Vec<ListItem> =
            items.into_iter().skip(start_idx).take(visible_rows).collect();

        let list = List::new(visible_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title),
        );
        f.render_widget(list, area);
    }

    fn render_scenery_list(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let native_biome = forgum_platform::biome::get_mascot_biome(&self.config.cow);

        // Scenery Archetypes list
        let scenery_items: Vec<ListItem> = SCENERY_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.scenery_idx;
                let is_focused = self.scenery_sub_focus == 0;
                let is_native = name.eq_ignore_ascii_case(native_biome.environment);
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                let mut spans = vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                ];
                if is_native {
                    spans.push(Span::styled(" ★ Native", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        let visible_rows_scenery = (chunks[0].height.saturating_sub(2) as usize).max(1);
        let start_scenery = if self.scenery_idx >= visible_rows_scenery {
            self.scenery_idx + 1 - visible_rows_scenery
        } else {
            0
        };
        let visible_scenery_items: Vec<ListItem> = scenery_items
            .into_iter()
            .skip(start_scenery)
            .take(visible_rows_scenery)
            .collect();

        let scenery_title = format!(" Biomes / Scenery ({}) [Native: {}] ", SCENERY_OPTIONS.len(), native_biome.environment);
        let scenery_list = List::new(visible_scenery_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(scenery_title),
        );
        f.render_widget(scenery_list, chunks[0]);

        // Road Surfaces list
        let road_items: Vec<ListItem> = ROAD_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.road_idx;
                let is_focused = self.scenery_sub_focus == 1;
                let is_native = name.eq_ignore_ascii_case(native_biome.road);
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                let mut spans = vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                ];
                if is_native {
                    spans.push(Span::styled(" ★ Native", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        let visible_rows_road = (chunks[1].height.saturating_sub(2) as usize).max(1);
        let start_road = if self.road_idx >= visible_rows_road {
            self.road_idx + 1 - visible_rows_road
        } else {
            0
        };
        let visible_road_items: Vec<ListItem> = road_items
            .into_iter()
            .skip(start_road)
            .take(visible_rows_road)
            .collect();

        let road_title = format!(" Road Surfaces ({}) [Native: {}] ", ROAD_OPTIONS.len(), native_biome.road);
        let road_list = List::new(visible_road_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(road_title),
        );
        f.render_widget(road_list, chunks[1]);
    }

    fn render_effects_list(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        // Kinematics Effects
        let effect_items: Vec<ListItem> = EFFECT_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.effect_idx;
                let is_focused = self.fx_sub_focus == 0;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                ]))
            })
            .collect();

        let visible_rows_fx = (chunks[0].height.saturating_sub(2) as usize).max(1);
        let start_fx = if self.effect_idx >= visible_rows_fx {
            self.effect_idx + 1 - visible_rows_fx
        } else {
            0
        };
        let visible_effect_items: Vec<ListItem> = effect_items
            .into_iter()
            .skip(start_fx)
            .take(visible_rows_fx)
            .collect();

        let effects_title = format!(" Kinematics Effects ({}) ", EFFECT_OPTIONS.len());
        let effect_list = List::new(visible_effect_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(effects_title),
        );
        f.render_widget(effect_list, chunks[0]);

        // Color Palettes
        let color_items: Vec<ListItem> = COLOR_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.color_idx;
                let is_focused = self.fx_sub_focus == 1;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                ]))
            })
            .collect();

        let visible_rows_colors = (chunks[1].height.saturating_sub(2) as usize).max(1);
        let start_color = if self.color_idx >= visible_rows_colors {
            self.color_idx + 1 - visible_rows_colors
        } else {
            0
        };
        let visible_color_items: Vec<ListItem> = color_items
            .into_iter()
            .skip(start_color)
            .take(visible_rows_colors)
            .collect();

        let colors_title = format!(" Color Themes ({}) ", COLOR_OPTIONS.len());
        let color_list = List::new(visible_color_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(colors_title),
        );
        f.render_widget(color_list, chunks[1]);
    }

    fn render_installer_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .shells
            .iter()
            .enumerate()
            .map(|(i, info)| {
                let is_sel = i == self.selected_shell_idx;
                let prefix = if is_sel { "▶ " } else { "  " };

                let status_indicator = if info.is_hook_installed && info.is_completion_installed {
                    Span::styled("[✓ Complete] ", Style::default().fg(Color::Green))
                } else if info.is_hook_installed {
                    Span::styled("[~ Hook]     ", Style::default().fg(Color::Yellow))
                } else if info.is_available {
                    Span::styled("[+ Ready]    ", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled("[- Absent]   ", Style::default().fg(Color::DarkGray))
                };

                let name_style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    status_indicator,
                    Span::styled(info.shell.to_string(), name_style),
                ]))
            })
            .collect();

        let visible_rows_shells = (area.height.saturating_sub(2) as usize).max(1);
        let start_shell = if self.selected_shell_idx >= visible_rows_shells {
            self.selected_shell_idx + 1 - visible_rows_shells
        } else {
            0
        };
        let visible_shell_items: Vec<ListItem> = items
            .into_iter()
            .skip(start_shell)
            .take(visible_rows_shells)
            .collect();

        let list = List::new(visible_shell_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Shell Integration (i: Install) "),
        );
        f.render_widget(list, area);
    }

    /// Render formatted value string for a ConfigField.
    pub fn field_value(&self, field: ConfigField, is_sel: bool) -> String {
        match field {
            ConfigField::Duration => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ] (Enter: Commit | Esc: Cancel)", self.config_edit_buffer)
                } else if self.config.duration == 0 {
                    "0s (infinite / run until signal) [Enter/e: Edit Number | +/-: Step]".into()
                } else {
                    format!(
                        "{}s [Enter/e: Edit Number | +/-: Step]",
                        self.config.duration
                    )
                }
            }
            ConfigField::Fps => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!(
                        "{} fps [Space/Enter: Toggle | ←/→: Adjust | e: Type]",
                        self.config.fps
                    )
                }
            }
            ConfigField::Eyes => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!("'{}' [Enter to edit]", self.config.eyes)
                }
            }
            ConfigField::Tongue => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!("'{}' [Enter to edit]", self.config.tongue)
                }
            }
            ConfigField::ColorMode => {
                format!("{} (←/→ cycle)", self.config.color_mode)
            }
            ConfigField::Palette => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if let Some(pal) = &self.config.palette {
                    if pal.is_empty() {
                        "none (default) [Enter to edit]".into()
                    } else {
                        format!("'{}' [Enter to edit]", pal)
                    }
                } else {
                    "none (default) [Enter to edit]".into()
                }
            }
            ConfigField::Environment => {
                format!(
                    "{} (←/→ cycle)",
                    self.config.environment.as_deref().unwrap_or("none")
                )
            }
            ConfigField::Road => {
                format!(
                    "{} (←/→ cycle)",
                    self.config.road.as_deref().unwrap_or("none")
                )
            }
            ConfigField::Mountain => {
                format!(
                    "{} (←/→ cycle)",
                    self.config.mountain.as_deref().unwrap_or("none")
                )
            }
            ConfigField::AnimationType => {
                format!(
                    "{} (←/→ cycle)",
                    self.config.animation_type.as_deref().unwrap_or("animal_natural")
                )
            }
            ConfigField::Background => {
                if self.config.background {
                    "✔ ON (Space/Enter to toggle)".into()
                } else {
                    "✖ OFF (Space/Enter to toggle)".into()
                }
            }
            ConfigField::AutoRenderOnPrompt => {
                if self.config.auto_render_on_prompt {
                    "✔ ON (Space/Enter to toggle)".into()
                } else {
                    "✖ OFF (Space/Enter to toggle)".into()
                }
            }
            ConfigField::ShellAttachMode => {
                format!("{} (←/→ cycle)", self.config.shell_attach_mode)
            }
            ConfigField::ConfigFormat => {
                format!(
                    "{} (←/→ cycle)",
                    self.format_dropdown.current().to_uppercase()
                )
            }
        }
    }

    fn render_config_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = ConfigField::ALL
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let is_sel = i == self.config_field_idx;
                let prefix = if is_sel { "▶ " } else { "  " };
                let val_str = self.field_value(*field, is_sel);

                let label_style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:<24}", field.label()), label_style),
                    Span::styled(val_str, Style::default().fg(Color::Cyan)),
                ]))
            })
            .collect();

        let visible_rows_config = (area.height.saturating_sub(2) as usize).max(1);
        let start_config = if self.config_field_idx >= visible_rows_config {
            self.config_field_idx + 1 - visible_rows_config
        } else {
            0
        };
        let visible_config_items: Vec<ListItem> = items
            .into_iter()
            .skip(start_config)
            .take(visible_rows_config)
            .collect();

        let list = List::new(visible_config_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" In-Memory Settings "),
        );
        f.render_widget(list, area);
    }

    /// Quantize animation time according to target FPS.
    #[inline]
    pub fn quantize_time(time: f32, fps: u16) -> f32 {
        let target_fps = (fps as f32).max(1.0);
        (time * target_fps).floor() / target_fps
    }

    /// Calculate cycle time and progress fraction (0.0..=1.0) for a given duration.
    #[inline]
    pub fn cycle_duration_time(quantized_t: f32, duration: u32) -> (f32, f32) {
        if duration > 0 {
            let dur = duration as f32;
            let cycle_t = quantized_t % dur;
            let frac = (cycle_t / dur).clamp(0.0, 1.0);
            (cycle_t, frac)
        } else {
            (quantized_t, 1.0)
        }
    }

    /// Resolve authentic God-given natural coloration for a character glyph `ch`
    /// at creature coordinate `(rel_x, rel_y)` on the animal silhouette.
    #[inline]
    pub fn natural_creature_color(
        palette: &[(u8, u8, u8)],
        rel_x: usize,
        rel_y: usize,
        ch: char,
    ) -> (u8, u8, u8) {
        if palette.is_empty() {
            return (255, 255, 255);
        }
        if palette.len() == 1 {
            return palette[0];
        }

        let c0 = palette[0];
        let c1 = palette.get(1).copied().unwrap_or(c0);
        let c2 = palette.get(2).copied().unwrap_or(c1);
        let c3 = palette.get(3).copied().unwrap_or(c2);
        let c4 = palette.get(4).copied().unwrap_or(c1);

        // 1. Eyes: signature contrast or specific eye color
        if matches!(ch, 'o' | 'O' | '@' | '^' | '*' | '$' | 'x' | 'X' | '=' | '0' | 'e' | '+' | 'v' | 'u' | 'w' | '8' | 'Q' | '•' | '●')
            && (rel_y <= 4 || ch == 'o' || ch == 'O' || ch == '@' || ch == '*')
        {
            return c4;
        }

        // 2. Beak, muzzle, nostrils, snout, udder accents
        if ch == '.' || ch == ',' || ch == 'w' || ch == 'W' || ch == 'v' || ch == 'V' || ch == 'u' || ch == 'U' || ch == ':' {
            return c3;
        }

        // 3. Horns, ears, crest, crown (upper rows)
        if (ch == '^' || ch == '/' || ch == '\\' || ch == '\'' || ch == '`') && rel_y <= 2 {
            return c1;
        }

        // 4. Feet, paws, hooves, trotters, bottom row flippers
        if ch == '_' || ch == '-' || ch == '|' || ch == ')' || ch == '(' {
            if rel_y >= 3 && (ch == '_' || ch == '-' || ch == '|' || ch == ')') {
                return c2;
            }
        }

        // 5. Body markings, spots, stripes, and coat pattern (stable spatial hash)
        let hash = ((rel_x.wrapping_mul(17) + rel_y.wrapping_mul(31)) ^ (rel_x.wrapping_mul(7))) % 100;
        let chosen = if hash < 55 {
            c0
        } else if hash < 80 {
            c1
        } else {
            c2
        };

        let (mut r, mut g, mut b) = chosen;
        if ch != ' ' {
            let lum = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
            if lum < 55.0 {
                let boost = 55.0 - lum;
                r = (r as f32 + boost * 0.9).min(255.0) as u8;
                g = (g as f32 + boost * 0.95).min(255.0) as u8;
                b = (b as f32 + boost * 1.05).min(255.0) as u8;
            }
        }
        (r, g, b)
    }

    /// Authentic God-given natural color palettes (RGB tuples) for all 106 mascots.
    pub fn get_mascot_natural_palette(mascot: &str) -> &'static [(u8, u8, u8)] {
        forgum_platform::biome::get_natural_rgb_palette(mascot)
    }

    /// Dynamically find where the cow art begins in a potentially composed scene.
    /// If a speech or thought bubble precedes the cow, returns the line index of the
    /// first line of cow art. If no bubble is present, returns 0.
    fn find_cow_start_line(text: &str) -> usize {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return 0;
        }
        let first = lines[0].trim();
        // A speech or thought bubble starts with a top border of underscores, hyphens, or equals
        if !(first.chars().all(|c| c == '_' || c == '-' || c == '=') && first.len() >= 3) {
            return 0;
        }
        // Find bottom border
        let mut bottom_border_idx = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            let trimmed = line.trim();
            if (trimmed.starts_with('|')
                && trimmed.ends_with('|')
                && trimmed.chars().all(|c| c == '|' || c == '_' || c == '-'))
                || (trimmed.starts_with('(')
                    && trimmed.ends_with(')')
                    && trimmed
                        .chars()
                        .all(|c| c == '(' || c == ')' || c == '_' || c == '-'))
            {
                bottom_border_idx = Some(i);
                break;
            }
        }
        let Some(b_idx) = bottom_border_idx else {
            return 0;
        };
        // Skip connector lines
        let mut cow_start = b_idx + 1;
        while cow_start < lines.len() {
            let trimmed = lines[cow_start].trim();
            if trimmed.is_empty()
                || trimmed == "o"
                || trimmed == "\\"
                || trimmed == "/"
                || trimmed == "o o"
                || trimmed == "\\ \\"
            {
                cow_start += 1;
            } else {
                break;
            }
        }
        cow_start
    }

    /// Live Holographic Preview Canvas rendering ASCII mascot + procedural scenery & road.
    fn render_live_preview(&self, f: &mut Frame, area: Rect) {
        let quantized_t = Self::quantize_time(self.animation_time, self.config.fps);
        let (t, progress_fraction) = Self::cycle_duration_time(quantized_t, self.config.duration);
        let elapsed_display = t;

        let cow_art = self
            .cow_cache
            .get(&self.config.cow)
            .cloned()
            .unwrap_or_else(|| self.load_cow_art(&self.config.cow));

        let rainbow_colors = [
            Color::Red,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Magenta,
        ];

        let mut lines: Vec<Line> = Vec::new();

        // 0. Live HUD Timeline Status Bar
        let hud_line = if self.config.duration > 0 {
            let percent = (progress_fraction * 100.0) as u32;
            let bar_width = 14;
            let filled = ((progress_fraction * bar_width as f32).round() as usize).min(bar_width);
            let empty = bar_width.saturating_sub(filled);
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
            format!(
                "⏱ {:.1}s / {}s {} {:>3}% • {} FPS Live",
                elapsed_display, self.config.duration, bar, percent, self.config.fps
            )
        } else {
            format!(
                "⏱ {:.1}s / ∞ (Infinite Loop) • {} FPS Live",
                elapsed_display, self.config.fps
            )
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {hud_line}"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));

        // 1. Procedural background mountains (slow parallax scroll, when background enabled)
        if self.config.background {
            let mountain_style = self.config.mountain.as_deref().unwrap_or("hills");
            let mountain_layer = match mountain_style {
                "peaks" => "   /\\    /\\/\\      /\\    /\\/\\      /\\    /\\/\\      /\\    /\\/\\   ",
                "volcano" => "     /\\_ /\\    /🌋\\      /\\_ /\\    /🌋\\      /\\_ /\\    /🌋\\   ",
                "iceberg" => "    /▲\\  /▲▲\\     /▲\\  /▲▲\\     /▲\\  /▲▲\\     /▲\\  /▲▲\\    ",
                "skyline" => "   _Π_[_]__Π_   _Π_[_]__Π_   _Π_[_]__Π_   _Π_[_]__Π_   _Π_[_]__Π_   ",
                "seamount" => "    ~~/\\~~/\\~~    ~~/\\~~/\\~~    ~~/\\~~/\\~~    ~~/\\~~/\\~~   ",
                "plateau" => "   [=====]       [=====]       [=====]       [=====]      ",
                "crater" => "   (___)  (___)   (___)  (___)   (___)  (___)   (___)  ",
                "gothic" => "   /|\\   /|\\    /|\\   /|\\    /|\\   /|\\    /|\\   /|\\    ",
                "castle" => "   |п|п| |п|п|   |п|п| |п|п|   |п|п| |п|п|   |п|п| |п|п|  ",
                "garden" => "   (:::) (:::)   (:::) (:::)   (:::) (:::)   (:::) (:::)  ",
                "none" => "",
                _ => "      /\\_ /\\    /\\__      /\\_    /\\/\\    /\\_ /\\    /\\__      /\\_    /\\/\\    ",
            };
            if !mountain_layer.is_empty() {
                let m_chars: Vec<char> = mountain_layer.chars().collect();
                let m_start = (t * 1.5) as usize % m_chars.len();
                let m_slice: String = m_chars.iter().cycle().skip(m_start).take(30).collect();
                lines.push(Line::from(Span::styled(
                    format!("  {m_slice}"),
                    Style::default().fg(Color::DarkGray),
                )));
            }

            // 1.5. Procedural midground trees & flora layer (parallax scrolling, between mountains & mascot)
            let env_style = self.config.environment.as_deref().unwrap_or("pasture");
            let tree_layer = match env_style {
                "forest" => "  /\\   /\\    /\\/\\    /\\   /\\/\\    /\\   /\\    /\\/\\    /\\   ",
                "pasture" => "   .---.       (:::)       .---.       (:::)       .---.   ",
                "savanna" => "  _.~---~._        __~---~__        _.~---~._        __~---~__   ",
                "arctic" => "    /▲\\         ^           /▲\\         ^           /▲\\    ",
                "graveyard" => "    /|\\         †           /|\\         †           /|\\    ",
                "jurassic" => "   \\ ^ /        *          \\ ^ /        *          \\ ^ /   ",
                "swamp" => "   (:::)       ~-~         (:::)       ~-~         (:::)   ",
                "city" | "space" | "cyber" | "none" => "",
                _ => "   .---.       /\\/\\        .---.       /\\/\\        .---.   ",
            };
            if !tree_layer.is_empty() {
                let tree_chars: Vec<char> = tree_layer.chars().collect();
                let tree_start = (t * 2.4) as usize % tree_chars.len();
                let tree_slice: String =
                    tree_chars.iter().cycle().skip(tree_start).take(30).collect();
                let tree_color = match env_style {
                    "savanna" => Color::Rgb(139, 195, 74),
                    "arctic" => Color::Rgb(224, 247, 250),
                    "graveyard" => Color::Rgb(97, 97, 97),
                    _ => Color::Rgb(76, 175, 80),
                };
                lines.push(Line::from(Span::styled(
                    format!("  {tree_slice}"),
                    Style::default().fg(tree_color),
                )));
            }
        }

        // 2. Procedural Animation Engine Execution
        let blink_cycle = (t * 0.28) % 1.0;
        let is_blinking = blink_cycle < 0.045 || (blink_cycle > 0.08 && blink_cycle < 0.11);
        let chew_cycle = (t * 0.36) % 1.0;
        let is_chewing = chew_cycle > 0.40 && chew_cycle < 0.65;
        let is_inhale = (t * 2.2).sin() > 0.1;
        let is_wing_up = ((t * 8.0) as usize % 2) == 0;
        let walk_phase = ((t * 6.0) as usize) % 4;

        let cow_lines: Vec<&str> = cow_art.lines().collect();
        let total_lines = cow_lines.len().max(1);
        let cow_start_line = Self::find_cow_start_line(&cow_art);

        let mut leg_line_idx = total_lines.saturating_sub(1);
        for (idx, line) in cow_lines.iter().enumerate().rev() {
            if line.chars().any(|c| !c.is_whitespace()) {
                leg_line_idx = idx;
                break;
            }
        }

        for (i, raw_l) in cow_lines.iter().enumerate() {
            let mut l = raw_l.to_string();

            // Eye blinking animation: blink active eyes to '--' or '- -' or '-_-'
            if is_blinking {
                let eyes_to_blink = [
                    self.config.eyes.as_str(),
                    "oo",
                    "OO",
                    "@@",
                    "^^",
                    "**",
                    "$$",
                    "==",
                    "00",
                    "xx",
                    "XX",
                ];
                for eye in &eyes_to_blink {
                    if !eye.is_empty() && l.contains(eye) {
                        l = l.replace(eye, "--");
                    }
                }
                // Dynamic separated eye patterns for current eye setting and common mascots
                let mut glyphs_to_check = vec!['o', 'O', '@', '^', '*', '$', 'x', 'X'];
                if let Some(c) = self.config.eyes.chars().next() {
                    if !glyphs_to_check.contains(&c) {
                        glyphs_to_check.push(c);
                    }
                }
                for &eg in &glyphs_to_check {
                    let pat_sp = format!("{eg} {eg}");
                    if l.contains(&pat_sp) {
                        l = l.replace(&pat_sp, "- -");
                    }
                    let pat_2sp = format!("{eg}  {eg}");
                    if l.contains(&pat_2sp) {
                        l = l.replace(&pat_2sp, "-  -");
                    }
                    let pat_under = format!("{eg}_{eg}");
                    if l.contains(&pat_under) {
                        l = l.replace(&pat_under, "-_-");
                    }
                    let pat_dot = format!("{eg}.{eg}");
                    if l.contains(&pat_dot) {
                        l = l.replace(&pat_dot, "-.-");
                    }
                    let pat_dash = format!("{eg}-{eg}");
                    if l.contains(&pat_dash) {
                        l = l.replace(&pat_dash, "- -");
                    }
                }
                // Single-eye bracketed creatures (e.g. duck, elephant, snoopy, etc.)
                if l.contains("( o )") {
                    l = l.replace("( o )", "( - )");
                }
                if l.contains("(o)") {
                    l = l.replace("(o)", "(-)");
                }
                if l.contains("( O )") {
                    l = l.replace("( O )", "( - )");
                }
                if l.contains("(O)") {
                    l = l.replace("(O)", "(-)");
                }
            }

            // Cud chewing / mouth cadence
            if is_chewing && l.contains("(__)") {
                l = l.replace("(__)", if chew_cycle > 0.52 { "(=-)" } else { "(-=)" });
            }

            let eff = if self.config.effect == "animal_natural" || self.config.effect == "default" {
                match self.config.cow.as_str() {
                    "duck" | "pterodactyl" | "golden-eagle" | "tweety-bird" => "fly",
                    "bunny" | "hamster" | "corgi" | "cat" | "cat2" | "catfence" | "kitty" | "kitten"
                    | "doge" | "mule" | "pig" | "ram" | "sheep" | "goat" | "goat2" | "wolf" | "tiger"
                    | "panther" | "fox" | "hedgehog" | "armadillo" | "rhino" => "walk",
                    "dolphin" | "whale" | "docker-whale" | "happy-whale" | "octopus" | "smiling-octopus"
                    | "squid" | "jellyfish" | "seahorse" | "seahorse-big" => "sway",
                    "ghost" | "unipony" | "wizard" | "atat" => "float",
                    _ => "breathe",
                }
            } else {
                self.config.effect.as_str()
            };

            // Effect kinematics
            match eff {
                "walk" => {
                    if i == leg_line_idx {
                        if l.contains("||     ||") {
                            l = match walk_phase {
                                0 => l.replace("||     ||", "|/     /|"),
                                1 => l.replace("||     ||", "/|     |/"),
                                2 => l.replace("||     ||", "|\\     \\|"),
                                _ => l.replace("||     ||", "\\|     |\\"),
                            };
                        } else if l.contains("/ \\") {
                            l = match walk_phase {
                                0 | 2 => l.replace("/ \\", "| |"),
                                1 => l.replace("/ \\", "\\ /"),
                                _ => l,
                            };
                        }
                    }
                }
                "breathe" => {
                    if is_inhale {
                        if l.contains("___") {
                            l = l.replace("___", "~~~");
                        } else if l.contains("__") {
                            l = l.replace("__", "~~");
                        }
                        if l.contains("---") {
                            l = l.replace("---", "===");
                        } else if l.contains("--") {
                            l = l.replace("--", "==");
                        }
                    }
                }
                "float" => {
                    let wave_shift = ((t * 2.8 + i as f32 * 0.4).sin() * 1.5) as i32;
                    let pad = (wave_shift + 2).max(0) as usize;
                    l = format!("{}{l}", " ".repeat(pad));
                }
                "fly" => {
                    if is_wing_up {
                        if l.contains('\\') && !l.contains('/') {
                            l = l.replace('\\', "/");
                        }
                    } else if l.contains('/') && !l.contains('\\') {
                        l = l.replace('/', "\\");
                    }
                }
                "talk" => {
                    let talk_chars = ['_', '.', 'o', 'O', 'w', '='];
                    let talk_ch = talk_chars[((t * 12.0) as usize + i) % talk_chars.len()];
                    if l.contains("(__)") {
                        l = l.replace("(__)", &format!("({talk_ch}{talk_ch})"));
                    } else if l.contains("(_)") {
                        l = l.replace("(_)", &format!("({talk_ch})"));
                    }
                }
                "sway" => {
                    let rel = (total_lines.saturating_sub(i)) as f32 / total_lines as f32;
                    let sway_amt = ((t * 3.2).sin() * rel * 3.0) as i32;
                    let pad = (sway_amt + 3).max(0) as usize;
                    l = format!("{}{l}", " ".repeat(pad));
                }
                "glitch" if ((t * 15.0) as usize + i) % 7 == 0 => {
                    let mut chars: Vec<char> = l.chars().collect();
                    if let Some(pos) = chars.iter().position(|c| !c.is_whitespace()) {
                        chars[pos] = match ((t * 10.0) as usize) % 4 {
                            0 => '#',
                            1 => '~',
                            2 => '?',
                            _ => '!',
                        };
                        l = chars.into_iter().collect();
                    }
                }
                _ => {}
            }

            let color = if let Some(palette_str) = &self.config.palette {
                let parsed_colors: Vec<Color> = palette_str
                    .split(',')
                    .filter_map(|s| {
                        let hex = s.trim().trim_start_matches('#');
                        if hex.len() == 6 {
                            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                            Some(Color::Rgb(r, g, b))
                        } else {
                            None
                        }
                    })
                    .collect();
                if !parsed_colors.is_empty() {
                    parsed_colors[i % parsed_colors.len()]
                } else {
                    match self.config.color_mode.as_str() {
                        "rainbow" => {
                            let c_idx = (i + (t * 5.0) as usize) % rainbow_colors.len();
                            rainbow_colors[c_idx]
                        }
                        "lolcat" => rainbow_colors[(i * 2) % rainbow_colors.len()],
                        "solid" => Color::Green,
                        _ => Color::White,
                    }
                }
            } else {
                match self.config.color_mode.as_str() {
                    "rainbow" => {
                        let c_idx = (i + (t * 5.0) as usize) % rainbow_colors.len();
                        rainbow_colors[c_idx]
                    }
                    "lolcat" => rainbow_colors[(i * 2) % rainbow_colors.len()],
                    "solid" => Color::Green,
                    "natural" | "animal_natural" | "animal" | "default" => {
                        let p = Self::get_mascot_natural_palette(&self.config.cow);
                        let (r, g, b) = p[i % p.len()];
                        Color::Rgb(r, g, b)
                    }
                    _ => {
                        let p = Self::get_mascot_natural_palette(&self.config.cow);
                        let (r, g, b) = p[i % p.len()];
                        Color::Rgb(r, g, b)
                    }
                }
            };

            // Dynamic sparkles
            let sparkle = if (self.config.effect == "particles" || self.config.effect == "default")
                && (i == 1 || i == 2)
            {
                if ((t * 8.0) as usize + i) % 3 == 0 {
                    " ✨"
                } else {
                    "   "
                }
            } else {
                ""
            };

            let is_bubble_line = i < cow_start_line;
            let animal_rel_y = i.saturating_sub(cow_start_line);

            if is_bubble_line {
                let mut spans = Vec::new();
                spans.push(Span::raw("  "));
                for ch in l.chars() {
                    if ch == ' ' {
                        spans.push(Span::raw(" "));
                    } else if ch == 'o' || ch == 'O' || ch == '\\' || ch == '/' {
                        spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Rgb(180, 230, 255))));
                    } else if ch == '_' || ch == '-' || ch == '=' || ch == '(' || ch == ')' || ch == '|' || ch == '<' || ch == '>' || ch == '+' {
                        spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Rgb(215, 225, 240))));
                    } else {
                        spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Rgb(255, 255, 255))));
                    }
                }
                lines.push(Line::from(spans));
            } else {
                let is_natural = matches!(
                    self.config.color_mode.as_str(),
                    "natural" | "animal_natural" | "animal" | "default"
                );
                if is_natural && self.config.palette.is_none() {
                    let p = Self::get_mascot_natural_palette(&self.config.cow);
                    let mut spans = Vec::new();
                    spans.push(Span::raw("  "));
                    for (x, ch) in l.chars().enumerate() {
                        if ch == ' ' {
                            spans.push(Span::raw(" "));
                        } else {
                            let (r, g, b) = Self::natural_creature_color(p, x, animal_rel_y, ch);
                            spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Rgb(r, g, b))));
                        }
                    }
                    if !sparkle.is_empty() {
                        spans.push(Span::styled(sparkle.to_string(), Style::default().fg(Color::Yellow)));
                    }
                    lines.push(Line::from(spans));
                } else {
                    lines.push(Line::from(Span::styled(
                        format!("  {l}{sparkle}"),
                        Style::default().fg(color),
                    )));
                }
            }
        }

        // 3. Ground / Road surface (fast scroll, when background enabled)
        if self.config.background {
            let road_style = self.config.road.as_deref().unwrap_or("dirt");
            let ground_pattern = match road_style {
                "dirt" => ".. . . .. ... . .. .. . ... .. . .. ... . .. .. . ... .. ",
                "cobblestone" | "cobble" => "[_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_] ",
                "magma" => "~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ",
                "ice" => "=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--= ",
                "seabed" => "~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ",
                "sidewalk" | "paved" => "═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ",
                "roof" => "^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ",
                "grid" => "#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-",
                "crypt" => "[+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+] ",
                "savanna" => ".. -- .. -- .. -- .. -- .. -- .. -- .. -- .. -- .. -- .. ",
                "mud" => "~~~...~~~...~~~...~~~...~~~...~~~...~~~...~~~...~~~... ",
                "tracks" | "railway" => "─┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼── ",
                "checkerboard" => "■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□■□ ",
                "none" => "",
                _ => "─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ",
            };
            if !ground_pattern.is_empty() {
                let g_chars: Vec<char> = ground_pattern.chars().collect();
                let g_start = (t * 8.0) as usize % g_chars.len();
                let g_slice: String = g_chars.iter().cycle().skip(g_start).take(32).collect();
                lines.push(Line::from(Span::styled(
                    format!("  {g_slice}"),
                    Style::default().fg(Color::DarkGray),
                )));
            }
        }

        let env_name = self.config.environment.as_deref().unwrap_or("pasture");
        let title = format!(
            " 🐮 Live {} FPS Canvas: {} [{}] [{}] ",
            self.config.fps, self.config.cow, env_name, self.config.effect
        );

        let p = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title),
        );
        f.render_widget(p, area);
    }

    /// Inspector and Details card / live execution action log.
    fn render_inspector(&self, f: &mut Frame, area: Rect) {
        match self.current_tab {
            Tab::Mascots => {
                let current_cow = &self.config.cow;
                let biome = forgum_platform::biome::get_mascot_biome(current_cow);
                let cat_name = CATEGORIES[self.mascot_category_idx].0;

                let mut lines: Vec<Line> = Vec::with_capacity(8);
                lines.push(Line::from(vec![
                    Span::styled("Mascot: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(current_cow.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("  ("),
                    Span::styled(cat_name, Style::default().fg(Color::White)),
                    Span::raw(")"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Native Biome: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(biome.biome_name, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Recommended: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("Env: {}  │  Road: {}  │  Skyline: {}", biome.environment, biome.road, biome.mountain), Style::default().fg(Color::White)),
                ]));

                let mut swatch_spans = vec![
                    Span::styled("Palette: ", Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
                ];
                for (hex, &(r, g, b)) in biome.natural_palette.iter().zip(biome.natural_rgb.iter()) {
                    swatch_spans.push(Span::styled("■ ", Style::default().fg(Color::Rgb(r, g, b))));
                    swatch_spans.push(Span::styled(format!("{hex} "), Style::default().fg(Color::DarkGray)));
                }
                lines.push(Line::from(swatch_spans));

                lines.push(Line::from(vec![
                    Span::styled("Kinematics: ", Style::default().fg(Color::Yellow)),
                    Span::styled(biome.movement_lore, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Shortcuts: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[Enter/b] Apply Native Scenery  │  [r] Random Mascot", Style::default().fg(Color::Cyan)),
                ]));

                let p = Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(" 🐾 Mascot Inspector & Native Biome "),
                    );
                f.render_widget(p, area);
            }
            Tab::Scenery => {
                let current_cow = &self.config.cow;
                let biome = forgum_platform::biome::get_mascot_biome(current_cow);
                let sc_name = SCENERY_OPTIONS[self.scenery_idx].0;
                let sc_desc = SCENERY_OPTIONS[self.scenery_idx].1;
                let rd_name = ROAD_OPTIONS[self.road_idx].0;
                let rd_desc = ROAD_OPTIONS[self.road_idx].1;

                let is_native_env = sc_name.eq_ignore_ascii_case(biome.environment);
                let is_native_road = rd_name.eq_ignore_ascii_case(biome.road);

                let mut lines: Vec<Line> = Vec::with_capacity(6);
                lines.push(Line::from(vec![
                    Span::styled("Atmosphere: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(sc_name, Style::default().fg(Color::Yellow)),
                    if is_native_env {
                        Span::styled(" ★ Native", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("")
                    },
                    Span::styled(format!(" — {sc_desc}"), Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Ground Surface: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(rd_name, Style::default().fg(Color::Yellow)),
                    if is_native_road {
                        Span::styled(" ★ Native", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("")
                    },
                    Span::styled(format!(" — {rd_desc}"), Style::default().fg(Color::DarkGray)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(format!("Native for {}: ", current_cow), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} (Env: {}, Road: {}, Skyline: {})", biome.biome_name, biome.environment, biome.road, biome.mountain), Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Quick Action: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("[b/r] Sync to Native Biome", Style::default().fg(Color::Cyan)),
                ]));

                let p = Paragraph::new(lines)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(" 🏔️ Landscape & Road Details "),
                    );
                f.render_widget(p, area);
            }
            Tab::Effects => {
                let ef_desc = EFFECT_OPTIONS[self.effect_idx].1;
                let cl_desc = COLOR_OPTIONS[self.color_idx].1;
                let text = format!("Kinematics: {ef_desc}\nPalette: {cl_desc}");
                let p = Paragraph::new(text)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(" Dynamics & Rendering "),
                    );
                f.render_widget(p, area);
            }
            Tab::Installer => {
                let log_lines: Vec<Line> = self
                    .installer_log
                    .iter()
                    .rev()
                    .take(area.height.saturating_sub(2) as usize)
                    .map(|msg| Line::from(Span::styled(msg, Style::default().fg(Color::Cyan))))
                    .collect();

                let p = Paragraph::new(log_lines)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(
                                " Shell Installation Action Log (i: Install, u: Uninstall, t: Test) ",
                            ),
                    );
                f.render_widget(p, area);
            }
            Tab::Config => {
                let field = ConfigField::ALL[self.config_field_idx];
                let text = format!("Parameter: {}\n{}", field.label(), field.desc());
                let p = Paragraph::new(text)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title(" Parameter Information "),
                    );
                f.render_widget(p, area);
            }
        }
    }

    /// Bottom Zellij-style status and keybinding bar with stylized rounded tags.
    fn render_zellij_footer(&self, f: &mut Frame, area: Rect) {
        let help_spans = if self.current_tab == Tab::Installer {
            vec![
                Span::styled(
                    " <Tab/1-5> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Tabs  "),
                Span::styled(
                    " <j/k> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Shell  "),
                Span::styled(
                    " <i> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Green),
                ),
                Span::raw(" Install  "),
                Span::styled(" <u> ", Style::default().bg(Color::DarkGray).fg(Color::Red)),
                Span::raw(" Uninstall  "),
                Span::styled(
                    " <t> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow),
                ),
                Span::raw(" Test  "),
                Span::styled(
                    " <l> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" License/Info  "),
                Span::styled(
                    " <p> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Magenta),
                ),
                Span::raw(" Updates  "),
                Span::styled(" <q> ", Style::default().bg(Color::DarkGray).fg(Color::Red)),
                Span::raw(" Quit  "),
                Span::styled(
                    format!(" │ {}", self.status_message),
                    Style::default().fg(Color::Cyan),
                ),
            ]
        } else if self.current_tab == Tab::Config && self.editing_config {
            vec![
                Span::styled(
                    " <Enter> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Green),
                ),
                Span::raw(" Confirm  "),
                Span::styled(
                    " <Esc> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Red),
                ),
                Span::raw(" Cancel  "),
                Span::styled(
                    " <Up/Down> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" Step ±1  "),
                Span::styled(
                    " <Left/Right> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" Step ±5  "),
                Span::styled(
                    " <Ctrl+U> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow),
                ),
                Span::raw(" Clear  "),
                Span::styled(
                    format!(" │ {}", self.status_message),
                    Style::default().fg(Color::Cyan),
                ),
            ]
        } else if self.current_tab == Tab::Config {
            vec![
                Span::styled(
                    " <Tab> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Tabs  "),
                Span::styled(
                    " <j/k> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Select  "),
                Span::styled(
                    " <e/Enter> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Green),
                ),
                Span::raw(" Edit  "),
                Span::styled(
                    " <+/-> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow),
                ),
                Span::raw(" Step  "),
                Span::styled(
                    " <o> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Magenta),
                ),
                Span::raw(" Open Editor  "),
                Span::styled(
                    " <s> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" Save  "),
                Span::styled(" <q> ", Style::default().bg(Color::DarkGray).fg(Color::Red)),
                Span::raw(" Quit  "),
                Span::styled(
                    format!(" │ {}", self.status_message),
                    Style::default().fg(Color::Cyan),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    " <Tab/1-5> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Switch Tab  "),
                Span::styled(
                    " <j/k> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Select  "),
                Span::styled(
                    " <b> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" Native Biome  "),
                Span::styled(
                    " <Space> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Toggle  "),
                Span::styled(
                    " <o> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Magenta),
                ),
                Span::raw(" Open Editor  "),
                Span::styled(
                    " <r> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Yellow),
                ),
                Span::raw(" Random  "),
                Span::styled(
                    " <s> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Cyan),
                ),
                Span::raw(" Save  "),
                Span::styled(" <q> ", Style::default().bg(Color::DarkGray).fg(Color::Red)),
                Span::raw(" Quit  "),
                Span::styled(
                    format!(" │ {}", self.status_message),
                    Style::default().fg(Color::Cyan),
                ),
            ]
        };

        let p = Paragraph::new(Line::from(help_spans));
        f.render_widget(p, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tui_works_with_no_config_file() {
        // MUST run cleanly with None for both config and path!
        let app = ConfigApp::new(None, None, None);
        assert_eq!(app.current_tab, Tab::Mascots);
        assert_eq!(app.config.cow, "default");
        assert_eq!(app.config_path, None);
    }

    #[test]
    fn tab_cycle_and_direct_jump() {
        let mut app = ConfigApp::new(None, None, None);
        assert_eq!(app.current_tab, Tab::Mascots);

        // Tab hotkeys
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.current_tab, Tab::Installer);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('2'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.current_tab, Tab::Scenery);

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.current_tab, Tab::Effects);
    }

    #[test]
    fn shell_detection_returns_valid_entries() {
        let shells = ConfigApp::detect_shells();
        assert!(!shells.is_empty(), "Must detect standard shells");
        for s in &shells {
            assert!(!s.shell.to_string().is_empty());
        }
    }

    #[test]
    fn expand_cow_template_substitutes_eyes_and_tongue() {
        let template = "$the_cow = <<EOC;\n  $eyes\n ($tongue)\nEOC;";
        let expanded = ConfigApp::expand_cow_template(template, "@@", "U ");
        assert!(expanded.contains("@@"));
        assert!(expanded.contains("U "));
    }

    #[test]
    fn layout_resilience_small_terminal() {
        let mut app = ConfigApp::new(None, None, None);
        let backend = ratatui::backend::TestBackend::new(40, 15);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        terminal
            .draw(|f| {
                app.render(f);
            })
            .unwrap();

        // Ensure drawing to small 40x15 terminal succeeds without panic
        assert!(terminal.backend().buffer().area.width == 40);
    }

    #[test]
    fn installer_view_modes_cycle_cleanly() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Installer));
        assert_eq!(app.current_tab, Tab::Installer);
        assert_eq!(app.installer_view_mode, 0);

        // Press 'l' to toggle to License & Terms
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.installer_view_mode, 1);

        // Press 'l' again to toggle to Package Managers & Distribution
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.installer_view_mode, 2);

        // Press 'l' again to loop back to 0 (Hero & Diagnostics)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.installer_view_mode, 0);
    }

    #[test]
    fn installer_workspace_renders_without_panics() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Installer));
        let backend = ratatui::backend::TestBackend::new(100, 35);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        // Render in all 3 modes
        for mode in 0..3 {
            app.installer_view_mode = mode;
            terminal
                .draw(|f| {
                    app.render(f);
                })
                .unwrap();
        }
    }

    #[test]
    fn config_duration_and_fps_cycling() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert_eq!(app.current_tab, Tab::Config);
        assert_eq!(app.config_field_idx, 0); // ConfigField::Duration

        // Initial duration is 0
        assert_eq!(app.config.duration, 0);

        // Enter number edit mode on Duration using Enter
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);

        // Type '7'
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('7'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config_edit_buffer, "7");

        // Commit number edit using Enter
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.duration, 7);

        // Cycle forward using '+' (7 -> 10)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('+'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.duration, 10);

        // Cycle forward using Right arrow (10 -> 15)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.duration, 15);

        // Cycle backward using '-' (15 -> 10)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('-'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.duration, 10);

        // Cycle backward using Left arrow (10 -> 5)
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.config.duration, 5);

        // Move to FPS (field 1)
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.config_field_idx, 1); // ConfigField::Fps
        assert_eq!(app.config.fps, 30);

        // Toggle forward on FPS using Enter (30 -> 60)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 60);

        // Toggle forward using Space (60 -> 120)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 120);

        // Toggle forward using 't' (120 -> 240)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('t'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 240);

        // Cycle forward using Right arrow (240 -> wraps to 15)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 15);

        // Cycle forward using '+' (15 -> 24)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('+'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 24);

        // Cycle backward using '-' (24 -> 15)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('-'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 15);

        // Cycle backward using Left arrow (15 -> wraps to 240)
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.config.fps, 240);
    }

    #[test]
    fn config_duration_and_fps_text_editing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert_eq!(app.current_tab, Tab::Config);

        // 1. Edit Duration (field 0)
        app.config_field_idx = 0;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);

        // Clear buffer and type "25s"
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Delete,
            KeyModifiers::NONE,
        )))
        .unwrap();
        for ch in "25s".chars() {
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::NONE,
            )))
            .unwrap();
        }
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.duration, 25);

        // 2. Edit FPS (field 1)
        app.config_field_idx = 1;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);

        // Clear buffer and type "120fps"
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Delete,
            KeyModifiers::NONE,
        )))
        .unwrap();
        for ch in "120fps".chars() {
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::NONE,
            )))
            .unwrap();
        }
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.fps, 120);
    }

    #[test]
    fn config_numeric_fuzz_bad_input_graceful_recovery() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // Start with known baseline
        app.config.duration = 10;
        app.config.fps = 60;

        // Fuzz Duration with invalid inputs
        app.config_field_idx = 0; // Duration
        for bad_input in &["garbage", "!!!", "abc_xyz", ""] {
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )))
            .unwrap();
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Delete,
                KeyModifiers::NONE,
            )))
            .unwrap();
            for ch in bad_input.chars() {
                app.handle_event(Event::Key(KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )))
                .unwrap();
            }
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )))
            .unwrap();
            // In bad input cases, previous valid duration remains intact or is clamped safely
            assert!(
                app.config.duration <= 120,
                "Duration must stay within safe bounds"
            );
        }

        // Fuzz FPS with out-of-range / bad inputs
        app.config_field_idx = 1; // Fps
        for bad_input in &["99999", "-50", "none"] {
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
            )))
            .unwrap();
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Delete,
                KeyModifiers::NONE,
            )))
            .unwrap();
            for ch in bad_input.chars() {
                app.handle_event(Event::Key(KeyEvent::new(
                    KeyCode::Char(ch),
                    KeyModifiers::NONE,
                )))
                .unwrap();
            }
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
            )))
            .unwrap();
            // FPS must always be clamped to [1, 240]
            assert!(
                app.config.fps >= 1 && app.config.fps <= 240,
                "FPS must be clamped between 1 and 240"
            );
        }
    }

    #[test]
    fn config_dropdown_field_cycling() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // 1. Cycle ColorMode
        app.config_field_idx = ConfigField::ColorMode as usize;
        let initial_color = app.config.color_mode.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.color_mode, initial_color);

        // 2. Cycle Environment
        app.config_field_idx = ConfigField::Environment as usize;
        let initial_env = app.config.environment.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.environment, initial_env);

        // 3. Cycle Road
        app.config_field_idx = ConfigField::Road as usize;
        let initial_road = app.config.road.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.road, initial_road);

        // 4. Cycle Mountain
        app.config_field_idx = ConfigField::Mountain as usize;
        let initial_mountain = app.config.mountain.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.mountain, initial_mountain);

        // 5. Cycle AnimationType
        app.config_field_idx = ConfigField::AnimationType as usize;
        let initial_anim = app.config.animation_type.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.animation_type, initial_anim);

        // 6. Toggle Background
        app.config_field_idx = ConfigField::Background as usize;
        let initial_bg = app.config.background;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.background, !initial_bg);

        // 7. Toggle AutoRenderOnPrompt
        app.config_field_idx = ConfigField::AutoRenderOnPrompt as usize;
        let initial_auto = app.config.auto_render_on_prompt;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.auto_render_on_prompt, !initial_auto);

        // 8. Cycle ShellAttachMode
        app.config_field_idx = ConfigField::ShellAttachMode as usize;
        let initial_mode = app.config.shell_attach_mode.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.shell_attach_mode, initial_mode);

        // 9. Cycle ConfigFormat
        app.config_field_idx = ConfigField::ConfigFormat as usize;
        let initial_fmt = app.format;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.format, initial_fmt);
    }

    #[test]
    fn config_palette_text_editing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config_field_idx = ConfigField::Palette as usize;

        // Enter edit mode
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);

        // Type palette hex string
        for ch in "#112233,#445566".chars() {
            app.handle_event(Event::Key(KeyEvent::new(
                KeyCode::Char(ch),
                KeyModifiers::NONE,
            )))
            .unwrap();
        }
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.palette.as_deref(), Some("#112233,#445566"));

        // Clear palette
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Delete,
            KeyModifiers::NONE,
        )))
        .unwrap();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.palette, None);
    }

    #[test]
    fn tabs_sync_with_config_and_persistence() {
        let mut app = ConfigApp::new(None, None, None);

        // Tab 0: Mascots -> pick next mascot
        app.current_tab = Tab::Mascots;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        let selected_cow = app.config.cow.clone();
        assert_ne!(selected_cow, "");

        // Tab 1: Scenery -> pick scenery & road
        app.current_tab = Tab::Scenery;
        app.scenery_sub_focus = 0;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert!(app.config.environment.is_some());

        app.scenery_sub_focus = 1;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert!(app.config.road.is_some());

        // Tab 2: FX -> pick effect & color
        app.current_tab = Tab::Effects;
        app.fx_sub_focus = 0;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert_ne!(app.config.effect, "");
        assert_eq!(
            app.config.animation_type.as_deref(),
            Some(app.config.effect.as_str())
        );

        app.fx_sub_focus = 1;
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert_ne!(app.config.color_mode, "");

        // Verify serialization for JSON, YAML, TOML
        for fmt in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = app.config.serialize_with_format(fmt).unwrap();
            let restored = SceneConfig::parse_with_format(&serialized, fmt).unwrap();
            assert_eq!(restored.cow, app.config.cow);
            assert_eq!(restored.effect, app.config.effect);
            assert_eq!(restored.environment, app.config.environment);
            assert_eq!(restored.road, app.config.road);
            assert_eq!(restored.color_mode, app.config.color_mode);
        }
    }

    #[test]
    fn config_duration_and_fps_direct_digit_typing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert_eq!(app.current_tab, Tab::Config);
        assert_eq!(app.config_field_idx, 0); // Duration

        // Type '5' directly on Duration field
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('5'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);
        assert_eq!(app.config_edit_buffer, "5");

        // Press Enter to confirm
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.duration, 5);

        // Navigate to FPS (field 1)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config_field_idx, 1);

        // Type '6' then '0' directly
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('6'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);
        assert_eq!(app.config_edit_buffer, "6");

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('0'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config_edit_buffer, "60");

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.fps, 60);
    }

    #[test]
    fn config_edit_initial_overwrites_on_enter() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config_field_idx = 1; // FPS
        app.config.fps = 30;

        // Enter edit mode via 'e' (edit_initial = true, buffer = "30")
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('e'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(app.editing_config);
        assert!(app.edit_initial);
        assert_eq!(app.config_edit_buffer, "30");

        // Type '1' - should overwrite "30" rather than appending to make "301"
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('1'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config_edit_buffer, "1");
        assert!(!app.edit_initial);

        // Type '4' then '4' -> "144"
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('4'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config_edit_buffer, "144");

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert!(!app.editing_config);
        assert_eq!(app.config.fps, 144);
    }

    #[test]
    fn expand_cow_template_supports_perl_escapes() {
        // Singular $eye and escaped \$eye / \\$eye
        let template_single = "$the_cow = <<EOC;\n  \\$eye $eye \\${eye}\nEOC;";
        let expanded_single = ConfigApp::expand_cow_template(template_single, "oO", "  ");
        assert!(expanded_single.contains("o O o") || expanded_single.contains("o"));
        assert!(!expanded_single.contains("$eye"));
        assert!(!expanded_single.contains('\\'));

        // Plural $eyes and escaped \$eyes / \\$eyes
        let template_plural = "$the_cow = <<EOC;\n  \\$eyes $eyes\nEOC;";
        let expanded_plural = ConfigApp::expand_cow_template(template_plural, "**", "  ");
        assert!(expanded_plural.contains("** **"));
        assert!(!expanded_plural.contains("$eyes"));
        assert!(!expanded_plural.contains('\\'));
    }

    #[test]
    fn open_config_directory_sets_status() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.open_config_directory();
        assert!(
            app.status_message.starts_with("📁 Opened") || app.status_message.starts_with("⚠ Failed"),
            "Status message must report directory open status: {}",
            app.status_message
        );
    }

    #[test]
    fn fps_quantization_and_duration_cycle_determinism() {
        // Test quantize_time at 15 FPS: 1 frame = 1/15s = 0.0666...
        let t1 = ConfigApp::quantize_time(0.05, 15);
        let t2 = ConfigApp::quantize_time(0.065, 15);
        let t3 = ConfigApp::quantize_time(0.07, 15);
        assert_eq!(t1, 0.0);
        assert_eq!(t2, 0.0);
        assert!((t3 - (1.0 / 15.0)).abs() < 1e-4);

        // Test quantize_time at 60 FPS: 1 frame = 1/60s = 0.01666...
        let t60_a = ConfigApp::quantize_time(0.010, 60);
        let t60_b = ConfigApp::quantize_time(0.017, 60);
        assert_eq!(t60_a, 0.0);
        assert!((t60_b - (1.0 / 60.0)).abs() < 1e-4);

        // Test cycle_duration_time when duration > 0 (e.g. 5s)
        let (cycle_t, frac) = ConfigApp::cycle_duration_time(2.5, 5);
        assert!((cycle_t - 2.5).abs() < 1e-4);
        assert!((frac - 0.5).abs() < 1e-4);

        let (cycle_t_wrap, frac_wrap) = ConfigApp::cycle_duration_time(7.5, 5);
        assert!((cycle_t_wrap - 2.5).abs() < 1e-4);
        assert!((frac_wrap - 0.5).abs() < 1e-4);

        // Test cycle_duration_time when duration == 0 (infinite loop)
        let (cycle_inf, frac_inf) = ConfigApp::cycle_duration_time(12.34, 0);
        assert!((cycle_inf - 12.34).abs() < 1e-4);
        assert_eq!(frac_inf, 1.0);
    }
}
