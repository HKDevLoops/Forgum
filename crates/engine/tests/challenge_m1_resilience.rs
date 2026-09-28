//! Empirical Challenge Suite for Milestone 1: Mascot DNA & Data Resilience
//!
//! Evaluates:
//! 1. Deserialization resilience against malicious/corrupted inputs (missing fields,
//!    wrong data types, empty strings, corrupt JSON syntax, unknown mascot names).
//! 2. Panic resistance and preservation of surviving mascots under corruption.
//! 3. 132/132 mascot catalog coverage and biological palette conformance.
//! 4. Empirical analysis of the "corrupt"/"poison" name-filtering heuristic in `dna.rs`.

use forgum_engine::dna::{
    get_dna, instance_phase, load_animations, load_animations_or_embedded,
    load_embedded_animations, parse_animations_json_str, synthesize_biome_catalog, BaseAnim,
    CowDna,
};
use forgum_platform::biome::get_all_mascots;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// =========================================================================
// CHALLENGE 1: All 132 Mascots Biological Palettes & Loadability
// =========================================================================

#[test]
fn challenge_132_mascots_loadable_and_match_biome_palettes() {
    let all_mascots = get_all_mascots();
    assert_eq!(
        all_mascots.len(),
        132,
        "Total mascots in biome.rs must be exactly 132"
    );

    // 1. Check data/Cows/animations.json
    let cows_anim_path = repo_root().join("data").join("Cows").join("animations.json");
    assert!(cows_anim_path.exists(), "data/Cows/animations.json must exist");
    let cows_anim_str = fs::read_to_string(&cows_anim_path).expect("read Cows/animations.json");
    let cows_catalog = parse_animations_json_str(&cows_anim_str, Some(&cows_anim_path))
        .expect("parse Cows/animations.json");

    assert_eq!(
        cows_catalog.len(),
        132,
        "Cows/animations.json must contain exactly 132 mascots"
    );

    // 2. Check data/animations.json
    let root_anim_path = repo_root().join("data").join("animations.json");
    assert!(root_anim_path.exists(), "data/animations.json must exist");
    let root_anim_str = fs::read_to_string(&root_anim_path).expect("read animations.json");
    let root_catalog = parse_animations_json_str(&root_anim_str, Some(&root_anim_path))
        .expect("parse animations.json");

    assert_eq!(
        root_catalog.len(),
        132,
        "animations.json must contain exactly 132 mascots"
    );

    // 3. Verify every mascot across all sources matches biological palette exactly
    for mascot in all_mascots {
        let name = mascot.name;

        // Verify slot count and hex format in biome definition
        assert_eq!(
            mascot.natural_palette.len(),
            5,
            "Mascot '{name}' must have 5 biological palette slots in biome.rs"
        );
        for (i, hex) in mascot.natural_palette.iter().enumerate() {
            assert!(
                hex.starts_with('#') && hex.len() == 7,
                "Mascot '{name}' slot {i} invalid hex: {hex}"
            );
            assert!(
                hex[1..].chars().all(|c| c.is_ascii_hexdigit()),
                "Mascot '{name}' slot {i} non-hex characters: {hex}"
            );
        }

        // Verify match in Cows/animations.json
        assert!(
            cows_catalog.contains_key(name),
            "Cows/animations.json missing mascot '{name}'"
        );
        let cows_dna = &cows_catalog[name];
        assert_eq!(
            cows_dna.palette,
            mascot.natural_palette,
            "Palette mismatch for '{name}' in Cows/animations.json"
        );

        // Verify match in data/animations.json
        assert!(
            root_catalog.contains_key(name),
            "animations.json missing mascot '{name}'"
        );
        let root_dna = &root_catalog[name];
        assert_eq!(
            root_dna.palette,
            mascot.natural_palette,
            "Palette mismatch for '{name}' in animations.json"
        );
    }
}

// =========================================================================
// CHALLENGE 2: Deserialization Resilience to Malicious & Corrupted Inputs
// =========================================================================

