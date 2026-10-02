//! Biological natural biomes, recommended scenery, and natural color palettes
//! for all 132 Forgum mascots.
//!
//! Single authoritative source of truth shared across `forgum-platform`,
//! `forgum-engine`, and `forgum-tui`.

/// Biological natural color variation (coat morph, plumage, or subspecies pattern)
/// for a mascot that naturally exists in nature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaturalPaletteVariation {
    /// Distinct common name of the natural morph (e.g. "Holstein (White & Black Patches)", "White Bengal").
    pub name: &'static str,
    /// Brief biological / natural description of this color variation.
    pub description: &'static str,
    /// 5-slot color palette in hex notation.
    pub palette: [&'static str; 5],
    /// 5-slot color palette in RGB values.
    pub rgb: [(u8, u8, u8); 5],
}

/// Canonical biological biome, recommended scenery, and natural coloration metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MascotBiomeInfo {
    pub name: &'static str,
    pub category: &'static str,
    pub biome_name: &'static str,
    pub environment: &'static str,
    pub road: &'static str,
    pub mountain: &'static str,
    pub natural_palette: [&'static str; 5],
    pub natural_rgb: [(u8, u8, u8); 5],
    pub description: &'static str,
    pub movement_lore: &'static str,
}

impl MascotBiomeInfo {
    /// Authentic biological coat/morph variations for this mascot that exist in nature.
    pub fn variations(&self) -> &'static [NaturalPaletteVariation] {
        get_mascot_variations(self.name)
    }

    /// Biological habitat classification:
    /// - `aquatic`: Marine & underwater creatures (swimming, floating with bubbles)
    /// - `amphibious`: Transitional land/water creatures (walking, swimming, floating)
    /// - `aerial`: Soaring & flying creatures (atmospheric flight, hovering, gliding)
    /// - `inanimate`: Static monuments, objects, art, architecture, chess pieces
    /// - `ethereal`: Spectral ghosts, cosmic bodies, eyes, quantum anomalies
    /// - `serpentine`: Legless reptiles (slithering, undulating lateral sway)
    /// - `sedentary`: Very slow arboreal mammals (slow breathing, gentle sway)
    /// - `terrestrial`: Ground-walking quadrupeds, bipeds, and domestic/wild animals
    pub fn habitat(&self) -> &'static str {
        match self.name {
            // Aquatic (15):
            "whale" | "happy-whale" | "docker-whale" | "dolphin" | "octopus"
            | "smiling-octopus" | "jellyfish" | "seahorse" | "seahorse-big" | "pufferfish"
            | "squid" | "lobster" | "turtle" | "ebi_furai" | "cthulhu-mini" => "aquatic",

            // Amphibious (5):
            "bud-frogs" | "walrus" | "duck" | "tux" | "tux-big" => "amphibious",

            // Aerial / Flying (11):
            "golden-eagle" | "pterodactyl" | "tweety-bird" | "owl" | "bees"
            | "dragon" | "dragon-and-cow" | "charizardvice" | "daemon" | "nyan"
            | "nyan_cat" | "nyancat" | "unipony" => "aerial",

            // Inanimate objects / structures / chess pieces (14):
            "periodic-table" | "mona-lisa" | "fence" | "snoopyhouse" | "snoopysleep"
            | "cower" | "claw-arm" | "supermilker" | "surgery" | "apt"
            | "king" | "queen" | "rook" | "pawn" => "inanimate",

            // Ethereal / Cosmic / Supernatural / Celestial (14):
            "ghost" | "ghostbusters" | "sauron" | "weeping-angel" | "satanic"
            | "personality-sphere" | "world" | "eyes" | "kosh" | "jesus"
            | "wizard" | "glados" | "hypno" | "mutilated" => "ethereal",

            // Serpentine / Legless reptiles (2):
            "viper" | "elephant-in-snake" => "serpentine",

            // Sedentary / Slow arboreal animals (3):
            "sloth" | "koala" | "luke-koala" => "sedentary",

            // Terrestrial land-walking animals:
            _ => "terrestrial",
        }
    }

    /// True if the creature is biologically aquatic.
    pub fn is_aquatic(&self) -> bool {
        self.habitat() == "aquatic"
    }

    /// True if the creature is biologically amphibious.
    pub fn is_amphibious(&self) -> bool {
        self.habitat() == "amphibious"
    }

    /// True if the creature is an aerial flyer.
    pub fn is_aerial(&self) -> bool {
        self.habitat() == "aerial"
    }

    /// True if the mascot is an inanimate object or structure.
    pub fn is_inanimate(&self) -> bool {
        self.habitat() == "inanimate"
    }

    /// True if the mascot is an ethereal or cosmic entity.
    pub fn is_ethereal(&self) -> bool {
        self.habitat() == "ethereal"
    }

    /// True if the creature is a legless serpentine reptile.
    pub fn is_serpentine(&self) -> bool {
        self.habitat() == "serpentine"
    }

    /// True if the creature is a slow sedentary arboreal mammal.
    pub fn is_sedentary(&self) -> bool {
        self.habitat() == "sedentary"
    }

    /// True if the creature naturally walks on ground with stepping cycles.
    pub fn can_walk(&self) -> bool {
        match self.habitat() {
            "terrestrial" | "amphibious" => true,
            _ => false,
        }
    }

    /// Gracefully remap an incompatible locomotion or animation effect to the mascot's
    /// natural physical kinematics, returning `Some((remapped_effect, explanation))`
    /// or `None` if the effect is already compatible.
    pub fn remap_incompatible_effect(&self, effect: &str) -> Option<(&'static str, &'static str)> {
        let eff = effect.trim().to_ascii_lowercase();
        let is_walk = matches!(eff.as_str(), "walk" | "walks" | "walking");
        let is_fly = matches!(eff.as_str(), "fly" | "flying");

        match self.habitat() {
            "aquatic" if is_walk => Some((
                "swim",
                "aquatic creature; swimming/floating with bubbles is required",
            )),
            "aerial" if is_walk => Some((
                "fly",
                "airborne flyer; soaring flight kinematics is required",
            )),
            "inanimate" if is_walk || is_fly => Some((
                "pulse",
                "inanimate structure/object; stationary resonant pulse is required",
            )),
            "ethereal" if is_walk => Some((
                "float",
                "ethereal/cosmic entity; zero-gravity levitation is required",
            )),
            "serpentine" if is_walk => Some((
                "sway",
                "legless reptile; undulating lateral slither is required",
            )),
            "sedentary" if is_walk => Some((
                "sway",
                "sedentary arboreal creature; slow harmonic sway is required",
            )),
            _ => None,
        }
    }

    /// Check if an animation/effect is logically compatible with this creature's habitat.
    pub fn is_effect_compatible(&self, effect: &str) -> bool {
        let eff = effect.trim().to_ascii_lowercase();
        if eff == "default"
            || eff == "natural"
            || eff == "animal_natural"
            || eff == "dna"
            || eff == "random"
            || eff == "static"
        {
            return true;
        }
        match self.habitat() {
            "aquatic" => matches!(
                eff.as_str(),
                "float" | "swim" | "drift" | "bubble" | "pulse" | "wave" | "glitch" | "dissolve" | "particles"
            ),
            "aerial" => matches!(
                eff.as_str(),
                "fly" | "hover" | "glide" | "breathe" | "pulse" | "float" | "particles" | "glitch" | "dissolve"
            ),
            "inanimate" => matches!(
                eff.as_str(),
                "pulse" | "breathe" | "sway" | "glitch" | "dissolve" | "static"
            ),
            "ethereal" => matches!(
                eff.as_str(),
                "float" | "drift" | "pulse" | "glitch" | "dissolve" | "breathe" | "wave" | "particles"
            ),
            "serpentine" => matches!(
                eff.as_str(),
                "sway" | "drift" | "breathe" | "pulse" | "glitch" | "dissolve"
            ),
            "sedentary" => matches!(
                eff.as_str(),
                "breathe" | "sway" | "pulse" | "squish" | "dissolve"
            ),
            "amphibious" => matches!(
                eff.as_str(),
                "walk" | "swim" | "float" | "drift" | "breathe" | "bubble" | "sway" | "pulse" | "particles" | "glitch" | "dissolve" | "talk"
            ),
            _ => matches!(
                eff.as_str(),
                "walk" | "breathe" | "sway" | "pulse" | "talk" | "particles" | "glitch" | "dissolve" | "squish"
            ),
        }
    }
}

