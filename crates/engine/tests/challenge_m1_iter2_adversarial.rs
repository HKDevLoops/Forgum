//! Empirical Adversarial Challenge Suite for Milestone 1 Iteration 2
//!
//! Stress-tests resilient deserialization, canonical vs non-canonical corrupted handling,
//! biological palette recovery, and kinematics invariants.

use forgum_engine::dna::{
    get_dna, parse_animations_json_str, BaseAnim, CowDna,
};
use forgum_platform::biome::get_all_mascots;
use std::collections::HashMap;

/// 1. Empirically verify that arbitrary corrupted non-canonical entries are omitted
///    without crashing or polluting the catalog.
#[test]
fn test_adversarial_arbitrary_corrupted_non_canonical_mascots_omitted() {
    let arbitrary_bad_names = vec![
        "bad_mascot",
        "invalid_cow",
        "broken_cow",
        "corrupt_cow",
        "poison_cow",
        "evil_dragon",
        "synthetic_mutant_123",
        "fake_doge",
        "hacker_cow",
        "__internal_mock__",
        "not_in_biome",
        "corrupt_tux",
        "poison_elephant",
        "broken_charizard",
        "totally_unknown_gibberish_xyz",
    ];

    let mut json_obj = serde_json::Map::new();

    // Valid mascot entry
    json_obj.insert(
        "valid_entry".to_string(),
        serde_json::json!({
            "base": "Float",
            "speed": 1.5,
            "palette": ["#111111", "#222222", "#333333", "#444444", "#555555"]
        }),
    );

    // Corrupted non-canonical entries
    for bad_name in &arbitrary_bad_names {
        json_obj.insert(
            bad_name.to_string(),
            serde_json::json!({
                "base": "NotARealAnimationVariant",
                "speed": "invalid_speed_string",
                "particles": 99999,
                "amplitude": { "x": "bad", "y": true }
            }),
        );
    }

    let raw_json = serde_json::to_string(&serde_json::Value::Object(json_obj)).unwrap();

    let catalog = parse_animations_json_str(&raw_json, None)
        .expect("parse_animations_json_str must succeed despite corrupt entries");

    // The valid entry must be present
    assert!(catalog.contains_key("valid_entry"), "Valid entry must be loaded");
    assert_eq!(catalog["valid_entry"].base, BaseAnim::Float);

    // NONE of the arbitrary corrupted non-canonical entries must be present
    for bad_name in &arbitrary_bad_names {
        assert!(
            !catalog.contains_key(*bad_name),
            "Corrupted non-canonical entry '{bad_name}' must NOT be in catalog"
        );
    }

    assert_eq!(
        catalog.len(),
        1,
        "Catalog must only contain the 1 valid entry and 0 corrupted non-canonical entries"
    );
}

/// 2. Empirically verify that when ALL 132 canonical mascots are corrupted simultaneously,
///    every single one recovers with BaseAnim::Walk, speed 1.0, and authentic biological palette.
#[test]
fn test_all_132_canonical_mascots_corrupted_simultaneously_recover() {
    let all_mascots = get_all_mascots();
    assert_eq!(all_mascots.len(), 132);

    let mut json_obj = serde_json::Map::new();

    // Corrupt all 132 canonical mascots
    for mascot in all_mascots {
        json_obj.insert(
            mascot.name.to_string(),
            serde_json::json!({
                "base": "BrokenEnumVariant",
                "speed": [1, 2, 3],
                "particles": "should_be_object_or_array",
                "palette": 42
            }),
        );
    }

    // Add 10 corrupted non-canonical entries as well
    for i in 0..10 {
        json_obj.insert(
            format!("adversarial_fake_{i}"),
            serde_json::json!({
                "base": "TotallyFake",
                "speed": "NaN"
            }),
        );
    }

    let raw_json = serde_json::to_string(&serde_json::Value::Object(json_obj)).unwrap();

    let catalog = parse_animations_json_str(&raw_json, None)
        .expect("parse_animations_json_str must succeed");

    // Must contain exactly the 132 canonical mascots and zero non-canonical entries
    assert_eq!(
        catalog.len(),
        132,
        "Catalog must contain exactly 132 recovered canonical mascots"
    );

    for mascot in all_mascots {
        assert!(
            catalog.contains_key(mascot.name),
            "Catalog must contain recovered mascot '{}'",
            mascot.name
        );

        let dna = &catalog[mascot.name];
        assert_eq!(
            dna.base,
            BaseAnim::Walk,
            "Recovered mascot '{}' must have BaseAnim::Walk",
            mascot.name
        );
        assert_eq!(
            dna.speed, 1.0,
            "Recovered mascot '{}' must have speed 1.0",
            mascot.name
        );
        assert_eq!(
            dna.palette.len(),
            5,
            "Recovered mascot '{}' must have 5 palette slots",
            mascot.name
        );
        assert_eq!(
            dna.palette, mascot.natural_palette,
            "Recovered mascot '{}' must have authentic biological palette matching biome.rs",
            mascot.name
        );
    }

    // Ensure none of the 10 adversarial fake entries made it into the catalog
    for i in 0..10 {
        let fake = format!("adversarial_fake_{i}");
        assert!(
            !catalog.contains_key(&fake),
            "Fake entry '{fake}' must NOT be in catalog"
        );
    }
}