#[test]
fn challenge_deserialization_corrupted_json_syntax_never_panics() {
    let adversarial_syntaxes = vec![
        "",                                    // Empty string
        "   \t\r\n   ",                       // Whitespace only
        "{",                                   // Unterminated object
        "}",                                   // Stray close brace
        "{\"cow\": ",                          // Incomplete key-value
        "{\"cow\": {\"base\": \"Walk\"",       // Unclosed nested object
        "{\"cow\": {\"base\": \"Walk\"},}",    // Trailing comma
        "{\"cow\": {\"speed\": 1.0}} \0 extra",// Null byte in content
        "\"just a bare string\"",              // Non-object primitive string
        "12345",                               // Non-object primitive number
        "[1, 2, 3]",                           // Non-object array
        "null",                                // Primitive null
        "true",                                // Primitive boolean
        "{\"a\": {\"b\": {\"c\": {\"d\": 1}}}}", // Deeply nested structure
    ];

    for (i, input) in adversarial_syntaxes.iter().enumerate() {
        // Must never panic
        let res = std::panic::catch_unwind(|| {
            parse_animations_json_str(input, None)
        });
        assert!(
            res.is_ok(),
            "Input {i} caused a panic in parse_animations_json_str: {:?}",
            input
        );

        // load_animations on directory containing this corrupt file must never panic
        let tmp = tempdir().expect("tempdir");
        let anim_file = tmp.path().join("animations.json");
        fs::write(&anim_file, input).expect("write file");

        let load_res = std::panic::catch_unwind(|| {
            load_animations(tmp.path())
        });
        assert!(
            load_res.is_ok(),
            "Input {i} caused a panic in load_animations: {:?}",
            input
        );
    }
}

#[test]
fn challenge_deserialization_wrong_types_never_panics_and_preserves_survivors() {
    let wrong_type_payloads = vec![
        // Field: base
        serde_json::json!({ "base": 12345 }),
        serde_json::json!({ "base": false }),
        serde_json::json!({ "base": ["Walk"] }),
        serde_json::json!({ "base": { "type": "Walk" } }),
        serde_json::json!({ "base": "NonExistentVariantXYZ" }),
        // Field: speed
        serde_json::json!({ "speed": "supersonic" }),
        serde_json::json!({ "speed": null }),
        serde_json::json!({ "speed": [1.0, 2.0] }),
        serde_json::json!({ "speed": { "val": 1.0 } }),
        // Field: particles
        serde_json::json!({ "particles": 99999 }),
        serde_json::json!({ "particles": "NonExistentParticleTypeXYZ" }),
        serde_json::json!({ "particles": { "rate": "invalid_string_rate" } }),
        serde_json::json!({ "particles": { "life": "not_an_array" } }),
        serde_json::json!({ "particles": { "life": [1.0] } }), // wrong array length (expected 2)
        serde_json::json!({ "particles": { "speed": ["a", "b"] } }),
        // Field: amplitude
        serde_json::json!({ "amplitude": "huge_amplitude" }),
        serde_json::json!({ "amplitude": { "breath": "not_a_float" } }),
        serde_json::json!({ "amplitude": 100 }),
        // Field: palette
        serde_json::json!({ "palette": "not_an_array" }),
        serde_json::json!({ "palette": [1, 2, 3, 4, 5] }),
        serde_json::json!({ "palette": { "color1": "#ffffff" } }),
        // Field: easing
        serde_json::json!({ "easing": 999 }),
        serde_json::json!({ "easing": { "base": 123 } }),
        // Field: phase_seed
        serde_json::json!({ "phase_seed": "seed_42" }),
        serde_json::json!({ "phase_seed": -10 }),
        // Field: glow
        serde_json::json!({ "glow": "shiny" }),
        serde_json::json!({ "glow": { "radius": "wide" } }),
        serde_json::json!({ "glow": { "color": 12345 } }),
    ];

    for (idx, payload) in wrong_type_payloads.iter().enumerate() {
        let mut map = serde_json::Map::new();

        // 3 valid sibling mascots
        map.insert(
            "valid_doge".to_string(),
            serde_json::json!({
                "base": "Float",
                "speed": 1.2,
                "palette": ["#ffaa00", "#ff8800", "#ffffff", "#000000", "#ffffff"]
            }),
        );
        map.insert(
            "valid_cat".to_string(),
            serde_json::json!({
                "base": "Breathe",
                "speed": 0.9,
                "palette": ["#555555", "#777777", "#ffffff", "#ff80ab", "#ffffff"]
            }),
        );
        map.insert(
            "valid_bunny".to_string(),
            serde_json::json!({
                "base": "Walk",
                "speed": 1.5,
                "palette": ["#ffffff", "#cccccc", "#ffb6c1", "#ff1744", "#ffffff"]
            }),
        );

        // Insert the corrupted entry
        map.insert("corrupted_record".to_string(), payload.clone());

        let json_str = serde_json::to_string(&serde_json::Value::Object(map))
            .expect("serialize json");

        let catalog = parse_animations_json_str(&json_str, None)
            .unwrap_or_else(|e| panic!("Case {idx} should not fail catalog parse: {e}"));

        // Rule D: Surviving valid mascots must NEVER be dropped
        assert!(
            catalog.contains_key("valid_doge"),
            "Case {idx}: valid_doge dropped from catalog!"
        );
        assert!(
            catalog.contains_key("valid_cat"),
            "Case {idx}: valid_cat dropped from catalog!"
        );
        assert!(
            catalog.contains_key("valid_bunny"),
            "Case {idx}: valid_bunny dropped from catalog!"
        );

        assert_eq!(
            catalog.get("valid_doge").unwrap().base,
            BaseAnim::Float,
            "Case {idx}: valid_doge base mutated!"
        );
        assert_eq!(
            catalog.get("valid_cat").unwrap().base,
            BaseAnim::Breathe,
            "Case {idx}: valid_cat base mutated!"
        );
        assert_eq!(
            catalog.get("valid_bunny").unwrap().base,
            BaseAnim::Walk,
            "Case {idx}: valid_bunny base mutated!"
        );
    }
}