/// Static registry of all 132 mascots.
pub static ALL_MASCOTS: [MascotBiomeInfo; 132] = [
    MascotBiomeInfo {
        name: "default",
        category: "Farm & Domestic",
        biome_name: "Pasture & Rolling Meadows",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#1a1a1a", "#2c2c2c", "#ffb6c1", "#ffffff"],
        natural_rgb: [(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (255, 255, 255)],
        description: "Classic domestic dairy Holstein cow with iconic black and white markings and pink udder.",
        movement_lore: "Peaceful 4-phase walking gait across sunlit pastures with procedural chewing rhythm.",
    },
    MascotBiomeInfo {
        name: "cat",
        category: "Farm & Domestic",
        biome_name: "Homestead Garden & Porch",
        environment: "pasture",
        road: "cobblestone",
        mountain: "hills",
        natural_palette: ["#d35400", "#795548", "#ffffff", "#ffb6c1", "#212121"],
        natural_rgb: [(211, 84, 0), (121, 85, 72), (255, 255, 255), (255, 182, 193), (33, 33, 33)],
        description: "Calico farm feline with warm ginger, chocolate brown, crisp white bib, and pink nose.",
        movement_lore: "Stealthy low-slung stalking prowl with delicate paw placements and tail balance flickers.",
    },
    MascotBiomeInfo {
        name: "cat2",
        category: "Farm & Domestic",
        biome_name: "Urban Cobblestone Patio",
        environment: "city",
        road: "cobblestone",
        mountain: "skyline",
        natural_palette: ["#78909c", "#546e7a", "#cfd8dc", "#ff80ab", "#69f0ae"],
        natural_rgb: [(120, 144, 156), (84, 110, 122), (207, 216, 220), (255, 128, 171), (105, 240, 174)],
        description: "Russian Blue / Silver Tabby with luminous jade-green eyes and shimmering silver coat.",
        movement_lore: "Graceful athletic rooftop traversal and curious harmonic head tilts.",
    },
    MascotBiomeInfo {
        name: "catfence",
        category: "Farm & Domestic",
        biome_name: "Rustic Paddock Fence",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#d35400", "#795548", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(211, 84, 0), (121, 85, 72), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Tabby cat perched curiously along weathered cedar farm fence rails.",
        movement_lore: "Delicate tightrope balance along wooden rails with periodic tail counter-sways.",
    },
    MascotBiomeInfo {
        name: "charlie",
        category: "Farm & Domestic",
        biome_name: "Sunny Country Paddock",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#5d4037", "#f5f5f5", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (245, 245, 245), (255, 128, 171), (255, 255, 255)],
        description: "Domestic chestnut steer with a distinctive white star blaze and gentle disposition.",
        movement_lore: "Heavy deliberate bovine amble with expressive head bobbing.",
    },
    MascotBiomeInfo {
        name: "corgi",
        category: "Farm & Domestic",
        biome_name: "Royal Palace Grounds",
        environment: "pasture",
        road: "cobblestone",
        mountain: "castle",
        natural_palette: ["#e59866", "#d35400", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(229, 152, 102), (211, 84, 0), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Pembroke Welsh Corgi with golden-red foxy coat, crisp white ruff, and bubbly spirit.",
        movement_lore: "Enthusiastic short-legged trot with joyful rhythmic rear-waddle.",
    },
    MascotBiomeInfo {
        name: "bunny",
        category: "Farm & Domestic",
        biome_name: "Clover Meadow & Warrens",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#f5f5f5", "#eceff1", "#ffb6c1", "#ffffff"],
        natural_rgb: [(255, 255, 255), (245, 245, 245), (236, 239, 241), (255, 182, 193), (255, 255, 255)],
        description: "Pure snowy white meadow bunny with delicate soft pink inner ears and twitching nose.",
        movement_lore: "Nimble double-hop bounding gait interspersed with alert ear-swiveling pauses.",
    },
    MascotBiomeInfo {
        name: "doge",
        category: "Farm & Domestic",
        biome_name: "Zen Garden & Temple Steps",
        environment: "pasture",
        road: "cobblestone",
        mountain: "garden",
        natural_palette: ["#e59866", "#d35400", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(229, 152, 102), (211, 84, 0), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Shiba Inu with iconic golden-tan sesame coat, pristine white urajiro, and soulful eyes.",
        movement_lore: "Proud spirited strut with curled-tail balance and quizzical side-head tilts.",
    },
    MascotBiomeInfo {
        name: "fat-cow",
        category: "Farm & Domestic",
        biome_name: "Lush Clover Pasture",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#37474f", "#cfd8dc", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (55, 71, 79), (207, 216, 220), (255, 128, 171), (255, 255, 255)],
        description: "Plump prize-winning farm cow contentedly grazing on sweet springtime clover.",
        movement_lore: "Slow ponderous heavy-cadence waddle with rhythmic tail swishes against flies.",
    },
    MascotBiomeInfo {
        name: "goat",
        category: "Farm & Domestic",
        biome_name: "Rocky Highland Escarpment",
        environment: "pasture",
        road: "cobblestone",
        mountain: "peaks",
        natural_palette: ["#d7ccc8", "#8d6e63", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(215, 204, 200), (141, 110, 99), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Sure-footed alpine goat with stony fleece, agile cleft hooves, and amber slit pupils.",
        movement_lore: "High-stepping agile leap and bound across steep rocky terrain.",
    },
    MascotBiomeInfo {
        name: "goat2",
        category: "Farm & Domestic",
        biome_name: "Mountain Crag Sanctuary",
        environment: "pasture",
        road: "cobblestone",
        mountain: "peaks",
        natural_palette: ["#cfd8dc", "#78909c", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(207, 216, 220), (120, 144, 156), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Weathered wild mountain billy goat with swept-back ridged horns and chin beard.",
        movement_lore: "Sturdy calculated climbing strides navigating sheer cliffside promontories.",
    },
    MascotBiomeInfo {
        name: "hippie",
        category: "Farm & Domestic",
        biome_name: "Woodstock Wildflower Field",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ff007f", "#00e5ff", "#ffff00", "#76ff03", "#ffffff"],
        natural_rgb: [(255, 0, 127), (0, 229, 255), (255, 255, 0), (118, 255, 3), (255, 255, 255)],
        description: "Peace-loving bohemian musician draped in dazzling rainbow tie-dye velvet vest.",
        movement_lore: "Harmonic rhythmic acoustic sway radiating good vibrations across the valley.",
    },
    MascotBiomeInfo {
        name: "kitty",
        category: "Farm & Domestic",
        biome_name: "Sunlit Parlor Rug",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#e67e22", "#6d4c41", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(230, 126, 34), (109, 76, 65), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Spirited ginger domestic kitten with playful bright eyes and soft cream belly.",
        movement_lore: "High-energy bouncing pounce with animated tail flicks and inquisitive whisker twitches.",
    },
    MascotBiomeInfo {
        name: "kitten",
        category: "Farm & Domestic",
        biome_name: "Cozy Homestead Barn",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#d35400", "#795548", "#ffffff", "#ffb6c1", "#212121"],
        natural_rgb: [(211, 84, 0), (121, 85, 72), (255, 255, 255), (255, 182, 193), (33, 33, 33)],
        description: "Inquisitive young calico kitten exploring haystack corners and wooden rafters.",
        movement_lore: "Light-footed curious scurrying with tiny paws and upright curiosity antenna tail.",
    },
    MascotBiomeInfo {
        name: "meow",
        category: "Farm & Domestic",
        biome_name: "Moonlit Slate Rooftops",
        environment: "city",
        road: "roof",
        mountain: "skyline",
        natural_palette: ["#455a64", "#263238", "#cfd8dc", "#ff80ab", "#00e676"],
        natural_rgb: [(69, 90, 100), (38, 50, 56), (207, 216, 220), (255, 128, 171), (0, 230, 118)],
        description: "Sleek tuxedo feline with midnight coat, pristine white paws, and piercing blue gaze.",
        movement_lore: "Silent shadow-stepping glide along ceramic roof crests with fluid serpentine spine.",
    },
    MascotBiomeInfo {
        name: "hamster",
        category: "Farm & Domestic",
        biome_name: "Golden Grain Burrow",
        environment: "pasture",
        road: "cobblestone",
        mountain: "hills",
        natural_palette: ["#d4a373", "#8d6e63", "#fff8e1", "#ff80ab", "#ffffff"],
        natural_rgb: [(212, 163, 115), (141, 110, 99), (255, 248, 225), (255, 128, 171), (255, 255, 255)],
        description: "Chubby Syrian hamster with golden-brown fur, cream underbelly, and stuffed seed pouches.",
        movement_lore: "Frenetic rapid-step scurry with adorable nose-twitching sniffing intervals.",
    },
    MascotBiomeInfo {
        name: "mule",
        category: "Farm & Domestic",
        biome_name: "Canyon Pack Trail",
        environment: "pasture",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#5c4033", "#795548", "#f5f5f5", "#d7ccc8", "#ffffff"],
        natural_rgb: [(92, 64, 51), (121, 85, 72), (245, 245, 245), (215, 204, 200), (255, 255, 255)],
        description: "Dependable pack mule with rich chestnut-brown coat, pale muzzle ring, and long ears.",
        movement_lore: "Steadfast rhythmic endurance pack march unyielding across rough mountain trails.",
    },
    MascotBiomeInfo {
        name: "pig",
        category: "Farm & Domestic",
        biome_name: "Mud Wallow & Orchard",
        environment: "pasture",
        road: "mud",
        mountain: "hills",
        natural_palette: ["#ffb6c1", "#ff80ab", "#fce4ec", "#ec407a", "#ffffff"],
        natural_rgb: [(255, 182, 193), (255, 128, 171), (252, 228, 236), (236, 64, 122), (255, 255, 255)],
        description: "Cheerful rosy pink domestic pig with curly corkscrew tail and disk snout.",
        movement_lore: "Bubbly trot and snout-rooting through fertile soil and cooling mud baths.",
    },
    MascotBiomeInfo {
        name: "ram",
        category: "Farm & Domestic",
        biome_name: "High Alpine Windswept Ridge",
        environment: "pasture",
        road: "cobblestone",
        mountain: "peaks",
        natural_palette: ["#f5f5f5", "#9e9e9e", "#ffffff", "#bcaaa4", "#ffffff"],
        natural_rgb: [(245, 245, 245), (158, 158, 158), (255, 255, 255), (188, 170, 164), (255, 255, 255)],
        description: "Majestic bighorn ram with dense snowy fleece and formidable curved ridged horns.",
        movement_lore: "Powerful defiant strides pacing the mountain precipice with dominant horn displays.",
    },
    MascotBiomeInfo {
        name: "rooster",
        category: "Farm & Domestic",
        biome_name: "Dawn Farmyard Perch",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#e53935", "#2e7d32", "#fbc02d", "#ff9800", "#ffffff"],
        natural_rgb: [(229, 57, 53), (46, 125, 50), (251, 192, 45), (255, 152, 0), (255, 255, 255)],
        description: "Proud barnyard cockerel with fiery crimson comb, iridescent midnight tail, and gold hackles.",
        movement_lore: "Strutting parade march puffing out chest plumage before greeting the morning sun.",
    },
    MascotBiomeInfo {
        name: "sheep",
        category: "Farm & Domestic",
        biome_name: "Rolling Green Downs",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#e0e0e0", "#cfd8dc", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (224, 224, 224), (207, 216, 220), (255, 128, 171), (255, 255, 255)],
        description: "Fluffy woolly sheep grazing serenely on dew-covered clover and wild ryegrass.",
        movement_lore: "Gentle synchronized flock amble moving like rolling white clouds across green pastures.",
    },
    MascotBiomeInfo {
        name: "turkey",
        category: "Farm & Domestic",
        biome_name: "Autumn Oak Woodlands",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#d32f2f", "#bcaaa4", "#e53935", "#ffffff"],
        natural_rgb: [(141, 110, 99), (211, 47, 47), (188, 170, 164), (229, 57, 53), (255, 255, 255)],
        description: "Wild woodland turkey with shimmering bronze plumage, crimson wattle, and sky-blue head.",
        movement_lore: "Stately fan-tailed strut displaying iridescent plumage through forest clearings.",
    },
    MascotBiomeInfo {
        name: "armadillo",
        category: "Wild & Safari",
        biome_name: "Arid Scrubland & Burrows",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#a1887f", "#8d6e63", "#f5f5f5", "#ff80ab", "#ffffff"],
        natural_rgb: [(161, 136, 127), (141, 110, 99), (245, 245, 245), (255, 128, 171), (255, 255, 255)],
        description: "Nine-banded armadillo with leathery protective carapace and formidable digging claws.",
        movement_lore: "Low ground-sniffing trot ready to curl into an impenetrable armored sphere.",
    },
    MascotBiomeInfo {
        name: "bearface",
        category: "Wild & Safari",
        biome_name: "Old-Growth Pine Woods",
        environment: "forest",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#8d6e63", "#5d4037", "#d7ccc8", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (215, 204, 200), (255, 128, 171), (255, 255, 255)],
        description: "Grizzly bear with deep umber fur, cinnamon snout ruff, and watchful honey-amber eyes.",
        movement_lore: "Heavy lumbering apex stroll with colossal shoulder mass sway.",
    },
    MascotBiomeInfo {
        name: "elephant",
        category: "Wild & Safari",
        biome_name: "African Veldt & Waterhole",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#90a4ae", "#546e7a", "#fffde7", "#ffb6c1", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 253, 231), (255, 182, 193), (255, 255, 255)],
        description: "Majestic African matriarch elephant with weathered wrinkled hide and gleaming ivory tusks.",
        movement_lore: "Silent dignified march cushioned by elastic foot pads with rhythmic trunk sways.",
    },
    MascotBiomeInfo {
        name: "elephant2",
        category: "Wild & Safari",
        biome_name: "Savanna Baobab Glade",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#90a4ae", "#546e7a", "#fffde7", "#ffb6c1", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 253, 231), (255, 182, 193), (255, 255, 255)],
        description: "Bull elephant proudly trumpeting across the acacia canopy under the midday sun.",
        movement_lore: "Commanding ground-shaking stride surveying expansive African horizons.",
    },
    MascotBiomeInfo {
        name: "elephant-in-snake",
        category: "Wild & Safari",
        biome_name: "Sahara Desert Erg",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#90a4ae", "#43a047", "#fffde7", "#ffb6c1", "#ffffff"],
        natural_rgb: [(144, 164, 174), (67, 160, 71), (255, 253, 231), (255, 182, 193), (255, 255, 255)],
        description: "The Little Prince's philosophical mystery: a giant boa constrictor digesting an elephant.",
        movement_lore: "Hypnotic undulating serpentine crawl harboring a massive interior secret.",
    },
    MascotBiomeInfo {
        name: "fox",
        category: "Wild & Safari",
        biome_name: "Sun-Dappled Woodland Edge",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#e65100", "#8d6e63", "#ffffff", "#ff80ab", "#ffd54f"],
        natural_rgb: [(230, 81, 0), (141, 110, 99), (255, 255, 255), (255, 128, 171), (255, 213, 79)],
        description: "Red fox with fiery auburn pelt, pure snow-white chest and tail brush, and black socks.",
        movement_lore: "Nimble silent bounding leap pouncing onto woodland moss.",
    },
    MascotBiomeInfo {
        name: "hedgehog",
        category: "Wild & Safari",
        biome_name: "Woodland Leaf Litter & Hedgerow",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#5d4037", "#fff3e0", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (255, 243, 224), (255, 128, 171), (255, 255, 255)],
        description: "European hedgehog covered in thousands of sharp banded quills and soft furry underbelly.",
        movement_lore: "Busy shuffling trot rustling through autumn oak leaves.",
    },
    MascotBiomeInfo {
        name: "koala",
        category: "Wild & Safari",
        biome_name: "Eucalyptus Forest Canopy",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#90a4ae", "#546e7a", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Australian koala with dense silver-grey fleece, fluffy white ear tufts, and black nose.",
        movement_lore: "Slow deliberate branch-hugging climb chewing aromatic eucalyptus gum leaves.",
    },
    MascotBiomeInfo {
        name: "luke-koala",
        category: "Wild & Safari",
        biome_name: "Jedi Temple Eucalyptus Grove",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#90a4ae", "#546e7a", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Force-sensitive koala armed with glowing cyan lightsaber maintaining peace in the trees.",
        movement_lore: "Meditative branch hover channeling the living Force through the forest.",
    },
    MascotBiomeInfo {
        name: "moofasa",
        category: "Wild & Safari",
        biome_name: "Serengeti Pride Rock",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#d4a373", "#795548", "#fff8e1", "#ff80ab", "#ffd54f"],
        natural_rgb: [(212, 163, 115), (121, 85, 72), (255, 248, 225), (255, 128, 171), (255, 213, 79)],
        description: "Regal golden lion monarch with voluminous dark mane commanding the golden savanna.",
        movement_lore: "Regal predatory stride across Pride Rock surveying the circle of life.",
    },
    MascotBiomeInfo {
        name: "panther",
        category: "Wild & Safari",
        biome_name: "Moonlit Tropical Rainforest",
        environment: "forest",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#424242", "#263238", "#cfd8dc", "#ff80ab", "#00e676"],
        natural_rgb: [(66, 66, 66), (38, 50, 56), (207, 216, 220), (255, 128, 171), (0, 230, 118)],
        description: "Melanistic black leopard with glossy obsidian coat and piercing fluorescent green eyes.",
        movement_lore: "Ghostly soundless muscular glide slipping like liquid night through tangled roots.",
    },
    MascotBiomeInfo {
        name: "rhino",
        category: "Wild & Safari",
        biome_name: "African Bushveld Scrub",
        environment: "savanna",
        road: "savanna",
        mountain: "plateau",
        natural_palette: ["#78909c", "#546e7a", "#fffde7", "#b0bec5", "#ffffff"],
        natural_rgb: [(120, 144, 156), (84, 110, 122), (255, 253, 231), (176, 190, 197), (255, 255, 255)],
        description: "White rhinoceros with thick plate-like armored hide and prehistoric keratin horn.",
        movement_lore: "Heavy bulldozer charge carving through dense savanna acacia thornbush.",
    },
    MascotBiomeInfo {
        name: "sloth",
        category: "Wild & Safari",
        biome_name: "Amazonian Cloud Forest",
        environment: "forest",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#8d6e63", "#558b2f", "#d7ccc8", "#3e2723", "#ffffff"],
        natural_rgb: [(141, 110, 99), (85, 139, 47), (215, 204, 200), (62, 39, 35), (255, 255, 255)],
        description: "Three-toed sloth covered in symbiotic green algae hanging inverted from high branches.",
        movement_lore: "Ultra-slow harmonic pendulum suspension celebrating the joy of unhurried life.",
    },
    MascotBiomeInfo {
        name: "telebears",
        category: "Wild & Safari",
        biome_name: "Broadcast Antenna Peak",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#1565c0", "#ffb300", "#e3f2fd", "#ff80ab", "#ffffff"],
        natural_rgb: [(21, 101, 192), (255, 179, 0), (227, 242, 253), (255, 128, 171), (255, 255, 255)],
        description: "High-tech broadcast bear transmitting telemetry packets across microwave relay stations.",
        movement_lore: "Synchronized digital step pulsing with RF signal wavefronts.",
    },
    MascotBiomeInfo {
        name: "tiger",
        category: "Wild & Safari",
        biome_name: "Sundarbans Mangrove Wilderness",
        environment: "savanna",
        road: "savanna",
        mountain: "peaks",
        natural_palette: ["#e65100", "#263238", "#ffffff", "#ff80ab", "#69f0ae"],
        natural_rgb: [(230, 81, 0), (38, 50, 56), (255, 255, 255), (255, 128, 171), (105, 240, 174)],
        description: "Bengal tiger with incandescent fiery orange coat, bold onyx stripes, and cream bib.",
        movement_lore: "Fluid rhythmic prowl with lethal coiled muscle spring ready at every stride.",
    },
    MascotBiomeInfo {
        name: "wolf",
        category: "Wild & Safari",
        biome_name: "Boreal Taiga Wilderness",
        environment: "forest",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#90a4ae", "#546e7a", "#ffffff", "#37474f", "#ffd54f"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 255, 255), (55, 71, 79), (255, 213, 79)],
        description: "Timber wolf with silver-grey guard hairs, slate shadow saddle, and piercing amber stare.",
        movement_lore: "Enduring long-distance lope keeping rhythmic pace with the pack under the northern stars.",
    },
    MascotBiomeInfo {
        name: "bud-frogs",
        category: "Oceanic & Amphibian",
        biome_name: "Lily Pad Pond & Marshland",
        environment: "swamp",
        road: "mud",
        mountain: "hills",
        natural_palette: ["#4caf50", "#2e7d32", "#aed581", "#ffeb3b", "#ffd54f"],
        natural_rgb: [(76, 175, 80), (46, 125, 50), (174, 213, 129), (255, 235, 59), (255, 213, 79)],
        description: "Tree frogs perched on dewy floating lily pads in singing chorus.",
        movement_lore: "Elastic spring-loaded frog leaps punctuated by throat sac vocalization pulses.",
    },
    MascotBiomeInfo {
        name: "docker-whale",
        category: "Oceanic & Amphibian",
        biome_name: "High Seas Container Terminal",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#0284c7", "#0369a1", "#ffffff", "#e0f7fa", "#ffffff"],
        natural_rgb: [(2, 132, 199), (3, 105, 161), (255, 255, 255), (224, 247, 250), (255, 255, 255)],
        description: "Beloved container mascot whale ferrying reliable application containers across the ocean.",
        movement_lore: "Steady buoyant maritime cruise effortlessly carrying modular freight above the waves.",
    },
    MascotBiomeInfo {
        name: "dolphin",
        category: "Oceanic & Amphibian",
        biome_name: "Sunlit Coastal Waves",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#4fc3f7", "#0288d1", "#ffffff", "#b3e5fc", "#ffffff"],
        natural_rgb: [(79, 195, 247), (2, 136, 209), (255, 255, 255), (179, 229, 252), (255, 255, 255)],
        description: "Bottlenose dolphin with two-tone oceanic steel-blue skin and pristine white belly.",
        movement_lore: "Aerodynamic porpoising leap surfing the bow waves with acrobatic barrel rolls.",
    },
    MascotBiomeInfo {
        name: "duck",
        category: "Oceanic & Amphibian",
        biome_name: "Reed-Bordered Freshwater Pond",
        environment: "swamp",
        road: "mud",
        mountain: "hills",
        natural_palette: ["#2e7d32", "#795548", "#cfd8dc", "#ff9800", "#ffffff"],
        natural_rgb: [(46, 125, 50), (121, 85, 72), (207, 216, 220), (255, 152, 0), (255, 255, 255)],
        description: "Wild mallard drake with iridescent bottle-green head, chestnut breast, and bright yellow bill.",
        movement_lore: "Cheerful paddle-stroke glide across calm waters dipping head for aquatic greens.",
    },
    MascotBiomeInfo {
        name: "ebi_furai",
        category: "Oceanic & Amphibian",
        biome_name: "Tokyo Seaside Bistro",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ff9800", "#ffb74d", "#e53935", "#fff3e0", "#ffffff"],
        natural_rgb: [(255, 152, 0), (255, 183, 77), (229, 57, 53), (255, 243, 224), (255, 255, 255)],
        description: "Crispy golden-brown deep-fried tiger prawn with crunchy panko coating and ruby tail.",
        movement_lore: "Crispy sizzling hop straight out of the hot sesame oil tempura wok.",
    },
    MascotBiomeInfo {
        name: "happy-whale",
        category: "Oceanic & Amphibian",
        biome_name: "Tropical Lagoon Sanctuary",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#0284c7", "#0369a1", "#ffffff", "#e0f7fa", "#ffffff"],
        natural_rgb: [(2, 132, 199), (3, 105, 161), (255, 255, 255), (224, 247, 250), (255, 255, 255)],
        description: "Playful jovial humpback whale singing oceanic symphonies and breaching into rainbows.",
        movement_lore: "Joyful rolling breach with massive tail fluke slap showering ocean spray.",
    },
    MascotBiomeInfo {
        name: "jellyfish",
        category: "Oceanic & Amphibian",
        biome_name: "Bioluminescent Pelagic Drift",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#ba68c8", "#80d8ff", "#f3e5f5", "#ea80fc", "#ffffff"],
        natural_rgb: [(186, 104, 200), (128, 216, 255), (243, 229, 245), (234, 128, 252), (255, 255, 255)],
        description: "Translucent moon jellyfish pulsating with ethereal lilac and aqua bioluminescence.",
        movement_lore: "Hypnotic radial bell contraction drifting peacefully along deep marine currents.",
    },
    MascotBiomeInfo {
        name: "octopus",
        category: "Oceanic & Amphibian",
        biome_name: "Coral Reef Maze & Grotto",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#c2185b", "#7b1fa2", "#f8bbd0", "#ff4081", "#ffffff"],
        natural_rgb: [(194, 24, 91), (123, 31, 162), (248, 187, 208), (255, 64, 129), (255, 255, 255)],
        description: "Intelligent coral octopus shifting chromatic camouflage with dexterous suction arms.",
        movement_lore: "Sinusoidal multi-arm crawl exploring crevices and jet-propelling across coral walls.",
    },
    MascotBiomeInfo {
        name: "pufferfish",
        category: "Oceanic & Amphibian",
        biome_name: "Sunlit Barrier Reef Lagoon",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#ffb74d", "#6d4c41", "#fffde7", "#ffe082", "#ffffff"],
        natural_rgb: [(255, 183, 77), (109, 76, 65), (255, 253, 231), (255, 224, 130), (255, 255, 255)],
        description: "Spiny porcupinefish armed with protective sharp quills and swollen buoyant body.",
        movement_lore: "Careful pectoral fin flutter ready to inflate into a prickly defensive sphere.",
    },
    MascotBiomeInfo {
        name: "seahorse",
        category: "Oceanic & Amphibian",
        biome_name: "Undulating Seagrass Meadow",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#ff7043", "#ffd54f", "#ffab91", "#fff9c4", "#ffffff"],
        natural_rgb: [(255, 112, 67), (255, 213, 79), (255, 171, 145), (255, 249, 196), (255, 255, 255)],
        description: "Slender seahorse with crowned coronet and prehensile tail anchored to marine eelgrass.",
        movement_lore: "Upright vertical flutter powered by miniature transparent dorsal fin oscillations.",
    },
    MascotBiomeInfo {
        name: "squid",
        category: "Oceanic & Amphibian",
        biome_name: "Midnight Bathypelagic Abyss",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#00bcd4", "#3f51b5", "#e0f7fa", "#64ffda", "#ffffff"],
        natural_rgb: [(0, 188, 212), (63, 81, 181), (224, 247, 250), (100, 255, 218), (255, 255, 255)],
        description: "Giant squid roaming total oceanic darkness with huge dinner-plate eyes and hunting tentacles.",
        movement_lore: "Swift backward jet-propulsion leaving glowing clouds of bioluminescent pseudomorph ink.",
    },
    MascotBiomeInfo {
        name: "turtle",
        category: "Oceanic & Amphibian",
        biome_name: "Pacific Coral Atoll",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#558b2f", "#33691e", "#fff59d", "#689f38", "#ffffff"],
        natural_rgb: [(85, 139, 47), (51, 105, 30), (255, 245, 157), (104, 159, 56), (255, 255, 255)],
        description: "Ancient green sea turtle with mosaic olive scutes gliding through crystal warm waters.",
        movement_lore: "Majestic underwater wing-stroke flight migrating across vast ocean basins.",
    },
    MascotBiomeInfo {
        name: "walrus",
        category: "Oceanic & Amphibian",
        biome_name: "Arctic Pack Ice & Floes",
        environment: "arctic",
        road: "ice",
        mountain: "iceberg",
        natural_palette: ["#8d6e63", "#5d4037", "#fffde7", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (255, 253, 231), (255, 128, 171), (255, 255, 255)],
        description: "Pacific walrus with thick cinnamon blubber, bristly whisker pads, and long ivory tusks.",
        movement_lore: "Heaving front flipper haul hauling massive blubbery frame onto floating ice floes.",
    },
    MascotBiomeInfo {
        name: "whale",
        category: "Oceanic & Amphibian",
        biome_name: "Abyssal Pelagic Trench",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#0284c7", "#0369a1", "#ffffff", "#e0f7fa", "#ffffff"],
        natural_rgb: [(2, 132, 199), (3, 105, 161), (255, 255, 255), (224, 247, 250), (255, 255, 255)],
        description: "Colossal blue whale, the largest creature on Earth, singing low-frequency ocean pulses.",
        movement_lore: "Grand panoramic glide sweeping open baleen plates through dense swarms of krill.",
    },
    MascotBiomeInfo {
        name: "atat",
        category: "Fantasy & Sci-Fi",
        biome_name: "Frozen Hoth Ice Plains",
        environment: "arctic",
        road: "ice",
        mountain: "iceberg",
        natural_palette: ["#90a4ae", "#546e7a", "#cfd8dc", "#ff1744", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (207, 216, 220), (255, 23, 68), (255, 255, 255)],
        description: "All Terrain Armored Transport towering over snowdrifts with heavy chin laser turrets.",
        movement_lore: "Relentless mechanical stomping strides shaking the permafrost with every step.",
    },
    MascotBiomeInfo {
        name: "cthulhu-mini",
        category: "Fantasy & Sci-Fi",
        biome_name: "Sunken City of R'lyeh",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#388e3c", "#1b5e20", "#66bb6a", "#a5d6a7", "#ffea00"],
        natural_rgb: [(56, 142, 60), (27, 94, 32), (102, 187, 106), (165, 214, 167), (255, 234, 0)],
        description: "Chibi slumbering Great Old One with writhing face tentacles and eldritch leathery wings.",
        movement_lore: "Unfathomable non-Euclidean cosmic drift that bends the sanity of mortal onlookers.",
    },
    MascotBiomeInfo {
        name: "daemon",
        category: "Fantasy & Sci-Fi",
        biome_name: "Nether Flame Caldera",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#d32f2f", "#7f0000", "#ff5722", "#ffd600", "#ffd600"],
        natural_rgb: [(211, 47, 47), (127, 0, 0), (255, 87, 34), (255, 214, 0), (255, 214, 0)],
        description: "BSD Daemon Beastie bearing legendary pitchfork and cheerful mischievous demon horns.",
        movement_lore: "Nimble fiery hovering hop keeping POSIX network pipes running at maximum speed.",
    },
    MascotBiomeInfo {
        name: "dragon",
        category: "Fantasy & Sci-Fi",
        biome_name: "Volcanic Magma Chamber",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#e53935", "#ff7043", "#ffd600", "#ff3d00", "#ffd600"],
        natural_rgb: [(229, 57, 53), (255, 112, 67), (255, 214, 0), (255, 61, 0), (255, 214, 0)],
        description: "Ancient European fire drake with impenetrable ruby scales and smoldering belly furnaces.",
        movement_lore: "Tremendous rhythmic thoracic expansion breathing searing columns of incinerating flame.",
    },
    MascotBiomeInfo {
        name: "dragon-and-cow",
        category: "Fantasy & Sci-Fi",
        biome_name: "Smoldering Wyrm Lair",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#e53935", "#ff7043", "#ffd600", "#ff3d00", "#ffd600"],
        natural_rgb: [(229, 57, 53), (255, 112, 67), (255, 214, 0), (255, 61, 0), (255, 214, 0)],
        description: "Legendary hybrid encounter: an ancient fire dragon conferring with an unbothered bovine.",
        movement_lore: "Duet of rhythmic dragon flame breathing countered by calm cow chewing.",
    },
    MascotBiomeInfo {
        name: "ghost",
        category: "Fantasy & Sci-Fi",
        biome_name: "Haunted Victorian Graveyard",
        environment: "graveyard",
        road: "crypt",
        mountain: "gothic",
        natural_palette: ["#80cbc4", "#4db6ac", "#e0f2f1", "#b2dfdb", "#ffffff"],
        natural_rgb: [(128, 203, 196), (77, 182, 172), (224, 242, 241), (178, 223, 219), (255, 255, 255)],
        description: "Spooky translucent spectral apparition floating weightlessly above ancient mossy headstones.",
        movement_lore: "Ethereal vertical bobbing and wispy horizontal drift through solid obstacles.",
    },
    MascotBiomeInfo {
        name: "ghostbusters",
        category: "Fantasy & Sci-Fi",
        biome_name: "Paranormal New York Street",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffffff", "#e53935", "#b0bec5", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (229, 57, 53), (176, 190, 197), (255, 128, 171), (255, 255, 255)],
        description: "Iconic no-ghost caution insignia warning paranormal entities across Manhattan.",
        movement_lore: "Urgent siren pulse and particle-beam oscillation capturing rogue ectoplasm.",
    },
    MascotBiomeInfo {
        name: "glados",
        category: "Fantasy & Sci-Fi",
        biome_name: "Aperture Science Enrichment Center",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#eceff1", "#455a64", "#78909c", "#ff1744", "#ffd600"],
        natural_rgb: [(236, 239, 241), (69, 90, 100), (120, 144, 156), (255, 23, 68), (255, 214, 0)],
        description: "Aperture Science Genetic Lifeform and Disk Operating System observing human testing.",
        movement_lore: "Mechanical ceiling-mounted pendulum sway focusing optical amber sensor on the subject.",
    },
    MascotBiomeInfo {
        name: "mech-and-cow",
        category: "Fantasy & Sci-Fi",
        biome_name: "Orbital Mecha Hangar",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#78909c", "#455a64", "#e0e0e0", "#0284c7", "#ffffff"],
        natural_rgb: [(120, 144, 156), (69, 90, 100), (224, 224, 224), (2, 132, 199), (255, 255, 255)],
        description: "Heavy bipedal robotic combat exoskeleton piloted by a decorated bovine commander.",
        movement_lore: "Hydraulic reinforced servo steps with roaring attitude-thruster micro-burns.",
    },
    MascotBiomeInfo {
        name: "minotaur",
        category: "Fantasy & Sci-Fi",
        biome_name: "Underground Knossos Labyrinth",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#795548", "#f5f5f5", "#8d6e63", "#ffc107", "#ff1744"],
        natural_rgb: [(121, 85, 72), (245, 245, 245), (141, 110, 99), (255, 193, 7), (255, 23, 68)],
        description: "Fearsome bull-headed monster haunting labyrinth corridors with brass septum ring.",
        movement_lore: "Thundering aggressive patrol echoing through dark stone subterranean passages.",
    },
    MascotBiomeInfo {
        name: "mooghidjirah",
        category: "Fantasy & Sci-Fi",
        biome_name: "Cosmic Kaiju Nebula",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#388e3c", "#1b5e20", "#c8e6c9", "#00e5ff", "#ffd600"],
        natural_rgb: [(56, 142, 60), (27, 94, 32), (200, 230, 201), (0, 229, 255), (255, 214, 0)],
        description: "Three-headed Golden King Kaiju dragon raining lightning upon scorched planetary terrain.",
        movement_lore: "Triple-headed undulating serpentine thrash generating hurricane-force sonic gale winds.",
    },
    MascotBiomeInfo {
        name: "moojira",
        category: "Fantasy & Sci-Fi",
        biome_name: "Atomic Trench & Seafront",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#388e3c", "#1b5e20", "#c8e6c9", "#00e5ff", "#ffd600"],
        natural_rgb: [(56, 142, 60), (27, 94, 32), (200, 230, 201), (0, 229, 255), (255, 214, 0)],
        description: "King of the Monsters bovine irradiated by atomic energy charging cyan dorsal spine spikes.",
        movement_lore: "Colossal tectonic march radiating nuclear heat and devastating atomic breath.",
    },
    MascotBiomeInfo {
        name: "pterodactyl",
        category: "Fantasy & Sci-Fi",
        biome_name: "Prehistoric Caldera Crags",
        environment: "jurassic",
        road: "tracks",
        mountain: "volcano",
        natural_palette: ["#bf360c", "#ff7043", "#ffd54f", "#e65100", "#ffffff"],
        natural_rgb: [(191, 54, 12), (255, 112, 67), (255, 213, 79), (230, 81, 0), (255, 255, 255)],
        description: "Prehistoric pterosaur soaring on wide terracotta wing membranes over primeval cycads.",
        movement_lore: "Majestic updraft thermal banking glide with sharp predatory talons poised.",
    },
    MascotBiomeInfo {
        name: "sauron",
        category: "Fantasy & Sci-Fi",
        biome_name: "Mount Doom Barad-dûr Spire",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#424242", "#616161", "#9e9e9e", "#ff3d00", "#ffd600"],
        natural_rgb: [(66, 66, 66), (97, 97, 97), (158, 158, 158), (255, 61, 0), (255, 214, 0)],
        description: "The Dark Lord of Mordor manifesting as the Great Lidless Eye rimmed in incandescent fire.",
        movement_lore: "Ominous pulsing focal glare sweeping across volcanic ash plains seeking the One Ring.",
    },
    MascotBiomeInfo {
        name: "stegosaurus",
        category: "Fantasy & Sci-Fi",
        biome_name: "Jurassic Primeval Rainforest",
        environment: "jurassic",
        road: "tracks",
        mountain: "volcano",
        natural_palette: ["#558b2f", "#d84315", "#dcedc8", "#fff9c4", "#ffd54f"],
        natural_rgb: [(85, 139, 47), (216, 67, 21), (220, 237, 200), (255, 249, 196), (255, 213, 79)],
        description: "Armored herbivore with dual rows of vascular thermoregulatory back plates and tail spikes.",
        movement_lore: "Heavy quadrupedal stride swinging spiked thagomizer in defensive sweeping arcs.",
    },
    MascotBiomeInfo {
        name: "unipony",
        category: "Fantasy & Sci-Fi",
        biome_name: "Enchanted Fairyland Glade",
        environment: "pasture",
        road: "cobblestone",
        mountain: "castle",
        natural_palette: ["#ffffff", "#e1bee7", "#ffd54f", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (225, 190, 231), (255, 213, 79), (255, 128, 171), (255, 255, 255)],
        description: "Magical winged alicorn with spiral pearl horn, pastel mane, and sparkling stardust hooves.",
        movement_lore: "Weightless graceful canter sprinkling glitter trails and sparkling rainbow magic.",
    },
    MascotBiomeInfo {
        name: "vader",
        category: "Fantasy & Sci-Fi",
        biome_name: "Death Star Overlook & Trench",
        environment: "space",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#424242", "#e0e0e0", "#616161", "#303030", "#ff1744"],
        natural_rgb: [(66, 66, 66), (224, 224, 224), (97, 97, 97), (48, 48, 48), (255, 23, 68)],
        description: "Sith Lord Darth Vader in obsidian battle armor with crimson lightsaber glow and respirator.",
        movement_lore: "Slow intimidating march backed by the mechanical hiss of synthetic respiratory chambers.",
    },
    MascotBiomeInfo {
        name: "wizard",
        category: "Fantasy & Sci-Fi",
        biome_name: "Arcane Spire Observatory",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#3949ab", "#eceff1", "#fdd835", "#8d6e63", "#00e5ff"],
        natural_rgb: [(57, 73, 171), (236, 239, 241), (253, 216, 53), (141, 110, 99), (0, 229, 255)],
        description: "Ancient archmage clad in midnight indigo robe with silver beard channeling raw arcane ether.",
        movement_lore: "Harmonic arcane levitation charging runic glyphs and mystic lightning pulses.",
    },
    MascotBiomeInfo {
        name: "yoda",
        category: "Fantasy & Sci-Fi",
        biome_name: "Dagobah Foggy Bayou",
        environment: "swamp",
        road: "mud",
        mountain: "hills",
        natural_palette: ["#7cb342", "#558b2f", "#d7ccc8", "#ffb6c1", "#ffd54f"],
        natural_rgb: [(124, 179, 66), (85, 139, 47), (215, 204, 200), (255, 182, 193), (255, 213, 79)],
        description: "Grand Master of the Jedi Order meditating in mossy swamp hut among twisted cypress roots.",
        movement_lore: "Slow walking-cane shuffle interchanged with weightless Force-assisted meditation floats.",
    },
    MascotBiomeInfo {
        name: "beavis.zen",
        category: "Pop Culture & Fun",
        biome_name: "Suburban Living Room Couch",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffd600", "#1e88e5", "#ffcc80", "#78909c", "#ffffff"],
        natural_rgb: [(255, 214, 0), (30, 136, 229), (255, 204, 128), (120, 144, 156), (255, 255, 255)],
        description: "Blond 90s rocker in iconic Metallica blue shirt rocking out to heavy metal guitar solos.",
        movement_lore: "Spastic energetic headbanging frenzy punctuated by comedic hand gestures.",
    },
    MascotBiomeInfo {
        name: "bill-the-cat",
        category: "Pop Culture & Fun",
        biome_name: "Back-Alley Dumpster Sanctuary",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#f57f17", "#d32f2f", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(245, 127, 23), (211, 47, 47), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Beloved scruffy mangy ginger cat with mismatched googly eyes and iconic tongue hang.",
        movement_lore: "Unpredictable twitchy hitch-step with comical eye-rolling and gasping expressions.",
    },
    MascotBiomeInfo {
        name: "charizardvice",
        category: "Pop Culture & Fun",
        biome_name: "Volcanic Battle Stadium",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#ef6c00", "#26a69a", "#ffe082", "#ff3d00", "#40c4ff"],
        natural_rgb: [(239, 108, 0), (38, 166, 154), (255, 224, 130), (255, 61, 0), (64, 196, 255)],
        description: "Iconic flame-tailed fire dragon roaring with blazing tail ember and turquoise wing membranes.",
        movement_lore: "Intense battle-ready breath pulse unleashing soaring plumes of blue and gold fire.",
    },
    MascotBiomeInfo {
        name: "fat-banana",
        category: "Pop Culture & Fun",
        biome_name: "Tropical Fruit Canopy",
        environment: "pasture",
        road: "dirt",
        mountain: "garden",
        natural_palette: ["#fdd835", "#8d6e63", "#fff9c4", "#c0ca33", "#ffffff"],
        natural_rgb: [(253, 216, 53), (141, 110, 99), (255, 249, 196), (192, 202, 51), (255, 255, 255)],
        description: "Plump sun-ripened Cavendish banana with sweet sugar freckles and fresh green stem.",
        movement_lore: "Joyful rhythmic dancing wiggle swaying to irresistible calypso island rhythms.",
    },
    MascotBiomeInfo {
        name: "flaming-sheep",
        category: "Pop Culture & Fun",
        biome_name: "Volcanic Ash Grazing Hill",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#ff7043", "#d84315", "#ffe082", "#ff3d00", "#ffd600"],
        natural_rgb: [(255, 112, 67), (216, 67, 21), (255, 224, 130), (255, 61, 0), (255, 214, 0)],
        description: "Combustible sheep burning with glorious incandescent flame wool yet grazing peacefully.",
        movement_lore: "Blazing fast galloping sprint leaving a glowing trail of burning embers.",
    },
    MascotBiomeInfo {
        name: "golden-eagle",
        category: "Pop Culture & Fun",
        biome_name: "High Mountain Thermal Updraft",
        environment: "pasture",
        road: "dirt",
        mountain: "peaks",
        natural_palette: ["#8d6e63", "#ffc107", "#f5f5f5", "#ffb300", "#ffe082"],
        natural_rgb: [(141, 110, 99), (255, 193, 7), (245, 245, 245), (255, 179, 0), (255, 224, 130)],
        description: "Majestic raptor soaring with dark brown wing plumage, golden crown, and keen yellow talons.",
        movement_lore: "Effortless wide-span soaring glide circling alpine crests with laser-focus vision.",
    },
    MascotBiomeInfo {
        name: "hellokitty",
        category: "Pop Culture & Fun",
        biome_name: "Pastel Shibuya Plaza",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffffff", "#e53935", "#fff9c4", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (229, 57, 53), (255, 249, 196), (255, 128, 171), (255, 255, 255)],
        description: "Universal icon of kindness and friendship with pure white fur and trademark crimson bow.",
        movement_lore: "Delicate sweet rhythmic parade step spreading cheer across the bustling boulevard.",
    },
    MascotBiomeInfo {
        name: "hypno",
        category: "Pop Culture & Fun",
        biome_name: "Kaleidoscopic Dimension",
        environment: "cyber",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#455a64", "#00e5ff", "#ffffff", "#7c4dff", "#ffffff"],
        natural_rgb: [(69, 90, 100), (0, 229, 255), (255, 255, 255), (124, 77, 255), (255, 255, 255)],
        description: "Mesmerizing mind-altering concentric swirl altering perception through psychic resonance.",
        movement_lore: "Continuous rotational vortex spiral mesmerizing the viewer into deep relaxation.",
    },
    MascotBiomeInfo {
        name: "kiss",
        category: "Pop Culture & Fun",
        biome_name: "Rock Arena World Tour Stage",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#e91e63", "#ff4081", "#fce4ec", "#d81b60", "#ffffff"],
        natural_rgb: [(233, 30, 99), (255, 64, 129), (252, 228, 236), (216, 27, 96), (255, 255, 255)],
        description: "Glam rock demon icon with stark white face paint, star eye design, and tongue extension.",
        movement_lore: "Theatrical arena stomping stride with fire-breathing bass guitar riffs.",
    },
    MascotBiomeInfo {
        name: "mona-lisa",
        category: "Pop Culture & Fun",
        biome_name: "Renaissance Louvre Gallery",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#8d6e63", "#5d4037", "#f5f5f5", "#a1887f", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (245, 245, 245), (161, 136, 127), (255, 255, 255)],
        description: "Da Vinci's immortal Renaissance masterpiece with subtle sfumato shading and enigmatic smile.",
        movement_lore: "Poised immortal stillness with soft thoracic breathing that follows you across the room.",
    },
    MascotBiomeInfo {
        name: "nyan",
        category: "Pop Culture & Fun",
        biome_name: "Cosmic Rainbow Void",
        environment: "space",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#ffe082", "#f48fb1", "#e91e63", "#b0bec5", "#ffffff"],
        natural_rgb: [(255, 224, 130), (244, 143, 177), (233, 30, 99), (176, 190, 197), (255, 255, 255)],
        description: "Interstellar pop-tart feline rocketing through cosmos leaving an eternal rainbow ribbon.",
        movement_lore: "Supersonic warp dash looping through constellations to upbeat chiptune cadence.",
    },
    MascotBiomeInfo {
        name: "radioactive-kitty",
        category: "Pop Culture & Fun",
        biome_name: "Chernobyl Glowing Reactor Core",
        environment: "cyber",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#76ff03", "#00e676", "#b2ff59", "#ff80ab", "#ffff00"],
        natural_rgb: [(118, 255, 3), (0, 230, 118), (178, 255, 89), (255, 128, 171), (255, 255, 0)],
        description: "Mutant feline glowing with radioactive chartreuse luminescence and trefoil hazard stripes.",
        movement_lore: "Twitchy glitch-step flickering between reality states with electric ion arcs.",
    },
    MascotBiomeInfo {
        name: "ren",
        category: "Pop Culture & Fun",
        biome_name: "Hollywood Boulevard Alley",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffb74d", "#ff80ab", "#42a5f5", "#ffffff", "#ffffff"],
        natural_rgb: [(255, 183, 77), (255, 128, 171), (66, 165, 245), (255, 255, 255), (255, 255, 255)],
        description: "High-strung asthmatic Chihuahua trembling with nervous energy and bloodshot bulging eyes.",
        movement_lore: "Frantic trembling vibration and sudden explosive exasperated pacing.",
    },
    MascotBiomeInfo {
        name: "snoopy",
        category: "Pop Culture & Fun",
        biome_name: "Suburban Red Doghouse Lawn",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffffff", "#263238", "#e53935", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (38, 50, 56), (229, 57, 53), (255, 128, 171), (255, 255, 255)],
        description: "Beloved beagle with jet-black ears, cherry-red collar, and boundless imaginative dreams.",
        movement_lore: "Exuberant high-stepping happy dance celebrating friendships and fresh supper bowls.",
    },
    MascotBiomeInfo {
        name: "stimpy",
        category: "Pop Culture & Fun",
        biome_name: "Cartoon Kitchen Floor",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#e65100", "#1e88e5", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(230, 81, 0), (30, 136, 229), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Good-natured rotund Manx cat with warm orange coat and prominent bulbous blue nose.",
        movement_lore: "Joyful bouncy waddle humming cheerful nonsensical tunes with blissful optimism.",
    },
    MascotBiomeInfo {
        name: "tux",
        category: "Pop Culture & Fun",
        biome_name: "Antarctic Pack Ice & Open Leads",
        environment: "arctic",
        road: "ice",
        mountain: "iceberg",
        natural_palette: ["#455a64", "#ffffff", "#ffa000", "#ffb300", "#ffffff"],
        natural_rgb: [(69, 90, 100), (255, 255, 255), (255, 160, 0), (255, 179, 0), (255, 255, 255)],
        description: "Official Linux penguin mascot with glossy black coat, snow-white belly, and golden webbed feet.",
        movement_lore: "Endearing charismatic waddle sliding on belly across slick polar ice fields.",
    },
    MascotBiomeInfo {
        name: "vulpix",
        category: "Pop Culture & Fun",
        biome_name: "Autumn Maple Forest Clearing",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#d35400", "#ff9800", "#fff3e0", "#e65100", "#ffffff"],
        natural_rgb: [(211, 84, 0), (255, 152, 0), (255, 243, 224), (230, 81, 0), (255, 255, 255)],
        description: "Fox Pokemon with six gorgeous curled amber tails and warm auburn winter coat.",
        movement_lore: "Nimble graceful trot fanning out flame-warmed tails in mesmerizing patterns.",
    },
    MascotBiomeInfo {
        name: "apt",
        category: "Abstract & Quirky",
        biome_name: "Debian Mainframe Data Center",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#d32f2f", "#ffffff", "#e57373", "#ff80ab", "#ffffff"],
        natural_rgb: [(211, 47, 47), (255, 255, 255), (229, 115, 115), (255, 128, 171), (255, 255, 255)],
        description: "Debian Advanced Package Tool swirl representing stable Free Software distribution.",
        movement_lore: "Reliable algorithmic packet verification processing byte streams with clockwork precision.",
    },
    MascotBiomeInfo {
        name: "bees",
        category: "Abstract & Quirky",
        biome_name: "Sunlit Meadow Hive & Wildflowers",
        environment: "hive",
        road: "dirt",
        mountain: "garden",
        natural_palette: ["#ffb300", "#263238", "#e0f7fa", "#fff59d", "#ffffff"],
        natural_rgb: [(255, 179, 0), (38, 50, 56), (224, 247, 250), (255, 245, 157), (255, 255, 255)],
        description: "Busy honeybee colony with golden-amber and velvet black warning stripes collecting nectar.",
        movement_lore: "Fast figure-eight waggle dance communicating sweet flower locations to the hive.",
    },
    MascotBiomeInfo {
        name: "claw-arm",
        category: "Abstract & Quirky",
        biome_name: "Automated Industrial Foundry",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#78909c", "#455a64", "#ffc107", "#ff1744", "#ffffff"],
        natural_rgb: [(120, 144, 156), (69, 90, 100), (255, 193, 7), (255, 23, 68), (255, 255, 255)],
        description: "Precision multi-axis cybernetic manipulator arm with chrome plating and warning strobe.",
        movement_lore: "Articulated multi-axis robotic extension calibrating micrometric grip tension.",
    },
    MascotBiomeInfo {
        name: "cower",
        category: "Abstract & Quirky",
        biome_name: "Highland Meadow Storm",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#4e342e", "#d7ccc8", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (78, 52, 46), (215, 204, 200), (255, 128, 171), (255, 255, 255)],
        description: "Gentle young jersey calf trembling slightly under thunderclaps in the mountain meadow.",
        movement_lore: "Subtle cautious stepping seeking shelter beside friendly herd elders.",
    },
    MascotBiomeInfo {
        name: "cowfee",
        category: "Abstract & Quirky",
        biome_name: "Artisan Italian Espresso Bar",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#8d6e63", "#5d4037", "#fff8e1", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (255, 248, 225), (255, 128, 171), (255, 255, 255)],
        description: "Barista cow serving rich double-shot espresso layered with steamed microfoam crema.",
        movement_lore: "Smooth rhythmic barista routine steaming milk and drawing delicate rosetta latte art.",
    },
    MascotBiomeInfo {
        name: "eyes",
        category: "Abstract & Quirky",
        biome_name: "Astral Observation Zenith",
        environment: "space",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#9c27b0", "#00e5ff", "#ffffff", "#e1bee7", "#ffffff"],
        natural_rgb: [(156, 39, 176), (0, 229, 255), (255, 255, 255), (225, 190, 231), (255, 255, 255)],
        description: "Omniscient cosmic gaze witnessing the dance of distant galaxies across space-time.",
        movement_lore: "Synchronized panoramic eye dilation tracking stellar movements across the void.",
    },
    MascotBiomeInfo {
        name: "fence",
        category: "Abstract & Quirky",
        biome_name: "Ranch Boundary Paddock",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#5d4037", "#d7ccc8", "#a1887f", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (215, 204, 200), (161, 136, 127), (255, 255, 255)],
        description: "Sturdy split-rail timber farm fence standing guard over open grazing ranges.",
        movement_lore: "Subtle harmonic swaying resisting seasonal prairie prairie winds.",
    },
    MascotBiomeInfo {
        name: "hiya",
        category: "Abstract & Quirky",
        biome_name: "Sunny Village Green",
        environment: "pasture",
        road: "dirt",
        mountain: "garden",
        natural_palette: ["#ffb74d", "#ff7043", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 183, 77), (255, 112, 67), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Infectiously happy friendly face radiating warm welcoming greetings to all travelers.",
        movement_lore: "Enthusiastic rhythmic hand-waving and radiant smiling greeting pulses.",
    },
    MascotBiomeInfo {
        name: "jesus",
        category: "Abstract & Quirky",
        biome_name: "Sacred Mount Olive Overlook",
        environment: "throne",
        road: "cobblestone",
        mountain: "peaks",
        natural_palette: ["#f5f5f5", "#8d6e63", "#e0e0e0", "#ffb6c1", "#ffd54f"],
        natural_rgb: [(245, 245, 245), (141, 110, 99), (224, 224, 224), (255, 182, 193), (255, 213, 79)],
        description: "Sacred shepherd icon clad in linen robe and scarlet mantle bearing divine golden halo.",
        movement_lore: "Peaceful blessing glide walking across waters and tranquil mountain paths.",
    },
    MascotBiomeInfo {
        name: "kosh",
        category: "Abstract & Quirky",
        biome_name: "Vorlon Sanctuary Chamber",
        environment: "space",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#80cbc4", "#00695c", "#e0f2f1", "#4db6ac", "#ffffff"],
        natural_rgb: [(128, 203, 196), (0, 105, 92), (224, 242, 241), (77, 182, 172), (255, 255, 255)],
        description: "Ambassador Kosh of the mysterious First Ones enclosed in bio-armored encounter suit.",
        movement_lore: "Majestic ponderous glide with bioluminescent suit petals pulsing cryptic messages.",
    },
    MascotBiomeInfo {
        name: "mutilated",
        category: "Abstract & Quirky",
        biome_name: "Nocturnal Crop Circle Field",
        environment: "space",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#90a4ae", "#00e5ff", "#e0e0e0", "#ff1744", "#ffffff"],
        natural_rgb: [(144, 164, 174), (0, 229, 255), (224, 224, 224), (255, 23, 68), (255, 255, 255)],
        description: "Mysterious nighttime flying saucer encounter beaming pasture animals into the night sky.",
        movement_lore: "Vertical tractor-beam anti-gravity levitation with rotating saucer light rings.",
    },
    MascotBiomeInfo {
        name: "queen",
        category: "Abstract & Quirky",
        biome_name: "Imperial Throne Room Cathedral",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#e91e63", "#ffffff", "#fce4ec", "#c2185b", "#ffffff"],
        natural_rgb: [(233, 30, 99), (255, 255, 255), (252, 228, 236), (194, 24, 91), (255, 255, 255)],
        description: "Reigning sovereign in ermine-trimmed royal crimson mantle and jewel-encrusted golden crown.",
        movement_lore: "Stately regal court procession with dignified royal waves and imperial poise.",
    },
    MascotBiomeInfo {
        name: "skeleton",
        category: "Abstract & Quirky",
        biome_name: "Ancient Crypt Mausoleum",
        environment: "graveyard",
        road: "crypt",
        mountain: "gothic",
        natural_palette: ["#f5f5f5", "#78909c", "#ffffff", "#b0bec5", "#ffffff"],
        natural_rgb: [(245, 245, 245), (120, 144, 156), (255, 255, 255), (176, 190, 197), (255, 255, 255)],
        description: "Clean articulated calcium skeleton dancing merrily under the midnight harvest moon.",
        movement_lore: "Clicky rhythmic bone-clattering tap dance echoing through ancient catacombs.",
    },
    MascotBiomeInfo {
        name: "small",
        category: "Abstract & Quirky",
        biome_name: "Spring Nursery Pasture",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#263238", "#e0e0e0", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (38, 50, 56), (224, 224, 224), (255, 128, 171), (255, 255, 255)],
        description: "Adorable miniature newborn calf taking its very first steps in the warming morning sun.",
        movement_lore: "Wobbly endearing baby steps discovering balance on four tiny hooves.",
    },
    MascotBiomeInfo {
        name: "supermilker",
        category: "Abstract & Quirky",
        biome_name: "Automated Cyber Dairy Parlor",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#78909c", "#0284c7", "#e1f5fe", "#ff80ab", "#ffffff"],
        natural_rgb: [(120, 144, 156), (2, 132, 199), (225, 245, 254), (255, 128, 171), (255, 255, 255)],
        description: "High-efficiency automated robotic milker optimizing dairy flow with gentle vacuum pulses.",
        movement_lore: "Precision cyclic pumping rhythm synchronizing with telemetry sensors.",
    },
    MascotBiomeInfo {
        name: "surgery",
        category: "Abstract & Quirky",
        biome_name: "Sterile Operating Theater",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#80deea", "#00838f", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(128, 222, 234), (0, 131, 143), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Advanced surgical suite with sterile titanium instruments and vital monitoring telemetry.",
        movement_lore: "Methodical precision micro-adjustments maintaining steady robotic surgical accuracy.",
    },
    MascotBiomeInfo {
        name: "three-eyes",
        category: "Abstract & Quirky",
        biome_name: "Toxic Wasteland Marsh",
        environment: "swamp",
        road: "mud",
        mountain: "crater",
        natural_palette: ["#76ff03", "#00e676", "#b2ff59", "#ff80ab", "#00e5ff"],
        natural_rgb: [(118, 255, 3), (0, 230, 118), (178, 255, 89), (255, 128, 171), (0, 229, 255)],
        description: "Mutated triclops bovine thriving in glowing mossy bogs with an extra supernatural eye.",
        movement_lore: "Careful three-eyed stereoscopic scanning surveying 270 degrees of surrounding marsh.",
    },
    MascotBiomeInfo {
        name: "viper",
        category: "Abstract & Quirky",
        biome_name: "Mangrove Serpent Mire",
        environment: "swamp",
        road: "mud",
        mountain: "plateau",
        natural_palette: ["#4caf50", "#2e7d32", "#c8e6c9", "#ff1744", "#ffd54f"],
        natural_rgb: [(76, 175, 80), (46, 125, 50), (200, 230, 201), (255, 23, 68), (255, 213, 79)],
        description: "Venomous pit viper with vivid diamond scales and coiled spring striking speed.",
        movement_lore: "Silent undulating lateral serpentine glide tasting the warm air with forked tongue.",
    },
    MascotBiomeInfo {
        name: "moose",
        category: "Wild & Safari",
        biome_name: "Boreal Pine Forest & Willow Thicket",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#5d4037", "#d7ccc8", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (93, 64, 55), (215, 204, 200), (255, 128, 171), (255, 255, 255)],
        description: "Majestic northern moose commanding the boreal forest with towering palmate antlers.",
        movement_lore: "Heavy deliberate forest gait pacing smoothly through underbrush.",
    },
    MascotBiomeInfo {
        name: "lamb",
        category: "Farm & Domestic",
        biome_name: "Sunny Hillside Meadow",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#f5f5f5", "#e0e0e0", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (245, 245, 245), (224, 224, 224), (255, 128, 171), (255, 255, 255)],
        description: "Gentle young fleece lamb grazing peacefully on sweet clover.",
        movement_lore: "Playful spring bounding and soft meadow prancing.",
    },
    MascotBiomeInfo {
        name: "lamb2",
        category: "Farm & Domestic",
        biome_name: "Rolling Pasture Warrens",
        environment: "pasture",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#ffffff", "#f5f5f5", "#e0e0e0", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (245, 245, 245), (224, 224, 224), (255, 128, 171), (255, 255, 255)],
        description: "Fluffy domestic lamb resting warmly beside the sheepfold.",
        movement_lore: "Gentle synchronized flank breathing and peaceful chewing.",
    },
    MascotBiomeInfo {
        name: "smiling-octopus",
        category: "Oceanic & Amphibian",
        biome_name: "Bioluminescent Coral Shelf",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#c2185b", "#7b1fa2", "#f8bbd0", "#ff4081", "#ffffff"],
        natural_rgb: [(194, 24, 91), (123, 31, 162), (248, 187, 208), (255, 64, 129), (255, 255, 255)],
        description: "Cheerful cheerful cephalopod rippling through warm crystal currents.",
        movement_lore: "Graceful undulating tentacle propulsion with rhythmic siphon jets.",
    },
    MascotBiomeInfo {
        name: "seahorse-big",
        category: "Oceanic & Amphibian",
        biome_name: "Sunlit Kelp Forest & Coral Spire",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#ff7043", "#ffd54f", "#ffab91", "#fff9c4", "#ffffff"],
        natural_rgb: [(255, 112, 67), (255, 213, 79), (255, 171, 145), (255, 249, 196), (255, 255, 255)],
        description: "Stately giant seahorse anchored gracefully to swaying golden kelp fronds.",
        movement_lore: "Upright buoyant hover with fluttering dorsal fin propulsion.",
    },
    MascotBiomeInfo {
        name: "lobster",
        category: "Oceanic & Amphibian",
        biome_name: "Rocky Atlantic Shoreline Trench",
        environment: "ocean",
        road: "seabed",
        mountain: "seamount",
        natural_palette: ["#c62828", "#8e0000", "#ff6f00", "#ff8a65", "#ffffff"],
        natural_rgb: [(198, 40, 40), (142, 0, 0), (255, 111, 0), (255, 138, 101), (255, 255, 255)],
        description: "Formidable clawed decapod crustacean patrolling deep tidal crevices.",
        movement_lore: "Buoyant benthic crawling glide and tail-flick swimming across tidal trenches.",
    },
    MascotBiomeInfo {
        name: "owl",
        category: "Wild & Safari",
        biome_name: "Old-Growth Midnight Woodland",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#d7ccc8", "#8d6e63", "#ffffff", "#ffa000", "#ffd54f"],
        natural_rgb: [(215, 204, 200), (141, 110, 99), (255, 255, 255), (255, 160, 0), (255, 213, 79)],
        description: "Wise nocturnal barn owl perched vigilantly upon an ancient oak branch.",
        movement_lore: "Subtle breathing pulse and stereoscopic silent head rotations.",
    },
    MascotBiomeInfo {
        name: "tweety-bird",
        category: "Pop Culture & Fun",
        biome_name: "Sunny Parlor Cage & Porch",
        environment: "city",
        road: "sidewalk",
        mountain: "skyline",
        natural_palette: ["#ffee58", "#ff9800", "#4fc3f7", "#fff59d", "#ffffff"],
        natural_rgb: [(255, 238, 88), (255, 152, 0), (79, 195, 247), (255, 245, 157), (255, 255, 255)],
        description: "Sprightly yellow canary with sweet melodic trills and curious wide eyes.",
        movement_lore: "Brisk fluttering wing hops and joyful harmonic head tilts.",
    },
    MascotBiomeInfo {
        name: "squirrel",
        category: "Wild & Safari",
        biome_name: "Autumn Oak Canopy & Forest Floor",
        environment: "forest",
        road: "dirt",
        mountain: "hills",
        natural_palette: ["#8d6e63", "#a1887f", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(141, 110, 99), (161, 136, 127), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Nimble bushy-tailed woodland squirrel foraging for acorn caches.",
        movement_lore: "Rapid skittering scurries with rhythmic tail balancing twitches.",
    },
    MascotBiomeInfo {
        name: "weeping-angel",
        category: "Fantasy & Sci-Fi",
        biome_name: "Gothic Graveyard Mausoleum",
        environment: "graveyard",
        road: "crypt",
        mountain: "gothic",
        natural_palette: ["#90a4ae", "#546e7a", "#cfd8dc", "#b0bec5", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (207, 216, 220), (176, 190, 197), (255, 255, 255)],
        description: "Quantum-locked stone angel statue watching silently in the gloom.",
        movement_lore: "Stark static presence with subtle quantum dimensional phase shifts.",
    },
    MascotBiomeInfo {
        name: "satanic",
        category: "Fantasy & Sci-Fi",
        biome_name: "Abyssal Brimstone Caldera",
        environment: "inferno",
        road: "magma",
        mountain: "volcano",
        natural_palette: ["#424242", "#c62828", "#757575", "#ff1744", "#ffd600"],
        natural_rgb: [(66, 66, 66), (198, 40, 40), (117, 117, 117), (255, 23, 68), (255, 214, 0)],
        description: "Demonic horned entity wreathed in smoldering nether ash.",
        movement_lore: "Rhythmic infernal breathing with flickering ember pulsations.",
    },
    MascotBiomeInfo {
        name: "personality-sphere",
        category: "Fantasy & Sci-Fi",
        biome_name: "Aperture Enrichment Facility",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#ffffff", "#37474f", "#78909c", "#cfd8dc", "#00e5ff"],
        natural_rgb: [(255, 255, 255), (55, 71, 79), (120, 144, 156), (207, 216, 220), (0, 229, 255)],
        description: "Autonomous robotic AI sphere suspended from robotic ceiling rails.",
        movement_lore: "Gyroscopic stabilization drift with expressive optical aperture twitches.",
    },
    MascotBiomeInfo {
        name: "snoopyhouse",
        category: "Pop Culture & Fun",
        biome_name: "Iconic Suburban Backyard",
        environment: "pasture",
        road: "sidewalk",
        mountain: "hills",
        natural_palette: ["#ffffff", "#263238", "#e53935", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (38, 50, 56), (229, 57, 53), (255, 128, 171), (255, 255, 255)],
        description: "World War I flying ace beagle perched boldly atop his red doghouse.",
        movement_lore: "Contemplative rhythmic rooftop breathing under starry skies.",
    },
    MascotBiomeInfo {
        name: "snoopysleep",
        category: "Pop Culture & Fun",
        biome_name: "Peaceful Doghouse Rooftop",
        environment: "pasture",
        road: "sidewalk",
        mountain: "hills",
        natural_palette: ["#ffffff", "#263238", "#e53935", "#ff80ab", "#ffffff"],
        natural_rgb: [(255, 255, 255), (38, 50, 56), (229, 57, 53), (255, 128, 171), (255, 255, 255)],
        description: "Contented beagle slumbering deeply atop his red pitched roof.",
        movement_lore: "Serene rhythmic sleep respiration dreaming of victory.",
    },
    MascotBiomeInfo {
        name: "spidercow",
        category: "Pop Culture & Fun",
        biome_name: "Metropolitan Rooftop Scaffolding",
        environment: "city",
        road: "roof",
        mountain: "skyline",
        natural_palette: ["#455a64", "#d32f2f", "#ffffff", "#1976d2", "#ffffff"],
        natural_rgb: [(69, 90, 100), (211, 47, 47), (255, 255, 255), (25, 118, 210), (255, 255, 255)],
        description: "Superhero arachnid bovine swinging acrobatically between skyscrapers.",
        movement_lore: "Agile multi-legged wall crawl with web-spinning stability.",
    },
    MascotBiomeInfo {
        name: "tortoise",
        category: "Wild & Safari",
        biome_name: "Galapagos Volcanic Island Slope",
        environment: "savanna",
        road: "dirt",
        mountain: "plateau",
        natural_palette: ["#558b2f", "#33691e", "#fff59d", "#689f38", "#ffffff"],
        natural_rgb: [(85, 139, 47), (51, 105, 30), (255, 245, 157), (104, 159, 56), (255, 255, 255)],
        description: "Ancient giant tortoise with venerable domed carapacial scutes.",
        movement_lore: "Slow unhurried ancient steps with patient breathing endurance.",
    },
    MascotBiomeInfo {
        name: "periodic-table",
        category: "Abstract & Quirky",
        biome_name: "Quantum Chemistry Laboratory",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#00bcd4", "#9c27b0", "#ff9800", "#4caf50", "#ffffff"],
        natural_rgb: [(0, 188, 212), (156, 39, 176), (255, 152, 0), (76, 175, 80), (255, 255, 255)],
        description: "Structured matrix of the chemical elements organized by atomic number.",
        movement_lore: "Harmonic orbital electron shell resonance and spectral emission pulses.",
    },
    MascotBiomeInfo {
        name: "world",
        category: "Abstract & Quirky",
        biome_name: "Low Earth Orbital Vista",
        environment: "space",
        road: "grid",
        mountain: "crater",
        natural_palette: ["#0288d1", "#4caf50", "#ffffff", "#81c784", "#ffffff"],
        natural_rgb: [(2, 136, 209), (76, 175, 80), (255, 255, 255), (129, 199, 132), (255, 255, 255)],
        description: "The blue planet Earth floating peacefully within the cosmic vacuum.",
        movement_lore: "Majestic orbital rotation surveying glowing continents and sapphire oceans.",
    },
    MascotBiomeInfo {
        name: "king",
        category: "Abstract & Quirky",
        biome_name: "Grand Royal Throne Room",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#ffd54f", "#bcaaa4", "#ffffff", "#d7ccc8", "#ffffff"],
        natural_rgb: [(255, 213, 79), (188, 170, 164), (255, 255, 255), (215, 204, 200), (255, 255, 255)],
        description: "Supreme monarch chess piece commanding the sixty-four squares.",
        movement_lore: "Stately deliberate one-square royal stride asserting sovereign authority.",
    },
    MascotBiomeInfo {
        name: "knight",
        category: "Abstract & Quirky",
        biome_name: "Medieval Tournament Arena",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#90a4ae", "#546e7a", "#ffffff", "#cfd8dc", "#ffffff"],
        natural_rgb: [(144, 164, 174), (84, 110, 122), (255, 255, 255), (207, 216, 220), (255, 255, 255)],
        description: "Cavalry equestrian chess piece executing cunning L-shaped maneuvers.",
        movement_lore: "Bold galloping hop leaping cleanly across opposing ranks.",
    },
    MascotBiomeInfo {
        name: "rook",
        category: "Abstract & Quirky",
        biome_name: "Castle Battlement Fortress",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#78909c", "#455a64", "#eceff1", "#b0bec5", "#ffffff"],
        natural_rgb: [(120, 144, 156), (69, 90, 100), (236, 239, 241), (176, 190, 197), (255, 255, 255)],
        description: "Heavy stone siege tower dominating long orthogonal ranks and files.",
        movement_lore: "Unstoppable linear advance sweeping unobstructed along ranks.",
    },
    MascotBiomeInfo {
        name: "pawn",
        category: "Abstract & Quirky",
        biome_name: "Frontline Checkered Vanguard",
        environment: "throne",
        road: "checkerboard",
        mountain: "castle",
        natural_palette: ["#d7ccc8", "#8d6e63", "#ffffff", "#bcaaa4", "#ffffff"],
        natural_rgb: [(215, 204, 200), (141, 110, 99), (255, 255, 255), (188, 170, 164), (255, 255, 255)],
        description: "Courageous infantry chess footman marching forward toward promotion.",
        movement_lore: "Steadfast forward march with brave diagonal capturing stance.",
    },
    MascotBiomeInfo {
        name: "tux-big",
        category: "Pop Culture & Fun",
        biome_name: "Antarctic Ice Shelf & Open Source Frontier",
        environment: "arctic",
        road: "ice",
        mountain: "iceberg",
        natural_palette: ["#455a64", "#ffffff", "#ffa000", "#ffb300", "#ffffff"],
        natural_rgb: [(69, 90, 100), (255, 255, 255), (255, 160, 0), (255, 179, 0), (255, 255, 255)],
        description: "Giant majestic Linux mascot penguin celebrating kernel freedom.",
        movement_lore: "Hearty waddling snow gait with rhythmic flipper counter-swings.",
    },
    MascotBiomeInfo {
        name: "lollerskates",
        category: "Pop Culture & Fun",
        biome_name: "Retro Roller Disco Rink",
        environment: "cyber",
        road: "grid",
        mountain: "skyline",
        natural_palette: ["#ff4081", "#00e5ff", "#ffff00", "#76ff03", "#ffffff"],
        natural_rgb: [(255, 64, 129), (0, 229, 255), (255, 255, 0), (118, 255, 3), (255, 255, 255)],
        description: "Classic internet lols on glowing neon roller quad wheels.",
        movement_lore: "Smooth gliding roller glide with flashing neon wheel rotation.",
    },
    MascotBiomeInfo {
        name: "shikato",
        category: "Wild & Safari",
        biome_name: "Ancient Nara Cedar Shrine",
        environment: "forest",
        road: "cobblestone",
        mountain: "hills",
        natural_palette: ["#d7ccc8", "#8d6e63", "#ffffff", "#ff80ab", "#ffffff"],
        natural_rgb: [(215, 204, 200), (141, 110, 99), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
        description: "Revered sacred sika deer bowing politely along lantern-lit stone paths.",
        movement_lore: "Graceful contemplative forest steps and gentle bowing head tilts.",
    },
    MascotBiomeInfo {
        name: "shrug",
        category: "Abstract & Quirky",
        biome_name: "Minimalist Digital Monologue",
        environment: "space",
        road: "none",
        mountain: "none",
        natural_palette: ["#ffffff", "#cfd8dc", "#b0bec5", "#90a4ae", "#ffffff"],
        natural_rgb: [(255, 255, 255), (207, 216, 220), (176, 190, 197), (144, 164, 174), (255, 255, 255)],
        description: "Iconic ASCII shrug emoticon expressing cosmic indifference.",
        movement_lore: "Subtle shoulder lift sway and balanced arm suspension.",
    },
];

/// Retrieve the comprehensive biological biome profile for any mascot.
pub fn get_mascot_biome(animal: &str) -> &'static MascotBiomeInfo {
    let clean = animal
        .strip_suffix(".cow")
        .unwrap_or(animal)
        .to_ascii_lowercase();
    for mascot in &ALL_MASCOTS {
        if mascot.name == clean {
            return mascot;
        }
    }
    // Aliases and fallbacks
    match clean.as_str() {
        "cow" | "cowsay" => &ALL_MASCOTS[0],
        "rabbit" => get_mascot_biome("bunny"),
        "beavis" => get_mascot_biome("beavis.zen"),
        "nyancat" | "nyan-cat" | "nyan_cat" => get_mascot_biome("nyan"),
        "shiba" | "shibu" | "shiba-inu" => get_mascot_biome("doge"),
        "kitten" | "kittens" => get_mascot_biome("cat"),
        _ => &ALL_MASCOTS[0], // Default cow
    }
}

// ── Authentic Biological Natural Palette Variations ─────────────────────────

/// Cattle color variations (Bos taurus) across domestic dairy, beef, and draught breeds.
pub static CATTLE_VARIATIONS: [NaturalPaletteVariation; 6] = [
    NaturalPaletteVariation {
        name: "Holstein (White & Black Patches)",
        description: "Classic domestic dairy cow with crisp white coat, iconic black patches, and pink udder.",
        palette: ["#ffffff", "#1a1a1a", "#2c2c2c", "#ffb6c1", "#ffffff"],
        rgb: [(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Charolais (Complete White)",
        description: "Pristine solid creamy-white French breed with soft porcelain sheen and pale muzzle.",
        palette: ["#fdfbf7", "#eae5dc", "#d8d0c5", "#f4a6b8", "#fdfbf7"],
        rgb: [(253, 251, 247), (234, 229, 220), (216, 208, 197), (244, 166, 184), (253, 251, 247)],
    },
    NaturalPaletteVariation {
        name: "Black Angus (Complete Black)",
        description: "Deep solid black Scottish Angus with charcoal hooves, horns, and dark muzzle.",
        palette: ["#1c1c1c", "#121212", "#2e2e2e", "#424242", "#ffffff"],
        rgb: [(28, 28, 28), (18, 18, 18), (46, 46, 46), (66, 66, 66), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Jersey (Brown Color)",
        description: "Warm golden-fawn Jersey dairy cow with rich caramel flanks and dark muzzle ring.",
        palette: ["#a06535", "#6e3f19", "#d49b6a", "#f4a6b8", "#ffffff"],
        rgb: [(160, 101, 53), (110, 63, 25), (212, 155, 106), (244, 166, 184), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Normande (Brown & Black Patches)",
        description: "Mottled brindle coat with irregular dark brown and black patches and pale underbelly.",
        palette: ["#6e4720", "#1a1a1a", "#d7ccc8", "#f4a6b8", "#ffffff"],
        rgb: [(110, 71, 32), (26, 26, 26), (215, 204, 200), (244, 166, 184), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Grey Ox (Grey with Black Color)",
        description: "Silver-grey working draught ox with charcoal neck, dark spine, and black hooves.",
        palette: ["#9e9e9e", "#424242", "#616161", "#e0e0e0", "#ffffff"],
        rgb: [(158, 158, 158), (66, 66, 66), (97, 97, 97), (224, 224, 224), (255, 255, 255)],
    },
];

/// Tiger color morphs (Panthera tigris).
pub static TIGER_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Bengal Tiger (Orange with Black Stripes)",
        description: "Fiery orange coat with dense black vertical stripes and snowy underbelly.",
        palette: ["#e65100", "#1a1a1a", "#ffffff", "#ff80ab", "#ffffff"],
        rgb: [(230, 81, 0), (26, 26, 26), (255, 255, 255), (255, 128, 171), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "White Bengal Tiger (White Color)",
        description: "Rare white morph with ivory coat, charcoal brown stripes, and ice-blue eyes.",
        palette: ["#f5f5f5", "#212121", "#ffffff", "#ffb6c1", "#64b5f6"],
        rgb: [(245, 245, 245), (33, 33, 33), (255, 255, 255), (255, 182, 193), (100, 181, 246)],
    },
    NaturalPaletteVariation {
        name: "Royal Bengal (Orange with Black & White Stripes)",
        description: "Vivid flame-orange coat with rich black stripes, white chest blaze, and amber eyes.",
        palette: ["#ff6f00", "#121212", "#f5f5f5", "#ff80ab", "#ffeb3b"],
        rgb: [(255, 111, 0), (18, 18, 18), (245, 245, 245), (255, 128, 171), (255, 235, 59)],
    },
];

/// Domestic and wild feline color patterns (Felis catus).
pub static FELINE_VARIATIONS: [NaturalPaletteVariation; 7] = [
    NaturalPaletteVariation {
        name: "Calico (Tri-color Ginger, Brown & White)",
        description: "Warm ginger and dark chocolate brown patches over pure white fur.",
        palette: ["#d35400", "#795548", "#ffffff", "#ffb6c1", "#212121"],
        rgb: [(211, 84, 0), (121, 85, 72), (255, 255, 255), (255, 182, 193), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Tuxedo (Black & White)",
        description: "Formal jet black coat with white chest bib, white paws, and emerald eyes.",
        palette: ["#1a1a1a", "#ffffff", "#333333", "#ffb6c1", "#69f0ae"],
        rgb: [(26, 26, 26), (255, 255, 255), (51, 51, 51), (255, 182, 193), (105, 240, 174)],
    },
    NaturalPaletteVariation {
        name: "Ginger Tabby (Marmalade Striped)",
        description: "Vibrant copper and apricot orange coat with prominent mackerel stripes.",
        palette: ["#e67e22", "#d35400", "#f39c12", "#ffb6c1", "#ffffff"],
        rgb: [(230, 126, 34), (211, 84, 0), (243, 156, 18), (255, 182, 193), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Russian Blue (Slate Grey)",
        description: "Even shimmering silver-blue coat with vivid jade green eyes.",
        palette: ["#78909c", "#546e7a", "#cfd8dc", "#ff80ab", "#69f0ae"],
        rgb: [(120, 144, 156), (84, 110, 122), (207, 216, 220), (255, 128, 171), (105, 240, 174)],
    },
    NaturalPaletteVariation {
        name: "Melanistic (Solid Black)",
        description: "Midnight black pantherine coat with striking luminescent golden eyes.",
        palette: ["#1e1e1e", "#0a0a0a", "#2a2a2a", "#333333", "#ffd700"],
        rgb: [(30, 30, 30), (10, 10, 10), (42, 42, 42), (51, 51, 51), (255, 215, 0)],
    },
    NaturalPaletteVariation {
        name: "Solid White (Snow White)",
        description: "Pristine snow white coat with soft pink nose and sapphire blue eyes.",
        palette: ["#ffffff", "#e0e0e0", "#f5f5f5", "#ffb6c1", "#42a5f5"],
        rgb: [(255, 255, 255), (224, 224, 224), (245, 245, 245), (255, 182, 193), (66, 165, 245)],
    },
    NaturalPaletteVariation {
        name: "Siamese (Seal Point)",
        description: "Pale fawn body with dark chocolate face mask, ears, tail, and sapphire eyes.",
        palette: ["#f5e6d3", "#3e2723", "#d7ccc8", "#ffb6c1", "#1e88e5"],
        rgb: [(245, 230, 211), (62, 39, 35), (215, 204, 200), (255, 182, 193), (30, 136, 229)],
    },
];

/// Wolf coat colorations (Canis lupus).
pub static CANINE_VARIATIONS: [NaturalPaletteVariation; 4] = [
    NaturalPaletteVariation {
        name: "Timber Wolf (Grey)",
        description: "Grizzled salt-and-pepper grey coat with tawny underwool and amber eyes.",
        palette: ["#757575", "#424242", "#9e9e9e", "#e0e0e0", "#ffd54f"],
        rgb: [(117, 117, 117), (66, 66, 66), (158, 158, 158), (224, 224, 224), (255, 213, 79)],
    },
    NaturalPaletteVariation {
        name: "Arctic Wolf (White)",
        description: "Thick insulating all-white polar pelt adapted for tundra camouflage.",
        palette: ["#f8f9fa", "#cfd8dc", "#eceff1", "#b0bec5", "#ffb300"],
        rgb: [(248, 249, 250), (207, 216, 220), (236, 239, 241), (176, 190, 197), (255, 179, 0)],
    },
    NaturalPaletteVariation {
        name: "Melanistic Wolf (Black)",
        description: "Striking black phase wolf with midnight fur and piercing golden irises.",
        palette: ["#212121", "#121212", "#424242", "#616161", "#ffca28"],
        rgb: [(33, 33, 33), (18, 18, 18), (66, 66, 66), (97, 97, 97), (255, 202, 40)],
    },
    NaturalPaletteVariation {
        name: "Red Wolf (Cinnamon)",
        description: "Tawny cinnamon-red coat with dark saddle and long slender reddish limbs.",
        palette: ["#bf360c", "#870000", "#f4511e", "#ff8a65", "#ffd54f"],
        rgb: [(191, 54, 12), (135, 0, 0), (244, 81, 30), (255, 138, 101), (255, 213, 79)],
    },
];

/// Bear colorations (Ursidae).
pub static BEAR_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Grizzly (Brown Bear)",
        description: "Deep chocolate brown fur tipped with silver-blonde grizzly highlights.",
        palette: ["#5d4037", "#3e2723", "#795548", "#8d6e63", "#212121"],
        rgb: [(93, 64, 55), (62, 39, 35), (121, 85, 72), (141, 110, 99), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Black Bear",
        description: "Glossy jet black coat with light brown muzzle.",
        palette: ["#212121", "#0d0d0d", "#424242", "#8d6e63", "#212121"],
        rgb: [(33, 33, 33), (13, 13, 13), (66, 66, 66), (141, 110, 99), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Kermode / Spirit Bear (White)",
        description: "Rare coastal rainforest white/cream subspecies with black nose.",
        palette: ["#f5f5dc", "#d7ccc8", "#efebe9", "#bcaaa4", "#212121"],
        rgb: [(245, 245, 220), (215, 204, 200), (239, 235, 233), (188, 170, 164), (33, 33, 33)],
    },
];

/// Fox variations (Vulpes).
pub static FOX_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Red Fox",
        description: "Rust-red body, snowy white throat and tail brush tip, and black ear backs/socks.",
        palette: ["#e64a19", "#d84315", "#ffffff", "#212121", "#ffffff"],
        rgb: [(230, 74, 25), (216, 67, 21), (255, 255, 255), (33, 33, 33), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Arctic Fox (Winter White)",
        description: "Dense snowy white winter pelt suited for sub-zero camouflage.",
        palette: ["#fdfbf7", "#cfd8dc", "#eceff1", "#90a4ae", "#212121"],
        rgb: [(253, 251, 247), (207, 216, 220), (236, 239, 241), (144, 164, 174), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Cross / Silver Fox",
        description: "Melanistic silver-tipped charcoal fur with distinct dark dorsal band.",
        palette: ["#37474f", "#212121", "#78909c", "#cfd8dc", "#ffd54f"],
        rgb: [(55, 71, 79), (33, 33, 33), (120, 144, 156), (207, 216, 220), (255, 213, 79)],
    },
];

/// Sheep variations (Ovis aries).
pub static SHEEP_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "White Merino",
        description: "Dense crinkled cream-white fleece with pale pink muzzle.",
        palette: ["#f5f5f5", "#e0e0e0", "#d7ccc8", "#ffb6c1", "#212121"],
        rgb: [(245, 245, 245), (224, 224, 224), (215, 204, 200), (255, 182, 193), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Black Suffolk",
        description: "Clean white fleece body with jet black woolless head and black legs.",
        palette: ["#212121", "#141414", "#d7ccc8", "#ffb6c1", "#212121"],
        rgb: [(33, 33, 33), (20, 20, 20), (215, 204, 200), (255, 182, 193), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Jacob Sheep (Piebald)",
        description: "Ancient spotted piebald fleece with alternating white and black patches.",
        palette: ["#f5f5f5", "#212121", "#616161", "#ffb6c1", "#212121"],
        rgb: [(245, 245, 245), (33, 33, 33), (97, 97, 97), (255, 182, 193), (33, 33, 33)],
    },
];

/// Bunny / Rabbit variations.
pub static BUNNY_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Snow White",
        description: "Pristine white fur with soft rose-pink inner ears and ruby eyes.",
        palette: ["#ffffff", "#f5f5f5", "#ffb6c1", "#ff80ab", "#ff1744"],
        rgb: [(255, 255, 255), (245, 245, 245), (255, 182, 193), (255, 128, 171), (255, 23, 68)],
    },
    NaturalPaletteVariation {
        name: "Wild Agouti Brown",
        description: "Natural wild agouti brown with grizzled grey guard hairs.",
        palette: ["#8d6e63", "#5d4037", "#d7ccc8", "#ffb6c1", "#212121"],
        rgb: [(141, 110, 99), (93, 64, 55), (215, 204, 200), (255, 182, 193), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Dutch Piebald (Black & White)",
        description: "Iconic white blaze, white front legs, and black cheeks and hindquarters.",
        palette: ["#ffffff", "#212121", "#424242", "#ffb6c1", "#212121"],
        rgb: [(255, 255, 255), (33, 33, 33), (66, 66, 66), (255, 182, 193), (33, 33, 33)],
    },
];

/// Duck variations (Anas platyrhynchos).
pub static DUCK_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Mallard Drake (Iridescent Green)",
        description: "Iridescent emerald green head, chestnut chest, white neck ring, and orange feet.",
        palette: ["#00796b", "#4e342e", "#ffd54f", "#ffffff", "#ff6f00"],
        rgb: [(0, 121, 107), (78, 52, 46), (255, 213, 79), (255, 255, 255), (255, 111, 0)],
    },
    NaturalPaletteVariation {
        name: "Mallard Hen (Camouflage Brown)",
        description: "Cryptic mottled buff-and-brown plumage with orange bill.",
        palette: ["#6d4c41", "#4e342e", "#8d6e63", "#d7ccc8", "#ff6f00"],
        rgb: [(109, 76, 65), (78, 52, 46), (141, 110, 99), (215, 204, 200), (255, 111, 0)],
    },
    NaturalPaletteVariation {
        name: "Pekin Duck (White)",
        description: "Pure domestic white feathers with vibrant bright orange bill and feet.",
        palette: ["#ffffff", "#f5f5f5", "#ff9800", "#ffb74d", "#212121"],
        rgb: [(255, 255, 255), (245, 245, 245), (255, 152, 0), (255, 183, 77), (33, 33, 33)],
    },
];

/// Owl variations (Strigiformes).
pub static OWL_VARIATIONS: [NaturalPaletteVariation; 3] = [
    NaturalPaletteVariation {
        name: "Barn Owl (Buff & White)",
        description: "Golden-buff and grey mantle with heart-shaped white facial disc.",
        palette: ["#d7ccc8", "#8d6e63", "#ffffff", "#4e342e", "#212121"],
        rgb: [(215, 204, 200), (141, 110, 99), (255, 255, 255), (78, 52, 46), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Snowy Owl (Arctic White)",
        description: "Stunning white plumage with narrow dark brown bars and bright yellow eyes.",
        palette: ["#ffffff", "#e0e0e0", "#424242", "#ffd600", "#212121"],
        rgb: [(255, 255, 255), (224, 224, 224), (66, 66, 66), (255, 214, 0), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Great Horned Owl (Mottled Brown)",
        description: "Heavily mottled grey-brown plumage with reddish-orange facial disc.",
        palette: ["#5d4037", "#3e2723", "#8d6e63", "#ffb300", "#212121"],
        rgb: [(93, 64, 55), (62, 39, 35), (141, 110, 99), (255, 179, 0), (33, 33, 33)],
    },
];

/// Equine variations (Equus).
pub static EQUINE_VARIATIONS: [NaturalPaletteVariation; 4] = [
    NaturalPaletteVariation {
        name: "Bay (Brown & Black)",
        description: "Rich reddish-brown body with black mane, tail, and lower legs.",
        palette: ["#8d4004", "#1a1a1a", "#542502", "#e0e0e0", "#212121"],
        rgb: [(141, 64, 4), (26, 26, 26), (84, 37, 2), (224, 224, 224), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Dapple Grey",
        description: "Lustrous grey coat with distinct darker concentric rings (dapples).",
        palette: ["#9e9e9e", "#e0e0e0", "#616161", "#212121", "#212121"],
        rgb: [(158, 158, 158), (224, 224, 224), (97, 97, 97), (33, 33, 33), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Black Stallion",
        description: "Solid deep black coat with dark mane and hooves.",
        palette: ["#1c1c1c", "#0f0f0f", "#383838", "#ffffff", "#212121"],
        rgb: [(28, 28, 28), (15, 15, 15), (56, 56, 56), (255, 255, 255), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Chestnut",
        description: "Warm copper-red coat with matching flaxen/chestnut points.",
        palette: ["#a04000", "#782800", "#cf6d17", "#ffffff", "#212121"],
        rgb: [(160, 64, 0), (120, 40, 0), (207, 109, 23), (255, 255, 255), (33, 33, 33)],
    },
];

/// Doge / Corgi variations.
pub static DOGE_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Red / Fawn (Classic Urajiro)",
        description: "Classic golden-red coat with white urajiro markings on cheeks and chest.",
        palette: ["#e67e22", "#ffffff", "#d35400", "#212121", "#ff80ab"],
        rgb: [(230, 126, 34), (255, 255, 255), (211, 84, 0), (33, 33, 33), (255, 128, 171)],
    },
    NaturalPaletteVariation {
        name: "Black & Tan (Tri-color)",
        description: "Glossy black saddle with tan points over eyebrows and white chest blaze.",
        palette: ["#212121", "#d35400", "#ffffff", "#424242", "#ff80ab"],
        rgb: [(33, 33, 33), (211, 84, 0), (255, 255, 255), (66, 66, 66), (255, 128, 171)],
    },
];

/// Koala variations (Phascolarctos cinereus) — monomorphic wild morph in nature.
pub static KOALA_VARIATIONS: [NaturalPaletteVariation; 1] = [
    NaturalPaletteVariation {
        name: "Ashen Grey (Eucalyptus Wild)",
        description: "Dense silver-grey wool with black leathery nose, white ear tufts, and dark eyes.",
        palette: ["#90a4ae", "#37474f", "#cfd8dc", "#263238", "#ffffff"],
        rgb: [(144, 164, 174), (55, 71, 79), (207, 216, 220), (38, 50, 56), (255, 255, 255)],
    },
];

/// Elephant variations (Elephantidae).
pub static ELEPHANT_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "African Savanna Elephant (Slate Grey)",
        description: "Wrinkled dusty slate-grey skin with dark ivory tusks.",
        palette: ["#78909c", "#546e7a", "#cfd8dc", "#37474f", "#212121"],
        rgb: [(120, 144, 156), (84, 110, 122), (207, 216, 220), (55, 71, 79), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Asian Forest Elephant (Earthy Brown & Pink Mottle)",
        description: "Dark earthy brown skin with pink depigmentation mottling across ears and trunk.",
        palette: ["#8d6e63", "#5d4037", "#ffcdd2", "#3e2723", "#212121"],
        rgb: [(141, 110, 99), (93, 64, 55), (255, 205, 210), (62, 39, 35), (33, 33, 33)],
    },
];

/// Penguin variations (Spheniscidae).
pub static PENGUIN_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Emperor Penguin",
        description: "Jet-black back, stark white breast, and glowing yellow/orange neck auricles.",
        palette: ["#1a1a1a", "#ffffff", "#ffd54f", "#ff9800", "#212121"],
        rgb: [(26, 26, 26), (255, 255, 255), (255, 213, 79), (255, 152, 0), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Adélie Penguin",
        description: "Classic tuxedo black and white plumage with distinctive white eye ring.",
        palette: ["#1a1a1a", "#ffffff", "#424242", "#ffffff", "#212121"],
        rgb: [(26, 26, 26), (255, 255, 255), (66, 66, 66), (255, 255, 255), (33, 33, 33)],
    },
];

/// Whale variations (Cetacea).
pub static WHALE_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Blue Whale (Oceanic Mottled)",
        description: "Mottled deep slate blue and cerulean dorsal with pale throat grooves.",
        palette: ["#37474f", "#263238", "#78909c", "#90a4ae", "#ffffff"],
        rgb: [(55, 71, 79), (38, 50, 56), (120, 144, 156), (144, 164, 174), (255, 255, 255)],
    },
    NaturalPaletteVariation {
        name: "Humpback Whale",
        description: "Dark charcoal-black back with bright white underbelly and pectoral fins.",
        palette: ["#212121", "#ffffff", "#424242", "#90a4ae", "#ffffff"],
        rgb: [(33, 33, 33), (255, 255, 255), (66, 66, 66), (144, 164, 174), (255, 255, 255)],
    },
];

/// Dolphin variations (Delphinidae).
pub static DOLPHIN_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Bottlenose Dolphin (Slate & Pearl)",
        description: "Sleek slate-grey dorsal blending into pearl-white abdominal belly.",
        palette: ["#607d8b", "#37474f", "#eceff1", "#90a4ae", "#212121"],
        rgb: [(96, 125, 139), (55, 71, 79), (236, 239, 241), (144, 164, 174), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Amazon River Dolphin (Pink Boto)",
        description: "Unique blushing bubblegum-pink freshwater river dolphin.",
        palette: ["#f48fb1", "#ec407a", "#fce4ec", "#c2185b", "#212121"],
        rgb: [(244, 143, 177), (236, 64, 122), (252, 228, 236), (194, 24, 91), (33, 33, 33)],
    },
];

/// Turtle variations (Testudines).
pub static TURTLE_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Green Sea Turtle",
        description: "Hydrodynamic olive-green and golden-amber sunburst carapace plates.",
        palette: ["#2e7d32", "#1b5e20", "#c8e6c9", "#fff9c4", "#212121"],
        rgb: [(46, 125, 50), (27, 94, 32), (200, 230, 201), (255, 249, 196), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Galapagos Giant Tortoise",
        description: "Massive dark weathered obsidian carapace with leathery dark limbs.",
        palette: ["#424242", "#212121", "#616161", "#8d6e63", "#212121"],
        rgb: [(66, 66, 66), (33, 33, 33), (97, 97, 97), (141, 110, 99), (33, 33, 33)],
    },
];

/// Cephalopod variations (Octopoda / Teuthida).
pub static CEPHALOPOD_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Coral Reef Octopus (Terracotta)",
        description: "Bioluminescent terracotta rust red with pale chromatophore suction cups.",
        palette: ["#d84315", "#bf360c", "#ff8a65", "#ffccbc", "#212121"],
        rgb: [(216, 67, 21), (191, 54, 12), (255, 138, 101), (255, 204, 188), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Deep Sea Giant Squid (Crimson & Pearl)",
        description: "Dark abyssal crimson mantle with opalescent pearlescent tentacles.",
        palette: ["#880e4f", "#4a148c", "#e1bee7", "#f8bbd0", "#ffffff"],
        rgb: [(136, 14, 79), (74, 20, 140), (225, 190, 231), (248, 187, 208), (255, 255, 255)],
    },
];

/// Amphibian / Frog variations.
pub static FROG_VARIATIONS: [NaturalPaletteVariation; 2] = [
    NaturalPaletteVariation {
        name: "Tree Frog (Emerald Green)",
        description: "Bright leaf-green skin with yellow throat and suction-cup pads.",
        palette: ["#43a047", "#1b5e20", "#a5d6a7", "#ffeb3b", "#212121"],
        rgb: [(67, 160, 71), (27, 94, 32), (165, 214, 167), (255, 235, 59), (33, 33, 33)],
    },
    NaturalPaletteVariation {
        name: "Poison Dart Frog (Cobalt & Gold)",
        description: "Electric sapphire blue body with brilliant sunshine-yellow dorsal stripe.",
        palette: ["#1976d2", "#0d47a1", "#ffd600", "#ffea00", "#212121"],
        rgb: [(25, 118, 210), (13, 71, 161), (255, 214, 0), (255, 234, 0), (33, 33, 33)],
    },
];

/// Monomorphic fallback variation for species with a single canonical natural phenotype.
pub static DEFAULT_VARIATION: [NaturalPaletteVariation; 1] = [
    NaturalPaletteVariation {
        name: "Natural Habitat Coat",
        description: "Standard natural pigmentation and markings authentic to species biology.",
        palette: ["#ffffff", "#1a1a1a", "#2c2c2c", "#ffb6c1", "#ffffff"],
        rgb: [(255, 255, 255), (26, 26, 26), (44, 44, 44), (255, 182, 193), (255, 255, 255)],
    },
];

/// Retrieve authentic biological natural color variations for any mascot.
pub fn get_mascot_variations(animal: &str) -> &'static [NaturalPaletteVariation] {
    let clean = animal
        .trim()
        .trim_end_matches(".cow")
        .to_ascii_lowercase();

    match clean.as_str() {
        "default" | "cow" | "cowsay" | "charlie" | "fat-cow" | "cowfee" | "cower" | "spidercow" => &CATTLE_VARIATIONS,
        "tiger" => &TIGER_VARIATIONS,
        "cat" | "cat2" | "catfence" | "kitty" | "kitten" | "kittens" | "meow" | "bill-the-cat" => &FELINE_VARIATIONS,
        "wolf" => &CANINE_VARIATIONS,
        "bearface" | "telebears" => &BEAR_VARIATIONS,
        "fox" | "vulpix" => &FOX_VARIATIONS,
        "sheep" | "lamb" | "lamb2" | "ram" | "flaming-sheep" => &SHEEP_VARIATIONS,
        "bunny" | "rabbit" => &BUNNY_VARIATIONS,
        "duck" => &DUCK_VARIATIONS,
        "owl" => &OWL_VARIATIONS,
        "mule" => &EQUINE_VARIATIONS,
        "doge" | "corgi" | "shiba" | "shibu" | "shiba-inu" => &DOGE_VARIATIONS,
        "koala" | "luke-koala" => &KOALA_VARIATIONS,
        "elephant" | "elephant2" | "elephant-in-snake" => &ELEPHANT_VARIATIONS,
        "tux" | "tux-big" => &PENGUIN_VARIATIONS,
        "whale" | "happy-whale" | "docker-whale" => &WHALE_VARIATIONS,
        "dolphin" => &DOLPHIN_VARIATIONS,
        "turtle" | "tortoise" => &TURTLE_VARIATIONS,
        "octopus" | "smiling-octopus" | "squid" => &CEPHALOPOD_VARIATIONS,
        "bud-frogs" => &FROG_VARIATIONS,
        _ => &DEFAULT_VARIATION,
    }
}


/// Retrieve the recommended scenery triad (Environment, Road, Mountain) for any mascot.
pub fn get_recommended_scenery(animal: &str) -> (&'static str, &'static str, &'static str) {
    let info = get_mascot_biome(animal);
    (info.environment, info.road, info.mountain)
}

/// Retrieve the authentic natural hex palette (5 colors) for any mascot.
pub fn get_natural_hex_palette(animal: &str) -> &'static [&'static str] {
    &get_mascot_biome(animal).natural_palette
}

/// Retrieve the authentic natural RGB palette (5 colors) for any mascot.
pub fn get_natural_rgb_palette(animal: &str) -> &'static [(u8, u8, u8)] {
    &get_mascot_biome(animal).natural_rgb
}

/// Retrieve the full slice of all 132 mascots.
pub fn get_all_mascots() -> &'static [MascotBiomeInfo] {
    &ALL_MASCOTS
}

/// Ensure a color has sufficient luminance to be clearly visible against dark terminal backgrounds.
pub fn ensure_contrast(r: u8, g: u8, b: u8, min_lum: f32) -> (u8, u8, u8) {
    let lum = 0.299 * (r as f32) + 0.587 * (g as f32) + 0.114 * (b as f32);
    if lum < min_lum {
        let boost = min_lum - lum;
        (
            (r as f32 + boost).min(255.0).round() as u8,
            (g as f32 + boost).min(255.0).round() as u8,
            (b as f32 + boost).min(255.0).round() as u8,
        )
    } else {
        (r, g, b)
    }
}

/// Ensure a color has sufficient contrast against light terminal backgrounds.
/// If the foreground color is too light (high luminance), darken it so it contrasts cleanly against light background.
pub fn ensure_light_contrast(rgb: (u8, u8, u8), bg_lum: f32, min_delta: f32) -> (u8, u8, u8) {
    let lum = 0.299 * (rgb.0 as f32) + 0.587 * (rgb.1 as f32) + 0.114 * (rgb.2 as f32);
    let delta = bg_lum - lum;
    if delta < min_delta {
        let scale = ((bg_lum - min_delta) / lum.max(1.0)).clamp(0.12, 0.95);
        (
            (rgb.0 as f32 * scale).round() as u8,
            (rgb.1 as f32 * scale).round() as u8,
            (rgb.2 as f32 * scale).round() as u8,
        )
    } else {
        rgb
    }
}

/// Resolve authentic God-given natural coloration for a character glyph `ch`
/// at creature coordinate `(rel_x, rel_y)` on the animal silhouette.
///
/// Multi-layer anatomical palette mapping:
/// - Index 0: Primary body coat / fleece / dorsal plumage
/// - Index 1: Secondary markings / horns / ears / crown / stripes
/// - Index 2: Highlight / chest / underbelly / muzzle / throat
/// - Index 3: Accent / beak / nose / snout / inner ears / pink udder
/// - Index 4: Contrast / eyes / claws / hooves / detail spots
pub fn natural_creature_color(
    palette: &[(u8, u8, u8)],
    _rel_x: usize,
    rel_y: usize,
    ch: char,
) -> (u8, u8, u8) {
    if palette.is_empty() {
        return (255, 255, 255);
    }
    if palette.len() == 1 {
        return palette[0];
    }

    let c0 = ensure_contrast(palette[0].0, palette[0].1, palette[0].2, 75.0);
    let p1 = palette.get(1).copied().unwrap_or(palette[0]);
    let c1 = ensure_contrast(p1.0, p1.1, p1.2, 70.0);
    let p2 = palette.get(2).copied().unwrap_or(p1);
    let c2 = ensure_contrast(p2.0, p2.1, p2.2, 80.0);
    let p3 = palette.get(3).copied().unwrap_or(p2);
    let c3 = ensure_contrast(p3.0, p3.1, p3.2, 85.0);
    let p4 = palette.get(4).copied().unwrap_or((255, 255, 255));
    let c4 = ensure_contrast(p4.0, p4.1, p4.2, 140.0);

    // Ensure eye contrast is clearly distinct from the primary coat
    let lum0 = 0.299 * (c0.0 as f32) + 0.587 * (c0.1 as f32) + 0.114 * (c0.2 as f32);
    let lum4 = 0.299 * (c4.0 as f32) + 0.587 * (c4.1 as f32) + 0.114 * (c4.2 as f32);
    let eye_color = if (lum4 - lum0).abs() < 25.0 {
        (255, 255, 255)
    } else {
        c4
    };

    // 1. Eyes: distinct signature contrast across full creature anatomy (up to 18 rows)
    if (ch == 'o' || ch == 'O' || ch == '@' || ch == '*' || ch == '$' || ch == '0')
        && (rel_y <= 18 || ch == '@' || ch == '*' || ch == 'o' || ch == 'O')
    {
        return eye_color;
    }

    // 2. Udder / Muzzle / Beak / Tongue / Whiskers
    if (ch == 'w' || ch == 'W') && rel_y >= 3 {
        return c3;
    }
    if (ch == 'U' || ch == 'V' || ch == 'p') && rel_y <= 8 {
        return c3;
    }
    if (ch == ':' || ch == '>' || ch == '~') && rel_y <= 8 {
        return c3;
    }

    // 3. Horns, ears, crest (upper rows)
    if (ch == '^' || ch == '/' || ch == '\\' || ch == '\'' || ch == '`') && rel_y <= 3 {
        return c1;
    }

    // 4. Underbelly / chest lines
    if (ch == '_' || ch == '=') && rel_y >= 2 {
        return c2;
    }
    // 5. Primary body coat / plumage / silhouette
    c0
}

/// Resolve authentic natural creature color adapted to the terminal's background.
///
/// If `bg_rgb` has a light luminance (>= 128), pure white and pale colors are
/// shaded into rich natural contour tones (deep taupe, charcoal, slate) and
/// eyes/markings are sharpened so they contrast brilliantly against light backgrounds.
pub fn natural_creature_color_adaptive(
    palette: &[(u8, u8, u8)],
    rel_x: usize,
    rel_y: usize,
    ch: char,
    bg_rgb: (u8, u8, u8),
) -> (u8, u8, u8) {
    let bg_lum = 0.299 * (bg_rgb.0 as f32) + 0.587 * (bg_rgb.1 as f32) + 0.114 * (bg_rgb.2 as f32);
    if bg_lum < 128.0 {
        return natural_creature_color(palette, rel_x, rel_y, ch);
    }

    if palette.is_empty() {
        return (20, 20, 20);
    }
    if palette.len() == 1 {
        return ensure_light_contrast(palette[0], bg_lum, 70.0);
    }

    // Light theme adaptation:
    // Scale coats to rich natural dark contours
    let c0 = ensure_light_contrast(palette[0], bg_lum, 80.0);
    let p1 = palette.get(1).copied().unwrap_or(palette[0]);
    let c1 = ensure_light_contrast(p1, bg_lum, 70.0);
    let p2 = palette.get(2).copied().unwrap_or(p1);
    let c2 = ensure_light_contrast(p2, bg_lum, 65.0);
    let p3 = palette.get(3).copied().unwrap_or(p2);
    let c3 = ensure_light_contrast(p3, bg_lum, 55.0);
    let p4 = palette.get(4).copied().unwrap_or((0, 0, 0));
    let c4 = ensure_light_contrast(p4, bg_lum, 95.0);

    let lum0 = 0.299 * (c0.0 as f32) + 0.587 * (c0.1 as f32) + 0.114 * (c0.2 as f32);
    let lum4 = 0.299 * (c4.0 as f32) + 0.587 * (c4.1 as f32) + 0.114 * (c4.2 as f32);
    let eye_color = if (lum4 - lum0).abs() < 25.0 {
        (10, 10, 10)
    } else {
        c4
    };

    // 1. Eyes: distinct signature contrast
    if (ch == 'o' || ch == 'O' || ch == '@' || ch == '*' || ch == '$' || ch == '0')
        && (rel_y <= 18 || ch == '@' || ch == '*' || ch == 'o' || ch == 'O')
    {
        return eye_color;
    }

    // 2. Udder / Muzzle / Beak / Tongue
    if (ch == 'w' || ch == 'W') && rel_y >= 3 {
        return c3;
    }
    if (ch == 'U' || ch == 'V' || ch == 'p') && rel_y <= 8 {
        return c3;
    }
    if (ch == ':' || ch == '>' || ch == '~') && rel_y <= 8 {
        return c3;
    }

    // 3. Horns, ears, crest
    if (ch == '^' || ch == '/' || ch == '\\' || ch == '\'' || ch == '`') && rel_y <= 3 {
        return c1;
    }

    // 4. Underbelly / chest lines
    if (ch == '_' || ch == '=') && rel_y >= 2 {
        return c2;
    }

    // 5. Primary body coat
    c0
}

/// Resolve natural creature color with the specified background contrast mode applied.
#[must_use]
pub fn natural_creature_color_with_contrast(
    palette: &[(u8, u8, u8)],
    rel_x: usize,
    rel_y: usize,
    ch: char,
    contrast_mode: crate::terminal_theme::ContrastMode,
) -> (u8, u8, u8) {
    let (r, g, b) = natural_creature_color(palette, rel_x, rel_y, ch);
    contrast_mode.adjust_rgb(r, g, b)
}