/// 3. Verify canonical mascots with `.cow` suffix corrupted recover properly,
///    while non-canonical `.cow` entries are omitted.
#[test]
fn test_canonical_mascots_with_cow_extension_corrupted() {
    let test_json = r##"{
        "dragon.cow": {
            "base": "InvalidAnim",
            "speed": "invalid"
        },
        "tux.cow": {
            "base": "InvalidAnim",
            "speed": "invalid"
        },
        "nonexistent_animal.cow": {
            "base": "InvalidAnim",
            "speed": "invalid"
        }
    }"##;

    let catalog = parse_animations_json_str(test_json, None)
        .expect("parse_animations_json_str must succeed");

    assert!(catalog.contains_key("dragon.cow"), "dragon.cow must recover");
    assert!(catalog.contains_key("tux.cow"), "tux.cow must recover");
    assert!(
        !catalog.contains_key("nonexistent_animal.cow"),
        "nonexistent_animal.cow must NOT be in catalog"
    );

    let dragon_dna = &catalog["dragon.cow"];
    assert_eq!(dragon_dna.base, BaseAnim::Walk);
    assert_eq!(dragon_dna.speed, 1.0);
    assert_eq!(
        dragon_dna.palette,
        forgum_platform::biome::get_natural_hex_palette("dragon")
    );

    let tux_dna = &catalog["tux.cow"];
    assert_eq!(tux_dna.base, BaseAnim::Walk);
    assert_eq!(tux_dna.speed, 1.0);
    assert_eq!(
        tux_dna.palette,
        forgum_platform::biome::get_natural_hex_palette("tux")
    );
}

/// 4. Case-insensitivity check: canonical mascots with arbitrary casing recover properly.
#[test]
fn test_canonical_mascots_case_insensitivity_recovery() {
    let test_json = r##"{
        "DRAGON": {
            "base": "Corrupt",
            "speed": "Corrupt"
        },
        "ElEpHaNt": {
            "base": "Corrupt",
            "speed": "Corrupt"
        }
    }"##;

    let catalog = parse_animations_json_str(test_json, None)
        .expect("parse_animations_json_str must succeed");

    assert!(catalog.contains_key("DRAGON"));
    assert!(catalog.contains_key("ElEpHaNt"));

    let dragon = &catalog["DRAGON"];
    assert_eq!(dragon.base, BaseAnim::Walk);
    assert_eq!(
        dragon.palette,
        forgum_platform::biome::get_natural_hex_palette("dragon")
    );

    let elephant = &catalog["ElEpHaNt"];
    assert_eq!(elephant.base, BaseAnim::Walk);
    assert_eq!(
        elephant.palette,
        forgum_platform::biome::get_natural_hex_palette("elephant")
    );
}

/// 5. Verify biological palette is populated when `"palette"` is omitted or empty,
///    but explicit custom palettes are preserved.
#[test]
fn test_palette_omission_vs_custom_preservation() {
    let test_json = r##"{
        "dragon": {
            "base": "Fly",
            "speed": 2.0
        },
        "tux": {
            "base": "Breathe",
            "speed": 0.8,
            "palette": []
        },
        "cat": {
            "base": "Walk",
            "speed": 1.2,
            "palette": ["#111111", "#222222", "#333333", "#444444", "#555555"]
        }
    }"##;

    let catalog = parse_animations_json_str(test_json, None)
        .expect("parse_animations_json_str must succeed");

    // dragon: omitted palette -> receives dragon's biological palette
    let dragon = &catalog["dragon"];
    assert_eq!(dragon.base, BaseAnim::Fly);
    assert_eq!(dragon.speed, 2.0);
    assert_eq!(
        dragon.palette,
        forgum_platform::biome::get_natural_hex_palette("dragon"),
        "Omitted palette must be populated with biological palette"
    );

    // tux: empty palette -> receives tux's biological palette
    let tux = &catalog["tux"];
    assert_eq!(tux.base, BaseAnim::Breathe);
    assert_eq!(tux.speed, 0.8);
    assert_eq!(
        tux.palette,
        forgum_platform::biome::get_natural_hex_palette("tux"),
        "Empty palette array must be populated with biological palette"
    );

    // cat: explicit custom palette -> custom palette preserved
    let cat = &catalog["cat"];
    assert_eq!(cat.base, BaseAnim::Walk);
    assert_eq!(cat.speed, 1.2);
    assert_eq!(
        cat.palette,
        vec!["#111111", "#222222", "#333333", "#444444", "#555555"],
        "Explicit custom palette must NOT be overwritten"
    );
}

/// 6. Double-extension and unknown mascot lookup invariants with get_dna
#[test]
fn test_get_dna_invariants() {
    let mut map = HashMap::new();
    map.insert(
        "dragon".to_string(),
        CowDna {
            base: BaseAnim::Fly,
            speed: 3.0,
            ..CowDna::from_biome("dragon")
        },
    );

    // Exact key
    assert_eq!(get_dna(&map, "dragon").speed, 3.0);

    // Single extension match
    assert_eq!(get_dna(&map, "dragon.cow").speed, 3.0);

    // Double extension must NOT match dragon; must fallback to CowDna::default()
    let double_ext = get_dna(&map, "dragon.cow.cow");
    assert_eq!(
        double_ext,
        CowDna::default(),
        "Double extension 'dragon.cow.cow' must fall back to CowDna::default()"
    );

    // Unregistered known canonical mascot in biome: recovers from biome
    let cat_dna = get_dna(&map, "cat");
    assert_eq!(cat_dna.base, BaseAnim::Walk);
    assert_eq!(
        cat_dna.palette,
        forgum_platform::biome::get_natural_hex_palette("cat")
    );

    // Completely unknown name: falls back to CowDna::default()
    let unknown_dna = get_dna(&map, "completely_unknown_entity_999");
    assert_eq!(
        unknown_dna,
        CowDna::default(),
        "Completely unknown mascot must fall back to CowDna::default()"
    );
}