// =========================================================================
// CHALLENGE 3: Massive Corruption Stress Test
// =========================================================================

#[test]
fn challenge_massive_corruption_preserves_all_uncorrupted_mascots() {
    let cows_anim_path = repo_root().join("data").join("Cows").join("animations.json");
    let content = fs::read_to_string(&cows_anim_path).expect("read baseline animations.json");
    let mut raw_map: serde_json::Value =
        serde_json::from_str(&content).expect("parse valid baseline JSON");

    let obj = raw_map.as_object_mut().expect("top-level object");
    let all_keys: Vec<String> = obj.keys().cloned().collect();
    assert_eq!(all_keys.len(), 132);

    // Corrupt 50 mascots
    let corrupt_count = 50;
    let corrupted_keys: Vec<String> = all_keys.iter().take(corrupt_count).cloned().collect();
    let surviving_keys: Vec<String> = all_keys.iter().skip(corrupt_count).cloned().collect();

    for key in &corrupted_keys {
        obj.insert(
            key.clone(),
            serde_json::json!({
                "base": "BogusAnimationVariant9999",
                "speed": "super_light_speed_error",
                "particles": 12345,
                "amplitude": false,
                "palette": "red"
            }),
        );
    }

    let mutated_json = serde_json::to_string(&raw_map).expect("serialize mutated JSON");
    let catalog = parse_animations_json_str(&mutated_json, None)
        .expect("catalog parse must succeed despite 50 corrupted records");

    // All surviving 82 mascots must remain 100% intact
    for key in &surviving_keys {
        assert!(
            catalog.contains_key(key),
            "Surviving mascot '{key}' must be present"
        );
        let dna = &catalog[key];
        assert!(dna.speed > 0.0, "Speed must be valid for '{key}'");
        assert_eq!(
            dna.palette.len(),
            5,
            "Palette must have 5 slots for '{key}'"
        );
    }
}

// =========================================================================
// CHALLENGE 4: Empirical Audit of String Filtering Heuristic ("corrupt" / "poison")
// =========================================================================

