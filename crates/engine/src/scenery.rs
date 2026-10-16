//! Per-animal thematic scenery engine.
//!
//! Generates layered procedural ASCII/ANSI backdrops (Mountain, Road, Environment)
//! tailored to each of the 106 creatures or customized via CLI flags.

use crate::dna::BaseAnim;
use crate::framebuffer::{Cell, Color, FrameBuffer};
use forgum_platform::AspectRatioFormat;

/// Mountain and horizon scenery styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MountainStyle {
    #[default]
    None,
    Hills,
    Peaks,
    Volcano,
    Iceberg,
    Skyline,
    Seamount,
    Plateau,
    Crater,
    Gothic,
    Castle,
    Garden,
}

impl MountainStyle {
    pub const ALL_NON_NONE: &'static [MountainStyle] = &[
        Self::Hills,
        Self::Peaks,
        Self::Volcano,
        Self::Iceberg,
        Self::Skyline,
        Self::Seamount,
        Self::Plateau,
        Self::Crater,
        Self::Gothic,
        Self::Castle,
        Self::Garden,
    ];

    pub fn random() -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        *Self::ALL_NON_NONE.choose(&mut rng).unwrap_or(&Self::Hills)
    }

    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("random") {
            return Self::random();
        }
        match s.trim().to_ascii_lowercase().as_str() {
            "hills" | "hill" => Self::Hills,
            "peaks" | "peak" | "mountain" | "mountains" => Self::Peaks,
            "volcano" | "volcanoes" | "magma" => Self::Volcano,
            "iceberg" | "ice" | "glacier" => Self::Iceberg,
            "skyline" | "city" | "towers" | "urban" => Self::Skyline,
            "seamount" | "sea" | "ocean" | "trench" => Self::Seamount,
            "plateau" | "mesa" | "canyon" => Self::Plateau,
            "crater" | "moon" | "lunar" => Self::Crater,
            "gothic" | "spire" | "spires" => Self::Gothic,
            "castle" | "fortress" | "ramparts" => Self::Castle,
            "garden" | "botanical" | "hedges" => Self::Garden,
            "none" | "off" | "empty" => Self::None,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Hills => "hills",
            Self::Peaks => "peaks",
            Self::Volcano => "volcano",
            Self::Iceberg => "iceberg",
            Self::Skyline => "skyline",
            Self::Seamount => "seamount",
            Self::Plateau => "plateau",
            Self::Crater => "crater",
            Self::Gothic => "gothic",
            Self::Castle => "castle",
            Self::Garden => "garden",
        }
    }
}

/// Road and ground baseline scenery styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoadStyle {
    #[default]
    None,
    Dirt,
    Cobblestone,
    Magma,
    Ice,
    Seabed,
    Sidewalk,
    Roof,
    Grid,
    Crypt,
    Savanna,
    Mud,
    Tracks,
    Checkerboard,
}

impl RoadStyle {
    pub const ALL_NON_NONE: &'static [RoadStyle] = &[
        Self::Dirt,
        Self::Cobblestone,
        Self::Magma,
        Self::Ice,
        Self::Seabed,
        Self::Sidewalk,
        Self::Roof,
        Self::Grid,
        Self::Crypt,
        Self::Savanna,
        Self::Mud,
        Self::Tracks,
        Self::Checkerboard,
    ];

    pub fn random() -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        *Self::ALL_NON_NONE.choose(&mut rng).unwrap_or(&Self::Dirt)
    }

    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("random") {
            return Self::random();
        }
        match s.trim().to_ascii_lowercase().as_str() {
            "dirt" | "trail" | "earth" | "ground" => Self::Dirt,
            "cobblestone" | "cobble" | "stone" => Self::Cobblestone,
            "magma" | "lava" | "fire" => Self::Magma,
            "ice" | "snow" | "glacier" => Self::Ice,
            "seabed" | "sea" | "ocean" | "sand" => Self::Seabed,
            "sidewalk" | "pavement" | "street" | "concrete" => Self::Sidewalk,
            "roof" | "rooftop" | "tiles" => Self::Roof,
            "grid" | "matrix" => Self::Grid,
            "crypt" | "catacomb" | "tomb" => Self::Crypt,
            "savanna" | "clay" | "desert" => Self::Savanna,
            "mud" | "marsh" => Self::Mud,
            "tracks" | "rail" | "railroad" | "train" => Self::Tracks,
            "checkerboard" | "checker" | "board" => Self::Checkerboard,
            "none" | "off" | "empty" => Self::None,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Dirt => "dirt",
            Self::Cobblestone => "cobblestone",
            Self::Magma => "magma",
            Self::Ice => "ice",
            Self::Seabed => "seabed",
            Self::Sidewalk => "sidewalk",
            Self::Roof => "roof",
            Self::Grid => "grid",
            Self::Crypt => "crypt",
            Self::Savanna => "savanna",
            Self::Mud => "mud",
            Self::Tracks => "tracks",
            Self::Checkerboard => "checkerboard",
        }
    }
}

/// Atmospheric environment backdrop scenery styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvironmentStyle {
    #[default]
    None,
    Pasture,
    Inferno,
    Ocean,
    Arctic,
    City,
    Forest,
    Savanna,
    Swamp,
    Space,
    Cyber,
    Graveyard,
    Jurassic,
    Hive,
    Throne,
}

impl EnvironmentStyle {
    pub const ALL_NON_NONE: &'static [EnvironmentStyle] = &[
        Self::Pasture,
        Self::Inferno,
        Self::Ocean,
        Self::Arctic,
        Self::City,
        Self::Forest,
        Self::Savanna,
        Self::Swamp,
        Self::Space,
        Self::Cyber,
        Self::Graveyard,
        Self::Jurassic,
        Self::Hive,
        Self::Throne,
    ];

    pub fn random() -> Self {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        *Self::ALL_NON_NONE
            .choose(&mut rng)
            .unwrap_or(&Self::Pasture)
    }

    pub fn parse(s: &str) -> Self {
        if s.trim().eq_ignore_ascii_case("random") {
            return Self::random();
        }
        match s.trim().to_ascii_lowercase().as_str() {
            "pasture" | "meadow" | "grass" | "field" => Self::Pasture,
            "inferno" | "fire" | "flame" | "embers" | "lava" => Self::Inferno,
            "ocean" | "sea" | "water" | "bubbles" => Self::Ocean,
            "arctic" | "snow" | "polar" | "blizzard" | "ice" => Self::Arctic,
            "city" | "urban" | "neon" => Self::City,
            "forest" | "woods" | "trees" | "leaves" => Self::Forest,
            "savanna" | "desert" | "safari" | "plains" => Self::Savanna,
            "swamp" | "marsh" | "bayou" => Self::Swamp,
            "space" | "stars" | "cosmos" | "nebula" => Self::Space,
            "cyber" | "matrix" | "digital" => Self::Cyber,
            "graveyard" | "cemetery" | "haunted" | "spooky" | "ghost" => Self::Graveyard,
            "jurassic" | "dino" | "prehistoric" | "jungle" => Self::Jurassic,
            "hive" | "bees" | "alien" => Self::Hive,
            "throne" | "castle" | "gold" | "royal" => Self::Throne,
            "none" | "off" | "clear" => Self::None,
            _ => Self::None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Pasture => "pasture",
            Self::Inferno => "inferno",
            Self::Ocean => "ocean",
            Self::Arctic => "arctic",
            Self::City => "city",
            Self::Forest => "forest",
            Self::Savanna => "savanna",
            Self::Swamp => "swamp",
            Self::Space => "space",
            Self::Cyber => "cyber",
            Self::Graveyard => "graveyard",
            Self::Jurassic => "jurassic",
            Self::Hive => "hive",
            Self::Throne => "throne",
        }
    }
}

impl std::str::FromStr for MountainStyle {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl std::str::FromStr for RoadStyle {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl std::str::FromStr for EnvironmentStyle {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Atmospheric time-of-day solar illumination states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimeOfDay {
    #[default]
    Auto,
    Dawn,
    Day,
    Dusk,
    Night,
}

impl TimeOfDay {
    pub const ALL: &'static [TimeOfDay] =
        &[Self::Auto, Self::Dawn, Self::Day, Self::Dusk, Self::Night];

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "dawn" | "sunrise" | "morning" => Self::Dawn,
            "day" | "noon" | "midday" | "daylight" => Self::Day,
            "dusk" | "sunset" | "evening" | "twilight" => Self::Dusk,
            "night" | "midnight" | "moon" | "dark" => Self::Night,
            "auto" | "" => Self::Auto,
            _ => Self::Auto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Dawn => "dawn",
            Self::Day => "day",
            Self::Dusk => "dusk",
            Self::Night => "night",
        }
    }

    pub fn resolve(&self) -> Self {
        match self {
            Self::Auto => {
                use chrono::Timelike;
                let hour = chrono::Local::now().hour();
                match hour {
                    5..=8 => Self::Dawn,
                    9..=17 => Self::Day,
                    18..=20 => Self::Dusk,
                    _ => Self::Night,
                }
            }
            resolved => *resolved,
        }
    }

    /// Color adjustment filter for foliage, terrain, and horizons under solar illumination.
    pub fn apply_lighting(&self, color: Color) -> Color {
        match self {
            Self::Dawn => {
                // Warm pink-gold sunrise light: boost R, soft ambient
                let r = (color.r as f32 * 1.08 + 12.0).clamp(0.0, 255.0) as u8;
                let g = (color.g as f32 * 0.98 + 4.0).clamp(0.0, 255.0) as u8;
                let b = (color.b as f32 * 0.92).clamp(0.0, 255.0) as u8;
                Color::rgb(r, g, b)
            }
            Self::Day | Self::Auto => color, // Full radiant natural daylight
            Self::Dusk => {
                // Golden hour amber/twilight: rich warm vermilion tint
                let r = (color.r as f32 * 1.15 + 18.0).clamp(0.0, 255.0) as u8;
                let g = (color.g as f32 * 0.95 + 6.0).clamp(0.0, 255.0) as u8;
                let b = (color.b as f32 * 0.82).clamp(0.0, 255.0) as u8;
                Color::rgb(r, g, b)
            }
            Self::Night => {
                // Cool lunar moonlight: desaturate warm tones, boost cool lunar blue/silver
                let r = (color.r as f32 * 0.65).clamp(0.0, 255.0) as u8;
                let g = (color.g as f32 * 0.72).clamp(0.0, 255.0) as u8;
                let b = (color.b as f32 * 0.95 + 16.0).clamp(0.0, 255.0) as u8;
                Color::rgb(r, g, b)
            }
        }
    }
}

impl std::str::FromStr for TimeOfDay {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// Phenological seasonal cycle for natural foliage and ground cover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Season {
    #[default]
    Auto,
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub const ALL: &'static [Season] = &[
        Self::Auto,
        Self::Spring,
        Self::Summer,
        Self::Autumn,
        Self::Winter,
    ];

    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "spring" | "blossom" | "bloom" => Self::Spring,
            "summer" | "lush" => Self::Summer,
            "autumn" | "fall" | "amber" | "foliage" => Self::Autumn,
            "winter" | "frost" | "snow" | "bare" => Self::Winter,
            "auto" | "" => Self::Auto,
            _ => Self::Auto,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Autumn => "autumn",
            Self::Winter => "winter",
        }
    }

    pub fn resolve(&self) -> Self {
        match self {
            Self::Auto => {
                use chrono::Datelike;
                let month = chrono::Local::now().month();
                match month {
                    3..=5 => Self::Spring,
                    6..=8 => Self::Summer,
                    9..=11 => Self::Autumn,
                    _ => Self::Winter,
                }
            }
            resolved => *resolved,
        }
    }

    /// Phenological foliage color and character adjustment.
    pub fn apply_foliage_phenology(
        &self,
        species: TreeSpecies,
        base_color: Color,
        ch: char,
        tree_idx: usize,
    ) -> (char, Color) {
        match self {
            Self::Spring => match species {
                TreeSpecies::CherryBlossom | TreeSpecies::BloomingShrub => {
                    ('*', FRUIT_CHERRY_BLOSSOM)
                }
                TreeSpecies::Oak
                | TreeSpecies::Birch
                | TreeSpecies::Maple
                | TreeSpecies::Bonsai => (
                    ch,
                    Color::rgb(
                        base_color.r.saturating_add(18),
                        base_color.g.saturating_add(32),
                        base_color.b.saturating_sub(10),
                    ),
                ),
                _ => (ch, base_color),
            },
            Self::Summer => (
                ch,
                Color::rgb(
                    base_color.r.saturating_sub(12),
                    base_color.g.saturating_add(16),
                    base_color.b.saturating_sub(6),
                ),
            ),
            Self::Autumn => {
                let hash = tree_idx.wrapping_mul(2654435761);
                match species {
                    TreeSpecies::Maple => {
                        let scarlet = match hash % 3 {
                            0 => Color::rgb(228, 62, 42),
                            1 => Color::rgb(240, 92, 38),
                            _ => Color::rgb(212, 48, 52),
                        };
                        (ch, scarlet)
                    }
                    TreeSpecies::Birch => (ch, Color::rgb(246, 212, 54)),
                    TreeSpecies::Oak => {
                        let russet = if (hash & 1) == 0 {
                            Color::rgb(216, 134, 46)
                        } else {
                            Color::rgb(188, 108, 42)
                        };
                        (ch, russet)
                    }
                    TreeSpecies::Pine
                    | TreeSpecies::SnowFir
                    | TreeSpecies::Cypress
                    | TreeSpecies::Redwood
                    | TreeSpecies::DeadTree => (ch, base_color),
                    _ => (
                        ch,
                        Color::rgb(
                            base_color.r.saturating_add(45),
                            base_color.g.saturating_sub(8),
                            base_color.b.saturating_sub(25),
                        ),
                    ),
                }
            }
            Self::Winter => match species {
                TreeSpecies::Pine
                | TreeSpecies::SnowFir
                | TreeSpecies::Cypress
                | TreeSpecies::Redwood => {
                    if ch == '^' || ch == '*' || ch == '~' {
                        ('*', Color::rgb(245, 248, 255))
                    } else {
                        (ch, Color::rgb(42, 82, 60))
                    }
                }
                TreeSpecies::DeadTree => (ch, Color::rgb(140, 145, 150)),
                _ => {
                    let is_frost = matches!(ch, '*' | '.' | '\'' | '`');
                    if is_frost {
                        ('*', Color::rgb(230, 240, 255))
                    } else {
                        let twig = if ch == '@' || ch == '%' || ch == '#' {
                            ':'
                        } else {
                            ch
                        };
                        (twig, Color::rgb(120, 126, 134))
                    }
                }
            },
            Self::Auto => (ch, base_color),
        }
    }
}

impl std::str::FromStr for Season {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// The complete biological and thematic identity profile for any mascot.
#[derive(Debug, Clone)]
pub struct AnimalProfile {
    pub name: &'static str,
    pub base_anim: BaseAnim,
    pub environment: EnvironmentStyle,
    pub road: RoadStyle,
    pub mountain: MountainStyle,
    pub wildlife_palette: &'static [&'static str],
    pub eyes: &'static str,
    pub tongue: &'static str,
}

/// Harmonious Mountain and Road scenery styles for dynamic scenario adaptation.
pub fn environment_scenery_defaults(env: EnvironmentStyle) -> (MountainStyle, RoadStyle) {
    match env {
        EnvironmentStyle::Ocean => (MountainStyle::Seamount, RoadStyle::Seabed),
        EnvironmentStyle::Inferno => (MountainStyle::Volcano, RoadStyle::Magma),
        EnvironmentStyle::Arctic => (MountainStyle::Iceberg, RoadStyle::Ice),
        EnvironmentStyle::Space => (MountainStyle::Crater, RoadStyle::Grid),
        EnvironmentStyle::City => (MountainStyle::Skyline, RoadStyle::Roof),
        EnvironmentStyle::Graveyard => (MountainStyle::Gothic, RoadStyle::Crypt),
        EnvironmentStyle::Jurassic => (MountainStyle::Volcano, RoadStyle::Tracks),
        EnvironmentStyle::Throne => (MountainStyle::Castle, RoadStyle::Checkerboard),
        EnvironmentStyle::Hive => (MountainStyle::Garden, RoadStyle::Dirt),
        EnvironmentStyle::Swamp => (MountainStyle::Hills, RoadStyle::Mud),
        EnvironmentStyle::Savanna => (MountainStyle::Plateau, RoadStyle::Savanna),
        EnvironmentStyle::Forest => (MountainStyle::Peaks, RoadStyle::Dirt),
        EnvironmentStyle::Pasture | EnvironmentStyle::Cyber | EnvironmentStyle::None => {
            (MountainStyle::Hills, RoadStyle::Dirt)
        }
    }
}

/// Retrieve the signature biological profile, defaults, and wildlife palette for any mascot.
pub fn get_animal_profile(animal: &str) -> AnimalProfile {
    let clean = animal
        .strip_suffix(".cow")
        .unwrap_or(animal)
        .to_ascii_lowercase();

    let biome = forgum_platform::biome::get_mascot_biome(clean.as_str());

    let (static_name, base_anim, eyes, tongue) = match clean.as_str() {
        "apt" => ("apt", BaseAnim::Walk, "oo", "  "),
        "armadillo" => ("armadillo", BaseAnim::Walk, "oo", "  "),
        "atat" => ("atat", BaseAnim::Walk, "==", "  "),
        "bearface" => ("bearface", BaseAnim::Breathe, "oo", "  "),
        "beavis.zen" => ("beavis.zen", BaseAnim::Sway, "oo", "  "),
        "beavis" => ("beavis", BaseAnim::Sway, "oo", "  "),
        "bees" => ("bees", BaseAnim::Fly, "oo", "  "),
        "bill-the-cat" => ("bill-the-cat", BaseAnim::Talk, "oO", "U "),
        "bud-frogs" => ("bud-frogs", BaseAnim::Float, "@@", "  "),
        "bunny" => ("bunny", BaseAnim::Walk, "oo", "  "),
        "cat" => ("cat", BaseAnim::Walk, "^^", "  "),
        "cat2" => ("cat2", BaseAnim::Walk, "^^", "  "),
        "catfence" => ("catfence", BaseAnim::Sway, "^^", "  "),
        "charizardvice" => ("charizardvice", BaseAnim::Breathe, "oo", "  "),
        "charlie" => ("charlie", BaseAnim::Talk, "oo", "U "),
        "claw-arm" => ("claw-arm", BaseAnim::Pulse, "[]", "  "),
        "corgi" => ("corgi", BaseAnim::Walk, "oo", "U "),
        "cower" => ("cower", BaseAnim::Breathe, "oo", "  "),
        "cowfee" => ("cowfee", BaseAnim::Breathe, "oo", "  "),
        "cthulhu-mini" => ("cthulhu-mini", BaseAnim::Float, "oo", "  "),
        "daemon" => ("daemon", BaseAnim::Fly, "XX", "  "),
        "docker-whale" => ("docker-whale", BaseAnim::Float, "oo", "  "),
        "doge" => ("doge", BaseAnim::Walk, "oo", "  "),
        "dolphin" => ("dolphin", BaseAnim::Float, "oo", "  "),
        "dragon" => ("dragon", BaseAnim::Breathe, "oo", "  "),
        "dragon-and-cow" => ("dragon-and-cow", BaseAnim::Breathe, "oo", "  "),
        "duck" => ("duck", BaseAnim::Walk, "oo", "  "),
        "ebi_furai" => ("ebi_furai", BaseAnim::Float, "oo", "  "),
        "elephant" => ("elephant", BaseAnim::Walk, "oo", "  "),
        "elephant-in-snake" => ("elephant-in-snake", BaseAnim::Breathe, "oo", "  "),
        "elephant2" => ("elephant2", BaseAnim::Walk, "oo", "  "),
        "eyes" => ("eyes", BaseAnim::Pulse, "OO", "  "),
        "fat-banana" => ("fat-banana", BaseAnim::Sway, "oo", "  "),
        "fat-cow" => ("fat-cow", BaseAnim::Walk, "oo", "  "),
        "fence" => ("fence", BaseAnim::Sway, "||", "  "),
        "flaming-sheep" => ("flaming-sheep", BaseAnim::Breathe, "oo", "  "),
        "fox" => ("fox", BaseAnim::Walk, "^^", "  "),
        "ghost" => ("ghost", BaseAnim::Float, "oo", "  "),
        "ghostbusters" => ("ghostbusters", BaseAnim::Pulse, "oo", "  "),
        "glados" => ("glados", BaseAnim::Glitch, "[]", "  "),
        "goat" => ("goat", BaseAnim::Walk, "oo", "  "),
        "goat2" => ("goat2", BaseAnim::Walk, "oo", "  "),
        "golden-eagle" => ("golden-eagle", BaseAnim::Fly, "oo", "  "),
        "hamster" => ("hamster", BaseAnim::Walk, "oo", "  "),
        "happy-whale" => ("happy-whale", BaseAnim::Float, "^^", "  "),
        "hedgehog" => ("hedgehog", BaseAnim::Walk, "..", "  "),
        "hellokitty" => ("hellokitty", BaseAnim::Walk, "oo", "  "),
        "hippie" => ("hippie", BaseAnim::Sway, "oo", "  "),
        "hiya" => ("hiya", BaseAnim::Talk, "^^", "U "),
        "hypno" => ("hypno", BaseAnim::Pulse, "@@", "  "),
        "jellyfish" => ("jellyfish", BaseAnim::Float, "oo", "  "),
        "jesus" => ("jesus", BaseAnim::Float, "oo", "  "),
        "king" => ("king", BaseAnim::Breathe, "oo", "  "),
        "kiss" => ("kiss", BaseAnim::Talk, "oo", "U "),
        "kitten" => ("kitten", BaseAnim::Walk, "^^", "  "),
        "kitty" => ("kitty", BaseAnim::Walk, "^^", "  "),
        "knight" => ("knight", BaseAnim::Walk, "oo", "  "),
        "koala" => ("koala", BaseAnim::Breathe, "oo", "  "),
        "kosh" => ("kosh", BaseAnim::Float, "oo", "  "),
        "lamb" => ("lamb", BaseAnim::Walk, "oo", "  "),
        "lamb2" => ("lamb2", BaseAnim::Walk, "oo", "  "),
        "lobster" => ("lobster", BaseAnim::Float, "oo", "  "),
        "lollerskates" => ("lollerskates", BaseAnim::Walk, "oo", "  "),
        "luke-koala" => ("luke-koala", BaseAnim::Breathe, "oo", "  "),
        "mech-and-cow" => ("mech-and-cow", BaseAnim::Walk, "oo", "  "),
        "meow" => ("meow", BaseAnim::Talk, "^^", "U "),
        "minotaur" => ("minotaur", BaseAnim::Walk, "XX", "  "),
        "mona-lisa" => ("mona-lisa", BaseAnim::Breathe, "oo", "  "),
        "moofasa" => ("moofasa", BaseAnim::Breathe, "oo", "  "),
        "mooghidjirah" => ("mooghidjirah", BaseAnim::Breathe, "XX", "  "),
        "moojira" => ("moojira", BaseAnim::Breathe, "XX", "  "),
        "moose" => ("moose", BaseAnim::Walk, "oo", "  "),
        "mule" => ("mule", BaseAnim::Walk, "oo", "  "),
        "mutilated" => ("mutilated", BaseAnim::Dissolve, "XX", "  "),
        "nyan_cat" => ("nyan_cat", BaseAnim::Fly, "^^", "  "),
        "octopus" => ("octopus", BaseAnim::Float, "oo", "  "),
        "owl" => ("owl", BaseAnim::Breathe, "OO", "  "),
        "panther" => ("panther", BaseAnim::Walk, "oo", "  "),
        "pawn" => ("pawn", BaseAnim::Walk, "oo", "  "),
        "periodic-table" => ("periodic-table", BaseAnim::Pulse, "[]", "  "),
        "personality-sphere" => ("personality-sphere", BaseAnim::Float, "OO", "  "),
        "pig" => ("pig", BaseAnim::Walk, "oo", "  "),
        "pterodactyl" => ("pterodactyl", BaseAnim::Fly, "oo", "  "),
        "pufferfish" => ("pufferfish", BaseAnim::Pulse, "oo", "  "),
        "queen" => ("queen", BaseAnim::Breathe, "oo", "  "),
        "radioactive-kitty" => ("radioactive-kitty", BaseAnim::Glitch, "^^", "  "),
        "ram" => ("ram", BaseAnim::Walk, "oo", "  "),
        "ren" => ("ren", BaseAnim::Talk, "oo", "U "),
        "rhino" => ("rhino", BaseAnim::Walk, "oo", "  "),
        "rook" => ("rook", BaseAnim::Walk, "[]", "  "),
        "rooster" => ("rooster", BaseAnim::Walk, "oo", "  "),
        "satanic" => ("satanic", BaseAnim::Pulse, "XX", "  "),
        "sauron" => ("sauron", BaseAnim::Pulse, "()", "  "),
        "seahorse" => ("seahorse", BaseAnim::Float, "oo", "  "),
        "seahorse-big" => ("seahorse-big", BaseAnim::Float, "oo", "  "),
        "sheep" => ("sheep", BaseAnim::Walk, "oo", "  "),
        "shikato" => ("shikato", BaseAnim::Breathe, "==", "  "),
        "shrug" => ("shrug", BaseAnim::Sway, "oo", "  "),
        "skeleton" => ("skeleton", BaseAnim::Walk, "OO", "  "),
        "sloth" => ("sloth", BaseAnim::Breathe, "oo", "  "),
        "small" => ("small", BaseAnim::Walk, "oo", "  "),
        "smiling-octopus" => ("smiling-octopus", BaseAnim::Float, "^^", "  "),
        "snoopy" => ("snoopy", BaseAnim::Breathe, "oo", "  "),
        "snoopyhouse" => ("snoopyhouse", BaseAnim::Breathe, "oo", "  "),
        "snoopysleep" => ("snoopysleep", BaseAnim::Breathe, "--", "  "),
        "spidercow" => ("spidercow", BaseAnim::Walk, "88", "  "),
        "squid" => ("squid", BaseAnim::Float, "oo", "  "),
        "squirrel" => ("squirrel", BaseAnim::Walk, "..", "  "),
        "stegosaurus" => ("stegosaurus", BaseAnim::Walk, "oo", "  "),
        "stimpy" => ("stimpy", BaseAnim::Talk, "oO", "U "),
        "supermilker" => ("supermilker", BaseAnim::Pulse, "oo", "  "),
        "surgery" => ("surgery", BaseAnim::Glitch, "XX", "  "),
        "telebears" => ("telebears", BaseAnim::Walk, "oo", "  "),
        "three-eyes" => ("three-eyes", BaseAnim::Breathe, "OOO", "  "),
        "tiger" => ("tiger", BaseAnim::Walk, "oo", "  "),
        "tortoise" => ("tortoise", BaseAnim::Walk, "..", "  "),
        "turkey" => ("turkey", BaseAnim::Walk, "oo", "  "),
        "turtle" => ("turtle", BaseAnim::Float, "..", "  "),
        "tux" => ("tux", BaseAnim::Walk, "oo", "  "),
        "tux-big" => ("tux-big", BaseAnim::Walk, "oo", "  "),
        "tweety-bird" => ("tweety-bird", BaseAnim::Fly, "oo", "  "),
        "unipony" => ("unipony", BaseAnim::Walk, "oo", "  "),
        "vader" => ("vader", BaseAnim::Breathe, "oo", "  "),
        "viper" => ("viper", BaseAnim::Sway, "oo", "U "),
        "vulpix" => ("vulpix", BaseAnim::Walk, "oo", "  "),
        "walrus" => ("walrus", BaseAnim::Breathe, "oo", "  "),
        "weeping-angel" => ("weeping-angel", BaseAnim::Pulse, "XX", "  "),
        "whale" => ("whale", BaseAnim::Float, "oo", "  "),
        "wizard" => ("wizard", BaseAnim::Pulse, "oo", "  "),
        "wolf" => ("wolf", BaseAnim::Walk, "oo", "  "),
        "world" => ("world", BaseAnim::Float, "oo", "  "),
        "yoda" => ("yoda", BaseAnim::Float, "oo", "  "),
        "default" => ("default", BaseAnim::Walk, "oo", "  "),
        "nyan" => ("nyan", BaseAnim::Fly, "^^", "  "),
        "nyan-cat" => ("nyan-cat", BaseAnim::Fly, "^^", "  "),
        "nyancat" => ("nyancat", BaseAnim::Fly, "^^", "  "),
        _ => (biome.name, BaseAnim::Walk, "oo", "  "),
    };

    AnimalProfile {
        name: static_name,
        base_anim,
        environment: EnvironmentStyle::parse(biome.environment),
        road: RoadStyle::parse(biome.road),
        mountain: MountainStyle::parse(biome.mountain),
        wildlife_palette: &biome.natural_palette,
        eyes,
        tongue,
    }
}

