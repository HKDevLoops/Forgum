//! Exhaustive DNA and mascot coverage audit tests.
//!
//! Validates:
//! 1. All 106 curated mascots from `crates/tui/src/app.rs` CATEGORIES exist in both:
//!    - `data/animations.json`
//!    - `data/Cows/animations.json`
//! 2. Every single entry contains:
//!    - `base`: valid `BaseAnim` variant
//!    - `particles`: valid `ParticleType` or null
//!    - `speed`: float > 0.0
//!    - `palette`: exactly 5-color array of valid hex strings matching `color::get_natural_hex_palette`
//! 3. `hamster` is present in both files and `milk` is 100% absent.
//! 4. `dna::load_animations` loads every single entry without fallback or deserialization errors.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use forgum_engine::color::get_natural_hex_palette;
use forgum_engine::dna::{load_animations, BaseAnim, ParticleType};

/// Curated 106 mascots from `crates/tui/src/app.rs` CATEGORIES.
const MASCOTS_106: &[&str] = &[
    // Farm & Domestic (22)
    "default", "cat", "cat2", "catfence", "charlie", "corgi", "bunny", "doge", "fat-cow",
    "goat", "goat2", "hippie", "kitty", "kitten", "meow", "hamster", "mule", "pig", "ram",
    "rooster", "sheep", "turkey",
    // Wild & Safari (16)
    "armadillo", "bearface", "elephant", "elephant2", "elephant-in-snake", "fox",
    "hedgehog", "koala", "luke-koala", "moofasa", "panther", "rhino", "sloth",
    "telebears", "tiger", "wolf",
    // Oceanic & Amphibian (14)
    "bud-frogs", "docker-whale", "dolphin", "duck", "ebi_furai", "happy-whale",
    "jellyfish", "octopus", "pufferfish", "seahorse", "squid", "turtle", "walrus", "whale",
    // Fantasy & Sci-Fi (19)
    "atat", "cthulhu-mini", "daemon", "dragon", "dragon-and-cow", "ghost", "ghostbusters",
    "glados", "mech-and-cow", "minotaur", "mooghidjirah", "moojira", "pterodactyl",
    "sauron", "stegosaurus", "unipony", "vader", "wizard", "yoda",
    // Pop Culture & Fun (17)
    "beavis.zen", "bill-the-cat", "charizardvice", "fat-banana", "flaming-sheep",
    "golden-eagle", "hellokitty", "hypno", "kiss", "mona-lisa", "nyan", "radioactive-kitty",
    "ren", "snoopy", "stimpy", "tux", "vulpix",
    // Abstract & Quirky (18)
    "apt", "bees", "claw-arm", "cower", "cowfee", "eyes", "fence", "hiya", "jesus",
    "kosh", "mutilated", "queen", "skeleton", "small", "supermilker", "surgery",
    "three-eyes", "viper",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn test_categories_count_is_exact_106() {
    assert_eq!(
        MASCOTS_106.len(),
        106,
        "Expected exactly 106 mascots in curated categories"
    );
    let mut seen = HashSet::new();
    for m in MASCOTS_106 {
        assert!(seen.insert(*m), "Duplicate mascot in MASCOTS_106: {m}");
    }
}

#[test]
fn test_all_106_mascots_present_in_both_animation_files() {
    let root = repo_root();
    let data_json_path = root.join("data").join("animations.json");
    let cows_json_path = root.join("data").join("Cows").join("animations.json");

    let data_str = std::fs::read_to_string(&data_json_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", data_json_path.display()));
    let cows_str = std::fs::read_to_string(&cows_json_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", cows_json_path.display()));

    let data_map: serde_json::Value =
        serde_json::from_str(&data_str).expect("Failed to parse data/animations.json");
    let cows_map: serde_json::Value =
        serde_json::from_str(&cows_str).expect("Failed to parse data/Cows/animations.json");

    for &mascot in MASCOTS_106 {
        assert!(
            data_map.get(mascot).is_some(),
            "Mascot '{mascot}' missing from data/animations.json"
        );
        assert!(
            cows_map.get(mascot).is_some(),
            "Mascot '{mascot}' missing from data/Cows/animations.json"
        );
    }
}

#[test]
fn test_hamster_present_and_milk_100_percent_absent() {
    let root = repo_root();
    let data_json_path = root.join("data").join("animations.json");
    let cows_json_path = root.join("data").join("Cows").join("animations.json");
    let cows_dir = root.join("data").join("Cows");

    let data_str = std::fs::read_to_string(&data_json_path).unwrap();
    let cows_str = std::fs::read_to_string(&cows_json_path).unwrap();

    let data_map: serde_json::Value = serde_json::from_str(&data_str).unwrap();
    let cows_map: serde_json::Value = serde_json::from_str(&cows_str).unwrap();

    // Validate hamster presence
    assert!(
        data_map.get("hamster").is_some(),
        "'hamster' must be present in data/animations.json"
    );
    assert!(
        cows_map.get("hamster").is_some(),
        "'hamster' must be present in data/Cows/animations.json"
    );
    assert!(
        cows_dir.join("hamster.cow").exists(),
        "data/Cows/hamster.cow must exist on disk"
    );

    // Validate milk 100% absence
    assert!(
        data_map.get("milk").is_none(),
        "'milk' must be 100% absent from data/animations.json"
    );
    assert!(
        cows_map.get("milk").is_none(),
        "'milk' must be 100% absent from data/Cows/animations.json"
    );
    assert!(
        !cows_dir.join("milk.cow").exists(),
        "data/Cows/milk.cow must be 100% absent from disk"
    );
}

#[test]
fn test_dna_properties_and_palette_matching() {
    let root = repo_root();
    let data_json_path = root.join("data").join("animations.json");
    let cows_json_path = root.join("data").join("Cows").join("animations.json");

    let data_str = std::fs::read_to_string(&data_json_path).unwrap();
    let cows_str = std::fs::read_to_string(&cows_json_path).unwrap();

    let data_map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&data_str).unwrap();
    let cows_map: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&cows_str).unwrap();

    for &mascot in MASCOTS_106 {
        let expected_palette = get_natural_hex_palette(mascot);
        assert_eq!(
            expected_palette.len(),
            5,
            "God-given natural palette for '{mascot}' in color.rs must be exactly 5 colors"
        );

        // Check entry in data/animations.json
        let d_entry = data_map
            .get(mascot)
            .unwrap_or_else(|| panic!("missing {mascot} in data/animations.json"));
        let d_base_str = d_entry["base"].as_str().expect("base string");
        assert!(
            BaseAnim::parse(d_base_str).is_some(),
            "Invalid base animation '{d_base_str}' for mascot '{mascot}'"
        );
        let d_speed = d_entry["speed"].as_f64().expect("speed float");
        assert!(
            d_speed > 0.0,
            "Speed must be > 0.0 for mascot '{mascot}', got {d_speed}"
        );
        // Validate particles in data/animations.json
        if let Some(p) = d_entry.get("particles") {
            if !p.is_null() {
                if let Some(s) = p.as_str() {
                    let _: ParticleType = serde_json::from_value(serde_json::Value::String(s.to_string()))
                        .unwrap_or_else(|_| panic!("Invalid particle shorthand '{s}' for mascot '{mascot}'"));
                } else if let Some(obj) = p.as_object() {
                    if let Some(t) = obj.get("type") {
                        let _: ParticleType = serde_json::from_value(t.clone())
                            .unwrap_or_else(|_| panic!("Invalid particle type '{t:?}' for mascot '{mascot}'"));
                    }
                }
            }
        }
        let d_pal: Vec<&str> = d_entry["palette"]
            .as_array()
            .expect("palette array")
            .iter()
            .map(|v| v.as_str().expect("hex string"))
            .collect();
        assert_eq!(
            d_pal.len(),
            5,
            "Palette in data/animations.json for '{mascot}' must have 5 colors"
        );
        assert_eq!(
            d_pal.as_slice(),
            expected_palette,
            "Palette in data/animations.json for '{mascot}' must match color.rs natural palette"
        );

        // Check entry in data/Cows/animations.json
        let c_entry = cows_map
            .get(mascot)
            .unwrap_or_else(|| panic!("missing {mascot} in data/Cows/animations.json"));
        let c_base_str = c_entry["base"].as_str().expect("base string");
        assert_eq!(
            c_base_str, d_base_str,
            "Base animation mismatch for mascot '{mascot}'"
        );
        let c_speed = c_entry["speed"].as_f64().expect("speed float");
        assert_eq!(c_speed, d_speed, "Speed mismatch for mascot '{mascot}'");
        // Validate particles in data/Cows/animations.json
        if let Some(p) = c_entry.get("particles") {
            if !p.is_null() {
                if let Some(s) = p.as_str() {
                    let _: ParticleType = serde_json::from_value(serde_json::Value::String(s.to_string()))
                        .unwrap_or_else(|_| panic!("Invalid particle shorthand '{s}' for mascot '{mascot}'"));
                } else if let Some(obj) = p.as_object() {
                    if let Some(t) = obj.get("type") {
                        let _: ParticleType = serde_json::from_value(t.clone())
                            .unwrap_or_else(|_| panic!("Invalid particle type '{t:?}' for mascot '{mascot}'"));
                    }
                }
            }
        }
        let c_pal: Vec<&str> = c_entry["palette"]
            .as_array()
            .expect("palette array")
            .iter()
            .map(|v| v.as_str().expect("hex string"))
            .collect();
        assert_eq!(
            c_pal.len(),
            5,
            "Palette in data/Cows/animations.json for '{mascot}' must have 5 colors"
        );
        assert_eq!(
            c_pal.as_slice(),
            expected_palette,
            "Palette in data/Cows/animations.json for '{mascot}' must match color.rs natural palette"
        );
    }
}

#[test]
fn test_dna_loads_every_single_entry_without_fallback() {
    let root = repo_root();
    let data_dir = root.join("data");
    let cows_dir = root.join("data").join("Cows");

    // Load from data/ (data/animations.json)
    let anims_data = load_animations(&data_dir);
    assert_eq!(
        anims_data.len(),
        132,
        "data/animations.json must load all 132 animals"
    );

    // Load from data/Cows/ (data/Cows/animations.json)
    let anims_cows = load_animations(&cows_dir);
    assert_eq!(
        anims_cows.len(),
        132,
        "data/Cows/animations.json must load all 132 animals"
    );

    for &mascot in MASCOTS_106 {
        let dna_data = anims_data
            .get(mascot)
            .unwrap_or_else(|| panic!("Failed to load DNA for '{mascot}' from data/"));
        let dna_cows = anims_cows
            .get(mascot)
            .unwrap_or_else(|| panic!("Failed to load DNA for '{mascot}' from data/Cows/"));

        assert_eq!(
            dna_data.base, dna_cows.base,
            "Base animation mismatch for '{mascot}'"
        );
        assert!(dna_data.speed > 0.0);
        assert!(dna_cows.speed > 0.0);
        assert_eq!(dna_data.palette.len(), 5);
        assert_eq!(dna_cows.palette.len(), 5);
        assert_eq!(dna_data.palette, dna_cows.palette);
    }
}
