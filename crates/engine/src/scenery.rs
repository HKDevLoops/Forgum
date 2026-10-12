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
    let scale = ((mountain_base_y as f32) * 0.45).clamp(2.5, 14.0);
    let scale_mult = (scale / 2.5) * amp_mult;
    let scaled_base_h = base_h * scale_mult;
    let scaled_max_var = max_var * scale_mult;
    let max_allowed = (mountain_base_y as f32 * 0.85).max(3.0);

    let n_peaks = calculate_mountain_peaks_aspect(width, style, aspect).max(1) as f32;
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
/// across columns with smooth horizontal parallax scrolling and aspect ratio adaptation.
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
                    let _ = fb.set(x, y, Cell::new(ch, col));
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
        EnvironmentStyle::Ocean => 0.55_f32,
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

/// Determine the mathematical tree species / order for tree index n within an environment biome.
/// Uses a low-discrepancy Weyl sequence with Golden Ratio Phi (1.6180339887) and animal seed
/// to guarantee non-repetitive, natural mixed flora distribution across the viewport width.
pub fn calculate_tree_species(n: usize, env: EnvironmentStyle, animal_seed: u32) -> TreeSpecies {
    let pool = get_biome_species_pool(env);
    let seed_offset = ((animal_seed as f32 * PHI_RECIPROCAL).fract() * 0.77).abs();
    let weyl = ((n as f32 + 1.0) * PHI_RECIPROCAL + seed_offset).fract();
    let idx = ((weyl * pool.len() as f32).floor() as usize) % pool.len();
    pool[idx]
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

    let tree_green = Color::rgb(68, 138, 86); // Calibrated woodland sage
    let snow_white = Color::rgb(205, 222, 226); // Frosty morning mist
    let acacia_gold = Color::rgb(142, 158, 84); // Savanna olive gold
    let birch_leaves = Color::rgb(118, 162, 120); // Gentle meadow birch
    let dark_wood = Color::rgb(92, 92, 92); // Weathered timber
    let pine_green = Color::rgb(42, 92, 60); // Calming deep spruce
    let willow_green = Color::rgb(108, 142, 88); // Serene weeping willow sage
    let bonsai_green = Color::rgb(58, 112, 74); // Refined juniper
    let bamboo_green = Color::rgb(112, 154, 98); // Gentle bamboo jade
    let mangrove_green = Color::rgb(54, 104, 72); // Tidal mangrove olive
    let cactus_green = Color::rgb(82, 132, 92); // Succulent sage green
    let fern_green = Color::rgb(92, 136, 76); // Temperate forest fern
    let vine_green = Color::rgb(108, 142, 88); // Climbing ivy olive
    let shrub_green = Color::rgb(64, 118, 76); // Boxwood green
    let blooming_green = Color::rgb(96, 148, 104); // Delicate spring leaf
    let herb_green = Color::rgb(120, 146, 88); // Herbal tea green
    let cypress_green = Color::rgb(34, 76, 52); // Muted Mediterranean evergreen
    let maple_red = Color::rgb(188, 96, 76); // Warm autumnal amber (zero harsh glare)
    let cherry_pink = Color::rgb(232, 178, 192); // Delicate sakura blossom pink
    let baobab_green = Color::rgb(130, 156, 96); // Warm arid savanna canopy
    let redwood_green = Color::rgb(48, 96, 68); // Ancient redwood deep canopy

    // Natural Fibonacci phyllotaxis tree range across the visible span
    let avg_stride = min_dist + var_dist * 0.5;
    let start_tree =
        (((scroll_x as isize - 35).max(0) as f32) / (avg_stride + var_dist)).floor() as usize;
    // Start 6 trees earlier so wide canopies exiting on the left screen boundary do not clip prematurely
    let start_tree = start_tree.saturating_sub(6);
    let end_tree = (((scroll_x + width + 40) as f32 / min_dist).ceil() as usize) + 4;
    let fb_cell = std::cell::RefCell::new(fb);

    for t_idx in start_tree..=end_tree {
        let world_x = calculate_tree_position(t_idx, min_dist, var_dist);
        let screen_x = (world_x as isize) - (scroll_x as isize);
        if screen_x < -35 || screen_x >= (width as isize) + 20 {
            continue;
        }

        let target_tree_h = calculate_tree_height(t_idx, animal_h, anim, tree_base_y);
        let species = calculate_tree_species(t_idx, env, animal_seed);
        let trunk_w = calculate_trunk_width(species, target_tree_h, t_idx);

        let plot_row = |dx_start: isize, dy: isize, s: &str, col: Color| {
            let mut fb_guard = fb_cell.borrow_mut();
            for (i, c) in s.chars().enumerate() {
                if c == ' ' {
                    continue;
                }
                let px = screen_x + dx_start + (i as isize);
                if px >= 0 && (px as usize) < width {
                    let py = (tree_base_y as isize) + dy;
                    if py >= 0 && (py as usize) < fb_guard.height {
                        let (final_ch, final_col) = if !matches!(
                            c,
                            '|' | '/' | '\\' | '-' | '_' | '=' | '#' | ' ' | '[' | ']' | '!' | '+'
                        ) {
                            if let Some((orn_ch, orn_col)) =
                                get_foliage_ornament(species, t_idx, dx_start + (i as isize), dy, c)
                            {
                                (orn_ch, orn_col)
                            } else {
                                (c, col)
                            }
                        } else {
                            (c, col)
                        };
                        let _ =
                            fb_guard.set(px as usize, py as usize, Cell::new(final_ch, final_col));
                    }
                }
            }
        };

        let draw_trunk = |trunk_center: isize, top_dy: isize, width_cols: usize| {
            let mut fb_guard = fb_cell.borrow_mut();
            let mut plot_cell = |dx: isize, dy: isize, ch: char, col: Color| {
                if ch == ' ' {
                    return;
                }
                let px = screen_x + dx;
                if px >= 0 && (px as usize) < width {
                    let py = (tree_base_y as isize) + dy;
                    if py >= 0 && (py as usize) < fb_guard.height {
                        let _ = fb_guard.set(px as usize, py as usize, Cell::new(ch, col));
                    }
                }
            };
            for dy in top_dy..=0 {
                let y_from_base = (-dy) as usize;
                match width_cols {
                    1 => {
                        let (ch, col) = get_bark_glyph_and_color(species, 0, 1, y_from_base, t_idx);
                        plot_cell(trunk_center, dy, ch, col);
                    }
                    2 => {
                        let (ch_l, col_l) =
                            get_bark_glyph_and_color(species, 0, 2, y_from_base, t_idx);
                        let (ch_r, col_r) =
                            get_bark_glyph_and_color(species, 1, 2, y_from_base, t_idx);
                        plot_cell(trunk_center, dy, ch_l, col_l);
                        plot_cell(trunk_center + 1, dy, ch_r, col_r);
                    }
                    3 => {
                        let (ch_l, col_l) =
                            get_bark_glyph_and_color(species, 0, 3, y_from_base, t_idx);
                        let (ch_m, col_m) =
                            get_bark_glyph_and_color(species, 1, 3, y_from_base, t_idx);
                        let (ch_r, col_r) =
                            get_bark_glyph_and_color(species, 2, 3, y_from_base, t_idx);
                        plot_cell(trunk_center - 1, dy, ch_l, col_l);
                        plot_cell(trunk_center, dy, ch_m, col_m);
                        plot_cell(trunk_center + 1, dy, ch_r, col_r);
                    }
                    _ => {
                        let (ch, col) = get_bark_glyph_and_color(species, 0, 1, y_from_base, t_idx);
                        plot_cell(trunk_center, dy, ch, col);
                    }
                }
            }
        };

        match species {
            TreeSpecies::Oak => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   .---.   ", tree_green);
                    plot_row(0, -7, " .'     '. ", tree_green);
                    plot_row(0, -6, "/  (@)    \\", tree_green);
                    plot_row(0, -5, "|  (@)  (@)|", tree_green);
                    plot_row(0, -4, "|    (@)   |", tree_green);
                    plot_row(0, -3, " \\  _____ / ", tree_green);
                    draw_trunk(5, -2, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .---. ", tree_green);
                    plot_row(0, -3, "( @ @ )", tree_green);
                    plot_row(0, -2, " \\___/ ", tree_green);
                    draw_trunk(3, -1, trunk_w.min(2));
                } else {
                    plot_row(0, -2, "(@)", tree_green);
                    plot_row(0, -1, "(_)", tree_green);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Pine => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "    ^    ", pine_green);
                    plot_row(0, -7, "   /|\\   ", pine_green);
                    plot_row(0, -6, "  /*|*\\  ", pine_green);
                    plot_row(0, -5, " /* | *\\ ", pine_green);
                    plot_row(0, -4, "/ * | * \\", pine_green);
                    plot_row(0, -3, " /__|__\\ ", pine_green);
                    draw_trunk(4, -2, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "  ^  ", pine_green);
                    plot_row(0, -3, " /|\\ ", pine_green);
                    plot_row(0, -2, "//|\\\\", pine_green);
                    draw_trunk(2, -1, 1);
                } else {
                    plot_row(0, -2, " ^ ", pine_green);
                    plot_row(0, -1, "/*\\", pine_green);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Acacia => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, "  .-----------.  ", acacia_gold);
                    plot_row(0, -6, " (_____________) ", acacia_gold);
                    plot_row(0, -5, "   \\   |   /   ", acacia_gold);
                    plot_row(0, -4, "    \\  |  /    ", acacia_gold);
                    draw_trunk(8, -3, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -3, " _.~---~._ ", acacia_gold);
                    plot_row(0, -2, "   \\ | /   ", acacia_gold);
                    draw_trunk(5, -1, 1);
                } else {
                    plot_row(0, -1, "__~---~__", acacia_gold);
                    draw_trunk(4, 0, 1);
                }
            }
            TreeSpecies::SnowFir => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "    ^    ", snow_white);
                    plot_row(0, -7, "   /|\\   ", snow_white);
                    plot_row(0, -6, "  /*|*\\  ", snow_white);
                    plot_row(0, -5, " /* | *\\ ", snow_white);
                    plot_row(0, -4, "/ * | * \\", snow_white);
                    plot_row(0, -3, " /__|__\\ ", snow_white);
                    draw_trunk(4, -2, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "  ^  ", snow_white);
                    plot_row(0, -3, " /|\\ ", snow_white);
                    plot_row(0, -2, "/*|*\\", snow_white);
                    draw_trunk(2, -1, 1);
                } else {
                    plot_row(0, -2, " ^ ", snow_white);
                    plot_row(0, -1, "/*\\", snow_white);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::DeadTree => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "\\   /   \\", dark_wood);
                    plot_row(0, -7, " \\ / \\ / ", dark_wood);
                    plot_row(0, -6, "  \\ | /  ", dark_wood);
                    plot_row(0, -5, "   \\|/   ", dark_wood);
                    draw_trunk(4, -4, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "\\ / \\", dark_wood);
                    plot_row(0, -3, " \\|/ ", dark_wood);
                    draw_trunk(2, -2, 1);
                } else {
                    plot_row(0, -2, "\\ /", dark_wood);
                    plot_row(0, -1, " | ", dark_wood);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Birch => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   .---.   ", birch_leaves);
                    plot_row(0, -7, " .(* * *). ", birch_leaves);
                    plot_row(0, -6, "(* * * * *)", birch_leaves);
                    plot_row(0, -5, " (* * * *) ", birch_leaves);
                    plot_row(0, -4, "  \\  |  /  ", birch_leaves);
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .(*). ", birch_leaves);
                    plot_row(0, -3, "(* * *)", birch_leaves);
                    plot_row(0, -2, " \\_|_/ ", birch_leaves);
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "(*)", birch_leaves);
                    plot_row(0, -1, "/*\\", birch_leaves);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Palm => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "  \\  |  /  ", tree_green);
                    plot_row(0, -7, " --( * )-- ", tree_green);
                    plot_row(0, -6, "  /  |  \\  ", tree_green);
                    plot_row(0, -5, "    (o)    ", tree_green);
                    draw_trunk(5, -4, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " \\ | / ", tree_green);
                    plot_row(0, -3, "--(*)--", tree_green);
                    plot_row(0, -2, "  (o)  ", tree_green);
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "\\|/", tree_green);
                    plot_row(0, -1, "(*)", tree_green);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::WeepingWillow => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   .---.   ", willow_green);
                    plot_row(0, -7, " .( ;;; ). ", willow_green);
                    plot_row(0, -6, "(: | : | :)", willow_green);
                    plot_row(0, -5, " | : | : | ", willow_green);
                    plot_row(0, -4, " : | : | : ", willow_green);
                    plot_row(0, -3, " :   |   : ", willow_green);
                    draw_trunk(5, -2, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .---. ", willow_green);
                    plot_row(0, -3, "(:|:|:)", willow_green);
                    plot_row(0, -2, " : | : ", willow_green);
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "(;:;)", willow_green);
                    plot_row(0, -1, " :|: ", willow_green);
                    draw_trunk(2, 0, 1);
                }
            }
            TreeSpecies::Bonsai => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, "    (~~)    ", bonsai_green);
                    plot_row(0, -6, "   _/-'     ", bonsai_green);
                    plot_row(0, -5, " (~~~)  (~) ", bonsai_green);
                    plot_row(0, -4, "  /.-/  /   ", bonsai_green);
                    plot_row(0, -3, "    \\_./    ", bonsai_green);
                    draw_trunk(6, -2, 1);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "  (~~) ", bonsai_green);
                    plot_row(0, -3, " ~/ -' ", bonsai_green);
                    plot_row(0, -2, "(~~) | ", bonsai_green);
                    draw_trunk(5, -1, 1);
                } else {
                    plot_row(0, -2, " (~) ", bonsai_green);
                    plot_row(0, -1, " _/| ", bonsai_green);
                    draw_trunk(3, 0, 1);
                }
            }
            TreeSpecies::Bamboo => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "  ~^~  ~^~  ", bamboo_green);
                    plot_row(0, -7, " //|\\\\ //|\\\\ ", bamboo_green);
                    plot_row(0, -6, "   |-|   |-|   ", bamboo_green);
                    plot_row(0, -5, "   | |   | |   ", bamboo_green);
                    plot_row(0, -4, " \\ |-| / |-| \\ ", bamboo_green);
                    plot_row(0, -3, "   | |   | |   ", bamboo_green);
                    plot_row(0, -2, "   |-|   |-|   ", bamboo_green);
                    plot_row(0, -1, "   | |   | |   ", bamboo_green);
                    plot_row(0, 0, "   |-|   |-|   ", bamboo_green);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " ~^~ ", bamboo_green);
                    plot_row(0, -3, "//|\\\\", bamboo_green);
                    plot_row(0, -2, " |-| ", bamboo_green);
                    plot_row(0, -1, " | | ", bamboo_green);
                    plot_row(0, 0, " |-| ", bamboo_green);
                } else {
                    plot_row(0, -2, " ~^~ ", bamboo_green);
                    plot_row(0, -1, " |-| ", bamboo_green);
                    plot_row(0, 0, " | | ", bamboo_green);
                }
            }
            TreeSpecies::Mangrove => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "  .-----.  ", mangrove_green);
                    plot_row(0, -7, " (_______) ", mangrove_green);
                    plot_row(0, -6, " (/ @ @ \\) ", mangrove_green);
                    plot_row(0, -5, "  \\_____/  ", mangrove_green);
                    plot_row(0, -4, "    |||    ", BARK_RAW_UMBER);
                    plot_row(0, -3, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -2, "  / / \\ \\  ", BARK_RAW_UMBER);
                    plot_row(0, -1, " / /   \\ \\ ", BARK_RAW_UMBER);
                    plot_row(0, 0, "/ /     \\ \\", BARK_RAW_UMBER);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " (____) ", mangrove_green);
                    plot_row(0, -3, "  /||\\  ", BARK_RAW_UMBER);
                    plot_row(0, -2, " / || \\ ", BARK_RAW_UMBER);
                    plot_row(0, -1, "/  ||  \\", BARK_RAW_UMBER);
                    plot_row(0, 0, "/        \\", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, " (__) ", mangrove_green);
                    plot_row(0, -1, " /||\\ ", BARK_RAW_UMBER);
                    plot_row(0, 0, "/    \\", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::Cactus => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, "   _     _   ", cactus_green);
                    plot_row(0, -6, "  | |   | |  ", cactus_green);
                    plot_row(0, -5, "  | |_._| |  ", cactus_green);
                    plot_row(0, -4, "  |_   _  |  ", cactus_green);
                    plot_row(0, -3, "    | | |    ", cactus_green);
                    plot_row(0, -2, "    | : |    ", cactus_green);
                    plot_row(0, -1, "    | : |    ", cactus_green);
                    plot_row(0, 0, "    | : |    ", cactus_green);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " _._ ", cactus_green);
                    plot_row(0, -3, "|_:_|", cactus_green);
                    plot_row(0, -2, " |:| ", cactus_green);
                    plot_row(0, -1, " |:| ", cactus_green);
                    plot_row(0, 0, " |:| ", cactus_green);
                } else {
                    plot_row(0, -2, "(_)", cactus_green);
                    plot_row(0, -1, "|:|", cactus_green);
                    plot_row(0, 0, "|:|", cactus_green);
                }
            }
            TreeSpecies::Fern => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, " \\\\  |  // ", fern_green);
                    plot_row(0, -7, "  \\\\ | //  ", fern_green);
                    plot_row(0, -6, "===~*~====", fern_green);
                    plot_row(0, -5, " (/ | \\)  ", fern_green);
                    plot_row(0, -4, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -3, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -2, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, -1, "    ||    ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    ||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " \\ | / ", fern_green);
                    plot_row(0, -3, "({%*})", fern_green);
                    plot_row(0, -2, "  / \\  ", fern_green);
                    plot_row(0, -1, "  |||  ", BARK_RAW_UMBER);
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, " ,\\|/, ", fern_green);
                    plot_row(0, -1, "  %%%  ", fern_green);
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::CreeperVine => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   ~@~   ", vine_green);
                    plot_row(0, -7, "  ~/|/~  ", vine_green);
                    plot_row(0, -6, " ~/ | ~  ", vine_green);
                    plot_row(0, -5, " ~ \\|/~  ", vine_green);
                    plot_row(0, -4, "  ~/|~   ", vine_green);
                    plot_row(0, -3, " ~/ | ~  ", vine_green);
                    plot_row(0, -2, " ~ \\|/~  ", vine_green);
                    plot_row(0, -1, "  ~/|~   ", vine_green);
                    plot_row(0, 0, "   |~    ", vine_green);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " ~@~ ", vine_green);
                    plot_row(0, -3, "~/|/~", vine_green);
                    plot_row(0, -2, " ~|~ ", vine_green);
                    plot_row(0, -1, "~/|~ ", vine_green);
                    plot_row(0, 0, " |~  ", vine_green);
                } else {
                    plot_row(0, -2, " ~@ ", vine_green);
                    plot_row(0, -1, "~/|~", vine_green);
                    plot_row(0, 0, " |  ", vine_green);
                }
            }
            TreeSpecies::BerryShrub => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, "   .---.   ", shrub_green);
                    plot_row(0, -6, " .( *o* ). ", shrub_green);
                    plot_row(0, -5, "(* o * o *)", shrub_green);
                    plot_row(0, -4, "( o * o * )", shrub_green);
                    plot_row(0, -3, " (_______) ", shrub_green);
                    plot_row(0, -2, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -1, "   \\ | /   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    |||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .(o*o). ", shrub_green);
                    plot_row(0, -3, "(* o * o)", shrub_green);
                    plot_row(0, -2, " (_____) ", shrub_green);
                    plot_row(0, -1, "   |||   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "   |||   ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, " (o*o) ", shrub_green);
                    plot_row(0, -1, " (___) ", shrub_green);
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::BloomingShrub => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, "   .---.   ", blooming_green);
                    plot_row(0, -6, " .(@ % *). ", blooming_green);
                    plot_row(0, -5, "(* % @ * %)", blooming_green);
                    plot_row(0, -4, "(% * % @ *)", blooming_green);
                    plot_row(0, -3, " (_______) ", blooming_green);
                    plot_row(0, -2, "   / | \\   ", BARK_RAW_UMBER);
                    plot_row(0, -1, "   \\ | /   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "    |||    ", BARK_RAW_UMBER);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .(*%*). ", blooming_green);
                    plot_row(0, -3, "(@ * % @)", blooming_green);
                    plot_row(0, -2, " (_____) ", blooming_green);
                    plot_row(0, -1, "   |||   ", BARK_RAW_UMBER);
                    plot_row(0, 0, "   |||   ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, " (*%*) ", blooming_green);
                    plot_row(0, -1, " (___) ", blooming_green);
                    plot_row(0, 0, "  |||  ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::HerbalCluster => {
                if target_tree_h >= 8 {
                    plot_row(0, -7, " |  |  | ", herb_green);
                    plot_row(0, -6, " |i |i |i", herb_green);
                    plot_row(0, -5, " |i |i |i", herb_green);
                    plot_row(0, -4, " \\||/||/ ", herb_green);
                    plot_row(0, -3, "  \\ || / ", herb_green);
                    plot_row(0, -2, "   \\||/  ", herb_green);
                    plot_row(0, -1, "   /||\\  ", herb_green);
                    plot_row(0, 0, "   \"\"\"  ", herb_green);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " i | i ", herb_green);
                    plot_row(0, -3, " |i|i|i", herb_green);
                    plot_row(0, -2, " \\|/|/ ", herb_green);
                    plot_row(0, -1, "  \\|/  ", herb_green);
                    plot_row(0, 0, "  \"\"\"  ", herb_green);
                } else {
                    plot_row(0, -2, " \\|/ ", herb_green);
                    plot_row(0, -1, " |i| ", herb_green);
                    plot_row(0, 0, " \"\"  ", herb_green);
                }
            }
            TreeSpecies::Cypress => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   ^   ", cypress_green);
                    plot_row(0, -7, "  /|\\  ", cypress_green);
                    plot_row(0, -6, " (| |) ", cypress_green);
                    plot_row(0, -5, " (| |) ", cypress_green);
                    plot_row(0, -4, " (| |) ", cypress_green);
                    plot_row(0, -3, " (| |) ", cypress_green);
                    plot_row(0, -2, "  \\|/  ", cypress_green);
                    draw_trunk(3, -1, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "  ^  ", cypress_green);
                    plot_row(0, -3, " /|\\ ", cypress_green);
                    plot_row(0, -2, " (| |) ", cypress_green);
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, " ^ ", cypress_green);
                    plot_row(0, -1, "(|)", cypress_green);
                    draw_trunk(1, 0, 1);
                }
            }
            TreeSpecies::Maple => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   .---.   ", maple_red);
                    plot_row(0, -7, " ./{% %}\\. ", maple_red);
                    plot_row(0, -6, "/{% % % %}\\", maple_red);
                    plot_row(0, -5, "|% % % % %|", maple_red);
                    plot_row(0, -4, " \\{% % %}/ ", maple_red);
                    plot_row(0, -3, "  \\_____/  ", maple_red);
                    draw_trunk(5, -2, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .{%}\\. ", maple_red);
                    plot_row(0, -3, "({% %})", maple_red);
                    plot_row(0, -2, " \\___/ ", maple_red);
                    draw_trunk(3, -1, 1);
                } else {
                    plot_row(0, -2, "({%})", maple_red);
                    plot_row(0, -1, " \\_/ ", maple_red);
                    draw_trunk(2, 0, 1);
                }
            }
            TreeSpecies::CherryBlossom => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "   .---.   ", cherry_pink);
                    plot_row(0, -7, " .(* @ *). ", cherry_pink);
                    plot_row(0, -6, "(* @ * @ *)", cherry_pink);
                    plot_row(0, -5, "(* \\   / *)", cherry_pink);
                    plot_row(0, -4, " \\  \\_/  / ", cherry_pink);
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " .(*@*). ", cherry_pink);
                    plot_row(0, -3, "(* @ * *)", cherry_pink);
                    plot_row(0, -2, "  \\_|_/  ", cherry_pink);
                    draw_trunk(4, -1, 1);
                } else {
                    plot_row(0, -2, " (*@*) ", cherry_pink);
                    plot_row(0, -1, "  \\|/  ", cherry_pink);
                    draw_trunk(3, 0, 1);
                }
            }
            TreeSpecies::Baobab => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "  \\ | /  ", baobab_green);
                    plot_row(0, -7, " (_____) ", baobab_green);
                    plot_row(0, -6, "  /---\\  ", BARK_RAW_UMBER);
                    plot_row(0, -5, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -4, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -3, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -2, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, -1, " | ::: | ", BARK_RAW_UMBER);
                    plot_row(0, 0, "  \\---/  ", BARK_RAW_UMBER);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, " \\ | / ", baobab_green);
                    plot_row(0, -3, " (___) ", baobab_green);
                    plot_row(0, -2, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, -1, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, 0, " \\---/ ", BARK_RAW_UMBER);
                } else {
                    plot_row(0, -2, "  \\|/  ", baobab_green);
                    plot_row(0, -1, " |:::| ", BARK_RAW_UMBER);
                    plot_row(0, 0, " \\---/ ", BARK_RAW_UMBER);
                }
            }
            TreeSpecies::Redwood => {
                if target_tree_h >= 8 {
                    plot_row(0, -8, "     ^     ", redwood_green);
                    plot_row(0, -7, "    /|\\    ", redwood_green);
                    plot_row(0, -6, "   //|\\\\   ", redwood_green);
                    plot_row(0, -5, "  ///|\\\\\\  ", redwood_green);
                    plot_row(0, -4, "   //|\\\\   ", redwood_green);
                    draw_trunk(5, -3, trunk_w);
                } else if target_tree_h >= 4 {
                    plot_row(0, -4, "   ^   ", redwood_green);
                    plot_row(0, -3, "  /|\\  ", redwood_green);
                    plot_row(0, -2, " //|\\\\ ", redwood_green);
                    draw_trunk(3, -1, trunk_w.min(2));
                } else {
                    plot_row(0, -2, "  ^  ", redwood_green);
                    plot_row(0, -1, " /|\\ ", redwood_green);
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
            let ch = if (x + ((time * 1.5) as usize)).is_multiple_of(9) {
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
                '-' // smooth pavement
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
            let ch = if (x + ((time * 1.5) as usize)).is_multiple_of(7) {
                '|' // deep ice fracture
            } else if roughness > 0.45 {
                '-' // compressed glacial shelf
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
            let tile = (x + ((time * 2.0) as usize)) % 4;
            let ch = if tile == 0 { '|' } else { '/' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 230, 118);
            let node = (x + ((time * 3.0) as usize)) % 6;
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
            let tile = (x + ((time * 2.0) as usize)) % 4;
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
            let slab = (x + ((time * 2.8) as usize)) % 5;
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
            let joint = (x + ((time * 2.5) as usize)) % 10;
            let ch = if joint == 0 { '|' } else { ' ' };
            (ch, fg)
        }
        RoadStyle::Roof => {
            let fg = Color::rgb(90, 60, 50);
            let tile = (x + ((time * 2.0) as usize)) % 4;
            let ch = if tile == 0 { '|' } else { '_' };
            (ch, fg)
        }
        RoadStyle::Grid => {
            let fg = Color::rgb(0, 160, 90);
            let node = (x + ((time * 3.0) as usize)) % 6;
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
            let tile = (x + ((time * 2.0) as usize)) % 4;
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
                let _ = fb.set(x, cow_bottom_y, Cell::new(ch, fg));
            }
        }

        // Row 1: Subsurface foundation bed
        let subsurface_y = cow_bottom_y + 1;
        if subsurface_y < fb.height {
            for i in 0..chunk_len {
                let x = chunk_start + i;
                let (ch, fg) = calculate_road_subsurface(x, style, time);
                if ch != ' ' {
                    let _ = fb.set(x, subsurface_y, Cell::new(ch, fg));
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
                    let _ = fb.set(x, curb_y, Cell::new(ch, fg));
                }
            }
        }

        chunk_start += chunk_len;
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
            // A single glorious golden sun placed high in the daytime sky
            let sun_fg = Color::rgb(255, 214, 0);
            let sun_x = width.saturating_sub(10);
            let sky_y = (ground_y / 5).clamp(1, fb.height.saturating_sub(1));
            if sun_x < width && sky_y < fb.height {
                let _ = fb.set(sun_x, sky_y, Cell::new('*', sun_fg));
            }

            // Gentle drifting white clouds (fluffy shapes that never split or look like duplicate orbs)
            let cloud_fg = Color::rgb(240, 245, 255);
            let cloud_pattern = ".--.";
            let c_span = width + 8;
            let c_pos = ((time * 1.2) as usize) % c_span.max(1);
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
            // Render surface ocean waves horizon across the water boundary
            let surface_y = if ground_y >= 6 { 1 } else { 0 };
            let wave_fg = Color::rgb(72, 202, 228); // Luminous sea surface cyan
            for x in 0..width {
                let wave_harmonic = (x as f32 * 0.28 + time * 2.8).sin() * 0.45;
                let wave_ch = if ((x + (time * 3.0) as usize) % 4) < 2 {
                    '~'
                } else {
                    '-'
                };
                let y_wave = ((surface_y as f32 + wave_harmonic).round() as usize)
                    .min(ground_y.saturating_sub(1));
                let _ = fb.set(x, y_wave, Cell::new(wave_ch, wave_fg));
            }

            // Rising bubbles ascending through the water column from seabed to surface
            let bubble_fg = Color::rgb(128, 222, 234);
            for i in 0..8 {
                let seed = i * 19;
                let x = (seed + i * 11 + ((time * 1.5) as usize)) % width;
                let rise = (((time * 2.5) as usize) + i * 2)
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
            if ((time * 3.0) as usize).is_multiple_of(2) {
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
                let x = (i * 27 + ((time * 3.0) as usize)) % width;
                let y = (i * 3) % (ground_y.max(1) + 1);
                let _ = fb.set(x, y, Cell::new('.', pollen_fg));
            }
        }
        EnvironmentStyle::Throne => {
            // Torch spark flicker beside the royal carpet
            let torch_fg = Color::rgb(255, 112, 67);
            if ((time * 4.0) as usize).is_multiple_of(2) && width > 4 {
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
}