/// Map any animal name to its signature Mountain, Road, and Environment archetype.
pub fn resolve_archetype(animal: &str) -> (MountainStyle, RoadStyle, EnvironmentStyle) {
    let profile = get_animal_profile(animal);
    (profile.mountain, profile.road, profile.environment)
}

/// Map any animal name to its archetype with optional CLI overrides and dynamic scenario adaptation.
pub fn resolve_archetype_with_overrides(
    animal: &str,
    cli_mtn: Option<MountainStyle>,
    cli_road: Option<RoadStyle>,
    cli_env: Option<EnvironmentStyle>,
) -> (MountainStyle, RoadStyle, EnvironmentStyle) {
    let profile = get_animal_profile(animal);
    let env = cli_env.unwrap_or(profile.environment);
    let (dyn_mtn, dyn_road) = environment_scenery_defaults(env);
    let mtn = cli_mtn.unwrap_or_else(|| {
        if cli_env.is_some() {
            dyn_mtn
        } else {
            profile.mountain
        }
    });
    let road = cli_road.unwrap_or_else(|| {
        if cli_env.is_some() {
            dyn_road
        } else {
            profile.road
        }
    });
    (mtn, road, env)
}

/// Calculate the exact number of natural mountain peaks across viewport width based on tectonic wavelength.
pub fn calculate_mountain_peaks(width: usize, style: MountainStyle) -> usize {
    calculate_mountain_peaks_aspect(width, style, None)
}

/// Calculate the exact number of natural mountain peaks across viewport width based on tectonic wavelength and aspect ratio.
pub fn calculate_mountain_peaks_aspect(
    width: usize,
    style: MountainStyle,
    aspect: Option<AspectRatioFormat>,
) -> usize {
    if style == MountainStyle::None || width == 0 {
        return 0;
    }
    let lambda = match style {
        MountainStyle::Peaks => 24.0,
        MountainStyle::Hills => 38.0,
        MountainStyle::Volcano => 48.0,
        MountainStyle::Iceberg => 20.0,
        MountainStyle::Skyline => 12.0,
        MountainStyle::Seamount => 32.0,
        MountainStyle::Plateau => 36.0,
        MountainStyle::Crater => 42.0,
        MountainStyle::Gothic => 16.0,
        MountainStyle::Castle => 22.0,
        MountainStyle::Garden => 28.0,
        MountainStyle::None => 1.0,
    };
    let phi = 1.618_034_f32;
    let base_peaks = (width as f32) / (lambda * (phi * 0.5));
    let mult = aspect.map(|a| a.peak_density_multiplier()).unwrap_or(1.0);
    let peak_count = (base_peaks * mult).round() as usize;
    peak_count.max(1)
}

/// Calculate the tectonic macro-range envelope factor E_range(x_world) in [0.0, 1.0].
/// Models natural plate tectonics: mountain ranges are finite geological belts that
/// rise through rolling foothills, peak majestically across the crest, slope down,
/// and terminate into open valleys/passes (elevation 0.0) where the sky and horizon
/// open up completely before a new mountain range emerges.
pub fn calculate_mountain_range_envelope(x_world: f32, style: MountainStyle) -> f32 {
    if style == MountainStyle::None {
        return 0.0;
    }
    let lambda_macro = match style {
        MountainStyle::Peaks | MountainStyle::Iceberg | MountainStyle::Gothic => 120.0_f32,
        MountainStyle::Hills | MountainStyle::Garden => 135.0_f32,
        MountainStyle::Volcano | MountainStyle::Plateau => 140.0_f32,
        MountainStyle::Skyline | MountainStyle::Castle => 110.0_f32,
        MountainStyle::Seamount | MountainStyle::Crater => 130.0_f32,
        MountainStyle::None => 1.0_f32,
    };

    let norm_x = ((x_world % lambda_macro) + lambda_macro) % lambda_macro;
    let tau = norm_x / lambda_macro; // in [0.0, 1.0)

    let range_idx = (x_world / lambda_macro).floor() as i32;
    let range_seed = ((((range_idx as u32).wrapping_mul(2654435761_u32)) as f32) * PHI_RECIPROCAL)
        .fract()
        .abs();
    let range_amplitude_scale = 0.85 + 0.30 * range_seed;

    let base_envelope = if tau < 0.15 {
        let u = tau / 0.15;
        u * u * (3.0 - 2.0 * u) // smoothstep foothills rise
    } else if tau < 0.62 {
        1.0 - 0.10 * ((tau - 0.15) * 4.0 * std::f32::consts::PI).sin().abs() // high crest
    } else if tau < 0.76 {
        let u = (tau - 0.62) / 0.14;
        1.0 - u * u * (3.0 - 2.0 * u) // smoothstep foothills descent
    } else {
        0.0 // True natural gap: mountain range has ended! Open sky & valley!
    };

    (base_envelope * range_amplitude_scale).clamp(0.0, 1.25)
}

/// Mathematical mountain height function using multi-harmonic Fourier synthesis.
/// Determines continuous elevation at column x incorporating erosion pinching and natural geomorphology.
pub fn calculate_mountain_height(
    x: f32,
    width: usize,
    mountain_base_y: usize,
    style: MountainStyle,
    time: f32,
) -> f32 {
    calculate_mountain_height_aspect(x, width, mountain_base_y, style, time, None)
}

/// Mathematical mountain height function using multi-harmonic Fourier synthesis with dynamic aspect ratio adaptation.
pub fn calculate_mountain_height_aspect(
    x: f32,
    width: usize,
    mountain_base_y: usize,
    style: MountainStyle,
    time: f32,
    aspect: Option<AspectRatioFormat>,
) -> f32 {
    let (base_h, max_var): (f32, f32) = match style {
        MountainStyle::Hills => (2.2, 1.2),
        MountainStyle::Peaks => (2.5, 1.8),
        MountainStyle::Volcano => (2.4, 1.5),
        MountainStyle::Iceberg => (2.6, 1.7),
        MountainStyle::Skyline => (2.7, 1.6),
        MountainStyle::Seamount => (2.0, 1.2),
        MountainStyle::Plateau => (2.3, 1.1),
        MountainStyle::Crater => (2.0, 1.2),
        MountainStyle::Gothic => (2.6, 1.6),
        MountainStyle::Castle => (2.4, 1.3),
        MountainStyle::Garden => (2.0, 1.0),
        MountainStyle::None => return 0.0,
    };

    let amp_mult = aspect
        .map(|a| a.mountain_amplitude_multiplier())
        .unwrap_or(1.0);
    let scale = ((mountain_base_y as f32) * 0.28).clamp(1.8, 8.0);
    let scale_mult = (scale / 2.5) * amp_mult;
    let scaled_base_h = base_h * scale_mult;
    let scaled_max_var = max_var * scale_mult;
    let max_allowed = (mountain_base_y as f32 * 0.38).max(2.5);

    let n_peaks = calculate_mountain_peaks_aspect(width, style, aspect).max(1) as f32;
    let w_f = (width.max(20)) as f32;
    let omega = (2.0 * std::f32::consts::PI * n_peaks) / w_f;
    let t_offset = time * 1.2;
    let x_world = x + t_offset;

    let raw_h = match style {
        MountainStyle::Peaks | MountainStyle::Iceberg | MountainStyle::Gothic => {
            // Glacial horn erosion: power-pinched acute peaks with broad cirque valleys
            let theta1 = omega * x_world;
            let theta2 = omega * 2.0 * x_world + 1.2;
            let theta3 = omega * 3.618 * x_world + 2.4;
            let h1 = theta1.sin().abs().powf(1.85) * scaled_max_var;
            let h2 = (theta2.sin() * 0.35 + theta3.cos() * 0.20) * scaled_max_var;
            scaled_base_h + h1 + h2
        }
        MountainStyle::Plateau => {
            // Mesa plateau: hyperbolic tangent cliff saturation produces sheer flat-topped mesas
            let theta = omega * x_world;
            let h1 = (theta.sin() * 3.0).tanh() * scaled_max_var;
            scaled_base_h + h1
        }
        MountainStyle::Volcano => {
            // Volcanic caldera: Lorentzian cone with central crater depression
            let period = w_f / n_peaks;
            let rel_x = ((x_world % period) + period) % period - (period * 0.5);
            let sigma = period * 0.25;
            let cone = scaled_max_var * 1.8 / (1.0 + (rel_x / sigma).powi(2));
            let crater = if rel_x.abs() < sigma * 0.35 {
                (1.0 - (rel_x / (sigma * 0.35)).abs()) * (scaled_max_var * 0.5)
            } else {
                0.0
            };
            scaled_base_h + cone - crater
        }
        MountainStyle::Skyline => {
            // Stepped building heights across urban skyline
            let theta1 = omega * x_world;
            let theta2 = omega * 2.23 * x_world + 0.8;
            let raw = theta1.sin() * 0.65 + theta2.sin() * 0.35;
            let steps = (raw * 4.0).floor() / 4.0;
            scaled_base_h + steps * scaled_max_var
        }
        _ => {
            // Natural rolling terrain: multi-harmonic smooth superposition
            let theta1 = omega * x_world;
            let theta2 = omega * 2.14 * x_world + 1.3;
            let theta3 = omega * 0.618 * x_world + 2.1;
            let h1 = theta1.sin() * (scaled_max_var * 0.60);
            let h2 = theta2.sin() * (scaled_max_var * 0.30);
            let h3 = theta3.cos() * (scaled_max_var * 0.20);
            scaled_base_h + h1 + h2 + h3
        }
    };

    let envelope = calculate_mountain_range_envelope(x_world, style);
    if envelope <= 0.02 {
        return 0.0;
    }
    (raw_h * envelope).clamp(0.0, max_allowed)
}

/// Render mountain / horizon layer directly behind the creature, anchored to the horizon above the road.
/// Uses a multi-harmonic mathematical algorithm to generate dynamic, non-uniform variable heights
/// across columns with smooth horizontal parallax scrolling and aspect ratio adaptation.
/// `mountain_base_y` is the row where the base of the mountain meets the ground.
pub fn render_mountain(
    fb: &mut FrameBuffer,
    style: MountainStyle,
    mountain_base_y: usize,
    width: usize,
    time: f32,
) {
    render_mountain_with_tod(fb, style, mountain_base_y, width, time, TimeOfDay::Auto);
}

/// Render mountain / horizon layer with explicit time-of-day solar illumination.
pub fn render_mountain_with_tod(
    fb: &mut FrameBuffer,
    style: MountainStyle,
    mountain_base_y: usize,
    width: usize,
    time: f32,
    tod: TimeOfDay,
) {
    if style == MountainStyle::None || fb.height < 2 || width == 0 {
        return;
    }

    let aspect = AspectRatioFormat::from_dimensions(fb.width as u16, fb.height as u16);

    let fg = match style {
        MountainStyle::Hills => Color::rgb(28, 68, 32),
        MountainStyle::Peaks => Color::rgb(64, 82, 92),
        MountainStyle::Volcano => Color::rgb(55, 50, 50),
        MountainStyle::Iceberg => Color::rgb(55, 115, 125),
        MountainStyle::Skyline => Color::rgb(42, 54, 62),
        MountainStyle::Seamount => Color::rgb(15, 42, 85),
        MountainStyle::Plateau => Color::rgb(90, 72, 65),
        MountainStyle::Crater => Color::rgb(52, 68, 78),
        MountainStyle::Gothic => Color::rgb(50, 32, 95),
        MountainStyle::Castle => Color::rgb(75, 58, 52),
        MountainStyle::Garden => Color::rgb(45, 95, 52),
        MountainStyle::None => return,
    };

    let compute = crate::gpu_compute::get_compute_engine(None);
    let mut heights = [0.0f32; 1024];
    let mut chunk_start = 0;
    while chunk_start < width {
        let chunk_len = (width - chunk_start).min(1024);
        compute.compute_mountain_elevations_chunk(
            chunk_start,
            chunk_len,
            width,
            mountain_base_y,
            style,
            time,
            Some(aspect),
            &mut heights[..chunk_len],
        );

        for i in 0..chunk_len {
            let x = chunk_start + i;
            let h_curr = heights[i];
            if h_curr < 0.5 {
                continue;
            }
            let h_prev = if i > 0 {
                heights[i - 1]
            } else if x > 0 {
                calculate_mountain_height_aspect(
                    (x - 1) as f32,
                    width,
                    mountain_base_y,
                    style,
                    time,
                    Some(aspect),
                )
            } else {
                calculate_mountain_height_aspect(
                    -1.0,
                    width,
                    mountain_base_y,
                    style,
                    time,
                    Some(aspect),
                )
            };
            let h_next = if i + 1 < chunk_len {
                heights[i + 1]
            } else {
                calculate_mountain_height_aspect(
                    (x + 1) as f32,
                    width,
                    mountain_base_y,
                    style,
                    time,
                    Some(aspect),
                )
            };

            let h_int = h_curr.round() as usize;
            let start_y = mountain_base_y.saturating_sub(h_int);

            let slope = h_next - h_prev;
            for y in start_y..=mountain_base_y {
                if y >= fb.height {
                    continue;
                }
                let (ch, col) = calculate_mountain_depth_cell(
                    style,
                    fg,
                    x,
                    y,
                    start_y,
                    mountain_base_y,
                    slope,
                    h_curr,
                    h_prev,
                    h_next,
                );

                if ch != ' ' {
                    let _ = fb.set(x, y, Cell::new(ch, tod.apply_lighting(col)));
                }
            }
        }
        chunk_start += chunk_len;
    }
}

/// Cool slate/cyan sfumato aerial mist tint for distant atmospheric mountain horizon.
pub const SFUMATO_SLATE: Color = Color::rgb(72, 105, 126);

/// Mathematical color blending with linear interpolation.
#[inline]
pub fn blend_color(c1: Color, c2: Color, t: f32) -> Color {
    let t_clamped = t.clamp(0.0, 1.0);
    let inv_t = 1.0 - t_clamped;
    Color::rgb(
        ((c1.r as f32 * inv_t + c2.r as f32 * t_clamped).round()) as u8,
        ((c1.g as f32 * inv_t + c2.g as f32 * t_clamped).round()) as u8,
        ((c1.b as f32 * inv_t + c2.b as f32 * t_clamped).round()) as u8,
    )
}

/// Calculate the depth-shaded cell glyph and TrueColor for mountain rendering across multiple strata.
/// Implements atmospheric aerial perspective (Leonardo da Vinci sfumato principle):
/// - Foreground crests: high-contrast sunlit highlights and deep shadow facets.
/// - Mid-ground foothills: rock strata and crevasse textures.
/// - Distant mountain layer: cool slate/cyan desaturation and soft mist textures.
pub fn calculate_mountain_depth_cell(
    style: MountainStyle,
    base_fg: Color,
    x: usize,
    y: usize,
    start_y: usize,
    mountain_base_y: usize,
    slope: f32,
    h_curr: f32,
    h_prev: f32,
    h_next: f32,
) -> (char, Color) {
    if y == start_y {
        // Foreground Crests: High-contrast sunlit highlights & deep shadow facets
        let ch = match style {
            MountainStyle::Skyline => {
                if x.is_multiple_of(3) {
                    '|'
                } else {
                    '_'
                }
            }
            MountainStyle::Castle => {
                if x.is_multiple_of(2) {
                    '#'
                } else {
                    '_'
                }
            }
            MountainStyle::Plateau => {
                if slope.abs() < 0.2 {
                    '_'
                } else if slope > 0.0 {
                    '/'
                } else {
                    '\\'
                }
            }
            MountainStyle::Garden => {
                if x.is_multiple_of(4) {
                    '*'
                } else {
                    '^'
                }
            }
            _ => {
                if h_curr > h_prev && h_curr > h_next {
                    '^'
                } else if slope > 0.25 {
                    '/'
                } else if slope < -0.25 {
                    '\\'
                } else {
                    '-'
                }
            }
        };

        let col = if h_curr > h_prev && h_curr > h_next {
            // Sunlit alpine peak apex highlight
            Color::rgb(
                base_fg.r.saturating_add(55),
                base_fg.g.saturating_add(55),
                base_fg.b.saturating_add(65),
            )
        } else if slope >= 0.0 {
            // Sunward slope highlight
            Color::rgb(
                base_fg.r.saturating_add(40),
                base_fg.g.saturating_add(40),
                base_fg.b.saturating_add(50),
            )
        } else {
            // Deep shadow facet on leeward slope
            Color::rgb(
                base_fg.r.saturating_sub(25),
                base_fg.g.saturating_sub(25),
                base_fg.b.saturating_sub(20),
            )
        };
        (ch, col)
    } else if y >= mountain_base_y.saturating_sub(1) {
        // Distant Mountain Layer: Cool slate/cyan desaturation and soft mist textures (Sfumato)
        let ch = if y == mountain_base_y {
            match style {
                MountainStyle::Volcano => '~',
                MountainStyle::Skyline => '_',
                MountainStyle::Seamount => '~',
                _ => '.',
            }
        } else {
            match (x + y) % 4 {
                0 => '~',
                1 => '.',
                2 => ':',
                _ => '-',
            }
        };
        let col = blend_color(base_fg, SFUMATO_SLATE, 0.55);
        (ch, col)
    } else {
        // Mid-Ground Foothills Layer: Rock strata and crevasse textures
        let strata = (y * 2 + x / 3) % 4;
        let is_crevasse = (x * 5 + y * 7).is_multiple_of(11);
        let ch = if is_crevasse {
            ':'
        } else {
            match style {
                MountainStyle::Skyline => {
                    if (x + y).is_multiple_of(2) {
                        '#'
                    } else {
                        '|'
                    }
                }
                MountainStyle::Iceberg => {
                    if (x + y).is_multiple_of(3) {
                        '^'
                    } else {
                        '|'
                    }
                }
                MountainStyle::Volcano => {
                    if (x + y).is_multiple_of(4) {
                        '/'
                    } else {
                        '\\'
                    }
                }
                MountainStyle::Gothic => {
                    if x.is_multiple_of(3) {
                        '|'
                    } else {
                        '/'
                    }
                }
                _ => {
                    if strata == 0 {
                        '='
                    } else if (x + y).is_multiple_of(3) {
                        '|'
                    } else {
                        '/'
                    }
                }
            }
        };

        let col = if is_crevasse {
            Color::rgb(
                base_fg.r.saturating_sub(20),
                base_fg.g.saturating_sub(20),
                base_fg.b.saturating_sub(15),
            )
        } else if strata == 0 {
            Color::rgb(
                base_fg.r.saturating_add(18),
                base_fg.g.saturating_add(18),
                base_fg.b.saturating_add(22),
            )
        } else {
            base_fg
        };
        (ch, col)
    }
}

/// Golden Ratio Phi constant for optimal low-discrepancy flora dispersion.
pub const PHI_GOLDEN: f32 = 1.618_033_988_7;

/// Fibonacci / Golden Ratio reciprocal constant for optimal low-discrepancy flora dispersion.
pub const PHI_RECIPROCAL: f32 = 0.618_033_988_7;

/// SplitMix64 hash for non-repeating, high-entropy procedural generation with maximal period 2^64.
#[inline]
pub fn splitmix64_hash(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
    x ^ (x >> 31)
}

/// Generate a non-repeating pseudo-random float in [0.0, 1.0) with zero short-cycle collapse.
#[inline]
pub fn non_repeating_unit_f32(seed: u64, idx: usize) -> f32 {
    let h = splitmix64_hash(seed.wrapping_add((idx as u64).wrapping_mul(0x517CC1B727220A95)));
    ((h >> 11) as f64 / ((1u64 << 53) as f64)) as f32
}

/// Mathematical distance between tree k and tree k+1 using Fibonacci phyllotaxis and grove clustering.
/// Models biological root exclusion and natural tree clumping (alternating stands and clearings).
pub fn calculate_inter_tree_distance(k: usize, min_dist: f32, var_dist: f32) -> f32 {
    let phi_fract = ((k as f32) * PHI_RECIPROCAL).fract();
    // Stand clustering cycle: groups trees in natural copse stands of 2-3 trees alternating with open glades
    let stand_phase = ((k as f32) * 0.785398).sin();
    let cluster_modulation = stand_phase * (var_dist * 0.28);
    (min_dist + phi_fract * var_dist * 0.60 + cluster_modulation)
        .clamp(min_dist * 0.70, min_dist + var_dist * 1.10)
}

/// Procedural grove parameters for grove index g: (cluster_size [1..=3], intra_spacing, clearing_width).
/// Models natural forest copse clustering and open meadow clearings.
pub fn get_grove_parameters(grove_idx: usize, min_dist: f32, var_dist: f32) -> (usize, f32, f32) {
    let r_size = non_repeating_unit_f32(0x9E37_79B9_CAFE_BABE, grove_idx);
    let r_intra = non_repeating_unit_f32(0x1234_5678_9ABC_DEF0, grove_idx);
    let r_clear = non_repeating_unit_f32(0xDEAD_BEEF_FEED_FACE, grove_idx);

    let cluster_size = if min_dist >= 70.0 {
        // Sparse biomes (Savanna, Pasture, Arctic, Inferno): mostly solitary, occasional pairs
        if r_size < 0.65 {
            1
        } else if r_size < 0.92 {
            2
        } else {
            3
        }
    } else if min_dist <= 55.0 {
        // Wooded biomes (Forest, Jurassic, Swamp): clusters of 2-3 trees with glades
        if r_size < 0.25 {
            1
        } else if r_size < 0.70 {
            2
        } else {
            3
        }
    } else {
        if r_size < 0.50 {
            1
        } else if r_size < 0.85 {
            2
        } else {
            3
        }
    };

    let intra_spacing = (min_dist * 0.28 + r_intra * var_dist * 0.25).clamp(18.0, 32.0);
    let clearing_width = min_dist + r_clear * var_dist;
    (cluster_size, intra_spacing, clearing_width)
}

/// Cumulative world coordinate of tree index n based on natural adaptive groves and open clearings.
/// Ensures trees form organic copse clusters of 1-3 trees followed by open meadow clearings,
/// with strictly monotonic, non-repeating spacing across the world.
pub fn calculate_tree_position(n: usize, min_dist: f32, var_dist: f32) -> usize {
    if n == 0 {
        return 12;
    }
    let mut accumulated_x = 12.0_f32;
    let mut tree_count = 0;
    let mut g = 0;
    loop {
        let (sz, intra, clear) = get_grove_parameters(g, min_dist, var_dist);
        if tree_count + sz > n {
            let offset_in_grove = n - tree_count;
            let pos = accumulated_x + (offset_in_grove as f32) * intra;
            return pos.round() as usize;
        }
        accumulated_x += ((sz.saturating_sub(1)) as f32) * intra + clear;
        tree_count += sz;
        g += 1;
    }
}

/// Calculate the distance across multiple trees (from tree `from` to tree `to`).
pub fn calculate_multi_tree_span(from: usize, to: usize, min_dist: f32, var_dist: f32) -> usize {
    if to <= from {
        return 0;
    }
    let p_from = calculate_tree_position(from, min_dist, var_dist);
    let p_to = calculate_tree_position(to, min_dist, var_dist);
    p_to.saturating_sub(p_from)
}

/// Calculate the mathematically optimal number of trees across viewport width
/// based on biome environmental canopy density and animal stature occupancy.
pub fn calculate_tree_count(width: usize, env: EnvironmentStyle, animal_height: usize) -> usize {
    if width == 0 {
        return 0;
    }
    let density_factor = match env {
        EnvironmentStyle::Forest => 1.40_f32,
        EnvironmentStyle::Pasture => 1.00_f32,
        EnvironmentStyle::Savanna => 0.60_f32,
        EnvironmentStyle::Arctic => 0.50_f32,
        EnvironmentStyle::Graveyard => 0.45_f32,
        EnvironmentStyle::Jurassic => 1.25_f32,
        EnvironmentStyle::Hive => 0.85_f32,
        EnvironmentStyle::Swamp => 0.90_f32,
        EnvironmentStyle::Ocean => 0.55_f32,
        EnvironmentStyle::City => 0.75_f32,
        EnvironmentStyle::Cyber => 0.70_f32,
        EnvironmentStyle::Space => 0.50_f32,
        EnvironmentStyle::None => return 0,
        _ => 0.90_f32,
    };

    let animal_scale_occupancy = if animal_height >= 12 {
        0.80_f32
    } else if animal_height <= 4 {
        1.15_f32
    } else {
        1.00_f32
    };

    let avg_stride = 55.0_f32;
    let count =
        ((width as f32 / avg_stride) * density_factor * animal_scale_occupancy).round() as usize;
    count.max(1)
}

