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
        *Self::ALL_NON_NONE.choose(&mut rng).unwrap_or(&Self::Pasture)
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

    let (base_anim, eyes, tongue) = match clean.as_str() {
        "apt" => (BaseAnim::Walk, "oo", "  "),
        "armadillo" => (BaseAnim::Walk, "oo", "  "),
        "atat" => (BaseAnim::Walk, "==", "  "),
        "bearface" => (BaseAnim::Breathe, "oo", "  "),
        "beavis.zen" | "beavis" => (BaseAnim::Sway, "oo", "  "),
        "bees" => (BaseAnim::Fly, "oo", "  "),
        "bill-the-cat" => (BaseAnim::Talk, "oO", "U "),
        "bud-frogs" => (BaseAnim::Float, "@@", "  "),
        "bunny" => (BaseAnim::Walk, "oo", "  "),
        "cat" => (BaseAnim::Walk, "^^", "  "),
        "cat2" => (BaseAnim::Walk, "^^", "  "),
        "catfence" => (BaseAnim::Sway, "^^", "  "),
        "charizardvice" => (BaseAnim::Breathe, "oo", "  "),
        "charlie" => (BaseAnim::Talk, "oo", "U "),
        "claw-arm" => (BaseAnim::Pulse, "[]", "  "),
        "corgi" => (BaseAnim::Walk, "oo", "U "),
        "cower" => (BaseAnim::Breathe, "oo", "  "),
        "cowfee" => (BaseAnim::Breathe, "oo", "  "),
        "cthulhu-mini" => (BaseAnim::Float, "oo", "  "),
        "daemon" => (BaseAnim::Fly, "XX", "  "),
        "docker-whale" => (BaseAnim::Float, "oo", "  "),
        "doge" => (BaseAnim::Walk, "oo", "  "),
        "dolphin" => (BaseAnim::Float, "oo", "  "),
        "dragon" => (BaseAnim::Breathe, "oo", "  "),
        "dragon-and-cow" => (BaseAnim::Breathe, "oo", "  "),
        "duck" => (BaseAnim::Walk, "oo", "  "),
        "ebi_furai" => (BaseAnim::Float, "oo", "  "),
        "elephant" => (BaseAnim::Walk, "oo", "  "),
        "elephant-in-snake" => (BaseAnim::Breathe, "oo", "  "),
        "elephant2" => (BaseAnim::Walk, "oo", "  "),
        "eyes" => (BaseAnim::Pulse, "OO", "  "),
        "fat-banana" => (BaseAnim::Sway, "oo", "  "),
        "fat-cow" => (BaseAnim::Walk, "oo", "  "),
        "fence" => (BaseAnim::Sway, "||", "  "),
        "flaming-sheep" => (BaseAnim::Breathe, "oo", "  "),
        "fox" => (BaseAnim::Walk, "^^", "  "),
        "ghost" => (BaseAnim::Float, "oo", "  "),
        "ghostbusters" => (BaseAnim::Pulse, "oo", "  "),
        "glados" => (BaseAnim::Glitch, "[]", "  "),
        "goat" => (BaseAnim::Walk, "oo", "  "),
        "goat2" => (BaseAnim::Walk, "oo", "  "),
        "golden-eagle" => (BaseAnim::Fly, "oo", "  "),
        "hamster" => (BaseAnim::Walk, "oo", "  "),
        "happy-whale" => (BaseAnim::Float, "^^", "  "),
        "hedgehog" => (BaseAnim::Walk, "..", "  "),
        "hellokitty" => (BaseAnim::Walk, "oo", "  "),
        "hippie" => (BaseAnim::Sway, "oo", "  "),
        "hiya" => (BaseAnim::Talk, "^^", "U "),
        "hypno" => (BaseAnim::Pulse, "@@", "  "),
        "jellyfish" => (BaseAnim::Float, "oo", "  "),
        "jesus" => (BaseAnim::Float, "oo", "  "),
        "king" => (BaseAnim::Breathe, "oo", "  "),
        "kiss" => (BaseAnim::Talk, "oo", "U "),
        "kitten" => (BaseAnim::Walk, "^^", "  "),
        "kitty" => (BaseAnim::Walk, "^^", "  "),
        "knight" => (BaseAnim::Walk, "oo", "  "),
        "koala" => (BaseAnim::Breathe, "oo", "  "),
        "kosh" => (BaseAnim::Float, "oo", "  "),
        "lamb" => (BaseAnim::Walk, "oo", "  "),
        "lamb2" => (BaseAnim::Walk, "oo", "  "),
        "lobster" => (BaseAnim::Walk, "oo", "  "),
        "lollerskates" => (BaseAnim::Walk, "oo", "  "),
        "luke-koala" => (BaseAnim::Breathe, "oo", "  "),
        "mech-and-cow" => (BaseAnim::Walk, "oo", "  "),
        "meow" => (BaseAnim::Talk, "^^", "U "),
        "minotaur" => (BaseAnim::Walk, "XX", "  "),
        "mona-lisa" => (BaseAnim::Breathe, "oo", "  "),
        "moofasa" => (BaseAnim::Breathe, "oo", "  "),
        "mooghidjirah" => (BaseAnim::Breathe, "XX", "  "),
        "moojira" => (BaseAnim::Breathe, "XX", "  "),
        "moose" => (BaseAnim::Walk, "oo", "  "),
        "mule" => (BaseAnim::Walk, "oo", "  "),
        "mutilated" => (BaseAnim::Dissolve, "XX", "  "),
        "nyan_cat" => (BaseAnim::Fly, "^^", "  "),
        "octopus" => (BaseAnim::Float, "oo", "  "),
        "owl" => (BaseAnim::Breathe, "OO", "  "),
        "panther" => (BaseAnim::Walk, "oo", "  "),
        "pawn" => (BaseAnim::Walk, "oo", "  "),
        "periodic-table" => (BaseAnim::Pulse, "[]", "  "),
        "personality-sphere" => (BaseAnim::Float, "OO", "  "),
        "pig" => (BaseAnim::Walk, "oo", "  "),
        "pterodactyl" => (BaseAnim::Fly, "oo", "  "),
        "pufferfish" => (BaseAnim::Pulse, "oo", "  "),
        "queen" => (BaseAnim::Breathe, "oo", "  "),
        "radioactive-kitty" => (BaseAnim::Glitch, "^^", "  "),
        "ram" => (BaseAnim::Walk, "oo", "  "),
        "ren" => (BaseAnim::Talk, "oo", "U "),
        "rhino" => (BaseAnim::Walk, "oo", "  "),
        "rook" => (BaseAnim::Walk, "[]", "  "),
        "rooster" => (BaseAnim::Walk, "oo", "  "),
        "satanic" => (BaseAnim::Pulse, "XX", "  "),
        "sauron" => (BaseAnim::Pulse, "()", "  "),
        "seahorse" => (BaseAnim::Float, "oo", "  "),
        "seahorse-big" => (BaseAnim::Float, "oo", "  "),
        "sheep" => (BaseAnim::Walk, "oo", "  "),
        "shikato" => (BaseAnim::Breathe, "==", "  "),
        "shrug" => (BaseAnim::Sway, "oo", "  "),
        "skeleton" => (BaseAnim::Walk, "OO", "  "),
        "sloth" => (BaseAnim::Breathe, "oo", "  "),
        "small" => (BaseAnim::Walk, "oo", "  "),
        "smiling-octopus" => (BaseAnim::Float, "^^", "  "),
        "snoopy" => (BaseAnim::Breathe, "oo", "  "),
        "snoopyhouse" => (BaseAnim::Breathe, "oo", "  "),
        "snoopysleep" => (BaseAnim::Breathe, "--", "  "),
        "spidercow" => (BaseAnim::Walk, "88", "  "),
        "squid" => (BaseAnim::Float, "oo", "  "),
        "squirrel" => (BaseAnim::Walk, "..", "  "),
        "stegosaurus" => (BaseAnim::Walk, "oo", "  "),
        "stimpy" => (BaseAnim::Talk, "oO", "U "),
        "supermilker" => (BaseAnim::Pulse, "oo", "  "),
        "surgery" => (BaseAnim::Glitch, "XX", "  "),
        "telebears" => (BaseAnim::Walk, "oo", "  "),
        "three-eyes" => (BaseAnim::Breathe, "OOO", "  "),
        "tiger" => (BaseAnim::Walk, "oo", "  "),
        "tortoise" => (BaseAnim::Walk, "..", "  "),
        "turkey" => (BaseAnim::Walk, "oo", "  "),
        "turtle" => (BaseAnim::Float, "..", "  "),
        "tux" => (BaseAnim::Walk, "oo", "  "),
        "tux-big" => (BaseAnim::Walk, "oo", "  "),
        "tweety-bird" => (BaseAnim::Fly, "oo", "  "),
        "unipony" => (BaseAnim::Walk, "oo", "  "),
        "vader" => (BaseAnim::Breathe, "oo", "  "),
        "viper" => (BaseAnim::Sway, "oo", "U "),
        "vulpix" => (BaseAnim::Walk, "oo", "  "),
        "walrus" => (BaseAnim::Breathe, "oo", "  "),
        "weeping-angel" => (BaseAnim::Pulse, "XX", "  "),
        "whale" => (BaseAnim::Float, "oo", "  "),
        "wizard" => (BaseAnim::Pulse, "oo", "  "),
        "wolf" => (BaseAnim::Walk, "oo", "  "),
        "world" => (BaseAnim::Float, "oo", "  "),
        "yoda" => (BaseAnim::Float, "oo", "  "),
        "default" => (BaseAnim::Walk, "oo", "  "),
        "nyan" | "nyan-cat" | "nyancat" => (BaseAnim::Fly, "^^", "  "),
        _ => (BaseAnim::Walk, "oo", "  "),
    };

    AnimalProfile {
        name: biome.name,
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
    let peak_count = ((width as f32) / (lambda * (phi * 0.5))).round() as usize;
    peak_count.max(1)
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

    let scale = ((mountain_base_y as f32) * 0.45).clamp(2.5, 14.0);
    let scale_mult = scale / 2.5;
    let scaled_base_h = base_h * scale_mult;
    let scaled_max_var = max_var * scale_mult;
    let max_allowed = (mountain_base_y as f32 * 0.85).max(3.0);

    let n_peaks = calculate_mountain_peaks(width, style).max(1) as f32;
    let w_f = (width.max(20)) as f32;
    let omega = (2.0 * std::f32::consts::PI * n_peaks) / w_f;
    let t_offset = time * 1.2;
    let x_world = x + t_offset;

    match style {
        MountainStyle::Peaks | MountainStyle::Iceberg | MountainStyle::Gothic => {
            // Glacial horn erosion: power-pinched acute peaks with broad cirque valleys
            let theta1 = omega * x_world;
            let theta2 = omega * 2.0 * x_world + 1.2;
            let theta3 = omega * 3.618 * x_world + 2.4;
            let h1 = theta1.sin().abs().powf(1.85) * scaled_max_var;
            let h2 = (theta2.sin() * 0.35 + theta3.cos() * 0.20) * scaled_max_var;
            (scaled_base_h + h1 + h2).clamp(1.0, max_allowed)
        }
        MountainStyle::Plateau => {
            // Mesa plateau: hyperbolic tangent cliff saturation produces sheer flat-topped mesas
            let theta = omega * x_world;
            let h1 = (theta.sin() * 3.0).tanh() * scaled_max_var;
            (scaled_base_h + h1).clamp(1.0, max_allowed)
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
            (scaled_base_h + cone - crater).clamp(1.0, max_allowed)
        }
        MountainStyle::Skyline => {
            // Stepped building heights across urban skyline
            let theta1 = omega * x_world;
            let theta2 = omega * 2.23 * x_world + 0.8;
            let raw = theta1.sin() * 0.65 + theta2.sin() * 0.35;
            let steps = (raw * 4.0).floor() / 4.0;
            (scaled_base_h + steps * scaled_max_var).clamp(1.0, max_allowed)
        }
        _ => {
            // Natural rolling terrain: multi-harmonic smooth superposition
            let theta1 = omega * x_world;
            let theta2 = omega * 2.14 * x_world + 1.3;
            let theta3 = omega * 0.618 * x_world + 2.1;
            let h1 = theta1.sin() * (scaled_max_var * 0.60);
            let h2 = theta2.sin() * (scaled_max_var * 0.30);
            let h3 = theta3.cos() * (scaled_max_var * 0.20);
            (scaled_base_h + h1 + h2 + h3).clamp(1.0, max_allowed)
        }
    }
}

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

    for x in 0..width {
        let x_f = x as f32;
        let h_curr = calculate_mountain_height(x_f, width, mountain_base_y, style, time);
        let h_prev = calculate_mountain_height(x_f - 1.0, width, mountain_base_y, style, time);
        let h_next = calculate_mountain_height(x_f + 1.0, width, mountain_base_y, style, time);

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

/// Fibonacci / Golden Ratio reciprocal constant for optimal low-discrepancy flora dispersion.
pub const PHI_RECIPROCAL: f32 = 0.618_034;

/// Mathematical distance between tree k and tree k+1 using Fibonacci phyllotaxis and grove clustering.
/// Models biological root exclusion and natural tree clumping (alternating stands and clearings).
pub fn calculate_inter_tree_distance(k: usize, min_dist: f32, var_dist: f32) -> f32 {
    let phi_fract = ((k as f32) * PHI_RECIPROCAL).fract();
    let phi_sq_fract = ((k as f32) * (PHI_RECIPROCAL * PHI_RECIPROCAL)).fract();
    // Grove clustering factor: alternates between tighter tree clusters (stands) and open clearings
    let cluster_modulation = (phi_sq_fract * 2.0 * std::f32::consts::PI).cos() * (var_dist * 0.25);
    (min_dist + phi_fract * var_dist + cluster_modulation).max(min_dist * 0.7)
}

/// Cumulative world coordinate of tree index n based on integrated Fibonacci distance.
pub fn calculate_tree_position(n: usize, min_dist: f32, var_dist: f32) -> usize {
    let avg_stride = min_dist + var_dist * 0.5;
    let phi_fract = ((n as f32) * PHI_RECIPROCAL).fract();
    let harmonic_drift = ((n as f32) * 0.381966).sin() * (var_dist * 0.35);
    let pos = (n as f32) * avg_stride + (phi_fract - 0.5) * var_dist + harmonic_drift;
    pos.max(0.0).round() as usize
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
        EnvironmentStyle::City
        | EnvironmentStyle::Space
        | EnvironmentStyle::Cyber
        | EnvironmentStyle::None => return 0,
        _ => 0.90_f32,
    };

    let animal_scale_occupancy = if animal_height >= 12 {
        0.80_f32
    } else if animal_height <= 4 {
        1.15_f32
    } else {
        1.00_f32
    };

    let avg_stride = 22.0_f32;
    let count =
        ((width as f32 / avg_stride) * density_factor * animal_scale_occupancy).round() as usize;
    count.max(1)
}

