//! Per-animal thematic scenery engine.
//!
//! Generates layered procedural ASCII/ANSI backdrops (Mountain, Road, Environment)
//! tailored to each of the 106 creatures or customized via CLI flags.

use crate::dna::BaseAnim;
use crate::framebuffer::{Cell, Color, FrameBuffer};

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
    pub fn parse(s: &str) -> Self {
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
    pub fn parse(s: &str) -> Self {
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
    pub fn parse(s: &str) -> Self {
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

    match clean.as_str() {
        // ── Volcanic, Mythic & Dragons ──────────────────────────────────────
        "dragon" => AnimalProfile {
            name: "dragon",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#d84315", "#f4511e", "#ffb300"],
            eyes: "oo",
            tongue: "  ",
        },
        "dragon-and-cow" => AnimalProfile {
            name: "dragon-and-cow",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#c62828", "#e53935", "#ff8f00"],
            eyes: "oo",
            tongue: "  ",
        },
        "charizardvice" => AnimalProfile {
            name: "charizardvice",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#ff6f00", "#ffab00", "#00b0ff"],
            eyes: "oo",
            tongue: "  ",
        },
        "daemon" => AnimalProfile {
            name: "daemon",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#d32f2f", "#f44336", "#212121"],
            eyes: "XX",
            tongue: "  ",
        },
        "satanic" => AnimalProfile {
            name: "satanic",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#b71c1c", "#d32f2f", "#212121"],
            eyes: "XX",
            tongue: "  ",
        },
        "minotaur" => AnimalProfile {
            name: "minotaur",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#4e342e", "#6d4c41", "#d32f2f"],
            eyes: "XX",
            tongue: "  ",
        },
        "mooghidjirah" => AnimalProfile {
            name: "mooghidjirah",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#c62828", "#ad1457", "#ff6f00"],
            eyes: "XX",
            tongue: "  ",
        },
        "moojira" => AnimalProfile {
            name: "moojira",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#2e7d32", "#1b5e20", "#ff1744"],
            eyes: "XX",
            tongue: "  ",
        },
        "flaming-sheep" => AnimalProfile {
            name: "flaming-sheep",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#ff5722", "#ff8a65", "#ffab00"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Ocean & Marine Depths ──────────────────────────────────────────
        "whale" => AnimalProfile {
            name: "whale",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#0288d1", "#29b6f6", "#e1f5fe"],
            eyes: "oo",
            tongue: "  ",
        },
        "happy-whale" => AnimalProfile {
            name: "happy-whale",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#039be5", "#4fc3f7", "#e1f5fe"],
            eyes: "^^",
            tongue: "  ",
        },
        "docker-whale" => AnimalProfile {
            name: "docker-whale",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#0288d1", "#29b6f6", "#ffffff"],
            eyes: "oo",
            tongue: "  ",
        },
        "dolphin" => AnimalProfile {
            name: "dolphin",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#4fc3f7", "#81d4fa", "#e1f5fe"],
            eyes: "oo",
            tongue: "  ",
        },
        "jellyfish" => AnimalProfile {
            name: "jellyfish",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#e040fb", "#ea80fc", "#80d8ff"],
            eyes: "oo",
            tongue: "  ",
        },
        "octopus" => AnimalProfile {
            name: "octopus",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#8e24aa", "#ab47bc", "#ce93d8"],
            eyes: "oo",
            tongue: "  ",
        },
        "smiling-octopus" => AnimalProfile {
            name: "smiling-octopus",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#ab47bc", "#ce93d8", "#f3e5f5"],
            eyes: "^^",
            tongue: "  ",
        },
        "lobster" => AnimalProfile {
            name: "lobster",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#d32f2f", "#f44336", "#ff8a80"],
            eyes: "oo",
            tongue: "  ",
        },
        "seahorse" => AnimalProfile {
            name: "seahorse",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#ffa000", "#ffb300", "#ffd54f"],
            eyes: "oo",
            tongue: "  ",
        },
        "seahorse-big" => AnimalProfile {
            name: "seahorse-big",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#ffb300", "#ffa000", "#ffe082"],
            eyes: "oo",
            tongue: "  ",
        },
        "turtle" => AnimalProfile {
            name: "turtle",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#2e7d32", "#4caf50", "#81c784"],
            eyes: "..",
            tongue: "  ",
        },
        "ebi_furai" => AnimalProfile {
            name: "ebi_furai",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#ff8f00", "#ffb300", "#fff8e1"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Arctic & Polar ─────────────────────────────────────────────────
        "tux" => AnimalProfile {
            name: "tux",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Arctic,
            road: RoadStyle::Ice,
            mountain: MountainStyle::Iceberg,
            wildlife_palette: &["#ffffff", "#212121", "#ff9800"],
            eyes: "oo",
            tongue: "  ",
        },
        "tux-big" => AnimalProfile {
            name: "tux-big",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Arctic,
            road: RoadStyle::Ice,
            mountain: MountainStyle::Iceberg,
            wildlife_palette: &["#ffffff", "#212121", "#ff9800"],
            eyes: "oo",
            tongue: "  ",
        },
        "bearface" => AnimalProfile {
            name: "bearface",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Arctic,
            road: RoadStyle::Ice,
            mountain: MountainStyle::Iceberg,
            wildlife_palette: &["#5d4037", "#8d6e63", "#d7ccc8"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Swamp & Wetlands ───────────────────────────────────────────────
        "bud-frogs" => AnimalProfile {
            name: "bud-frogs",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Swamp,
            road: RoadStyle::Mud,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#4caf50", "#81c784", "#1b5e20"],
            eyes: "@@",
            tongue: "  ",
        },

        // ── Urban, Pets & Domestic ─────────────────────────────────────────
        "cat" => AnimalProfile {
            name: "cat",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ff9800", "#ffffff", "#e65100"],
            eyes: "^^",
            tongue: "  ",
        },
        "cat2" => AnimalProfile {
            name: "cat2",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#9e9e9e", "#e0e0e0", "#424242"],
            eyes: "^^",
            tongue: "  ",
        },
        "catfence" => AnimalProfile {
            name: "catfence",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#616161", "#9e9e9e", "#212121"],
            eyes: "^^",
            tongue: "  ",
        },
        "kitten" => AnimalProfile {
            name: "kitten",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffcc80", "#ffe0b2", "#ffffff"],
            eyes: "^^",
            tongue: "  ",
        },
        "kitty" => AnimalProfile {
            name: "kitty",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffb74d", "#ffe082", "#ffffff"],
            eyes: "^^",
            tongue: "  ",
        },
        "meow" => AnimalProfile {
            name: "meow",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffa726", "#ffcc80", "#ffffff"],
            eyes: "^^",
            tongue: "U ",
        },
        "doge" => AnimalProfile {
            name: "doge",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffb74d", "#ffe082", "#ffffff"],
            eyes: "oo",
            tongue: "  ",
        },
        "bill-the-cat" => AnimalProfile {
            name: "bill-the-cat",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffb74d", "#e57373", "#fff176"],
            eyes: "oO",
            tongue: "U ",
        },
        "hellokitty" => AnimalProfile {
            name: "hellokitty",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffffff", "#ff4081", "#ffeb3b"],
            eyes: "oo",
            tongue: "  ",
        },
        "snoopy" => AnimalProfile {
            name: "snoopy",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffffff", "#212121", "#d32f2f"],
            eyes: "oo",
            tongue: "  ",
        },
        "snoopyhouse" => AnimalProfile {
            name: "snoopyhouse",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#e53935", "#ffffff", "#212121"],
            eyes: "oo",
            tongue: "  ",
        },
        "snoopysleep" => AnimalProfile {
            name: "snoopysleep",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Roof,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ffffff", "#e53935", "#212121"],
            eyes: "--",
            tongue: "  ",
        },
        "ren" => AnimalProfile {
            name: "ren",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#d7ccc8", "#a1887f", "#ff80ab"],
            eyes: "oo",
            tongue: "U ",
        },
        "stimpy" => AnimalProfile {
            name: "stimpy",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#e53935", "#ef5350", "#3949ab"],
            eyes: "oO",
            tongue: "U ",
        },
        "kiss" => AnimalProfile {
            name: "kiss",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#ff4081", "#ff80ab", "#ffffff"],
            eyes: "oo",
            tongue: "U ",
        },
        "cowfee" => AnimalProfile {
            name: "cowfee",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#6d4c41", "#8d6e63", "#d7ccc8"],
            eyes: "oo",
            tongue: "  ",
        },
        "surgery" => AnimalProfile {
            name: "surgery",
            base_anim: BaseAnim::Glitch,
            environment: EnvironmentStyle::City,
            road: RoadStyle::Sidewalk,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#00bcd4", "#80deea", "#ffffff"],
            eyes: "XX",
            tongue: "  ",
        },

        // ── Forest & Woodlands ─────────────────────────────────────────────
        "fox" => AnimalProfile {
            name: "fox",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#e65100", "#ff9800", "#ffffff"],
            eyes: "^^",
            tongue: "  ",
        },
        "koala" => AnimalProfile {
            name: "koala",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#9e9e9e", "#bdbdbd", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
        "luke-koala" => AnimalProfile {
            name: "luke-koala",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#90a4ae", "#b0bec5", "#cfd8dc"],
            eyes: "oo",
            tongue: "  ",
        },
        "squirrel" => AnimalProfile {
            name: "squirrel",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#8d6e63", "#a1887f", "#d7ccc8"],
            eyes: "..",
            tongue: "  ",
        },
        "hedgehog" => AnimalProfile {
            name: "hedgehog",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#6d4c41", "#8d6e63", "#d7ccc8"],
            eyes: "..",
            tongue: "  ",
        },
        "owl" => AnimalProfile {
            name: "owl",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#795548", "#8d6e63", "#ffb300"],
            eyes: "OO",
            tongue: "  ",
        },
        "moose" => AnimalProfile {
            name: "moose",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#4e342e", "#5d4037", "#795548"],
            eyes: "oo",
            tongue: "  ",
        },
        "goat" => AnimalProfile {
            name: "goat",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#d7ccc8", "#a1887f", "#8d6e63"],
            eyes: "oo",
            tongue: "  ",
        },
        "goat2" => AnimalProfile {
            name: "goat2",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#bcaaa4", "#8d6e63", "#5d4037"],
            eyes: "oo",
            tongue: "  ",
        },
        "golden-eagle" => AnimalProfile {
            name: "golden-eagle",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#ffb300", "#ffa000", "#5d4037"],
            eyes: "oo",
            tongue: "  ",
        },
        "tweety-bird" => AnimalProfile {
            name: "tweety-bird",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#ffeb3b", "#fff59d", "#ff9800"],
            eyes: "oo",
            tongue: "  ",
        },
        "fat-banana" => AnimalProfile {
            name: "fat-banana",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffeb3b", "#fff59d", "#fbc02d"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Savanna & Safari ───────────────────────────────────────────────
        "elephant" => AnimalProfile {
            name: "elephant",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#78909c", "#90a4ae", "#cfd8dc"],
            eyes: "oo",
            tongue: "  ",
        },
        "elephant2" => AnimalProfile {
            name: "elephant2",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#607d8b", "#78909c", "#b0bec5"],
            eyes: "oo",
            tongue: "  ",
        },
        "elephant-in-snake" => AnimalProfile {
            name: "elephant-in-snake",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#78909c", "#b0bec5", "#546e7a"],
            eyes: "oo",
            tongue: "  ",
        },
        "moofasa" => AnimalProfile {
            name: "moofasa",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#ff8f00", "#ffa000", "#ffc107"],
            eyes: "oo",
            tongue: "  ",
        },
        "armadillo" => AnimalProfile {
            name: "armadillo",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#8d6e63", "#d7ccc8"],
            eyes: "oo",
            tongue: "  ",
        },
        "tortoise" => AnimalProfile {
            name: "tortoise",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#558b2f", "#689f38", "#9e9d24"],
            eyes: "..",
            tongue: "  ",
        },
        "mule" => AnimalProfile {
            name: "mule",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#6d4c41", "#8d6e63", "#a1887f"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Space & Cyber ──────────────────────────────────────────────────
        "nyan" | "nyan-cat" | "nyancat" | "nyan_cat" => AnimalProfile {
            name: "nyan",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#ff0033", "#ff7f00", "#ffff00", "#33ff00", "#0099ff", "#9933ff"],
            eyes: "^^",
            tongue: "  ",
        },
        "atat" => AnimalProfile {
            name: "atat",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#9e9e9e", "#cfd8dc", "#37474f"],
            eyes: "==",
            tongue: "  ",
        },
        "glados" => AnimalProfile {
            name: "glados",
            base_anim: BaseAnim::Glitch,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#eceff1", "#ffab00", "#212121"],
            eyes: "[]",
            tongue: "  ",
        },
        "personality-sphere" => AnimalProfile {
            name: "personality-sphere",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#03a9f4", "#e0e0e0", "#212121"],
            eyes: "OO",
            tongue: "  ",
        },
        "world" => AnimalProfile {
            name: "world",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#29b6f6", "#66bb6a", "#ffffff"],
            eyes: "oo",
            tongue: "  ",
        },
        "beavis.zen" | "beavis" => AnimalProfile {
            name: "beavis.zen",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#ffca28", "#42a5f5", "#ef5350"],
            eyes: "oo",
            tongue: "  ",
        },
        "shikato" => AnimalProfile {
            name: "shikato",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#9e9e9e", "#e0e0e0", "#212121"],
            eyes: "==",
            tongue: "  ",
        },
        "hypno" => AnimalProfile {
            name: "hypno",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#7c4dff", "#00e5ff", "#e040fb"],
            eyes: "@@",
            tongue: "  ",
        },
        "kosh" => AnimalProfile {
            name: "kosh",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#80cbc4", "#4db6ac", "#00897b"],
            eyes: "oo",
            tongue: "  ",
        },
        "lollerskates" => AnimalProfile {
            name: "lollerskates",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#00e676", "#00b0ff", "#ffea00"],
            eyes: "oo",
            tongue: "  ",
        },
        "claw-arm" => AnimalProfile {
            name: "claw-arm",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#00e5ff", "#76ff03", "#2979ff"],
            eyes: "[]",
            tongue: "  ",
        },
        "periodic-table" => AnimalProfile {
            name: "periodic-table",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#00e5ff", "#76ff03", "#ffd600"],
            eyes: "[]",
            tongue: "  ",
        },

        // ── Crypt & Gothic ─────────────────────────────────────────────────
        "ghost" => AnimalProfile {
            name: "ghost",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#eceff1", "#cfd8dc", "#b0bec5"],
            eyes: "oo",
            tongue: "  ",
        },
        "ghostbusters" => AnimalProfile {
            name: "ghostbusters",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#e53935", "#ffffff", "#212121"],
            eyes: "oo",
            tongue: "  ",
        },
        "skeleton" => AnimalProfile {
            name: "skeleton",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#f5f5f5", "#e0e0e0", "#9e9e9e"],
            eyes: "OO",
            tongue: "  ",
        },
        "weeping-angel" => AnimalProfile {
            name: "weeping-angel",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#9e9e9e", "#757575", "#bdbdbd"],
            eyes: "XX",
            tongue: "  ",
        },
        "cthulhu-mini" => AnimalProfile {
            name: "cthulhu-mini",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#2e7d32", "#1b5e20", "#81c784"],
            eyes: "oo",
            tongue: "  ",
        },
        "mutilated" => AnimalProfile {
            name: "mutilated",
            base_anim: BaseAnim::Dissolve,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#d32f2f", "#b71c1c", "#212121"],
            eyes: "XX",
            tongue: "  ",
        },
        "eyes" => AnimalProfile {
            name: "eyes",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Graveyard,
            road: RoadStyle::Crypt,
            mountain: MountainStyle::Gothic,
            wildlife_palette: &["#00e676", "#b9f6ca", "#ffffff"],
            eyes: "OO",
            tongue: "  ",
        },

        // ── Prehistoric & Dinosaurs ────────────────────────────────────────
        "stegosaurus" => AnimalProfile {
            name: "stegosaurus",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Jurassic,
            road: RoadStyle::Tracks,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#388e3c", "#4caf50", "#81c784"],
            eyes: "oo",
            tongue: "  ",
        },
        "pterodactyl" => AnimalProfile {
            name: "pterodactyl",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Jurassic,
            road: RoadStyle::Tracks,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#689f38", "#8bc34a", "#cddc39"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Royal & Chess ──────────────────────────────────────────────────
        "king" => AnimalProfile {
            name: "king",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#ffd700", "#c0c0c0", "#e53935"],
            eyes: "oo",
            tongue: "  ",
        },
        "queen" => AnimalProfile {
            name: "queen",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#ffd700", "#e91e63", "#ffffff"],
            eyes: "oo",
            tongue: "  ",
        },
        "knight" => AnimalProfile {
            name: "knight",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#b0bec5", "#cfd8dc", "#78909c"],
            eyes: "oo",
            tongue: "  ",
        },
        "pawn" => AnimalProfile {
            name: "pawn",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#cfd8dc", "#eceff1", "#90a4ae"],
            eyes: "oo",
            tongue: "  ",
        },
        "rook" => AnimalProfile {
            name: "rook",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#78909c", "#90a4ae", "#b0bec5"],
            eyes: "[]",
            tongue: "  ",
        },
        "wizard" => AnimalProfile {
            name: "wizard",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#7c4dff", "#b388ff", "#ffd700"],
            eyes: "oo",
            tongue: "  ",
        },
        "charlie" => AnimalProfile {
            name: "charlie",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#ffd54f", "#ffe082", "#bcaaa4"],
            eyes: "oo",
            tongue: "U ",
        },
        "mona-lisa" => AnimalProfile {
            name: "mona-lisa",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Checkerboard,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#8d6e63", "#a1887f", "#d7ccc8"],
            eyes: "oo",
            tongue: "  ",
        },

        // ── Hive & Insects ─────────────────────────────────────────────────
        "bees" => AnimalProfile {
            name: "bees",
            base_anim: BaseAnim::Fly,
            environment: EnvironmentStyle::Hive,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Garden,
            wildlife_palette: &["#ffeb3b", "#212121", "#fff59d"],
            eyes: "oo",
            tongue: "  ",
        },
        "spidercow" => AnimalProfile {
            name: "spidercow",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Hive,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Garden,
            wildlife_palette: &["#212121", "#d32f2f", "#616161"],
            eyes: "88",
            tongue: "  ",
        },

        // ── Pastoral, Bovines & Countryside ────────────────────────────────
        "bunny" => AnimalProfile {
            name: "bunny",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#f5f5f5", "#ff80ab", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
        "cower" => AnimalProfile {
            name: "cower",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#212121", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
        "fat-cow" => AnimalProfile {
            name: "fat-cow",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#424242", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
        "fence" => AnimalProfile {
            name: "fence",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#8d6e63", "#a1887f", "#d7ccc8"],
            eyes: "||",
            tongue: "  ",
        },
        "hippie" => AnimalProfile {
            name: "hippie",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#e040fb", "#7c4dff", "#00e676"],
            eyes: "oo",
            tongue: "  ",
        },
        "hiya" => AnimalProfile {
            name: "hiya",
            base_anim: BaseAnim::Talk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#212121", "#ff80ab"],
            eyes: "^^",
            tongue: "U ",
        },
        "lamb" => AnimalProfile {
            name: "lamb",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#fafafa", "#f5f5f5", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
        "lamb2" => AnimalProfile {
            name: "lamb2",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#f5f5f5", "#eeeeee", "#bdbdbd"],
            eyes: "oo",
            tongue: "  ",
        },
        "sheep" => AnimalProfile {
            name: "sheep",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#f5f5f5", "#424242"],
            eyes: "oo",
            tongue: "  ",
        },
        "shrug" => AnimalProfile {
            name: "shrug",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#e0e0e0", "#bdbdbd"],
            eyes: "oo",
            tongue: "  ",
        },
        "small" => AnimalProfile {
            name: "small",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#e0e0e0", "#212121"],
            eyes: "oo",
            tongue: "  ",
        },
        "supermilker" => AnimalProfile {
            name: "supermilker",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#00e676", "#00b0ff", "#ffffff"],
            eyes: "oo",
            tongue: "  ",
        },
        "turkey" => AnimalProfile {
            name: "turkey",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#6d4c41", "#c62828", "#f57f17"],
            eyes: "oo",
            tongue: "  ",
        },
        "corgi" => AnimalProfile {
            name: "corgi",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Cobblestone,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#d97706", "#fef3c7", "#b45309"],
            eyes: "oo",
            tongue: "U ",
        },
        "duck" => AnimalProfile {
            name: "duck",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Swamp,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#059669", "#fbbf24", "#d97706"],
            eyes: "oo",
            tongue: "  ",
        },
        "pig" => AnimalProfile {
            name: "pig",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Mud,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#f472b6", "#fbcfe8", "#db2777"],
            eyes: "oo",
            tongue: "  ",
        },
        "tiger" => AnimalProfile {
            name: "tiger",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#ea580c", "#18181b", "#fb923c"],
            eyes: "oo",
            tongue: "  ",
        },
        "wolf" => AnimalProfile {
            name: "wolf",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#9ca3af", "#4b5563", "#1f2937"],
            eyes: "oo",
            tongue: "  ",
        },
        "squid" => AnimalProfile {
            name: "squid",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#06b6d4", "#3b82f6", "#0284c7"],
            eyes: "oo",
            tongue: "  ",
        },
        "walrus" => AnimalProfile {
            name: "walrus",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Arctic,
            road: RoadStyle::Ice,
            mountain: MountainStyle::Iceberg,
            wildlife_palette: &["#78716c", "#d6d3d1", "#57534e"],
            eyes: "oo",
            tongue: "  ",
        },
        "vader" => AnimalProfile {
            name: "vader",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#dc2626", "#18181b", "#7f1d1d"],
            eyes: "oo",
            tongue: "  ",
        },
        "yoda" => AnimalProfile {
            name: "yoda",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Swamp,
            road: RoadStyle::Mud,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#84cc16", "#65a30d", "#a3e635"],
            eyes: "oo",
            tongue: "  ",
        },
        "panther" => AnimalProfile {
            name: "panther",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#27272a", "#3f3f46", "#18181b"],
            eyes: "oo",
            tongue: "  ",
        },
        "rhino" => AnimalProfile {
            name: "rhino",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Savanna,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#64748b", "#94a3b8", "#475569"],
            eyes: "oo",
            tongue: "  ",
        },
        "sloth" => AnimalProfile {
            name: "sloth",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Forest,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#a16207", "#ca8a04", "#713f12"],
            eyes: "oo",
            tongue: "  ",
        },
        "pufferfish" => AnimalProfile {
            name: "pufferfish",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Ocean,
            road: RoadStyle::Seabed,
            mountain: MountainStyle::Seamount,
            wildlife_palette: &["#facc15", "#fde047", "#eab308"],
            eyes: "oo",
            tongue: "  ",
        },
        "sauron" => AnimalProfile {
            name: "sauron",
            base_anim: BaseAnim::Pulse,
            environment: EnvironmentStyle::Inferno,
            road: RoadStyle::Magma,
            mountain: MountainStyle::Volcano,
            wildlife_palette: &["#f97316", "#dc2626", "#b91c1c"],
            eyes: "()",
            tongue: "  ",
        },
        "unipony" => AnimalProfile {
            name: "unipony",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Cobblestone,
            mountain: MountainStyle::Castle,
            wildlife_palette: &["#e879f9", "#38bdf8", "#c084fc"],
            eyes: "oo",
            tongue: "  ",
        },
        "radioactive-kitty" => AnimalProfile {
            name: "radioactive-kitty",
            base_anim: BaseAnim::Glitch,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#22c55e", "#86efac", "#16a34a"],
            eyes: "^^",
            tongue: "  ",
        },
        "vulpix" => AnimalProfile {
            name: "vulpix",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Savanna,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#f97316", "#ea580c", "#fb923c"],
            eyes: "oo",
            tongue: "  ",
        },
        "ram" => AnimalProfile {
            name: "ram",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Cobblestone,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#d97706", "#f59e0b", "#b45309"],
            eyes: "oo",
            tongue: "  ",
        },
        "rooster" => AnimalProfile {
            name: "rooster",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ef4444", "#f59e0b", "#dc2626"],
            eyes: "oo",
            tongue: "  ",
        },
        "milk" => AnimalProfile {
            name: "milk",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#f8fafc", "#e2e8f0", "#94a3b8"],
            eyes: "oo",
            tongue: "  ",
        },
        "apt" => AnimalProfile {
            name: "apt",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#e11d48", "#f43f5e", "#be123c"],
            eyes: "oo",
            tongue: "  ",
        },
        "jesus" => AnimalProfile {
            name: "jesus",
            base_anim: BaseAnim::Float,
            environment: EnvironmentStyle::Throne,
            road: RoadStyle::Cobblestone,
            mountain: MountainStyle::Peaks,
            wildlife_palette: &["#fbbf24", "#fef08a", "#f59e0b"],
            eyes: "oo",
            tongue: "  ",
        },
        "three-eyes" => AnimalProfile {
            name: "three-eyes",
            base_anim: BaseAnim::Breathe,
            environment: EnvironmentStyle::Space,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Crater,
            wildlife_palette: &["#10b981", "#34d399", "#059669"],
            eyes: "OOO",
            tongue: "  ",
        },
        "viper" => AnimalProfile {
            name: "viper",
            base_anim: BaseAnim::Sway,
            environment: EnvironmentStyle::Swamp,
            road: RoadStyle::Mud,
            mountain: MountainStyle::Plateau,
            wildlife_palette: &["#84cc16", "#4d7c0f", "#65a30d"],
            eyes: "oo",
            tongue: "U ",
        },
        "telebears" => AnimalProfile {
            name: "telebears",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#3b82f6", "#60a5fa", "#2563eb"],
            eyes: "oo",
            tongue: "  ",
        },
        "mech-and-cow" => AnimalProfile {
            name: "mech-and-cow",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Cyber,
            road: RoadStyle::Grid,
            mountain: MountainStyle::Skyline,
            wildlife_palette: &["#0284c7", "#e0f2fe", "#0369a1"],
            eyes: "oo",
            tongue: "  ",
        },

        // Default / Canonical Bovine
        _ => AnimalProfile {
            name: "default",
            base_anim: BaseAnim::Walk,
            environment: EnvironmentStyle::Pasture,
            road: RoadStyle::Dirt,
            mountain: MountainStyle::Hills,
            wildlife_palette: &["#ffffff", "#212121", "#e0e0e0"],
            eyes: "oo",
            tongue: "  ",
        },
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

// ── Scenery Layer Renderers ─────────────────────────────────────────

/// Render mountain / horizon layer directly behind the creature, anchored to the horizon above the road.
/// Uses a multi-harmonic mathematical algorithm to generate dynamic, non-uniform variable heights
/// across columns with smooth horizontal parallax scrolling.
/// `mountain_base_y` is the row where the base of the mountain meets the ground.
pub fn render_mountain(
    fb: &mut FrameBuffer,
    style: MountainStyle,
    mountain_base_y: usize,
    width: usize,
    time: f32,
) {
    if style == MountainStyle::None || fb.height < 2 || width == 0 {
        return;
    }

    let (fg, base_h, max_var): (Color, f32, f32) = match style {
        MountainStyle::Hills => (Color::rgb(28, 68, 32), 2.2, 1.2),
        MountainStyle::Peaks => (Color::rgb(64, 82, 92), 2.4, 1.5),
        MountainStyle::Volcano => (Color::rgb(55, 50, 50), 2.3, 1.3),
        MountainStyle::Iceberg => (Color::rgb(55, 115, 125), 2.5, 1.4),
        MountainStyle::Skyline => (Color::rgb(42, 54, 62), 2.7, 1.5),
        MountainStyle::Seamount => (Color::rgb(15, 42, 85), 2.0, 1.1),
        MountainStyle::Plateau => (Color::rgb(90, 72, 65), 2.2, 1.0),
        MountainStyle::Crater => (Color::rgb(52, 68, 78), 2.0, 1.1),
        MountainStyle::Gothic => (Color::rgb(50, 32, 95), 2.5, 1.4),
        MountainStyle::Castle => (Color::rgb(75, 58, 52), 2.4, 1.2),
        MountainStyle::Garden => (Color::rgb(45, 95, 52), 2.0, 1.0),
        MountainStyle::None => return,
    };

    // Parallax scroll offset (mountains move smoothly at gentle speed)
    let t_offset = time * 1.2;

    // Mathematical height function for column x
    let calc_h = |x: f32| -> f32 {
        let p = (x + t_offset) * 0.16;
        let h1 = p.sin() * (max_var * 0.65);
        let h2 = (p * 2.23 + 1.2).sin() * (max_var * 0.35);
        let h3 = (p * 0.53 + 2.4).cos() * (max_var * 0.25);
        (base_h + h1 + h2 + h3).clamp(1.0, 4.0)
    };

    for x in 0..width {
        let x_f = x as f32;
        let h_curr = calc_h(x_f);
        let h_prev = calc_h(x_f - 1.0);
        let h_next = calc_h(x_f + 1.0);

        let h_int = h_curr.round() as usize;
        let start_y = mountain_base_y.saturating_sub(h_int);

        for y in start_y..=mountain_base_y {
            if y >= fb.height {
                continue;
            }
            let ch = if y == start_y {
                // Summit / ridge line glyph based on slope
                let slope = h_next - h_prev;
                match style {
                    MountainStyle::Skyline => {
                        if x % 3 == 0 {
                            '|'
                        } else {
                            '_'
                        }
                    }
                    MountainStyle::Castle => {
                        if x % 2 == 0 {
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
                        if x % 4 == 0 {
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
                }
            } else if y == mountain_base_y {
                match style {
                    MountainStyle::Volcano => '~',
                    MountainStyle::Skyline => '_',
                    MountainStyle::Seamount => '~',
                    _ => '.',
                }
            } else {
                // Body texture
                match style {
                    MountainStyle::Skyline => {
                        if (x + y) % 2 == 0 {
                            '#'
                        } else {
                            '|'
                        }
                    }
                    MountainStyle::Iceberg => {
                        if (x + y) % 3 == 0 {
                            '^'
                        } else {
                            '|'
                        }
                    }
                    MountainStyle::Volcano => {
                        if (x + y) % 4 == 0 {
                            '/'
                        } else {
                            '\\'
                        }
                    }
                    MountainStyle::Gothic => {
                        if x % 3 == 0 {
                            '|'
                        } else {
                            '/'
                        }
                    }
                    _ => {
                        if (x + y) % 3 == 0 {
                            '|'
                        } else {
                            '/'
                        }
                    }
                }
            };

            if ch != ' ' {
                let _ = fb.set(x, y, Cell::new(ch, fg));
            }
        }
    }
}

/// Render procedural midground trees and flora layer.
/// Planted on the horizon line directly above the road (`tree_base_y = road_y.saturating_sub(1)`).
/// Scrolls at midground speed `(time * 2.4)` between mountains and the road.
pub fn render_trees(
    fb: &mut FrameBuffer,
    _mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
) {
    if tree_base_y >= fb.height || width == 0 {
        return;
    }

    // Midground parallax speed
    let scroll_x = (time * 2.4) as usize;
    let period = 16; // spacing between trees

    let tree_green = Color::rgb(76, 175, 80);
    let trunk_brown = Color::rgb(121, 85, 72);
    let snow_white = Color::rgb(224, 247, 250);
    let acacia_gold = Color::rgb(139, 195, 74);
    let dark_wood = Color::rgb(97, 97, 97);

    // Trees across the visible span
    let start_tree = scroll_x / period;
    let end_tree = (scroll_x + width + period) / period;

    for t_idx in start_tree..=end_tree {
        let world_x = t_idx * period + (t_idx * 7 % 5);
        if world_x < scroll_x {
            continue;
        }
        let screen_x = world_x - scroll_x;
        if screen_x >= width {
            continue;
        }

        // Preserve clear mascot silhouette corridor (columns 10..34)
        // so trees never spawn on or blend directly into the mascot
        if (10..=34).contains(&screen_x) {
            continue;
        }

        let is_pine = t_idx % 2 == 0;

        match env {
            EnvironmentStyle::Arctic => {
                // Frost-covered snow fir
                if tree_base_y >= 2 && screen_x + 2 < width {
                    let _ = fb.set(screen_x + 1, tree_base_y - 2, Cell::new('^', snow_white));
                    let _ = fb.set(screen_x, tree_base_y - 1, Cell::new('/', snow_white));
                    let _ = fb.set(screen_x + 1, tree_base_y - 1, Cell::new('*', snow_white));
                    let _ = fb.set(screen_x + 2, tree_base_y - 1, Cell::new('\\', snow_white));
                    let _ = fb.set(screen_x + 1, tree_base_y, Cell::new('|', dark_wood));
                }
            }
            EnvironmentStyle::Savanna => {
                // Flat-topped Acacia tree
                if tree_base_y >= 1 && screen_x + 4 < width {
                    for (i, c) in "__~---~__".chars().take(5).enumerate() {
                        if screen_x + i < width {
                            let _ =
                                fb.set(screen_x + i, tree_base_y - 1, Cell::new(c, acacia_gold));
                        }
                    }
                    let _ = fb.set(screen_x + 2, tree_base_y, Cell::new('|', trunk_brown));
                }
            }
            EnvironmentStyle::Graveyard => {
                // Barren gnarled spooky dead tree
                if tree_base_y >= 2 && screen_x + 2 < width {
                    let _ = fb.set(screen_x, tree_base_y - 2, Cell::new('\\', dark_wood));
                    let _ = fb.set(screen_x + 1, tree_base_y - 2, Cell::new('|', dark_wood));
                    let _ = fb.set(screen_x + 2, tree_base_y - 2, Cell::new('/', dark_wood));
                    let _ = fb.set(screen_x + 1, tree_base_y - 1, Cell::new('|', dark_wood));
                    let _ = fb.set(screen_x + 1, tree_base_y, Cell::new('|', dark_wood));
                }
            }
            EnvironmentStyle::City | EnvironmentStyle::Space | EnvironmentStyle::Cyber => {
                // No organic trees in futuristic/cyber backdrops
            }
            _ => {
                // Pastoral, Forest, Jungle, Hive, Default: Oak or Pine
                if is_pine {
                    // Evergreen Pine
                    if tree_base_y >= 2 && screen_x + 2 < width {
                        let _ = fb.set(screen_x + 1, tree_base_y - 2, Cell::new('^', tree_green));
                        let _ = fb.set(screen_x, tree_base_y - 1, Cell::new('/', tree_green));
                        let _ = fb.set(screen_x + 1, tree_base_y - 1, Cell::new('|', tree_green));
                        let _ = fb.set(screen_x + 2, tree_base_y - 1, Cell::new('\\', tree_green));
                        let _ = fb.set(screen_x + 1, tree_base_y, Cell::new('|', trunk_brown));
                    }
                } else {
                    // Pastoral Round Oak
                    if tree_base_y >= 2 && screen_x + 2 < width {
                        let _ = fb.set(screen_x, tree_base_y - 2, Cell::new('(', tree_green));
                        let _ = fb.set(screen_x + 1, tree_base_y - 2, Cell::new('\'', tree_green));
                        let _ = fb.set(screen_x + 2, tree_base_y - 2, Cell::new(')', tree_green));
                        let _ = fb.set(screen_x, tree_base_y - 1, Cell::new('(', tree_green));
                        let _ = fb.set(screen_x + 1, tree_base_y - 1, Cell::new('_', tree_green));
                        let _ = fb.set(screen_x + 2, tree_base_y - 1, Cell::new(')', tree_green));
                        let _ = fb.set(screen_x + 1, tree_base_y, Cell::new('|', trunk_brown));
                    }
                }
            }
        }
    }
}

/// Render ground / road baseline directly at cow_bottom_y.
pub fn render_road(
    fb: &mut FrameBuffer,
    style: RoadStyle,
    cow_bottom_y: usize,
    width: usize,
    time: f32,
) {
    if style == RoadStyle::None || cow_bottom_y >= fb.height {
        return;
    }

    let (pattern, fg) = match style {
        RoadStyle::Dirt => (
            "..  ...   .   ...  ..   .   ..   ...   ..",
            Color::rgb(141, 110, 99),
        ),
        RoadStyle::Cobblestone => (
            "[===][===][===][===][===][===][===][===]",
            Color::rgb(158, 158, 158),
        ),
        RoadStyle::Magma => (
            "~~.~~.~~~..~~~.~~~.~~..~~~.~~~..~~~.~~",
            Color::rgb(255, 87, 34),
        ),
        RoadStyle::Ice => (
            "====/======/====/======/====/======/====",
            Color::rgb(224, 247, 250),
        ),
        RoadStyle::Seabed => (
            "~ . ~ . ~ . ~ . ~ . ~ . ~ . ~ . ~ . ~ . ",
            Color::rgb(255, 213, 79),
        ),
        RoadStyle::Sidewalk => (
            "──────[____]──────[____]──────[____]────",
            Color::rgb(189, 189, 189),
        ),
        RoadStyle::Roof => (
            "########################################",
            Color::rgb(120, 144, 156),
        ),
        RoadStyle::Grid => (
            "+---+---+---+---+---+---+---+---+---+---+",
            Color::rgb(0, 229, 255),
        ),
        RoadStyle::Crypt => (
            "oo__oo__oo__oo__oo__oo__oo__oo__oo__oo__",
            Color::rgb(117, 117, 117),
        ),
        RoadStyle::Savanna => (
            "__.-..-.__.-..-.__.-..-.__.-..-.__.-..-_",
            Color::rgb(215, 204, 200),
        ),
        RoadStyle::Mud => (
            "o..O..o..O..o..O..o..O..o..O..o..O..o..O",
            Color::rgb(109, 76, 65),
        ),
        RoadStyle::Tracks => (
            "=||===||===||===||===||===||===||===||= ",
            Color::rgb(144, 164, 174),
        ),
        RoadStyle::Checkerboard => (
            "[#][ ][#][ ][#][ ][#][ ][#][ ][#][ ][#] ",
            Color::rgb(238, 238, 238),
        ),
        RoadStyle::None => return,
    };

    let pchars: Vec<char> = pattern.chars().collect();
    if pchars.is_empty() {
        return;
    }

    let offset = (time * 4.0) as usize % pchars.len();

    for x in 0..width {
        let ch = pchars[(x + offset) % pchars.len()];
        if ch != ' ' {
            let _ = fb.set(x, cow_bottom_y, Cell::new(ch, fg));
        }
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
    if style == EnvironmentStyle::None || width == 0 || height == 0 {
        return;
    }

    match style {
        EnvironmentStyle::Pasture => {
            // Floating clouds in upper sky above the mountains
            let cloud_fg = Color::rgb(255, 255, 255);
            let cloud_x = ((time * 2.0) as usize) % width.max(1);
            let sky_y = ground_y.saturating_sub(5).min(1);
            if sky_y < fb.height {
                let s = "(   )";
                for (i, ch) in s.chars().enumerate() {
                    let x = (cloud_x + i) % width;
                    let _ = fb.set(x, sky_y, Cell::new(ch, cloud_fg));
                }
            }
            // Wildflowers blooming directly on the road / pasture ground
            let flower_fg = Color::rgb(255, 235, 59);
            for x in (5..width).step_by(18) {
                let _ = fb.set(x, ground_y, Cell::new('*', flower_fg));
            }
        }
        EnvironmentStyle::Inferno => {
            // Rising sparks and embers ascending from the magma ground
            let ember_fg = Color::rgb(255, 171, 0);
            for i in 0..6 {
                let seed = i * 17;
                let x = (seed + ((time * 5.0) as usize)) % width;
                let rise = (((time * 3.0) as usize) + i * 2) % (ground_y.max(1) + 1);
                let y = ground_y.saturating_sub(rise);
                let ch = if i % 2 == 0 { '*' } else { '^' };
                let _ = fb.set(x, y, Cell::new(ch, ember_fg));
            }
        }
        EnvironmentStyle::Ocean => {
            // Rising bubbles ascending from the seabed ground
            let bubble_fg = Color::rgb(128, 222, 234);
            for i in 0..5 {
                let seed = i * 23;
                let x = (seed + i * 7) % width;
                let rise = (((time * 2.5) as usize) + i * 2) % (ground_y.max(1) + 1);
                let y = ground_y.saturating_sub(rise);
                let ch = if i % 2 == 0 { 'o' } else { '.' };
                let _ = fb.set(x, y, Cell::new(ch, bubble_fg));
            }
        }
        EnvironmentStyle::Arctic => {
            // Drifting snowflakes settling toward the ice road
            let snow_fg = Color::rgb(255, 255, 255);
            for i in 0..8 {
                let seed = i * 19;
                let x = (seed + ((time * 1.5) as usize) + i) % width;
                let y = (((time * 2.0) as usize) + seed) % (ground_y.max(1) + 1);
                let ch = if i % 3 == 0 { '*' } else { '.' };
                let _ = fb.set(x, y, Cell::new(ch, snow_fg));
            }
        }
        EnvironmentStyle::City => {
            // Crescent moon and stars in the night sky above the skyline
            let moon_fg = Color::rgb(255, 245, 157);
            let mx = width.saturating_sub(6);
            let sky_y = ground_y.saturating_sub(5).min(1);
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
            let offset = ((time * 1.5) as usize) % 8;
            for x in (offset..width).step_by(12) {
                let _ = fb.set(x, ground_y, Cell::new('~', mist_fg));
            }
        }
        EnvironmentStyle::Forest => {
            // Leaves drifting down from canopy toward the woodland path
            let leaf_fg = Color::rgb(255, 179, 0);
            for i in 0..5 {
                let x = (i * 31 + ((time * 2.0) as usize)) % width;
                let y = (((time * 1.5) as usize) + i * 3) % (ground_y.max(1) + 1);
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
            if ((time * 3.0) as usize) % 2 == 0 {
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
                let y = (((time * 4.0) as usize) + i * 2) % (ground_y.max(1) + 1);
                let ch = if i % 2 == 0 { '0' } else { '1' };
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
                let x = (i * 27 + ((time * 3.0) as usize)) % width;
                let y = (i * 3) % (ground_y.max(1) + 1);
                let _ = fb.set(x, y, Cell::new('.', pollen_fg));
            }
        }
        EnvironmentStyle::Throne => {
            // Torch spark flicker beside the royal carpet
            let torch_fg = Color::rgb(255, 112, 67);
            if ((time * 4.0) as usize) % 2 == 0 && width > 4 {
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
    let w = fb.width;
    let h = fb.height;
    if w == 0 || h == 0 {
        return;
    }

    let mountain_base_y = road_y.saturating_sub(1);
    render_mountain(fb, mountain, mountain_base_y, w, time);
    render_trees(fb, mountain, env, mountain_base_y, w, time);
    render_environment(fb, env, road_y, w, h, time);
    render_road(fb, road, road_y, w, time);
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