/// The 22 distinct botanical and ecological flora species variants for procedural mixed-stand rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TreeSpecies {
    Oak,
    Pine,
    Acacia,
    SnowFir,
    DeadTree,
    Birch,
    Palm,
    WeepingWillow,
    Bonsai,
    Bamboo,
    Mangrove,
    Cactus,
    Fern,
    CreeperVine,
    BerryShrub,
    BloomingShrub,
    HerbalCluster,
    Cypress,
    Maple,
    CherryBlossom,
    Baobab,
    Redwood,
}

impl TreeSpecies {
    /// Comprehensive array of all 22 distinct flora species.
    pub const ALL_SPECIES: &'static [TreeSpecies] = &[
        Self::Oak,
        Self::Pine,
        Self::Acacia,
        Self::SnowFir,
        Self::DeadTree,
        Self::Birch,
        Self::Palm,
        Self::WeepingWillow,
        Self::Bonsai,
        Self::Bamboo,
        Self::Mangrove,
        Self::Cactus,
        Self::Fern,
        Self::CreeperVine,
        Self::BerryShrub,
        Self::BloomingShrub,
        Self::HerbalCluster,
        Self::Cypress,
        Self::Maple,
        Self::CherryBlossom,
        Self::Baobab,
        Self::Redwood,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Oak => "oak",
            Self::Pine => "pine",
            Self::Acacia => "acacia",
            Self::SnowFir => "snow_fir",
            Self::DeadTree => "dead_tree",
            Self::Birch => "birch",
            Self::Palm => "palm",
            Self::WeepingWillow => "weeping_willow",
            Self::Bonsai => "bonsai",
            Self::Bamboo => "bamboo",
            Self::Mangrove => "mangrove",
            Self::Cactus => "cactus",
            Self::Fern => "fern",
            Self::CreeperVine => "creeper_vine",
            Self::BerryShrub => "berry_shrub",
            Self::BloomingShrub => "blooming_shrub",
            Self::HerbalCluster => "herbal_cluster",
            Self::Cypress => "cypress",
            Self::Maple => "maple",
            Self::CherryBlossom => "cherry_blossom",
            Self::Baobab => "baobab",
            Self::Redwood => "redwood",
        }
    }
}

/// Retrieve the ecologically authentic flora species pool for a given environment biome.
pub fn get_biome_species_pool(env: EnvironmentStyle) -> &'static [TreeSpecies] {
    match env {
        // Cold / Arctic / Glacier: Conifers (SnowFir, Pine, Cypress), hardy shrubs. Zero creepers/vines.
        EnvironmentStyle::Arctic => &[
            TreeSpecies::SnowFir,
            TreeSpecies::Pine,
            TreeSpecies::Cypress,
            TreeSpecies::BerryShrub,
            TreeSpecies::HerbalCluster,
        ],
        // Water / Ocean / Beach: Palm, Mangrove, aquatic reeds / Ferns / HerbalCluster. Zero temperate creepers.
        EnvironmentStyle::Ocean => &[
            TreeSpecies::Palm,
            TreeSpecies::Mangrove,
            TreeSpecies::Fern,
            TreeSpecies::HerbalCluster,
        ],
        // Rainy / Jungle / Wetlands: Broadleaf trees (Mangrove, WeepingWillow, Oak, Bamboo), climbing CreeperVine, lush Ferns.
        EnvironmentStyle::Jurassic | EnvironmentStyle::Swamp => &[
            TreeSpecies::Mangrove,
            TreeSpecies::WeepingWillow,
            TreeSpecies::Oak,
            TreeSpecies::Bamboo,
            TreeSpecies::CreeperVine,
            TreeSpecies::Fern,
        ],
        // Savanna / Desert: Acacia, Baobab, Cactus, sparse arid herbs, DeadTree.
        EnvironmentStyle::Savanna => &[
            TreeSpecies::Acacia,
            TreeSpecies::Baobab,
            TreeSpecies::Cactus,
            TreeSpecies::HerbalCluster,
            TreeSpecies::DeadTree,
        ],
        // Graveyard: DeadTree, WeepingWillow, eerie Oak, thorny CreeperVine, Cypress.
        EnvironmentStyle::Graveyard => &[
            TreeSpecies::DeadTree,
            TreeSpecies::WeepingWillow,
            TreeSpecies::Oak,
            TreeSpecies::CreeperVine,
            TreeSpecies::Cypress,
        ],
        // Inferno: Arid scorched flora
        EnvironmentStyle::Inferno => &[
            TreeSpecies::DeadTree,
            TreeSpecies::Cactus,
            TreeSpecies::Bonsai,
            TreeSpecies::HerbalCluster,
        ],
        // Hive: Floral nectar-rich shrubs & flowering trees
        EnvironmentStyle::Hive => &[
            TreeSpecies::BloomingShrub,
            TreeSpecies::BerryShrub,
            TreeSpecies::HerbalCluster,
            TreeSpecies::CherryBlossom,
            TreeSpecies::Bonsai,
        ],
        // Throne: Formal topiary and stately trees
        EnvironmentStyle::Throne => &[
            TreeSpecies::Cypress,
            TreeSpecies::CherryBlossom,
            TreeSpecies::Bonsai,
            TreeSpecies::Oak,
            TreeSpecies::BloomingShrub,
        ],
        // City: Urban street plane trees, boulevard maples, park birches, formal cypresses, cherry avenues
        EnvironmentStyle::City => &[
            TreeSpecies::Birch,
            TreeSpecies::Oak,
            TreeSpecies::Maple,
            TreeSpecies::Cypress,
            TreeSpecies::CherryBlossom,
            TreeSpecies::Bonsai,
        ],
        // Cyber: Holographic data palms, circuit willows, neon grid pines, digital bamboo matrices
        EnvironmentStyle::Cyber => &[
            TreeSpecies::Palm,
            TreeSpecies::Pine,
            TreeSpecies::WeepingWillow,
            TreeSpecies::Bamboo,
            TreeSpecies::Bonsai,
        ],
        // Space: Prismatic crystal spires, starlight monoliths, alien xenoflora
        EnvironmentStyle::Space => &[
            TreeSpecies::Cypress,
            TreeSpecies::Pine,
            TreeSpecies::Fern,
            TreeSpecies::Bonsai,
            TreeSpecies::HerbalCluster,
        ],
        // Pasture: Open meadow shade trees and pastoral flowering shrubs (no deep-forest ferns or dark pines)
        EnvironmentStyle::Pasture => &[
            TreeSpecies::Oak,
            TreeSpecies::Birch,
            TreeSpecies::Maple,
            TreeSpecies::BerryShrub,
            TreeSpecies::BloomingShrub,
            TreeSpecies::HerbalCluster,
        ],
        // Forest / Pasture (and default): Oak, Maple, Birch, Pine, CherryBlossom, BerryShrub, BloomingShrub, HerbalCluster.
        _ => &[
            TreeSpecies::Oak,
            TreeSpecies::Maple,
            TreeSpecies::Birch,
            TreeSpecies::Pine,
            TreeSpecies::CherryBlossom,
            TreeSpecies::BerryShrub,
            TreeSpecies::BloomingShrub,
            TreeSpecies::HerbalCluster,
        ],
    }
}

/// Compute effective base distances (min_dist, var_dist) for an environment,
/// dynamically widening glades and open clearings on wide viewports or sparse biomes
/// so the visible tree count never exceeds 3x the species pool length, preserving open vistas.
pub fn get_effective_tree_distances(env: EnvironmentStyle, width: usize) -> (f32, f32) {
    let (min_dist, var_dist) = match env {
        EnvironmentStyle::Savanna | EnvironmentStyle::Inferno => (110.0_f32, 65.0_f32),
        EnvironmentStyle::Arctic => (95.0_f32, 55.0_f32),
        EnvironmentStyle::Pasture => (85.0_f32, 50.0_f32),
        EnvironmentStyle::City | EnvironmentStyle::Graveyard | EnvironmentStyle::Cyber => {
            (80.0_f32, 45.0_f32)
        }
        EnvironmentStyle::Forest => (48.0_f32, 28.0_f32),
        EnvironmentStyle::Hive | EnvironmentStyle::Throne => (56.0_f32, 32.0_f32),
        EnvironmentStyle::Swamp | EnvironmentStyle::Jurassic => (54.0_f32, 32.0_f32),
        _ => (85.0_f32, 50.0_f32),
    };
    let pool = get_biome_species_pool(env);
    let pool_len = pool.len().max(1);
    let max_trees_on_screen = (pool_len * 3) as f32;
    let min_stride_needed = (width as f32) / max_trees_on_screen;
    if min_stride_needed > min_dist {
        let expansion_scale = min_stride_needed / min_dist;
        (min_dist * expansion_scale, var_dist * expansion_scale)
    } else {
        (min_dist, var_dist)
    }
}

/// Determine the tree species for tree index n within an environment biome,
/// strictly enforcing the ecological constraint that no same type of tree
/// is present more than 3 times within the same screen width window.
pub fn calculate_tree_species_with_width(
    n: usize,
    env: EnvironmentStyle,
    animal_seed: u32,
    _window_width: usize,
) -> TreeSpecies {
    let pool = get_biome_species_pool(env);
    if pool.is_empty() {
        return TreeSpecies::Oak;
    }
    if pool.len() == 1 {
        return pool[0];
    }

    let p_len = pool.len();
    let round = n / p_len;
    let pos_in_round = n % p_len;

    // Deterministic stack-allocated Fisher-Yates permutation of the biome species pool
    let mut perm = [0usize; 16];
    for (i, val) in perm.iter_mut().enumerate().take(p_len) {
        *val = i;
    }

    // High-entropy 64-bit SplitMix64 seed for this round
    let round_seed = (animal_seed as u64)
        .wrapping_add((round as u64).wrapping_mul(0x9E3779B97F4A7C15))
        .wrapping_add(0xCAFE_F00D);
    let mut rng_state = splitmix64_hash(round_seed);

    for i in (1..p_len).rev() {
        rng_state = splitmix64_hash(rng_state);
        let j = (rng_state % ((i + 1) as u64)) as usize;
        perm.swap(i, j);
    }

    // Boundary smoothing: ensure the first element of this round doesn't match
    // the last element of the previous round to prevent back-to-back duplicates
    if round > 0 {
        let prev_seed = (animal_seed as u64)
            .wrapping_add(((round - 1) as u64).wrapping_mul(0x9E3779B97F4A7C15))
            .wrapping_add(0xCAFE_F00D);
        let mut prev_rng = splitmix64_hash(prev_seed);
        let mut prev_perm = [0usize; 16];
        for (i, val) in prev_perm.iter_mut().enumerate().take(p_len) {
            *val = i;
        }
        for i in (1..p_len).rev() {
            prev_rng = splitmix64_hash(prev_rng);
            let j = (prev_rng % ((i + 1) as u64)) as usize;
            prev_perm.swap(i, j);
        }
        let last_prev = prev_perm[p_len - 1];
        if perm[0] == last_prev && p_len > 1 {
            perm.swap(0, 1);
        }
    }

    pool[perm[pos_in_round]]
}

/// Determine the mathematical tree species for tree index n within an environment biome.
/// Guaranteed to never exceed 3 instances of any species within a screen width window.
pub fn calculate_tree_species(n: usize, env: EnvironmentStyle, animal_seed: u32) -> TreeSpecies {
    calculate_tree_species_with_width(n, env, animal_seed, 160)
}

/// Allometric dynamic trunk width scaling (1, 2, or 3 columns) based on species allometry,
/// target tree height, and non-repetitive tree index variation.
pub fn calculate_trunk_width(species: TreeSpecies, target_height: usize, tree_idx: usize) -> usize {
    let var = ((tree_idx as f32 * PHI_RECIPROCAL).fract() * 100.0) as usize;
    match species {
        TreeSpecies::Redwood => {
            if target_height >= 8 {
                3
            } else if target_height >= 4 {
                2
            } else {
                1
            }
        }
        TreeSpecies::Baobab => {
            if target_height >= 6 {
                3
            } else {
                2
            }
        }
        TreeSpecies::Oak | TreeSpecies::Maple | TreeSpecies::Mangrove => {
            if target_height >= 9 && var.is_multiple_of(2) {
                3
            } else if target_height >= 5 {
                2
            } else {
                1
            }
        }
        TreeSpecies::Pine
        | TreeSpecies::SnowFir
        | TreeSpecies::Birch
        | TreeSpecies::Palm
        | TreeSpecies::WeepingWillow
        | TreeSpecies::CherryBlossom
        | TreeSpecies::Cypress
        | TreeSpecies::DeadTree
            if target_height >= 8 && var.is_multiple_of(3) =>
        {
            2
        }
        _ => 1,
    }
}

// Calibrated multi-shade bark palette (organic woodland tones)
pub const BARK_ESPRESSO: Color = Color::rgb(58, 42, 34);
pub const BARK_CHESTNUT: Color = Color::rgb(128, 82, 48);
pub const BARK_RAW_UMBER: Color = Color::rgb(106, 78, 54);
pub const BARK_BIRCH_SILVER: Color = Color::rgb(216, 214, 206);
pub const BARK_MOSSY: Color = Color::rgb(72, 92, 64);
pub const BARK_TERRACOTTA: Color = Color::rgb(168, 88, 58);

/// Procedural bark texturing and rich multi-shade bark coloring.
/// Generates authentic textures ('||', '|/|', '(|)', '||', '!!', '|:|', '|;|')
/// across espresso, chestnut, raw umber, birch silver, mossy bark, and terracotta palettes.
pub fn get_bark_glyph_and_color(
    species: TreeSpecies,
    col_offset: usize,
    total_width: usize,
    y_from_base: usize,
    tree_idx: usize,
) -> (char, Color) {
    // 1. Determine base and accent colors for this species
    let (primary_col, accent_col) = match species {
        TreeSpecies::Birch => (BARK_BIRCH_SILVER, BARK_ESPRESSO),
        TreeSpecies::Redwood => (BARK_TERRACOTTA, BARK_CHESTNUT),
        TreeSpecies::DeadTree => (BARK_RAW_UMBER, BARK_ESPRESSO),
        TreeSpecies::Baobab | TreeSpecies::Acacia => (BARK_RAW_UMBER, BARK_CHESTNUT),
        TreeSpecies::CherryBlossom => (BARK_ESPRESSO, BARK_RAW_UMBER),
        TreeSpecies::Pine | TreeSpecies::SnowFir | TreeSpecies::Cypress => {
            (BARK_ESPRESSO, BARK_CHESTNUT)
        }
        _ => (BARK_CHESTNUT, BARK_RAW_UMBER),
    };

    // Mossy bark grounding near the soil contact line
    let col = if y_from_base == 0
        && matches!(
            species,
            TreeSpecies::Oak
                | TreeSpecies::Maple
                | TreeSpecies::WeepingWillow
                | TreeSpecies::Redwood
                | TreeSpecies::Mangrove
                | TreeSpecies::Bonsai
        ) {
        BARK_MOSSY
    } else if col_offset == 1 && total_width == 3 {
        accent_col
    } else {
        primary_col
    };

    // 2. Procedural bark texturing
    let row_phase = (y_from_base + tree_idx) % 5;
    let ch = match total_width {
        1 => match row_phase % 4 {
            0 => '|',
            1 => '|',
            2 => '!',
            _ => ':',
        },
        2 => {
            if col_offset == 0 {
                match row_phase % 4 {
                    0 | 2 => '|',
                    1 => '!',
                    _ => '|',
                }
            } else {
                match row_phase % 4 {
                    0 => '|', // "||"
                    1 => '!', // "!!"
                    2 => '/', // "|/"
                    _ => ':', // "|:"
                }
            }
        }
        3 => {
            if col_offset == 0 {
                match row_phase {
                    0 | 4 => '|',
                    1 => '(',
                    2 | 3 => '|',
                    _ => '|',
                }
            } else if col_offset == 1 {
                match row_phase {
                    0 => '/', // "|/|"
                    1 => '|', // "(|)"
                    2 => ':', // "|:|"
                    3 => ';', // "|;|"
                    _ => '|', // "|||"
                }
            } else {
                match row_phase {
                    0 | 4 => '|',
                    1 => ')',
                    2 | 3 => '|',
                    _ => '|',
                }
            }
        }
        _ => '|',
    };

    (ch, col)
}

// Soothing, calibrated fruit, flower, and star ornament palette (harmonious nature-calibrated shades)
pub const FRUIT_RUBY_BERRY: Color = Color::rgb(188, 84, 98); // Soft dusty roseberry / wild currant (warm, gentle, zero eye poke)
pub const FRUIT_GOLDEN_CITRUS: Color = Color::rgb(218, 165, 82); // Calibrated golden honey / soft apricot (subtle amber warmth)
pub const FRUIT_AMETHYST_PLUM: Color = Color::rgb(138, 108, 148); // Soothing elderberry / muted lavender plum (serene tranquility)
pub const FRUIT_CHERRY_BLOSSOM: Color = Color::rgb(232, 180, 194); // Delicate sakura petal pink (gentle pastel)
pub const FRUIT_IVORY_STAR: Color = Color::rgb(240, 238, 218); // Pale warm jasmine star flower (soothing warm white)

/// Procedurally place dynamic fruits, flowers, and star patterns on foliage envelopes
/// using a deterministic phyllotaxis spiral, modulated by tree index.
/// Guaranteed unique and never repeating within the animation viewport width.
pub fn get_foliage_ornament(
    species: TreeSpecies,
    tree_idx: usize,
    dx: isize,
    dy: isize,
    foliage_ch: char,
) -> Option<(char, Color)> {
    if species == TreeSpecies::DeadTree {
        return None;
    }

    // Deterministic phyllotaxis spiral node calculation
    let r2 = (dx * dx + dy * dy) as f32;
    let dist = r2.sqrt();
    let angle = (dy as f32).atan2(dx as f32);
    let tree_spiral_phase = (tree_idx as f32 * PHI_GOLDEN).fract() * 6.2831853;
    let spiral_val = ((dist * PHI_RECIPROCAL + angle * 0.381966 + tree_spiral_phase)
        / std::f32::consts::PI)
        .fract()
        .abs();

    let is_prime_organ = matches!(foliage_ch, '@' | '*' | '%' | 'o' | 'O');
    if !is_prime_organ && spiral_val > 0.28 {
        return None;
    }

    // Multiplicative hash ensures uniqueness across tree_idx
    let orn_seed = (tree_idx.wrapping_mul(2654435761)
        ^ (dx.wrapping_mul(31) as usize)
        ^ (dy.wrapping_mul(17) as usize))
        % 5;

    let (ch, col) = match species {
        TreeSpecies::BerryShrub => {
            if orn_seed == 0 {
                ('o', FRUIT_GOLDEN_CITRUS)
            } else if orn_seed == 1 {
                ('@', FRUIT_AMETHYST_PLUM)
            } else {
                ('*', FRUIT_RUBY_BERRY)
            }
        }
        TreeSpecies::BloomingShrub => match orn_seed {
            0 => ('*', FRUIT_IVORY_STAR),
            1 => ('@', FRUIT_CHERRY_BLOSSOM),
            2 => ('*', FRUIT_RUBY_BERRY),
            3 => ('%', FRUIT_AMETHYST_PLUM),
            _ => ('o', FRUIT_GOLDEN_CITRUS),
        },
        TreeSpecies::CherryBlossom => {
            if orn_seed.is_multiple_of(2) {
                ('*', FRUIT_CHERRY_BLOSSOM)
            } else {
                ('*', FRUIT_IVORY_STAR)
            }
        }
        TreeSpecies::Cactus => {
            if dy <= -4 {
                ('*', FRUIT_RUBY_BERRY)
            } else {
                return None;
            }
        }
        TreeSpecies::Palm => {
            if matches!(foliage_ch, 'o' | '*') {
                ('o', FRUIT_GOLDEN_CITRUS)
            } else {
                return None;
            }
        }
        TreeSpecies::Oak
        | TreeSpecies::Maple
        | TreeSpecies::Bonsai
        | TreeSpecies::HerbalCluster => match orn_seed {
            0 => ('o', FRUIT_RUBY_BERRY),
            1 => ('o', FRUIT_GOLDEN_CITRUS),
            2 => ('@', FRUIT_AMETHYST_PLUM),
            3 => ('*', FRUIT_CHERRY_BLOSSOM),
            _ => ('*', FRUIT_IVORY_STAR),
        },
        TreeSpecies::SnowFir => ('*', FRUIT_IVORY_STAR),
        TreeSpecies::Pine | TreeSpecies::Cypress | TreeSpecies::Redwood => {
            if orn_seed.is_multiple_of(2) {
                ('*', FRUIT_GOLDEN_CITRUS)
            } else {
                ('*', FRUIT_IVORY_STAR)
            }
        }
        TreeSpecies::WeepingWillow
        | TreeSpecies::Fern
        | TreeSpecies::Bamboo
        | TreeSpecies::Birch
        | TreeSpecies::Acacia
        | TreeSpecies::Mangrove
        | TreeSpecies::CreeperVine
        | TreeSpecies::Baobab => match orn_seed {
            0 => ('*', FRUIT_IVORY_STAR),
            1 => ('o', FRUIT_GOLDEN_CITRUS),
            2 => ('*', FRUIT_RUBY_BERRY),
            3 => ('@', FRUIT_AMETHYST_PLUM),
            _ => ('*', FRUIT_CHERRY_BLOSSOM),
        },
        TreeSpecies::DeadTree => return None,
    };

    Some((ch, col))
}

/// Mathematical tree height calculation proportioned with respect to the mascot's stature,
/// base animation kinematics, and environmental perspective.
///
/// If animal is tall/colossal (dragon, stegosaurus), trees scale up gracefully.
/// If animal is airborne/levitating (Float, Fly), tree height is scaled down so the creature
/// hovers above or level with the canopy rather than being swallowed inside the branches.
pub fn calculate_tree_height(
    tree_idx: usize,
    animal_height: usize,
    base_anim: BaseAnim,
    tree_base_y: usize,
) -> usize {
    let base_animal_h = animal_height.max(3) as f32;

    let anim_mult = match base_anim {
        BaseAnim::Walk | BaseAnim::Moonwalk => 1.25_f32,
        BaseAnim::Breathe => 1.35_f32,
        BaseAnim::Float | BaseAnim::Fly | BaseAnim::Abduction => 0.80_f32,
        BaseAnim::Sway => 1.15_f32,
        BaseAnim::Squish | BaseAnim::Liquid => 1.55_f32,
        _ => 1.20_f32,
    };

    // Biological maturity distribution using SplitMix64 Weyl non-repeating sequence:
    // Produces genuine forest diversity from ancient monarch giants to young saplings
    let r_maturity = non_repeating_unit_f32(0xA076_1D64_78BD_642F, tree_idx);
    let maturity_scale = match (r_maturity * 10.0) as usize {
        0 => 1.45_f32,     // Ancient Colossal Monarch
        1..=3 => 1.15_f32, // Stately Mature Tree
        4 | 5 => 0.85_f32, // Slender Adolescent Tree
        6 | 7 => 0.55_f32, // Young Understory Sapling
        8 => 1.35_f32,     // Giant Canopy Tree
        _ => 0.40_f32,     // Sprout / Dwarf Bush
    };

    let r_micro = non_repeating_unit_f32(0x517C_C1B7_2722_0A95, tree_idx);
    let micro_var = 0.90 + 0.20 * r_micro;

    let computed_h = base_animal_h * anim_mult * maturity_scale * micro_var;
    let max_h = ((tree_base_y as f32) * 0.90).max(2.0);
    computed_h.clamp(2.0, max_h).round() as usize
}

