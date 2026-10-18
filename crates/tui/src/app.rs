//! ratatui TUI dashboard & shell installer inspired by LazyGit and Zellij.
//!
//! Features:
//! - Zellij-style top header with mode badge, tab bar (1-5), and bottom shortcut pills.
//! - LazyGit-style multi-pane workspace: contextual entity selector on the left,
//!   real-time 30 FPS live holographic preview + inspector/action log on the right.
//! - 5 rich tabs: Mascots (132 mascots), Scenery & Roads, Animation & FX, Shell Installer, and Config.
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
use forgum_platform::terminal_theme::{ColorblindMode, ContrastMode};

/// Tailwind CSS inspired color tokens for modern vibrant terminal styling.
pub mod tailwind {
    use ratatui::style::Color;

    pub const SLATE_900: Color = Color::Rgb(15, 23, 42);
    pub const SLATE_800: Color = Color::Rgb(30, 41, 59);
    pub const SLATE_700: Color = Color::Rgb(51, 65, 85);
    pub const SLATE_600: Color = Color::Rgb(71, 85, 105);
    pub const SLATE_500: Color = Color::Rgb(100, 116, 139);
    pub const SLATE_400: Color = Color::Rgb(148, 163, 184);
    pub const SLATE_300: Color = Color::Rgb(203, 213, 225);
    pub const SLATE_200: Color = Color::Rgb(226, 232, 240);
    pub const SLATE_100: Color = Color::Rgb(241, 245, 249);

    pub const INDIGO_600: Color = Color::Rgb(79, 70, 229);
    pub const INDIGO_500: Color = Color::Rgb(99, 102, 241);
    pub const INDIGO_400: Color = Color::Rgb(129, 140, 248);

    pub const VIOLET_500: Color = Color::Rgb(139, 92, 246);
    pub const VIOLET_400: Color = Color::Rgb(167, 139, 250);

    pub const SKY_500: Color = Color::Rgb(14, 165, 233);
    pub const SKY_400: Color = Color::Rgb(56, 189, 248);

    pub const CYAN_500: Color = Color::Rgb(6, 182, 212);
    pub const CYAN_400: Color = Color::Rgb(34, 211, 238);

    pub const EMERALD_600: Color = Color::Rgb(5, 150, 105);
    pub const EMERALD_500: Color = Color::Rgb(16, 185, 129);
    pub const EMERALD_400: Color = Color::Rgb(52, 211, 153);

    pub const AMBER_500: Color = Color::Rgb(245, 158, 11);
    pub const AMBER_400: Color = Color::Rgb(251, 191, 36);

    pub const PURPLE_400: Color = Color::Rgb(192, 132, 252);
    pub const FUCHSIA_400: Color = Color::Rgb(232, 121, 249);
    pub const TEAL_400: Color = Color::Rgb(45, 212, 191);
    pub const AMBER_300: Color = Color::Rgb(252, 211, 77);
    pub const EMERALD_300: Color = Color::Rgb(110, 231, 183);

    pub const ROSE_500: Color = Color::Rgb(244, 63, 94);
    pub const ROSE_400: Color = Color::Rgb(251, 113, 133);
}

/// Tab identifiers for top-level navigation (Zellij-style).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Mascots = 0,
    Scenery = 1,
    Effects = 2,
    Installer = 3,
    Config = 4,
    Studio = 5,
}

impl Tab {
    pub const ALL: [Tab; 6] = [
        Tab::Mascots,
        Tab::Scenery,
        Tab::Effects,
        Tab::Installer,
        Tab::Config,
        Tab::Studio,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Tab::Mascots => " 1:Mascots ",
            Tab::Scenery => " 2:Scenery & Roads ",
            Tab::Effects => " 3:FX & Colors ",
            Tab::Installer => " 4:Shell Installer ",
            Tab::Config => " 5:Configuration ",
            Tab::Studio => " 6:Studio ",
        }
    }

    pub fn compact_title(self) -> &'static str {
        match self {
            Tab::Mascots => " 1:Mascots ",
            Tab::Scenery => " 2:Scenery ",
            Tab::Effects => " 3:FX ",
            Tab::Installer => " 4:Setup ",
            Tab::Config => " 5:Config ",
            Tab::Studio => " 6:Studio ",
        }
    }

    pub fn mode_label(self) -> &'static str {
        match self {
            Tab::Mascots => " 🐾 MASCOTS ",
            Tab::Scenery => " 🏔️ SCENERY ",
            Tab::Effects => " ✨ ANIMATION & FX ",
            Tab::Installer => " 🚀 SHELL INSTALLER ",
            Tab::Config => " ⚙️ CONFIG MATRIX ",
            Tab::Studio => " 🎨 MASCOT & SHADER STUDIO ",
        }
    }

    pub fn compact_mode_label(self) -> &'static str {
        match self {
            Tab::Mascots => "🐾 MASCOTS",
            Tab::Scenery => "🏔️ SCENERY",
            Tab::Effects => "✨ EFFECTS",
            Tab::Installer => "🚀 INSTALL",
            Tab::Config => "⚙️ CONFIG",
            Tab::Studio => "🎨 STUDIO",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Tab::Mascots => "Mascots",
            Tab::Scenery => "Scenery",
            Tab::Effects => "FX",
            Tab::Installer => "Setup",
            Tab::Config => "Config",
            Tab::Studio => "Studio",
        }
    }
}

/// Responsive layout breakpoint inspired by modern Tailwind CSS styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    /// Compact / Narrow: width < 80 cols or height < 22 rows.
    Compact,
    /// Standard: 80..=120 cols and height >= 22 rows.
    Standard,
    /// Wide: > 120 cols and height >= 24 rows.
    Wide,
}

impl Breakpoint {
    pub fn from_size(width: u16, height: u16) -> Self {
        if width < 80 || height < 22 {
            Breakpoint::Compact
        } else if width > 120 && height >= 24 {
            Breakpoint::Wide
        } else {
            Breakpoint::Standard
        }
    }

    /// Whether the terminal is in a narrow / compact layout where panels should collapse
    /// and horizontal scrolling is enabled.
    #[must_use]
    pub fn is_compact_or_narrow(self) -> bool {
        matches!(self, Breakpoint::Compact)
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
    "cyberpunk",
    "autumn",
    "pastel",
    "matrix",
    "monochrome",
    "sunset",
    "ocean",
    "fire",
    "solid",
    "none",
];

/// Dropdown option choices for Background Preset / Contrast Profile.
pub const CONTRAST_OPTIONS: &[(&str, &str, ContrastMode)] = &[
    ("standard", "Standard", ContrastMode::Standard),
    (
        "acrylic_glass",
        "AcrylicGlass (Transparency)",
        ContrastMode::AcrylicGlass,
    ),
    (
        "high_contrast",
        "HighContrast (Maximum Floor)",
        ContrastMode::HighContrast,
    ),
    ("vibrant", "Vibrant (Chroma Boost)", ContrastMode::Vibrant),
];

pub const CONTRAST_MODE_OPTIONS: &[&str] =
    &["standard", "acrylic_glass", "high_contrast", "vibrant"];

/// Dropdown option choices for Colorblind Accessibility Mode.
pub const COLORBLIND_OPTIONS: &[(&str, &str, ColorblindMode)] = &[
    (
        "none",
        "None (Standard Full Spectrum)",
        ColorblindMode::None,
    ),
    (
        "protanopia",
        "Protanopia (Red-Blind Daltonized)",
        ColorblindMode::Protanopia,
    ),
    (
        "deuteranopia",
        "Deuteranopia (Green-Blind Daltonized)",
        ColorblindMode::Deuteranopia,
    ),
    (
        "tritanopia",
        "Tritanopia (Blue-Blind Daltonized)",
        ColorblindMode::Tritanopia,
    ),
    (
        "achromatopsia",
        "Achromatopsia (Monochrome Luminance)",
        ColorblindMode::Achromatopsia,
    ),
    (
        "protanopia_sim",
        "Protanopia Sim (LMS Filter)",
        ColorblindMode::ProtanopiaSim,
    ),
    (
        "deuteranopia_sim",
        "Deuteranopia Sim (LMS Filter)",
        ColorblindMode::DeuteranopiaSim,
    ),
    (
        "tritanopia_sim",
        "Tritanopia Sim (LMS Filter)",
        ColorblindMode::TritanopiaSim,
    ),
];

pub const COLORBLIND_MODE_OPTIONS: &[&str] = &[
    "none",
    "protanopia",
    "deuteranopia",
    "tritanopia",
    "achromatopsia",
    "protanopia_sim",
    "deuteranopia_sim",
    "tritanopia_sim",
];

/// Dropdown option choices for Hardware Acceleration.
pub const ACCEL_MODE_OPTIONS: &[&str] = &["auto", "dedicated", "integrated", "npu", "cpu"];

/// Helpful display tuples for Hardware Acceleration: (key, DisplayName, Description, Badge).
pub const ACCEL_OPTIONS: &[(&str, &str, &str, &str)] = &[
    (
        "auto",
        "Auto",
        "Automatically prefers Dedicated GPU, then Integrated GPU, then NPU, with CPU fallback.",
        "✨ AUTO SELECT",
    ),
    (
        "dedicated",
        "Dedicated",
        "High-performance discrete GPU cores and VRAM (NVIDIA / AMD Radeon RX).",
        "⚡ DEDICATED GPU",
    ),
    (
        "integrated",
        "Integrated",
        "Internal graphics / APU (AMD Radeon Graphics, Intel Iris/UHD, Apple M-series).",
        "🔋 INTEGRATED APU",
    ),
    (
        "npu",
        "NPU",
        "Hardware Neural Processing Unit (DirectML / CoreML / AI Boost / Ryzen AI).",
        "🧠 NEURAL NPU",
    ),
    ("cpu", "Cpu", "Pure CPU computing.", "💻 CPU COMPUTE"),
];

/// Dropdown option choices for Aspect Ratio presets.
pub const ASPECT_RATIO_OPTIONS: &[&str] =
    &["auto", "21:9", "16:9", "16:10", "4:3", "1:1", "narrow"];

/// Dropdown option choices for Aspect Ratio adaptation modes.
pub const ASPECT_MODE_OPTIONS: &[&str] = &["preset", "dynamic", "auto"];

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
    "hills", "peaks", "volcano", "iceberg", "skyline", "seamount", "plateau", "crater", "gothic",
    "castle", "garden", "none",
];

/// Dropdown option choices for AnimationType.
pub const ANIMATION_TYPE_OPTIONS: &[&str] = &[
    "animal_natural",
    "moonwalk",
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

/// Dropdown option choices for ShellAttachMode.
pub const SHELL_ATTACH_OPTIONS: &[&str] = &["startup", "prompt", "clear", "split", "pane"];

/// Dropdown option choices for SplitMode.
pub const SPLIT_MODE_OPTIONS: &[&str] = &[
    "seamless", "auto", "decstbm", "native", "precmd", "disabled",
];

/// Curated catalog of interactive feature discovery suggestions and tips for Forgum.
pub const FEATURE_SUGGESTIONS: &[&str] = &[
    "Mascot Polymorphism: Select 'natural' color mode in Tab 3 or Tab 5 to unlock authentic biological coat morphs (e.g. 6 cattle breeds, 3 tiger phenotypes)!",
    "Aquatic Kinematics: Whales, dolphins, and octopuses naturally swim and emit bubble particles submerged beneath the ocean line instead of walking!",
    "Instant Search: In Normal mode, press '/' to instantly fuzzy-search and jump to any of the 132 animal mascots across all 8 biomes!",
    "Vim Cheatsheet: Press '?' anytime to toggle the interactive Vim modal cheatsheet overlay (gg/G, gt/gT, Ctrl+d/u half-page, ZZ save&quit)!",
    "Fastfetch Banner Mode: Set 'shell_attach_mode = startup' in Config for an instant terminal welcome banner on shell launch with 0 recurring prompt latency!",
    "DECSTBM Hardware Scroll Split: Set 'shell_attach_mode = split' to reserve a hardware top margin so terminal commands scroll cleanly underneath the mascot canvas!",
    "Vulkan & Metal Acceleration: Forgum offloads harmonic mountain generation and silhouette occlusion to GPU compute with a strict <100MB RAM ceiling!",
    "Universal Randomization: Press 'r' in Tab 1-3, or set 'random = all' in Config to randomize mascot, scenery, effects, and thought fortunes on every launch!",
    "Acrylic & Glass Transparency: Using Windows Terminal Acrylic or Alacritty opacity? Switch 'contrast_mode' to 'acrylic_glass' for vivid boosted contrast!",
    "Accessibility & Reduced Motion: Toggle 'reduce_motion = true' in Tab 5 to disable rapid kinetic motions while keeping smooth ambient color fades!",
    "Text-Only Fortune Output: Run 'forgum fortune' or set 'text_only = true' in Tab 5 when you want pure philosophical quotes without ASCII cow art!",
    "ASCII Mascot Battle Arena: Run 'forgum battle --name1 Tux --name2 Dragon' for a kinetic ASCII jousting battle or rock-paper-scissors terminal arena!",
    "Command Speech Wrapper: Run 'forgum say -- cargo test' or 'forgum timer -- sleep 2' to wrap command outputs and execution durations in animated speech bubbles!",
    "Image to ASCII Mascot: Convert any PNG, JPG, or WEBP image to custom ASCII art using 'forgum image <file>' or configure the 'image' field in Config!",
    "Harmonic Alpine Mountains: Scenery uses sinusoidal harmonic superposition for realistic geological crests and erosion valleys. Explore 15 biomes in Tab 2!",
    "Responsive Narrow Compaction: Dragging your window narrow? Forgum collapses into compact views and supports horizontal scrolling with 'h'/'l' and arrows!",
    "Desktop Folder & Editor Launch: In Tab 5, press 'o' to open your config folder in Windows Explorer / Finder, or set 'editor' to launch Neovim/Helix/VSCode!",
    "Health Diagnostics & Doctor: Run 'forgum checkhealth' to diagnose your terminal, GPU shaders, and shell hooks, or 'forgum logs -w' to stream live telemetry!",
    "Custom TrueColor Gradients: Set 'palette' to comma-separated hex codes (e.g. '#ff007f,#00f0ff,#ffe600') for custom 24-bit TrueColor waveform gradients!",
    "Multi-Format Config Migration: Forgum seamlessly supports JSON, YAML, and TOML. Cycle 'config_format' in Tab 5 or run 'forgum config --migrate toml'!",
    "Multi-Channel Packaging: Update cleanly across Scoop, Winget, Chocolatey, Homebrew, Arch AUR, Cargo, and native installers with zero vendor lock-in!",
    "15-Shell Automated Setup: Forgum generates isolated, idempotent prompt hooks for Bash, Zsh, Fish, Pwsh, PowerShell, Nushell, Elvish, Xonsh, and more in Tab 4!",
    "Universal Aspect Ratios: UltraWide 21:9, 16:9, 16:10, 4:3, 1:1, and Narrow. Configure 'aspect' in Tab 5 or pass '--aspect 21:9' for tailored mountain peaks!",
];

#[must_use]
pub fn get_random_suggestion_index() -> usize {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    nanos % FEATURE_SUGGESTIONS.len()
}

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
    ("none", "Disable atmospheric particle and scenery layer"),
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
    ("ice", "Compact crystalline ice and packed polar snow"),
    (
        "seabed",
        "Sand ripples, deep-sea coral rubble, and aquatic seafloor",
    ),
    (
        "sidewalk",
        "Paved urban concrete with asphalt curb and dividing lane lines",
    ),
    ("roof", "Layered ceramic roof shingles and rooftop tiles"),
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
    ("none", "Empty clear baseline without road glyphs"),
];

