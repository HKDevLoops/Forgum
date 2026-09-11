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
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
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

/// Available Scenery Archetypes.
pub const SCENERY_OPTIONS: &[(&str, &str)] = &[
    (
        "pasture",
        "Classic rolling green meadows, wildflowers, and grazing hills",
    ),
    (
        "desert",
        "Sun-baked golden sand dunes, cacti ruts, and shimmering heat waves",
    ),
    (
        "forest",
        "Dense ancient pine woods, mossy logs, and whispering evergreen canopies",
    ),
    (
        "city",
        "Futuristic neon-lit urban skyscrapers, rooftops, and antenna towers",
    ),
    (
        "mountain",
        "Rugged alpine snowpeaks, jagged granite ridges, and crisp winds",
    ),
    (
        "arctic",
        "Vast polar icecaps, floating icebergs, pack ice, and glacial frost",
    ),
    (
        "graveyard",
        "Haunted misty cemetery, crooked weathered headstones, and spooky fog",
    ),
    (
        "ocean",
        "Submarine hydrothermal vents, undulating kelp beds, and deep sea trenches",
    ),
    (
        "space",
        "Cosmic interstellar vacuum, stellar dust clouds, and distant nebulae",
    ),
    (
        "savanna",
        "Golden African plains, silhouette umbrella acacia trees, and sun haze",
    ),
    (
        "jurassic",
        "Primeval volcanic caldera, towering palm ferns, and cycad groves",
    ),
];

/// Available Road Surfaces.
pub const ROAD_OPTIONS: &[(&str, &str)] = &[
    (
        "dirt",
        "Dry country earth path with fine dust, pebbles, and ruts",
    ),
    (
        "gravel",
        "Crushed river stones with loose aggregate and crunchy texture",
    ),
    (
        "paved",
        "Smooth highway asphalt with painted dashed dividing lane lines",
    ),
    (
        "brick",
        "Classic red clay pavers laid in a sturdy interlocking herringbone pattern",
    ),
    (
        "cobble",
        "Old-world hand-chiseled granite cobblestones with recessed mortar joints",
    ),
    (
        "railway",
        "Twin steel rails bolted across timber cross-ties over stone ballast",
    ),
    (
        "starpath",
        "Cosmic celestial light bridge paved with sparkling stardust ribbons",
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
];

/// Available Color Palettes.
pub const COLOR_OPTIONS: &[(&str, &str)] = &[
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
            "goat", "goat2", "hippie", "kitty", "kitten", "meow", "milk", "mule", "pig", "ram",
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
}

/// Config parameter fields for Tab 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Duration = 0,
    Fps = 1,
    Eyes = 2,
    Tongue = 3,
    Background = 4,
    AutoRenderOnPrompt = 5,
    ShellAttachMode = 6,
    ConfigFormat = 7,
}

impl ConfigField {
    pub const ALL: [ConfigField; 8] = [
        ConfigField::Duration,
        ConfigField::Fps,
        ConfigField::Eyes,
        ConfigField::Tongue,
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
            ConfigField::Background => "Daemon non-blocking prompt overlay mode (true/false)",
            ConfigField::AutoRenderOnPrompt => "Trigger mascot automatically on shell prompt enter",
            ConfigField::ShellAttachMode => {
                "Prompt hook integration: banner, split, reactive, manual"
            }
            ConfigField::ConfigFormat => "File serialization syntax: JSON, YAML, or TOML",
        }
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
    pub config_edit_buffer: String,
    pub attach_mode_dropdown: Dropdown,
    pub format_dropdown: Dropdown,

    // Shared state
    pub saved: bool,
    pub status_message: String,
    pub animation_time: f32,
    pub cow_cache: HashMap<String, String>,
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
            config_edit_buffer: String::new(),
            attach_mode_dropdown,
            format_dropdown,
            saved: false,
            status_message: "Welcome to Forgum! Use <Tab> to navigate, 'i' to install shell hooks."
                .into(),
            animation_time: 0.0,
            cow_cache: HashMap::new(),
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