/// Procedural organic dynamic foliage color generator.
/// Evaluates authentic per-tree genetic hue shifts, Da Vinci atmospheric sunlit/shade vertical depth gradients,
/// and environmental biome lighting harmony.
pub fn get_tree_foliage_color(
    species: TreeSpecies,
    tree_idx: usize,
    dy_from_crown_top: usize,
    total_crown_h: usize,
    env: EnvironmentStyle,
) -> Color {
    // 1. Botanical signature base color
    let base = match species {
        TreeSpecies::Oak => Color::rgb(68, 138, 86),
        TreeSpecies::Pine => Color::rgb(42, 92, 60),
        TreeSpecies::Acacia => Color::rgb(142, 158, 84),
        TreeSpecies::SnowFir => Color::rgb(205, 222, 226),
        TreeSpecies::DeadTree => Color::rgb(92, 92, 92),
        TreeSpecies::Birch => Color::rgb(118, 162, 120),
        TreeSpecies::Palm => Color::rgb(64, 150, 92),
        TreeSpecies::WeepingWillow => Color::rgb(108, 142, 88),
        TreeSpecies::Bonsai => Color::rgb(58, 112, 74),
        TreeSpecies::Bamboo => Color::rgb(112, 154, 98),
        TreeSpecies::Mangrove => Color::rgb(54, 104, 72),
        TreeSpecies::Cactus => Color::rgb(82, 132, 92),
        TreeSpecies::Fern => Color::rgb(92, 136, 76),
        TreeSpecies::CreeperVine => Color::rgb(108, 142, 88),
        TreeSpecies::BerryShrub => Color::rgb(64, 118, 76),
        TreeSpecies::BloomingShrub => Color::rgb(96, 148, 104),
        TreeSpecies::HerbalCluster => Color::rgb(120, 146, 88),
        TreeSpecies::Cypress => Color::rgb(34, 76, 52),
        TreeSpecies::Maple => Color::rgb(188, 96, 76),
        TreeSpecies::CherryBlossom => Color::rgb(232, 178, 192),
        TreeSpecies::Baobab => Color::rgb(130, 156, 96),
        TreeSpecies::Redwood => Color::rgb(48, 96, 68),
    };

    // 2. Per-tree genetic micro-variation (ensures no two trees share identical flat hex)
    let hash = tree_idx.wrapping_mul(2654435761);
    let shift_r = (((hash % 29) as i32) - 14) * 2 / 3;
    let shift_g = ((((hash >> 8) % 27) as i32) - 13) * 2 / 3;
    let shift_b = ((((hash >> 16) % 25) as i32) - 12) * 2 / 3;

    let gen_r = (base.r as i32 + shift_r).clamp(10, 250) as u8;
    let gen_g = (base.g as i32 + shift_g).clamp(10, 250) as u8;
    let gen_b = (base.b as i32 + shift_b).clamp(10, 250) as u8;

    // 3. Vertical sunlit-to-shade depth gradient (Leonardo da Vinci sfumato lighting):
    // Top crown apex row catches direct golden sun; lower canopy rows sit in deep forest understory shadow.
    let crown_h = total_crown_h.max(1) as f32;
    let t_sun = 1.0 - (dy_from_crown_top as f32 / crown_h).clamp(0.0, 1.0); // 1.0 at apex, 0.0 at base

    let (sun_dr, sun_dg, sun_db) = if t_sun > 0.65 {
        // Direct sunlit highlight on top leaves
        (26, 32, 18)
    } else if t_sun > 0.35 {
        // Mid-canopy ambient
        (0, 0, 0)
    } else {
        // Deep shaded understory boughs
        (-22, -26, -18)
    };

    let lit_r = (gen_r as i32 + sun_dr).clamp(10, 255) as u8;
    let lit_g = (gen_g as i32 + sun_dg).clamp(10, 255) as u8;
    let lit_b = (gen_b as i32 + sun_db).clamp(10, 255) as u8;

    // 4. Environmental biome atmospheric color adaptation
    match env {
        EnvironmentStyle::Savanna => Color::rgb(
            lit_r.saturating_add(18),
            lit_g.saturating_add(12),
            lit_b.saturating_sub(10),
        ),
        EnvironmentStyle::Arctic => Color::rgb(
            lit_r.saturating_sub(8),
            lit_g.saturating_add(14),
            lit_b.saturating_add(26),
        ),
        EnvironmentStyle::Graveyard => Color::rgb(
            lit_r.saturating_sub(18),
            lit_g.saturating_sub(14),
            lit_b.saturating_add(4),
        ),
        EnvironmentStyle::Inferno => Color::rgb(
            lit_r.saturating_add(32),
            lit_g.saturating_add(10),
            lit_b.saturating_sub(20),
        ),
        EnvironmentStyle::Jurassic => Color::rgb(
            lit_r.saturating_sub(6),
            lit_g.saturating_add(22),
            lit_b.saturating_sub(6),
        ),
        EnvironmentStyle::Cyber => {
            // High-voltage holographic neon cyber tinting
            let cyber_mode = tree_idx % 3;
            match cyber_mode {
                0 => Color::rgb(0, 229, 255),  // Cyber Electric Cyan
                1 => Color::rgb(0, 255, 128),  // Cyber Matrix Emerald
                _ => Color::rgb(255, 64, 180), // Cyber Neon Magenta
            }
        }
        EnvironmentStyle::Space => {
            // Prismatic cosmic crystal shimmer
            let space_mode = tree_idx % 3;
            match space_mode {
                0 => Color::rgb(186, 104, 200), // Nebula Amethyst
                1 => Color::rgb(100, 220, 240), // Starlight Cyan
                _ => Color::rgb(255, 200, 100), // Solar Amber
            }
        }
        _ => Color::rgb(lit_r, lit_g, lit_b),
    }
}

/// Multi-harmonic wind gust and tree inertia sway calculation.
/// - Taller trees (e.g. Oak, Pine, Redwood, Baobab, Cypress, WeepingWillow) possess high physical inertia:
///   they sway with slow resonant frequency (~1.35 rad/s) and larger apex deflection, while anchored firmly at the base.
/// - Smaller shrubs, bushes, herbs, and bonsai possess low mass:
///   they flutter rapidly with high-frequency aerodynamic micro-turbulence (~4.8 rad/s).
pub fn calculate_tree_sway_offset(
    species: TreeSpecies,
    target_tree_h: usize,
    dy_from_base: usize,
    total_tree_h: usize,
    time: f32,
    tree_idx: usize,
) -> isize {
    if dy_from_base == 0 || total_tree_h == 0 {
        return 0; // The root base is firmly anchored in the earth; zero sway
    }

    let h_norm = (dy_from_base as f32 / total_tree_h as f32).clamp(0.0, 1.0);
    let seed = (tree_idx as f32) * 0.73;

    let is_small_shrub = matches!(
        species,
        TreeSpecies::BerryShrub
            | TreeSpecies::BloomingShrub
            | TreeSpecies::HerbalCluster
            | TreeSpecies::Bonsai
            | TreeSpecies::Fern
            | TreeSpecies::Cactus
    ) || target_tree_h <= 4;
    let is_willow = species == TreeSpecies::WeepingWillow;

    let compute = crate::gpu_compute::get_compute_engine(None);
    let mut out = [0isize; 1];
    compute.compute_tree_sway_batch(
        &[h_norm],
        &[is_small_shrub],
        &[is_willow],
        &[seed],
        time,
        &mut out,
    );
    out[0]
}

/// Render procedural midground trees and flora layer.
/// Planted on the horizon line directly above the road (`tree_base_y = road_y.saturating_sub(1)`).
/// Scrolls at midground speed `(time * 2.4)` between mountains and the road using Fibonacci spacing.
pub fn render_trees(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
) {
    render_trees_with_animal(
        fb,
        mountain,
        env,
        tree_base_y,
        width,
        time,
        None,
        None,
        None,
    );
}

/// Render procedural midground trees and flora layer with explicit animal-relative height and properties.
#[allow(clippy::too_many_arguments)]
pub fn render_trees_with_animal(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
) {
    render_trees_with_animal_and_tod(
        fb,
        mountain,
        env,
        tree_base_y,
        width,
        time,
        animal_name,
        animal_height,
        base_anim,
        None,
    );
}

/// Render procedural midground trees and flora layer with explicit animal kinematics and solar illumination.
#[allow(clippy::too_many_arguments)]
pub fn render_trees_with_animal_and_tod(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
    time_of_day: Option<TimeOfDay>,
) {
    render_trees_with_animal_and_tod_and_season(
        fb,
        mountain,
        env,
        tree_base_y,
        width,
        time,
        animal_name,
        animal_height,
        base_anim,
        time_of_day,
        None,
    );
}