#[test]
fn challenge_audit_corrupt_poison_heuristic_anomaly() {
    // We construct 4 malformed entries that fail CowDna serde deserialization:
    // 1. "dragon" (known mascot in biome.rs, name has neither 'corrupt' nor 'poison')
    // 2. "malformed_cow" (unknown name, neither 'corrupt' nor 'poison')
    // 3. "corrupt_dragon" (known base mascot, but name contains 'corrupt')
    // 4. "poison_dragon" (known base mascot, but name contains 'poison')

    let test_json = r##"{
        "valid_doge": {
            "base": "Walk",
            "speed": 1.0
        },
        "dragon": {
            "base": "InvalidVariant1",
            "speed": "not_a_number"
        },
        "malformed_cow": {
            "base": "InvalidVariant2",
            "speed": "not_a_number"
        },
        "corrupt_dragon": {
            "base": "InvalidVariant3",
            "speed": "not_a_number"
        },
        "poison_dragon": {
            "base": "InvalidVariant4",
            "speed": "not_a_number"
        }
    }"##;

    let catalog = parse_animations_json_str(test_json, None)
        .expect("parse_animations_json_str must succeed");

    // Valid doge must always be present
    assert!(catalog.contains_key("valid_doge"));

    // EMPIRICAL OBSERVATION OF THE HEURISTIC:
    // Notice what happens due to line 367 in dna.rs:
    // `if !lower.contains("corrupt") && !lower.contains("poison")`
    let dragon_present = catalog.contains_key("dragon");
    let malformed_cow_present = catalog.contains_key("malformed_cow");
    let corrupt_dragon_present = catalog.contains_key("corrupt_dragon");
    let poison_dragon_present = catalog.contains_key("poison_dragon");

    println!("Empirical Heuristic Audit:");
    println!("  'dragon' present: {dragon_present}");
    println!("  'malformed_cow' present: {malformed_cow_present}");
    println!("  'corrupt_dragon' present: {corrupt_dragon_present}");
    println!("  'poison_dragon' present: {poison_dragon_present}");

    // "dragon" failed deserialization, but because its name does not contain "corrupt" or "poison",
    // it was recovered via CowDna::from_biome("dragon").
    assert!(dragon_present, "'dragon' was recovered via from_biome");

    // "malformed_cow" also failed deserialization and is NOT in biome.rs!
    // Under principled canonical mascot validation, it is cleanly omitted from the catalog.
    assert!(
        !malformed_cow_present,
        "'malformed_cow' must NOT be inserted into catalog because it is not a known mascot"
    );

    // "corrupt_dragon" and "poison_dragon" were dropped completely solely because of their names:
    assert!(
        !corrupt_dragon_present,
        "'corrupt_dragon' was dropped due to name filter"
    );
    assert!(
        !poison_dragon_present,
        "'poison_dragon' was dropped due to name filter"
    );
}

// =========================================================================
// CHALLENGE 5: Regression Audit on Pre-existing Unit Tests
// =========================================================================

#[test]
fn challenge_audit_existing_unit_test_failures() {
    // Check 1: CowDna::default() palette contents
    let default_dna = CowDna::default();
    println!(
        "CowDna::default().palette length: {}",
        default_dna.palette.len()
    );
    // Notice: worker changed CowDna::default() to populate 5 colors,
    // which broke `assert!(dna.palette.is_empty())` in dna.rs:560!
    let default_is_empty = default_dna.palette.is_empty();
    println!("Is CowDna::default().palette empty? {default_is_empty}");

    // Check 2: get_dna with double extension
    let mut map = HashMap::new();
    map.insert(
        "dragon".to_string(),
        CowDna {
            speed: 2.0,
            ..CowDna::default()
        },
    );
    let resolved = get_dna(&map, "dragon.cow.cow");
    let fallback_matches_default = resolved == CowDna::default();
    println!(
        "get_dna(&map, 'dragon.cow.cow') matches CowDna::default()? {fallback_matches_default}"
    );
    // If resolved != CowDna::default(), it returned dragon's palette instead of default!
}

// =========================================================================
// CHALLENGE 6: Standalone Fallback & API Coverage
// =========================================================================

#[test]
fn challenge_fallback_loaders_and_kinematics() {
    let embedded = load_embedded_animations();
    assert_eq!(embedded.len(), 132, "Embedded animations must contain 132 mascots");

    let synthesized = synthesize_biome_catalog();
    assert_eq!(synthesized.len(), 132, "Synthesized biome catalog must contain 132 mascots");

    let tmp = tempdir().expect("tempdir");
    let loaded = load_animations_or_embedded(tmp.path());
    assert_eq!(loaded.len(), 132, "load_animations_or_embedded on empty dir must load embedded");

    let p0 = instance_phase(1234, 0);
    let p1 = instance_phase(1234, 1);
    assert_ne!(p0, p1, "Instance phases must differ across instance IDs");
}