/// Tree species variants for procedural mixed-stand rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSpecies {
    Oak,
    Pine,
    Acacia,
    SnowFir,
    DeadTree,
    Birch,
    Palm,
}

/// Determine the mathematical tree species / order for tree index n within an environment biome.
/// Uses a low-discrepancy Weyl sequence to guarantee natural mixed diversity without artificial repetition.
pub fn calculate_tree_species(n: usize, env: EnvironmentStyle, animal_seed: u32) -> TreeSpecies {
    let seed_offset = (animal_seed % 7) as f32 * 0.142857;
    let sequence_val = (((n as f32) * PHI_RECIPROCAL + seed_offset).fract() * 100.0) as usize;

    match env {
        EnvironmentStyle::Arctic => {
            if sequence_val % 3 == 0 {
                TreeSpecies::Pine
            } else {
                TreeSpecies::SnowFir
            }
        }
        EnvironmentStyle::Savanna => {
            if sequence_val % 4 == 0 {
                TreeSpecies::DeadTree
            } else {
                TreeSpecies::Acacia
            }
        }
        EnvironmentStyle::Graveyard => {
            if sequence_val % 3 == 0 {
                TreeSpecies::Oak
            } else {
                TreeSpecies::DeadTree
            }
        }
        EnvironmentStyle::Jurassic => {
            if sequence_val % 2 == 0 {
                TreeSpecies::Palm
            } else {
                TreeSpecies::Pine
            }
        }
        _ => {
            let r = sequence_val % 3;
            if r == 0 {
                TreeSpecies::Oak
            } else if r == 1 {
                TreeSpecies::Pine
            } else {
                TreeSpecies::Birch
            }
        }
    }
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
        BaseAnim::Walk => 1.25_f32,
        BaseAnim::Breathe => 1.35_f32,
        BaseAnim::Float | BaseAnim::Fly | BaseAnim::Abduction => 0.80_f32,
        BaseAnim::Sway => 1.15_f32,
        BaseAnim::Squish | BaseAnim::Liquid => 1.55_f32,
        _ => 1.20_f32,
    };

    let idx_f = tree_idx as f32;
    let variation = 1.0 + 0.22 * (idx_f * 2.39996).sin() + 0.12 * (idx_f * 1.61803).cos();

    let computed_h = base_animal_h * anim_mult * variation;
    let max_h = ((tree_base_y as f32) * 0.90).max(2.0);
    computed_h.clamp(2.0, max_h).round() as usize
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
    _mountain: MountainStyle,
    env: EnvironmentStyle,
    tree_base_y: usize,
    width: usize,
    time: f32,
    animal_name: Option<&str>,
    animal_height: Option<usize>,
    base_anim: Option<BaseAnim>,
) {
    if tree_base_y >= fb.height
        || width == 0
        || matches!(
            env,
            EnvironmentStyle::None
                | EnvironmentStyle::City
                | EnvironmentStyle::Space
                | EnvironmentStyle::Cyber
        )
    {
        return;
    }

    let animal_h = animal_height.unwrap_or(8);
    let anim = base_anim.unwrap_or_else(|| {
        if let Some(name) = animal_name {
            get_animal_profile(name).base_anim
        } else {
            BaseAnim::Walk
        }
    });
    let animal_seed = animal_name.map_or(0, |n| n.len() as u32);

    // Midground parallax speed
    let scroll_x = (time * 2.4) as usize;
    let min_dist = 16.0_f32;
    let var_dist = 12.0_f32;

    let tree_green = Color::rgb(76, 175, 80);
    let trunk_brown = Color::rgb(121, 85, 72);
    let snow_white = Color::rgb(224, 247, 250);
    let acacia_gold = Color::rgb(139, 195, 74);
    let dark_wood = Color::rgb(97, 97, 97);
    let birch_cream = Color::rgb(238, 238, 210);

    // Natural Fibonacci phyllotaxis tree range across the visible span
    let avg_stride = min_dist + var_dist * 0.5;
    let start_tree = (((scroll_x as isize - 35).max(0) as f32) / (avg_stride + var_dist)).floor() as usize;
    // Start 6 trees earlier so wide canopies exiting on the left screen boundary do not clip prematurely
    let start_tree = start_tree.saturating_sub(6);
    let end_tree = (((scroll_x + width + 40) as f32 / min_dist).ceil() as usize) + 4;

    for t_idx in start_tree..=end_tree {
        let world_x = calculate_tree_position(t_idx, min_dist, var_dist);
        let screen_x = (world_x as isize) - (scroll_x as isize);
        if screen_x < -35 || screen_x >= (width as isize) + 20 {
            continue;
        }

        let target_tree_h = calculate_tree_height(t_idx, animal_h, anim, tree_base_y);
        let species = calculate_tree_species(t_idx, env, animal_seed);

        let mut plot = |dx: isize, dy: isize, ch: char, col: Color| {
            if ch == ' ' {
                return;
            }
            let px = screen_x + dx;
            if px >= 0 && (px as usize) < width {
                let py = (tree_base_y as isize) + dy;
                if py >= 0 && (py as usize) < fb.height {
                    let _ = fb.set(px as usize, py as usize, Cell::new(ch, col));
                }
            }
        };

        match species {
            TreeSpecies::SnowFir => {
                if target_tree_h >= 8 {
                    plot(4, -8, '^', snow_white);
                    plot(3, -7, '/', snow_white);
                    plot(4, -7, '|', snow_white);
                    plot(5, -7, '\\', snow_white);
                    plot(2, -6, '/', snow_white);
                    plot(3, -6, '*', snow_white);
                    plot(4, -6, '|', snow_white);
                    plot(5, -6, '*', snow_white);
                    plot(6, -6, '\\', snow_white);
                    plot(1, -5, '/', snow_white);
                    plot(2, -5, '*', snow_white);
                    plot(4, -5, '|', snow_white);
                    plot(6, -5, '*', snow_white);
                    plot(7, -5, '\\', snow_white);
                    plot(0, -4, '/', snow_white);
                    plot(2, -4, '*', snow_white);
                    plot(4, -4, '|', snow_white);
                    plot(6, -4, '*', snow_white);
                    plot(8, -4, '\\', snow_white);
                    plot(1, -3, '/', snow_white);
                    plot(3, -3, '_', snow_white);
                    plot(4, -3, '|', snow_white);
                    plot(5, -3, '_', snow_white);
                    plot(7, -3, '\\', snow_white);
                    plot(4, -2, '|', dark_wood);
                    plot(4, -1, '|', dark_wood);
                    plot(4, 0, '|', dark_wood);
                } else if target_tree_h >= 4 {
                    plot(2, -4, '^', snow_white);
                    plot(1, -3, '/', snow_white);
                    plot(2, -3, '*', snow_white);
                    plot(3, -3, '\\', snow_white);
                    plot(0, -2, '/', snow_white);
                    plot(1, -2, '*', snow_white);
                    plot(2, -2, '|', snow_white);
                    plot(3, -2, '*', snow_white);
                    plot(4, -2, '\\', snow_white);
                    plot(2, -1, '|', dark_wood);
                    plot(2, 0, '|', dark_wood);
                } else if target_tree_h >= 2 {
                    plot(1, -2, '^', snow_white);
                    plot(0, -1, '/', snow_white);
                    plot(1, -1, '*', snow_white);
                    plot(2, -1, '\\', snow_white);
                    plot(1, 0, '|', dark_wood);
                }
            }
            TreeSpecies::Acacia => {
                if target_tree_h >= 7 {
                    for (i, c) in "  .-----------.  ".chars().enumerate() {
                        plot(i as isize, -7, c, acacia_gold);
                    }
                    for (i, c) in " (_____________) ".chars().enumerate() {
                        plot(i as isize, -6, c, acacia_gold);
                    }
                    for (i, c) in "   \\   |   /   ".chars().enumerate() {
                        plot(i as isize, -5, c, acacia_gold);
                    }
                    plot(4, -4, '\\', trunk_brown);
                    plot(7, -4, '|', trunk_brown);
                    plot(10, -4, '/', trunk_brown);
                    plot(5, -3, '\\', trunk_brown);
                    plot(7, -3, '|', trunk_brown);
                    plot(9, -3, '/', trunk_brown);
                    plot(7, -2, '|', trunk_brown);
                    plot(7, -1, '|', trunk_brown);
                    plot(7, 0, '|', trunk_brown);
                } else if target_tree_h >= 4 {
                    for (i, c) in " _.~---~._ ".chars().take(9).enumerate() {
                        plot(i as isize, -3, c, acacia_gold);
                    }
                    plot(2, -2, '\\', trunk_brown);
                    plot(4, -2, '/', trunk_brown);
                    plot(3, -1, '|', trunk_brown);
                    plot(3, 0, '|', trunk_brown);
                } else if target_tree_h >= 1 {
                    for (i, c) in "__~---~__".chars().take(5).enumerate() {
                        plot(i as isize, -1, c, acacia_gold);
                    }
                    plot(2, 0, '|', trunk_brown);
                }
            }
            TreeSpecies::DeadTree => {
                if target_tree_h >= 8 {
                    plot(0, -8, '\\', dark_wood);
                    plot(4, -8, '/', dark_wood);
                    plot(5, -8, '\\', dark_wood);
                    plot(8, -8, '/', dark_wood);
                    plot(1, -7, '\\', dark_wood);
                    plot(3, -7, '/', dark_wood);
                    plot(6, -7, '\\', dark_wood);
                    plot(7, -7, '/', dark_wood);
                    plot(2, -6, '\\', dark_wood);
                    plot(4, -6, '|', dark_wood);
                    plot(6, -6, '/', dark_wood);
                    plot(3, -5, '\\', dark_wood);
                    plot(4, -5, '|', dark_wood);
                    plot(5, -5, '/', dark_wood);
                    plot(4, -4, '|', dark_wood);
                    plot(4, -3, '|', dark_wood);
                    plot(4, -2, '|', dark_wood);
                    plot(4, -1, '|', dark_wood);
                    plot(4, 0, '|', dark_wood);
                } else if target_tree_h >= 4 {
                    plot(0, -4, '\\', dark_wood);
                    plot(4, -4, '/', dark_wood);
                    plot(1, -3, '\\', dark_wood);
                    plot(2, -3, '|', dark_wood);
                    plot(3, -3, '/', dark_wood);
                    plot(2, -2, '|', dark_wood);
                    plot(2, -1, '|', dark_wood);
                    plot(2, 0, '|', dark_wood);
                } else if target_tree_h >= 2 {
                    plot(0, -2, '\\', dark_wood);
                    plot(1, -2, '|', dark_wood);
                    plot(2, -2, '/', dark_wood);
                    plot(1, -1, '|', dark_wood);
                    plot(1, 0, '|', dark_wood);
                }
            }
            TreeSpecies::Pine => {
                if target_tree_h >= 8 {
                    plot(4, -8, '^', tree_green);
                    plot(3, -7, '/', tree_green);
                    plot(4, -7, '|', tree_green);
                    plot(5, -7, '\\', tree_green);
                    plot(2, -6, '/', tree_green);
                    plot(3, -6, '*', tree_green);
                    plot(4, -6, '|', tree_green);
                    plot(5, -6, '*', tree_green);
                    plot(6, -6, '\\', tree_green);
                    plot(1, -5, '/', tree_green);
                    plot(2, -5, '*', tree_green);
                    plot(4, -5, '|', tree_green);
                    plot(6, -5, '*', tree_green);
                    plot(7, -5, '\\', tree_green);
                    plot(0, -4, '/', tree_green);
                    plot(2, -4, '*', tree_green);
                    plot(4, -4, '|', tree_green);
                    plot(6, -4, '*', tree_green);
                    plot(8, -4, '\\', tree_green);
                    plot(1, -3, '/', tree_green);
                    plot(3, -3, '_', tree_green);
                    plot(4, -3, '|', tree_green);
                    plot(5, -3, '_', tree_green);
                    plot(7, -3, '\\', tree_green);
                    plot(4, -2, '|', trunk_brown);
                    plot(4, -1, '|', trunk_brown);
                    plot(4, 0, '|', trunk_brown);
                } else if target_tree_h >= 4 {
                    plot(2, -4, '^', tree_green);
                    plot(1, -3, '/', tree_green);
                    plot(2, -3, '|', tree_green);
                    plot(3, -3, '\\', tree_green);
                    plot(0, -2, '/', tree_green);
                    plot(1, -2, '/', tree_green);
                    plot(2, -2, '|', tree_green);
                    plot(3, -2, '\\', tree_green);
                    plot(4, -2, '\\', tree_green);
                    plot(2, -1, '|', trunk_brown);
                    plot(2, 0, '|', trunk_brown);
                } else if target_tree_h >= 2 {
                    plot(1, -2, '^', tree_green);
                    plot(0, -1, '/', tree_green);
                    plot(1, -1, '|', tree_green);
                    plot(2, -1, '\\', tree_green);
                    plot(1, 0, '|', trunk_brown);
                }
            }
            TreeSpecies::Birch => {
                if target_tree_h >= 6 {
                    plot(2, -5, '^', tree_green);
                    plot(1, -4, '(', tree_green);
                    plot(2, -4, '*', tree_green);
                    plot(3, -4, ')', tree_green);
                    plot(1, -3, '/', tree_green);
                    plot(2, -3, '|', birch_cream);
                    plot(3, -3, '\\', tree_green);
                    plot(2, -2, '|', birch_cream);
                    plot(2, -1, '|', birch_cream);
                    plot(2, 0, '|', birch_cream);
                } else if target_tree_h >= 3 {
                    plot(1, -2, '*', tree_green);
                    plot(1, -1, '|', birch_cream);
                    plot(1, 0, '|', birch_cream);
                }
            }
            TreeSpecies::Palm => {
                if target_tree_h >= 6 {
                    plot(1, -5, '\\', tree_green);
                    plot(3, -5, '^', tree_green);
                    plot(5, -5, '/', tree_green);
                    plot(2, -4, '\\', tree_green);
                    plot(3, -4, '*', tree_green);
                    plot(4, -4, '/', tree_green);
                    plot(3, -3, '/', trunk_brown);
                    plot(3, -2, '|', trunk_brown);
                    plot(3, -1, '|', trunk_brown);
                    plot(3, 0, '|', trunk_brown);
                } else if target_tree_h >= 3 {
                    plot(1, -2, '\\', tree_green);
                    plot(2, -2, '^', tree_green);
                    plot(3, -2, '/', tree_green);
                    plot(2, -1, '|', trunk_brown);
                    plot(2, 0, '|', trunk_brown);
                }
            }
            TreeSpecies::Oak => {
                if target_tree_h >= 8 {
                    for (i, c) in "   .---.   ".chars().enumerate() {
                        plot(i as isize, -8, c, tree_green);
                    }
                    for (i, c) in " .'     '. ".chars().enumerate() {
                        plot(i as isize, -7, c, tree_green);
                    }
                    for (i, c) in "/  (@)    \\".chars().enumerate() {
                        plot(i as isize, -6, c, tree_green);
                    }
                    for (i, c) in "|  (@)  (@)|".chars().enumerate() {
                        plot(i as isize, -5, c, tree_green);
                    }
                    for (i, c) in "|    (@)   |".chars().enumerate() {
                        plot(i as isize, -4, c, tree_green);
                    }
                    for (i, c) in " \\  _____ / ".chars().enumerate() {
                        plot(i as isize, -3, c, tree_green);
                    }
                    plot(5, -2, '|', trunk_brown);
                    plot(5, -1, '|', trunk_brown);
                    plot(5, 0, '|', trunk_brown);
                } else if target_tree_h >= 4 {
                    plot(1, -4, '(', tree_green);
                    plot(2, -4, '_', tree_green);
                    plot(3, -4, ')', tree_green);
                    plot(0, -3, '(', tree_green);
                    plot(2, -3, '@', tree_green);
                    plot(4, -3, ')', tree_green);
                    plot(0, -2, '(', tree_green);
                    plot(1, -2, '_', tree_green);
                    plot(2, -2, '_', tree_green);
                    plot(3, -2, '_', tree_green);
                    plot(4, -2, ')', tree_green);
                    plot(2, -1, '|', trunk_brown);
                    plot(2, 0, '|', trunk_brown);
                } else if target_tree_h >= 2 {
                    plot(0, -2, '(', tree_green);
                    plot(1, -2, '\'', tree_green);
                    plot(2, -2, ')', tree_green);
                    plot(0, -1, '(', tree_green);
                    plot(1, -1, '_', tree_green);
                    plot(2, -1, ')', tree_green);
                    plot(1, 0, '|', trunk_brown);
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
            let fg = Color::rgb(141, 110, 99);
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
            let fg = Color::rgb(158, 158, 158);
            // Paving stone geometry: periodic mortar joint with weathered surface variation
            let stone_idx = (x + ((time * 2.8) as usize)) % 5;
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
            let fg = if roughness > 0.65 {
                Color::rgb(255, 171, 0) // incandescent magma vent
            } else {
                Color::rgb(255, 87, 34) // cooling basalt crust
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
            let fg = Color::rgb(224, 247, 250);
            let ch = if (x + ((time * 1.5) as usize)) % 9 == 0 {
                '/' // glacial fissure / crevasse
            } else if roughness > 0.5 {
                '=' // glazed ice sheet
            } else {
                '-' // packed snow
            };
            (ch, fg)
        }
        RoadStyle::Seabed => {
            let fg = Color::rgb(255, 213, 79);
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
            let slab = (x + ((time * 2.5) as usize)) % 10;
            let ch = if slab == 0 {
                '|' // expansion joint
            } else if slab == 5 {
                '_'
            } else {
                '─' // smooth pavement
            };
            (ch, fg)
        }
        RoadStyle::Roof => {
            let fg = Color::rgb(120, 144, 156);
            let tile = (x + ((time * 2.0) as usize)) % 4;
            let ch = if tile == 0 { '#' } else { '=' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 229, 255);
            let node = (x + ((time * 3.0) as usize)) % 6;
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
            let tie = (x + ((time * 3.5) as usize)) % 5;
            let ch = if tie == 0 { '|' } else { '=' };
            (ch, fg)
        }
        RoadStyle::Checkerboard => {
            let fg = Color::rgb(238, 238, 238);
            let tile = (x + ((time * 2.0) as usize)) % 4;
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
            let stone_idx = (x + ((time * 2.8) as usize)) % 5;
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
                '≈' // convection current
            } else if roughness > 0.30 {
                '~' // molten wave
            } else {
                '=' // cooling magma shelf
            };
            (ch, fg)
        }
        RoadStyle::Ice => {
            let fg = Color::rgb(128, 222, 234);
            let ch = if (x + ((time * 1.5) as usize)) % 7 == 0 {
                '|' // deep ice fracture
            } else if roughness > 0.45 {
                '≡' // compressed glacial shelf
            } else {
                '=' // blue ice bed
            };
            (ch, fg)
        }
        RoadStyle::Tracks => {
            let tie = (x + ((time * 3.5) as usize)) % 5;
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
            let slab = (x + ((time * 2.5) as usize)) % 10;
            let ch = if slab == 0 {
                '|' // curb expansion seam
            } else {
                '▔' // street curb stone ledge
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
            let tile = (x + ((time * 2.0) as usize)) % 4;
            let ch = if tile == 0 { '|' } else { '/' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 230, 118);
            let node = (x + ((time * 3.0) as usize)) % 6;
            let ch = if node == 0 { '╩' } else { '│' };
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
                '≈' // bubbling mire
            } else if roughness > 0.35 {
                '~' // thick peat slurry
            } else {
                '=' // heavy bog bed
            };
            (ch, fg)
        }
        RoadStyle::Checkerboard => {
            let fg = Color::rgb(189, 189, 189);
            let tile = (x + ((time * 2.0) as usize)) % 4;
            let ch = if tile < 2 { ' ' } else { '#' };
            (ch, fg)
        }
        RoadStyle::None => (' ', Color::rgb(0, 0, 0)),
    }
}

/// Render ground / road baseline directly at cow_bottom_y with procedural harmonic texture synthesis.
/// Supports multi-row realistic ground depth (contact surface + subsurface foundation bed).
pub fn render_road(
    fb: &mut FrameBuffer,
    style: RoadStyle,
    cow_bottom_y: usize,
    width: usize,
    time: f32,
) {
    if style == RoadStyle::None || cow_bottom_y >= fb.height || width == 0 {
        return;
    }

    // Row 0: Contact surface
    for x in 0..width {
        let (ch, fg) = calculate_road_texture(x, style, time);
        if ch != ' ' {
            let _ = fb.set(x, cow_bottom_y, Cell::new(ch, fg));
        }
    }

    // Row 1: Subsurface foundation bed
    let subsurface_y = cow_bottom_y + 1;
    if subsurface_y < fb.height {
        for x in 0..width {
            let (ch, fg) = calculate_road_subsurface(x, style, time);
            if ch != ' ' {
                let _ = fb.set(x, subsurface_y, Cell::new(ch, fg));
            }
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
            let sky_y = (ground_y / 5).clamp(1, fb.height.saturating_sub(1));
            if sky_y < fb.height {
                let s = "(   )";
                for (i, ch) in s.chars().enumerate() {
                    let x = (cloud_x + i) % width;
                    let _ = fb.set(x, sky_y, Cell::new(ch, cloud_fg));
                }
            }
            if ground_y >= 12 {
                let cloud_x2 = ((time * 1.4 + 28.0) as usize) % width.max(1);
                let sky_y2 = (ground_y / 3).clamp(1, fb.height.saturating_sub(1));
                if sky_y2 < fb.height && sky_y2 != sky_y {
                    let s2 = "(      )";
                    for (i, ch) in s2.chars().enumerate() {
                        let x = (cloud_x2 + i) % width;
                        let _ = fb.set(x, sky_y2, Cell::new(ch, cloud_fg));
                    }
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
    let w = fb.width;
    let h = fb.height;
    if w == 0 || h == 0 {
        return;
    }

    let mountain_base_y = road_y.saturating_sub(1);
    render_mountain(fb, mountain, mountain_base_y, w, time);
    render_trees_with_animal(
        fb,
        mountain,
        env,
        mountain_base_y,
        w,
        time,
        animal_name,
        animal_height,
        base_anim,
    );
    render_environment(fb, env, road_y, w, h, time);
    render_road(fb, road, road_y, w, time);
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
        assert!(forest_trees > savanna_trees);
        assert_eq!(city_trees, 0);

        // Colossal mascot vs tiny mascot scale factor
        let colossal_trees = calculate_tree_count(100, EnvironmentStyle::Pasture, 16);
        let tiny_trees = calculate_tree_count(100, EnvironmentStyle::Pasture, 3);
        assert!(tiny_trees >= colossal_trees);
    }

    #[test]
    fn test_tree_species_sequencing() {
        let s0 = calculate_tree_species(0, EnvironmentStyle::Pasture, 42);
        let s1 = calculate_tree_species(1, EnvironmentStyle::Pasture, 42);
        let s2 = calculate_tree_species(2, EnvironmentStyle::Pasture, 42);
        // Ensure species are valid variants
        assert!(matches!(
            s0,
            TreeSpecies::Oak | TreeSpecies::Pine | TreeSpecies::Birch
        ));
        assert!(matches!(
            s1,
            TreeSpecies::Oak | TreeSpecies::Pine | TreeSpecies::Birch
        ));
        assert!(matches!(
            s2,
            TreeSpecies::Oak | TreeSpecies::Pine | TreeSpecies::Birch
        ));

        let arctic_s = calculate_tree_species(0, EnvironmentStyle::Arctic, 10);
        assert!(matches!(arctic_s, TreeSpecies::Pine | TreeSpecies::SnowFir));
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