/// Render procedural midground trees and flora layer with explicit animal kinematics, solar illumination, and seasonal phenology.
#[allow(clippy::too_many_arguments)]
pub fn render_trees_with_animal_and_tod_and_season(
    fb: &mut FrameBuffer,
    _mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
    time_of_day: Option<TimeOfDay>,
    season: Option<Season>,
) {
    if tree_base_y >= fb.height || width == 0 || env == EnvironmentStyle::None {
        return;
    }

    let tod = time_of_day.unwrap_or_default().resolve();
    let current_season = season.unwrap_or_default().resolve();
    let animal_h = animal_height.unwrap_or(8);
    let anim = base_anim.unwrap_or_else(|| {
        if let Some(name) = animal_name {
            get_animal_profile(name).base_anim
        } else {
            BaseAnim::Walk
        }
    });
    let animal_seed = animal_name.map_or(0, |n| n.len() as u32);

    // Continuous midground parallax motion: smooth floating-point progression for high refresh rates (120/144/240Hz)
    let scroll_speed = 7.5_f32;
    let scroll_xf = time * scroll_speed;
    let (min_dist, var_dist) = get_effective_tree_distances(env, width);
    if matches!(
        env,
        EnvironmentStyle::Ocean | EnvironmentStyle::Space | EnvironmentStyle::None
    ) {
        return;
    }

    // Natural grove and clearing range across the visible span
    let avg_tree_stride = (min_dist * 0.70).max(30.0);
    let min_world_x = (scroll_xf - 45.0).max(0.0);
    let start_tree = ((min_world_x / (avg_tree_stride * 1.5)).floor() as usize).saturating_sub(4);
    let max_world_x = scroll_xf + (width as f32) + 45.0;
    let end_tree = (((max_world_x / (min_dist * 0.65)).ceil() as usize) + 6).max(start_tree + 2);
    let fb_cell = std::cell::RefCell::new(fb);

    // Track visible species counts on the active screen viewport
    // Strictly enforces the invariant: <= 3 instances of any species per screen width
    // AND consecutive neighbor exclusion (adjacent trees never share identical species)
    let pool = get_biome_species_pool(env);
    let mut visible_species_counts = [0u8; 32];
    let mut last_rendered_species: Option<TreeSpecies> = None;

    for t_idx in start_tree..=end_tree {
        let world_x = calculate_tree_position(t_idx, min_dist, var_dist);
        let screen_xf = (world_x as f32) - scroll_xf;
        let screen_x = screen_xf.round() as isize;
        if screen_x < -35 {
            continue;
        }
        if screen_x >= (width as isize) + 25 {
            break;
        }

        // Active viewport visibility test: does tree canopy/trunk span touch the screen [0, width)?
        let is_in_viewport = screen_x >= -14 && screen_x < (width as isize) + 14;

        let preferred_species = calculate_tree_species_with_width(t_idx, env, animal_seed, width);
        let mut species = preferred_species;

        if is_in_viewport && !pool.is_empty() {
            // If the entire pool is saturated at >= 3 instances on screen, no more trees can be placed
            if visible_species_counts
                .iter()
                .take(pool.len())
                .all(|&cnt| cnt >= 3)
            {
                continue;
            }

            // Find an eligible species with count < 3 and != last_rendered_species
            let pref_idx = pool
                .iter()
                .position(|&s| s == preferred_species)
                .unwrap_or(0);
            let mut chosen_idx = None;

            // First check if preferred species satisfies both constraints
            if visible_species_counts[pref_idx] < 3
                && (pool.len() == 1 || Some(pool[pref_idx]) != last_rendered_species)
            {
                chosen_idx = Some(pref_idx);
            } else {
                // Cyclically search for an alternative candidate in the pool satisfying both
                for step in 1..pool.len() {
                    let cand_idx = (pref_idx + step) % pool.len();
                    if visible_species_counts[cand_idx] < 3
                        && (pool.len() == 1 || Some(pool[cand_idx]) != last_rendered_species)
                    {
                        chosen_idx = Some(cand_idx);
                        break;
                    }
                }
                // Fallback: if all candidate checks failed because of neighbor clash, take any with count < 3
                if chosen_idx.is_none() {
                    for step in 0..pool.len() {
                        let cand_idx = (pref_idx + step) % pool.len();
                        if visible_species_counts[cand_idx] < 3 {
                            chosen_idx = Some(cand_idx);
                            break;
                        }
                    }
                }
            }

            if let Some(idx) = chosen_idx {
                species = pool[idx];
                visible_species_counts[idx] += 1;
                last_rendered_species = Some(species);
            } else {
                continue;
            }
        }

        // Continuous sub-pixel parallax lighting interpolation in [-0.5, 0.5] (liquid smooth motion)
        let subpixel_fract = screen_xf - (screen_x as f32);
        let subpixel_lum = (1.0 + subpixel_fract * 0.16).clamp(0.80, 1.25);

        let target_tree_h = calculate_tree_height(t_idx, animal_h, anim, tree_base_y);
        let trunk_w = calculate_trunk_width(species, target_tree_h, t_idx);

        let plot_row = |dx_start: isize, dy: isize, s: &str, col: Color| {
            let y_from_base = (-dy).max(0) as usize;
            let sway = calculate_tree_sway_offset(
                species,
                target_tree_h,
                y_from_base,
                target_tree_h,
                time,
                t_idx,
            );
            let mut fb_guard = fb_cell.borrow_mut();
            for (i, c) in s.chars().enumerate() {
                if c == ' ' {
                    continue;
                }
                let px = screen_x + dx_start + sway + (i as isize);
                if px >= 0 && (px as usize) < width {
                    let py = (tree_base_y as isize) + dy;
                    if py >= 0 && (py as usize) < fb_guard.height {
                        let is_foliage = !matches!(
                            c,
                            '|' | '/' | '\\' | '-' | '_' | '=' | '#' | ' ' | '[' | ']' | '!' | '+'
                        );
                        let (final_ch, final_col) = if is_foliage {
                            let (pheno_c, pheno_col) =
                                current_season.apply_foliage_phenology(species, col, c, t_idx);
                            if let Some((orn_ch, orn_col)) = get_foliage_ornament(
                                species,
                                t_idx,
                                dx_start + (i as isize),
                                dy,
                                pheno_c,
                            ) {
                                let (orn_pheno_ch, orn_pheno_col) = current_season
                                    .apply_foliage_phenology(species, orn_col, orn_ch, t_idx);
                                (
                                    orn_pheno_ch,
                                    tod.apply_lighting(orn_pheno_col)
                                        .scale_luminance(subpixel_lum),
                                )
                            } else {
                                (
                                    pheno_c,
                                    tod.apply_lighting(pheno_col).scale_luminance(subpixel_lum),
                                )
                            }
                        } else {
                            (c, tod.apply_lighting(col).scale_luminance(subpixel_lum))
                        };
                        let _ =
                            fb_guard.set(px as usize, py as usize, Cell::new(final_ch, final_col));
                    }
                }
            }
        };

        let draw_trunk = |trunk_center: isize, top_dy: isize, width_cols: usize| {
            let mut fb_guard = fb_cell.borrow_mut();
            for dy in top_dy..=0 {
                let y_from_base = (-dy) as usize;
                let sway = calculate_tree_sway_offset(
                    species,
                    target_tree_h,
                    y_from_base,
                    target_tree_h,
                    time,
                    t_idx,
                );
                let center = trunk_center + sway;
                let mut plot_cell = |dx: isize, dy: isize, ch: char, col: Color| {
                    if ch == ' ' {
                        return;
                    }
                    let px = screen_x + dx;
                    if px >= 0 && (px as usize) < width {
                        let py = (tree_base_y as isize) + dy;
                        if py >= 0 && (py as usize) < fb_guard.height {
                            let _ = fb_guard.set(
                                px as usize,
                                py as usize,
                                Cell::new(
                                    ch,
                                    tod.apply_lighting(col).scale_luminance(subpixel_lum),
                                ),
                            );
                        }
                    }
                };
                match width_cols {
                    1 => {
                        let (ch, col) = get_bark_glyph_and_color(species, 0, 1, y_from_base, t_idx);
                        plot_cell(center, dy, ch, col);
                    }
                    2 => {
                        let (ch_l, col_l) =
                            get_bark_glyph_and_color(species, 0, 2, y_from_base, t_idx);
                        let (ch_r, col_r) =
                            get_bark_glyph_and_color(species, 1, 2, y_from_base, t_idx);
                        plot_cell(center, dy, ch_l, col_l);
                        plot_cell(center + 1, dy, ch_r, col_r);
                    }
                    3 => {
                        let (ch_l, col_l) =
                            get_bark_glyph_and_color(species, 0, 3, y_from_base, t_idx);
                        let (ch_m, col_m) =
                            get_bark_glyph_and_color(species, 1, 3, y_from_base, t_idx);
                        let (ch_r, col_r) =
                            get_bark_glyph_and_color(species, 2, 3, y_from_base, t_idx);
                        plot_cell(center - 1, dy, ch_l, col_l);
                        plot_cell(center, dy, ch_m, col_m);
                        plot_cell(center + 1, dy, ch_r, col_r);
                    }
                    _ => {
                        let (ch, col) = get_bark_glyph_and_color(species, 0, 1, y_from_base, t_idx);
                        plot_cell(center, dy, ch, col);
                    }
                }
            }
        };

        let fol_col = |dy_from_top: usize, total_crown_h: usize| {
            tod.apply_lighting(get_tree_foliage_color(
                species,
                t_idx,
                dy_from_top,
                total_crown_h,
                env,
            ))
        };
        let crown_var = t_idx % 2;

        match species {
            TreeSpecies::Oak => {
                if target_tree_h >= 10 {
                    plot_row(-1, -11, "     .-----.     ", fol_col(0, 8));
                    plot_row(-1, -10, "   .'  (o)  '.   ", fol_col(1, 8));
                    plot_row(-1, -9, "  /  (@)   (@)\\  ", fol_col(2, 8));
                    plot_row(-1, -8, " / (@)  (@)  (@)\\ ", fol_col(3, 8));
                    plot_row(-1, -7, "| (@)  (@)  (@)  |", fol_col(4, 8));
                    plot_row(-1, -6, " \\  /|\\   /|\\   / ", fol_col(5, 8));
                    plot_row(-1, -5, "  \\/_|____|_\\_/  ", fol_col(6, 8));
                    plot_row(-1, -4, "     \\ ||| /     ", fol_col(7, 8));
                    draw_trunk(6, -3, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    if crown_var == 0 {
                        plot_row(0, -8, "   .---.   ", fol_col(0, 6));
                        plot_row(0, -7, " .'     '. ", fol_col(1, 6));
                        plot_row(0, -6, "/  (@)    \\", fol_col(2, 6));
                        plot_row(0, -5, "|  (@)  (@)|", fol_col(3, 6));
                        plot_row(0, -4, "|    (@)   |", fol_col(4, 6));
                        plot_row(0, -3, " \\  _____ / ", fol_col(5, 6));
                        draw_trunk(5, -2, trunk_w);
                    } else {
                        plot_row(0, -8, " .---. .---. ", fol_col(0, 6));
                        plot_row(0, -7, "(  @  V  @  )", fol_col(1, 6));
                        plot_row(0, -6, " \\  (@)   /  ", fol_col(2, 6));
                        plot_row(0, -5, "  |  (@) |   ", fol_col(3, 6));
                        plot_row(0, -4, "   \\____/    ", fol_col(4, 6));
                        plot_row(0, -3, "    \\  /     ", fol_col(5, 6));
                        draw_trunk(5, -2, trunk_w);
                    }
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "  .---.  ", fol_col(0, 4));
                    plot_row(0, -4, " ( @ @ ) ", fol_col(1, 4));
                    plot_row(0, -3, "  \\___/  ", fol_col(2, 4));
                    plot_row(0, -2, "   \\|/   ", fol_col(3, 4));
                    draw_trunk(4, -1, trunk_w.min(2));
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " .---. ", fol_col(0, 2));
                    plot_row(0, -2, " ( @ ) ", fol_col(1, 2));
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "(@)", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::Pine => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     ^     ", fol_col(0, 9));
                    plot_row(0, -10, "    /|\\    ", fol_col(1, 9));
                    plot_row(0, -9, "   /*|*\\   ", fol_col(2, 9));
                    plot_row(0, -8, "  /* | *\\  ", fol_col(3, 9));
                    plot_row(0, -7, " /*  |  *\\ ", fol_col(4, 9));
                    plot_row(0, -6, "/ *  |  * \\", fol_col(5, 9));
                    plot_row(0, -5, "/*   |   *\\", fol_col(6, 9));
                    plot_row(0, -4, "/____|____\\", fol_col(7, 9));
                    plot_row(0, -3, "    /|\\    ", fol_col(8, 9));
                    draw_trunk(5, -2, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    if crown_var == 0 {
                        plot_row(0, -8, "    ^    ", fol_col(0, 6));
                        plot_row(0, -7, "   /|\\   ", fol_col(1, 6));
                        plot_row(0, -6, "  /*|*\\  ", fol_col(2, 6));
                        plot_row(0, -5, " /* | *\\ ", fol_col(3, 6));
                        plot_row(0, -4, "/ * | * \\", fol_col(4, 6));
                        plot_row(0, -3, " /__|__\\ ", fol_col(5, 6));
                        draw_trunk(4, -2, trunk_w);
                    } else {
                        plot_row(0, -8, "   /|\\   ", fol_col(0, 6));
                        plot_row(0, -7, "  /*|*\\  ", fol_col(1, 6));
                        plot_row(0, -6, "  /\\|/\\  ", fol_col(2, 6));
                        plot_row(0, -5, " /* | *\\ ", fol_col(3, 6));
                        plot_row(0, -4, "/*  |  *\\", fol_col(4, 6));
                        plot_row(0, -3, "/___|___\\", fol_col(5, 6));
                        draw_trunk(4, -2, trunk_w);
                    }
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "  ^  ", fol_col(0, 4));
                    plot_row(0, -4, " /|\\ ", fol_col(1, 4));
                    plot_row(0, -3, "//|\\\\", fol_col(2, 4));
                    plot_row(0, -2, "/_|_\\", fol_col(3, 4));
                    draw_trunk(2, -1, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " ^ ", fol_col(0, 2));
                    plot_row(0, -2, "/*\\", fol_col(1, 2));
                    draw_trunk(1, -1, 1);
                } else {
                    plot_row(0, -2, "^", fol_col(0, 1));
                    draw_trunk(0, -1, 1);
                }
            }
            TreeSpecies::Acacia => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   .-------------.   ", fol_col(0, 4));
                    plot_row(0, -10, "  (_______________)  ", fol_col(1, 4));
                    plot_row(0, -9, "    \\   \\ | /   /    ", fol_col(2, 4));
                    plot_row(0, -8, "     \\   \\|/   /     ", fol_col(3, 4));
                    plot_row(0, -7, "      \\   |   /      ", fol_col(4, 4));
                    plot_row(0, -6, "       \\  |  /       ", fol_col(5, 4));
                    draw_trunk(10, -5, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, "  .-----------.  ", fol_col(0, 4));
                    plot_row(0, -6, " (_____________) ", fol_col(1, 4));
                    plot_row(0, -5, "   \\   |   /   ", fol_col(2, 4));
                    plot_row(0, -4, "    \\  |  /    ", fol_col(3, 4));
                    draw_trunk(8, -3, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " _.~---~._ ", fol_col(0, 3));
                    plot_row(0, -4, "   \\ | /   ", fol_col(1, 3));
                    plot_row(0, -3, "    \\|/    ", fol_col(2, 3));
                    draw_trunk(5, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, "_~---~_", fol_col(0, 2));
                    plot_row(0, -2, "  \\|/  ", fol_col(1, 2));
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -1, "~---~", fol_col(0, 1));
                    draw_trunk(2, 0, 1);
                }
            }
            TreeSpecies::SnowFir => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     ^     ", fol_col(0, 9));
                    plot_row(0, -10, "    /|\\    ", fol_col(1, 9));
                    plot_row(0, -9, "   /*|*\\   ", fol_col(2, 9));
                    plot_row(0, -8, "  /* | *\\  ", fol_col(3, 9));
                    plot_row(0, -7, " /*  |  *\\ ", fol_col(4, 9));
                    plot_row(0, -6, "/ *  |  * \\", fol_col(5, 9));
                    plot_row(0, -5, "/*   |   *\\", fol_col(6, 9));
                    plot_row(0, -4, "/____|____\\", fol_col(7, 9));
                    draw_trunk(5, -3, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "    ^    ", fol_col(0, 6));
                    plot_row(0, -7, "   /|\\   ", fol_col(1, 6));
                    plot_row(0, -6, "  /*|*\\  ", fol_col(2, 6));
                    plot_row(0, -5, " /* | *\\ ", fol_col(3, 6));
                    plot_row(0, -4, "/ * | * \\", fol_col(4, 6));
                    plot_row(0, -3, " /__|__\\ ", fol_col(5, 6));
                    draw_trunk(4, -2, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "  ^  ", fol_col(0, 4));
                    plot_row(0, -4, " /|\\ ", fol_col(1, 4));
                    plot_row(0, -3, "/*|*\\", fol_col(2, 4));
                    plot_row(0, -2, "/_|_\\", fol_col(3, 4));
                    draw_trunk(2, -1, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " ^ ", fol_col(0, 2));
                    plot_row(0, -2, "/*\\", fol_col(1, 2));
                    draw_trunk(1, -1, 1);
                } else {
                    plot_row(0, -2, "^", fol_col(0, 1));
                    draw_trunk(0, -1, 1);
                }
            }
            TreeSpecies::DeadTree => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, " \\   /   \\   / ", fol_col(0, 6));
                    plot_row(0, -10, "  \\ / \\ / \\ /  ", fol_col(1, 6));
                    plot_row(0, -9, "   \\ |   | /   ", fol_col(2, 6));
                    plot_row(0, -8, "    \\ \\ / /    ", fol_col(3, 6));
                    plot_row(0, -7, "     \\ | /     ", fol_col(4, 6));
                    plot_row(0, -6, "      \\|/      ", fol_col(5, 6));
                    draw_trunk(6, -5, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "\\   /   \\", fol_col(0, 4));
                    plot_row(0, -7, " \\ / \\ / ", fol_col(1, 4));
                    plot_row(0, -6, "  \\ | /  ", fol_col(2, 4));
                    plot_row(0, -5, "   \\|/   ", fol_col(3, 4));
                    draw_trunk(4, -4, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, "\\ / \\", fol_col(0, 2));
                    plot_row(0, -3, " \\|/ ", fol_col(1, 2));
                    draw_trunk(2, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, "\\ /", fol_col(0, 1));
                    plot_row(0, -1, " | ", fol_col(0, 1));
                    draw_trunk(1, 0, 1);
                } else {
                    plot_row(0, -1, "\\/", fol_col(0, 1));
                    draw_trunk(0, 0, 1);
                }
            }
            TreeSpecies::Birch => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .---.     ", fol_col(0, 8));
                    plot_row(0, -10, "   .(* * *).   ", fol_col(1, 8));
                    plot_row(0, -9, " .(* * * * *). ", fol_col(2, 8));
                    plot_row(0, -8, "(* * * * * * *)", fol_col(3, 8));
                    plot_row(0, -7, " (* * * * * *) ", fol_col(4, 8));
                    plot_row(0, -6, "  \\(* * * *)/  ", fol_col(5, 8));
                    plot_row(0, -5, "   \\  | |  /   ", fol_col(6, 8));
                    plot_row(0, -4, "    \\_|_|_/    ", fol_col(7, 8));
                    draw_trunk(7, -3, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   .---.   ", fol_col(0, 5));
                    plot_row(0, -7, " .(* * *). ", fol_col(1, 5));
                    plot_row(0, -6, "(* * * * *)", fol_col(2, 5));
                    plot_row(0, -5, " (* * * *) ", fol_col(3, 5));
                    plot_row(0, -4, "  \\  |  /  ", fol_col(4, 5));
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "  .---.  ", fol_col(0, 3));
                    plot_row(0, -4, " .(* *). ", fol_col(1, 3));
                    plot_row(0, -3, "  \\_|_/  ", fol_col(2, 3));
                    draw_trunk(4, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " .(*). ", fol_col(0, 2));
                    plot_row(0, -2, "(* * *)", fol_col(1, 2));
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "(*)", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::Palm => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   \\\\  ||  //   ", fol_col(0, 6));
                    plot_row(0, -10, "  ---\\( * )/--- ", fol_col(1, 6));
                    plot_row(0, -9, "    // || \\\\    ", fol_col(2, 6));
                    plot_row(0, -8, "   -- (o o) --  ", fol_col(3, 6));
                    plot_row(0, -7, "     / || \\     ", fol_col(4, 6));
                    plot_row(0, -6, "       ||       ", fol_col(5, 6));
                    draw_trunk(7, -5, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "  \\  |  /  ", fol_col(0, 4));
                    plot_row(0, -7, " --( * )-- ", fol_col(1, 4));
                    plot_row(0, -6, "  /  |  \\  ", fol_col(2, 4));
                    plot_row(0, -5, "    (o)    ", fol_col(3, 4));
                    draw_trunk(5, -4, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " \\ | / ", fol_col(0, 3));
                    plot_row(0, -4, "--(*)--", fol_col(1, 3));
                    plot_row(0, -3, "  (o)  ", fol_col(2, 3));
                    draw_trunk(3, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, "\\|/", fol_col(0, 2));
                    plot_row(0, -2, "(*)", fol_col(1, 2));
                    draw_trunk(1, -1, 1);
                } else {
                    plot_row(0, -2, "\\|/", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::WeepingWillow => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .-----.     ", fol_col(0, 9));
                    plot_row(0, -10, "   .( ;;;;; ).   ", fol_col(1, 9));
                    plot_row(0, -9, " .(: | : | : | :). ", fol_col(2, 9));
                    plot_row(0, -8, "(: | : | : | : | :)", fol_col(3, 9));
                    plot_row(0, -7, " | : | : | : | : | ", fol_col(4, 9));
                    plot_row(0, -6, " : | : | : | : | : ", fol_col(5, 9));
                    plot_row(0, -5, " : :   | | |   : : ", fol_col(6, 9));
                    plot_row(0, -4, " :     | | |     : ", fol_col(7, 9));
                    plot_row(0, -3, "       \\_|_/       ", fol_col(8, 9));
                    draw_trunk(9, -2, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   .---.   ", fol_col(0, 6));
                    plot_row(0, -7, " .( ;;; ). ", fol_col(1, 6));
                    plot_row(0, -6, "(: | : | :)", fol_col(2, 6));
                    plot_row(0, -5, " | : | : | ", fol_col(3, 6));
                    plot_row(0, -4, " : | : | : ", fol_col(4, 6));
                    plot_row(0, -3, " :   |   : ", fol_col(5, 6));
                    draw_trunk(5, -2, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " .---. ", fol_col(0, 4));
                    plot_row(0, -4, "(:|:|:)", fol_col(1, 4));
                    plot_row(0, -3, " : | : ", fol_col(2, 4));
                    plot_row(0, -2, " :   : ", fol_col(3, 4));
                    draw_trunk(3, -1, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, "(;:;)", fol_col(0, 2));
                    plot_row(0, -2, " :|: ", fol_col(1, 2));
                    draw_trunk(2, -1, 1);
                } else {
                    plot_row(0, -2, ";|;", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::Bonsai => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     (~~~~)     ", fol_col(0, 8));
                    plot_row(0, -10, "    _/-'  `--_  ", fol_col(1, 8));
                    plot_row(0, -9, "  (~~~)    (~~) ", fol_col(2, 8));
                    plot_row(0, -8, "   /.-/    /    ", fol_col(3, 8));
                    plot_row(0, -7, " (~)  \\_._/     ", fol_col(4, 8));
                    plot_row(0, -6, "  /.-/  |       ", fol_col(5, 8));
                    plot_row(0, -5, "    \\_./        ", fol_col(6, 8));
                    draw_trunk(6, -4, 1);
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, "    (~~)    ", fol_col(0, 5));
                    plot_row(0, -6, "   _/-'     ", fol_col(1, 5));
                    plot_row(0, -5, " (~~~)  (~) ", fol_col(2, 5));
                    plot_row(0, -4, "  /.-/  /   ", fol_col(3, 5));
                    plot_row(0, -3, "    \\_./    ", fol_col(4, 5));
                    draw_trunk(6, -2, 1);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, "  (~~) ", fol_col(0, 3));
                    plot_row(0, -3, " ~/ -' ", fol_col(1, 3));
                    plot_row(0, -2, "(~~) | ", fol_col(2, 3));
                    draw_trunk(5, -1, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " (~) ", fol_col(0, 2));
                    plot_row(0, -1, " _/| ", fol_col(1, 2));
                    draw_trunk(3, 0, 1);
                } else {
                    plot_row(0, -1, "(~)", fol_col(0, 1));
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Bamboo => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "  ~^~   ~^~   ~^~  ", fol_col(0, 11));
                    plot_row(0, -10, " //|\\\\ //|\\\\ //|\\\\ ", fol_col(1, 11));
                    plot_row(0, -9, "   |-|   |-|   |-|   ", fol_col(2, 11));
                    plot_row(0, -8, "   | |   | |   | |   ", fol_col(3, 11));
                    plot_row(0, -7, " \\ |-| / |-| \\ |-| / ", fol_col(4, 11));
                    plot_row(0, -6, "   | |   | |   | |   ", fol_col(5, 11));
                    plot_row(0, -5, "   |-|   |-|   |-|   ", fol_col(6, 11));
                    plot_row(0, -4, "   | |   | |   | |   ", fol_col(7, 11));
                    plot_row(0, -3, "   |-|   |-|   |-|   ", fol_col(8, 11));
                    plot_row(0, -2, "   | |   | |   | |   ", fol_col(9, 11));
                    plot_row(0, -1, "   |-|   |-|   |-|   ", fol_col(10, 11));
                    plot_row(0, 0, "   | |   | |   | |   ", fol_col(11, 11));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "  ~^~  ~^~  ", fol_col(0, 8));
                    plot_row(0, -7, " //|\\\\ //|\\\\ ", fol_col(1, 8));
                    plot_row(0, -6, "   |-|   |-|   ", fol_col(2, 8));
                    plot_row(0, -5, "   | |   | |   ", fol_col(3, 8));
                    plot_row(0, -4, " \\ |-| / |-| \\ ", fol_col(4, 8));
                    plot_row(0, -3, "   | |   | |   ", fol_col(5, 8));
                    plot_row(0, -2, "   |-|   |-|   ", fol_col(6, 8));
                    plot_row(0, -1, "   | |   | |   ", fol_col(7, 8));
                    plot_row(0, 0, "   |-|   |-|   ", fol_col(8, 8));
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " ~^~ ", fol_col(0, 5));
                    plot_row(0, -4, "//|\\\\", fol_col(1, 5));
                    plot_row(0, -3, " |-| ", fol_col(2, 5));
                    plot_row(0, -2, " | | ", fol_col(3, 5));
                    plot_row(0, -1, " |-| ", fol_col(4, 5));
                    plot_row(0, 0, " | | ", fol_col(5, 5));
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " ~^~ ", fol_col(0, 3));
                    plot_row(0, -2, " |-| ", fol_col(1, 3));
                    plot_row(0, -1, " | | ", fol_col(2, 3));
                    plot_row(0, 0, " |-| ", fol_col(3, 3));
                } else {
                    plot_row(0, -2, "~^~", fol_col(0, 2));
                    plot_row(0, -1, "|-|", fol_col(1, 2));
                    plot_row(0, 0, "| |", fol_col(2, 2));
                }
            }
            TreeSpecies::Mangrove => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   .-------.   ", fol_col(0, 5));
                    plot_row(0, -10, "  (_________)  ", fol_col(1, 5));
                    plot_row(0, -9, "  (/ @ @ @ \\)  ", fol_col(2, 5));
                    plot_row(0, -8, "   \\_______/   ", fol_col(3, 5));
                    plot_row(0, -7, "     |||||     ", BARK_RAW_UMBER);
                    plot_row(0, -6, "    / ||| \\    ", BARK_RAW_UMBER);
                    plot_row(0, -5, "   / / | \\ \\   ", BARK_RAW_UMBER);
                    plot_row(0, -4, "  / /  |  \\ \\  ", BARK_RAW_UMBER);
                    plot_row(0, -3, " / /   |   \\ \\ ", BARK_RAW_UMBER);
                    plot_row(0, -2, "/ /    |    \\ \\", BARK_RAW_UMBER);
                    plot_row(0, -1, "/ /         \\ \\", BARK_RAW_UMBER);
                    plot_row(0, 0, "/ /           \\ \\", BARK_RAW_UMBER);
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "  .-----.  ", fol_col(0, 4));
                    plot_row(0, -7, " (_______) ", fol_col(1, 4));
                    plot_row(0, -6, " (/ @ @ \\) ", fol_col(2, 4));
                    plot_row(0, -5, "  \\_____/  ", fol_col(3, 4));
                    plot_row(0, -4, "    |||    ", BARK_RAW_UMBER);
                    plot_row(0, -3, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -2, "  / / \\ \\  ", BARK_RAW_UMBER);
                    plot_row(0, -1, " / /   \\ \\ ", BARK_RAW_UMBER);
                    plot_row(0, 0, "/ /     \\ \\", BARK_RAW_UMBER);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " (____) ", fol_col(0, 2));
                    plot_row(0, -3, "  /||\\  ", BARK_RAW_UMBER);
                    plot_row(0, -2, " / || \\ ", BARK_RAW_UMBER);
                    plot_row(0, -1, "/  ||  \\", BARK_RAW_UMBER);
                    plot_row(0, 0, "/        \\", BARK_RAW_UMBER);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " (__) ", fol_col(0, 1));
                    plot_row(0, -1, " /||\\ ", BARK_RAW_UMBER);
                    plot_row(0, 0, "/    \\", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -1, "(_)", fol_col(0, 1));
                    plot_row(0, 0, "/ \\", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::Cactus => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   _       _   ", fol_col(0, 8));
                    plot_row(0, -10, "  | |  _  | |  ", fol_col(1, 8));
                    plot_row(0, -9, "  | | | | | |  ", fol_col(2, 8));
                    plot_row(0, -8, "  | |_| |_| |  ", fol_col(3, 8));
                    plot_row(0, -7, "  |_   _   _|  ", fol_col(4, 8));
                    plot_row(0, -6, "    | | | |    ", fol_col(5, 8));
                    plot_row(0, -5, "    | : : |    ", fol_col(6, 8));
                    plot_row(0, -4, "    | : : |    ", fol_col(7, 8));
                    plot_row(0, -3, "    | : : |    ", fol_col(7, 8));
                    plot_row(0, -2, "    | : : |    ", fol_col(7, 8));
                    plot_row(0, -1, "    | : : |    ", fol_col(7, 8));
                    plot_row(0, 0, "    | : : |    ", fol_col(7, 8));
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, "   _     _   ", fol_col(0, 5));
                    plot_row(0, -6, "  | |   | |  ", fol_col(1, 5));
                    plot_row(0, -5, "  | |_._| |  ", fol_col(2, 5));
                    plot_row(0, -4, "  |_   _  |  ", fol_col(3, 5));
                    plot_row(0, -3, "    | | |    ", fol_col(4, 5));
                    plot_row(0, -2, "    | : |    ", fol_col(4, 5));
                    plot_row(0, -1, "    | : |    ", fol_col(4, 5));
                    plot_row(0, 0, "    | : |    ", fol_col(4, 5));
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " _._ ", fol_col(0, 3));
                    plot_row(0, -3, "|_:_|", fol_col(1, 3));
                    plot_row(0, -2, " |:| ", fol_col(2, 3));
                    plot_row(0, -1, " |:| ", fol_col(2, 3));
                    plot_row(0, 0, " |:| ", fol_col(2, 3));
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, "(_)", fol_col(0, 2));
                    plot_row(0, -1, "|:|", fol_col(1, 2));
                    plot_row(0, 0, "|:|", fol_col(1, 2));
                } else {
                    plot_row(0, -1, "o", fol_col(0, 1));
                    plot_row(0, 0, ":", fol_col(0, 1));
                }
            }
            TreeSpecies::Fern => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, " \\\\\\   ||   /// ", fol_col(0, 5));
                    plot_row(0, -10, "  \\\\\\  ||  ///  ", fol_col(1, 5));
                    plot_row(0, -9, "   \\\\\\ || ///   ", fol_col(2, 5));
                    plot_row(0, -8, "====~*~~*~======", fol_col(3, 5));
                    plot_row(0, -7, "  (//  ||  \\\\)  ", fol_col(4, 5));
                    plot_row(0, -6, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, -5, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, -4, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, -3, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, -2, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, -1, "      ||||      ", BARK_RAW_UMBER);
                    plot_row(0, 0, "      ||||      ", BARK_RAW_UMBER);
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, " \\\\  |  // ", fol_col(0, 4));
                    plot_row(0, -7, "  \\\\ | //  ", fol_col(1, 4));
                    plot_row(0, -6, "===~*~====", fol_col(2, 4));
                    plot_row(0, -5, " (/ | \\)  ", fol_col(3, 4));
                    plot_row(0, -4, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -3, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -2, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -1, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    ||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " \\ | / ", fol_col(0, 3));
                    plot_row(0, -3, "({%*})", fol_col(1, 3));
                    plot_row(0, -2, "  / \\  ", fol_col(2, 3));
                    plot_row(0, -1, "  |||  ", BARK_RAW_UMBER);
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " ,\\|/, ", fol_col(0, 2));
                    plot_row(0, -1, "  %%%  ", fol_col(1, 2));
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -1, "\\|/", fol_col(0, 1));
                    plot_row(0, 0, " | ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::CreeperVine => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "    ~@~    ", fol_col(0, 11));
                    plot_row(0, -10, "   ~/|/~   ", fol_col(1, 11));
                    plot_row(0, -9, "  ~/ | ~   ", fol_col(2, 11));
                    plot_row(0, -8, "  ~ \\|/~   ", fol_col(3, 11));
                    plot_row(0, -7, "   ~/|~    ", fol_col(4, 11));
                    plot_row(0, -6, "  ~/ | ~   ", fol_col(5, 11));
                    plot_row(0, -5, "  ~ \\|/~   ", fol_col(6, 11));
                    plot_row(0, -4, "   ~/|~    ", fol_col(7, 11));
                    plot_row(0, -3, "  ~/ | ~   ", fol_col(8, 11));
                    plot_row(0, -2, "  ~ \\|/~   ", fol_col(9, 11));
                    plot_row(0, -1, "   ~/|~    ", fol_col(10, 11));
                    plot_row(0, 0, "    |~     ", fol_col(11, 11));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   ~@~   ", fol_col(0, 8));
                    plot_row(0, -7, "  ~/|/~  ", fol_col(1, 8));
                    plot_row(0, -6, " ~/ | ~  ", fol_col(2, 8));
                    plot_row(0, -5, " ~ \\|/~  ", fol_col(3, 8));
                    plot_row(0, -4, "  ~/|~   ", fol_col(4, 8));
                    plot_row(0, -3, " ~/ | ~  ", fol_col(5, 8));
                    plot_row(0, -2, " ~ \\|/~  ", fol_col(6, 8));
                    plot_row(0, -1, "  ~/|~   ", fol_col(7, 8));
                    plot_row(0, 0, "   |~    ", fol_col(8, 8));
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " ~@~ ", fol_col(0, 4));
                    plot_row(0, -3, "~/|/~", fol_col(1, 4));
                    plot_row(0, -2, " ~|~ ", fol_col(2, 4));
                    plot_row(0, -1, "~/|~ ", fol_col(3, 4));
                    plot_row(0, 0, " |~  ", fol_col(4, 4));
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " ~@ ", fol_col(0, 2));
                    plot_row(0, -1, "~/|~", fol_col(1, 2));
                    plot_row(0, 0, " |  ", fol_col(2, 2));
                } else {
                    plot_row(0, -1, "~@", fol_col(0, 1));
                    plot_row(0, 0, "| ", fol_col(0, 1));
                }
            }
            TreeSpecies::BerryShrub => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .-----.     ", fol_col(0, 7));
                    plot_row(0, -10, "   .( *o*o* ).   ", fol_col(1, 7));
                    plot_row(0, -9, " .(* o * o * o). ", fol_col(2, 7));
                    plot_row(0, -8, "(* o * o * o * *)", fol_col(3, 7));
                    plot_row(0, -7, "( o * o * o * o )", fol_col(4, 7));
                    plot_row(0, -6, " (_____________) ", fol_col(5, 7));
                    plot_row(0, -5, "   /  | | |  \\   ", BARK_RAW_UMBER);
                    plot_row(0, -4, "   \\  | | |  /   ", BARK_RAW_UMBER);
                    plot_row(0, -3, "    \\_|_|_|_/    ", BARK_RAW_UMBER);
                    plot_row(0, -2, "       |||       ", BARK_RAW_UMBER);
                    plot_row(0, -1, "       |||       ", BARK_RAW_UMBER);
                    plot_row(0, 0, "       |||       ", BARK_RAW_UMBER);
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, "   .---.   ", fol_col(0, 5));
                    plot_row(0, -6, " .( *o* ). ", fol_col(1, 5));
                    plot_row(0, -5, "(* o * o *)", fol_col(2, 5));
                    plot_row(0, -4, "( o * o * )", fol_col(3, 5));
                    plot_row(0, -3, " (_______) ", fol_col(4, 5));
                    plot_row(0, -2, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -1, "   \\ | /   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    |||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " .(o*o). ", fol_col(0, 3));
                    plot_row(0, -3, "(* o * o)", fol_col(1, 3));
                    plot_row(0, -2, " (_____) ", fol_col(2, 3));
                    plot_row(0, -1, "   |||   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "   |||   ", BARK_RAW_UMBER);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " (o*o) ", fol_col(0, 2));
                    plot_row(0, -1, " (___) ", fol_col(1, 2));
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -1, "(o)", fol_col(0, 1));
                    plot_row(0, 0, " | ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::BloomingShrub => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .-----.     ", fol_col(0, 7));
                    plot_row(0, -10, "   .(@ % * @).   ", fol_col(1, 7));
                    plot_row(0, -9, " .(* % @ * % @). ", fol_col(2, 7));
                    plot_row(0, -8, "(* % @ * % @ * %)", fol_col(3, 7));
                    plot_row(0, -7, "(% * % @ * % @ *)", fol_col(4, 7));
                    plot_row(0, -6, " (_____________) ", fol_col(5, 7));
                    plot_row(0, -5, "   /  | | |  \\   ", BARK_RAW_UMBER);
                    plot_row(0, -4, "   \\  | | |  /   ", BARK_RAW_UMBER);
                    plot_row(0, -3, "    \\_|_|_|_/    ", BARK_RAW_UMBER);
                    plot_row(0, -2, "       |||       ", BARK_RAW_UMBER);
                    plot_row(0, -1, "       |||       ", BARK_RAW_UMBER);
                    plot_row(0, 0, "       |||       ", BARK_RAW_UMBER);
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, "   .---.   ", fol_col(0, 5));
                    plot_row(0, -6, " .(@ % *). ", fol_col(1, 5));
                    plot_row(0, -5, "(* % @ * %)", fol_col(2, 5));
                    plot_row(0, -4, "(% * % @ *)", fol_col(3, 5));
                    plot_row(0, -3, " (_______) ", fol_col(4, 5));
                    plot_row(0, -2, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -1, "   \\ | /   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    |||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " .(*%*). ", fol_col(0, 3));
                    plot_row(0, -3, "(@ * % @)", fol_col(1, 3));
                    plot_row(0, -2, " (_____) ", fol_col(2, 3));
                    plot_row(0, -1, "   |||   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "   |||   ", BARK_RAW_UMBER);
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " (*%*) ", fol_col(0, 2));
                    plot_row(0, -1, " (___) ", fol_col(1, 2));
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -1, "(*)", fol_col(0, 1));
                    plot_row(0, 0, " | ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::HerbalCluster => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "  |   |   |   |  ", fol_col(0, 11));
                    plot_row(0, -10, "  |i  |i  |i  |i ", fol_col(1, 11));
                    plot_row(0, -9, "  |i  |i  |i  |i ", fol_col(2, 11));
                    plot_row(0, -8, "  |i  |i  |i  |i ", fol_col(3, 11));
                    plot_row(0, -7, "  \\||/||/||/||/  ", fol_col(4, 11));
                    plot_row(0, -6, "   \\ ||| ||| /   ", fol_col(5, 11));
                    plot_row(0, -5, "    \\|||/|||/    ", fol_col(6, 11));
                    plot_row(0, -4, "     \\ ||| /     ", fol_col(7, 11));
                    plot_row(0, -3, "      \\||/      ", fol_col(8, 11));
                    plot_row(0, -2, "      /||\\      ", fol_col(9, 11));
                    plot_row(0, -1, "      /||\\      ", fol_col(10, 11));
                    plot_row(0, 0, "      \"\"\"\"      ", fol_col(11, 11));
                } else if target_tree_h >= 7 {
                    plot_row(0, -7, " |  |  | ", fol_col(0, 8));
                    plot_row(0, -6, " |i |i |i", fol_col(1, 8));
                    plot_row(0, -5, " |i |i |i", fol_col(2, 8));
                    plot_row(0, -4, " \\||/||/ ", fol_col(3, 8));
                    plot_row(0, -3, "  \\ || / ", fol_col(4, 8));
                    plot_row(0, -2, "   \\||/  ", fol_col(5, 8));
                    plot_row(0, -1, "   /||\\  ", fol_col(6, 8));
                    plot_row(0, 0, "   \"\"\"  ", fol_col(7, 8));
                } else if target_tree_h >= 5 {
                    plot_row(0, -4, " i | i ", fol_col(0, 5));
                    plot_row(0, -3, " |i|i|i", fol_col(1, 5));
                    plot_row(0, -2, " \\|/|/ ", fol_col(2, 5));
                    plot_row(0, -1, "  \\|/  ", fol_col(3, 5));
                    plot_row(0, 0, "  \"\"\"  ", fol_col(4, 5));
                } else if target_tree_h >= 3 {
                    plot_row(0, -2, " \\|/ ", fol_col(0, 3));
                    plot_row(0, -1, " |i| ", fol_col(1, 3));
                    plot_row(0, 0, " \"\"  ", fol_col(2, 3));
                } else {
                    plot_row(0, -1, "\\|/", fol_col(0, 2));
                    plot_row(0, 0, "\"\"", fol_col(1, 2));
                }
            }
            TreeSpecies::Cypress => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   ^   ", fol_col(0, 10));
                    plot_row(0, -10, "  /|\\  ", fol_col(1, 10));
                    plot_row(0, -9, " (| |) ", fol_col(2, 10));
                    plot_row(0, -8, " (| |) ", fol_col(3, 10));
                    plot_row(0, -7, " (| |) ", fol_col(4, 10));
                    plot_row(0, -6, " (| |) ", fol_col(5, 10));
                    plot_row(0, -5, " (| |) ", fol_col(6, 10));
                    plot_row(0, -4, " (| |) ", fol_col(7, 10));
                    plot_row(0, -3, " (| |) ", fol_col(8, 10));
                    plot_row(0, -2, "  \\|/  ", fol_col(9, 10));
                    draw_trunk(3, -1, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   ^   ", fol_col(0, 7));
                    plot_row(0, -7, "  /|\\  ", fol_col(1, 7));
                    plot_row(0, -6, " (| |) ", fol_col(2, 7));
                    plot_row(0, -5, " (| |) ", fol_col(3, 7));
                    plot_row(0, -4, " (| |) ", fol_col(4, 7));
                    plot_row(0, -3, " (| |) ", fol_col(5, 7));
                    plot_row(0, -2, "  \\|/  ", fol_col(6, 7));
                    draw_trunk(3, -1, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "  ^  ", fol_col(0, 4));
                    plot_row(0, -4, " /|\\ ", fol_col(1, 4));
                    plot_row(0, -3, " (| |) ", fol_col(2, 4));
                    plot_row(0, -2, "  \\|/  ", fol_col(3, 4));
                    draw_trunk(3, -1, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " ^ ", fol_col(0, 2));
                    plot_row(0, -2, "(|)", fol_col(1, 2));
                    draw_trunk(1, -1, 1);
                } else {
                    plot_row(0, -2, "^", fol_col(0, 1));
                    draw_trunk(0, -1, 1);
                }
            }
            TreeSpecies::Maple => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .---.     ", fol_col(0, 8));
                    plot_row(0, -10, "   ./{% %}\\.   ", fol_col(1, 8));
                    plot_row(0, -9, " ./{% % % %}\\. ", fol_col(2, 8));
                    plot_row(0, -8, "/{% % % % % %}\\", fol_col(3, 8));
                    plot_row(0, -7, "|% % % % % % %|", fol_col(4, 8));
                    plot_row(0, -6, " \\{% % % % %}/ ", fol_col(5, 8));
                    plot_row(0, -5, "  \\{% % % %}/  ", fol_col(6, 8));
                    plot_row(0, -4, "   \\_______/   ", fol_col(7, 8));
                    draw_trunk(7, -3, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   .---.   ", fol_col(0, 6));
                    plot_row(0, -7, " ./{% %}\\. ", fol_col(1, 6));
                    plot_row(0, -6, "/{% % % %}\\", fol_col(2, 6));
                    plot_row(0, -5, "|% % % % %|", fol_col(3, 6));
                    plot_row(0, -4, " \\{% % %}/ ", fol_col(4, 6));
                    plot_row(0, -3, "  \\_____/  ", fol_col(5, 6));
                    draw_trunk(5, -2, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " .{%}\\. ", fol_col(0, 3));
                    plot_row(0, -4, "({% %})", fol_col(1, 3));
                    plot_row(0, -3, " \\___/ ", fol_col(2, 3));
                    draw_trunk(3, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, "({%})", fol_col(0, 2));
                    plot_row(0, -2, " \\_/ ", fol_col(1, 2));
                    draw_trunk(2, -1, 1);
                } else {
                    plot_row(0, -2, "{%}", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::CherryBlossom => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "     .---.     ", fol_col(0, 8));
                    plot_row(0, -10, "   .(* @ *).   ", fol_col(1, 8));
                    plot_row(0, -9, " .(* @ * @ *). ", fol_col(2, 8));
                    plot_row(0, -8, "(* @ * @ * @ *)", fol_col(3, 8));
                    plot_row(0, -7, "(* \\   |   / *)", fol_col(4, 8));
                    plot_row(0, -6, " \\  \\_/ \\_/  / ", fol_col(5, 8));
                    plot_row(0, -5, "  \\   /|\\   /  ", fol_col(6, 8));
                    plot_row(0, -4, "   \\___|___/   ", fol_col(7, 8));
                    draw_trunk(7, -3, trunk_w.max(2));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "   .---.   ", fol_col(0, 5));
                    plot_row(0, -7, " .(* @ *). ", fol_col(1, 5));
                    plot_row(0, -6, "(* @ * @ *)", fol_col(2, 5));
                    plot_row(0, -5, "(* \\   / *)", fol_col(3, 5));
                    plot_row(0, -4, " \\  \\_/  / ", fol_col(4, 5));
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " .(*@*). ", fol_col(0, 3));
                    plot_row(0, -4, "(* @ * *)", fol_col(1, 3));
                    plot_row(0, -3, "  \\_|_/  ", fol_col(2, 3));
                    draw_trunk(4, -2, 1);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " (*@*) ", fol_col(0, 2));
                    plot_row(0, -2, "  \\|/  ", fol_col(1, 2));
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "(*)", fol_col(0, 1));
                    draw_trunk(1, -1, 1);
                }
            }
            TreeSpecies::Baobab => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "   \\\\ || //   ", fol_col(0, 3));
                    plot_row(0, -10, "  (_________)  ", fol_col(1, 3));
                    plot_row(0, -9, "   /-------/   ", BARK_RAW_UMBER);
                    plot_row(0, -8, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -7, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -6, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -5, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -4, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -3, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -2, "  |  :::::  |  ", BARK_RAW_UMBER);
                    plot_row(0, -1, "   \\-------/   ", BARK_RAW_UMBER);
                    draw_trunk(6, 0, 3);
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "  \\ | /  ", fol_col(0, 2));
                    plot_row(0, -7, " (_____) ", fol_col(1, 2));
                    plot_row(0, -6, "  /---\\  ", BARK_RAW_UMBER);
                    plot_row(0, -5, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -4, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -3, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -2, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -1, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, 0, "  \\---/  ", BARK_RAW_UMBER);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, " \\ | / ", fol_col(0, 2));
                    plot_row(0, -4, " (___) ", fol_col(1, 2));
                    plot_row(0, -3, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, -2, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, -1, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, 0, " \\---/ ", BARK_RAW_UMBER);
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, " \\|/ ", fol_col(0, 2));
                    plot_row(0, -2, "(___)", fol_col(1, 2));
                    plot_row(0, -1, " |:| ", BARK_RAW_UMBER);
                    plot_row(0, 0, " \\-/ ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, "  \\|/  ", fol_col(0, 1));
                    plot_row(0, -1, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, 0, " \\---/ ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::Redwood => {
                if target_tree_h >= 10 {
                    plot_row(0, -11, "      ^      ", fol_col(0, 9));
                    plot_row(0, -10, "     /|\\     ", fol_col(1, 9));
                    plot_row(0, -9, "    //|\\\\    ", fol_col(2, 9));
                    plot_row(0, -8, "   ///|\\\\\\   ", fol_col(3, 9));
                    plot_row(0, -7, "  ////|\\\\\\\\  ", fol_col(4, 9));
                    plot_row(0, -6, "   ///|\\\\\\   ", fol_col(5, 9));
                    plot_row(0, -5, "    //|\\\\    ", fol_col(6, 9));
                    plot_row(0, -4, "     /|\\     ", fol_col(7, 9));
                    plot_row(0, -3, "    //|\\\\    ", fol_col(8, 9));
                    draw_trunk(6, -2, trunk_w.max(3));
                } else if target_tree_h >= 7 {
                    plot_row(0, -8, "     ^     ", fol_col(0, 6));
                    plot_row(0, -7, "    /|\\    ", fol_col(1, 6));
                    plot_row(0, -6, "   //|\\\\   ", fol_col(2, 6));
                    plot_row(0, -5, "  ///|\\\\\\  ", fol_col(3, 6));
                    plot_row(0, -4, "   //|\\\\   ", fol_col(4, 6));
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 5 {
                    plot_row(0, -5, "   ^   ", fol_col(0, 4));
                    plot_row(0, -4, "  /|\\  ", fol_col(1, 4));
                    plot_row(0, -3, " //|\\\\ ", fol_col(2, 4));
                    draw_trunk(3, -2, trunk_w.min(2));
                } else if target_tree_h >= 3 {
                    plot_row(0, -3, "  ^  ", fol_col(0, 2));
                    plot_row(0, -2, " /|\\ ", fol_col(1, 2));
                    draw_trunk(2, -1, 1);
                } else {
                    plot_row(0, -2, "  ^  ", fol_col(0, 1));
                    plot_row(0, -1, " /|\\ ", fol_col(0, 1));
                    draw_trunk(2, 0, 1);
                }
            }
        }
    }
}