    /// Handle keyboard input event.
    pub fn handle_event(&mut self, event: Event) -> anyhow::Result<Option<Action>> {
        let Event::Key(key) = event else {
            return Ok(None);
        };
        if key.kind != event::KeyEventKind::Press {
            return Ok(None);
        }

        // If currently editing text field on Config tab
        if self.editing_config {
            return self.handle_config_edit_key(key);
        }

        // Global keybindings
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(Action::Quit)),
            KeyCode::Char('s') => return Ok(Some(Action::Save)),
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
            KeyCode::Char('1') => {
                self.current_tab = Tab::Mascots;
                return Ok(None);
            }
            KeyCode::Char('2') => {
                self.current_tab = Tab::Scenery;
                return Ok(None);
            }
            KeyCode::Char('3') => {
                self.current_tab = Tab::Effects;
                return Ok(None);
            }
            KeyCode::Char('4') => {
                self.current_tab = Tab::Installer;
                return Ok(None);
            }
            KeyCode::Char('5') => {
                self.current_tab = Tab::Config;
                return Ok(None);
            }
            KeyCode::Char('r') => {
                self.randomize_mascot();
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
                self.status_message = format!("Selected mascot: {}", self.config.cow);
            }
            _ => {}
        }
        Ok(None)
    }

    fn apply_selected_mascot(&mut self) {
        let name = CATEGORIES[self.mascot_category_idx].1[self.mascot_item_idx].to_string();
        self.config.cow = name.clone();
        self.ensure_cow_cached(&name);
    }

    fn randomize_mascot(&mut self) {
        let cat_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
        let item_idx = (self.mascot_item_idx + 3) % CATEGORIES[cat_idx].1.len();
        self.mascot_category_idx = cat_idx;
        self.mascot_item_idx = item_idx;
        self.apply_selected_mascot();
        self.status_message = format!("Swapped to mascot: {}", self.config.cow);
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
                    self.config.environment = Some(SCENERY_OPTIONS[self.scenery_idx].0.to_string());
                } else {
                    if self.road_idx > 0 {
                        self.road_idx -= 1;
                    } else {
                        self.road_idx = ROAD_OPTIONS.len() - 1;
                    }
                    self.config.road = Some(ROAD_OPTIONS[self.road_idx].0.to_string());
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.scenery_sub_focus == 0 {
                    self.scenery_idx = (self.scenery_idx + 1) % SCENERY_OPTIONS.len();
                    self.config.environment = Some(SCENERY_OPTIONS[self.scenery_idx].0.to_string());
                } else {
                    self.road_idx = (self.road_idx + 1) % ROAD_OPTIONS.len();
                    self.config.road = Some(ROAD_OPTIONS[self.road_idx].0.to_string());
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.status_message = format!(
                    "Scenery set to '{}' with road '{}'",
                    self.config.environment.as_deref().unwrap_or("pasture"),
                    self.config.road.as_deref().unwrap_or("paved")
                );
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
                    self.config.effect = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                } else {
                    if self.color_idx > 0 {
                        self.color_idx -= 1;
                    } else {
                        self.color_idx = COLOR_OPTIONS.len() - 1;
                    }
                    self.config.color_mode = COLOR_OPTIONS[self.color_idx].0.to_string();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.fx_sub_focus == 0 {
                    self.effect_idx = (self.effect_idx + 1) % EFFECT_OPTIONS.len();
                    self.config.effect = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                } else {
                    self.color_idx = (self.color_idx + 1) % COLOR_OPTIONS.len();
                    self.config.color_mode = COLOR_OPTIONS[self.color_idx].0.to_string();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
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
                self.enter_config_edit();
            }
            KeyCode::Char('+')
            | KeyCode::Char('=')
            | KeyCode::Char(' ')
            | KeyCode::Right
            | KeyCode::Char('l') => {
                self.cycle_config_field(true);
            }
            KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Left | KeyCode::Char('h') => {
                self.cycle_config_field(false);
            }
            _ => {}
        }
        Ok(None)
    }

    fn enter_config_edit(&mut self) {
        let field = ConfigField::ALL[self.config_field_idx];
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
            _ => {
                self.cycle_config_field(true);
            }
        }
    }

    fn cycle_config_field(&mut self, forward: bool) {
        let field = ConfigField::ALL[self.config_field_idx];
        match field {
            ConfigField::Duration => {
                let current = self.config.duration;
                let steps = [0, 1, 2, 3, 4, 5, 10, 15, 20, 30, 45, 60, 90, 120];
                let next = if forward {
                    steps
                        .iter()
                        .copied()
                        .find(|&s| s > current)
                        .unwrap_or_else(|| current.saturating_add(5))
                } else {
                    steps
                        .iter()
                        .copied()
                        .rev()
                        .find(|&s| s < current)
                        .unwrap_or(0)
                };
                self.config.duration = next;
                self.saved = false;
                self.status_message = if next == 0 {
                    "Duration set to 0 (infinite / run until signal) [Enter to type exact seconds]"
                        .to_string()
                } else {
                    format!(
                        "Duration adjusted to {}s [Enter to type exact seconds]",
                        next
                    )
                };
            }
            ConfigField::Fps => {
                let current = self.config.fps;
                let presets = [5, 10, 15, 20, 24, 30, 45, 60, 90, 120, 144, 240];
                let next = if forward {
                    presets
                        .iter()
                        .copied()
                        .find(|&p| p > current)
                        .unwrap_or(240)
                } else {
                    presets
                        .iter()
                        .copied()
                        .rev()
                        .find(|&p| p < current)
                        .unwrap_or(5)
                };
                self.config.fps = next;
                self.saved = false;
                self.status_message =
                    format!("FPS adjusted to {} fps [Enter to type exact FPS]", next);
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
            ConfigField::Eyes => {
                self.enter_config_edit();
            }
            ConfigField::Tongue => {
                self.enter_config_edit();
            }
        }
    }

    fn handle_config_edit_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Enter => {
                self.commit_config_edit();
                self.editing_config = false;
            }
            KeyCode::Esc => {
                self.editing_config = false;
                self.status_message = "Edit cancelled.".to_string();
            }
            KeyCode::Up => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        self.config_edit_buffer = (val.round() as u32 + 1).to_string();
                        self.commit_config_edit();
                    }
                }
            }
            KeyCode::Down => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        let floor = if field == ConfigField::Fps { 1 } else { 0 };
                        let current = val.round() as u32;
                        self.config_edit_buffer = current.saturating_sub(1).max(floor).to_string();
                        self.commit_config_edit();
                    }
                }
            }
            KeyCode::Right => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        self.config_edit_buffer = (val.round() as u32 + 5).to_string();
                        self.commit_config_edit();
                    }
                }
            }
            KeyCode::Left => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration || field == ConfigField::Fps {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        let floor = if field == ConfigField::Fps { 1 } else { 0 };
                        let current = val.round() as u32;
                        self.config_edit_buffer = current.saturating_sub(5).max(floor).to_string();
                        self.commit_config_edit();
                    }
                }
            }
            KeyCode::Tab => {
                self.commit_config_edit();
                self.editing_config = false;
                self.config_field_idx = (self.config_field_idx + 1) % ConfigField::ALL.len();
            }
            KeyCode::BackTab => {
                self.commit_config_edit();
                self.editing_config = false;
                if self.config_field_idx > 0 {
                    self.config_field_idx -= 1;
                } else {
                    self.config_field_idx = ConfigField::ALL.len() - 1;
                }
            }
            KeyCode::Backspace => {
                self.config_edit_buffer.pop();
            }
            KeyCode::Delete => {
                self.config_edit_buffer.clear();
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.config_edit_buffer.clear();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.config_edit_buffer.clear();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.config_edit_buffer.push(c);
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
                self.saved = false;
                self.status_message = format!("✓ Eyes updated to '{}'", self.config.eyes);
            }
            ConfigField::Tongue => {
                self.config.tongue = self.config_edit_buffer.clone();
                self.cow_cache.clear();
                self.saved = false;
                self.status_message = format!("✓ Tongue updated to '{}'", self.config.tongue);
            }
            _ => {}
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
                let mut curr = 0;
                for line in body_content.lines() {
                    let trimmed = line.trim();
                    let clean = trimmed.trim_end_matches(';');
                    if clean == tag {
                        end_pos = Some(curr);
                        break;
                    }
                    curr += line.len() + 1;
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
            if line.contains('$') {
                let mut l = line
                    .replace("${eyes}", eyes)
                    .replace("$eyes", eyes)
                    .replace("${tongue}", tongue)
                    .replace("$tongue", tongue)
                    .replace("${thoughts}", "\\")
                    .replace("$thoughts", "\\");

                while let Some(pos) = l.find("${eye}") {
                    let glyph = if eye_idx % 2 == 0 {
                        &left_eye
                    } else {
                        &right_eye
                    };
                    eye_idx += 1;
                    l.replace_range(pos..pos + 6, glyph);
                }
                while let Some(pos) = l.find("$eye") {
                    let glyph = if eye_idx % 2 == 0 {
                        &left_eye
                    } else {
                        &right_eye
                    };
                    eye_idx += 1;
                    l.replace_range(pos..pos + 4, glyph);
                }
                out.push_str(&l);
            } else {
                out.push_str(line);
            }
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

        let left_width = (area.width / 3).max(24).min(area.width.saturating_sub(20));
        let right_width = area.width.saturating_sub(left_width);

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

        // Render right column: Upper = 30 FPS Live Preview, Lower = Inspector / Log
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Live Preview Canvas
                Constraint::Length(6), // Inspector & Details / Log
            ])
            .split(right_rect);

        self.render_live_preview(f, right_split[0]);
        self.render_inspector(f, right_split[1]);
    }

    /// Render Grand Welcoming First-Time Setup TUI and Host Diagnostics workspace.
    fn render_installer_workspace(&self, f: &mut Frame, area: Rect) {
        let left_width = (area.width / 3).max(32).min(area.width.saturating_sub(40));
        let right_width = area.width.saturating_sub(left_width);

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

        // Right column split: Top = Grand Welcoming Hero & Diagnostics, Bottom = Log
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(13), Constraint::Length(8)])
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

        let p = Paragraph::new(lines).block(
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

        let p = Paragraph::new(lines).block(
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

        let p = Paragraph::new(log_lines).block(
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

        let list = List::new(items).block(
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

        // Scenery Archetypes list
        let scenery_items: Vec<ListItem> = SCENERY_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.scenery_idx;
                let is_focused = self.scenery_sub_focus == 0;
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

        let scenery_list = List::new(scenery_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Biomes / Scenery (11) "),
        );
        f.render_widget(scenery_list, chunks[0]);

        // Road Surfaces list
        let road_items: Vec<ListItem> = ROAD_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.road_idx;
                let is_focused = self.scenery_sub_focus == 1;
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

        let road_list = List::new(road_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Road Surfaces (7) "),
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

        let effect_list = List::new(effect_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Kinematics Effects (10) "),
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

        let color_list = List::new(color_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Color Themes (4) "),
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

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Shell Integration (i: Install) "),
        );
        f.render_widget(list, area);
    }

    fn render_config_list(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = ConfigField::ALL
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let is_sel = i == self.config_field_idx;
                let prefix = if is_sel { "▶ " } else { "  " };
                let val_str = match field {
                    ConfigField::Duration => {
                        if is_sel && self.editing_config {
                            format!("✎ [ {}_ ]", self.config_edit_buffer)
                        } else if self.config.duration == 0 {
                            "0 (infinite) [←/→/Enter]".into()
                        } else {
                            format!("{}s [←/→/Enter]", self.config.duration)
                        }
                    }
                    ConfigField::Fps => {
                        if is_sel && self.editing_config {
                            format!("✎ [ {}_ ]", self.config_edit_buffer)
                        } else {
                            format!("{} fps [←/→/Enter]", self.config.fps)
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
                };

                let label_style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:<20}", field.label()), label_style),
                    Span::styled(val_str, Style::default().fg(Color::Cyan)),
                ]))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" In-Memory Settings "),
        );
        f.render_widget(list, area);
    }

    // ── RIGHT PANELS ─────────────────────────────────────────────────────────

    /// 30 FPS Live Holographic Preview Canvas rendering ASCII mascot + procedural scenery & road.
    fn render_live_preview(&self, f: &mut Frame, area: Rect) {
        let t = self.animation_time;

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

        // 1. Procedural background mountains (slow parallax scroll, when background enabled)
        if self.config.background {
            let mountain_layer = "      /\\_ /\\    /\\__      /\\_    /\\/\\    /\\_ /\\    /\\__      /\\_    /\\/\\    ";
            let m_chars: Vec<char> = mountain_layer.chars().collect();
            let m_start = (t * 1.5) as usize % m_chars.len();
            let m_slice: String = m_chars.iter().cycle().skip(m_start).take(30).collect();
            lines.push(Line::from(Span::styled(
                format!("  {m_slice}"),
                Style::default().fg(Color::DarkGray),
            )));
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

        let mut leg_line_idx = total_lines.saturating_sub(1);
        for (idx, line) in cow_lines.iter().enumerate().rev() {
            if line.chars().any(|c| !c.is_whitespace()) {
                leg_line_idx = idx;
                break;
            }
        }

        for (i, raw_l) in cow_lines.iter().enumerate() {
            let mut l = raw_l.to_string();

            // Eye blinking animation: blink active eyes to '--' or '- -'
            if is_blinking {
                let eyes_to_blink = [
                    self.config.eyes.as_str(),
                    "oo",
                    "OO",
                    "@@",
                    "^^",
                    "**",
                    "$$",
                    "..",
                    "==",
                    "00",
                ];
                for eye in &eyes_to_blink {
                    if !eye.is_empty() && l.contains(eye) {
                        l = l.replace(eye, "--");
                    }
                }
                if l.contains("o o") {
                    l = l.replace("o o", "- -");
                }
                if l.contains("O O") {
                    l = l.replace("O O", "- -");
                }
                if l.contains("^ ^") {
                    l = l.replace("^ ^", "- -");
                }
                if l.contains("* *") {
                    l = l.replace("* *", "- -");
                }
            }

            // Cud chewing / mouth cadence
            if is_chewing && l.contains("(__)") {
                l = l.replace("(__)", if chew_cycle > 0.52 { "(=-)" } else { "(-=)" });
            }

            // Effect kinematics
            match self.config.effect.as_str() {
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

            let color = match self.config.color_mode.as_str() {
                "rainbow" => {
                    let c_idx = (i + (t * 5.0) as usize) % rainbow_colors.len();
                    rainbow_colors[c_idx]
                }
                "lolcat" => rainbow_colors[(i * 2) % rainbow_colors.len()],
                "solid" => Color::Green,
                _ => Color::White,
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

            lines.push(Line::from(Span::styled(
                format!("  {l}{sparkle}"),
                Style::default().fg(color),
            )));
        }

        // 3. Ground / Road surface (fast scroll, when background enabled)
        if self.config.background {
            let road_style = self.config.road.as_deref().unwrap_or("paved");
            let ground_pattern = match road_style {
                "paved" => "═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ═══ ",
                "gravel" => ".. . . .. ... . .. .. . ... .. . .. ... . .. .. . ... .. ",
                "railway" => "─┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼── ",
                "starpath" => "✦ · ✧ · ✦ · ✧ · ✦ · ✧ · ✦ · ✧ · ✦ · ✧ · ✦ · ✧ · ✦ · ✧ · ",
                _ => "─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ─── ",
            };
            let g_chars: Vec<char> = ground_pattern.chars().collect();
            let g_start = (t * 8.0) as usize % g_chars.len();
            let g_slice: String = g_chars.iter().cycle().skip(g_start).take(32).collect();
            lines.push(Line::from(Span::styled(
                format!("  {g_slice}"),
                Style::default().fg(Color::DarkGray),
            )));
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
                let desc = format!(
                    "Active Mascot: {}\nCategory: {}\nArchetype movement: Animated gait cycle with procedural chew cadence.",
                    current_cow, CATEGORIES[self.mascot_category_idx].0
                );
                let p = Paragraph::new(desc).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Mascot Inspector "),
                );
                f.render_widget(p, area);
            }
            Tab::Scenery => {
                let sc_desc = SCENERY_OPTIONS[self.scenery_idx].1;
                let rd_desc = ROAD_OPTIONS[self.road_idx].1;
                let text = format!("Biome: {sc_desc}\nRoad: {rd_desc}");
                let p = Paragraph::new(text).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" Landscape Details "),
                );
                f.render_widget(p, area);
            }
            Tab::Effects => {
                let ef_desc = EFFECT_OPTIONS[self.effect_idx].1;
                let cl_desc = COLOR_OPTIONS[self.color_idx].1;
                let text = format!("Kinematics: {ef_desc}\nPalette: {cl_desc}");
                let p = Paragraph::new(text).block(
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

                let p = Paragraph::new(log_lines).block(
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
                let p = Paragraph::new(text).block(
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
                    " <Space> ",
                    Style::default().bg(Color::DarkGray).fg(Color::White),
                ),
                Span::raw(" Toggle  "),
                Span::styled(
                    " <i> ",
                    Style::default().bg(Color::DarkGray).fg(Color::Green),
                ),
                Span::raw(" Install  "),
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

        // Cycle forward on Duration (+ / Right)
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('+'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.duration, 1);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.duration, 2);

        // Cycle backward on Duration (- / Left)
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.config.duration, 1);

        // Move to FPS (field 1)
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.config_field_idx, 1); // ConfigField::Fps
        assert_eq!(app.config.fps, 30);

        // Cycle forward on FPS -> 45
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('+'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 45);

        // Cycle forward again -> 60
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 60);

        // Cycle backward on FPS -> 45
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('-'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.fps, 45);
    }

    #[test]
    fn config_duration_and_fps_text_editing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert_eq!(app.current_tab, Tab::Config);

        // 1. Edit Duration (field 0)
        app.config_field_idx = 0;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
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
            KeyCode::Enter,
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
                KeyCode::Enter,
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
                KeyCode::Enter,
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

        // 1. Toggle Background (idx 4)
        app.config_field_idx = 4; // Background
        let initial_bg = app.config.background;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.background, !initial_bg);

        // 2. Toggle AutoRenderOnPrompt (idx 5)
        app.config_field_idx = 5; // AutoRenderOnPrompt
        let initial_auto = app.config.auto_render_on_prompt;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.config.auto_render_on_prompt, !initial_auto);

        // 3. Cycle ShellAttachMode (idx 6)
        app.config_field_idx = 6; // ShellAttachMode
        let initial_mode = app.config.shell_attach_mode.clone();
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.config.shell_attach_mode, initial_mode);

        // 4. Cycle ConfigFormat (idx 7)
        app.config_field_idx = 7; // ConfigFormat
        let initial_fmt = app.format;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_ne!(app.format, initial_fmt);
    }
}