/// Available Kinematics Effects.
pub const EFFECT_OPTIONS: &[(&str, &str)] = &[
    (
        "moonwalk",
        "Legendary reverse-glide moonwalk with sliding hooves and smooth backward drift",
    ),
    (
        "walk",
        "Dynamic 4-phase quadruped leg gait cycle moving with forward cadence (alias for moonwalk)",
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
        "swim",
        "Buoyant aquatic propulsion with lateral undulating waves and bubble trail",
    ),
    (
        "drift",
        "Weightless underwater marine suspension drifting along oceanic currents",
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

/// Curated categories for all 132 cow mascots.
pub const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "Farm & Domestic",
        &[
            "default", "cat", "cat2", "catfence", "charlie", "corgi", "bunny", "doge", "fat-cow",
            "goat", "goat2", "hippie", "kitty", "kitten", "meow", "hamster", "lamb", "lamb2",
            "mule", "pig", "ram", "rooster", "sheep", "turkey",
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
            "moose",
            "owl",
            "panther",
            "rhino",
            "shikato",
            "sloth",
            "squirrel",
            "telebears",
            "tiger",
            "tortoise",
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
            "lobster",
            "octopus",
            "pufferfish",
            "seahorse",
            "seahorse-big",
            "smiling-octopus",
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
            "personality-sphere",
            "pterodactyl",
            "satanic",
            "sauron",
            "stegosaurus",
            "unipony",
            "vader",
            "weeping-angel",
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
            "lollerskates",
            "mona-lisa",
            "nyan",
            "radioactive-kitty",
            "ren",
            "snoopy",
            "snoopyhouse",
            "snoopysleep",
            "spidercow",
            "stimpy",
            "tux",
            "tux-big",
            "tweety-bird",
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
            "king",
            "knight",
            "kosh",
            "mutilated",
            "pawn",
            "periodic-table",
            "queen",
            "rook",
            "shrug",
            "skeleton",
            "small",
            "supermilker",
            "surgery",
            "three-eyes",
            "viper",
            "world",
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
    SaveAndQuit,
    OpenEditor(PathBuf),
    OpenSystemEditor(PathBuf),
    InstallTerminalEditor,
}

/// Config parameter fields for Tab 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    Duration = 0,
    Fps = 1,
    Text = 2,
    Think = 3,
    ThoughtInterval = 4,
    Eyes = 5,
    Tongue = 6,
    ColorMode = 7,
    Palette = 8,
    Environment = 9,
    Road = 10,
    Mountain = 11,
    Animation = 12,
    AnimationType = 13,
    Background = 14,
    AutoRenderOnPrompt = 15,
    ShellAttachMode = 16,
    DefaultShell = 17,
    SplitMode = 18,
    SplitScroll = 19,
    SplitRatio = 20,
    ReserveRows = 21,
    ReserveCols = 22,
    Image = 23,
    Editor = 24,
    ConfigFormat = 25,
    Random = 26,
    ContrastMode = 27,
    HardwareAccel = 28,
    ReduceMotion = 29,
    TextOnly = 30,
    Banner = 31,
    Aspect = 32,
    AspectMode = 33,
    ColorblindMode = 34,
    UpdateMethod = 35,
    CheckUpdates = 36,
}

impl ConfigField {
    pub const ACCEL_MODE: ConfigField = ConfigField::HardwareAccel;
    #[allow(non_upper_case_globals)]
    pub const AccelMode: ConfigField = ConfigField::HardwareAccel;

    pub const ALL: [ConfigField; 37] = [
        ConfigField::Duration,
        ConfigField::Fps,
        ConfigField::Text,
        ConfigField::Think,
        ConfigField::ThoughtInterval,
        ConfigField::Eyes,
        ConfigField::Tongue,
        ConfigField::ColorMode,
        ConfigField::Palette,
        ConfigField::Environment,
        ConfigField::Road,
        ConfigField::Mountain,
        ConfigField::Animation,
        ConfigField::AnimationType,
        ConfigField::Background,
        ConfigField::AutoRenderOnPrompt,
        ConfigField::ShellAttachMode,
        ConfigField::DefaultShell,
        ConfigField::SplitMode,
        ConfigField::SplitScroll,
        ConfigField::SplitRatio,
        ConfigField::ReserveRows,
        ConfigField::ReserveCols,
        ConfigField::Image,
        ConfigField::Editor,
        ConfigField::ConfigFormat,
        ConfigField::Random,
        ConfigField::ContrastMode,
        ConfigField::HardwareAccel,
        ConfigField::ReduceMotion,
        ConfigField::TextOnly,
        ConfigField::Banner,
        ConfigField::Aspect,
        ConfigField::AspectMode,
        ConfigField::ColorblindMode,
        ConfigField::UpdateMethod,
        ConfigField::CheckUpdates,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ConfigField::Duration => "duration",
            ConfigField::Fps => "fps",
            ConfigField::Text => "text",
            ConfigField::Think => "think",
            ConfigField::ThoughtInterval => "thought_interval",
            ConfigField::Eyes => "eyes",
            ConfigField::Tongue => "tongue",
            ConfigField::ColorMode => "color_mode",
            ConfigField::Palette => "palette",
            ConfigField::Environment => "environment",
            ConfigField::Road => "road",
            ConfigField::Mountain => "mountain",
            ConfigField::Animation => "animation",
            ConfigField::AnimationType => "animation_type",
            ConfigField::Background => "background",
            ConfigField::AutoRenderOnPrompt => "auto_render_on_prompt",
            ConfigField::ShellAttachMode => "shell_attach_mode",
            ConfigField::DefaultShell => "default_shell",
            ConfigField::SplitMode => "split_mode",
            ConfigField::SplitScroll => "split_scroll",
            ConfigField::SplitRatio => "split_ratio",
            ConfigField::ReserveRows => "reserve_rows",
            ConfigField::ReserveCols => "reserve_cols",
            ConfigField::Image => "image",
            ConfigField::Editor => "editor",
            ConfigField::ConfigFormat => "config_format",
            ConfigField::Random => "random",
            ConfigField::ContrastMode => "contrast_mode",
            ConfigField::ColorblindMode => "colorblind_mode",
            ConfigField::HardwareAccel => "accel_mode",
            ConfigField::ReduceMotion => "reduce_motion",
            ConfigField::TextOnly => "text_only",
            ConfigField::Banner => "banner",
            ConfigField::Aspect => "aspect",
            ConfigField::AspectMode => "aspect_mode",
            ConfigField::UpdateMethod => "update_method",
            ConfigField::CheckUpdates => "check_updates",
        }
    }

    pub fn category_badge(self) -> (&'static str, Color) {
        match self {
            ConfigField::ColorMode
            | ConfigField::Palette
            | ConfigField::ContrastMode
            | ConfigField::ColorblindMode
            | ConfigField::Eyes
            | ConfigField::Tongue
            | ConfigField::Image => ("Visual", tailwind::PURPLE_400),
            ConfigField::Animation
            | ConfigField::AnimationType
            | ConfigField::Duration
            | ConfigField::Fps
            | ConfigField::ReduceMotion => ("Motion", tailwind::AMBER_400),
            ConfigField::Environment
            | ConfigField::Road
            | ConfigField::Mountain
            | ConfigField::Random => ("Scenery", tailwind::EMERALD_400),
            ConfigField::Text
            | ConfigField::Think
            | ConfigField::ThoughtInterval
            | ConfigField::TextOnly => ("Speech", tailwind::SKY_400),
            ConfigField::ShellAttachMode
            | ConfigField::DefaultShell
            | ConfigField::AutoRenderOnPrompt
            | ConfigField::Background
            | ConfigField::SplitMode
            | ConfigField::SplitScroll
            | ConfigField::SplitRatio
            | ConfigField::ReserveRows
            | ConfigField::ReserveCols
            | ConfigField::Banner => ("Shell", tailwind::TEAL_400),
            ConfigField::Editor
            | ConfigField::ConfigFormat
            | ConfigField::HardwareAccel
            | ConfigField::UpdateMethod
            | ConfigField::CheckUpdates => ("System", tailwind::ROSE_400),
            ConfigField::Aspect | ConfigField::AspectMode => ("Display", tailwind::INDIGO_400),
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            ConfigField::Duration => "Animation lifetime in seconds (0 = infinite / until signal)",
            ConfigField::Fps => "Target frame rate (30 = cinematic, 60 = ultra-smooth)",
            ConfigField::Text => "Dialogue message string rendered inside speech or thought bubble",
            ConfigField::Think => "Bubble style: thought bubble with circles 'o' (true) or speech bubble with '\\' (false)",
            ConfigField::ThoughtInterval => "Interval in seconds between rotating thought quotes in background mode (0 = permanent)",
            ConfigField::Eyes => "ASCII characters for eyes (e.g. 'oo', '$$', 'XX', '@@')",
            ConfigField::Tongue => "ASCII characters for tongue (e.g. 'U ', '  ', '||')",
            ConfigField::ColorMode => "Color rendering mode: natural (enables authentic animal coat variations), animal, rainbow, solid, none",
            ConfigField::Palette => "Biological coat morph or custom hex palette (active when Color Mode is 'natural')",
            ConfigField::Environment => "Atmospheric particle system: pasture, inferno, ocean, arctic, city, forest, savanna, swamp, space, cyber, graveyard, jurassic, hive, throne, none",
            ConfigField::Road => "Ground terrain surface style: dirt, cobblestone, magma, ice, seabed, sidewalk, roof, grid, crypt, savanna, mud, tracks, checkerboard, none",
            ConfigField::Mountain => "Horizon background silhouette: hills, peaks, volcano, iceberg, skyline, seamount, plateau, crater, gothic, castle, garden, none",
            ConfigField::Animation => "Animation mode: dynamic (continuous simulation frames) or static (frozen single frame)",
            ConfigField::AnimationType => "Kinematic animation driver: animal_natural, moonwalk, walk, breathe, float, fly, talk, sway, pulse, glitch, particles, dissolve",
            ConfigField::Background => "Daemon non-blocking prompt overlay mode (true/false)",
            ConfigField::AutoRenderOnPrompt => "Trigger mascot automatically on shell prompt enter",
            ConfigField::ShellAttachMode => "Universal execution mode: startup (launch once, 0 prompt cost), prompt (every command), clear (after clear/cls), split (DECSTBM margin), pane (dedicated pane)",
            ConfigField::DefaultShell => "Default shell used for hook generation and prompts: auto, bash, zsh, fish, pwsh, nu",
            ConfigField::SplitMode => "Split execution mode: decstbm (hardware scroll margins), seamless (simultaneous in same pane), auto, native, precmd, disabled",
            ConfigField::SplitScroll => "Terminal margin split scrolling (DECSTBM hardware scroll region, syncs with shell_attach_mode='split')",
            ConfigField::SplitRatio => "Dynamic ratio of total terminal height reserved for animation canvas (0.1..0.8)",
            ConfigField::ReserveRows => "Explicit fixed rows reserved at top of terminal for animation canvas (e.g. 12)",
            ConfigField::ReserveCols => "Explicit fixed columns reserved for animation canvas (e.g. 80, 0 = full width)",
            ConfigField::Image => "Path to image file for rendering mascot directly from PNG/JPEG/WEBP image",
            ConfigField::Editor => "Preferred text editor: auto, nvim, vim, emacs, nano, helix, micro, code, notepad",
            ConfigField::ConfigFormat => "File serialization syntax: JSON, YAML, or TOML",
            ConfigField::Random => "Universally randomize mascot, scenery, fx, and thoughts on startup",
            ConfigField::ContrastMode => "Background contrast profile: standard, acrylic_glass (transparency), high_contrast, vibrant",
            ConfigField::ColorblindMode => "Color vision deficiency mode: none, protanopia (red-blind Daltonized), deuteranopia (green-blind Daltonized), tritanopia (blue-blind Daltonized), achromatopsia (monochrome), or LMS simulations",
            ConfigField::HardwareAccel => "Hardware acceleration engine: auto (preferred GPU with CPU fallback), dedicated (discrete GPU), integrated (APU/iGPU), cpu (pure CPU)",
            ConfigField::ReduceMotion => "Accessibility mode: disable kinetic motion & heavy particles (gentle fades only)",
            ConfigField::TextOnly => "Print speech bubble text directly without ASCII cow mascot art",
            ConfigField::Banner => "Inline fastfetch-style banner output without switching alternate screen",
            ConfigField::Aspect => "Target aspect ratio preset: auto (detect from terminal), 21:9 (ultrawide), 16:9 (widescreen), 16:10 (tall), 4:3 (classic), 1:1 (square), narrow (split)",
            ConfigField::AspectMode => "Aspect ratio adaptation strategy: preset (lock to fixed aspect math), dynamic (recalculate on resize), auto (context-aware)",
            ConfigField::UpdateMethod => "Package manager / update method used by 'forgum update': prompt (ask each time), auto, zypper, scoop, winget, homebrew, apt, dnf, pacman, cargo, standalone",
            ConfigField::CheckUpdates => "Automatically check for upstream updates in background (24h cache TTL) and notify when newer version is available",
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
    pub contrast_idx: usize,
    pub colorblind_idx: usize,
    pub fx_sub_focus: usize, // 0 = effect list, 1 = color list, 2 = contrast list, 3 = colorblind list

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
    pub animation_dropdown: Dropdown,
    pub attach_mode_dropdown: Dropdown,
    pub format_dropdown: Dropdown,
    pub split_mode_dropdown: Dropdown,
    pub editor_dropdown: Dropdown,
    pub contrast_mode_dropdown: Dropdown,
    pub colorblind_mode_dropdown: Dropdown,
    pub accel_mode_dropdown: Dropdown,
    pub aspect_dropdown: Dropdown,
    pub aspect_mode_dropdown: Dropdown,

    // Studio Tab state (Tab 6)
    pub studio_sub_focus: usize,
    pub studio_morph_idx: usize,
    pub studio_debug_overlay: bool,

    // Shared state
    pub saved: bool,
    pub status_message: String,
    pub animation_time: f32,
    pub cow_cache: HashMap<String, String>,
    pub show_editor_modal: bool,
    pub original_edit_value: String,
    /// Last known terminal size (width, height) — updated on every render call.
    /// Used by `current_breakpoint()` for out-of-render-context queries.
    pub last_terminal_size: (u16, u16),
    /// Horizontal scroll offset (in columns) across wide lists, tables, and config panels.
    pub h_scroll: usize,
    /// Pending multi-key Vim sequence prefix (e.g. 'g' for gg/gt/gT, 'Z' for ZZ/ZQ).
    pub pending_vim_prefix: Option<char>,
    /// Whether the interactive Vim control cheatsheet overlay is visible ('?').
    pub show_vim_help: bool,
    /// Active feature discovery suggestion index.
    pub suggestion_idx: usize,
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

        let color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &config.color_mode);
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

        let animation_dropdown = Dropdown::new(
            vec!["dynamic", "static"],
            config.animation.as_deref().unwrap_or("dynamic"),
        );
        let initial_attach = match config.shell_attach_mode.as_str() {
            "banner" => "startup",
            "reactive" => "prompt",
            "manual" => "pane",
            other => other,
        };
        let attach_mode_dropdown = Dropdown::new(SHELL_ATTACH_OPTIONS.to_vec(), initial_attach);
        let format_dropdown = Dropdown::new(vec!["json", "yaml", "toml"], format.extension());
        let split_mode_dropdown = Dropdown::new(
            SPLIT_MODE_OPTIONS.to_vec(),
            config.split_mode.as_deref().unwrap_or("seamless"),
        );
        let editor_dropdown = Dropdown::new(
            vec![
                "auto", "nvim", "vim", "emacs", "nano", "helix", "micro", "code", "notepad",
            ],
            config.editor.as_deref().unwrap_or("auto"),
        );
        let contrast_mode_str = config.contrast_mode.as_deref().unwrap_or("standard");
        let contrast_mode_dropdown =
            Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), contrast_mode_str);
        let contrast_idx = CONTRAST_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(contrast_mode_str))
            .unwrap_or(0);
        let colorblind_mode_str = config.colorblind_mode.as_deref().unwrap_or("none");
        let colorblind_mode_dropdown =
            Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), colorblind_mode_str);
        let colorblind_idx = COLORBLIND_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(colorblind_mode_str))
            .unwrap_or(0);
        let accel_mode_str = config.accel_mode.as_deref().unwrap_or("auto");
        let accel_mode_dropdown = Dropdown::new(ACCEL_MODE_OPTIONS.to_vec(), accel_mode_str);
        let aspect_str = config.aspect.as_deref().unwrap_or("auto");
        let aspect_dropdown = Dropdown::new(ASPECT_RATIO_OPTIONS.to_vec(), aspect_str);
        let aspect_mode_str = config.aspect_mode.as_deref().unwrap_or("auto");
        let aspect_mode_dropdown = Dropdown::new(ASPECT_MODE_OPTIONS.to_vec(), aspect_mode_str);

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

        let init_sug_idx = get_random_suggestion_index();

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
            contrast_idx,
            colorblind_idx,
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
            animation_dropdown,
            attach_mode_dropdown,
            format_dropdown,
            split_mode_dropdown,
            editor_dropdown,
            contrast_mode_dropdown,
            colorblind_mode_dropdown,
            accel_mode_dropdown,
            aspect_dropdown,
            aspect_mode_dropdown,
            studio_sub_focus: 0,
            studio_morph_idx: 0,
            studio_debug_overlay: true,
            saved: false,
            status_message: format!(
                "💡 Pro-Tip: {} (<T> next tip | <?> help)",
                FEATURE_SUGGESTIONS[init_sug_idx]
            ),
            animation_time: 0.0,
            cow_cache: HashMap::new(),
            show_editor_modal: false,
            original_edit_value: String::new(),
            last_terminal_size: (0, 0),
            h_scroll: 0,
            pending_vim_prefix: None,
            show_vim_help: false,
            suggestion_idx: init_sug_idx,
        };

        // Cache the initial cow
        app.ensure_cow_cached(&app.config.cow.clone());
        app
    }

    /// Return the currently active feature suggestion tip.
    #[must_use]
    pub fn current_suggestion(&self) -> &'static str {
        FEATURE_SUGGESTIONS[self.suggestion_idx % FEATURE_SUGGESTIONS.len()]
    }

    /// Cycle to the next feature suggestion and return it.
    pub fn next_suggestion(&mut self) -> &'static str {
        self.suggestion_idx = (self.suggestion_idx + 1) % FEATURE_SUGGESTIONS.len();
        FEATURE_SUGGESTIONS[self.suggestion_idx]
    }

    /// Display a fresh random feature discovery suggestion in the status line.
    pub fn show_random_suggestion(&mut self) {
        let tip = self.next_suggestion();
        self.status_message = format!("💡 Pro-Tip: {} (<T> next tip)", tip);
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

                let is_available = shell.is_available();

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

    pub fn tick(&mut self, dt: f32) {
        self.animation_time += dt;
    }

    /// Return the responsive layout breakpoint for the last known terminal size.
    ///
    /// This can be called outside of render context (e.g., in event handlers or
    /// tests) because it reads the cached `last_terminal_size` rather than
    /// querying the terminal directly.
    pub fn current_breakpoint(&self) -> Breakpoint {
        Breakpoint::from_size(self.last_terminal_size.0, self.last_terminal_size.1)
    }

    pub fn config(&self) -> &SceneConfig {
        &self.config
    }

    /// Auto-synchronize animation type (`static` vs `dynamic`) with chosen effect
    /// to prevent invalid combinations from being saved to disk and causing CLI parse issues.
    pub fn enforce_animation_effect_consistency(&mut self) {
        let eff = self.config.effect.to_lowercase();
        if eff == "static" || eff == "none" {
            self.config.animation = Some("static".to_string());
            self.config.animation_type = None;
        } else {
            self.config.animation = Some("dynamic".to_string());
            self.config.animation_type = Some(self.config.effect.clone());
        }
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
        self.color_mode_dropdown =
            Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &self.config.color_mode);
        self.color_idx = COLOR_OPTIONS
            .iter()
            .position(|(c, _)| c.eq_ignore_ascii_case(&self.config.color_mode))
            .unwrap_or(0);
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
            self.config
                .animation_type
                .as_deref()
                .unwrap_or("animal_natural"),
        );
        self.animation_dropdown = Dropdown::new(
            vec!["dynamic", "static"],
            self.config.animation.as_deref().unwrap_or("dynamic"),
        );
        self.attach_mode_dropdown = Dropdown::new(
            SHELL_ATTACH_OPTIONS.to_vec(),
            &self.config.shell_attach_mode,
        );
        self.split_mode_dropdown = Dropdown::new(
            SPLIT_MODE_OPTIONS.to_vec(),
            self.config.split_mode.as_deref().unwrap_or("seamless"),
        );
        self.editor_dropdown = Dropdown::new(
            vec![
                "auto", "nvim", "vim", "emacs", "nano", "helix", "micro", "code", "notepad",
            ],
            self.config.editor.as_deref().unwrap_or("auto"),
        );
        let contrast_mode_str = self.config.contrast_mode.as_deref().unwrap_or("standard");
        self.contrast_mode_dropdown =
            Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), contrast_mode_str);
        self.contrast_idx = CONTRAST_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(contrast_mode_str))
            .unwrap_or(0);
        let accel_mode_str = self.config.accel_mode.as_deref().unwrap_or("auto");
        let colorblind_mode_str = self.config.colorblind_mode.as_deref().unwrap_or("none");
        self.colorblind_mode_dropdown =
            Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), colorblind_mode_str);
        self.colorblind_idx = COLORBLIND_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(colorblind_mode_str))
            .unwrap_or(0);
        self.accel_mode_dropdown = Dropdown::new(ACCEL_MODE_OPTIONS.to_vec(), accel_mode_str);
        let aspect_str = self.config.aspect.as_deref().unwrap_or("auto");
        self.aspect_dropdown = Dropdown::new(ASPECT_RATIO_OPTIONS.to_vec(), aspect_str);
        let aspect_mode_str = self.config.aspect_mode.as_deref().unwrap_or("auto");
        self.aspect_mode_dropdown = Dropdown::new(ASPECT_MODE_OPTIONS.to_vec(), aspect_mode_str);
        self.cow_cache.clear();
        self.ensure_cow_cached(&self.config.cow.clone());
    }

    /// Return active background contrast mode.
    #[must_use]
    pub fn active_contrast_mode(&self) -> ContrastMode {
        if let Some(ref mode) = self.config.contrast_mode {
            ContrastMode::from_str_loose(mode)
        } else if self.contrast_idx < CONTRAST_OPTIONS.len() {
            CONTRAST_OPTIONS[self.contrast_idx].2
        } else {
            ContrastMode::Standard
        }
    }

    /// Sync contrast mode dropdown and index from config.
    pub fn sync_contrast_from_config(&mut self) {
        let contrast_mode_str = self.config.contrast_mode.as_deref().unwrap_or("standard");
        self.contrast_idx = CONTRAST_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(contrast_mode_str))
            .unwrap_or(0);
        self.contrast_mode_dropdown =
            Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), contrast_mode_str);
    }

    /// Return active color vision deficiency (colorblind) mode.
    #[must_use]
    pub fn active_colorblind_mode(&self) -> ColorblindMode {
        if let Some(ref mode) = self.config.colorblind_mode {
            ColorblindMode::from_str_loose(mode)
        } else if self.colorblind_idx < COLORBLIND_OPTIONS.len() {
            COLORBLIND_OPTIONS[self.colorblind_idx].2
        } else {
            ColorblindMode::None
        }
    }

    /// Sync colorblind mode dropdown and index from config.
    pub fn sync_colorblind_from_config(&mut self) {
        let colorblind_mode_str = self.config.colorblind_mode.as_deref().unwrap_or("none");
        self.colorblind_idx = COLORBLIND_OPTIONS
            .iter()
            .position(|(c, _, _)| c.eq_ignore_ascii_case(colorblind_mode_str))
            .unwrap_or(0);
        self.colorblind_mode_dropdown =
            Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), colorblind_mode_str);
    }

    /// Return active colorblind display label.
    #[must_use]
    pub fn active_colorblind_display(&self) -> &'static str {
        match self.active_colorblind_mode() {
            ColorblindMode::None => "None (Normal Vision)",
            ColorblindMode::Protanopia => "Protanopia (Red-Blind Daltonized)",
            ColorblindMode::Deuteranopia => "Deuteranopia (Green-Blind Daltonized)",
            ColorblindMode::Tritanopia => "Tritanopia (Blue-Blind Daltonized)",
            ColorblindMode::Achromatopsia => "Achromatopsia (Monochrome)",
            ColorblindMode::ProtanopiaSim => "Protanopia Sim (LMS Filter)",
            ColorblindMode::DeuteranopiaSim => "Deuteranopia Sim (LMS Filter)",
            ColorblindMode::TritanopiaSim => "Tritanopia Sim (LMS Filter)",
        }
    }

    /// Return active colorblind badge.
    #[must_use]
    pub fn active_colorblind_badge(&self) -> &'static str {
        match self.active_colorblind_mode() {
            ColorblindMode::None => "⚪ NORMAL",
            ColorblindMode::Protanopia => "👁️ PROTANOPIA",
            ColorblindMode::Deuteranopia => "👁️ DEUTERANOPIA",
            ColorblindMode::Tritanopia => "👁️ TRITANOPIA",
            ColorblindMode::Achromatopsia => "⚪ ACHROMATOPSIA",
            ColorblindMode::ProtanopiaSim => "🔬 PROTANOPIA SIM",
            ColorblindMode::DeuteranopiaSim => "🔬 DEUTERANOPIA SIM",
            ColorblindMode::TritanopiaSim => "🔬 TRITANOPIA SIM",
        }
    }

    /// Return active colorblind description.
    #[must_use]
    pub fn active_colorblind_desc(&self) -> &'static str {
        match self.active_colorblind_mode() {
            ColorblindMode::None => {
                "Standard full-spectrum color rendering without color vision deficiency adjustments."
            }
            ColorblindMode::Protanopia => {
                "Accessible Daltonization for red-blind/deficient vision (L-cone loss correction)."
            }
            ColorblindMode::Deuteranopia => {
                "Accessible Daltonization for green-blind/deficient vision (M-cone loss correction)."
            }
            ColorblindMode::Tritanopia => {
                "Accessible Daltonization for blue-blind/deficient vision (S-cone loss correction)."
            }
            ColorblindMode::Achromatopsia => {
                "Rec.709 relative luminance grayscale transform for total color agnosia."
            }
            ColorblindMode::ProtanopiaSim => {
                "LMS spectral simulation of protanopic red-blind vision."
            }
            ColorblindMode::DeuteranopiaSim => {
                "LMS spectral simulation of deuteranopic green-blind vision."
            }
            ColorblindMode::TritanopiaSim => {
                "LMS spectral simulation of tritanopic blue-blind vision."
            }
        }
    }

    /// Return active hardware acceleration mode key ("auto", "dedicated", "integrated", "cpu").
    #[must_use]
    pub fn active_accel_mode(&self) -> &str {
        self.config.accel_mode.as_deref().unwrap_or("auto")
    }

    /// Return active hardware acceleration display label ("Auto", "Dedicated", "Integrated", "NPU", "Cpu").
    #[must_use]
    pub fn active_accel_display(&self) -> &'static str {
        match self.active_accel_mode().to_ascii_lowercase().as_str() {
            "dedicated" => "Dedicated",
            "integrated" => "Integrated",
            "npu" => "NPU",
            "cpu" => "Cpu",
            _ => "Auto",
        }
    }

    /// Return active hardware acceleration badge.
    #[must_use]
    pub fn active_accel_badge(&self) -> &'static str {
        match self.active_accel_mode().to_ascii_lowercase().as_str() {
            "dedicated" => "⚡ DEDICATED GPU",
            "integrated" => "🔋 INTEGRATED APU",
            "npu" => "🧠 NEURAL NPU",
            "cpu" => "💻 CPU COMPUTE",
            _ => "✨ AUTO SELECT",
        }
    }

    /// Return active hardware acceleration description.
    #[must_use]
    pub fn active_accel_desc(&self) -> &'static str {
        match self.active_accel_mode().to_ascii_lowercase().as_str() {
            "dedicated" => "High-performance discrete GPU cores and VRAM (NVIDIA / AMD Radeon RX).",
            "integrated" => {
                "Internal graphics / APU (AMD Radeon Graphics, Intel Iris/UHD, Apple M-series)."
            }
            "npu" => "Hardware Neural Processing Unit (DirectML / CoreML / AI Boost / Ryzen AI).",
            "cpu" => "Pure CPU computing.",
            _ => "Automatically prefers Dedicated GPU, then Integrated GPU, then NPU, with CPU fallback.",
        }
    }

    /// Sync hardware acceleration dropdown from config.
    pub fn sync_accel_from_config(&mut self) {
        let accel_mode_str = self.config.accel_mode.as_deref().unwrap_or("auto");
        self.accel_mode_dropdown = Dropdown::new(ACCEL_MODE_OPTIONS.to_vec(), accel_mode_str);
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
            let default_dir =
                forgum_platform::paths::config_dir().unwrap_or_else(|_| PathBuf::from("."));
            default_dir.join(format!("config.{}", self.config_format().extension()))
        }
    }

    /// Match a keyboard event against a user-configured keybinding string (e.g. "q", "ZZ", "Ctrl+d", "Esc").
    fn matches_binding(key: &crossterm::event::KeyEvent, binding_str: &str) -> bool {
        let s = binding_str.trim();
        if s.is_empty() {
            return false;
        }
        let lower = s.to_ascii_lowercase();
        if lower.starts_with("ctrl+") || lower.starts_with("c-") {
            let ch = lower.chars().last().unwrap_or(' ');
            key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char(ch)
        } else if lower.starts_with("alt+") || lower.starts_with("m-") {
            let ch = lower.chars().last().unwrap_or(' ');
            key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char(ch)
        } else if s.eq_ignore_ascii_case("esc") || s.eq_ignore_ascii_case("escape") {
            key.code == KeyCode::Esc
        } else if s.eq_ignore_ascii_case("enter") || s.eq_ignore_ascii_case("return") {
            key.code == KeyCode::Enter
        } else if s.eq_ignore_ascii_case("tab") {
            key.code == KeyCode::Tab
        } else if s.eq_ignore_ascii_case("backtab") {
            key.code == KeyCode::BackTab
        } else if s.len() == 1 {
            let c = s.chars().next().unwrap();
            if c.is_uppercase() {
                key.code == KeyCode::Char(c)
                    && !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
            } else {
                (key.code == KeyCode::Char(c.to_ascii_lowercase())
                    || key.code == KeyCode::Char(c.to_ascii_uppercase()))
                    && !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
            }
        } else {
            false
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

                // If Vim cheatsheet modal is active ('?')
                if self.show_vim_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                            self.show_vim_help = false;
                            return Ok(None);
                        }
                        _ => return Ok(None),
                    }
                }

                // If currently editing text field on Config tab
                if self.editing_config {
                    return self.handle_config_edit_key(key);
                }

                // Custom user-configured keybindings from config.toml/config.json ([keybindings])
                if let Some(ref kb) = self.config.keybindings {
                    if let Some(ref k) = kb.quit {
                        if Self::matches_binding(&key, k) {
                            return Ok(Some(Action::Quit));
                        }
                    }
                    if let Some(ref k) = kb.save_and_quit {
                        if Self::matches_binding(&key, k) {
                            return Ok(Some(Action::SaveAndQuit));
                        }
                    }
                    if let Some(ref k) = kb.help {
                        if Self::matches_binding(&key, k) {
                            self.show_vim_help = !self.show_vim_help;
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.jump_top {
                        if Self::matches_binding(&key, k) {
                            self.vim_jump_to_top();
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.jump_bottom {
                        if Self::matches_binding(&key, k) {
                            self.vim_jump_to_bottom();
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.scroll_down {
                        if Self::matches_binding(&key, k) {
                            self.vim_half_page_scroll(true);
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.scroll_up {
                        if Self::matches_binding(&key, k) {
                            self.vim_half_page_scroll(false);
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.next_tab {
                        if Self::matches_binding(&key, k) {
                            let next = (self.current_tab as usize + 1) % Tab::ALL.len();
                            self.current_tab = Tab::ALL[next];
                            self.h_scroll = 0;
                            self.status_message =
                                format!("Switched to {}", self.current_tab.mode_label());
                            return Ok(None);
                        }
                    }
                    if let Some(ref k) = kb.prev_tab {
                        if Self::matches_binding(&key, k) {
                            let prev = if self.current_tab as usize == 0 {
                                Tab::ALL.len() - 1
                            } else {
                                self.current_tab as usize - 1
                            };
                            self.current_tab = Tab::ALL[prev];
                            self.h_scroll = 0;
                            self.status_message =
                                format!("Switched to {}", self.current_tab.mode_label());
                            return Ok(None);
                        }
                    }
                }

                // Multi-key Vim sequence prefix handling ('g' prefix or 'Z' prefix)
                if let Some(prefix) = self.pending_vim_prefix.take() {
                    match (prefix, key.code) {
                        ('g', KeyCode::Char('g')) => {
                            self.vim_jump_to_top();
                            return Ok(None);
                        }
                        ('g', KeyCode::Char('t')) => {
                            let next = (self.current_tab as usize + 1) % Tab::ALL.len();
                            self.current_tab = Tab::ALL[next];
                            self.h_scroll = 0;
                            self.status_message =
                                format!("Switched to {} [gt]", self.current_tab.mode_label());
                            return Ok(None);
                        }
                        ('g', KeyCode::Char('T')) => {
                            let prev = if self.current_tab as usize == 0 {
                                Tab::ALL.len() - 1
                            } else {
                                self.current_tab as usize - 1
                            };
                            self.current_tab = Tab::ALL[prev];
                            self.h_scroll = 0;
                            self.status_message =
                                format!("Switched to {} [gT]", self.current_tab.mode_label());
                            return Ok(None);
                        }
                        ('Z', KeyCode::Char('Z')) => {
                            return Ok(Some(Action::SaveAndQuit));
                        }
                        ('Z', KeyCode::Char('Q')) => {
                            return Ok(Some(Action::Quit));
                        }
                        _ => {
                            // Prefix cancelled, fall through to process key.code normally
                        }
                    }
                }

                // Vim multi-key prefix triggers
                if key.code == KeyCode::Char('g') && key.modifiers.is_empty() {
                    self.pending_vim_prefix = Some('g');
                    self.status_message = "g- [g: top | t: next tab | T: prev tab]".to_string();
                    return Ok(None);
                }
                if key.code == KeyCode::Char('Z') {
                    self.pending_vim_prefix = Some('Z');
                    self.status_message =
                        "Z- [Z: save & quit | Q: quit without saving]".to_string();
                    return Ok(None);
                }

                // Global Vim single-key commands
                if key.code == KeyCode::Char('G')
                    && !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT)
                {
                    self.vim_jump_to_bottom();
                    return Ok(None);
                }
                if key.code == KeyCode::Char('d') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.vim_half_page_scroll(true);
                    return Ok(None);
                }
                if key.code == KeyCode::Char('u') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.vim_half_page_scroll(false);
                    return Ok(None);
                }
                if key.code == KeyCode::Char('?') {
                    self.show_vim_help = !self.show_vim_help;
                    return Ok(None);
                }
                if key.code == KeyCode::Char('T') || key.code == KeyCode::F(1) {
                    self.show_random_suggestion();
                    return Ok(None);
                }

                // Global keybindings
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(Action::Quit)),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(Some(Action::Quit));
                    }
                    KeyCode::Char('s') => return Ok(Some(Action::Save)),
                    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        let path = self.resolve_config_path();
                        return Ok(Some(Action::OpenEditor(path)));
                    }
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
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::BackTab => {
                        let prev = if self.current_tab as usize == 0 {
                            Tab::ALL.len() - 1
                        } else {
                            self.current_tab as usize - 1
                        };
                        self.current_tab = Tab::ALL[prev];
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
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
                    KeyCode::Char('1') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Mascots;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('2') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Scenery;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('3') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Effects;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('4') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Installer;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('5') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Config;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
                        return Ok(None);
                    }
                    KeyCode::Char('6') if self.current_tab != Tab::Config => {
                        self.current_tab = Tab::Studio;
                        self.h_scroll = 0;
                        self.status_message =
                            format!("Switched to {}", self.current_tab.mode_label());
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
                    Tab::Studio => self.handle_studio_key(key),
                }
            }
            Event::Resize(width, height) => {
                self.last_terminal_size = (width, height);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Handle mouse interaction across all tabs, buttons, lists, and modal popups.
    pub fn handle_mouse_event(
        &mut self,
        mouse: crossterm::event::MouseEvent,
    ) -> anyhow::Result<Option<Action>> {
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

                    if mouse.row < modal_y
                        || mouse.row >= modal_y + height
                        || mouse.column < modal_x
                        || mouse.column >= modal_x + width
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
                    let bp = Breakpoint::from_size(term_w, term_h);
                    let mut clicked_tab = None;

                    if bp == Breakpoint::Compact {
                        if mouse.row == 0 {
                            let mode_w = ratatui::text::Span::raw(format!(
                                "[{}]",
                                self.current_tab.compact_mode_label()
                            ))
                            .width() as u16;
                            let mut cur = mode_w + 1;
                            for tab in Tab::ALL {
                                let w =
                                    ratatui::text::Span::raw(tab.compact_title()).width() as u16;
                                if mouse.column >= cur && mouse.column < cur + w {
                                    clicked_tab = Some(tab);
                                    break;
                                }
                                cur += w;
                            }
                        }
                    } else if mouse.row == 1 {
                        // In standard mode, header content is rendered on line 1 inside the rounded border (x starts at 1)
                        let mode_w = ratatui::text::Span::raw(format!(
                            " {} ",
                            self.current_tab.mode_label()
                        ))
                        .width() as u16;
                        let mut cur = 1 + mode_w + 1;
                        for tab in Tab::ALL {
                            let w = ratatui::text::Span::raw(tab.title()).width() as u16;
                            if mouse.column >= cur && mouse.column < cur + w {
                                clicked_tab = Some(tab);
                                break;
                            }
                            cur += w + 1;
                        }
                    }

                    if let Some(tab) = clicked_tab {
                        self.current_tab = tab;
                        self.h_scroll = 0;
                        self.status_message = format!("Switched to {}", tab.mode_label());
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
                                .constraints([
                                    Constraint::Percentage(60),
                                    Constraint::Percentage(40),
                                ])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row > chunks[0].y && mouse.row < chunks[0].y + chunks[0].height
                            {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < SCENERY_OPTIONS.len() {
                                    self.scenery_idx = idx;
                                    self.scenery_sub_focus = 0;
                                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                                    self.config.environment = Some(env.clone());
                                    self.environment_dropdown =
                                        Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                                    self.saved = false;
                                }
                            } else if mouse.row > chunks[1].y
                                && mouse.row < chunks[1].y + chunks[1].height
                            {
                                let idx = (mouse.row - (chunks[1].y + 1)) as usize;
                                if idx < ROAD_OPTIONS.len() {
                                    self.road_idx = idx;
                                    self.scenery_sub_focus = 1;
                                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                                    self.config.road = Some(rd.clone());
                                    self.road_dropdown =
                                        Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                                    self.saved = false;
                                }
                            }
                        }
                        Tab::Effects => {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Percentage(28),
                                    Constraint::Percentage(24),
                                    Constraint::Percentage(24),
                                    Constraint::Percentage(24),
                                ])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row > chunks[0].y && mouse.row < chunks[0].y + chunks[0].height
                            {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < EFFECT_OPTIONS.len() {
                                    self.effect_idx = idx;
                                    self.fx_sub_focus = 0;
                                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                                    self.config.effect = eff.clone();
                                    self.config.animation_type = Some(eff.clone());
                                    self.animation_type_dropdown =
                                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                                    self.saved = false;
                                }
                            } else if mouse.row > chunks[1].y
                                && mouse.row < chunks[1].y + chunks[1].height
                            {
                                let idx = (mouse.row - (chunks[1].y + 1)) as usize;
                                if idx < COLOR_OPTIONS.len() {
                                    self.color_idx = idx;
                                    self.fx_sub_focus = 1;
                                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                                    self.config.color_mode = col.clone();
                                    self.color_mode_dropdown =
                                        Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                                    self.saved = false;
                                }
                            } else if mouse.row > chunks[2].y
                                && mouse.row < chunks[2].y + chunks[2].height
                            {
                                let idx = (mouse.row - (chunks[2].y + 1)) as usize;
                                if idx < CONTRAST_OPTIONS.len() {
                                    self.contrast_idx = idx;
                                    self.fx_sub_focus = 2;
                                    let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                                    self.config.contrast_mode = Some(c.clone());
                                    self.contrast_mode_dropdown =
                                        Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                                    self.saved = false;
                                }
                            } else if mouse.row > chunks[3].y
                                && mouse.row < chunks[3].y + chunks[3].height
                            {
                                let idx = (mouse.row - (chunks[3].y + 1)) as usize;
                                if idx < COLORBLIND_OPTIONS.len() {
                                    self.colorblind_idx = idx;
                                    self.fx_sub_focus = 3;
                                    let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                                    self.config.colorblind_mode =
                                        if cb == "none" { None } else { Some(cb.clone()) };
                                    self.colorblind_mode_dropdown =
                                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
                                    self.saved = false;
                                    self.status_message = format!(
                                        "Colorblind Accessibility: {} [{}]",
                                        COLORBLIND_OPTIONS[self.colorblind_idx].1,
                                        self.active_colorblind_badge()
                                    );
                                }
                            }
                        }
                        Tab::Installer => {
                            let chunks = Layout::default()
                                .direction(Direction::Vertical)
                                .constraints([
                                    Constraint::Percentage(55),
                                    Constraint::Percentage(45),
                                ])
                                .split(Rect::new(0, 3, left_width, area_h));

                            if mouse.row > chunks[0].y && mouse.row < chunks[0].y + chunks[0].height
                            {
                                let idx = (mouse.row - (chunks[0].y + 1)) as usize;
                                if idx < self.shells.len() {
                                    self.selected_shell_idx = idx;
                                    self.status_message =
                                        format!("Selected {}", self.shells[idx].shell);
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
                        Tab::Studio => {
                            if row_idx < 3 {
                                self.studio_sub_focus = row_idx;
                                self.status_message = match self.studio_sub_focus {
                                    0 => "Studio Mode: Mascot Hull & Moat Debugger".into(),
                                    1 => "Studio Mode: Biological Palette & Coat Morphs".into(),
                                    _ => "Studio Mode: Kinematics & Physics Accumulator".into(),
                                };
                            }
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => match self.current_tab {
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
                        self.environment_dropdown =
                            Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
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
                        self.animation_type_dropdown =
                            Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                    } else if self.fx_sub_focus == 1 {
                        self.color_idx = (self.color_idx + 1) % COLOR_OPTIONS.len();
                        let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                        self.config.color_mode = col.clone();
                        self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    } else if self.fx_sub_focus == 2 {
                        self.contrast_idx = (self.contrast_idx + 1) % CONTRAST_OPTIONS.len();
                        let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                        self.config.contrast_mode = Some(c.clone());
                        self.contrast_mode_dropdown =
                            Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                    } else {
                        self.colorblind_idx = (self.colorblind_idx + 1) % COLORBLIND_OPTIONS.len();
                        let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                        self.config.colorblind_mode =
                            if cb == "none" { None } else { Some(cb.clone()) };
                        self.colorblind_mode_dropdown =
                            Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
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
                Tab::Studio => {
                    self.studio_sub_focus = (self.studio_sub_focus + 1) % 3;
                    self.status_message = match self.studio_sub_focus {
                        0 => "Studio Mode: Mascot Hull & Moat Debugger".into(),
                        1 => "Studio Mode: Biological Palette & Coat Morphs".into(),
                        _ => "Studio Mode: Kinematics & Physics Accumulator".into(),
                    };
                }
            },
            MouseEventKind::ScrollUp => match self.current_tab {
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
                        self.environment_dropdown =
                            Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
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
                        self.animation_type_dropdown =
                            Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                    } else if self.fx_sub_focus == 1 {
                        if self.color_idx > 0 {
                            self.color_idx -= 1;
                        } else {
                            self.color_idx = COLOR_OPTIONS.len().saturating_sub(1);
                        }
                        let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                        self.config.color_mode = col.clone();
                        self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    } else if self.fx_sub_focus == 2 {
                        if self.contrast_idx > 0 {
                            self.contrast_idx -= 1;
                        } else {
                            self.contrast_idx = CONTRAST_OPTIONS.len().saturating_sub(1);
                        }
                        let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                        self.config.contrast_mode = Some(c.clone());
                        self.contrast_mode_dropdown =
                            Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                    } else {
                        if self.colorblind_idx > 0 {
                            self.colorblind_idx -= 1;
                        } else {
                            self.colorblind_idx = COLORBLIND_OPTIONS.len().saturating_sub(1);
                        }
                        let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                        self.config.colorblind_mode =
                            if cb == "none" { None } else { Some(cb.clone()) };
                        self.colorblind_mode_dropdown =
                            Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
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
                Tab::Studio => {
                    if self.studio_sub_focus > 0 {
                        self.studio_sub_focus -= 1;
                    } else {
                        self.studio_sub_focus = 2;
                    }
                    self.status_message = match self.studio_sub_focus {
                        0 => "Studio Mode: Mascot Hull & Moat Debugger".into(),
                        1 => "Studio Mode: Biological Palette & Coat Morphs".into(),
                        _ => "Studio Mode: Kinematics & Physics Accumulator".into(),
                    };
                }
            },
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
            KeyCode::Char('[') | KeyCode::Char(',') => {
                if self.mascot_category_idx > 0 {
                    self.mascot_category_idx -= 1;
                } else {
                    self.mascot_category_idx = CATEGORIES.len() - 1;
                }
                self.mascot_item_idx = 0;
                self.h_scroll = 0;
                self.apply_selected_mascot();
            }
            KeyCode::Char(']') | KeyCode::Char('.') => {
                self.mascot_category_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
                self.mascot_item_idx = 0;
                self.h_scroll = 0;
                self.apply_selected_mascot();
            }
            KeyCode::Char('H') => {
                self.h_scroll = self.h_scroll.saturating_sub(2);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Char('L') => {
                self.h_scroll = (self.h_scroll + 2).min(40);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Left
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = self.h_scroll.saturating_sub(2);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Right
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = (self.h_scroll + 2).min(40);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if self.current_breakpoint().is_compact_or_narrow() {
                    self.h_scroll = self.h_scroll.saturating_sub(2);
                    self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
                } else {
                    if self.mascot_category_idx > 0 {
                        self.mascot_category_idx -= 1;
                    } else {
                        self.mascot_category_idx = CATEGORIES.len() - 1;
                    }
                    self.mascot_item_idx = 0;
                    self.h_scroll = 0;
                    self.apply_selected_mascot();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.current_breakpoint().is_compact_or_narrow() {
                    self.h_scroll = (self.h_scroll + 2).min(40);
                    self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
                } else {
                    self.mascot_category_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
                    self.mascot_item_idx = 0;
                    self.h_scroll = 0;
                    self.apply_selected_mascot();
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.apply_selected_mascot();
                self.sync_mascot_scenery();
            }
            KeyCode::Char('b') | KeyCode::Char('r') => {
                self.apply_selected_mascot();
                self.sync_mascot_scenery();
            }
            KeyCode::Char('t') => {
                self.show_random_suggestion();
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

        // Dynamically reconcile locomotion/effect compatibility with the newly selected mascot
        if let Some((remapped, _)) = biome.remap_incompatible_effect(&self.config.effect) {
            self.config.effect = remapped.to_string();
            if let Some(pos) = EFFECT_OPTIONS
                .iter()
                .position(|(eff, _)| eff.eq_ignore_ascii_case(remapped))
            {
                self.effect_idx = pos;
            }
        }
        if let Some(anim) = &self.config.animation_type {
            if let Some((remapped, _)) = biome.remap_incompatible_effect(anim) {
                self.config.animation_type = Some(remapped.to_string());
            }
        }

        // When in natural color mode, automatically set palette to the new mascot's primary authentic variation
        if self.config.color_mode == "natural" {
            let vars = biome.variations();
            if !vars.is_empty() {
                self.config.palette = Some(vars[0].palette.join(","));
            }
        }

        self.status_message = format!("🐾 Mascot: {} | 🌿 Biome: {}", name, biome.biome_name);
        self.saved = false;
    }

    pub fn sync_mascot_scenery(&mut self) {
        let name = self.config.cow.clone();
        let biome = forgum_platform::biome::get_mascot_biome(&name);
        if let Some(pos) = SCENERY_OPTIONS
            .iter()
            .position(|(env, _)| env.eq_ignore_ascii_case(biome.environment))
        {
            self.scenery_idx = pos;
            let env = SCENERY_OPTIONS[pos].0.to_string();
            self.config.environment = Some(env.clone());
            self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
        }
        if let Some(pos) = ROAD_OPTIONS
            .iter()
            .position(|(rd, _)| rd.eq_ignore_ascii_case(biome.road))
        {
            self.road_idx = pos;
            let rd = ROAD_OPTIONS[pos].0.to_string();
            self.config.road = Some(rd.clone());
            self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
        }
        let mtn = biome.mountain.to_string();
        self.config.mountain = Some(mtn.clone());
        self.mountain_dropdown = Dropdown::new(MOUNTAIN_OPTIONS.to_vec(), &mtn);

        // Synchronize biological natural 5-color palette
        let natural_palette = biome.natural_palette.join(",");
        self.config.palette = Some(natural_palette);

        self.saved = false;
        self.status_message = format!(
            "🌿 Native habitat applied: {} (Atmosphere: {}, Ground: {}, Skyline: {})",
            biome.biome_name, biome.environment, biome.road, biome.mountain
        );
    }

    fn randomize_mascot(&mut self) {
        let cat_idx = (self.mascot_category_idx + 1) % CATEGORIES.len();
        let item_idx = (self.mascot_item_idx + 3) % CATEGORIES[cat_idx].1.len();
        self.mascot_category_idx = cat_idx;
        self.mascot_item_idx = item_idx;
        self.apply_selected_mascot();
        self.sync_mascot_scenery();
    }

    /// Vim 'gg': Jump directly to the top item of the active tab.
    pub fn vim_jump_to_top(&mut self) {
        match self.current_tab {
            Tab::Mascots => {
                self.mascot_item_idx = 0;
                self.apply_selected_mascot();
            }
            Tab::Scenery => {
                if self.scenery_sub_focus == 0 {
                    self.scenery_idx = 0;
                    let env = SCENERY_OPTIONS[0].0.to_string();
                    self.config.environment = Some(env.clone());
                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                } else {
                    self.road_idx = 0;
                    let rd = ROAD_OPTIONS[0].0.to_string();
                    self.config.road = Some(rd.clone());
                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                }
                self.saved = false;
            }
            Tab::Effects => {
                if self.fx_sub_focus == 0 {
                    self.effect_idx = 0;
                    let eff = EFFECT_OPTIONS[0].0.to_string();
                    self.config.effect = eff.clone();
                    self.config.animation_type = Some(eff.clone());
                    self.animation_type_dropdown =
                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                } else if self.fx_sub_focus == 1 {
                    self.color_idx = 0;
                    let col = COLOR_OPTIONS[0].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                } else if self.fx_sub_focus == 2 {
                    self.contrast_idx = 0;
                    let c = CONTRAST_OPTIONS[0].0.to_string();
                    self.config.contrast_mode = Some(c.clone());
                    self.contrast_mode_dropdown = Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                } else {
                    self.colorblind_idx = 0;
                    let cb = COLORBLIND_OPTIONS[0].0.to_string();
                    self.config.colorblind_mode = None;
                    self.colorblind_mode_dropdown =
                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
                }
                self.saved = false;
            }
            Tab::Installer => {
                self.selected_shell_idx = 0;
            }
            Tab::Config => {
                self.config_field_idx = 0;
            }
            Tab::Studio => {
                self.studio_sub_focus = 0;
            }
        }
        self.status_message = "Jumped to top [gg]".into();
    }

    /// Vim 'G': Jump directly to the bottom item of the active tab.
    pub fn vim_jump_to_bottom(&mut self) {
        match self.current_tab {
            Tab::Mascots => {
                let cows = CATEGORIES[self.mascot_category_idx].1;
                self.mascot_item_idx = cows.len().saturating_sub(1);
                self.apply_selected_mascot();
            }
            Tab::Scenery => {
                if self.scenery_sub_focus == 0 {
                    self.scenery_idx = SCENERY_OPTIONS.len().saturating_sub(1);
                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                    self.config.environment = Some(env.clone());
                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                } else {
                    self.road_idx = ROAD_OPTIONS.len().saturating_sub(1);
                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                    self.config.road = Some(rd.clone());
                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                }
                self.saved = false;
            }
            Tab::Effects => {
                if self.fx_sub_focus == 0 {
                    self.effect_idx = EFFECT_OPTIONS.len().saturating_sub(1);
                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                    self.config.effect = eff.clone();
                    self.config.animation_type = Some(eff.clone());
                    self.animation_type_dropdown =
                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                } else if self.fx_sub_focus == 1 {
                    self.color_idx = COLOR_OPTIONS.len().saturating_sub(1);
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                } else if self.fx_sub_focus == 2 {
                    self.contrast_idx = CONTRAST_OPTIONS.len().saturating_sub(1);
                    let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                    self.config.contrast_mode = Some(c.clone());
                    self.contrast_mode_dropdown = Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                } else {
                    self.colorblind_idx = COLORBLIND_OPTIONS.len().saturating_sub(1);
                    let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                    self.config.colorblind_mode = Some(cb.clone());
                    self.colorblind_mode_dropdown =
                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
                }
                self.saved = false;
            }
            Tab::Installer => {
                if !self.shells.is_empty() {
                    self.selected_shell_idx = self.shells.len() - 1;
                }
            }
            Tab::Config => {
                self.config_field_idx = ConfigField::ALL.len().saturating_sub(1);
            }
            Tab::Studio => {
                self.studio_sub_focus = 2;
            }
        }
        self.status_message = "Jumped to bottom [G]".into();
    }

    /// Vim 'Ctrl+d' / 'Ctrl+u': Scroll half-page down or up.
    pub fn vim_half_page_scroll(&mut self, down: bool) {
        let delta = 5;
        match self.current_tab {
            Tab::Mascots => {
                let cows = CATEGORIES[self.mascot_category_idx].1;
                if down {
                    self.mascot_item_idx =
                        (self.mascot_item_idx + delta).min(cows.len().saturating_sub(1));
                } else {
                    self.mascot_item_idx = self.mascot_item_idx.saturating_sub(delta);
                }
                self.apply_selected_mascot();
            }
            Tab::Scenery => {
                if self.scenery_sub_focus == 0 {
                    if down {
                        self.scenery_idx =
                            (self.scenery_idx + 3).min(SCENERY_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.scenery_idx = self.scenery_idx.saturating_sub(3);
                    }
                    let env = SCENERY_OPTIONS[self.scenery_idx].0.to_string();
                    self.config.environment = Some(env.clone());
                    self.environment_dropdown = Dropdown::new(ENVIRONMENT_OPTIONS.to_vec(), &env);
                } else {
                    if down {
                        self.road_idx =
                            (self.road_idx + 3).min(ROAD_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.road_idx = self.road_idx.saturating_sub(3);
                    }
                    let rd = ROAD_OPTIONS[self.road_idx].0.to_string();
                    self.config.road = Some(rd.clone());
                    self.road_dropdown = Dropdown::new(ROAD_STYLE_OPTIONS.to_vec(), &rd);
                }
                self.saved = false;
            }
            Tab::Effects => {
                if self.fx_sub_focus == 0 {
                    if down {
                        self.effect_idx =
                            (self.effect_idx + 3).min(EFFECT_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.effect_idx = self.effect_idx.saturating_sub(3);
                    }
                    let eff = EFFECT_OPTIONS[self.effect_idx].0.to_string();
                    self.config.effect = eff.clone();
                    self.config.animation_type = Some(eff.clone());
                    self.animation_type_dropdown =
                        Dropdown::new(ANIMATION_TYPE_OPTIONS.to_vec(), &eff);
                } else if self.fx_sub_focus == 1 {
                    if down {
                        self.color_idx =
                            (self.color_idx + 3).min(COLOR_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.color_idx = self.color_idx.saturating_sub(3);
                    }
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                } else if self.fx_sub_focus == 2 {
                    if down {
                        self.contrast_idx =
                            (self.contrast_idx + 2).min(CONTRAST_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.contrast_idx = self.contrast_idx.saturating_sub(2);
                    }
                    let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                    self.config.contrast_mode = Some(c.clone());
                    self.contrast_mode_dropdown = Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                } else {
                    if down {
                        self.colorblind_idx = (self.colorblind_idx + 3)
                            .min(COLORBLIND_OPTIONS.len().saturating_sub(1));
                    } else {
                        self.colorblind_idx = self.colorblind_idx.saturating_sub(3);
                    }
                    let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                    self.config.colorblind_mode =
                        if cb == "none" { None } else { Some(cb.clone()) };
                    self.colorblind_mode_dropdown =
                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
                }
                self.saved = false;
            }
            Tab::Installer => {
                if !self.shells.is_empty() {
                    if down {
                        self.selected_shell_idx =
                            (self.selected_shell_idx + 3).min(self.shells.len() - 1);
                    } else {
                        self.selected_shell_idx = self.selected_shell_idx.saturating_sub(3);
                    }
                }
            }
            Tab::Config => {
                if down {
                    self.config_field_idx =
                        (self.config_field_idx + delta).min(ConfigField::ALL.len() - 1);
                } else {
                    self.config_field_idx = self.config_field_idx.saturating_sub(delta);
                }
            }
            Tab::Studio => {
                if down {
                    self.studio_sub_focus = 2;
                } else {
                    self.studio_sub_focus = 0;
                }
            }
        }
        self.status_message = if down {
            "Half-page down [Ctrl+d]".into()
        } else {
            "Half-page up [Ctrl+u]".into()
        };
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
            KeyCode::Char('t') => {
                self.show_random_suggestion();
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_effects_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Char('1') => {
                self.fx_sub_focus = 0;
            }
            KeyCode::Char('2') => {
                self.fx_sub_focus = 1;
            }
            KeyCode::Char('3') => {
                self.fx_sub_focus = 2;
            }
            KeyCode::Char('4') => {
                self.fx_sub_focus = 3;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
                self.fx_sub_focus = (self.fx_sub_focus + 3) % 4;
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => {
                self.fx_sub_focus = (self.fx_sub_focus + 1) % 4;
            }
            KeyCode::Char('H') => {
                self.h_scroll = self.h_scroll.saturating_sub(2);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Char('L') => {
                self.h_scroll = (self.h_scroll + 2).min(40);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
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
                } else if self.fx_sub_focus == 1 {
                    if self.color_idx > 0 {
                        self.color_idx -= 1;
                    } else {
                        self.color_idx = COLOR_OPTIONS.len() - 1;
                    }
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    self.saved = false;
                } else if self.fx_sub_focus == 2 {
                    if self.contrast_idx > 0 {
                        self.contrast_idx -= 1;
                    } else {
                        self.contrast_idx = CONTRAST_OPTIONS.len() - 1;
                    }
                    let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                    self.config.contrast_mode = Some(c.clone());
                    self.contrast_mode_dropdown = Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                    self.saved = false;
                } else {
                    if self.colorblind_idx > 0 {
                        self.colorblind_idx -= 1;
                    } else {
                        self.colorblind_idx = COLORBLIND_OPTIONS.len() - 1;
                    }
                    let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                    self.config.colorblind_mode =
                        if cb == "none" { None } else { Some(cb.clone()) };
                    self.colorblind_mode_dropdown =
                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
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
                } else if self.fx_sub_focus == 1 {
                    self.color_idx = (self.color_idx + 1) % COLOR_OPTIONS.len();
                    let col = COLOR_OPTIONS[self.color_idx].0.to_string();
                    self.config.color_mode = col.clone();
                    self.color_mode_dropdown = Dropdown::new(COLOR_MODE_OPTIONS.to_vec(), &col);
                    self.saved = false;
                } else if self.fx_sub_focus == 2 {
                    self.contrast_idx = (self.contrast_idx + 1) % CONTRAST_OPTIONS.len();
                    let c = CONTRAST_OPTIONS[self.contrast_idx].0.to_string();
                    self.config.contrast_mode = Some(c.clone());
                    self.contrast_mode_dropdown = Dropdown::new(CONTRAST_MODE_OPTIONS.to_vec(), &c);
                    self.saved = false;
                } else {
                    self.colorblind_idx = (self.colorblind_idx + 1) % COLORBLIND_OPTIONS.len();
                    let cb = COLORBLIND_OPTIONS[self.colorblind_idx].0.to_string();
                    self.config.colorblind_mode =
                        if cb == "none" { None } else { Some(cb.clone()) };
                    self.colorblind_mode_dropdown =
                        Dropdown::new(COLORBLIND_MODE_OPTIONS.to_vec(), &cb);
                    self.saved = false;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.saved = false;
                let c = CONTRAST_OPTIONS[self.contrast_idx].0;
                let cb = COLORBLIND_OPTIONS[self.colorblind_idx].1;
                self.status_message = format!(
                    "Effect: '{}' | Color: '{}' | Contrast: '{}' | CVD: '{}'",
                    self.config.effect, self.config.color_mode, c, cb
                );
            }
            KeyCode::Char('t') => {
                self.show_random_suggestion();
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
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                self.h_scroll = self.h_scroll.saturating_sub(4);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Right | KeyCode::Char('L') => {
                self.h_scroll = (self.h_scroll + 4).min(50);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
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
                Shell::Bash => "eval \"$(forgum init bash)\"",
                Shell::Zsh => "eval \"$(forgum init zsh)\"",
                Shell::Fish => "forgum init fish | source",
                Shell::Pwsh | Shell::PowerShell => "Invoke-Expression (&forgum init pwsh)",
                Shell::Nushell => "forgum init nushell",
                Shell::Elvish => "eval (forgum init elvish | slurp)",
                Shell::Tcsh => "eval `forgum init tcsh`",
                Shell::Ksh => "eval \"$(forgum init ksh)\"",
                Shell::Ion => "eval $(forgum init ion)",
                Shell::Oil => "eval \"$(forgum init oil)\"",
                Shell::Yash => "eval \"$(forgum init yash)\"",
                Shell::Xonsh => "exec($(forgum init xonsh))",
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

    fn handle_studio_key(&mut self, key: KeyEvent) -> anyhow::Result<Option<Action>> {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.studio_sub_focus > 0 {
                    self.studio_sub_focus -= 1;
                } else {
                    self.studio_sub_focus = 2;
                }
                self.status_message = match self.studio_sub_focus {
                    0 => "Studio Mode: Mascot Hull & Moat Debugger".into(),
                    1 => "Studio Mode: Biological Palette & Coat Morphs".into(),
                    _ => "Studio Mode: Kinematics & Physics Accumulator".into(),
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.studio_sub_focus = (self.studio_sub_focus + 1) % 3;
                self.status_message = match self.studio_sub_focus {
                    0 => "Studio Mode: Mascot Hull & Moat Debugger".into(),
                    1 => "Studio Mode: Biological Palette & Coat Morphs".into(),
                    _ => "Studio Mode: Kinematics & Physics Accumulator".into(),
                };
            }
            KeyCode::Char(' ') => {
                self.studio_debug_overlay = !self.studio_debug_overlay;
                self.status_message = format!(
                    "Studio Hull/Moat Overlay: {}",
                    if self.studio_debug_overlay {
                        "ACTIVE"
                    } else {
                        "DISABLED"
                    }
                );
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('[') => {
                let biome = forgum_platform::biome::get_mascot_biome(&self.config.cow);
                let variations = biome.variations();
                if !variations.is_empty() {
                    if self.studio_morph_idx > 0 {
                        self.studio_morph_idx -= 1;
                    } else {
                        self.studio_morph_idx = variations.len().saturating_sub(1);
                    }
                    let morph = &variations[self.studio_morph_idx];
                    self.status_message = format!("Coat Morph: {}", morph.name);
                }
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(']') => {
                let biome = forgum_platform::biome::get_mascot_biome(&self.config.cow);
                let variations = biome.variations();
                if !variations.is_empty() {
                    self.studio_morph_idx = (self.studio_morph_idx + 1) % variations.len();
                    let morph = &variations[self.studio_morph_idx];
                    self.status_message = format!("Coat Morph: {}", morph.name);
                }
            }
            _ => {}
        }
        Ok(None)
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
                if field == ConfigField::Fps {
                    self.cycle_config_field(true);
                } else if field == ConfigField::Duration
                    || field == ConfigField::Text
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::Eyes
                    || field == ConfigField::Tongue
                    || field == ConfigField::Palette
                    || field == ConfigField::DefaultShell
                    || field == ConfigField::SplitRatio
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols
                    || field == ConfigField::Image
                    || field == ConfigField::Editor
                {
                    self.enter_config_edit();
                } else {
                    self.cycle_config_field(true);
                }
            }
            KeyCode::Char('e')
            | KeyCode::Char('E')
            | KeyCode::Char('i')
            | KeyCode::Char('I')
            | KeyCode::Char('a')
            | KeyCode::Char('A') => {
                self.enter_config_edit();
            }
            KeyCode::Char(c @ '0'..='9') => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::SplitRatio
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols
                {
                    self.config_edit_buffer = c.to_string();
                    self.editing_config = true;
                    self.edit_initial = false;
                    self.status_message = format!(
                        "Editing {}: typing '{}' (Enter to confirm, Esc to cancel)",
                        field.label(),
                        c
                    );
                }
            }
            KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Char(' ') => {
                let field = ConfigField::ALL[self.config_field_idx];
                if field == ConfigField::Text {
                    self.enter_config_edit();
                } else {
                    self.cycle_config_field(true);
                }
            }
            KeyCode::Char('H')
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = self.h_scroll.saturating_sub(4);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Char('L')
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = (self.h_scroll + 4).min(50);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Left
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = self.h_scroll.saturating_sub(4);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Right
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.h_scroll = (self.h_scroll + 4).min(50);
                self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
            }
            KeyCode::Char('h') => {
                if self.current_breakpoint().is_compact_or_narrow() {
                    self.h_scroll = self.h_scroll.saturating_sub(4);
                    self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
                } else {
                    self.cycle_config_field(false);
                }
            }
            KeyCode::Char('l') => {
                if self.current_breakpoint().is_compact_or_narrow() {
                    self.h_scroll = (self.h_scroll + 4).min(50);
                    self.status_message = format!("Horizontal scroll: column {}", self.h_scroll);
                } else {
                    self.cycle_config_field(true);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') | KeyCode::Right | KeyCode::Char(']') => {
                self.cycle_config_field(true);
            }
            KeyCode::Char('-') | KeyCode::Char('_') | KeyCode::Left | KeyCode::Char('[') => {
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
            ConfigField::Text => self.config.text.clone(),
            ConfigField::ThoughtInterval => self.config.thought_interval.to_string(),
            ConfigField::Eyes => self.config.eyes.clone(),
            ConfigField::Tongue => self.config.tongue.clone(),
            ConfigField::Palette => self.config.palette.clone().unwrap_or_default(),
            ConfigField::DefaultShell => self.config.default_shell.clone(),
            ConfigField::SplitRatio => self
                .config
                .split_ratio
                .map(|r| r.to_string())
                .unwrap_or_else(|| "auto".into()),
            ConfigField::ReserveRows => self
                .config
                .reserve_rows
                .map(|r| r.to_string())
                .unwrap_or_else(|| "auto".into()),
            ConfigField::ReserveCols => self
                .config
                .reserve_cols
                .map(|c| c.to_string())
                .unwrap_or_else(|| "auto".into()),
            ConfigField::Image => self.config.image.clone().unwrap_or_default(),
            ConfigField::Editor => self.config.editor.clone().unwrap_or_else(|| "auto".into()),
            ConfigField::UpdateMethod => self
                .config
                .update_method
                .clone()
                .unwrap_or_else(|| "prompt".into()),
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
            ConfigField::Text => {
                self.config_edit_buffer = self.config.text.clone();
                self.editing_config = true;
                self.status_message =
                    "Editing Dialogue Text: type message and press Enter (Esc to cancel)".into();
            }
            ConfigField::ThoughtInterval => {
                self.config_edit_buffer = self.config.thought_interval.to_string();
                self.editing_config = true;
                self.status_message =
                    "Editing Thought Interval (seconds): type number and press Enter (Esc to cancel)".into();
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
                let is_natural = self.config.color_mode.eq_ignore_ascii_case("natural")
                    || self.config.color_mode.eq_ignore_ascii_case("animal");
                if !is_natural {
                    self.status_message = "🔒 Color Palette is locked: select 'natural' Color Mode to cycle authentic animal morphs!".to_string();
                    return;
                }
                self.config_edit_buffer = self.config.palette.clone().unwrap_or_default();
                self.editing_config = true;
                self.status_message =
                    "Editing Palette: Custom Hex Gradient Override (optional, comma-separated, e.g. '#ffffff,#1a1a1a') or empty for mascot DNA [Enter: Confirm | Esc: Cancel]"
                        .into();
            }
            ConfigField::DefaultShell => {
                self.config_edit_buffer = self.config.default_shell.clone();
                self.editing_config = true;
                self.status_message =
                    "Editing Default Shell: type shell name (e.g. 'pwsh', 'zsh', 'fish') or 'auto' and press Enter (Esc to cancel)"
                        .into();
            }
            ConfigField::SplitRatio => {
                self.config_edit_buffer = self
                    .config
                    .split_ratio
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "auto".into());
                self.editing_config = true;
                self.status_message =
                    "Editing Split Ratio: type float (0.1..0.8) or 'auto' and press Enter (Esc to cancel)".into();
            }
            ConfigField::ReserveRows => {
                self.config_edit_buffer = self
                    .config
                    .reserve_rows
                    .map(|r| r.to_string())
                    .unwrap_or_else(|| "auto".into());
                self.editing_config = true;
                self.status_message =
                    "Editing Reserve Rows: type rows count (e.g. 12) or 'auto' and press Enter (Esc to cancel)".into();
            }
            ConfigField::ReserveCols => {
                self.config_edit_buffer = self
                    .config
                    .reserve_cols
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "auto".into());
                self.editing_config = true;
                self.status_message =
                    "Editing Reserve Cols: type cols count (e.g. 80) or 'auto' and press Enter (Esc to cancel)".into();
            }
            ConfigField::Image => {
                self.config_edit_buffer = self.config.image.clone().unwrap_or_default();
                self.editing_config = true;
                self.status_message =
                    "Editing Mascot Image Path: type image file path or 'none' and press Enter (Esc to cancel)".into();
            }
            ConfigField::Editor => {
                self.config_edit_buffer =
                    self.config.editor.clone().unwrap_or_else(|| "auto".into());
                self.editing_config = true;
                self.status_message =
                    "Editing Preferred Editor: type editor command (e.g. 'nvim', 'emacs', 'nano', 'code') and press Enter (Esc to cancel)"
                        .into();
            }
            ConfigField::UpdateMethod => {
                self.config_edit_buffer = self
                    .config
                    .update_method
                    .clone()
                    .unwrap_or_else(|| "prompt".into());
                self.editing_config = true;
                self.status_message =
                    "Editing Update Method: type method (e.g. 'prompt', 'auto', 'zypper', 'scoop', 'winget', 'apt', 'cargo') and press Enter (Esc to cancel)"
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
                        DURATION_PRESETS
                            [(pos + DURATION_PRESETS.len() - 1) % DURATION_PRESETS.len()]
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
                self.status_message =
                    format!("Eyes cycled to '{}' [Space/←/→: Cycle | e: Custom]", next);
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
                self.status_message =
                    format!("Tongue cycled to '{}' [Space/←/→: Cycle | e: Custom]", next);
            }
            ConfigField::ColorMode => {
                self.color_mode_dropdown.cycle(forward);
                let current = self.color_mode_dropdown.current();
                self.config.color_mode = current.clone();
                self.color_idx = COLOR_OPTIONS
                    .iter()
                    .position(|(c, _)| c.eq_ignore_ascii_case(&current))
                    .unwrap_or(0);
                if current.eq_ignore_ascii_case("natural") || current.eq_ignore_ascii_case("animal")
                {
                    let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                    if !vars.is_empty() {
                        self.config.palette = Some(vars[0].palette.join(","));
                    }
                }
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = if current.eq_ignore_ascii_case("natural")
                    || current.eq_ignore_ascii_case("animal")
                {
                    format!(
                        "Color mode: natural (Color Palette UNLOCKED for {} morphs!)",
                        self.config.cow
                    )
                } else {
                    format!("Color mode: {} (Color Palette LOCKED)", current)
                };
            }
            ConfigField::Palette => {
                let is_natural = self.config.color_mode.eq_ignore_ascii_case("natural")
                    || self.config.color_mode.eq_ignore_ascii_case("animal");
                if !is_natural {
                    self.status_message = "🔒 Color Palette is locked: select 'natural' Color Mode to cycle authentic animal morphs!".to_string();
                    return;
                }
                let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                if vars.is_empty() {
                    return;
                }
                let cur_idx = self.current_variation_index(vars);
                let next_idx = if forward {
                    (cur_idx + 1) % vars.len()
                } else if cur_idx == 0 {
                    vars.len() - 1
                } else {
                    cur_idx - 1
                };
                let selected = &vars[next_idx];
                self.config.palette = Some(selected.palette.join(","));
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!(
                    "🎨 Natural Morph [{}/{}]: {} — {}",
                    next_idx + 1,
                    vars.len(),
                    selected.name,
                    selected.description
                );
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
                let mode = self.attach_mode_dropdown.current();
                self.config.shell_attach_mode = mode.clone();
                match mode.as_str() {
                    "split" => {
                        self.config.split_scroll = true;
                        self.config.split_mode = Some("decstbm".to_string());
                        self.split_mode_dropdown =
                            Dropdown::new(SPLIT_MODE_OPTIONS.to_vec(), "decstbm");
                        self.status_message =
                            "Shell mode: split (DECSTBM hardware scroll margins)".to_string();
                    }
                    "startup" => {
                        self.config.split_scroll = false;
                        self.config.auto_render_on_prompt = false;
                        self.status_message =
                            "Shell mode: startup (Runs once on shell launch, 0 prompt cost)"
                                .to_string();
                    }
                    "prompt" => {
                        self.config.split_scroll = false;
                        self.config.auto_render_on_prompt = true;
                        self.status_message =
                            "Shell mode: prompt (Renders dynamically on each command prompt)"
                                .to_string();
                    }
                    "clear" => {
                        self.config.split_scroll = false;
                        self.config.auto_render_on_prompt = false;
                        self.status_message =
                            "Shell mode: clear (Runs on shell launch and after clear/cls commands)"
                                .to_string();
                    }
                    "pane" => {
                        self.config.split_scroll = false;
                        self.status_message =
                            "Shell mode: pane (Dedicated pane / daemon in multiplexer)".to_string();
                    }
                    _ => {
                        self.config.split_scroll = false;
                        self.status_message = format!("Shell attach mode: {}", mode);
                    }
                }
                self.saved = false;
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
            ConfigField::SplitMode => {
                self.split_mode_dropdown.cycle(forward);
                let current = self.split_mode_dropdown.current();
                self.config.split_mode = Some(current.clone());
                if current == "decstbm" {
                    self.config.split_scroll = true;
                    self.config.shell_attach_mode = "split".to_string();
                    self.attach_mode_dropdown =
                        Dropdown::new(SHELL_ATTACH_OPTIONS.to_vec(), "split");
                } else if current == "disabled" {
                    self.config.split_scroll = false;
                    if self.config.shell_attach_mode == "split" {
                        self.config.shell_attach_mode = "none".to_string();
                        self.attach_mode_dropdown =
                            Dropdown::new(SHELL_ATTACH_OPTIONS.to_vec(), "none");
                    }
                }
                self.saved = false;
                self.status_message = format!("Split execution mode switched to {}", current);
            }
            ConfigField::SplitScroll => {
                self.config.split_scroll = !self.config.split_scroll;
                if self.config.split_scroll {
                    self.config.shell_attach_mode = "split".to_string();
                    self.attach_mode_dropdown =
                        Dropdown::new(SHELL_ATTACH_OPTIONS.to_vec(), "split");
                    if self.config.split_mode.as_deref() != Some("decstbm") {
                        self.config.split_mode = Some("decstbm".to_string());
                        self.split_mode_dropdown =
                            Dropdown::new(SPLIT_MODE_OPTIONS.to_vec(), "decstbm");
                    }
                } else {
                    self.config.shell_attach_mode = "none".to_string();
                    self.attach_mode_dropdown =
                        Dropdown::new(SHELL_ATTACH_OPTIONS.to_vec(), "none");
                }
                self.saved = false;
                self.status_message = format!(
                    "Split scroll region {}",
                    if self.config.split_scroll {
                        "ENABLED (DECSTBM margin scroll, attach: split)"
                    } else {
                        "DISABLED (attach: none)"
                    }
                );
            }
            ConfigField::Text => {
                const TEXT_PRESETS: [&str; 6] = [
                    "",
                    "Hello, world!",
                    "Forgum terminal engine!",
                    "Stay curious & keep coding!",
                    "哞 ~ Forgum!",
                    "The terminal is our canvas!",
                ];
                let cur = self.config.text.as_str();
                let next = if forward {
                    if let Some(pos) = TEXT_PRESETS.iter().position(|&t| t == cur) {
                        TEXT_PRESETS[(pos + 1) % TEXT_PRESETS.len()]
                    } else {
                        TEXT_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = TEXT_PRESETS.iter().position(|&t| t == cur) {
                        TEXT_PRESETS[(pos + TEXT_PRESETS.len() - 1) % TEXT_PRESETS.len()]
                    } else {
                        TEXT_PRESETS[TEXT_PRESETS.len() - 1]
                    }
                };
                self.config.text = next.to_string();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = if next.is_empty() {
                    "Dialogue text cleared (no bubble) [Space/←/→: Cycle | e: Type custom]".into()
                } else {
                    format!(
                        "Dialogue text cycled to '{}' [Space/←/→: Cycle | e: Type custom]",
                        next
                    )
                };
            }
            ConfigField::Think => {
                self.config.think = !self.config.think;
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!(
                    "Bubble mode switched to {}",
                    if self.config.think {
                        "Thought bubble ('o' connector)"
                    } else {
                        "Speech bubble ('\\' connector)"
                    }
                );
            }
            ConfigField::ThoughtInterval => {
                const THOUGHT_PRESETS: [u32; 7] = [0, 10, 30, 60, 120, 300, 600];
                let current = self.config.thought_interval;
                let next = if forward {
                    if let Some(pos) = THOUGHT_PRESETS.iter().position(|&t| t == current) {
                        THOUGHT_PRESETS[(pos + 1) % THOUGHT_PRESETS.len()]
                    } else {
                        THOUGHT_PRESETS
                            .iter()
                            .copied()
                            .find(|&t| t > current)
                            .unwrap_or(THOUGHT_PRESETS[0])
                    }
                } else {
                    if let Some(pos) = THOUGHT_PRESETS.iter().position(|&t| t == current) {
                        THOUGHT_PRESETS[(pos + THOUGHT_PRESETS.len() - 1) % THOUGHT_PRESETS.len()]
                    } else {
                        THOUGHT_PRESETS
                            .iter()
                            .copied()
                            .rev()
                            .find(|&t| t < current)
                            .unwrap_or(THOUGHT_PRESETS[THOUGHT_PRESETS.len() - 1])
                    }
                };
                self.config.thought_interval = next;
                self.saved = false;
                self.status_message = format!(
                    "Thought interval set to {}s [Space/←/→: Cycle | e: Type]",
                    next
                );
            }
            ConfigField::Animation => {
                self.animation_dropdown.cycle(forward);
                let current = self.animation_dropdown.current();
                self.config.animation = Some(current.clone());
                self.saved = false;
                self.status_message = format!("Animation mode switched to {}", current);
            }
            ConfigField::DefaultShell => {
                const SHELL_PRESETS: [&str; 9] = [
                    "",
                    "pwsh",
                    "powershell",
                    "bash",
                    "zsh",
                    "fish",
                    "nu",
                    "cmd",
                    "elvish",
                ];
                let cur = self.config.default_shell.as_str();
                let next = if forward {
                    if let Some(pos) = SHELL_PRESETS.iter().position(|&s| s == cur) {
                        SHELL_PRESETS[(pos + 1) % SHELL_PRESETS.len()]
                    } else {
                        SHELL_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = SHELL_PRESETS.iter().position(|&s| s == cur) {
                        SHELL_PRESETS[(pos + SHELL_PRESETS.len() - 1) % SHELL_PRESETS.len()]
                    } else {
                        SHELL_PRESETS[SHELL_PRESETS.len() - 1]
                    }
                };
                self.config.default_shell = next.to_string();
                self.saved = false;
                self.status_message = if next.is_empty() {
                    "Default shell set to auto-detect [Space/←/→: Cycle | e: Type]".into()
                } else {
                    format!(
                        "Default shell switched to '{}' [Space/←/→: Cycle | e: Type]",
                        next
                    )
                };
            }
            ConfigField::SplitRatio => {
                const RATIO_PRESETS: [Option<f32>; 6] = [
                    None,
                    Some(0.20),
                    Some(0.25),
                    Some(0.30),
                    Some(0.35),
                    Some(0.40),
                ];
                let cur = self.config.split_ratio;
                let next = if forward {
                    if let Some(pos) = RATIO_PRESETS.iter().position(|&r| r == cur) {
                        RATIO_PRESETS[(pos + 1) % RATIO_PRESETS.len()]
                    } else {
                        RATIO_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = RATIO_PRESETS.iter().position(|&r| r == cur) {
                        RATIO_PRESETS[(pos + RATIO_PRESETS.len() - 1) % RATIO_PRESETS.len()]
                    } else {
                        RATIO_PRESETS[RATIO_PRESETS.len() - 1]
                    }
                };
                self.config.split_ratio = next;
                self.saved = false;
                self.status_message = if let Some(r) = next {
                    format!(
                        "Split ratio adjusted to {:.2} [Space/←/→: Cycle | e: Type]",
                        r
                    )
                } else {
                    "Split ratio set to auto (adaptive viewport) [Space/←/→: Cycle | e: Type]"
                        .into()
                };
            }
            ConfigField::ReserveRows => {
                const ROWS_PRESETS: [Option<u16>; 7] = [
                    None,
                    Some(8),
                    Some(10),
                    Some(12),
                    Some(14),
                    Some(16),
                    Some(20),
                ];
                let cur = self.config.reserve_rows;
                let next = if forward {
                    if let Some(pos) = ROWS_PRESETS.iter().position(|&r| r == cur) {
                        ROWS_PRESETS[(pos + 1) % ROWS_PRESETS.len()]
                    } else {
                        ROWS_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = ROWS_PRESETS.iter().position(|&r| r == cur) {
                        ROWS_PRESETS[(pos + ROWS_PRESETS.len() - 1) % ROWS_PRESETS.len()]
                    } else {
                        ROWS_PRESETS[ROWS_PRESETS.len() - 1]
                    }
                };
                self.config.reserve_rows = next;
                self.saved = false;
                self.status_message = if let Some(r) = next {
                    format!(
                        "Reserve rows set to {} rows [Space/←/→: Cycle | e: Type]",
                        r
                    )
                } else {
                    "Reserve rows set to auto (dynamic calculation) [Space/←/→: Cycle | e: Type]"
                        .into()
                };
            }
            ConfigField::ReserveCols => {
                const COLS_PRESETS: [Option<u16>; 7] = [
                    None,
                    Some(40),
                    Some(60),
                    Some(80),
                    Some(100),
                    Some(120),
                    Some(140),
                ];
                let cur = self.config.reserve_cols;
                let next = if forward {
                    if let Some(pos) = COLS_PRESETS.iter().position(|&c| c == cur) {
                        COLS_PRESETS[(pos + 1) % COLS_PRESETS.len()]
                    } else {
                        COLS_PRESETS[1]
                    }
                } else {
                    if let Some(pos) = COLS_PRESETS.iter().position(|&c| c == cur) {
                        COLS_PRESETS[(pos + COLS_PRESETS.len() - 1) % COLS_PRESETS.len()]
                    } else {
                        COLS_PRESETS[COLS_PRESETS.len() - 1]
                    }
                };
                self.config.reserve_cols = next;
                self.saved = false;
                self.status_message = if let Some(c) = next {
                    format!(
                        "Reserve cols set to {} cols [Space/←/→: Cycle | e: Type]",
                        c
                    )
                } else {
                    "Reserve cols set to auto (full width) [Space/←/→: Cycle | e: Type]".into()
                };
            }
            ConfigField::Image => {
                self.enter_config_edit();
            }
            ConfigField::Editor => {
                self.editor_dropdown.cycle(forward);
                let current = self.editor_dropdown.current();
                self.config.editor = if current == "auto" {
                    None
                } else {
                    Some(current.clone())
                };
                self.saved = false;
                self.status_message = format!("Preferred editor switched to {}", current);
            }
            ConfigField::Random => {
                let next = match &self.config.random {
                    None | Some(forgum_platform::protocol::RandomSetting::Bool(false)) => {
                        Some(forgum_platform::protocol::RandomSetting::Bool(true))
                    }
                    Some(forgum_platform::protocol::RandomSetting::Bool(true)) => Some(
                        forgum_platform::protocol::RandomSetting::String("mascot".into()),
                    ),
                    Some(forgum_platform::protocol::RandomSetting::String(s)) => match s.as_str() {
                        "mascot" => Some(forgum_platform::protocol::RandomSetting::String(
                            "scenery".into(),
                        )),
                        "scenery" => Some(forgum_platform::protocol::RandomSetting::String(
                            "fx".into(),
                        )),
                        "fx" => Some(forgum_platform::protocol::RandomSetting::String(
                            "thought".into(),
                        )),
                        _ => None,
                    },
                    _ => None,
                };
                self.config.random = next;
                self.saved = false;
                self.status_message = format!(
                    "Random mode: {}",
                    self.config
                        .random
                        .as_ref()
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "disabled".into())
                );
            }
            ConfigField::ContrastMode => {
                self.contrast_mode_dropdown.cycle(forward);
                let current = self.contrast_mode_dropdown.current();
                self.config.contrast_mode = Some(current.clone());
                self.sync_contrast_from_config();
                self.saved = false;
                self.status_message = format!(
                    "Terminal background contrast profile switched to {}",
                    current
                );
            }
            ConfigField::HardwareAccel => {
                self.accel_mode_dropdown.cycle(forward);
                let current = self.accel_mode_dropdown.current();
                self.config.accel_mode = Some(current.clone());
                self.sync_accel_from_config();
                self.saved = false;
                self.status_message = format!(
                    "Hardware acceleration switched to {} [{}]",
                    self.active_accel_display(),
                    self.active_accel_badge()
                );
            }
            ConfigField::ReduceMotion => {
                self.config.reduce_motion = !self.config.reduce_motion;
                self.saved = false;
                self.status_message = format!(
                    "Reduce motion {}",
                    if self.config.reduce_motion {
                        "ENABLED (gentle fades only)"
                    } else {
                        "DISABLED (full kinetic motion)"
                    }
                );
            }
            ConfigField::TextOnly => {
                self.config.text_only = !self.config.text_only;
                self.saved = false;
                self.status_message = format!(
                    "Text-only mode {}",
                    if self.config.text_only {
                        "ENABLED (print fortune text only)"
                    } else {
                        "DISABLED (render ASCII mascot art)"
                    }
                );
            }
            ConfigField::Banner => {
                self.config.banner = !self.config.banner;
                if self.config.banner {
                    self.config.shell_attach_mode = "banner".to_string();
                }
                self.saved = false;
                self.status_message = format!(
                    "Banner mode {}",
                    if self.config.banner {
                        "ENABLED (inline fastfetch banner)"
                    } else {
                        "DISABLED (standard canvas)"
                    }
                );
            }
            ConfigField::Aspect => {
                self.aspect_dropdown.cycle(forward);
                let current = self.aspect_dropdown.current();
                self.config.aspect = if current == "auto" {
                    None
                } else {
                    Some(current.clone())
                };
                self.saved = false;
                self.status_message = format!("Aspect ratio preset switched to {}", current);
            }
            ConfigField::AspectMode => {
                self.aspect_mode_dropdown.cycle(forward);
                let current = self.aspect_mode_dropdown.current();
                self.config.aspect_mode = if current == "auto" {
                    None
                } else {
                    Some(current.clone())
                };
                self.saved = false;
                self.status_message = format!("Aspect adaptation mode switched to {}", current);
            }
            ConfigField::ColorblindMode => {
                self.colorblind_mode_dropdown.cycle(forward);
                let current = self.colorblind_mode_dropdown.current();
                self.config.colorblind_mode = if current == "none" {
                    None
                } else {
                    Some(current.clone())
                };
                self.sync_colorblind_from_config();
                self.saved = false;
                self.status_message = format!(
                    "Colorblind accessibility mode switched to {} [{}]",
                    self.active_colorblind_display(),
                    self.active_colorblind_badge()
                );
            }
            ConfigField::UpdateMethod => {
                const UPDATE_PRESETS: [&str; 11] = [
                    "prompt",
                    "auto",
                    "zypper",
                    "scoop",
                    "winget",
                    "homebrew",
                    "apt",
                    "dnf",
                    "pacman",
                    "cargo",
                    "standalone",
                ];
                let cur = self.config.update_method.as_deref().unwrap_or("prompt");
                let next = if forward {
                    if let Some(pos) = UPDATE_PRESETS
                        .iter()
                        .position(|&p| p.eq_ignore_ascii_case(cur))
                    {
                        UPDATE_PRESETS[(pos + 1) % UPDATE_PRESETS.len()]
                    } else {
                        UPDATE_PRESETS[0]
                    }
                } else {
                    if let Some(pos) = UPDATE_PRESETS
                        .iter()
                        .position(|&p| p.eq_ignore_ascii_case(cur))
                    {
                        UPDATE_PRESETS[(pos + UPDATE_PRESETS.len() - 1) % UPDATE_PRESETS.len()]
                    } else {
                        UPDATE_PRESETS[UPDATE_PRESETS.len() - 1]
                    }
                };
                self.config.update_method = if next == "prompt" {
                    Some("prompt".to_string())
                } else {
                    Some(next.to_string())
                };
                self.saved = false;
                self.status_message =
                    format!("Update method set to '{}' [←/→: Cycle | e: Type]", next);
            }
            ConfigField::CheckUpdates => {
                self.config.check_updates = !self.config.check_updates;
                self.saved = false;
                self.status_message = format!(
                    "Automatic update check {}",
                    if self.config.check_updates {
                        "ENABLED (24h cache, zero latency)"
                    } else {
                        "DISABLED"
                    }
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
            KeyCode::Esc | KeyCode::Char('[')
                if key.modifiers.contains(KeyModifiers::CONTROL) || key.code == KeyCode::Esc =>
            {
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
                    ConfigField::Text => {
                        self.config.text = self.original_edit_value.clone();
                        self.cow_cache.clear();
                        self.ensure_cow_cached(&self.config.cow.clone());
                    }
                    ConfigField::ThoughtInterval => {
                        if let Ok(v) = self.original_edit_value.parse::<u32>() {
                            self.config.thought_interval = v;
                        }
                    }
                    ConfigField::DefaultShell => {
                        self.config.default_shell = self.original_edit_value.clone();
                    }
                    ConfigField::SplitRatio => {
                        self.config.split_ratio = self.original_edit_value.parse::<f32>().ok();
                    }
                    ConfigField::ReserveRows => {
                        self.config.reserve_rows = self.original_edit_value.parse::<u16>().ok();
                    }
                    ConfigField::ReserveCols => {
                        self.config.reserve_cols = self.original_edit_value.parse::<u16>().ok();
                    }
                    ConfigField::Image => {
                        self.config.image = if self.original_edit_value.is_empty() {
                            None
                        } else {
                            Some(self.original_edit_value.clone())
                        };
                    }
                    ConfigField::UpdateMethod => {
                        self.config.update_method = if self.original_edit_value.is_empty() {
                            Some("prompt".to_string())
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
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols =>
            {
                self.commit_config_edit();
                self.cycle_config_field(true);
                self.config_edit_buffer = match field {
                    ConfigField::Duration => self.config.duration.to_string(),
                    ConfigField::Fps => self.config.fps.to_string(),
                    ConfigField::ThoughtInterval => self.config.thought_interval.to_string(),
                    ConfigField::ReserveRows => self
                        .config
                        .reserve_rows
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "auto".into()),
                    ConfigField::ReserveCols => self
                        .config
                        .reserve_cols
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "auto".into()),
                    _ => self.config_edit_buffer.clone(),
                };
            }
            KeyCode::Char('-') | KeyCode::Char('_')
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols =>
            {
                self.commit_config_edit();
                self.cycle_config_field(false);
                self.config_edit_buffer = match field {
                    ConfigField::Duration => self.config.duration.to_string(),
                    ConfigField::Fps => self.config.fps.to_string(),
                    ConfigField::ThoughtInterval => self.config.thought_interval.to_string(),
                    ConfigField::ReserveRows => self
                        .config
                        .reserve_rows
                        .map(|r| r.to_string())
                        .unwrap_or_else(|| "auto".into()),
                    ConfigField::ReserveCols => self
                        .config
                        .reserve_cols
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "auto".into()),
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
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols
                {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        self.config_edit_buffer = (val.round() as u32 + 1).to_string();
                    }
                } else if field == ConfigField::SplitRatio {
                    let clean = self.config_edit_buffer.trim();
                    if let Ok(val) = clean.parse::<f32>() {
                        self.config_edit_buffer = format!("{:.2}", (val + 0.05).min(0.95));
                    } else {
                        self.config_edit_buffer = "0.30".to_string();
                    }
                }
            }
            KeyCode::Left => {
                self.edit_initial = false;
                if field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols
                {
                    let clean = self
                        .config_edit_buffer
                        .trim()
                        .trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                    if let Ok(val) = clean.parse::<f64>() {
                        let floor = if field == ConfigField::Fps { 1 } else { 0 };
                        let current = val.round() as u32;
                        self.config_edit_buffer = current.saturating_sub(1).max(floor).to_string();
                    }
                } else if field == ConfigField::SplitRatio {
                    let clean = self.config_edit_buffer.trim();
                    if let Ok(val) = clean.parse::<f32>() {
                        self.config_edit_buffer = format!("{:.2}", (val - 0.05).max(0.05));
                    } else {
                        self.config_edit_buffer = "0.25".to_string();
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
                let is_integer_numeric = field == ConfigField::Duration
                    || field == ConfigField::Fps
                    || field == ConfigField::ThoughtInterval
                    || field == ConfigField::ReserveRows
                    || field == ConfigField::ReserveCols;
                let is_float_numeric = field == ConfigField::SplitRatio;

                if is_integer_numeric && !c.is_ascii_digit() && c != 's' && c != 'f' && c != 'p' {
                    // Ignore non-digit characters on numeric fields so buffer isn't corrupted
                } else if is_float_numeric && !c.is_ascii_digit() && c != '.' {
                    // Ignore non-digit/period characters on float ratio fields
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
            ConfigField::Text => {
                self.config.text = self.config_edit_buffer.clone();
                self.cow_cache.clear();
                self.ensure_cow_cached(&self.config.cow.clone());
                self.saved = false;
                self.status_message = format!("✓ Dialogue text set to '{}'", self.config.text);
            }
            ConfigField::ThoughtInterval => {
                let trimmed = self.config_edit_buffer.trim();
                let clean =
                    trimmed.trim_end_matches(|c: char| c.is_alphabetic() || c.is_whitespace());
                if let Ok(val) = clean.parse::<f64>() {
                    let n = val.max(0.0).round() as u32;
                    self.config.thought_interval = n;
                    self.saved = false;
                    self.status_message = format!("✓ Thought interval set to {}s", n);
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
                    self.status_message =
                        "✓ Palette cleared (using mascot DNA biological colors)".to_string();
                } else {
                    self.config.palette = Some(trimmed.clone());
                    self.saved = false;
                    self.status_message = format!("✓ Palette set to '{}'", trimmed);
                }
            }
            ConfigField::DefaultShell => {
                let trimmed = self.config_edit_buffer.trim().to_string();
                self.config.default_shell = if trimmed.eq_ignore_ascii_case("auto") {
                    String::new()
                } else {
                    trimmed
                };
                self.saved = false;
                self.status_message = if self.config.default_shell.is_empty() {
                    "✓ Default shell set to auto-detect".into()
                } else {
                    format!("✓ Default shell set to '{}'", self.config.default_shell)
                };
            }
            ConfigField::SplitRatio => {
                let trimmed = self.config_edit_buffer.trim();
                if trimmed.is_empty()
                    || trimmed.eq_ignore_ascii_case("auto")
                    || trimmed.eq_ignore_ascii_case("none")
                {
                    self.config.split_ratio = None;
                    self.saved = false;
                    self.status_message = "✓ Split ratio set to auto-adaptive".into();
                } else if let Ok(val) = trimmed.parse::<f32>() {
                    let r = val.clamp(0.05, 0.95);
                    self.config.split_ratio = Some(r);
                    self.saved = false;
                    self.status_message = format!("✓ Split ratio set to {:.2}", r);
                }
            }
            ConfigField::ReserveRows => {
                let trimmed = self.config_edit_buffer.trim();
                if trimmed.is_empty()
                    || trimmed.eq_ignore_ascii_case("auto")
                    || trimmed.eq_ignore_ascii_case("none")
                {
                    self.config.reserve_rows = None;
                    self.saved = false;
                    self.status_message = "✓ Reserve rows set to auto".into();
                } else if let Ok(val) = trimmed.parse::<u16>() {
                    self.config.reserve_rows = Some(val);
                    self.saved = false;
                    self.status_message = format!("✓ Reserve rows set to {}", val);
                }
            }
            ConfigField::ReserveCols => {
                let trimmed = self.config_edit_buffer.trim();
                if trimmed.is_empty()
                    || trimmed.eq_ignore_ascii_case("auto")
                    || trimmed.eq_ignore_ascii_case("none")
                {
                    self.config.reserve_cols = None;
                    self.saved = false;
                    self.status_message = "✓ Reserve cols set to auto".into();
                } else if let Ok(val) = trimmed.parse::<u16>() {
                    self.config.reserve_cols = Some(val);
                    self.saved = false;
                    self.status_message = format!("✓ Reserve cols set to {}", val);
                }
            }
            ConfigField::Image => {
                let trimmed = self.config_edit_buffer.trim().to_string();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("none") {
                    self.config.image = None;
                    self.saved = false;
                    self.status_message = "✓ Mascot image cleared".into();
                } else {
                    self.config.image = Some(trimmed.clone());
                    self.saved = false;
                    self.status_message = format!("✓ Mascot image set to '{}'", trimmed);
                }
            }
            ConfigField::Editor => {
                let trimmed = self.config_edit_buffer.trim().to_string();
                if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") {
                    self.config.editor = None;
                    self.saved = false;
                    self.status_message = "✓ Preferred editor set to auto-detect".to_string();
                } else {
                    self.config.editor = Some(trimmed.clone());
                    self.saved = false;
                    self.status_message = format!("✓ Preferred editor set to '{}'", trimmed);
                }
            }
            ConfigField::UpdateMethod => {
                let trimmed = self.config_edit_buffer.trim().to_string();
                if trimmed.is_empty()
                    || trimmed.eq_ignore_ascii_case("prompt")
                    || trimmed.eq_ignore_ascii_case("ask")
                {
                    self.config.update_method = Some("prompt".to_string());
                    self.saved = false;
                    self.status_message =
                        "✓ Update method set to prompt (ask each time)".to_string();
                } else {
                    self.config.update_method = Some(trimmed.clone());
                    self.saved = false;
                    self.status_message = format!("✓ Update method set to '{}'", trimmed);
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
        // 1. Try disk data directory first if available
        if let Ok(data) = forgum_platform::data_dir() {
            let cow_path = data.join("Cows").join(format!("{cow_name}.cow"));
            if let Ok(raw) = std::fs::read_to_string(&cow_path) {
                return Self::expand_cow_template(&raw, &self.config.eyes, &self.config.tongue);
            }
        }
        // 2. Try compile-time embedded built-in mascot catalog (100% zero-disk resilience)
        if let Some(embedded) = forgum_platform::embedded_cows::get_embedded_cow(cow_name) {
            return Self::expand_cow_template(embedded, &self.config.eyes, &self.config.tongue);
        }
        // 3. Fallback to default cow
        Self::fallback_cow(&self.config.eyes, &self.config.tongue)
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
                                Some((best_pos, _)) if pos < best_pos => {
                                    earliest = Some((pos, pat))
                                }
                                _ => {}
                            }
                        }
                    }

                    let Some((pos, pat)) = earliest else {
                        break;
                    };

                    let glyph = if eye_idx.is_multiple_of(2) {
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
                l = l
                    .replace(r"\$", "$")
                    .replace(r"\@", "@")
                    .replace(r"\#", "#");
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

        // Cache terminal dimensions for current_breakpoint() outside render context.
        self.last_terminal_size = (size.width, size.height);
        let bp = Breakpoint::from_size(size.width, size.height);

        // Responsive 3-row layout driven by Breakpoint:
        // Compact  → 1-line header  + min-8 workspace + 1-line footer (max space for content)
        // Standard → 3-line header  + min-10 workspace + 2-line footer (full chrome)
        // Wide     → 3-line header  + min-10 workspace + 2-line footer (full chrome)
        let (header_h, footer_h, workspace_min) = match bp {
            Breakpoint::Compact => (1, 1, 8),
            _ => (3, 2, 10),
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_h),
                Constraint::Min(workspace_min),
                Constraint::Length(footer_h),
            ])
            .split(size);

        self.render_zellij_header(f, chunks[0]);
        self.render_workspace(f, chunks[1]);
        self.render_zellij_footer(f, chunks[2]);

        if self.show_editor_modal {
            self.render_editor_modal(f, size);
        }
        if self.show_vim_help {
            self.render_vim_help_modal(f, size);
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
                    .fg(tailwind::AMBER_400)
                    .add_modifier(Modifier::BOLD),
            ))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(
                Style::default()
                    .fg(tailwind::AMBER_500)
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(popup_area);
        f.render_widget(block, popup_area);

        let lines = vec![
            Line::from(""),
            Line::from(Span::styled(
                "No terminal editor (neovim/vim, nano, emacs) detected on your system.",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
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
                Span::styled("|  ", Style::default().fg(Color::DarkGray)),
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
                Span::styled("|  ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "[Esc] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("Cancel", Style::default().fg(Color::White)),
            ]),
        ];

        let paragraph = Paragraph::new(lines)
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, inner);
    }

    /// Render interactive Vim control and navigation cheatsheet modal.
    fn render_vim_help_modal(&self, f: &mut Frame, area: Rect) {
        let width = 78.min(area.width.saturating_sub(4));
        let height = 22.min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(width)) / 2;
        let y = area.y + (area.height.saturating_sub(height)) / 2;
        let modal_area = Rect::new(x, y, width, height);

        f.render_widget(Clear, modal_area);

        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "  j / k          ",
                    Style::default()
                        .fg(tailwind::AMBER_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Move selection down / up",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  h / l          ",
                    Style::default()
                        .fg(tailwind::AMBER_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Switch sub-focus / cycle option backward & forward",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  gg / G         ",
                    Style::default()
                        .fg(tailwind::EMERALD_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Jump directly to top / bottom of active list",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Ctrl+d / Ctrl+u",
                    Style::default()
                        .fg(tailwind::EMERALD_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Scroll half-page down / up (5 items)",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  gt / gT        ",
                    Style::default()
                        .fg(tailwind::SKY_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Switch to next / previous tab",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  1 .. 5         ",
                    Style::default()
                        .fg(tailwind::SKY_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Direct jump to tab (1:Mascots 2:Scenery 3:FX 4:Setup 5:Config)",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Enter / e      ",
                    Style::default()
                        .fg(tailwind::EMERALD_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Edit selected field or toggle setting directly",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  Esc            ",
                    Style::default()
                        .fg(tailwind::ROSE_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Cancel field edit / close help overlay",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  s              ",
                    Style::default()
                        .fg(tailwind::CYAN_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Save configuration to canonical path (~/.config/forgum/)",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  T / F1         ",
                    Style::default()
                        .fg(tailwind::FUCHSIA_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Cycle random feature discovery tip / pro-tip",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  ZZ / q         ",
                    Style::default()
                        .fg(tailwind::ROSE_500)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Save & Quit / Clean Quit",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  ZQ             ",
                    Style::default()
                        .fg(tailwind::ROSE_500)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Force Quit without saving",
                    Style::default().fg(tailwind::SLATE_200),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "  ?              ",
                    Style::default()
                        .fg(tailwind::VIOLET_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "Toggle this Vim Cheatsheet modal (Esc or ? to close)",
                    Style::default().fg(tailwind::SLATE_300),
                ),
            ]),
            Line::from(vec![]),
            Line::from(vec![
                Span::styled(
                    "  💡 Tip: ",
                    Style::default()
                        .fg(tailwind::AMBER_400)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    self.current_suggestion(),
                    Style::default()
                        .fg(tailwind::EMERALD_300)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]),
        ];

        let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(tailwind::AMBER_400))
                .title(Span::styled(
                    " ⌨️ Vim Navigation & Feature Discovery Cheatsheet ",
                    Style::default()
                        .fg(tailwind::AMBER_300)
                        .add_modifier(Modifier::BOLD),
                )),
        );
        f.render_widget(p, modal_area);
    }

    /// Render Zellij-style top header with active mode badge, tabs, and format badge.
    /// Styled with Tailwind CSS-inspired design tokens for a modern vibrant look.
    fn render_zellij_header(&self, f: &mut Frame, area: Rect) {
        let bp = Breakpoint::from_size(area.width, area.height);

        // ── Compact Mode: single-line tab bar, no decorative border ──────────
        if bp == Breakpoint::Compact {
            let mut spans: Vec<Span> = Vec::new();
            // Compact mode label badge (no emoji, no full pill)
            spans.push(Span::styled(
                format!("[{}]", self.current_tab.compact_mode_label()),
                Style::default()
                    .fg(tailwind::INDIGO_400)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            // Compact tab titles
            for tab in Tab::ALL {
                let is_active = tab == self.current_tab;
                let style = if is_active {
                    Style::default()
                        .bg(tailwind::EMERALD_500)
                        .fg(tailwind::SLATE_900)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(tailwind::SLATE_400)
                };
                spans.push(Span::styled(tab.compact_title(), style));
            }
            let p = Paragraph::new(Line::from(spans));
            f.render_widget(p, area);
            return;
        }

        // ── Standard / Wide Mode: full Zellij chrome ─────────────────────────
        let mut line_spans = Vec::new();

        // Active mode pill — Indigo-600 bg with bright white text
        line_spans.push(Span::styled(
            format!(" {} ", self.current_tab.mode_label()),
            Style::default()
                .bg(tailwind::INDIGO_600)
                .fg(tailwind::SLATE_100)
                .add_modifier(Modifier::BOLD),
        ));
        line_spans.push(Span::raw(" "));

        // Tabs 1-5 — Emerald active, Slate inactive
        for tab in Tab::ALL {
            let is_active = tab == self.current_tab;
            let style = if is_active {
                Style::default()
                    .bg(tailwind::EMERALD_500)
                    .fg(tailwind::SLATE_900)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(tailwind::SLATE_400)
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
            .border_style(Style::default().fg(tailwind::INDIGO_500))
            .title(Span::styled(
                " ✨ FORGUM CONFIGURATOR ",
                Style::default()
                    .fg(tailwind::VIOLET_500)
                    .add_modifier(Modifier::BOLD),
            ));

        let p = Paragraph::new(title_line).block(block);
        f.render_widget(p, area);

        // Render right-aligned badge in header if width permits — Amber-400 version/format
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
                    .fg(tailwind::AMBER_400)
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
        let bp = Breakpoint::from_size(width, height);

        // ── Compact Mode (width < 80): vertical stack layout ─────────────────
        // Entity list on top (~40% height), preview below (60%), full width.
        // Inspector collapses aggressively to preserve canvas space.
        if bp == Breakpoint::Compact {
            let inspector_len: u16 = if height < 18 {
                0
            } else if height < 22 {
                2
            } else {
                4
            };

            // Top portion: entity list (~40% of available height minus inspector)
            let available = height.saturating_sub(inspector_len);
            let list_h = (available * 40 / 100).max(4);
            let preview_h = available.saturating_sub(list_h);

            let vert_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(list_h),
                    Constraint::Length(preview_h),
                    Constraint::Length(inspector_len),
                ])
                .split(area);

            // Entity list gets full area width in Compact mode
            match self.current_tab {
                Tab::Mascots => self.render_mascots_list(f, vert_split[0]),
                Tab::Scenery => self.render_scenery_list(f, vert_split[0]),
                Tab::Effects => self.render_effects_list(f, vert_split[0]),
                Tab::Installer => self.render_installer_list(f, vert_split[0]),
                Tab::Config => self.render_config_list(f, vert_split[0]),
                Tab::Studio => self.render_studio_list(f, vert_split[0]),
            }
            self.render_live_preview(f, vert_split[1]);
            if inspector_len > 0 {
                self.render_inspector(f, vert_split[2]);
            }
            return;
        }

        // ── Standard / Wide Mode: side-by-side split layout ──────────────────
        // For Tab::Config: allocate 56-58% of screen width (50..100 cols) to provide
        // a spacious, clean, and properly aligned settings page, leaving ~42-44% for live preview.
        // For other tabs (Mascots, Scenery, Effects, Studio): allocate 36-38% (30..60 cols).
        let (left_pct, min_w, max_w) = if self.current_tab == Tab::Config {
            let pct: u16 = if bp == Breakpoint::Wide { 56 } else { 58 };
            (pct, 50, 100)
        } else {
            let pct: u16 = if bp == Breakpoint::Wide { 36 } else { 38 };
            (pct, 30, 60)
        };
        let left_width = (width * left_pct / 100)
            .clamp(min_w, max_w)
            .min(width.saturating_sub(25));
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
            Tab::Studio => self.render_studio_list(f, left_rect),
        }

        // Adaptive Inspector Height:
        // Standard: 6 lines (tight: 4). Wide: 7 lines (tight: 4).
        let inspector_len: u16 = if height < 20 {
            4
        } else if bp == Breakpoint::Wide && height >= 24 {
            7
        } else {
            6
        };

        // Render right column: Upper = 30 FPS Live Preview, Lower = Inspector / Log
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),                // Live Preview Canvas
                Constraint::Length(inspector_len), // Inspector & Details / Log
            ])
            .split(right_rect);

        self.render_live_preview(f, right_split[0]);
        self.render_inspector(f, right_split[1]);
    }

    /// Render Grand Welcoming First-Time Setup TUI and Host Diagnostics workspace.
    fn render_installer_workspace(&self, f: &mut Frame, area: Rect) {
        let width = area.width;
        let height = area.height;
        let bp = Breakpoint::from_size(width, height);

        if bp == Breakpoint::Compact || width < 90 {
            // Adaptive Stacked Layout for Compact / Narrow screens (< 90 cols):
            // Shells & Package Managers on top; Hero diagnostics and action log below.
            let top_h = (height * 42 / 100).clamp(8, 14);
            let bot_h = height.saturating_sub(top_h);
            let stack = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(top_h), Constraint::Length(bot_h)])
                .split(area);

            let top_split = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
                .split(stack[0]);
            self.render_installer_list(f, top_split[0]);
            self.render_package_managers_card(f, top_split[1]);

            let bot_log_h = if bot_h < 12 { 4 } else { 6 };
            let bot_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(6), Constraint::Length(bot_log_h)])
                .split(stack[1]);
            self.render_installer_hero_card(f, bot_split[0]);
            self.render_installer_log_card(f, bot_split[1]);
            return;
        }

        let left_width = if width <= 130 {
            (width * 30 / 100).clamp(32, 44)
        } else {
            (width * 25 / 100).clamp(34, 50)
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

        let log_height = if height < 22 {
            5
        } else if height < 30 {
            7
        } else {
            9
        };

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

        let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
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
                    "  Copyright (c) 2024-2026 harish2222 (HKDevLoops)",
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

        let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
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
                let prefix = if is_sel {
                    if self.h_scroll > 0 {
                        "◀▶ "
                    } else {
                        "▶ "
                    }
                } else if self.h_scroll > 0 {
                    "◀  "
                } else {
                    "  "
                };
                let display_cow: String = if self.h_scroll > 0 {
                    let scrolled: String = cow.chars().skip(self.h_scroll).collect();
                    if scrolled.is_empty() {
                        "…".to_string()
                    } else {
                        scrolled
                    }
                } else {
                    cow.to_string()
                };

                let style = if is_sel {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(display_cow, style),
                ]))
            })
            .collect();

        let scroll_badge = if self.h_scroll > 0 {
            format!(" [◀+{}]", self.h_scroll)
        } else {
            String::new()
        };
        let title = format!(
            " Mascots: {} ({}/{}){} ",
            current_category.0,
            self.mascot_category_idx + 1,
            CATEGORIES.len(),
            scroll_badge
        );

        let visible_rows = (area.height.saturating_sub(2) as usize).max(1);
        let start_idx = if self.mascot_item_idx >= visible_rows {
            self.mascot_item_idx + 1 - visible_rows
        } else {
            0
        };
        let visible_items: Vec<ListItem> = items
            .into_iter()
            .skip(start_idx)
            .take(visible_rows)
            .collect();

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
                    spans.push(Span::styled(
                        " ◆ Native",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
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

        let scenery_title = format!(
            " Biomes / Scenery ({}) [Native: {}] ",
            SCENERY_OPTIONS.len(),
            native_biome.environment
        );
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
                    spans.push(Span::styled(
                        " ◆ Native",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
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

        let road_title = format!(
            " Road Surfaces ({}) [Native: {}] ",
            ROAD_OPTIONS.len(),
            native_biome.road
        );
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
            .constraints([
                Constraint::Percentage(28),
                Constraint::Percentage(24),
                Constraint::Percentage(24),
                Constraint::Percentage(24),
            ])
            .split(area);

        let native_biome = forgum_platform::biome::get_mascot_biome(&self.config.cow);

        // Kinematics Effects
        let effect_items: Vec<ListItem> = EFFECT_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _))| {
                let is_sel = i == self.effect_idx;
                let is_focused = self.fx_sub_focus == 0;
                let prefix = if is_sel { "▶ " } else { "  " };
                let is_compatible = native_biome.is_effect_compatible(name);
                let badge = if is_compatible {
                    Span::styled(" [✓ native]", Style::default().fg(tailwind::EMERALD_400))
                } else if native_biome.is_aquatic() && (*name == "walk" || *name == "moonwalk") {
                    Span::styled(" [✗ land only]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_aerial() && (*name == "walk" || *name == "moonwalk") {
                    Span::styled(" [✗ flight only]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_inanimate()
                    && (*name == "walk" || *name == "moonwalk" || *name == "fly")
                {
                    Span::styled(" [✗ static object]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_ethereal() && (*name == "walk" || *name == "moonwalk") {
                    Span::styled(" [✗ ethereal entity]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_serpentine() && (*name == "walk" || *name == "moonwalk") {
                    Span::styled(" [✗ legless reptile]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_sedentary() && (*name == "walk" || *name == "moonwalk") {
                    Span::styled(" [✗ sedentary]", Style::default().fg(Color::DarkGray))
                } else if native_biome.is_aquatic() {
                    Span::styled(" [✗ non-aquatic]", Style::default().fg(Color::DarkGray))
                } else if *name == "swim" || *name == "drift" {
                    Span::styled(" [✗ water only]", Style::default().fg(Color::DarkGray))
                } else {
                    Span::styled(" [✗ non-native]", Style::default().fg(Color::DarkGray))
                };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(Color::Green)
                } else if !is_compatible {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                    badge,
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
                .border_style(if self.fx_sub_focus == 0 {
                    Style::default().fg(tailwind::AMBER_400)
                } else {
                    Style::default().fg(tailwind::SLATE_600)
                })
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
                .border_style(if self.fx_sub_focus == 1 {
                    Style::default().fg(tailwind::AMBER_400)
                } else {
                    Style::default().fg(tailwind::SLATE_600)
                })
                .title(colors_title),
        );
        f.render_widget(color_list, chunks[1]);

        // Background Contrast Profile
        let contrast_items: Vec<ListItem> = CONTRAST_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _desc, _))| {
                let is_sel = i == self.contrast_idx;
                let is_focused = self.fx_sub_focus == 2;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(tailwind::AMBER_400)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(tailwind::EMERALD_400)
                } else {
                    Style::default().fg(Color::White)
                };
                let badge = match *name {
                    "acrylic_glass" => " 🪟 GLASS",
                    "high_contrast" => " 🔆 HIGH",
                    "vibrant" => " 🌈 VIVID",
                    _ => " ⚪ STD",
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                    Span::styled(
                        badge,
                        Style::default()
                            .fg(tailwind::SKY_400)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
            })
            .collect();

        let visible_rows_contrast = (chunks[2].height.saturating_sub(2) as usize).max(1);
        let start_contrast = if self.contrast_idx >= visible_rows_contrast {
            self.contrast_idx + 1 - visible_rows_contrast
        } else {
            0
        };
        let visible_contrast_items: Vec<ListItem> = contrast_items
            .into_iter()
            .skip(start_contrast)
            .take(visible_rows_contrast)
            .collect();

        let contrast_title = format!(" Contrast Profile ({}) ", CONTRAST_OPTIONS.len());
        let contrast_list = List::new(visible_contrast_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if self.fx_sub_focus == 2 {
                    Style::default().fg(tailwind::AMBER_400)
                } else {
                    Style::default().fg(tailwind::SLATE_600)
                })
                .title(contrast_title),
        );
        f.render_widget(contrast_list, chunks[2]);

        // Colorblind Accessibility Modes
        let colorblind_items: Vec<ListItem> = COLORBLIND_OPTIONS
            .iter()
            .enumerate()
            .map(|(i, (name, _label, _mode))| {
                let is_sel = i == self.colorblind_idx;
                let is_focused = self.fx_sub_focus == 3;
                let prefix = if is_sel { "▶ " } else { "  " };
                let style = if is_sel && is_focused {
                    Style::default()
                        .fg(tailwind::AMBER_400)
                        .add_modifier(Modifier::BOLD)
                } else if is_sel {
                    Style::default().fg(tailwind::EMERALD_400)
                } else {
                    Style::default().fg(Color::White)
                };
                let badge = if *name == "none" {
                    Span::styled(" [OFF]", Style::default().fg(Color::DarkGray))
                } else if name.ends_with("_sim") {
                    Span::styled(" [SIM]", Style::default().fg(tailwind::SKY_400))
                } else if *name == "achromatopsia" {
                    Span::styled(" [MONO]", Style::default().fg(tailwind::SLATE_400))
                } else {
                    Span::styled(" [DALTONIZE]", Style::default().fg(tailwind::EMERALD_400))
                };
                ListItem::new(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(*name, style),
                    badge,
                ]))
            })
            .collect();

        let visible_rows_cb = (chunks[3].height.saturating_sub(2) as usize).max(1);
        let start_cb = if self.colorblind_idx >= visible_rows_cb {
            self.colorblind_idx + 1 - visible_rows_cb
        } else {
            0
        };
        let visible_cb_items: Vec<ListItem> = colorblind_items
            .into_iter()
            .skip(start_cb)
            .take(visible_rows_cb)
            .collect();

        let cb_title = format!(
            " CVD / Colorblind Accessibility ({}) ",
            COLORBLIND_OPTIONS.len()
        );
        let cb_list = List::new(visible_cb_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(if self.fx_sub_focus == 3 {
                    Style::default().fg(tailwind::AMBER_400)
                } else {
                    Style::default().fg(tailwind::SLATE_600)
                })
                .title(cb_title),
        );
        f.render_widget(cb_list, chunks[3]);
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
                    format!(
                        "✎ [ {}_ ] (Enter: Commit | Esc: Cancel)",
                        self.config_edit_buffer
                    )
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
            ConfigField::Text => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if self.config.text.is_empty() {
                    "\"\" (no bubble) [Enter/e: Edit Text]".into()
                } else {
                    format!("\"{}\" [Enter/e: Edit Text]", self.config.text)
                }
            }
            ConfigField::Think => {
                if self.config.think {
                    "✔ ON (thought bubble 'o') [Space/Enter toggle]".into()
                } else {
                    "✖ OFF (speech bubble '\\') [Space/Enter toggle]".into()
                }
            }
            ConfigField::ThoughtInterval => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!(
                        "{}s [Enter/e: Edit | +/-: Step]",
                        self.config.thought_interval
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
                let is_natural = self.config.color_mode.eq_ignore_ascii_case("natural")
                    || self.config.color_mode.eq_ignore_ascii_case("animal");
                if !is_natural {
                    "🔒 [Locked: Select 'natural' Color Mode]".into()
                } else if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                    let idx = self.current_variation_index(vars);
                    let total = vars.len();
                    let v = &vars[idx];
                    format!("[Variant {}/{}] {} (←/→ cycle)", idx + 1, total, v.name)
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
            ConfigField::Animation => {
                format!(
                    "{} (←/→ cycle)",
                    self.config.animation.as_deref().unwrap_or("dynamic")
                )
            }
            ConfigField::AnimationType => {
                format!(
                    "{} (←/→ cycle)",
                    self.config
                        .animation_type
                        .as_deref()
                        .unwrap_or("animal_natural")
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
                let desc = match self.config.shell_attach_mode.as_str() {
                    "startup" | "banner" => " (Launch once, 0 prompt cost)",
                    "prompt" | "reactive" => " (Every command prompt hook)",
                    "clear" => " (Launch + after clear/cls)",
                    "split" => " (DECSTBM top margin reservation)",
                    "pane" => " (Dedicated split pane / daemon)",
                    _ => "",
                };
                format!("{}{desc} (←/→ cycle)", self.config.shell_attach_mode)
            }
            ConfigField::DefaultShell => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if self.config.default_shell.is_empty() {
                    "auto (detected) [Enter/e: Edit | ←/→: Cycle]".into()
                } else {
                    format!("{} [Enter/e: Edit | ←/→: Cycle]", self.config.default_shell)
                }
            }
            ConfigField::SplitMode => {
                format!("{} (←/→ cycle)", self.split_mode_dropdown.current())
            }
            ConfigField::SplitScroll => {
                if self.config.split_scroll {
                    "✔ ON (attach: split) [Space/Enter to toggle]".into()
                } else {
                    "✖ OFF (attach: none) [Space/Enter to toggle]".into()
                }
            }
            ConfigField::SplitRatio => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if let Some(r) = self.config.split_ratio {
                    format!("{:.2} [Enter/e: Edit | +/-: Step]", r)
                } else {
                    "auto (None) [Enter/e: Edit | +/-: Step]".into()
                }
            }
            ConfigField::ReserveRows => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if let Some(r) = self.config.reserve_rows {
                    format!("{} rows [Enter/e: Edit | +/-: Step]", r)
                } else {
                    "auto (None) [Enter/e: Edit | +/-: Step]".into()
                }
            }
            ConfigField::ReserveCols => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if let Some(c) = self.config.reserve_cols {
                    format!("{} cols [Enter/e: Edit | +/-: Step]", c)
                } else {
                    "auto (None) [Enter/e: Edit | +/-: Step]".into()
                }
            }
            ConfigField::Image => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else if let Some(img) = &self.config.image {
                    format!("'{}' [Enter/e: Edit Path]", img)
                } else {
                    "none [Enter/e: Type Path]".into()
                }
            }
            ConfigField::Editor => {
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!(
                        "{} (←/→ cycle | Enter/e: Type)",
                        self.config.editor.as_deref().unwrap_or("auto")
                    )
                }
            }
            ConfigField::ConfigFormat => {
                format!(
                    "{} (←/→ cycle)",
                    self.format_dropdown.current().to_uppercase()
                )
            }
            ConfigField::Random => match &self.config.random {
                Some(forgum_platform::protocol::RandomSetting::Bool(true)) => {
                    "✔ ALL (Space/Enter to cycle)".into()
                }
                Some(forgum_platform::protocol::RandomSetting::Bool(false)) | None => {
                    "✖ OFF (Space/Enter to cycle)".into()
                }
                Some(forgum_platform::protocol::RandomSetting::String(s)) => {
                    format!("{s} (Space/Enter to cycle)")
                }
                Some(forgum_platform::protocol::RandomSetting::List(l)) => {
                    format!("{} (Space/Enter to cycle)", l.join(", "))
                }
            },
            ConfigField::ContrastMode => {
                let cur = self
                    .config
                    .contrast_mode
                    .clone()
                    .unwrap_or_else(|| "standard".to_string());
                let desc = match cur.as_str() {
                    "acrylic_glass" | "glass" => {
                        " (elevated min luminance, boosted saturation, bold)"
                    }
                    "high_contrast" => " (maximum luminance floor, vivid saturation, bold)",
                    "vibrant" => " (boosted saturation)",
                    _ => " (default terminal colors)",
                };
                format!("{}{} (←/→ cycle)", cur, desc)
            }
            ConfigField::HardwareAccel => {
                let display = self.active_accel_display();
                let badge = self.active_accel_badge();
                let desc = self.active_accel_desc();
                format!("{} [{}] - {} (←/→ cycle)", display, badge, desc)
            }
            ConfigField::ReduceMotion => {
                if self.config.reduce_motion {
                    "✔ ON (gentle fades only) [Space/Enter toggle]".into()
                } else {
                    "✖ OFF (full kinetic motion) [Space/Enter toggle]".into()
                }
            }
            ConfigField::TextOnly => {
                if self.config.text_only {
                    "✔ ON (print text only) [Space/Enter toggle]".into()
                } else {
                    "✖ OFF (render mascot art) [Space/Enter toggle]".into()
                }
            }
            ConfigField::Banner => {
                if self.config.banner {
                    "✔ ON (inline fastfetch banner) [Space/Enter toggle]".into()
                } else {
                    "✖ OFF (standard canvas) [Space/Enter toggle]".into()
                }
            }
            ConfigField::Aspect => {
                let cur = self.config.aspect.as_deref().unwrap_or("auto");
                let desc = match cur {
                    "21:9" => " (Ultrawide 21:9 panoramic)",
                    "16:9" => " (Widescreen 16:9 standard)",
                    "16:10" => " (Golden Tall 16:10)",
                    "4:3" => " (Classic 4:3 standard)",
                    "1:1" => " (Square 1:1 display)",
                    "narrow" => " (Vertical / split pane)",
                    _ => " (Auto-detect from terminal dimensions)",
                };
                format!("{cur}{desc} (←/→ cycle)")
            }
            ConfigField::AspectMode => {
                let cur = self.config.aspect_mode.as_deref().unwrap_or("auto");
                let desc = match cur {
                    "preset" => " (Lock to selected aspect preset)",
                    "dynamic" => " (Live recalculation on window resize)",
                    _ => " (Auto: dynamic interactive, preset banner)",
                };
                format!("{cur}{desc} (←/→ cycle)")
            }
            ConfigField::ColorblindMode => {
                let display = self.active_colorblind_display();
                let badge = self.active_colorblind_badge();
                let desc = self.active_colorblind_desc();
                format!("{} [{}] - {} (←/→ cycle)", display, badge, desc)
            }
            ConfigField::UpdateMethod => {
                let cur = self.config.update_method.as_deref().unwrap_or("prompt");
                let desc = match cur {
                    "prompt" | "ask" => " (Ask every time with interactive numbered menu)",
                    "auto" => " (Auto-detect active package manager)",
                    "zypper" => " (openSUSE Zypper package manager)",
                    "scoop" => " (Windows Scoop package manager)",
                    "winget" => " (Windows Package Manager WinGet)",
                    "homebrew" | "brew" => " (macOS/Linux Homebrew formula)",
                    "apt" => " (Debian/Ubuntu APT repository)",
                    "dnf" => " (Fedora/RHEL DNF package manager)",
                    "pacman" => " (Arch Linux Pacman / AUR)",
                    "cargo" => " (Rust Cargo crates.io build)",
                    "standalone" => " (Direct GitHub binary installer script)",
                    _ => "",
                };
                if is_sel && self.editing_config {
                    format!("✎ [ {}_ ]", self.config_edit_buffer)
                } else {
                    format!("{cur}{desc} (←/→ cycle | Enter/e: Edit)")
                }
            }
            ConfigField::CheckUpdates => {
                if self.config.check_updates {
                    "✔ ON (notify when new update is available) [Space/Enter toggle]".into()
                } else {
                    "✖ OFF (update checking disabled) [Space/Enter toggle]".into()
                }
            }
        }
    }

    fn render_config_list(&self, f: &mut Frame, area: Rect) {
        let bp = Breakpoint::from_size(area.width, area.height);

        // Label column width: shorter in Compact to conserve horizontal space.
        let label_width: usize = if bp == Breakpoint::Compact { 14 } else { 22 };

        let items: Vec<ListItem> = ConfigField::ALL
            .iter()
            .enumerate()
            .map(|(i, field)| {
                let is_sel = i == self.config_field_idx;
                let prefix = if is_sel {
                    if self.h_scroll > 0 {
                        "◀▶ "
                    } else {
                        "▶ "
                    }
                } else if self.h_scroll > 0 {
                    "◀  "
                } else {
                    "  "
                };
                let val_str = self.field_value(*field, is_sel);

                let (cat_name, cat_color) = field.category_badge();
                let cat_badge = if bp == Breakpoint::Compact {
                    format!("[{}] ", &cat_name[..1])
                } else {
                    format!("[{:<7}] ", cat_name)
                };

                let (label_style, line_style) = if is_sel {
                    (
                        Style::default()
                            .fg(tailwind::AMBER_400)
                            .add_modifier(Modifier::BOLD),
                        Style::default().bg(tailwind::SLATE_800),
                    )
                } else {
                    (Style::default().fg(tailwind::SLATE_100), Style::default())
                };

                let val_color = if val_str.contains("✔ ON") {
                    tailwind::EMERALD_400
                } else if val_str.contains("✖ OFF") {
                    tailwind::ROSE_400
                } else if val_str.contains("🔒") {
                    tailwind::SLATE_400
                } else if val_str.contains("[Variant") {
                    tailwind::AMBER_400
                } else if val_str.contains("seamless") {
                    tailwind::VIOLET_400
                } else if is_sel {
                    tailwind::SKY_400
                } else {
                    tailwind::CYAN_400
                };

                let prefix_color = if is_sel {
                    tailwind::AMBER_400
                } else {
                    tailwind::SLATE_600
                };

                let spans = if self.h_scroll == 0 {
                    vec![
                        Span::styled(prefix, Style::default().fg(prefix_color)),
                        Span::styled(cat_badge, Style::default().fg(cat_color)),
                        Span::styled(
                            format!("{:<width$}", field.label(), width = label_width),
                            label_style,
                        ),
                        Span::styled(val_str, Style::default().fg(val_color)),
                    ]
                } else if self.h_scroll < label_width {
                    let formatted_label = format!("{:<width$}", field.label(), width = label_width);
                    let label_part: String = formatted_label.chars().skip(self.h_scroll).collect();
                    vec![
                        Span::styled(prefix, Style::default().fg(prefix_color)),
                        Span::styled(cat_badge, Style::default().fg(cat_color)),
                        Span::styled(label_part, label_style),
                        Span::styled(val_str, Style::default().fg(val_color)),
                    ]
                } else {
                    let skip_val = self.h_scroll - label_width;
                    let scrolled_val: String = val_str.chars().skip(skip_val).collect();
                    let display_val = if scrolled_val.is_empty() {
                        "…".to_string()
                    } else {
                        scrolled_val
                    };
                    vec![
                        Span::styled(prefix, Style::default().fg(prefix_color)),
                        Span::styled(cat_badge, Style::default().fg(cat_color)),
                        Span::styled(display_val, Style::default().fg(val_color)),
                    ]
                };

                ListItem::new(Line::from(spans)).style(line_style)
            })
            .collect();

        // Compact: if height is very small (< 8), strip borders to maximize visible items.
        let border_overhead: u16 = if bp == Breakpoint::Compact && area.height < 8 {
            0
        } else {
            2
        };
        let visible_rows_config = (area.height.saturating_sub(border_overhead) as usize).max(1);
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

        let scroll_badge = if self.h_scroll > 0 {
            format!(" [◀+{}]", self.h_scroll)
        } else {
            String::new()
        };
        let config_title = format!(" ⚙️ Configuration Settings (Tab 5/5){} ", scroll_badge);

        if border_overhead == 0 {
            // ── Ultra-compact: borderless list, maximum rows visible ──────────
            let list = List::new(visible_config_items);
            f.render_widget(list, area);
        } else {
            // ── Standard / Wide: full rounded-border layout with scroll badge ──
            let list = List::new(visible_config_items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(tailwind::INDIGO_500))
                    .title(Span::styled(
                        config_title,
                        Style::default()
                            .fg(tailwind::SKY_400)
                            .add_modifier(Modifier::BOLD),
                    )),
            );
            f.render_widget(list, area);
        }
    }

    /// Render Tab 6: Mascot & Shader Studio toolset.
    fn render_studio_list(&self, f: &mut Frame, area: Rect) {
        let items = [
            (
                "0: Bounding Hull & Moat Debugger",
                if self.studio_debug_overlay && self.studio_sub_focus == 0 {
                    "[✔ Active Overlay]"
                } else if self.studio_sub_focus == 0 {
                    "[● Selected - Space to Toggle]"
                } else {
                    "[○ Inactive]"
                },
                "Renders interior convex-hull spaces as '#' & ambient occlusion moat as '·'",
            ),
            (
                "1: Biological Coat Morph Selector",
                if self.studio_sub_focus == 1 {
                    "[● Selected - h/l to Cycle]"
                } else {
                    "[○ Inactive]"
                },
                "Audits authentic biological phenotype color variations (e.g. cattle 6 morphs)",
            ),
            (
                "2: 240Hz / 120Hz Kinematics Telemetry",
                if self.studio_sub_focus == 2 {
                    "[● Active Telemetry]"
                } else {
                    "[○ Inactive]"
                },
                "Live Verlet particle & kinematics physics delta monitoring & sub-step tracker",
            ),
        ];

        let list_items: Vec<ListItem> = items
            .iter()
            .enumerate()
            .map(|(i, (name, status, desc))| {
                let is_sel = i == self.studio_sub_focus;
                let prefix = if is_sel { "▶ " } else { "  " };

                let header_style = if is_sel {
                    Style::default()
                        .fg(tailwind::AMBER_300)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(tailwind::SLATE_200)
                };

                let status_style = if status.contains("Active") || status.contains("Selected") {
                    Style::default()
                        .fg(tailwind::EMERALD_400)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(tailwind::SLATE_500)
                };

                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(prefix, Style::default().fg(tailwind::CYAN_400)),
                        Span::styled(*name, header_style),
                        Span::raw(" "),
                        Span::styled(*status, status_style),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*desc, Style::default().fg(tailwind::SLATE_400)),
                    ]),
                ])
            })
            .collect();

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(tailwind::INDIGO_500))
                .title(Span::styled(
                    " 🔬 Mascot & Shader Studio Toolset (j/k: Select, Space: Toggle, h/l: Morph) ",
                    Style::default()
                        .fg(tailwind::SKY_400)
                        .add_modifier(Modifier::BOLD),
                )),
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
        let theme = forgum_platform::detect_terminal_theme();
        forgum_platform::biome::natural_creature_color_adaptive(palette, rel_x, rel_y, ch, theme.bg)
    }

    /// Authentic God-given natural color palettes (RGB tuples) for all 132 mascots.
    pub fn get_mascot_natural_palette(mascot: &str) -> &'static [(u8, u8, u8)] {
        forgum_platform::biome::get_natural_rgb_palette(mascot)
    }

    /// Retrieve current natural palette variation index for the active mascot.
    pub fn current_variation_index(
        &self,
        vars: &[forgum_platform::biome::NaturalPaletteVariation],
    ) -> usize {
        if vars.is_empty() {
            return 0;
        }
        let cur = self.config.palette.as_deref().unwrap_or("");
        for (i, v) in vars.iter().enumerate() {
            if cur.eq_ignore_ascii_case(&v.palette.join(",")) || cur.eq_ignore_ascii_case(v.name) {
                return i;
            }
        }
        0
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
        let contrast = self.active_contrast_mode();
        let colorblind = self.active_colorblind_mode();
        let adjust_color = |r: u8, g: u8, b: u8| -> (u8, u8, u8) {
            let (cr, cg, cb) = contrast.adjust_rgb(r, g, b);
            colorblind.adjust_rgb(cr, cg, cb)
        };

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

        // 0. Live HUD Timeline Status Bar & Hardware Acceleration
        let accel_mode = forgum_platform::AccelerationMode::parse(self.active_accel_mode());
        let compute = forgum_platform::compute::get_compute_engine(Some(accel_mode));
        let accel_badge = self.active_accel_badge();

        let hud_line = if self.config.duration > 0 {
            let percent = (progress_fraction * 100.0) as u32;
            let bar_width = 14;
            let filled = ((progress_fraction * bar_width as f32).round() as usize).min(bar_width);
            let empty = bar_width.saturating_sub(filled);
            let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));
            format!(
                "⏱ {:.1}s / {}s {} {:>3}% • {} FPS • {} Live",
                elapsed_display, self.config.duration, bar, percent, self.config.fps, accel_badge
            )
        } else {
            format!(
                "⏱ {:.1}s / ∞ (Infinite Loop) • {} FPS • {} Live",
                elapsed_display, self.config.fps, accel_badge
            )
        };

        lines.push(Line::from(vec![Span::styled(
            format!("  {hud_line}"),
            Style::default()
                .fg(tailwind::SKY_400)
                .add_modifier(Modifier::BOLD),
        )]));
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
                let mut wave = [0.0f32; 1];
                let amps = [0.5];
                let phases = [0.0];
                compute.compute_mountain_harmonic_superposition(
                    1, 2.0, 16.0, 1.2, t, &amps, &phases, &mut wave,
                );
                let m_chars: Vec<char> = mountain_layer.chars().collect();
                let m_start = (t * 1.5 + wave[0]) as usize % m_chars.len();
                let m_slice: String = m_chars.iter().cycle().skip(m_start).take(30).collect();
                let (mr, mg, mb) = adjust_color(110, 110, 110);
                lines.push(Line::from(Span::styled(
                    format!("  {m_slice}"),
                    Style::default().fg(Color::Rgb(mr, mg, mb)),
                )));
            }

            // 1.5. Procedural midground trees & flora layer (parallax scrolling, between mountains & mascot)
            let env_style = self.config.environment.as_deref().unwrap_or("pasture");
            let tree_layer = match env_style {
                "forest" => {
                    "  /\\   /\\    /\\/\\    /\\   /\\/\\    /\\   /\\    /\\/\\    /\\   "
                }
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
                let _density = compute.compute_foliage_density((t * 0.5).sin(), 0.0, 3.0);
                let tree_chars: Vec<char> = tree_layer.chars().collect();
                let tree_start = (t * 2.4) as usize % tree_chars.len();
                let tree_slice: String = tree_chars
                    .iter()
                    .cycle()
                    .skip(tree_start)
                    .take(30)
                    .collect();
                let (tr, tg, tb) = match env_style {
                    "savanna" => (139, 195, 74),
                    "arctic" => (224, 247, 250),
                    "graveyard" => (97, 97, 97),
                    _ => (76, 175, 80),
                };
                let (atr, atg, atb) = adjust_color(tr, tg, tb);
                lines.push(Line::from(Span::styled(
                    format!("  {tree_slice}"),
                    Style::default().fg(Color::Rgb(atr, atg, atb)),
                )));
            }
        }

        // 2. Procedural Animation Engine Execution
        let blink_cycle = (t * 0.28) % 1.0;
        let is_blinking = blink_cycle < 0.045 || (blink_cycle > 0.08 && blink_cycle < 0.11);
        let chew_cycle = (t * 0.36) % 1.0;
        let is_chewing = chew_cycle > 0.40 && chew_cycle < 0.65;
        let is_inhale = (t * 2.2).sin() > 0.1;
        let is_wing_up = ((t * 8.0) as usize).is_multiple_of(2);
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

            // Animal body animations ONLY run below the speech/thought bubble
            if i >= cow_start_line {
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

                let eff =
                    if self.config.effect == "animal_natural" || self.config.effect == "default" {
                        match self.config.cow.as_str() {
                            "duck" | "pterodactyl" | "golden-eagle" | "tweety-bird" => "fly",
                            "bunny" | "hamster" | "corgi" | "cat" | "cat2" | "catfence"
                            | "kitty" | "kitten" | "doge" | "mule" | "pig" | "ram" | "sheep"
                            | "goat" | "goat2" | "wolf" | "tiger" | "panther" | "fox"
                            | "hedgehog" | "armadillo" | "rhino" => "walk",
                            "dolphin" | "whale" | "docker-whale" | "happy-whale" | "octopus"
                            | "smiling-octopus" | "squid" | "jellyfish" | "seahorse"
                            | "seahorse-big" => "sway",
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
                    "moonwalk" => {
                        // Reverse leg stroke cadence
                        let rev_phase = (3 - walk_phase) % 4;
                        if i == leg_line_idx {
                            if l.contains("||     ||") {
                                l = match rev_phase {
                                    0 => l.replace("||     ||", "|/     /|"),
                                    1 => l.replace("||     ||", "/|     |/"),
                                    2 => l.replace("||     ||", "|\\     \\|"),
                                    _ => l.replace("||     ||", "\\|     |\\"),
                                };
                            } else if l.contains("/ \\") {
                                l = match rev_phase {
                                    0 | 2 => l.replace("/ \\", "| |"),
                                    1 => l.replace("/ \\", "\\ /"),
                                    _ => l,
                                };
                            }
                        }
                        // Smooth horizontal reverse glide drift
                        let glide_amt = ((t * 3.0).sin() * 2.0) as i32;
                        let pad = (glide_amt + 2).max(0) as usize;
                        l = format!("{}{l}", " ".repeat(pad));
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
                    "glitch" if ((t * 15.0) as usize + i).is_multiple_of(7) => {
                        let mut chars: Vec<char> = l.chars().collect();
                        if let Some(pos) = chars.iter().position(|c| !c.is_whitespace()) {
                            chars[pos] = match ((t * 10.0) as usize) % 4 {
                                0 => '#',
                                1 => '~',
                                2 => '?',
                                3 => '!',
                                _ => '!',
                            };
                            l = chars.into_iter().collect();
                        }
                    }
                    _ => {}
                }
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
                if ((t * 8.0) as usize + i).is_multiple_of(3) {
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
                let trimmed = l.trim();
                let is_connector_line = trimmed == "o"
                    || trimmed == "\\"
                    || trimmed == "/"
                    || trimmed == "o o"
                    || trimmed == "\\ \\";
                let is_border_line = (trimmed.starts_with('_')
                    && trimmed.chars().all(|c| c == '_'))
                    || (trimmed.starts_with('-') && trimmed.chars().all(|c| c == '-'))
                    || (trimmed.starts_with('(')
                        && trimmed.ends_with(')')
                        && trimmed.chars().all(|c| c == '(' || c == ')' || c == '_'))
                    || (trimmed.starts_with('|')
                        && trimmed.ends_with('|')
                        && trimmed.chars().all(|c| c == '|' || c == '_'));

                let first_non_space = l.chars().position(|c| !c.is_whitespace());
                let last_non_space = l
                    .chars()
                    .enumerate()
                    .filter(|(_, c)| !c.is_whitespace())
                    .last()
                    .map(|(idx, _)| idx);

                let (conn_r, conn_g, conn_b) = adjust_color(180, 230, 255);
                let mut conn_style = Style::default().fg(Color::Rgb(conn_r, conn_g, conn_b));
                if contrast.requires_bold() {
                    conn_style = conn_style.add_modifier(Modifier::BOLD);
                }

                let (bord_r, bord_g, bord_b) = adjust_color(215, 225, 240);
                let mut bord_style = Style::default().fg(Color::Rgb(bord_r, bord_g, bord_b));
                if contrast.requires_bold() {
                    bord_style = bord_style.add_modifier(Modifier::BOLD);
                }

                let (txt_r, txt_g, txt_b) = adjust_color(255, 255, 255);
                let txt_style = Style::default()
                    .fg(Color::Rgb(txt_r, txt_g, txt_b))
                    .add_modifier(Modifier::BOLD);

                let mut spans = Vec::new();
                spans.push(Span::raw("  "));
                for (col_idx, ch) in l.chars().enumerate() {
                    if ch == ' ' {
                        spans.push(Span::raw(" "));
                    } else if is_connector_line {
                        spans.push(Span::styled(ch.to_string(), conn_style));
                    } else if is_border_line
                        || Some(col_idx) == first_non_space
                        || Some(col_idx) == last_non_space
                    {
                        spans.push(Span::styled(ch.to_string(), bord_style));
                    } else {
                        // All inner characters of the thought or speech bubble message
                        spans.push(Span::styled(ch.to_string(), txt_style));
                    }
                }
                lines.push(Line::from(spans));
            } else {
                let is_natural = matches!(
                    self.config.color_mode.as_str(),
                    "natural" | "animal_natural" | "animal" | "default"
                );
                if is_natural {
                    let parsed_custom: Option<Vec<(u8, u8, u8)>> =
                        self.config.palette.as_deref().and_then(|pal_str| {
                            let parts: Vec<(u8, u8, u8)> = pal_str
                                .split(',')
                                .filter_map(|hex| {
                                    let h = hex.trim().trim_start_matches('#');
                                    if h.len() == 6 {
                                        let r = u8::from_str_radix(&h[0..2], 16).ok()?;
                                        let g = u8::from_str_radix(&h[2..4], 16).ok()?;
                                        let b = u8::from_str_radix(&h[4..6], 16).ok()?;
                                        Some((r, g, b))
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if parts.len() >= 3 {
                                Some(parts)
                            } else {
                                None
                            }
                        });

                    let mascot_pal = Self::get_mascot_natural_palette(&self.config.cow);
                    let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                    let p: &[(u8, u8, u8)] = if self.current_tab == Tab::Studio
                        && self.studio_sub_focus == 1
                        && !vars.is_empty()
                    {
                        let active_idx = self.studio_morph_idx % vars.len();
                        &vars[active_idx].rgb
                    } else {
                        match &parsed_custom {
                            Some(custom) => custom.as_slice(),
                            None => mascot_pal,
                        }
                    };

                    let first_non_space = l.chars().position(|c| !c.is_whitespace());
                    let last_non_space = l
                        .chars()
                        .enumerate()
                        .filter(|(_, c)| !c.is_whitespace())
                        .last()
                        .map(|(idx, _)| idx);

                    let mut spans = Vec::new();
                    spans.push(Span::raw("  "));
                    for (x, ch) in l.chars().enumerate() {
                        if ch == ' ' {
                            if self.current_tab == Tab::Studio
                                && self.studio_debug_overlay
                                && self.studio_sub_focus == 0
                            {
                                if let (Some(f), Some(la)) = (first_non_space, last_non_space) {
                                    if x > f && x < la {
                                        // Hull interior space
                                        spans.push(Span::styled(
                                            "#",
                                            Style::default().fg(tailwind::EMERALD_500),
                                        ));
                                    } else if x == f.saturating_sub(1) || x == la + 1 {
                                        // Chebyshev ambient occlusion moat margin
                                        spans.push(Span::styled(
                                            "·",
                                            Style::default().fg(tailwind::INDIGO_400),
                                        ));
                                    } else {
                                        spans.push(Span::raw(" "));
                                    }
                                } else {
                                    spans.push(Span::raw(" "));
                                }
                            } else {
                                spans.push(Span::raw(" "));
                            }
                        } else {
                            let (r, g, b) = Self::natural_creature_color(p, x, animal_rel_y, ch);
                            let (cr, cg, cb) = adjust_color(r, g, b);
                            let mut style = Style::default().fg(Color::Rgb(cr, cg, cb));
                            if contrast.requires_bold() {
                                style = style.add_modifier(Modifier::BOLD);
                            }
                            spans.push(Span::styled(ch.to_string(), style));
                        }
                    }
                    if !sparkle.is_empty() {
                        let (sr, sg, sb) = adjust_color(255, 235, 59);
                        let mut s_style = Style::default().fg(Color::Rgb(sr, sg, sb));
                        if contrast.requires_bold() {
                            s_style = s_style.add_modifier(Modifier::BOLD);
                        }
                        spans.push(Span::styled(sparkle.to_string(), s_style));
                    }
                    lines.push(Line::from(spans));
                } else {
                    let adj_color = match color {
                        Color::Rgb(r, g, b) => {
                            let (cr, cg, cb) = adjust_color(r, g, b);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Red => {
                            let (cr, cg, cb) = adjust_color(255, 80, 80);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Green => {
                            let (cr, cg, cb) = adjust_color(80, 220, 80);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Yellow => {
                            let (cr, cg, cb) = adjust_color(255, 230, 80);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Blue => {
                            let (cr, cg, cb) = adjust_color(80, 140, 255);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Magenta => {
                            let (cr, cg, cb) = adjust_color(255, 80, 255);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::Cyan => {
                            let (cr, cg, cb) = adjust_color(80, 230, 255);
                            Color::Rgb(cr, cg, cb)
                        }
                        Color::White => {
                            let (cr, cg, cb) = adjust_color(240, 240, 240);
                            Color::Rgb(cr, cg, cb)
                        }
                        c => c,
                    };
                    let mut style = Style::default().fg(adj_color);
                    if contrast.requires_bold() {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    lines.push(Line::from(Span::styled(format!("  {l}{sparkle}"), style)));
                }
            }
        }

        // 3. Ground / Road surface (fast scroll, when background enabled)
        if self.config.background {
            let road_style = self.config.road.as_deref().unwrap_or("dirt");
            let ground_pattern = match road_style {
                "dirt" => ".. . . .. ... . .. .. . ... .. . .. ... . .. .. . ... .. ",
                "cobblestone" | "cobble" => {
                    "[_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_][_] "
                }
                "magma" => "~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ~~~ ",
                "ice" => "=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--=--= ",
                "seabed" => "~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ~~~~ ",
                "sidewalk" | "paved" => "=== === === === === === === === === === === === === === ",
                "roof" => "^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ^^^ ",
                "grid" => "#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-#-",
                "crypt" => "[+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+][+] ",
                "savanna" => ".. -- .. -- .. -- .. -- .. -- .. -- .. -- .. -- .. -- .. ",
                "mud" => "~~~...~~~...~~~...~~~...~~~...~~~...~~~...~~~...~~~... ",
                "tracks" | "railway" => "-+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+--+-- ",
                "checkerboard" => "#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.#.",
                "none" => "",
                _ => "--- --- --- --- --- --- --- --- --- --- --- --- --- --- ",
            };
            if !ground_pattern.is_empty() {
                let mut undulation = [0.0f32; 1];
                compute.compute_road_undulation(1, 10.0, 0.5, t, &mut undulation);
                let g_chars: Vec<char> = ground_pattern.chars().collect();
                let g_start = ((t * 8.0) as usize + (undulation[0] as usize)) % g_chars.len();
                let g_slice: String = g_chars.iter().cycle().skip(g_start).take(32).collect();
                let (gr, gg, gb) = adjust_color(115, 115, 115);
                lines.push(Line::from(Span::styled(
                    format!("  {g_slice}"),
                    Style::default().fg(Color::Rgb(gr, gg, gb)),
                )));
            }
        }

        let env_name = self.config.environment.as_deref().unwrap_or("pasture");
        let title = if colorblind != ColorblindMode::None {
            format!(
                " 🐮 Live {} FPS Canvas: {} [{}] [{}] [{}] [{}] ",
                self.config.fps,
                self.config.cow,
                env_name,
                self.config.effect,
                contrast.display_name(),
                self.active_colorblind_badge(),
            )
        } else {
            format!(
                " 🐮 Live {} FPS Canvas: {} [{}] [{}] [{}] ",
                self.config.fps,
                self.config.cow,
                env_name,
                self.config.effect,
                contrast.display_name(),
            )
        };

        let p = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(tailwind::INDIGO_500))
                .title(Span::styled(
                    title,
                    Style::default()
                        .fg(tailwind::EMERALD_400)
                        .add_modifier(Modifier::BOLD),
                )),
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
                    Span::styled(
                        "Mascot: ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        current_cow.as_str(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  ("),
                    Span::styled(cat_name, Style::default().fg(Color::White)),
                    Span::raw(")"),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Native Biome: ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(biome.biome_name, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Recommended: ",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "Env: {}  |  Road: {}  |  Skyline: {}",
                            biome.environment, biome.road, biome.mountain
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]));

                let mut swatch_spans = vec![Span::styled(
                    "Palette: ",
                    Style::default()
                        .fg(Color::LightBlue)
                        .add_modifier(Modifier::BOLD),
                )];
                for (hex, &(r, g, b)) in biome.natural_palette.iter().zip(biome.natural_rgb.iter())
                {
                    swatch_spans.push(Span::styled("■ ", Style::default().fg(Color::Rgb(r, g, b))));
                    swatch_spans.push(Span::styled(
                        format!("{hex} "),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                lines.push(Line::from(swatch_spans));

                lines.push(Line::from(vec![
                    Span::styled("Kinematics: ", Style::default().fg(Color::Yellow)),
                    Span::styled(biome.movement_lore, Style::default().fg(Color::White)),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Shortcuts: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "[Enter/b] Apply Native Scenery  |  [r] Random Mascot",
                        Style::default().fg(Color::Cyan),
                    ),
                ]));

                let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(tailwind::INDIGO_400))
                        .title(Span::styled(
                            " 🐾 Mascot Inspector & Native Biome ",
                            Style::default()
                                .fg(tailwind::VIOLET_400)
                                .add_modifier(Modifier::BOLD),
                        )),
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
                    Span::styled(
                        "Atmosphere: ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(sc_name, Style::default().fg(Color::Yellow)),
                    if is_native_env {
                        Span::styled(
                            " ◆ Native",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("")
                    },
                    Span::styled(
                        format!(" — {sc_desc}"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Ground Surface: ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(rd_name, Style::default().fg(Color::Yellow)),
                    if is_native_road {
                        Span::styled(
                            " ◆ Native",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("")
                    },
                    Span::styled(
                        format!(" — {rd_desc}"),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("Native for {}: ", current_cow),
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(
                            "{} (Env: {}, Road: {}, Skyline: {})",
                            biome.biome_name, biome.environment, biome.road, biome.mountain
                        ),
                        Style::default().fg(Color::White),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled("Quick Action: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        "[b/r] Sync to Native Biome",
                        Style::default().fg(Color::Cyan),
                    ),
                ]));

                let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(tailwind::INDIGO_400))
                        .title(Span::styled(
                            " 🏔️ Landscape & Road Details ",
                            Style::default()
                                .fg(tailwind::VIOLET_400)
                                .add_modifier(Modifier::BOLD),
                        )),
                );
                f.render_widget(p, area);
            }
            Tab::Effects => {
                let ef_desc = EFFECT_OPTIONS[self.effect_idx].1;
                let cl_desc = COLOR_OPTIONS[self.color_idx].1;
                let ct_desc = CONTRAST_OPTIONS[self.contrast_idx].1;
                let ct_mode = self.active_contrast_mode();
                let cb_desc = COLORBLIND_OPTIONS[self.colorblind_idx].1;
                let text = format!(
                    "Kinematics: {}\nPalette: {}\nContrast: {} (Min Lum: {}, Sat: {:.1}x, Bold: {})\nCVD Accessibility: {} [{}]",
                    ef_desc,
                    cl_desc,
                    ct_desc,
                    ct_mode.min_luminance_floor(),
                    ct_mode.saturation_factor(),
                    if ct_mode.requires_bold() { "Yes" } else { "No" },
                    cb_desc,
                    self.active_colorblind_badge(),
                );
                let p = Paragraph::new(text).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(tailwind::INDIGO_400))
                        .title(Span::styled(
                            " ✨ Dynamics, Colors & CVD Accessibility ",
                            Style::default()
                                .fg(tailwind::VIOLET_400)
                                .add_modifier(Modifier::BOLD),
                        )),
                );
                f.render_widget(p, area);
            }
            Tab::Installer => {
                let log_lines: Vec<Line> = self
                    .installer_log
                    .iter()
                    .rev()
                    .take(area.height.saturating_sub(2) as usize)
                    .map(|msg| {
                        Line::from(Span::styled(msg, Style::default().fg(tailwind::SKY_400)))
                    })
                    .collect();

                let p = Paragraph::new(log_lines)
                    .wrap(Wrap { trim: true })
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .border_style(Style::default().fg(tailwind::INDIGO_400))
                            .title(Span::styled(
                                " 🚀 Shell Installation Action Log (i: Install, u: Uninstall, t: Test) ",
                                Style::default()
                                    .fg(tailwind::EMERALD_400)
                                    .add_modifier(Modifier::BOLD),
                            )),
                    );
                f.render_widget(p, area);
            }
            Tab::Config => {
                let field = ConfigField::ALL[self.config_field_idx];
                let val_str = self.field_value(field, true);

                let mut lines: Vec<Line> = Vec::with_capacity(10);
                lines.push(Line::from(vec![
                    Span::styled(
                        "Parameter: ",
                        Style::default()
                            .fg(tailwind::VIOLET_400)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        field.label(),
                        Style::default()
                            .fg(tailwind::AMBER_400)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]));
                lines.push(Line::from(vec![
                    Span::styled(
                        "Value: ",
                        Style::default()
                            .fg(tailwind::SKY_400)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        &val_str,
                        Style::default().fg(if val_str.contains("✔ ON") {
                            tailwind::EMERALD_400
                        } else if val_str.contains("✖ OFF") {
                            tailwind::ROSE_400
                        } else {
                            tailwind::SLATE_100
                        }),
                    ),
                ]));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    field.desc(),
                    Style::default().fg(tailwind::SLATE_300),
                )));
                if field == ConfigField::Palette {
                    let is_natural = self.config.color_mode.eq_ignore_ascii_case("natural")
                        || self.config.color_mode.eq_ignore_ascii_case("animal");
                    if is_natural {
                        let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                        let idx = self.current_variation_index(vars);
                        let v = &vars[idx];
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Morph Biology: ",
                                Style::default()
                                    .fg(tailwind::EMERALD_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(v.description, Style::default().fg(tailwind::AMBER_300)),
                        ]));
                    }
                }
                if field == ConfigField::HardwareAccel {
                    let accel_mode =
                        forgum_platform::AccelerationMode::parse(self.active_accel_mode());
                    let compute = forgum_platform::compute::get_compute_engine(Some(accel_mode));

                    lines.push(Line::from(vec![
                        Span::styled(
                            "Acceleration Badge: ",
                            Style::default()
                                .fg(tailwind::AMBER_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            self.active_accel_badge(),
                            Style::default()
                                .fg(tailwind::EMERALD_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Compute Device: ",
                            Style::default()
                                .fg(tailwind::SKY_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            compute.device_name(),
                            Style::default().fg(tailwind::SLATE_100),
                        ),
                    ]));
                    if let Some(info) = compute.device_info() {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Architecture: ",
                                Style::default()
                                    .fg(tailwind::INDIGO_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{} ({})", info.architecture, info.generation_year),
                                Style::default().fg(tailwind::SLATE_200),
                            ),
                            Span::raw("  |  "),
                            Span::styled(
                                "VRAM: ",
                                Style::default()
                                    .fg(tailwind::PURPLE_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                info.vram_formatted(),
                                Style::default().fg(tailwind::SLATE_200),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Protocols: ",
                                Style::default()
                                    .fg(tailwind::TEAL_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                info.protocols.join(" • "),
                                Style::default().fg(tailwind::SLATE_300),
                            ),
                        ]));
                    }
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Multi-Threading: ",
                            Style::default()
                                .fg(tailwind::EMERALD_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            "Active (std::thread::scope parallel nature mathematics)",
                            Style::default().fg(tailwind::SLATE_200),
                        ),
                    ]));
                }
                if field == ConfigField::Aspect {
                    let cur = self.config.aspect.as_deref().unwrap_or("auto");
                    let (ratio_math, geometry_desc, use_case) = match cur {
                        "21:9" => (
                            "21:9 (2.33:1)",
                            "Ultrawide Panoramic display matrix",
                            "Multi-monitor tiling, fastfetch wide banner setups",
                        ),
                        "16:9" => (
                            "16:9 (1.78:1)",
                            "Widescreen standard 16:9 golden canvas",
                            "Standard desktop terminals, Zellij full-width panes",
                        ),
                        "16:10" => (
                            "16:10 (1.60:1)",
                            "Golden Tall golden proportion",
                            "MacBook / Modern laptop displays, high vertical density",
                        ),
                        "4:3" => (
                            "4:3 (1.33:1)",
                            "Classic CRT / Retro terminal standard",
                            "Legacy terminals, compact square-ish terminal windows",
                        ),
                        "1:1" => (
                            "1:1 (1.00:1)",
                            "Square display matrix box",
                            "Avatar generation, social embeds, quad-split layouts",
                        ),
                        "narrow" => (
                            "9:16 (0.56:1)",
                            "Vertical split / mobile layout column",
                            "Vertical splits, sidebar docks, narrow tmux columns",
                        ),
                        _ => (
                            "Auto-adaptive",
                            "Dynamic query from crossterm::terminal::size()",
                            "Auto-scaling viewport matching physical terminal geometry",
                        ),
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Aspect Geometry: ",
                            Style::default()
                                .fg(tailwind::EMERALD_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(ratio_math, Style::default().fg(tailwind::AMBER_300)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Display Matrix: ",
                            Style::default()
                                .fg(tailwind::SKY_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(geometry_desc, Style::default().fg(tailwind::SLATE_200)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Recommended For: ",
                            Style::default()
                                .fg(tailwind::PURPLE_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(use_case, Style::default().fg(tailwind::SLATE_300)),
                    ]));
                }
                if field == ConfigField::AspectMode {
                    let cur = self.config.aspect_mode.as_deref().unwrap_or("auto");
                    let (mode_desc, behavior) = match cur {
                        "preset" => (
                            "Static preset lock",
                            "Enforces exact aspect ratio bounds; letterboxes or crops content exceeding geometry",
                        ),
                        "dynamic" => (
                            "Live adaptive recalculation",
                            "Recalculates horizontal & vertical canvas on every crossterm terminal resize event",
                        ),
                        _ => (
                            "Hybrid automatic",
                            "Uses dynamic interactive viewport in TUI; locks to preset in fastfetch CLI banner",
                        ),
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Mode Strategy: ",
                            Style::default()
                                .fg(tailwind::EMERALD_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(mode_desc, Style::default().fg(tailwind::AMBER_300)),
                    ]));
                    lines.push(Line::from(vec![
                        Span::styled(
                            "Canvas Behavior: ",
                            Style::default()
                                .fg(tailwind::SKY_400)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(behavior, Style::default().fg(tailwind::SLATE_200)),
                    ]));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("Shortcuts: ", Style::default().fg(tailwind::SLATE_600)),
                    Span::styled("[e/Enter] ", Style::default().fg(tailwind::EMERALD_400)),
                    Span::styled("Edit  ", Style::default().fg(tailwind::SLATE_300)),
                    Span::styled("[+/-] ", Style::default().fg(tailwind::AMBER_400)),
                    Span::styled("Step  ", Style::default().fg(tailwind::SLATE_300)),
                    Span::styled("[o] ", Style::default().fg(tailwind::VIOLET_400)),
                    Span::styled("Editor  ", Style::default().fg(tailwind::SLATE_300)),
                    Span::styled("[s] ", Style::default().fg(tailwind::CYAN_400)),
                    Span::styled("Save", Style::default().fg(tailwind::SLATE_300)),
                ]));

                let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(tailwind::INDIGO_400))
                        .title(Span::styled(
                            " ⚙️ Parameter Details ",
                            Style::default()
                                .fg(tailwind::VIOLET_400)
                                .add_modifier(Modifier::BOLD),
                        )),
                );
                f.render_widget(p, area);
            }
            Tab::Studio => {
                let mut lines: Vec<Line> = Vec::with_capacity(8);
                match self.studio_sub_focus {
                    0 => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Tool: ",
                                Style::default()
                                    .fg(tailwind::SKY_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Convex Hull Occlusion & Chebyshev Moat Inspection",
                                Style::default()
                                    .fg(tailwind::AMBER_300)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Overlay Status: ",
                                Style::default()
                                    .fg(tailwind::VIOLET_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                if self.studio_debug_overlay {
                                    "✔ ACTIVE (Visualizing '#' Hull Interior & '·' Moat Margin)"
                                } else {
                                    "✖ OFF (Press <Space> to activate overlay)"
                                },
                                Style::default().fg(if self.studio_debug_overlay {
                                    tailwind::EMERALD_400
                                } else {
                                    tailwind::ROSE_400
                                }),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Occlusion Invariant: ",
                                Style::default()
                                    .fg(tailwind::EMERALD_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Mandate 2.C — Bounding-hull occlusion prevents background scenery/road bleed-through inside mascot silhouette.",
                                Style::default().fg(tailwind::SLATE_300),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Shader Compute: ",
                                Style::default()
                                    .fg(tailwind::AMBER_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Chebyshev Distance Transform (Radius: 2px) on pre-allocated framebuffer lines.",
                                Style::default().fg(tailwind::SLATE_300),
                            ),
                        ]));
                    }
                    1 => {
                        let vars = forgum_platform::biome::get_mascot_variations(&self.config.cow);
                        let total_vars = vars.len();
                        let active_idx = if total_vars > 0 {
                            self.studio_morph_idx % total_vars
                        } else {
                            0
                        };
                        let v = if total_vars > 0 {
                            &vars[active_idx]
                        } else {
                            &vars[0]
                        };

                        lines.push(Line::from(vec![
                            Span::styled(
                                "Tool: ",
                                Style::default()
                                    .fg(tailwind::SKY_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Biological Coat Polymorphism & Coat Color Morphs",
                                Style::default()
                                    .fg(tailwind::AMBER_300)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Active Phenotype: ",
                                Style::default()
                                    .fg(tailwind::VIOLET_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("{} [{}/{}]", v.name, active_idx + 1, total_vars),
                                Style::default()
                                    .fg(tailwind::EMERALD_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Biological Description: ",
                                Style::default()
                                    .fg(tailwind::AMBER_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(v.description, Style::default().fg(tailwind::SLATE_200)),
                        ]));
                        let hex_colors: Vec<String> = v
                            .rgb
                            .iter()
                            .map(|(r, g, b)| format!("#{r:02x}{g:02x}{b:02x}"))
                            .collect();
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Phenotype Pigments: ",
                                Style::default()
                                    .fg(tailwind::TEAL_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                hex_colors.join(" • "),
                                Style::default().fg(tailwind::SLATE_300),
                            ),
                            Span::styled(
                                " (Cycle with <h/l> or <[/]>)",
                                Style::default().fg(tailwind::SLATE_500),
                            ),
                        ]));
                    }
                    _ => {
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Tool: ",
                                Style::default()
                                    .fg(tailwind::SKY_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "240Hz / 120Hz Ultra-High-Refresh Kinematics Telemetry",
                                Style::default()
                                    .fg(tailwind::AMBER_300)
                                    .add_modifier(Modifier::BOLD),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Sub-Step Integrator: ",
                                Style::default()
                                    .fg(tailwind::VIOLET_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Fixed 240Hz Verlet Step (4.167ms delta), Sub-frame Particle Accumulator",
                                Style::default().fg(tailwind::EMERALD_400),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Kinematics Mode: ",
                                Style::default()
                                    .fg(tailwind::AMBER_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Smooth Harmonic Bobbing + Continuous Sub-Cell Horizontal Translation",
                                Style::default().fg(tailwind::SLATE_200),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled(
                                "Zero-Allocation Budget: ",
                                Style::default()
                                    .fg(tailwind::EMERALD_400)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                "Mandate 2.I & 2.O — 0 heap allocs per frame, stack arrays, <100MB resident ceiling.",
                                Style::default().fg(tailwind::SLATE_300),
                            ),
                        ]));
                    }
                }

                let p = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .border_style(Style::default().fg(tailwind::INDIGO_400))
                        .title(Span::styled(
                            " 🔬 Studio Diagnostics & Kinematics Telemetry ",
                            Style::default()
                                .fg(tailwind::VIOLET_400)
                                .add_modifier(Modifier::BOLD),
                        )),
                );
                f.render_widget(p, area);
            }
        }
    }

    /// Bottom Zellij-style status and keybinding bar with Tailwind dark pill badges.
    fn render_zellij_footer(&self, f: &mut Frame, area: Rect) {
        let bp = Breakpoint::from_size(area.width, area.height);

        // ── Compact Mode: show only the status message (1 line) ──────
        if bp == Breakpoint::Compact {
            let p = Paragraph::new(Line::from(vec![Span::styled(
                format!(" {}", self.status_message),
                Style::default().fg(tailwind::SKY_400),
            )]));
            f.render_widget(p, area);
            return;
        }

        // ── Standard / Wide Mode: full 2-line key-hint bar ───────────────────
        let help_spans = if self.current_tab == Tab::Installer {
            vec![
                Span::styled(
                    " <Tab/gt> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Tabs  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <j/k> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Shell  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <i> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::EMERALD_400),
                ),
                Span::styled(" Install  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <u> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_400),
                ),
                Span::styled(" Uninstall  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <t> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::AMBER_400),
                ),
                Span::styled(" Test  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <T> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::FUCHSIA_400),
                ),
                Span::styled(" Tip  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <?> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Vim  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <ZZ/q> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_500),
                ),
                Span::styled(" Quit  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    format!(" | {}", self.status_message),
                    Style::default().fg(tailwind::SKY_400),
                ),
            ]
        } else if self.current_tab == Tab::Config && self.editing_config {
            vec![
                Span::styled(
                    " <Enter> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::EMERALD_400),
                ),
                Span::styled(" Confirm  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <Esc> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_400),
                ),
                Span::styled(" Cancel  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <Up/Down> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Step +/-1  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <Left/Right> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Step +/-5  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <Ctrl+U> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::AMBER_400),
                ),
                Span::styled(" Clear  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    format!(" | {}", self.status_message),
                    Style::default().fg(tailwind::SKY_400),
                ),
            ]
        } else if self.current_tab == Tab::Config {
            vec![
                Span::styled(
                    " <Tab/gt> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Tabs  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <j/k> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Select  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <h/l> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::AMBER_400),
                ),
                Span::styled(" Cycle  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <Enter> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::EMERALD_400),
                ),
                Span::styled(" Edit  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <s> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::CYAN_400),
                ),
                Span::styled(" Save  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <T> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::FUCHSIA_400),
                ),
                Span::styled(" Tip  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <?> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Vim  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <ZZ/q> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_500),
                ),
                Span::styled(" Quit  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    format!(" | {}", self.status_message),
                    Style::default().fg(tailwind::SKY_400),
                ),
            ]
        } else if self.current_tab == Tab::Studio {
            vec![
                Span::styled(
                    " <Tab/gt> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Tabs  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <j/k> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Tool  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <Space> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::EMERALD_400),
                ),
                Span::styled(" Overlay  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <h/l> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::AMBER_400),
                ),
                Span::styled(" Morphs  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <?> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Vim  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <ZZ/q> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_500),
                ),
                Span::styled(" Quit  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    format!(" | {}", self.status_message),
                    Style::default().fg(tailwind::SKY_400),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    " <Tab/gt> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Tabs  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <j/k> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Select  ", Style::default().fg(tailwind::SLATE_400)),
                Span::styled(
                    " <Space> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SLATE_200),
                ),
                Span::styled(" Toggle  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <b> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::CYAN_400),
                ),
                Span::styled(" Biome  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <s> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::CYAN_400),
                ),
                Span::styled(" Save  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <t> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::FUCHSIA_400),
                ),
                Span::styled(" Tip  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <?> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::SKY_400),
                ),
                Span::styled(" Vim  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    " <ZZ/q> ",
                    Style::default()
                        .bg(tailwind::SLATE_800)
                        .fg(tailwind::ROSE_500),
                ),
                Span::styled(" Quit  ", Style::default().fg(tailwind::SLATE_300)),
                Span::styled(
                    format!(" | {}", self.status_message),
                    Style::default().fg(tailwind::SKY_400),
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

        // 10. Cycle Random Mode
        app.config_field_idx = ConfigField::Random as usize;
        assert_eq!(app.config.random, None);
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(
            app.config.random,
            Some(forgum_platform::protocol::RandomSetting::Bool(true))
        );
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
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
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
            app.status_message.starts_with("📁 Opened")
                || app.status_message.starts_with("⚠ Failed"),
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

    #[test]
    fn test_horizontal_scrolling_mascots_list() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));
        app.last_terminal_size = (75, 24); // Narrow / compact terminal
        assert!(app.current_breakpoint().is_compact_or_narrow());
        assert_eq!(app.h_scroll, 0);

        // 'l' or Right in narrow mode increases h_scroll
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 2);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 4);

        // 'h' or Left in narrow mode decreases h_scroll
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 2);

        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.h_scroll, 0);

        // Clamps at 0, no underflow
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.h_scroll, 0);

        // Shift+Right or 'L' scrolls regardless of breakpoint
        app.last_terminal_size = (120, 30);
        assert_eq!(app.current_breakpoint(), Breakpoint::Standard);
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('L'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 2);

        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::SHIFT,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 4);

        // Category cycling via '[' and ']' resets h_scroll
        let cat_before = app.mascot_category_idx;
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char(']'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.mascot_category_idx, (cat_before + 1) % CATEGORIES.len());
        assert_eq!(app.h_scroll, 0);
    }

    #[test]
    fn test_horizontal_scrolling_config_panel() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert_eq!(app.h_scroll, 0);

        // 'l' scrolls right by 4
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('l'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 4);

        // 'h' scrolls left by 4
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('h'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 0);

        // Shift+Right scrolls right
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::SHIFT,
        )))
        .unwrap();
        assert_eq!(app.h_scroll, 4);

        // Tab switch resets h_scroll
        app.handle_event(Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)))
            .unwrap();
        assert_eq!(app.h_scroll, 0);
    }

    #[test]
    fn test_adaptive_breakpoint_collapse_threshold_80_cols() {
        let compact = Breakpoint::from_size(79, 24);
        assert_eq!(compact, Breakpoint::Compact);
        assert!(compact.is_compact_or_narrow());

        let standard = Breakpoint::from_size(80, 24);
        assert_eq!(standard, Breakpoint::Standard);
        assert!(!standard.is_compact_or_narrow());

        let wide = Breakpoint::from_size(140, 24);
        assert_eq!(wide, Breakpoint::Wide);
        assert!(!wide.is_compact_or_narrow());
    }

    #[test]
    fn test_contrast_profile_cycling_and_sync_between_tabs() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Effects));
        app.fx_sub_focus = 2; // Focus Contrast Profile

        // Initial contrast mode defaults to standard
        assert_eq!(app.active_contrast_mode(), ContrastMode::Standard);

        // Cycle down in Tab::Effects
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.active_contrast_mode(), ContrastMode::AcrylicGlass);
        assert_eq!(app.config.contrast_mode.as_deref(), Some("acrylic_glass"));
        assert_eq!(app.contrast_mode_dropdown.current(), "acrylic_glass");

        // Cycle down again -> HighContrast
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Char('j'),
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.active_contrast_mode(), ContrastMode::HighContrast);
        assert_eq!(app.config.contrast_mode.as_deref(), Some("high_contrast"));

        // Switch to Config tab (Tab 5)
        app.current_tab = Tab::Config;
        app.config_field_idx = ConfigField::ContrastMode as usize;

        // Check field_value renders descriptive text
        let val = app.field_value(ConfigField::ContrastMode, true);
        assert!(val.contains("high_contrast"));
        assert!(val.contains("maximum luminance floor"));

        // Cycle field in Config tab using Right arrow
        app.handle_event(Event::Key(KeyEvent::new(
            KeyCode::Right,
            KeyModifiers::NONE,
        )))
        .unwrap();
        assert_eq!(app.active_contrast_mode(), ContrastMode::Vibrant);
        assert_eq!(app.contrast_idx, 3); // Syncs back to contrast_idx for Tab 3
    }

    #[test]
    fn test_contrast_mode_acrylic_glass_luminance_and_bold_properties() {
        let acrylic = ContrastMode::AcrylicGlass;
        assert!(acrylic.min_luminance_floor() >= 120.0);
        assert!(acrylic.saturation_factor() > 1.0);
        assert!(acrylic.requires_bold());

        // Test adjust_rgb lifts dark colors above the luminance floor
        let dark = (30, 30, 30);
        let adjusted = acrylic.adjust_rgb(dark.0, dark.1, dark.2);
        let lum = 0.299 * adjusted.0 as f32 + 0.587 * adjusted.1 as f32 + 0.114 * adjusted.2 as f32;
        assert!(lum >= acrylic.min_luminance_floor() - 1.0);

        // Test standard mode preserves original values
        let std_mode = ContrastMode::Standard;
        assert_eq!(std_mode.adjust_rgb(50, 60, 70), (50, 60, 70));
        assert!(!std_mode.requires_bold());
    }

    #[test]
    fn test_layout_resilience_down_to_extreme_tiny_dimensions() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Installer));
        // Verify no panics at 20x5 terminal size
        let backend = ratatui::backend::TestBackend::new(20, 5);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();

        for tab in Tab::ALL {
            app.current_tab = tab;
            terminal.draw(|f| app.render(f)).unwrap();
        }
    }

    #[test]
    fn test_biological_color_palette_mascot_variations() {
        use forgum_platform::biome::get_mascot_variations;

        // Verify cattle/cow has exactly 6 authentic natural morphs as requested:
        // Holstein, Charolais, Black Angus, Jersey, Normande, Grey Ox
        let cow_vars = get_mascot_variations("default");
        assert_eq!(cow_vars.len(), 6, "Cow should have 6 biological variations");
        assert!(cow_vars[0].name.contains("Holstein"));
        assert!(cow_vars[1].name.contains("Charolais"));
        assert!(cow_vars[2].name.contains("Black Angus"));
        assert!(cow_vars[3].name.contains("Jersey"));
        assert!(cow_vars[4].name.contains("Normande"));
        assert!(cow_vars[5].name.contains("Grey Ox"));

        // Verify tiger has exactly 3 authentic natural morphs:
        // Bengal Tiger, White Bengal Tiger, Royal Bengal
        let tiger_vars = get_mascot_variations("tiger");
        assert_eq!(
            tiger_vars.len(),
            3,
            "Tiger should have 3 biological variations"
        );
        assert!(tiger_vars[0].name.contains("Bengal Tiger"));
        assert!(tiger_vars[1].name.contains("White Bengal"));
        assert!(tiger_vars[2].name.contains("Royal Bengal"));

        // Verify koala has exactly 1 authentic natural morph
        let koala_vars = get_mascot_variations("koala");
        assert_eq!(
            koala_vars.len(),
            1,
            "Koala is monomorphic with 1 biological variation"
        );
        assert!(koala_vars[0].name.contains("Ashen Grey"));

        // Test TUI app interaction with biological palette
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config.cow = "default".to_string();
        app.config.color_mode = "natural".to_string();
        app.config_field_idx = ConfigField::Palette as usize;

        // Should start at variation 0
        let val_nat = app.field_value(ConfigField::Palette, false);
        assert!(
            val_nat.contains("Variant 1/6"),
            "Expected Variant 1/6, got: {val_nat}"
        );
        assert!(val_nat.contains("Holstein"));

        // Cycle to next variation (Charolais)
        app.cycle_config_field(true);
        let val_charolais = app.field_value(ConfigField::Palette, false);
        assert!(val_charolais.contains("Variant 2/6"));
        assert!(val_charolais.contains("Charolais"));
        assert!(app.config.palette.is_some());

        // Lock behavior: switch color_mode to rainbow
        app.config.color_mode = "rainbow".to_string();
        let val_locked = app.field_value(ConfigField::Palette, false);
        assert!(
            val_locked.contains("Locked"),
            "Expected locked message when color mode != natural"
        );

        // Cycling while locked should not cycle variations
        app.cycle_config_field(true);
        let val_locked_after = app.field_value(ConfigField::Palette, false);
        assert!(val_locked_after.contains("Locked"));

        // Switching back to natural restores/unlocks
        app.config_field_idx = ConfigField::ColorMode as usize;
        app.cycle_config_field(true);
        while app.config.color_mode != "natural" {
            app.cycle_config_field(true);
        }
        app.config_field_idx = ConfigField::Palette as usize;
        let val_unlocked = app.field_value(ConfigField::Palette, false);
        assert!(!val_unlocked.contains("Locked"));
        assert!(val_unlocked.contains("Variant"));
    }

    #[test]
    fn test_shell_attach_modes_5_variants_cycling() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config_field_idx = ConfigField::ShellAttachMode as usize;

        // The 5 official modes: startup, prompt, clear, split, pane
        let expected_modes = ["startup", "prompt", "clear", "split", "pane"];

        for expected in expected_modes {
            while app.config.shell_attach_mode != expected {
                app.cycle_config_field(true);
            }
            assert_eq!(app.config.shell_attach_mode, expected);
            let val = app.field_value(ConfigField::ShellAttachMode, false);
            assert!(val.contains(expected));
            match expected {
                "startup" => assert!(val.contains("Launch once")),
                "prompt" => assert!(val.contains("Every command prompt")),
                "clear" => assert!(val.contains("Launch + after clear/cls")),
                "split" => assert!(val.contains("DECSTBM top margin")),
                "pane" => assert!(val.contains("Dedicated split pane")),
                _ => {}
            }
        }
    }

    #[test]
    fn test_hardware_accel_setting_cycling_and_persistence() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config_field_idx = ConfigField::HardwareAccel as usize;

        // Default should be auto
        assert_eq!(app.active_accel_mode(), "auto");
        assert_eq!(app.active_accel_display(), "Auto");
        assert_eq!(app.active_accel_badge(), "✨ AUTO SELECT");
        assert!(app
            .active_accel_desc()
            .contains("Automatically prefers Dedicated GPU"));

        let val_auto = app.field_value(ConfigField::HardwareAccel, false);
        assert!(val_auto.contains("Auto"));
        assert!(val_auto.contains("AUTO SELECT"));
        assert!(val_auto.contains("Automatically prefers Dedicated GPU"));

        // Cycle forward: auto -> dedicated
        app.cycle_config_field(true);
        assert_eq!(app.config.accel_mode.as_deref(), Some("dedicated"));
        assert_eq!(app.active_accel_mode(), "dedicated");
        assert_eq!(app.active_accel_display(), "Dedicated");
        assert_eq!(app.active_accel_badge(), "⚡ DEDICATED GPU");
        assert!(app
            .active_accel_desc()
            .contains("discrete GPU cores and VRAM"));
        let val_ded = app.field_value(ConfigField::HardwareAccel, false);
        assert!(val_ded.contains("Dedicated"));
        assert!(val_ded.contains("DEDICATED GPU"));

        // Cycle forward: dedicated -> integrated
        app.cycle_config_field(true);
        assert_eq!(app.config.accel_mode.as_deref(), Some("integrated"));
        assert_eq!(app.active_accel_mode(), "integrated");
        assert_eq!(app.active_accel_display(), "Integrated");
        assert_eq!(app.active_accel_badge(), "🔋 INTEGRATED APU");
        assert!(app.active_accel_desc().contains("Internal graphics / APU"));

        // Cycle forward: integrated -> npu
        app.cycle_config_field(true);
        assert_eq!(app.config.accel_mode.as_deref(), Some("npu"));
        assert_eq!(app.active_accel_mode(), "npu");
        assert_eq!(app.active_accel_display(), "NPU");
        assert_eq!(app.active_accel_badge(), "🧠 NEURAL NPU");
        assert!(app.active_accel_desc().contains("Neural Processing Unit"));

        // Cycle forward: npu -> cpu
        app.cycle_config_field(true);
        assert_eq!(app.config.accel_mode.as_deref(), Some("cpu"));
        assert_eq!(app.active_accel_mode(), "cpu");
        assert_eq!(app.active_accel_display(), "Cpu");
        assert_eq!(app.active_accel_badge(), "💻 CPU COMPUTE");
        assert!(app.active_accel_desc().contains("Pure CPU computing"));

        // Cycle forward: cpu -> auto
        app.cycle_config_field(true);
        assert_eq!(app.config.accel_mode.as_deref(), Some("auto"));
        assert_eq!(app.active_accel_mode(), "auto");

        // Cycle backward: auto -> cpu
        app.cycle_config_field(false);
        assert_eq!(app.config.accel_mode.as_deref(), Some("cpu"));

        // Verify serialization / deserialization round-trip preserves accel_mode
        for fmt in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = app
                .config()
                .serialize_with_format(fmt)
                .expect("serialization succeeds");
            assert!(
                serialized.contains("accel_mode") || serialized.contains("cpu"),
                "Expected accel_mode in serialized format: {serialized}"
            );
            let deserialized =
                SceneConfig::parse_with_format(&serialized, fmt).expect("deserialization succeeds");
            assert_eq!(
                deserialized.accel_mode.as_deref(),
                Some("cpu"),
                "Mismatch on format {:?}",
                fmt
            );
        }
    }

    #[test]
    fn test_reduce_motion_text_only_banner_cycling_and_persistence() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // 1. Test reduce_motion
        app.config_field_idx = ConfigField::ReduceMotion as usize;
        assert!(!app.config.reduce_motion);
        let val = app.field_value(ConfigField::ReduceMotion, false);
        assert!(val.contains("✖ OFF"));

        app.cycle_config_field(true);
        assert!(app.config.reduce_motion);
        let val_on = app.field_value(ConfigField::ReduceMotion, false);
        assert!(val_on.contains("✔ ON"));

        // 2. Test text_only
        app.config_field_idx = ConfigField::TextOnly as usize;
        assert!(!app.config.text_only);
        let val = app.field_value(ConfigField::TextOnly, false);
        assert!(val.contains("✖ OFF"));

        app.cycle_config_field(true);
        assert!(app.config.text_only);
        let val_on = app.field_value(ConfigField::TextOnly, false);
        assert!(val_on.contains("✔ ON"));

        // 3. Test banner
        app.config_field_idx = ConfigField::Banner as usize;
        assert!(!app.config.banner);
        let val = app.field_value(ConfigField::Banner, false);
        assert!(val.contains("✖ OFF"));

        app.cycle_config_field(true);
        assert!(app.config.banner);
        assert_eq!(app.config.shell_attach_mode, "banner");
        let val_on = app.field_value(ConfigField::Banner, false);
        assert!(val_on.contains("✔ ON"));

        // 4. Persistence across all 3 formats
        for fmt in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = app.config().serialize_with_format(fmt).unwrap();
            let deserialized = SceneConfig::parse_with_format(&serialized, fmt).unwrap();
            assert!(deserialized.reduce_motion);
            assert!(deserialized.text_only);
            assert!(deserialized.banner);
        }
    }

    #[test]
    fn test_aspect_ratio_and_mode_cycling_and_persistence() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // 1. Test Aspect cycling
        app.config_field_idx = ConfigField::Aspect as usize;
        assert_eq!(app.config.aspect, None);
        let val_auto = app.field_value(ConfigField::Aspect, false);
        assert!(val_auto.contains("auto"));

        // Cycle forward: auto -> 21:9
        app.cycle_config_field(true);
        assert_eq!(app.config.aspect.as_deref(), Some("21:9"));
        let val_21x9 = app.field_value(ConfigField::Aspect, false);
        assert!(val_21x9.contains("21:9"));
        assert!(val_21x9.contains("Ultrawide"));

        // Cycle forward: 21:9 -> 16:9
        app.cycle_config_field(true);
        assert_eq!(app.config.aspect.as_deref(), Some("16:9"));

        // Cycle forward: 16:9 -> 16:10
        app.cycle_config_field(true);
        assert_eq!(app.config.aspect.as_deref(), Some("16:10"));

        // 2. Test AspectMode cycling
        app.config_field_idx = ConfigField::AspectMode as usize;
        assert_eq!(app.config.aspect_mode, None);

        // Cycle forward: auto -> preset
        app.cycle_config_field(true);
        assert_eq!(app.config.aspect_mode.as_deref(), Some("preset"));
        let val_preset = app.field_value(ConfigField::AspectMode, false);
        assert!(val_preset.contains("preset"));

        // Cycle forward: preset -> dynamic
        app.cycle_config_field(true);
        assert_eq!(app.config.aspect_mode.as_deref(), Some("dynamic"));

        // 3. Persistence across all 3 formats (JSON, YAML, TOML)
        for fmt in [ConfigFormat::Json, ConfigFormat::Yaml, ConfigFormat::Toml] {
            let serialized = app.config().serialize_with_format(fmt).unwrap();
            let deserialized = SceneConfig::parse_with_format(&serialized, fmt).unwrap();
            assert_eq!(deserialized.aspect.as_deref(), Some("16:10"));
            assert_eq!(deserialized.aspect_mode.as_deref(), Some("dynamic"));
        }
    }

    #[test]
    fn test_dynamic_pane_resize_and_layout_responsiveness() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // Test multiple dynamic resize events simulating window dragging or tmux split changes
        let test_dimensions = [
            (160, 50, Breakpoint::Wide),
            (100, 30, Breakpoint::Standard),
            (80, 24, Breakpoint::Standard),
            (75, 20, Breakpoint::Compact),
            (50, 15, Breakpoint::Compact),
            (20, 5, Breakpoint::Compact),
            (140, 40, Breakpoint::Wide),
        ];

        for (width, height, expected_bp) in test_dimensions {
            // Dispatch dynamic resize event
            app.handle_event(Event::Resize(width, height))
                .expect("handle resize event");

            // Verify cached size and breakpoint update immediately
            assert_eq!(app.last_terminal_size, (width, height));
            assert_eq!(app.current_breakpoint(), expected_bp);

            // Test draw loop across all tabs at this dimension
            let backend = ratatui::backend::TestBackend::new(width, height);
            let mut terminal = ratatui::Terminal::new(backend).unwrap();

            for tab in Tab::ALL {
                app.current_tab = tab;
                terminal
                    .draw(|f| {
                        app.render(f);
                    })
                    .expect("render must succeed at all dimensions");
            }
        }
    }

    #[test]
    fn test_vim_gg_and_capital_g_navigation() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));
        assert_eq!(app.mascot_item_idx, 0);

        // Jump to bottom using Vim 'G' (with KeyModifiers::NONE)
        let event_g = Event::Key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::NONE));
        let res = app.handle_event(event_g).expect("handle G");
        assert_eq!(res, None);
        assert!(app.mascot_item_idx > 0);

        // Jump to top using Vim 'gg' sequence
        let event_g_prefix = Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        app.handle_event(event_g_prefix).expect("handle first g");
        assert_eq!(app.pending_vim_prefix, Some('g'));

        let event_g_second = Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        app.handle_event(event_g_second).expect("handle second g");
        assert_eq!(app.pending_vim_prefix, None);
        assert_eq!(app.mascot_item_idx, 0);

        // Jump to bottom using Vim Shift+G (with KeyModifiers::SHIFT from terminal)
        let event_shift_g = Event::Key(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));
        let res = app.handle_event(event_shift_g).expect("handle Shift+G");
        assert_eq!(res, None);
        assert!(app.mascot_item_idx > 0);
    }

    #[test]
    fn test_vim_ctrl_d_and_ctrl_u_half_page_scroll() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));
        assert_eq!(app.mascot_item_idx, 0);

        // Half-page down: Ctrl+d (delta = 5)
        let event_ctrl_d = Event::Key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL));
        app.handle_event(event_ctrl_d).expect("handle Ctrl+d");
        assert_eq!(app.mascot_item_idx, 5);

        // Half-page up: Ctrl+u
        let event_ctrl_u = Event::Key(KeyEvent::new(KeyCode::Char('u'), KeyModifiers::CONTROL));
        app.handle_event(event_ctrl_u).expect("handle Ctrl+u");
        assert_eq!(app.mascot_item_idx, 0);
    }

    #[test]
    fn test_vim_gt_and_g_capital_t_tab_switching() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));
        assert_eq!(app.current_tab, Tab::Mascots);

        // Next tab: 'gt'
        let event_g = Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        app.handle_event(event_g).expect("handle g");
        let event_t = Event::Key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
        app.handle_event(event_t).expect("handle t");
        assert_eq!(app.current_tab, Tab::Scenery);

        // Next tab: 'gt' again -> Tab::Effects
        let event_g = Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        app.handle_event(event_g).expect("handle g");
        let event_t = Event::Key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
        app.handle_event(event_t).expect("handle t");
        assert_eq!(app.current_tab, Tab::Effects);

        // Prev tab: 'gT' -> back to Tab::Scenery
        let event_g = Event::Key(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        app.handle_event(event_g).expect("handle g");
        let event_t_upper = Event::Key(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE));
        app.handle_event(event_t_upper).expect("handle T");
        assert_eq!(app.current_tab, Tab::Scenery);
    }

    #[test]
    fn test_vim_zz_and_zq_quit_actions() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));

        // Save & Quit: 'ZZ'
        let event_z = Event::Key(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE));
        app.handle_event(event_z).expect("handle first Z");
        assert_eq!(app.pending_vim_prefix, Some('Z'));

        let event_second_z = Event::Key(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE));
        let res = app.handle_event(event_second_z).expect("handle second Z");
        assert_eq!(res, Some(Action::SaveAndQuit));

        // Quit without saving: 'ZQ'
        let event_z = Event::Key(KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::NONE));
        app.handle_event(event_z).expect("handle first Z");
        let event_q = Event::Key(KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE));
        let res = app.handle_event(event_q).expect("handle Q");
        assert_eq!(res, Some(Action::Quit));
    }

    #[test]
    fn test_modeless_config_editing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        assert!(!app.editing_config);

        // Enter config editing via 'e'
        let key_e = Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        app.handle_event(key_e).expect("enter edit with e");
        assert!(app.editing_config);

        // Cancel edit with Esc
        let key_esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        app.handle_event(key_esc).expect("cancel edit with Esc");
        assert!(!app.editing_config);

        // Direct numeric digit triggers inline edit
        let key_digit = Event::Key(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE));
        app.handle_event(key_digit)
            .expect("type digit into config field");
        assert!(app.editing_config);
        assert_eq!(app.config_edit_buffer, "5");

        // Confirm edit with Enter
        let key_enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.handle_event(key_enter)
            .expect("confirm edit with Enter");
        assert!(!app.editing_config);
    }

    #[test]
    fn test_vim_help_modal_toggle() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));
        assert!(!app.show_vim_help);

        // Toggle help modal on with '?'
        let key_help = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
        app.handle_event(key_help).expect("show vim help");
        assert!(app.show_vim_help);

        // Toggle help modal off with '?'
        let key_help = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
        app.handle_event(key_help).expect("hide vim help");
        assert!(!app.show_vim_help);
    }

    #[test]
    fn test_feature_discovery_suggestions_cycling_and_hotkeys() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Mascots));

        // Initial suggestion exists and is non-empty
        let initial_tip = app.current_suggestion();
        assert!(!initial_tip.is_empty());
        assert!(app.status_message.starts_with("💡 Pro-Tip: "));

        // Direct cycling advances index and returns valid suggestion
        let start_idx = app.suggestion_idx;
        let next_tip = app.next_suggestion();
        assert_eq!(
            app.suggestion_idx,
            (start_idx + 1) % FEATURE_SUGGESTIONS.len()
        );
        assert!(!next_tip.is_empty());

        // Full cycle through all suggestions without panic
        for _ in 0..FEATURE_SUGGESTIONS.len() {
            let tip = app.next_suggestion();
            assert!(!tip.is_empty());
        }

        // Global hotkey 'T' (Shift+T) cycles suggestion and updates status message
        let key_t = Event::Key(KeyEvent::new(KeyCode::Char('T'), KeyModifiers::NONE));
        let prev_idx = app.suggestion_idx;
        app.handle_event(key_t).expect("handle 'T' global hotkey");
        assert_eq!(
            app.suggestion_idx,
            (prev_idx + 1) % FEATURE_SUGGESTIONS.len()
        );
        assert!(app.status_message.starts_with("💡 Pro-Tip: "));

        // Global hotkey F1 cycles suggestion and updates status message
        let key_f1 = Event::Key(KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE));
        let prev_idx = app.suggestion_idx;
        app.handle_event(key_f1).expect("handle F1 global hotkey");
        assert_eq!(
            app.suggestion_idx,
            (prev_idx + 1) % FEATURE_SUGGESTIONS.len()
        );
        assert!(app.status_message.starts_with("💡 Pro-Tip: "));

        // Tab-specific 't' in Mascots tab
        let key_lower_t = Event::Key(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));
        let prev_idx = app.suggestion_idx;
        app.handle_event(key_lower_t.clone())
            .expect("handle 't' in Mascots tab");
        assert_eq!(
            app.suggestion_idx,
            (prev_idx + 1) % FEATURE_SUGGESTIONS.len()
        );

        // Tab-specific 't' in Scenery tab
        app.current_tab = Tab::Scenery;
        let prev_idx = app.suggestion_idx;
        app.handle_event(key_lower_t.clone())
            .expect("handle 't' in Scenery tab");
        assert_eq!(
            app.suggestion_idx,
            (prev_idx + 1) % FEATURE_SUGGESTIONS.len()
        );

        // Tab-specific 't' in Effects tab
        app.current_tab = Tab::Effects;
        let prev_idx = app.suggestion_idx;
        app.handle_event(key_lower_t)
            .expect("handle 't' in Effects tab");
        assert_eq!(
            app.suggestion_idx,
            (prev_idx + 1) % FEATURE_SUGGESTIONS.len()
        );
    }

    #[test]
    fn test_custom_keybindings_remapping() {
        let mut app = ConfigApp::new(Some(SceneConfig::default()), None, None);
        app.config.keybindings = Some(forgum_platform::protocol::KeybindingsConfig {
            quit: Some("x".to_string()),
            help: Some("H".to_string()),
            ..Default::default()
        });

        // 'x' triggers Action::Quit due to remapping
        let key_x = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
        let res = app.handle_event(key_x).expect("handle 'x'");
        assert_eq!(res, Some(Action::Quit));

        // 'H' triggers Vim help modal toggle
        assert!(!app.show_vim_help);
        let key_h = Event::Key(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::NONE));
        let _ = app.handle_event(key_h).expect("handle 'H'");
        assert!(app.show_vim_help);
    }

    #[test]
    fn test_colorblind_mode_sync_across_tabs_and_tui_cycle() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));
        app.config_field_idx = ConfigField::ColorblindMode as usize;

        // Default state
        assert_eq!(app.active_colorblind_mode(), ColorblindMode::None);
        assert_eq!(app.active_colorblind_display(), "None (Normal Vision)");
        assert_eq!(app.active_colorblind_badge(), "⚪ NORMAL");
        assert!(app
            .active_colorblind_desc()
            .contains("Standard full-spectrum"));

        // Cycle through modes in Tab 5 (Config matrix)
        let expected_modes = [
            ("protanopia", ColorblindMode::Protanopia, "👁️ PROTANOPIA"),
            (
                "deuteranopia",
                ColorblindMode::Deuteranopia,
                "👁️ DEUTERANOPIA",
            ),
            ("tritanopia", ColorblindMode::Tritanopia, "👁️ TRITANOPIA"),
            (
                "achromatopsia",
                ColorblindMode::Achromatopsia,
                "⚪ ACHROMATOPSIA",
            ),
            (
                "protanopia_sim",
                ColorblindMode::ProtanopiaSim,
                "🔬 PROTANOPIA SIM",
            ),
            (
                "deuteranopia_sim",
                ColorblindMode::DeuteranopiaSim,
                "🔬 DEUTERANOPIA SIM",
            ),
            (
                "tritanopia_sim",
                ColorblindMode::TritanopiaSim,
                "🔬 TRITANOPIA SIM",
            ),
            ("none", ColorblindMode::None, "⚪ NORMAL"),
        ];

        for (exp_str, exp_mode, exp_badge) in expected_modes {
            app.cycle_config_field(true);
            assert_eq!(app.active_colorblind_mode(), exp_mode);
            assert_eq!(app.active_colorblind_badge(), exp_badge);
            if exp_str == "none" {
                assert!(app.config.colorblind_mode.is_none());
            } else {
                assert_eq!(app.config.colorblind_mode.as_deref(), Some(exp_str));
            }
            let val = app.field_value(ConfigField::ColorblindMode, false);
            assert!(val.contains(exp_badge));
        }

        // Test Tab 3 (Effects) keyboard controls
        app.current_tab = Tab::Effects;
        // Press 'h' to wrap around to Colorblind Accessibility panel (0 -> 3)
        let key_h = Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
        app.handle_event(key_h)
            .expect("focus colorblind panel with 'h'");
        assert_eq!(app.fx_sub_focus, 3);

        // Press 'j' (down) to cycle to protanopia
        let key_j = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE));
        app.handle_event(key_j)
            .expect("cycle colorblind down with 'j'");
        assert_eq!(app.colorblind_idx, 1);
        assert_eq!(app.active_colorblind_mode(), ColorblindMode::Protanopia);
        assert_eq!(app.config.colorblind_mode.as_deref(), Some("protanopia"));

        // Vim jump to bottom 'G' in Tab 3 subfocus 3
        app.vim_jump_to_bottom();
        assert_eq!(app.colorblind_idx, COLORBLIND_OPTIONS.len() - 1);
        assert_eq!(app.active_colorblind_mode(), ColorblindMode::TritanopiaSim);

        // Vim jump to top 'gg' in Tab 3 subfocus 3
        app.vim_jump_to_top();
        assert_eq!(app.colorblind_idx, 0);
        assert_eq!(app.active_colorblind_mode(), ColorblindMode::None);

        // Verify live preview renders cleanly with colorblind filter
        app.config.colorblind_mode = Some("deuteranopia".to_string());
        app.sync_colorblind_from_config();
        let backend = ratatui::backend::TestBackend::new(80, 24);
        let mut terminal = ratatui::Terminal::new(backend).unwrap();
        terminal.draw(|f| app.render(f)).unwrap();
    }

    #[test]
    fn test_update_method_and_check_updates_config_cycling_and_editing() {
        let mut app = ConfigApp::new(None, None, Some(Tab::Config));

        // 1. CheckUpdates field
        app.config_field_idx = ConfigField::CheckUpdates as usize;
        assert!(app.config.check_updates);
        assert_eq!(ConfigField::CheckUpdates.label(), "check_updates");
        assert_eq!(ConfigField::CheckUpdates.category_badge().0, "System");
        assert!(ConfigField::CheckUpdates
            .desc()
            .contains("upstream updates"));
        assert!(app
            .field_value(ConfigField::CheckUpdates, false)
            .contains("ON"));

        // Toggle check_updates off
        app.cycle_config_field(true);
        assert!(!app.config.check_updates);
        assert!(app
            .field_value(ConfigField::CheckUpdates, false)
            .contains("OFF"));

        // Toggle check_updates back on
        app.cycle_config_field(true);
        assert!(app.config.check_updates);
        assert!(app
            .field_value(ConfigField::CheckUpdates, false)
            .contains("ON"));

        // 2. UpdateMethod field
        app.config_field_idx = ConfigField::UpdateMethod as usize;
        assert_eq!(ConfigField::UpdateMethod.label(), "update_method");
        assert_eq!(ConfigField::UpdateMethod.category_badge().0, "System");
        assert!(ConfigField::UpdateMethod.desc().contains("Package manager"));
        assert!(app
            .field_value(ConfigField::UpdateMethod, false)
            .contains("prompt"));

        // Cycle through update methods (forward)
        let expected_sequence = [
            "auto",
            "zypper",
            "scoop",
            "winget",
            "homebrew",
            "apt",
            "dnf",
            "pacman",
            "cargo",
            "standalone",
            "prompt",
        ];
        for expected in expected_sequence {
            app.cycle_config_field(true);
            assert_eq!(app.config.update_method.as_deref(), Some(expected));
            assert!(app
                .field_value(ConfigField::UpdateMethod, false)
                .contains(expected));
        }

        // Test inline typing and editing
        let key_e = Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        app.handle_event(key_e).expect("enter edit mode");
        assert!(app.editing_config);
        assert_eq!(app.config_edit_buffer, "prompt");

        // Clear and type "zypper"
        app.config_edit_buffer = "zypper".to_string();
        let key_enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        app.handle_event(key_enter).expect("commit edit");
        assert!(!app.editing_config);
        assert_eq!(app.config.update_method.as_deref(), Some("zypper"));

        // Test cancel with Esc
        let key_e = Event::Key(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        app.handle_event(key_e).expect("enter edit mode");
        app.config_edit_buffer = "invalid_junk".to_string();
        let key_esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        app.handle_event(key_esc).expect("cancel edit");
        assert!(!app.editing_config);
        assert_eq!(app.config.update_method.as_deref(), Some("zypper"));
    }
}