/// Mathematical road surface texture generator based on harmonic roughness synthesis.
/// Mimics physical soil grains, cobblestone mortar joints, cooled magma crust, and ice fractures.
pub fn calculate_road_texture(x: usize, style: RoadStyle, time: f32) -> (char, Color) {
    let x_f = x as f32;
    // Multi-harmonic deterministic surface roughness synthesis
    let r1 = (x_f * 0.43 + time * 1.8).sin();
    let r2 = (x_f * 0.97 - time * 0.7).cos();
    let r3 = (x_f * 2.17 + time * 3.1).sin();
    let roughness = (r1 * 0.5 + r2 * 0.3 + r3 * 0.2).abs();

    match style {
        RoadStyle::Dirt => {
            let light = (r1 * 0.35 + 0.65).clamp(0.4, 1.0);
            let fg = Color::rgb(
                (141.0 * light) as u8,
                (110.0 * light) as u8,
                (99.0 * light) as u8,
            );
            let ch = if roughness > 0.82 {
                'o' // small pebble
            } else if roughness > 0.55 {
                '.' // gravel grain
            } else if roughness > 0.30 {
                '_' // earth furrow
            } else {
                ' ' // packed soil
            };
            (ch, fg)
        }
        RoadStyle::Cobblestone => {
            let p = ((x as f32 + time * 7.5) % 5.0 + 5.0) % 5.0;
            let stone_idx = p.floor() as usize;
            let light = (r2 * 0.25 + 0.85).clamp(0.6, 1.1);
            let base_v = if stone_idx == 0 { 110.0 } else { 158.0 };
            let fg = Color::rgb(
                (base_v * light) as u8,
                (base_v * light) as u8,
                (base_v * light) as u8,
            );
            let ch = if stone_idx == 0 {
                '|' // mortar joint between pavers
            } else if roughness > 0.6 {
                '=' // worn flagstone
            } else {
                '-' // smooth paver
            };
            (ch, fg)
        }
        RoadStyle::Magma => {
            let light = (r3 * 0.3 + 0.7).clamp(0.4, 1.0);
            let fg = if roughness > 0.65 {
                Color::rgb(255, (171.0 * light) as u8, 0) // incandescent magma vent
            } else {
                Color::rgb(255, (87.0 * light) as u8, (34.0 * light) as u8) // cooling basalt crust
            };
            let ch = if roughness > 0.75 {
                '*' // bubbling magma fountain
            } else if roughness > 0.35 {
                '~' // convective crust ripple
            } else {
                '.' // basalt grain
            };
            (ch, fg)
        }
        RoadStyle::Ice => {
            let light = (r1 * 0.2 + 0.8).clamp(0.6, 1.0);
            let fg = Color::rgb(
                (224.0 * light) as u8,
                (247.0 * light) as u8,
                (250.0 * light) as u8,
            );
            let fissure = (((x as f32 + time * 4.5) % 9.0 + 9.0) % 9.0).floor() as usize;
            let ch = if fissure == 0 {
                '/' // glacial fissure / crevasse
            } else if roughness > 0.5 {
                '=' // glazed ice sheet
            } else {
                '-' // packed snow
            };
            (ch, fg)
        }
        RoadStyle::Seabed => {
            let light = (r2 * 0.25 + 0.75).clamp(0.5, 1.0);
            let fg = Color::rgb(
                (255.0 * light) as u8,
                (213.0 * light) as u8,
                (79.0 * light) as u8,
            );
            let ch = if roughness > 0.70 {
                '~' // ocean wave ripple in sand
            } else if roughness > 0.40 {
                '.' // fine sand grain
            } else {
                ' ' // seabed hollow
            };
            (ch, fg)
        }
        RoadStyle::Sidewalk => {
            let fg = Color::rgb(189, 189, 189);
            let slab = (((x as f32 + time * 6.5) % 10.0 + 10.0) % 10.0).floor() as usize;
            let ch = if slab == 0 {
                '|' // expansion joint
            } else if slab == 5 {
                '_'
            } else {
                '-' // smooth pavement
            };
            (ch, fg)
        }
        RoadStyle::Roof => {
            let fg = Color::rgb(120, 144, 156);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile == 0 { '#' } else { '=' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 229, 255);
            let node = (((x as f32 + time * 7.5) % 6.0 + 6.0) % 6.0).floor() as usize;
            let ch = if node == 0 { '+' } else { '-' };
            (ch, fg)
        }
        RoadStyle::Crypt => {
            let fg = Color::rgb(117, 117, 117);
            let ch = if roughness > 0.75 {
                'o' // ancient bone/skull fragment
            } else if roughness > 0.40 {
                '_' // stone slab seam
            } else {
                '.' // catacomb dust
            };
            (ch, fg)
        }
        RoadStyle::Savanna => {
            let fg = Color::rgb(215, 204, 200);
            let ch = if roughness > 0.70 {
                '_' // sun-cracked dry clay furrow
            } else if roughness > 0.40 {
                '.' // steppe dust
            } else {
                '-' // dry clay
            };
            (ch, fg)
        }
        RoadStyle::Mud => {
            let fg = Color::rgb(109, 76, 65);
            let ch = if roughness > 0.75 {
                'O' // mud bubble
            } else if roughness > 0.50 {
                'o' // small puddle
            } else {
                '.' // viscous slurry
            };
            (ch, fg)
        }
        RoadStyle::Tracks => {
            let fg = Color::rgb(144, 164, 174);
            let tie = (((x as f32 + time * 8.0) % 5.0 + 5.0) % 5.0).floor() as usize;
            let ch = if tie == 0 { '|' } else { '=' };
            (ch, fg)
        }
        RoadStyle::Checkerboard => {
            let fg = Color::rgb(238, 238, 238);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile < 2 { '#' } else { ' ' };
            (ch, fg)
        }
        RoadStyle::None => (' ', Color::rgb(0, 0, 0)),
    }
}

/// Generate subsurface foundation bed texture for realistic multi-row ground surfaces.
pub fn calculate_road_subsurface(x: usize, style: RoadStyle, time: f32) -> (char, Color) {
    let x_f = x as f32;
    let r1 = (x_f * 0.37 + time * 1.5).sin();
    let r2 = (x_f * 0.83 - time * 0.6).cos();
    let roughness = (r1 * 0.6 + r2 * 0.4).abs();

    match style {
        RoadStyle::Dirt => {
            let fg = Color::rgb(109, 76, 65);
            let ch = if roughness > 0.70 {
                ',' // root fiber / embedded pebble
            } else if roughness > 0.35 {
                ';' // packed subsoil loam
            } else {
                '~' // earth sub-layer
            };
            (ch, fg)
        }
        RoadStyle::Cobblestone => {
            let fg = Color::rgb(97, 97, 97);
            let stone_idx = (((x as f32 + time * 7.5) % 5.0 + 5.0) % 5.0).floor() as usize;
            let ch = if stone_idx == 0 {
                '|' // mortar bed joint
            } else if stone_idx == 1 {
                '('
            } else if stone_idx == 4 {
                ')'
            } else {
                '-' // stone foundation paver
            };
            (ch, fg)
        }
        RoadStyle::Magma => {
            let fg = if roughness > 0.55 {
                Color::rgb(255, 214, 0) // subterranean incandescent rift
            } else {
                Color::rgb(213, 0, 0) // molten mantle
            };
            let ch = if roughness > 0.65 {
                '^' // spitting magma fountain
            } else if roughness > 0.30 {
                '~' // molten wave
            } else {
                '=' // cooling magma shelf
            };
            (ch, fg)
        }
        RoadStyle::Ice => {
            let fg = Color::rgb(128, 222, 234);
            let fissure = (((x as f32 + time * 4.5) % 7.0 + 7.0) % 7.0).floor() as usize;
            let ch = if fissure == 0 {
                '|' // deep ice fracture
            } else if roughness > 0.45 {
                '-' // compressed glacial shelf
            } else {
                '=' // blue ice bed
            };
            (ch, fg)
        }
        RoadStyle::Tracks => {
            let tie = (((x as f32 + time * 8.0) % 5.0 + 5.0) % 5.0).floor() as usize;
            if tie == 0 || tie == 1 {
                ('|', Color::rgb(93, 64, 55)) // heavy timber cross-tie
            } else {
                (
                    if roughness > 0.5 { '.' } else { '*' },
                    Color::rgb(120, 144, 156), // crushed stone track ballast
                )
            }
        }
        RoadStyle::Sidewalk => {
            let fg = Color::rgb(158, 158, 158);
            let slab = (((x as f32 + time * 6.5) % 10.0 + 10.0) % 10.0).floor() as usize;
            let ch = if slab == 0 {
                '|' // curb expansion seam
            } else {
                '_' // street curb stone ledge
            };
            (ch, fg)
        }
        RoadStyle::Seabed => {
            let fg = Color::rgb(212, 163, 115);
            let ch = if roughness > 0.60 {
                's' // burrowing marine sediment
            } else if roughness > 0.30 {
                '.' // dense sub-sand
            } else {
                '~' // undulating sediment bed
            };
            (ch, fg)
        }
        RoadStyle::Roof => {
            let fg = Color::rgb(121, 85, 72);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile == 0 { '|' } else { '/' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 230, 118);
            let node = (((x as f32 + time * 7.5) % 6.0 + 6.0) % 6.0).floor() as usize;
            let ch = if node == 0 { '+' } else { '|' };
            (ch, fg)
        }
        RoadStyle::Crypt => {
            let fg = Color::rgb(189, 189, 189);
            let ch = if roughness > 0.70 {
                'x' // ancient catacomb cross-bone
            } else if roughness > 0.35 {
                '=' // weathered crypt slab foundation
            } else {
                '.' // subterranean catacomb ash
            };
            (ch, fg)
        }
        RoadStyle::Savanna => {
            let fg = Color::rgb(161, 110, 80);
            let ch = if roughness > 0.60 {
                '~' // acacia subsurface root
            } else if roughness > 0.30 {
                '_' // compact red earth
            } else {
                '-' // arid soil horizon
            };
            (ch, fg)
        }
        RoadStyle::Mud => {
            let fg = Color::rgb(62, 39, 35);
            let ch = if roughness > 0.65 {
                'o' // bubbling mire
            } else if roughness > 0.35 {
                '~' // thick peat slurry
            } else {
                '=' // heavy bog bed
            };
            (ch, fg)
        }
        RoadStyle::Checkerboard => {
            let fg = Color::rgb(189, 189, 189);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile < 2 { ' ' } else { '#' };
            (ch, fg)
        }
        RoadStyle::None => (' ', Color::rgb(0, 0, 0)),
    }
}

/// Generate near curb gutter band texture for multi-band perspective gradient from horizon to near curb.
pub fn calculate_road_curb_gutter(x: usize, style: RoadStyle, time: f32) -> (char, Color) {
    let x_f = x as f32;
    let r1 = (x_f * 0.51 + time * 1.2).sin();
    let r2 = (x_f * 1.13 - time * 0.4).cos();
    let roughness = (r1 * 0.55 + r2 * 0.45).abs();

    match style {
        RoadStyle::Dirt => {
            let fg = Color::rgb(82, 54, 44);
            let ch = if roughness > 0.65 {
                '.'
            } else if roughness > 0.35 {
                ','
            } else {
                ' '
            };
            (ch, fg)
        }
        RoadStyle::Cobblestone => {
            let fg = Color::rgb(75, 75, 75);
            let slab = (((x as f32 + time * 7.5) % 5.0 + 5.0) % 5.0).floor() as usize;
            let ch = if slab == 0 { '|' } else { '_' };
            (ch, fg)
        }
        RoadStyle::Magma => {
            let fg = Color::rgb(150, 20, 10);
            let ch = if roughness > 0.60 { '~' } else { '.' };
            (ch, fg)
        }
        RoadStyle::Ice => {
            let fg = Color::rgb(92, 172, 184);
            let ch = if roughness > 0.50 { '=' } else { '-' };
            (ch, fg)
        }
        RoadStyle::Seabed => {
            let fg = Color::rgb(175, 130, 90);
            let ch = if roughness > 0.55 { '~' } else { '.' };
            (ch, fg)
        }
        RoadStyle::Sidewalk => {
            let fg = Color::rgb(110, 110, 110);
            let joint = (((x as f32 + time * 6.5) % 10.0 + 10.0) % 10.0).floor() as usize;
            let ch = if joint == 0 { '|' } else { ' ' };
            (ch, fg)
        }
        RoadStyle::Roof => {
            let fg = Color::rgb(90, 60, 50);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile == 0 { '|' } else { '_' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 160, 90);
            let node = (((x as f32 + time * 7.5) % 6.0 + 6.0) % 6.0).floor() as usize;
            let ch = if node == 0 { '+' } else { '-' };
            (ch, fg)
        }
        RoadStyle::Crypt => {
            let fg = Color::rgb(70, 70, 70);
            let ch = if roughness > 0.60 { '.' } else { ' ' };
            (ch, fg)
        }
        RoadStyle::Savanna => {
            let fg = Color::rgb(125, 82, 58);
            let ch = if roughness > 0.60 { '_' } else { '.' };
            (ch, fg)
        }
        RoadStyle::Mud => {
            let fg = Color::rgb(42, 28, 24);
            let ch = if roughness > 0.60 { '~' } else { '=' };
            (ch, fg)
        }
        RoadStyle::Tracks => {
            let fg = Color::rgb(72, 86, 96);
            let ch = if roughness > 0.50 { '.' } else { ',' };
            (ch, fg)
        }
        RoadStyle::Checkerboard => {
            let fg = Color::rgb(120, 120, 120);
            let tile = (((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0).floor() as usize;
            let ch = if tile < 2 { '#' } else { ' ' };
            (ch, fg)
        }
        RoadStyle::None => (' ', Color::rgb(0, 0, 0)),
    }
}

/// Render ground / road baseline directly at cow_bottom_y with procedural harmonic texture synthesis.
/// Supports multi-row realistic ground depth (contact surface + subsurface foundation bed + near curb gutter).
pub fn render_road(
    fb: &mut FrameBuffer,
    style: RoadStyle,
    cow_bottom_y: usize,
    width: usize,
    time: f32,
) {
    render_road_with_tod(fb, style, cow_bottom_y, width, time, TimeOfDay::Auto);
}

/// Render ground / road baseline with explicit solar illumination time of day.
pub fn render_road_with_tod(
    fb: &mut FrameBuffer,
    style: RoadStyle,
    cow_bottom_y: usize,
    width: usize,
    time: f32,
    tod: TimeOfDay,
) {
    if style == RoadStyle::None || cow_bottom_y >= fb.height || width == 0 {
        return;
    }

    let compute = crate::gpu_compute::get_compute_engine(None);
    let mut undulation = [0.0f32; 1024];
    let mut chunk_start = 0;
    while chunk_start < width {
        let chunk_len = (width - chunk_start).min(1024);
        compute.compute_road_undulation_chunk(
            chunk_start,
            chunk_len,
            cow_bottom_y as f32,
            1.0,
            time,
            &mut undulation[..chunk_len],
        );

        // Row 0: Contact surface
        for i in 0..chunk_len {
            let x = chunk_start + i;
            let (ch, fg) = calculate_road_texture(x, style, time);
            if ch != ' ' {
                let _ = fb.set(x, cow_bottom_y, Cell::new(ch, tod.apply_lighting(fg)));
            }
        }

        // Row 1: Subsurface foundation bed
        let subsurface_y = cow_bottom_y + 1;
        if subsurface_y < fb.height {
            for i in 0..chunk_len {
                let x = chunk_start + i;
                let (ch, fg) = calculate_road_subsurface(x, style, time);
                if ch != ' ' {
                    let _ = fb.set(x, subsurface_y, Cell::new(ch, tod.apply_lighting(fg)));
                }
            }
        }

        // Row 2: Near curb gutter band (multi-band perspective gradient to near curb)
        let curb_y = cow_bottom_y + 2;
        if curb_y < fb.height {
            for i in 0..chunk_len {
                let x = chunk_start + i;
                let (ch, fg) = calculate_road_curb_gutter(x, style, time);
                if ch != ' ' {
                    let _ = fb.set(x, curb_y, Cell::new(ch, tod.apply_lighting(fg)));
                }
            }
        }

        chunk_start += chunk_len;
    }
}

/// Render sparse procedural ground cover (grass tufts, clover patches, pebbles, wildflowers)
/// in open clearings between tree groves using Golden Ratio Weyl sequence spacing.
pub fn render_procedural_ground_cover(
    fb: &mut FrameBuffer,
    env: EnvironmentStyle,
    road_y: usize,
    width: usize,
    time: f32,
    tod: TimeOfDay,
) {
    if road_y == 0 || road_y >= fb.height || width == 0 {
        return;
    }
    let verge_y = road_y.saturating_sub(1);
    let scroll_x = time * 7.5;
    let weyl_step = 23.0_f32;
    let inv_phi = 0.618_033_988_7_f32;

    let min_world = (scroll_x - 10.0).max(0.0);
    let max_world = scroll_x + (width as f32) + 10.0;
    let start_k = (min_world / weyl_step).floor() as usize;
    let end_k = (max_world / weyl_step).ceil() as usize + 2;

    for k in start_k..=end_k {
        let k_f = k as f32;
        let weyl_offset = ((k_f * inv_phi).fract()) * 19.0;
        let world_x = k_f * weyl_step + weyl_offset;
        let screen_x = ((world_x - scroll_x).round()) as isize;

        if screen_x < 0 || screen_x as usize >= width {
            continue;
        }
        let x = screen_x as usize;

        let cell = fb.get_back(x, verge_y);
        if cell.ch != ' ' && cell.alpha != 0 {
            continue;
        }

        let hash = splitmix64_hash(k as u64 ^ 0xB529_7A4D);
        let variant = (hash % 100) as usize;

        let (ch, raw_col) = match env {
            EnvironmentStyle::Pasture => {
                if variant < 42 {
                    let g_ch = if (hash & 1) == 0 { ',' } else { '\'' };
                    (g_ch, Color::rgb(102, 187, 106)) // fresh meadow green grass tuft
                } else if variant < 68 {
                    ('%', Color::rgb(67, 160, 71)) // clover emerald patch
                } else if variant < 84 {
                    ('.', Color::rgb(176, 190, 197)) // smooth river pebble
                } else if variant < 94 {
                    ('*', Color::rgb(255, 235, 59)) // golden buttercup
                } else {
                    ('*', Color::rgb(186, 104, 200)) // lavender blossom
                }
            }
            EnvironmentStyle::Forest => {
                if variant < 50 {
                    (',', Color::rgb(76, 140, 74)) // woodland moss tuft
                } else if variant < 75 {
                    ('~', Color::rgb(160, 110, 60)) // fallen woodland leaf
                } else {
                    ('.', Color::rgb(141, 110, 99)) // loam pebble
                }
            }
            EnvironmentStyle::Savanna => {
                if variant < 60 {
                    (',', Color::rgb(218, 185, 107)) // dry steppe grass
                } else {
                    ('.', Color::rgb(188, 143, 100)) // arid desert stone
                }
            }
            EnvironmentStyle::Arctic => {
                if variant < 50 {
                    ('*', Color::rgb(224, 247, 250)) // frost ice crystal
                } else {
                    ('.', Color::rgb(207, 216, 220)) // snow drift ridge
                }
            }
            EnvironmentStyle::Graveyard => {
                if variant < 60 {
                    (';', Color::rgb(117, 117, 117)) // weathered dried grass
                } else {
                    ('.', Color::rgb(158, 158, 158)) // ancient gravestone chip
                }
            }
            _ => continue,
        };

        let lit_col = tod.apply_lighting(raw_col);
        let _ = fb.set(x, verge_y, Cell::new(ch, lit_col));
    }
}

/// Render atmospheric environment particles (clouds, embers, snowflakes, stars, bubbles)
/// grounded at `ground_y`.
pub fn render_environment(
    fb: &mut FrameBuffer,
    style: EnvironmentStyle,
    ground_y: usize,
    width: usize,
    height: usize,
    time: f32,
) {
    render_environment_with_tod(fb, style, ground_y, width, height, time, TimeOfDay::Auto);
}

/// Render atmospheric environment particles with explicit time-of-day solar illumination.
pub fn render_environment_with_tod(
    fb: &mut FrameBuffer,
    style: EnvironmentStyle,
    ground_y: usize,
    width: usize,
    height: usize,
    time: f32,
    tod: TimeOfDay,
) {
    if style == EnvironmentStyle::None || width == 0 || height == 0 {
        return;
    }
    let tod_resolved = tod.resolve();
    let compute = crate::gpu_compute::get_compute_engine(None);

    match style {
        EnvironmentStyle::Pasture => {
            let sky_horizon = (ground_y / 3).max(2).min(fb.height.saturating_sub(1));
            match tod_resolved {
                TimeOfDay::Dawn => {
                    let sun_fg = Color::rgb(255, 167, 38);
                    let (sun_x, sky_y) =
                        compute.compute_celestial_trajectory(0, time, width, sky_horizon);
                    if sun_x < width && sky_y < fb.height {
                        let _ = fb.set(sun_x, sky_y, Cell::new('*', sun_fg));
                    }
                    let cloud_fg = Color::rgb(255, 224, 230);
                    let cloud_pattern = ".--.";
                    let c_span = width + 8;
                    let c_pos = ((time * 3.5).round() as usize) % c_span.max(1);
                    if c_pos >= 4 {
                        let start_x = c_pos - 4;
                        let c_y = (ground_y / 4).clamp(1, fb.height.saturating_sub(1));
                        if c_y < fb.height && c_y != sky_y {
                            for (i, ch) in cloud_pattern.chars().enumerate() {
                                let x = start_x + i;
                                if x < width && x != sun_x {
                                    let _ = fb.set(x, c_y, Cell::new(ch, cloud_fg));
                                }
                            }
                        }
                    }
                }
                TimeOfDay::Day | TimeOfDay::Auto => {
                    let sun_fg = Color::rgb(255, 214, 0);
                    let (sun_x, sky_y) =
                        compute.compute_celestial_trajectory(1, time, width, sky_horizon);
                    if sun_x < width && sky_y < fb.height {
                        let _ = fb.set(sun_x, sky_y, Cell::new('*', sun_fg));
                    }
                    let cloud_fg = Color::rgb(240, 245, 255);
                    let cloud_pattern = ".--.";
                    let c_span = width + 8;
                    let c_pos = ((time * 4.0).round() as usize) % c_span.max(1);
                    if c_pos >= 4 {
                        let start_x = c_pos - 4;
                        let c_y = (ground_y / 4).clamp(1, fb.height.saturating_sub(1));
                        if c_y < fb.height && c_y != sky_y {
                            for (i, ch) in cloud_pattern.chars().enumerate() {
                                let x = start_x + i;
                                if x < width && x != sun_x {
                                    let _ = fb.set(x, c_y, Cell::new(ch, cloud_fg));
                                }
                            }
                        }
                    }
                }
                TimeOfDay::Dusk => {
                    let sun_fg = Color::rgb(255, 110, 64);
                    let (sun_x, sky_y) =
                        compute.compute_celestial_trajectory(2, time, width, sky_horizon);
                    if sun_x < width && sky_y < fb.height {
                        let _ = fb.set(sun_x, sky_y, Cell::new('*', sun_fg));
                    }
                    let cloud_fg = Color::rgb(209, 196, 233);
                    let cloud_pattern = ".~~.";
                    let c_span = width + 8;
                    let c_pos = ((time * 3.0).round() as usize) % c_span.max(1);
                    if c_pos >= 4 {
                        let start_x = c_pos - 4;
                        let c_y = (ground_y / 4).clamp(1, fb.height.saturating_sub(1));
                        if c_y < fb.height && c_y != sky_y {
                            for (i, ch) in cloud_pattern.chars().enumerate() {
                                let x = start_x + i;
                                if x < width && x != sun_x {
                                    let _ = fb.set(x, c_y, Cell::new(ch, cloud_fg));
                                }
                            }
                        }
                    }
                    let star_fg = Color::rgb(255, 249, 196);
                    for x in (width.saturating_sub(25)..width).step_by(12) {
                        let _ = fb.set(x, 1, Cell::new('.', star_fg));
                    }
                }
                TimeOfDay::Night => {
                    let moon_fg = Color::rgb(255, 249, 196);
                    let (mx, sky_y) =
                        compute.compute_celestial_trajectory(3, time, width, sky_horizon);
                    if mx + 1 < width && sky_y < fb.height {
                        let _ = fb.set(mx, sky_y, Cell::new('(', moon_fg));
                        let _ = fb.set(mx + 1, sky_y, Cell::new(')', moon_fg));
                    }
                    let star_fg_bright = Color::rgb(255, 255, 220);
                    let star_fg_dim = Color::rgb(160, 180, 220);
                    for (idx, x) in (3..width.saturating_sub(2)).step_by(12).enumerate() {
                        let sy = (1 + (x * 7 + idx * 3) % 4).clamp(1, fb.height.saturating_sub(1));
                        let twinkle = ((time * 2.8 + (x as f32) * 0.785).sin() * 0.5 + 0.5) > 0.5;
                        let (ch, col) = if twinkle {
                            ('*', star_fg_bright)
                        } else {
                            ('.', star_fg_dim)
                        };
                        if x != mx && x != mx + 1 {
                            let _ = fb.set(x, sy, Cell::new(ch, col));
                        }
                    }
                }
            }
        }
        EnvironmentStyle::Inferno => {
            // Rising sparks and embers ascending from the magma ground
            let ember_fg = Color::rgb(255, 171, 0);
            for i in 0..6 {
                let seed = i * 17;
                let x = ((seed as f32 + time * 12.0).round() as usize) % width;
                let rise = ((time * 7.5 + (i * 2) as f32).round() as usize) % (ground_y.max(1) + 1);
                let y = ground_y.saturating_sub(rise);
                let ch = if i % 2 == 0 { '*' } else { '^' };
                let _ = fb.set(x, y, Cell::new(ch, ember_fg));
            }
        }
        EnvironmentStyle::Ocean => {
            // Render surface ocean waves horizon across the water boundary
            let surface_y = if ground_y >= 6 { 1 } else { 0 };
            let wave_fg = Color::rgb(72, 202, 228); // Luminous sea surface cyan
            for x in 0..width {
                let wave_harmonic = (x as f32 * 0.28 + time * 3.5).sin() * 0.45;
                let wave_pos = ((x as f32 + time * 6.0) % 4.0 + 4.0) % 4.0;
                let wave_ch = if wave_pos < 2.0 { '~' } else { '-' };
                let y_wave = ((surface_y as f32 + wave_harmonic).round() as usize)
                    .min(ground_y.saturating_sub(1));
                let _ = fb.set(x, y_wave, Cell::new(wave_ch, wave_fg));
            }

            // Rising bubbles ascending through the water column from seabed to surface
            let bubble_fg = Color::rgb(128, 222, 234);
            for i in 0..8 {
                let seed = i * 19;
                let x = ((seed + i * 11) as f32 + time * 5.0).round() as usize % width;
                let rise = ((time * 6.0 + (i * 2) as f32).round() as usize)
                    % (ground_y.saturating_sub(surface_y).max(1) + 1);
                let y = ground_y.saturating_sub(rise);
                if y > surface_y && y < ground_y {
                    let ch = if i % 2 == 0 { 'o' } else { '.' };
                    let _ = fb.set(x, y, Cell::new(ch, bubble_fg));
                }
            }
        }
        EnvironmentStyle::Arctic => {
            // Drifting snowflakes settling toward the ice road
            let snow_fg = Color::rgb(255, 255, 255);
            for i in 0..8 {
                let seed = i * 19;
                let x = ((seed + i) as f32 + time * 4.5).round() as usize % width;
                let y = ((time * 5.5 + seed as f32).round() as usize) % (ground_y.max(1) + 1);
                let ch = if i % 3 == 0 { '*' } else { '.' };
                let _ = fb.set(x, y, Cell::new(ch, snow_fg));
            }
        }
        EnvironmentStyle::City => {
            // Crescent moon and stars in the night sky above the skyline
            let moon_fg = Color::rgb(255, 245, 157);
            let mx = width.saturating_sub(6);
            let sky_y = (ground_y / 5).clamp(1, fb.height.saturating_sub(1));
            if mx < width && sky_y < fb.height {
                let _ = fb.set(mx, sky_y, Cell::new('(', moon_fg));
                if mx + 1 < width {
                    let _ = fb.set(mx + 1, sky_y, Cell::new(')', moon_fg));
                }
            }
            let star_fg = Color::rgb(255, 255, 255);
            for x in (3..width).step_by(14) {
                let _ = fb.set(x, sky_y, Cell::new('.', star_fg));
            }
            if ground_y >= 12 {
                let sky_y2 = (ground_y / 3).clamp(1, fb.height.saturating_sub(1));
                if sky_y2 != sky_y {
                    for x in (10..width).step_by(18) {
                        let _ = fb.set(x, sky_y2, Cell::new('*', star_fg));
                    }
                }
            }
        }
        EnvironmentStyle::Space => {
            // Twinkling starfield in the background cosmos
            let star_fg = Color::rgb(224, 64, 251);
            for i in 0..10 {
                let x = (i * 29 + 3) % width;
                let y = (i * 7 + 1) % (ground_y.max(1) + 1);
                let ch = if i % 3 == 0 { '+' } else { '.' };
                let _ = fb.set(x, y, Cell::new(ch, star_fg));
            }
        }
        EnvironmentStyle::Graveyard => {
            // Mist wisps floating along the crypt road
            let mist_fg = Color::rgb(128, 203, 196);
            let offset = ((time * 4.5).round() as usize) % 8;
            for x in (offset..width).step_by(12) {
                let _ = fb.set(x, ground_y, Cell::new('~', mist_fg));
            }
        }
        EnvironmentStyle::Forest => {
            // Leaves drifting down from canopy toward the woodland path
            let leaf_fg = Color::rgb(255, 179, 0);
            for i in 0..5 {
                let x = ((i * 31) as f32 + time * 6.0).round() as usize % width;
                let y = ((time * 4.5 + (i * 3) as f32).round() as usize) % (ground_y.max(1) + 1);
                let _ = fb.set(x, y, Cell::new('~', leaf_fg));
            }
        }
        EnvironmentStyle::Savanna => {
            // Heat shimmering waves above the savanna trail
            let heat_fg = Color::rgb(255, 213, 79);
            let heat_y = ground_y.saturating_sub(1);
            for x in (2..width).step_by(15) {
                let _ = fb.set(x, heat_y, Cell::new('~', heat_fg));
            }
        }
        EnvironmentStyle::Swamp => {
            // Fireflies flickering over the muddy ground
            let fly_fg = Color::rgb(205, 220, 57);
            if ((time * 6.0).round() as usize).is_multiple_of(2) {
                for i in 0..4 {
                    let x = (i * 23 + 7) % width;
                    let y = ground_y.saturating_sub(1 + (i % 3));
                    let _ = fb.set(x, y, Cell::new('*', fly_fg));
                }
            }
        }
        EnvironmentStyle::Cyber => {
            // Digital matrix rain descending to the cyber grid
            let matrix_fg = Color::rgb(0, 230, 118);
            for i in 0..6 {
                let x = (i * 19 + 5) % width;
                let y = ((time * 9.0 + (i * 2) as f32).round() as usize) % (ground_y.max(1) + 1);
                let ch = if i.is_multiple_of(2) { '0' } else { '1' };
                let _ = fb.set(x, y, Cell::new(ch, matrix_fg));
            }
        }
        EnvironmentStyle::Jurassic => {
            // Prehistoric fern foliage dotting the dirt road
            let fern_fg = Color::rgb(104, 159, 56);
            for x in (4..width).step_by(16) {
                let _ = fb.set(x, ground_y, Cell::new('%', fern_fg));
            }
        }
        EnvironmentStyle::Hive => {
            // Pollen motes hovering around the floral ground
            let pollen_fg = Color::rgb(255, 235, 59);
            for i in 0..5 {
                let x = ((i * 27) as f32 + time * 7.5).round() as usize % width;
                let y = (i * 3) % (ground_y.max(1) + 1);
                let _ = fb.set(x, y, Cell::new('.', pollen_fg));
            }
        }
        EnvironmentStyle::Throne => {
            // Torch spark flicker beside the royal carpet
            let torch_fg = Color::rgb(255, 112, 67);
            if ((time * 8.0).round() as usize).is_multiple_of(2) && width > 4 {
                let ty = ground_y.saturating_sub(1);
                let _ = fb.set(2, ty, Cell::new('*', torch_fg));
                let _ = fb.set(width - 3, ty, Cell::new('*', torch_fg));
            }
        }
        EnvironmentStyle::None => {}
    }
}

/// Composite full scenery (Mountain, Environment, Road) into the FrameBuffer.
/// `road_y` is the row immediately below the creature's legs.
pub fn render_scenery(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    road: RoadStyle,
    env: EnvironmentStyle,
    road_y: usize,
    time: f32,
) {
    render_scenery_full(fb, mountain, road, env, road_y, time, None, None, None);
}

/// Composite full scenery with explicit animal-relative height and kinematic locomotion properties.
#[allow(clippy::too_many_arguments)]
pub fn render_scenery_full(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    road: RoadStyle,
    env: EnvironmentStyle,
    road_y: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
) {
    render_scenery_full_with_tod(
        fb,
        mountain,
        road,
        env,
        road_y,
        time,
        animal_name,
        animal_height,
        base_anim,
        None,
    );
}

/// Composite full scenery with explicit animal kinematics, clearing ground cover, and solar illumination.
#[allow(clippy::too_many_arguments)]
pub fn render_scenery_full_with_tod(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    road: RoadStyle,
    env: EnvironmentStyle,
    road_y: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
    time_of_day: Option<TimeOfDay>,
) {
    render_scenery_full_with_tod_and_season(
        fb,
        mountain,
        road,
        env,
        road_y,
        time,
        animal_name,
        animal_height,
        base_anim,
        time_of_day,
        None,
    );
}

/// Composite full scenery with explicit animal kinematics, clearing ground cover, solar illumination, and seasonal phenology.
#[allow(clippy::too_many_arguments)]
pub fn render_scenery_full_with_tod_and_season(
    fb: &mut FrameBuffer,
    mountain: MountainStyle,
    road: RoadStyle,
    env: EnvironmentStyle,
    road_y: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
    time_of_day: Option<TimeOfDay>,
    season: Option<Season>,
) {
    let w = fb.width;
    let h = fb.height;
    if w == 0 || h == 0 {
        return;
    }

    let tod = time_of_day.unwrap_or_default().resolve();
    let current_season = season.unwrap_or_default().resolve();
    let mountain_base_y = road_y.saturating_sub(1);
    render_mountain_with_tod(fb, mountain, mountain_base_y, w, time, tod);
    render_trees_with_animal_and_tod_and_season(
        fb,
        mountain,
        env,
        mountain_base_y,
        w,
        time,
        animal_name,
        animal_height,
        base_anim,
        Some(tod),
        Some(current_season),
    );
    render_procedural_ground_cover(fb, env, road_y, w, time, tod);
    render_environment_with_tod(fb, env, road_y, w, h, time, tod);
    render_road_with_tod(fb, road, road_y, w, time, tod);
}

/// Compress background scenery luminance to maintain >3.5:1 figure-ground contrast against foreground mascots.
#[inline]
pub fn compress_scenery_contrast(fb: &mut FrameBuffer) {
    for cell in fb.back.iter_mut() {
        if cell.ch != ' ' && cell.fg != Color::WHITE {
            cell.fg = cell.fg.scale_luminance(0.52);
        }
    }
}

/// Scenery cache key to determine whether background scenery can be reused without recalculating
/// trigonometric alpine crests, road undulation, and Fibonacci trees.
#[derive(Debug, Clone, PartialEq)]
pub struct SceneryCacheKey {
    pub width: usize,
    pub height: usize,
    pub mountain: MountainStyle,
    pub road: RoadStyle,
    pub env: EnvironmentStyle,
    pub road_y: usize,
    pub tod: TimeOfDay,
    pub season: Season,
    pub animal_height: usize,
    pub time_bucket: u32,
}

/// Memoization cache for procedural background scenery (PERF-001).
///
/// Avoids recalculating trigonometric alpine crests, road undulations, and Fibonacci foliage
/// on every frame at 240Hz when camera coordinates, season, time-of-day, and dimensions are stationary.
#[derive(Debug, Clone, Default)]
pub struct SceneryCache {
    key: Option<SceneryCacheKey>,
    cached_cells: Vec<Cell>,
}

impl SceneryCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear cache entries.
    pub fn clear(&mut self) {
        self.key = None;
        self.cached_cells.clear();
    }

    /// Render scenery into `fb`, reading from the memoization cache when stationary.
    #[allow(clippy::too_many_arguments)]
    pub fn get_or_render(
        &mut self,
        fb: &mut FrameBuffer,
        mountain: MountainStyle,
        road: RoadStyle,
        env: EnvironmentStyle,
        road_y: usize,
        time: f32,
        animal_name: Option<&str>,
        animal_height: Option<usize>,
        base_anim: Option<BaseAnim>,
        time_of_day: Option<TimeOfDay>,
        season: Option<Season>,
    ) {
        let w = fb.width;
        let h = fb.height;
        if w == 0 || h == 0 {
            return;
        }

        let tod = time_of_day.unwrap_or_default().resolve();
        let cur_season = season.unwrap_or_default().resolve();
        let anim_h = animal_height.unwrap_or(12);

        // Quantize time into 100ms buckets so elements drift naturally while
        // caching ~96% of frames at 240Hz.
        let time_bucket = (time * 10.0).floor() as u32;

        let key = SceneryCacheKey {
            width: w,
            height: h,
            mountain,
            road,
            env,
            road_y,
            tod,
            season: cur_season,
            animal_height: anim_h,
            time_bucket,
        };

        if self.key.as_ref() == Some(&key) && self.cached_cells.len() == w * h {
            // Cache hit: populate back buffer from cached cells
            for y in 0..h {
                for x in 0..w {
                    let cell = self.cached_cells[y * w + x];
                    if cell.alpha > 0 || cell.ch != ' ' {
                        let _ = fb.set(x, y, cell);
                    }
                }
            }
            return;
        }

        // Cache miss: render freshly
        render_scenery_full_with_tod_and_season(
            fb,
            mountain,
            road,
            env,
            road_y,
            time,
            animal_name,
            Some(anim_h),
            base_anim,
            Some(tod),
            Some(cur_season),
        );

        // Snapshot generated scenery cells
        self.cached_cells.clear();
        self.cached_cells.reserve(w * h);
        for y in 0..h {
            for x in 0..w {
                self.cached_cells.push(fb.get_back(x, y));
            }
        }
        self.key = Some(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mountain_mathematics() {
        let peaks_peaks = calculate_mountain_peaks(80, MountainStyle::Peaks);
        let hills_peaks = calculate_mountain_peaks(80, MountainStyle::Hills);
        let volcano_peaks = calculate_mountain_peaks(80, MountainStyle::Volcano);
        assert!(peaks_peaks >= 2);
        assert!(hills_peaks >= 1);
        assert!(volcano_peaks >= 1);

        let h1 = calculate_mountain_height(10.0, 80, 15, MountainStyle::Peaks, 0.0);
        let h2 = calculate_mountain_height(20.0, 80, 15, MountainStyle::Peaks, 0.0);
        assert!(h1 > 0.0 && h1 <= 15.0);
        assert!(h2 > 0.0 && h2 <= 15.0);

        // Active crest elevation vs natural geological valley gap (elevation 0.0)
        let h_crest = calculate_mountain_height(30.0, 80, 15, MountainStyle::Peaks, 0.0);
        let h_gap = calculate_mountain_height(100.0, 80, 15, MountainStyle::Peaks, 0.0);
        assert!(h_crest > 0.0);
        assert_eq!(h_gap, 0.0);

        let env_crest = calculate_mountain_range_envelope(30.0, MountainStyle::Peaks);
        let env_gap = calculate_mountain_range_envelope(100.0, MountainStyle::Peaks);
        assert!(env_crest > 0.0);
        assert_eq!(env_gap, 0.0);
    }

    #[test]
    fn test_tree_mathematics_distance_and_clustering() {
        let d0 = calculate_inter_tree_distance(0, 16.0, 12.0);
        let d1 = calculate_inter_tree_distance(1, 16.0, 12.0);
        let d2 = calculate_inter_tree_distance(2, 16.0, 12.0);
        assert!((11.0..=30.0).contains(&d0));
        assert!((11.0..=30.0).contains(&d1));
        assert!((11.0..=30.0).contains(&d2));

        // Verify monotonic positioning and multi-tree span
        let p0 = calculate_tree_position(0, 16.0, 12.0);
        let p1 = calculate_tree_position(1, 16.0, 12.0);
        let p2 = calculate_tree_position(2, 16.0, 12.0);
        assert!(p1 > p0);
        assert!(p2 > p1);

        let span_0_2 = calculate_multi_tree_span(0, 2, 16.0, 12.0);
        assert_eq!(span_0_2, p2 - p0);
    }

    #[test]
    fn test_tree_count_and_density() {
        let forest_trees = calculate_tree_count(100, EnvironmentStyle::Forest, 8);
        let savanna_trees = calculate_tree_count(100, EnvironmentStyle::Savanna, 8);
        let city_trees = calculate_tree_count(100, EnvironmentStyle::City, 8);
        let none_trees = calculate_tree_count(100, EnvironmentStyle::None, 8);
        assert!(forest_trees > savanna_trees);
        assert!(city_trees > 0);
        assert_eq!(none_trees, 0);

        // Colossal mascot vs tiny mascot scale factor
        let colossal_trees = calculate_tree_count(100, EnvironmentStyle::Pasture, 16);
        let tiny_trees = calculate_tree_count(100, EnvironmentStyle::Pasture, 3);
        assert!(tiny_trees >= colossal_trees);
    }

    #[test]
    fn test_procedural_tree_foliage_colors_and_sfumato() {
        // Genetic variation between different tree indices
        let c0 = get_tree_foliage_color(TreeSpecies::Oak, 0, 0, 8, EnvironmentStyle::Forest);
        let c1 = get_tree_foliage_color(TreeSpecies::Oak, 1, 0, 8, EnvironmentStyle::Forest);
        let c2 = get_tree_foliage_color(TreeSpecies::Oak, 2, 0, 8, EnvironmentStyle::Forest);
        // Not all trees share identical color
        assert!(c0 != c1 || c1 != c2);

        // Vertical da Vinci Sfumato gradient: apex (dy = 0) is brighter than deep base (dy = crown_h)
        let c_apex = get_tree_foliage_color(TreeSpecies::Oak, 0, 0, 8, EnvironmentStyle::Forest);
        let c_base = get_tree_foliage_color(TreeSpecies::Oak, 0, 7, 8, EnvironmentStyle::Forest);
        let lum_apex = c_apex.r as u32 + c_apex.g as u32 + c_apex.b as u32;
        let lum_base = c_base.r as u32 + c_base.g as u32 + c_base.b as u32;
        assert!(
            lum_apex > lum_base,
            "Apex highlights must be brighter than shaded base"
        );
    }

    #[test]
    fn test_tree_species_sequencing() {
        let s0 = calculate_tree_species(0, EnvironmentStyle::Pasture, 42);
        let s1 = calculate_tree_species(1, EnvironmentStyle::Pasture, 42);
        let s2 = calculate_tree_species(2, EnvironmentStyle::Pasture, 42);
        let pasture_pool = get_biome_species_pool(EnvironmentStyle::Pasture);
        assert!(pasture_pool.contains(&s0));
        assert!(pasture_pool.contains(&s1));
        assert!(pasture_pool.contains(&s2));

        let arctic_s = calculate_tree_species(0, EnvironmentStyle::Arctic, 10);
        let arctic_pool = get_biome_species_pool(EnvironmentStyle::Arctic);
        assert!(arctic_pool.contains(&arctic_s));
    }

    #[test]
    fn test_22_species_sequencing_and_biome_filtering() {
        // 1. Verify that ALL_SPECIES contains all 22 distinct species
        assert_eq!(TreeSpecies::ALL_SPECIES.len(), 22);
        for sp in TreeSpecies::ALL_SPECIES {
            assert!(!sp.as_str().is_empty());
        }

        // 2. Biome filtering invariants:
        // Cold / Arctic: Conifers (SnowFir, Pine, Cypress), hardy shrubs. Zero creepers/vines.
        let arctic_pool = get_biome_species_pool(EnvironmentStyle::Arctic);
        assert!(arctic_pool.contains(&TreeSpecies::SnowFir));
        assert!(arctic_pool.contains(&TreeSpecies::Pine));
        assert!(arctic_pool.contains(&TreeSpecies::Cypress));
        assert!(!arctic_pool.contains(&TreeSpecies::CreeperVine));

        // Water / Ocean / Beach: Palm, Mangrove, aquatic reeds. Zero temperate creepers.
        let ocean_pool = get_biome_species_pool(EnvironmentStyle::Ocean);
        assert!(ocean_pool.contains(&TreeSpecies::Palm));
        assert!(ocean_pool.contains(&TreeSpecies::Mangrove));
        assert!(!ocean_pool.contains(&TreeSpecies::CreeperVine));

        // Savanna / Desert: Acacia, Baobab, Cactus, sparse arid herbs, DeadTree.
        let savanna_pool = get_biome_species_pool(EnvironmentStyle::Savanna);
        assert!(savanna_pool.contains(&TreeSpecies::Acacia));
        assert!(savanna_pool.contains(&TreeSpecies::Baobab));
        assert!(savanna_pool.contains(&TreeSpecies::Cactus));
        assert!(savanna_pool.contains(&TreeSpecies::DeadTree));

        // Graveyard: DeadTree, WeepingWillow, eerie Oak, thorny CreeperVine, Cypress.
        let grave_pool = get_biome_species_pool(EnvironmentStyle::Graveyard);
        assert!(grave_pool.contains(&TreeSpecies::DeadTree));
        assert!(grave_pool.contains(&TreeSpecies::WeepingWillow));
        assert!(grave_pool.contains(&TreeSpecies::Oak));
        assert!(grave_pool.contains(&TreeSpecies::CreeperVine));
        assert!(grave_pool.contains(&TreeSpecies::Cypress));

        // Forest / Pasture: Oak, Maple, Birch, Pine, CherryBlossom, BerryShrub, BloomingShrub, HerbalCluster.
        let forest_pool = get_biome_species_pool(EnvironmentStyle::Forest);
        assert!(forest_pool.contains(&TreeSpecies::Oak));
        assert!(forest_pool.contains(&TreeSpecies::Maple));
        assert!(forest_pool.contains(&TreeSpecies::Birch));
        assert!(forest_pool.contains(&TreeSpecies::Pine));
        assert!(forest_pool.contains(&TreeSpecies::CherryBlossom));
        assert!(forest_pool.contains(&TreeSpecies::BerryShrub));
        assert!(forest_pool.contains(&TreeSpecies::BloomingShrub));
        assert!(forest_pool.contains(&TreeSpecies::HerbalCluster));

        // 3. Low-discrepancy Weyl sequence natural mixed distribution test
        let mut sampled_species = std::collections::HashSet::new();
        for n in 0..40 {
            let sp = calculate_tree_species(n, EnvironmentStyle::Forest, 7);
            sampled_species.insert(sp);
        }
        assert!(
            sampled_species.len() >= 6,
            "Expected at least 6 species sampled across 40 trees, got {}",
            sampled_species.len()
        );
    }

    #[test]
    fn test_fruit_star_uniqueness_across_viewport_widths() {
        let mut all_tree_patterns = Vec::new();
        let mut colors_seen = std::collections::HashSet::new();

        for t_idx in 0..25 {
            let species = calculate_tree_species(t_idx, EnvironmentStyle::Forest, 42);
            let mut tree_ornaments = Vec::new();
            for dy in -8..=-2 {
                for dx in -5..=5 {
                    if let Some((ch, col)) = get_foliage_ornament(species, t_idx, dx, dy, '@') {
                        tree_ornaments.push((dx, dy, ch, col.r, col.g, col.b));
                        colors_seen.insert((col.r, col.g, col.b));
                    }
                }
            }
            if !tree_ornaments.is_empty() {
                for prev in &all_tree_patterns {
                    assert_ne!(
                        &tree_ornaments, prev,
                        "Tree {} produced duplicate ornament pattern with a prior tree!",
                        t_idx
                    );
                }
                all_tree_patterns.push(tree_ornaments);
            }
        }
        assert!(all_tree_patterns.len() >= 15);
        assert!(colors_seen.contains(&(
            FRUIT_RUBY_BERRY.r,
            FRUIT_RUBY_BERRY.g,
            FRUIT_RUBY_BERRY.b
        )));
        assert!(colors_seen.contains(&(
            FRUIT_GOLDEN_CITRUS.r,
            FRUIT_GOLDEN_CITRUS.g,
            FRUIT_GOLDEN_CITRUS.b
        )));
        assert!(colors_seen.contains(&(
            FRUIT_AMETHYST_PLUM.r,
            FRUIT_AMETHYST_PLUM.g,
            FRUIT_AMETHYST_PLUM.b
        )));
        assert!(colors_seen.contains(&(
            FRUIT_CHERRY_BLOSSOM.r,
            FRUIT_CHERRY_BLOSSOM.g,
            FRUIT_CHERRY_BLOSSOM.b
        )));
        assert!(colors_seen.contains(&(
            FRUIT_IVORY_STAR.r,
            FRUIT_IVORY_STAR.g,
            FRUIT_IVORY_STAR.b
        )));
    }

    #[test]
    fn test_trunk_thickness_and_multi_shade_coloring() {
        // 1. Allometric width scaling
        let rw_tall = calculate_trunk_width(TreeSpecies::Redwood, 9, 0);
        let rw_med = calculate_trunk_width(TreeSpecies::Redwood, 5, 0);
        assert_eq!(rw_tall, 3);
        assert_eq!(rw_med, 2);

        let bb_tall = calculate_trunk_width(TreeSpecies::Baobab, 8, 0);
        assert_eq!(bb_tall, 3);

        let herb_w = calculate_trunk_width(TreeSpecies::HerbalCluster, 8, 0);
        assert_eq!(herb_w, 1);

        // 2. Multi-shade bark palette coverage
        let mut bark_colors_seen = std::collections::HashSet::new();
        for sp in TreeSpecies::ALL_SPECIES {
            for y in 0..5 {
                let (_, col) = get_bark_glyph_and_color(*sp, 0, 1, y, 0);
                bark_colors_seen.insert((col.r, col.g, col.b));
            }
        }
        assert!(bark_colors_seen.contains(&(BARK_ESPRESSO.r, BARK_ESPRESSO.g, BARK_ESPRESSO.b)));
        assert!(bark_colors_seen.contains(&(BARK_CHESTNUT.r, BARK_CHESTNUT.g, BARK_CHESTNUT.b)));
        assert!(bark_colors_seen.contains(&(BARK_RAW_UMBER.r, BARK_RAW_UMBER.g, BARK_RAW_UMBER.b)));
        assert!(bark_colors_seen.contains(&(
            BARK_BIRCH_SILVER.r,
            BARK_BIRCH_SILVER.g,
            BARK_BIRCH_SILVER.b
        )));
        assert!(bark_colors_seen.contains(&(BARK_MOSSY.r, BARK_MOSSY.g, BARK_MOSSY.b)));
        assert!(bark_colors_seen.contains(&(
            BARK_TERRACOTTA.r,
            BARK_TERRACOTTA.g,
            BARK_TERRACOTTA.b
        )));

        // 3. Bark texturing characters produce requested procedural textures:
        // '||', '|/|', '(|)', '||', '!!', '|:|', '|;|'
        let (ch_w1, _) = get_bark_glyph_and_color(TreeSpecies::Pine, 0, 1, 0, 0);
        assert_eq!(ch_w1, '|');

        let (w2_0, _) = get_bark_glyph_and_color(TreeSpecies::Pine, 0, 2, 0, 0);
        let (w2_1, _) = get_bark_glyph_and_color(TreeSpecies::Pine, 1, 2, 0, 0);
        assert_eq!((w2_0, w2_1), ('|', '|')); // "||"

        let (w2_0_alt, _) = get_bark_glyph_and_color(TreeSpecies::Pine, 0, 2, 1, 0);
        let (w2_1_alt, _) = get_bark_glyph_and_color(TreeSpecies::Pine, 1, 2, 1, 0);
        assert_eq!((w2_0_alt, w2_1_alt), ('!', '!')); // "!!"

        let (w3_l0, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 0, 3, 0, 0);
        let (w3_m0, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 1, 3, 0, 0);
        let (w3_r0, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 2, 3, 0, 0);
        assert_eq!((w3_l0, w3_m0, w3_r0), ('|', '/', '|')); // "|/|"

        let (w3_l1, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 0, 3, 1, 0);
        let (w3_m1, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 1, 3, 1, 0);
        let (w3_r1, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 2, 3, 1, 0);
        assert_eq!((w3_l1, w3_m1, w3_r1), ('(', '|', ')')); // "(|)"

        let (w3_l2, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 0, 3, 2, 0);
        let (w3_m2, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 1, 3, 2, 0);
        let (w3_r2, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 2, 3, 2, 0);
        assert_eq!((w3_l2, w3_m2, w3_r2), ('|', ':', '|')); // "|:|"

        let (w3_l3, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 0, 3, 3, 0);
        let (w3_m3, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 1, 3, 3, 0);
        let (w3_r3, _) = get_bark_glyph_and_color(TreeSpecies::Redwood, 2, 3, 3, 0);
        assert_eq!((w3_l3, w3_m3, w3_r3), ('|', ';', '|')); // "|;|"
    }

    #[test]
    fn test_mountain_depth_shading_layers() {
        let base_fg = Color::rgb(64, 82, 92);
        let start_y = 5;
        let mountain_base_y = 20;

        // 1. Foreground crests: apex highlight is brighter than base
        let (ch_apex, col_apex) = calculate_mountain_depth_cell(
            MountainStyle::Peaks,
            base_fg,
            10,
            start_y,
            start_y,
            mountain_base_y,
            0.5,
            15.0,
            12.0,
            12.0,
        );
        assert_eq!(ch_apex, '^');
        assert!(col_apex.r > base_fg.r && col_apex.g > base_fg.g && col_apex.b > base_fg.b);

        // Leeward shadow facet is darker than base
        let (_ch_shadow, col_shadow) = calculate_mountain_depth_cell(
            MountainStyle::Peaks,
            base_fg,
            10,
            start_y,
            start_y,
            mountain_base_y,
            -0.8,
            10.0,
            12.0,
            8.0,
        );
        assert!(col_shadow.r < base_fg.r && col_shadow.g < base_fg.g && col_shadow.b < base_fg.b);

        // 2. Mid-ground foothills layer: strata / crevasse texturing
        let (ch_mid, _) = calculate_mountain_depth_cell(
            MountainStyle::Peaks,
            base_fg,
            10,
            10,
            start_y,
            mountain_base_y,
            0.0,
            10.0,
            10.0,
            10.0,
        );
        assert!(matches!(ch_mid, '=' | ':' | '|' | '/'));

        // 3. Distant mountain layer: Leonardo da Vinci sfumato desaturation & mist textures
        let (ch_base, col_base) = calculate_mountain_depth_cell(
            MountainStyle::Peaks,
            base_fg,
            10,
            mountain_base_y,
            start_y,
            mountain_base_y,
            0.0,
            5.0,
            5.0,
            5.0,
        );
        assert_eq!(ch_base, '.');
        let dist_to_slate_base = (col_base.r as i32 - SFUMATO_SLATE.r as i32).abs();
        let dist_to_slate_orig = (base_fg.r as i32 - SFUMATO_SLATE.r as i32).abs();
        assert!(dist_to_slate_base <= dist_to_slate_orig);
    }

    #[test]
    fn test_tree_height_wrt_animal_properties() {
        let base_y = 20;
        // Tall mascot (dragon, height 14) vs small mascot (bunny, height 3)
        let dragon_tree_h = calculate_tree_height(0, 14, BaseAnim::Breathe, base_y);
        let bunny_tree_h = calculate_tree_height(0, 3, BaseAnim::Walk, base_y);
        assert!(dragon_tree_h > bunny_tree_h);

        // Airborne mascot (Float / Fly) hovers above canopy: tree height is scaled lower
        let fly_tree_h = calculate_tree_height(0, 10, BaseAnim::Fly, base_y);
        let walk_tree_h = calculate_tree_height(0, 10, BaseAnim::Walk, base_y);
        assert!(fly_tree_h < walk_tree_h);
    }

    #[test]
    fn test_road_texture_synthesis() {
        let (ch1, col1) = calculate_road_texture(0, RoadStyle::Dirt, 0.0);
        let (ch2, col2) = calculate_road_texture(5, RoadStyle::Cobblestone, 1.0);
        let (ch3, col3) = calculate_road_texture(10, RoadStyle::Magma, 2.0);
        assert_ne!(ch1, '\0');
        assert_ne!(ch2, '\0');
        assert_ne!(ch3, '\0');
        assert!(col1.r > 0 || col1.g > 0 || col1.b > 0);
        assert!(col2.r > 0 || col2.g > 0 || col2.b > 0);
        assert!(col3.r > 0 || col3.g > 0 || col3.b > 0);
    }

    #[test]
    fn parse_styles() {
        assert_eq!(MountainStyle::parse("volcano"), MountainStyle::Volcano);
        assert_eq!(MountainStyle::parse("HILLS"), MountainStyle::Hills);
        assert_eq!(MountainStyle::parse("none"), MountainStyle::None);
        assert_eq!(RoadStyle::parse("magma"), RoadStyle::Magma);
        assert_eq!(RoadStyle::parse("ice"), RoadStyle::Ice);
        assert_eq!(
            EnvironmentStyle::parse("inferno"),
            EnvironmentStyle::Inferno
        );
        assert_eq!(EnvironmentStyle::parse("arctic"), EnvironmentStyle::Arctic);

        // Test random styles produce valid non-none styles
        let rand_mtn = MountainStyle::parse("random");
        assert_ne!(rand_mtn, MountainStyle::None);
        let rand_mtn_upper = MountainStyle::parse("RANDOM");
        assert_ne!(rand_mtn_upper, MountainStyle::None);

        let rand_road = RoadStyle::parse("random");
        assert_ne!(rand_road, RoadStyle::None);
        let rand_road_upper = RoadStyle::parse("RANDOM");
        assert_ne!(rand_road_upper, RoadStyle::None);

        let rand_env = EnvironmentStyle::parse("random");
        assert_ne!(rand_env, EnvironmentStyle::None);
        let rand_env_upper = EnvironmentStyle::parse("RANDOM");
        assert_ne!(rand_env_upper, EnvironmentStyle::None);
    }

    #[test]
    fn resolve_archetypes_for_known_animals() {
        let (m, r, e) = resolve_archetype("dragon");
        assert_eq!(m, MountainStyle::Volcano);
        assert_eq!(r, RoadStyle::Magma);
        assert_eq!(e, EnvironmentStyle::Inferno);

        let (m, r, e) = resolve_archetype("tux");
        assert_eq!(m, MountainStyle::Iceberg);
        assert_eq!(r, RoadStyle::Ice);
        assert_eq!(e, EnvironmentStyle::Arctic);

        let (m, r, e) = resolve_archetype("happy-whale");
        assert_eq!(m, MountainStyle::Seamount);
        assert_eq!(r, RoadStyle::Seabed);
        assert_eq!(e, EnvironmentStyle::Ocean);

        let (_m, _r, e) = resolve_archetype("bud-frogs");
        assert_eq!(e, EnvironmentStyle::Swamp);

        let (m, _r, e) = resolve_archetype("cat");
        assert_eq!(m, MountainStyle::Hills);
        assert_eq!(e, EnvironmentStyle::Pasture);

        let (m, _r, e) = resolve_archetype("cat2");
        assert_eq!(m, MountainStyle::Skyline);
        assert_eq!(e, EnvironmentStyle::City);

        let (m, r, e) = resolve_archetype("cow");
        assert_eq!(m, MountainStyle::Hills);
        assert_eq!(r, RoadStyle::Dirt);
        assert_eq!(e, EnvironmentStyle::Pasture);
    }

    #[test]
    fn render_scenery_does_not_panic() {
        let mut fb = FrameBuffer::new(40, 10);
        render_scenery(
            &mut fb,
            MountainStyle::Volcano,
            RoadStyle::Magma,
            EnvironmentStyle::Inferno,
            8,
            1.0,
        );
        // Ensure some cells were written
        assert!(fb.back.iter().any(|c| c.alpha != 0));
    }

    #[test]
    fn test_mountain_mathematics_with_aspect_ratio_formats() {
        let width = 120;
        let p_uw = calculate_mountain_peaks_aspect(
            width,
            MountainStyle::Peaks,
            Some(AspectRatioFormat::UltraWide21x9),
        );
        let p_ws = calculate_mountain_peaks_aspect(
            width,
            MountainStyle::Peaks,
            Some(AspectRatioFormat::Widescreen16x9),
        );
        let p_sq = calculate_mountain_peaks_aspect(
            width,
            MountainStyle::Peaks,
            Some(AspectRatioFormat::Square1x1),
        );

        assert!(p_uw > p_ws);
        assert!(p_ws > p_sq);

        let h_sq = calculate_mountain_height_aspect(
            15.0,
            width,
            20,
            MountainStyle::Peaks,
            0.0,
            Some(AspectRatioFormat::Square1x1),
        );
        let h_uw = calculate_mountain_height_aspect(
            15.0,
            width,
            20,
            MountainStyle::Peaks,
            0.0,
            Some(AspectRatioFormat::UltraWide21x9),
        );
        assert!(h_sq > h_uw);

        // Verify multi-format scenery rendering without panics or memory overflow
        let formats = [
            (210, 40), // 21:9 UltraWide
            (120, 34), // 16:9 Widescreen
            (100, 32), // 16:10 Golden
            (80, 30),  // 4:3 Classic
            (80, 42),  // 1:1 Square
            (50, 30),  // Narrow
        ];

        for (cols, rows) in formats {
            let mut fb = FrameBuffer::new(cols, rows);
            let road_y = rows.saturating_sub(4);
            render_scenery(
                &mut fb,
                MountainStyle::Peaks,
                RoadStyle::Cobblestone,
                EnvironmentStyle::Pasture,
                road_y,
                2.5,
            );
            assert!(fb.back.iter().any(|c| c.alpha != 0));
        }
    }

    #[test]
    fn test_arbitrary_width_rendering_exceeding_1024_columns() {
        // User mandate: "1024 is not the limit the screen width of the user and the terminal width of the window is limit"
        // Test rendering across 1200, 2048, 2560, and 3840 column viewports.
        let wide_widths = [1200, 2048, 2560, 3840];
        for width in wide_widths {
            let mut fb = FrameBuffer::new(width, 40);
            let road_y: usize = 35;
            render_mountain(
                &mut fb,
                MountainStyle::Peaks,
                road_y.saturating_sub(10),
                width,
                1.0,
            );
            render_road(&mut fb, RoadStyle::Cobblestone, road_y, width, 1.0);

            // Verify that columns far beyond 1024 (e.g. at col 1050, 2000, 3800) have non-empty rendered cells
            let far_col = width - 10;
            let cell = fb.get_back(far_col, road_y);
            assert!(
                cell.ch != ' ',
                "Road should render across all columns up to width {} (checked col {})",
                width,
                far_col
            );
        }
    }

    #[test]
    fn test_adaptive_groves_and_clearings_natural_flow() {
        // 1. Verify monotonicity across 100 consecutive trees
        let mut prev_pos = 0;
        for t in 0..100 {
            let pos = calculate_tree_position(t, 85.0, 50.0);
            if t > 0 {
                assert!(
                    pos > prev_pos,
                    "Tree positions must be strictly monotonically increasing (tree {t}: {pos} vs {prev_pos})"
                );
            }
            prev_pos = pos;
        }

        // 2. Verify groves and open clearings in Pasture
        // In pasture, intra-grove distances are <= 32 cols, while clearings are >= 70 cols
        let mut intra_distances = Vec::new();
        let mut clearing_distances = Vec::new();
        for t in 0..20 {
            let p0 = calculate_tree_position(t, 85.0, 50.0);
            let p1 = calculate_tree_position(t + 1, 85.0, 50.0);
            let delta = p1 - p0;
            if delta <= 32 {
                intra_distances.push(delta);
            } else {
                clearing_distances.push(delta);
            }
        }
        // Must contain both clustered intra-grove steps and spacious open clearings
        assert!(
            !clearing_distances.is_empty(),
            "Must feature spacious open clearings"
        );
        for &clear in &clearing_distances {
            assert!(
                clear >= 70,
                "Clearing distance must be at least 70 cols, got {clear}"
            );
        }
    }

    #[test]
    fn test_sliding_window_max_three_species_quota() {
        // Test that across arbitrary screen widths and tree sequences,
        // no single flora species appears more than 3 times in any window.
        let biomes = [
            EnvironmentStyle::Pasture,
            EnvironmentStyle::Forest,
            EnvironmentStyle::Arctic,
            EnvironmentStyle::Savanna,
            EnvironmentStyle::City,
        ];

        for &env in &biomes {
            // Check sliding windows of size 10 consecutive trees across 50 trees
            for start in 0..40 {
                let mut species_counts = std::collections::HashMap::new();
                for t in start..start + 10 {
                    let sp = calculate_tree_species_with_width(t, env, 42, 160);
                    *species_counts.entry(sp).or_insert(0) += 1;
                }
                for (sp, count) in species_counts {
                    assert!(
                        count <= 3,
                        "Biome {:?}: Species {:?} appeared {} times in window {start}..{} (max allowed is 3)",
                        env,
                        sp,
                        count,
                        start + 10
                    );
                }
            }
        }
    }

    #[test]
    fn test_active_screen_render_tree_species_quota() {
        // Test that render_trees_with_animal strictly enforces <= 3 instances of any species
        // across the visible screen width without panicking.
        for width in [80, 120, 160, 200, 250] {
            let mut fb = FrameBuffer::new(width, 24);
            render_trees_with_animal(
                &mut fb,
                MountainStyle::Hills,
                EnvironmentStyle::Pasture,
                20,
                width,
                0.0,
                Some("cow"),
                Some(8),
                Some(BaseAnim::Walk),
            );
            assert!(fb.back.iter().any(|c| c.alpha != 0 || c.ch != ' '));
        }
    }

    #[test]
    fn test_tree_sway_kinematics() {
        // Ground base row (dy_from_base == 0) must strictly have 0 sway offset under all wind conditions
        for time in [0.0, 0.5, 1.25, 2.7, 5.0, 10.0, 42.0] {
            for height in [3, 5, 8, 12, 16] {
                for tree_idx in 0..10 {
                    let offset = calculate_tree_sway_offset(
                        TreeSpecies::Oak,
                        height,
                        0,
                        height,
                        time,
                        tree_idx,
                    );
                    assert_eq!(
                        offset, 0,
                        "Tree trunk root base at dy_from_base=0 must never detach or sway (got {offset} at t={time}, h={height})"
                    );
                }
            }
        }

        // Canopy apex (dy_from_base == height) must stay within physically stable bounds (-2..=2)
        for time in [0.0, 0.8, 1.6, 2.4, 3.2, 4.0] {
            let offset_tall = calculate_tree_sway_offset(TreeSpecies::Redwood, 14, 12, 14, time, 1);
            assert!(
                (-2..=2).contains(&offset_tall),
                "Canopy sway offset must be bounded in [-2, 2], got {offset_tall}"
            );
        }
    }

    #[test]
    fn test_procedural_ground_cover_distribution_and_verge() {
        let width: usize = 120;
        let road_y: usize = 18;
        let mut fb = FrameBuffer::new(width, 24);

        // Pre-populate an obstacle on the verge (e.g. tree trunk or mascot foot)
        let obstacle_x: usize = 45;
        let verge_y: usize = road_y.saturating_sub(1);
        let _ = fb.set(obstacle_x, verge_y, Cell::new('|', Color::rgb(100, 50, 20)));

        render_procedural_ground_cover(
            &mut fb,
            EnvironmentStyle::Pasture,
            road_y,
            width,
            0.0,
            TimeOfDay::Day,
        );

        // Verify obstacle was preserved and not overwritten
        let obstacle_cell = fb.get_back(obstacle_x, verge_y);
        assert_eq!(obstacle_cell.ch, '|');

        // Verify ground cover items were placed on the meadow verge
        let mut ground_items = 0;
        for x in 0..width {
            let c = fb.get_back(x, verge_y);
            if matches!(c.ch, ',' | '\'' | '%' | '.' | '*') {
                ground_items += 1;
            }
        }
        assert!(
            ground_items >= 2,
            "Pasture meadow verge must feature sparse procedural ground cover items, found {ground_items}"
        );
    }

    #[test]
    fn test_time_of_day_illumination_and_parsing() {
        // Parsing and string conversion
        assert_eq!(TimeOfDay::parse("dawn"), TimeOfDay::Dawn);
        assert_eq!(TimeOfDay::parse("sunrise"), TimeOfDay::Dawn);
        assert_eq!(TimeOfDay::parse("day"), TimeOfDay::Day);
        assert_eq!(TimeOfDay::parse("noon"), TimeOfDay::Day);
        assert_eq!(TimeOfDay::parse("dusk"), TimeOfDay::Dusk);
        assert_eq!(TimeOfDay::parse("sunset"), TimeOfDay::Dusk);
        assert_eq!(TimeOfDay::parse("night"), TimeOfDay::Night);
        assert_eq!(TimeOfDay::parse("midnight"), TimeOfDay::Night);
        assert_eq!(TimeOfDay::parse("auto"), TimeOfDay::Auto);

        assert_eq!(TimeOfDay::Dawn.as_str(), "dawn");
        assert_eq!(TimeOfDay::Day.as_str(), "day");
        assert_eq!(TimeOfDay::Dusk.as_str(), "dusk");
        assert_eq!(TimeOfDay::Night.as_str(), "night");
        assert_eq!(TimeOfDay::Auto.as_str(), "auto");

        // Color lighting filters
        let base_green = Color::rgb(60, 160, 60);

        // Dawn warms red channel
        let dawn_col = TimeOfDay::Dawn.apply_lighting(base_green);
        assert!(dawn_col.r > base_green.r);

        // Night cools and darkens foliage
        let night_col = TimeOfDay::Night.apply_lighting(base_green);
        assert!(night_col.g < base_green.g);
        assert!(night_col.b >= (night_col.g / 2));

        // Render scenery with each time of day
        for tod in [
            TimeOfDay::Dawn,
            TimeOfDay::Day,
            TimeOfDay::Dusk,
            TimeOfDay::Night,
        ] {
            let mut fb = FrameBuffer::new(100, 24);
            render_scenery_full_with_tod(
                &mut fb,
                MountainStyle::Hills,
                RoadStyle::Dirt,
                EnvironmentStyle::Pasture,
                18,
                1.0,
                Some("cow"),
                Some(6),
                Some(BaseAnim::Walk),
                Some(tod),
            );
            assert!(fb.back.iter().any(|c| c.alpha != 0 || c.ch != ' '));
        }
    }

    #[test]
    fn test_consecutive_trees_never_repeat_species() {
        let biomes = [
            EnvironmentStyle::Pasture,
            EnvironmentStyle::Forest,
            EnvironmentStyle::Arctic,
            EnvironmentStyle::Savanna,
            EnvironmentStyle::City,
            EnvironmentStyle::Jurassic,
            EnvironmentStyle::Swamp,
            EnvironmentStyle::Inferno,
            EnvironmentStyle::Hive,
            EnvironmentStyle::Throne,
            EnvironmentStyle::Cyber,
        ];

        for &env in &biomes {
            let pool = get_biome_species_pool(env);
            if pool.len() <= 1 {
                continue;
            }
            for seed in [0u32, 42, 1337, 99999] {
                let mut prev_species = calculate_tree_species_with_width(0, env, seed, 160);
                for n in 1..80 {
                    let cur_species = calculate_tree_species_with_width(n, env, seed, 160);
                    assert_ne!(
                        cur_species,
                        prev_species,
                        "Biome {:?}: Tree {} and {} must never share the same species ({:?})",
                        env,
                        n - 1,
                        n,
                        cur_species
                    );
                    prev_species = cur_species;
                }
            }
        }
    }

    #[test]
    fn test_dynamic_clearing_expansion_ultra_wide() {
        let (min_base, _var_base) = get_effective_tree_distances(EnvironmentStyle::Arctic, 80);
        let (min_wide, _var_wide) = get_effective_tree_distances(EnvironmentStyle::Arctic, 1500);
        // Arctic has 5 species -> max 15 trees on screen. At width 1500, min_stride must expand
        assert!(
            min_wide > min_base,
            "Effective min_dist must expand on wide viewports (got {} vs base {})",
            min_wide,
            min_base
        );
        assert!(
            min_wide >= 100.0,
            "Expanded clearing distance must be >= 100 on 1500-column viewport"
        );
    }

    #[test]
    fn test_seasonal_phenology_and_rendering() {
        assert_eq!(Season::parse("spring"), Season::Spring);
        assert_eq!(Season::parse("summer"), Season::Summer);
        assert_eq!(Season::parse("autumn"), Season::Autumn);
        assert_eq!(Season::parse("winter"), Season::Winter);
        assert_eq!(Season::parse("auto"), Season::Auto);

        // Maple leaves turn scarlet in Autumn
        let base_leaf = Color::rgb(50, 150, 50);
        let (_ch_aut, col_aut) =
            Season::Autumn.apply_foliage_phenology(TreeSpecies::Maple, base_leaf, '@', 0);
        assert!(
            col_aut.r > 200,
            "Maple in Autumn must shift to vibrant scarlet, red channel was {}",
            col_aut.r
        );

        // Cherry blossoms bloom in Spring
        let (ch_spr, _col_spr) =
            Season::Spring.apply_foliage_phenology(TreeSpecies::CherryBlossom, base_leaf, '@', 0);
        assert_eq!(
            ch_spr, '*',
            "Cherry blossom in Spring must bloom with petal glyph '*'"
        );

        // Render full scenery with Season override
        for season in [
            Season::Spring,
            Season::Summer,
            Season::Autumn,
            Season::Winter,
        ] {
            let mut fb = FrameBuffer::new(120, 24);
            render_scenery_full_with_tod_and_season(
                &mut fb,
                MountainStyle::Peaks,
                RoadStyle::Cobblestone,
                EnvironmentStyle::Forest,
                18,
                2.0,
                Some("cow"),
                Some(7),
                Some(BaseAnim::Walk),
                Some(TimeOfDay::Day),
                Some(season),
            );
            assert!(fb.back.iter().any(|c| c.alpha != 0 || c.ch != ' '));
        }
    }

    #[test]
    fn test_celestial_arc_bounds() {
        for time in [0.0, 1.0, 5.0, 12.5, 25.0, 48.0] {
            let mut fb = FrameBuffer::new(100, 25);
            render_environment_with_tod(
                &mut fb,
                EnvironmentStyle::Pasture,
                18,
                100,
                25,
                time,
                TimeOfDay::Day,
            );
            // Ensure sun is rendered in upper sky region (y in 1..=6)
            let has_sun = (1..=6).any(|y| (0..100).any(|x| fb.get_back(x, y).ch == '*'));
            assert!(has_sun, "Sun must be visible in the sky at time {time}");
        }
    }
}
