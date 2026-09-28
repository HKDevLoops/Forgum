//! Empirical Challenge Suite (Generation 3) for Milestone 1: Mascot DNA & Data Resilience
//!
//! Exhaustively stress-tests:
//! 1. Empty files & directories handling across all loader APIs.
//! 2. Malformed JSON (syntax errors, truncated buffers, non-object roots, type mismatches).
//! 3. Unicode mascot names (emojis, multi-byte scripts, diacritics, RTL, symbols).
//! 4. Double extensions, trailing dots, empty strings, case variants in `get_dna`.
//! 5. Non-existent files, invalid paths, and directories.
//! 6. Boundary conditions (extreme floats, u32 limits, phase computation invariants).
//! 7. Resilient recovery of canonical mascots vs strict rejection of corrupt non-canonical keys.
//! 8. Standalone embedded catalog integrity and biological palette conformance.

use forgum_engine::dna::{
    get_dna, instance_phase, load_animations, load_animations_or_embedded,
    load_embedded_animations, parse_animations_json_str, synthesize_biome_catalog, BaseAnim,
    CowDna,
};
use forgum_platform::biome::{get_all_mascots, get_natural_hex_palette};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

// =========================================================================
// TEST 1: Empty Files, Whitespace Files, and Missing Directory Handling
// =========================================================================

#[test]
fn test_edge_case_empty_files_and_fallback_resilience() {
    // 1. Direct empty string parsing must return Err without panic
    let empty_res = parse_animations_json_str("", None);
    assert!(empty_res.is_err(), "Empty string should return serde Err");

    let whitespace_res = parse_animations_json_str("   \r\n\t   ", None);
    assert!(whitespace_res.is_err(), "Whitespace string should return serde Err");

    // 2. Directory with completely empty animations.json
    let tmp = tempdir().expect("tempdir");
    let anim_empty = tmp.path().join("animations.json");
    fs::write(&anim_empty, "").expect("write empty file");

    // load_animations finds candidate, encounters syntax error, and falls back to embedded
    let catalog = load_animations(tmp.path());
    assert_eq!(
        catalog.len(),
        132,
        "load_animations with empty animations.json must fall back to 132 embedded mascots"
    );

    // 3. Directory with empty Cows/animations.json
    let tmp2 = tempdir().expect("tempdir");
    let cows_dir = tmp2.path().join("Cows");
    fs::create_dir_all(&cows_dir).expect("create Cows dir");
    fs::write(cows_dir.join("animations.json"), "").expect("write empty Cows/animations.json");

    let catalog2 = load_animations(tmp2.path());
    assert_eq!(
        catalog2.len(),
        132,
        "load_animations with empty Cows/animations.json must fall back to 132 embedded mascots"
    );

    // 4. Directory with both files empty
    fs::write(tmp2.path().join("animations.json"), "   \n").expect("write whitespace root");
    let catalog3 = load_animations(tmp2.path());
    assert_eq!(
        catalog3.len(),
        132,
        "load_animations with both candidates invalid must fall back to 132 embedded mascots"
    );

    // 5. Completely empty directory without any files
    let tmp_empty = tempdir().expect("tempdir");
    let empty_catalog = load_animations(tmp_empty.path());
    assert!(
        empty_catalog.is_empty(),
        "load_animations on directory without candidates must return empty map"
    );

    // load_animations_or_embedded must return 132 mascots
    let embedded_catalog = load_animations_or_embedded(tmp_empty.path());
    assert_eq!(
        embedded_catalog.len(),
        132,
        "load_animations_or_embedded on empty directory must return 132 mascots"
    );
}

// =========================================================================
// TEST 2: Malformed JSON Syntax & Truncated Buffers
// =========================================================================

#[test]
fn test_edge_case_malformed_json_syntax_never_panics() {
    let truncated_or_corrupt_buffers = vec![
        r##"{"##,
        r##"}"##,
        r##"{"dragon":"##,
        r##"{"dragon": {"base":"##,
        r##"{"dragon": {"base": "Walk"##,
        r##"{"dragon": {"base": "Walk"}} extra_characters"##,
        r##"{"dragon": {"base": "Walk"}, "cat": }"##,
        r##"{"dragon": {"palette": ["#111111",]}"##, // trailing comma in array
        r##"{"dragon": {"base": "Walk",}}"##,         // trailing comma in object
        r##"[{"dragon": {}}]"##,                      // array at root
        r##""just a string""##,                       // string at root
        r##"99999"##,                                 // number at root
        r##"true"##,                                  // boolean at root
        r##"null"##,                                  // null at root
    ];

    for (i, malformed) in truncated_or_corrupt_buffers.iter().enumerate() {
        let res = std::panic::catch_unwind(|| parse_animations_json_str(malformed, None));
        assert!(
            res.is_ok(),
            "Buffer {i} panicked in parse_animations_json_str: {malformed:?}"
        );
        assert!(
            res.unwrap().is_err(),
            "Buffer {i} should cleanly fail deserialization: {malformed:?}"
        );

        let tmp = tempdir().expect("tempdir");
        fs::write(tmp.path().join("animations.json"), malformed).expect("write file");
        let load_res = std::panic::catch_unwind(|| load_animations(tmp.path()));
        assert!(
            load_res.is_ok(),
            "Buffer {i} panicked in load_animations: {malformed:?}"
        );
        assert_eq!(
            load_res.unwrap().len(),
            132,
            "Buffer {i} must fall back to 132 embedded mascots"
        );
    }

    // Valid JSON with null byte key and null value: parses without panic and drops unknown key
    let null_byte_json = r##"{"\u0000": null}"##;
    let nb_res = parse_animations_json_str(null_byte_json, None);
    assert!(nb_res.is_ok());
    assert!(
        nb_res.unwrap().is_empty(),
        "Non-canonical null byte key must be omitted"
    );
}

// =========================================================================
// TEST 3: Unicode Mascot Names (Emojis, Non-Latin, Diacritics, RTL, Symbols)
// =========================================================================

#[test]
fn test_edge_case_unicode_mascot_names_resilience() {
    let unicode_mascots = vec![
        ("🦀_rust_crab", "Float", 1.8),
        ("🐄_dairy_cow", "Walk", 1.0),
        ("龙_chinese_dragon", "Fly", 2.5),
        ("дракон_russian_dragon", "Walk", 1.1),
        ("بقرة_arabic_cow", "Breathe", 0.9),
        ("תַּנִּין_hebrew_monster", "Float", 1.2),
        ("élégant_french_cow", "Float", 1.4),
        ("münchen_bavarian_bull", "Walk", 1.3),
        ("dragon\u{200B}zero_width", "Walk", 1.0), // zero-width space
        ("mascot/with/slashes", "Walk", 1.0),
        ("mascot\\with\\backslashes", "Walk", 1.0),
    ];

    let mut json_obj = serde_json::Map::new();

    // 1. Add valid unicode entries
    for (name, base, speed) in &unicode_mascots {
        json_obj.insert(
            name.to_string(),
            serde_json::json!({
                "base": base,
                "speed": speed,
                "palette": ["#111111", "#222222", "#333333", "#444444", "#555555"]
            }),
        );
    }

    // 2. Add corrupted unicode entries
    let corrupted_unicode = vec![
        "🐮_corrupt_cow",
        "🔥_fire_beast",
        "неизвестный_corrupt",
        "תַּנִּין_corrupt",
    ];
    for bad in &corrupted_unicode {
        json_obj.insert(
            bad.to_string(),
            serde_json::json!({
                "base": "BogusNonExistentVariant",
                "speed": "invalid_speed_nan"
            }),
        );
    }

    let serialized = serde_json::to_string(&serde_json::Value::Object(json_obj)).unwrap();
    let catalog = parse_animations_json_str(&serialized, None)
        .expect("parse_animations_json_str must succeed with unicode names");

    // All valid unicode entries must be cleanly stored and retrievable
    for (name, _, speed) in &unicode_mascots {
        assert!(
            catalog.contains_key(*name),
            "Unicode mascot '{name}' must be present in catalog"
        );
        let dna = &catalog[*name];
        assert_eq!(dna.speed, *speed);
        assert_eq!(dna.palette.len(), 5);
    }

    // Corrupted unicode non-canonical entries must NOT be inserted into catalog
    for bad in &corrupted_unicode {
        assert!(
            !catalog.contains_key(*bad),
            "Corrupted non-canonical unicode entry '{bad}' must NOT be in catalog"
        );
    }

    // Test get_dna with unicode names
    for (name, _, speed) in &unicode_mascots {
        let dna = get_dna(&catalog, name);
        assert_eq!(dna.speed, *speed);
    }

    // Lookup for unregistered unicode mascot falls back to CowDna::default() without panic
    let fallback = get_dna(&catalog, "🐉_unknown_creature");
    assert_eq!(fallback, CowDna::default());
}

// =========================================================================
// TEST 4: Double Extensions, Suffix Quirks, and Case Sensitivity in get_dna
// =========================================================================

#[test]
fn test_edge_case_double_extensions_and_lookup_quirks() {
    let mut catalog = HashMap::new();

    catalog.insert(
        "dragon".to_string(),
        CowDna {
            base: BaseAnim::Fly,
            speed: 3.5,
            palette: get_natural_hex_palette("dragon").iter().map(|s| s.to_string()).collect(),
            ..CowDna::default()
        },
    );

    catalog.insert(
        "cat.cow".to_string(),
        CowDna {
            base: BaseAnim::Breathe,
            speed: 0.8,
            palette: get_natural_hex_palette("cat").iter().map(|s| s.to_string()).collect(),
            ..CowDna::default()
        },
    );

    // 1. Exact match
    assert_eq!(get_dna(&catalog, "dragon").speed, 3.5);

    // 2. Suffix match when key is without .cow
    assert_eq!(get_dna(&catalog, "dragon.cow").speed, 3.5);

    // 3. Suffix match when key is WITH .cow
    assert_eq!(get_dna(&catalog, "cat.cow").speed, 0.8);
    assert_eq!(get_dna(&catalog, "cat").speed, 0.8);

    // 4. Double extension "dragon.cow.cow" MUST NOT match "dragon"
    let double_cow = get_dna(&catalog, "dragon.cow.cow");
    assert_eq!(
        double_cow,
        CowDna::default(),
        "Double extension 'dragon.cow.cow' must fall back to CowDna::default()"
    );

    // 5. Triple extension "dragon.cow.cow.cow"
    let triple_cow = get_dna(&catalog, "dragon.cow.cow.cow");
    assert_eq!(triple_cow, CowDna::default());

    // 6. Just ".cow.cow" and ".cow"
    assert_eq!(get_dna(&catalog, ".cow.cow"), CowDna::default());
    assert_eq!(get_dna(&catalog, ".cow"), CowDna::default());

    // 7. Empty string
    assert_eq!(get_dna(&catalog, ""), CowDna::default());

    // 8. Trailing dots
    assert_eq!(get_dna(&catalog, "dragon."), CowDna::default());
    assert_eq!(get_dna(&catalog, "dragon.cow."), CowDna::default());

    // 9. Unregistered canonical mascot recovers biological palette from biome
    let elephant = get_dna(&catalog, "elephant");
    assert_eq!(elephant.base, BaseAnim::Walk);
    assert_eq!(
        elephant.palette,
        get_natural_hex_palette("elephant")
    );

    let elephant_with_cow = get_dna(&catalog, "elephant.cow");
    assert_eq!(elephant_with_cow.base, BaseAnim::Walk);
    assert_eq!(
        elephant_with_cow.palette,
        get_natural_hex_palette("elephant")
    );

    // Case-insensitivity check in biome lookup
    let elephant_upper = get_dna(&catalog, "ELEPHANT");
    assert_eq!(
        elephant_upper.palette,
        get_natural_hex_palette("elephant")
    );
    let elephant_upper_cow = get_dna(&catalog, "ELEPHANT.COW");
    let _ = elephant_upper_cow;
}

// =========================================================================
// TEST 5: Non-Existent Files, Invalid Paths, and Deep Hierarchy Handling
// =========================================================================

#[test]
fn test_edge_case_non_existent_files_and_directories() {
    let non_existent_paths = vec![
        PathBuf::from("Z:\\NonExistent_Dir_404_Path_XYZ\\SubDir"),
        PathBuf::from("/non/existent/unix/style/path/999"),
        PathBuf::from("./relative/nonexistent/sub/path"),
        PathBuf::from(""),
    ];

    for path in &non_existent_paths {
        // load_animations must return empty map without panic
        let res = std::panic::catch_unwind(|| load_animations(path));
        assert!(res.is_ok(), "load_animations panicked for path {path:?}");
        assert!(
            res.unwrap().is_empty(),
            "load_animations should return empty map for non-existent path {path:?}"
        );

        // load_animations_or_embedded must return 132 embedded mascots without panic
        let emb_res = std::panic::catch_unwind(|| load_animations_or_embedded(path));
        assert!(emb_res.is_ok(), "load_animations_or_embedded panicked for path {path:?}");
        assert_eq!(
            emb_res.unwrap().len(),
            132,
            "load_animations_or_embedded must return 132 mascots for non-existent path {path:?}"
        );
    }

    // Passing a path to a regular file instead of a directory
    let tmp = tempdir().expect("tempdir");
    let regular_file = tmp.path().join("regular_file.txt");
    fs::write(&regular_file, "just some text content").expect("write file");

    let file_as_dir_res = std::panic::catch_unwind(|| load_animations(&regular_file));
    assert!(file_as_dir_res.is_ok());
    // Since regular_file.txt contains invalid JSON syntax, it falls back to embedded 132 mascots
    assert_eq!(file_as_dir_res.unwrap().len(), 132);
}

// =========================================================================
// TEST 6: Boundary Values, Extreme Numbers, and Phase Math
// =========================================================================

#[test]
fn test_edge_case_boundary_values_and_extreme_parameters() {
    // 1. Extreme numeric values in JSON payload
    let test_json = r##"{
        "extreme_speed_zero": {
            "speed": 0.0
        },
        "extreme_speed_large": {
            "speed": 1e30
        },
        "extreme_speed_tiny": {
            "speed": 1e-30
        },
        "extreme_particles_max": {
            "particles": {
                "rate": 4294967295,
                "type": "Fire",
                "life": [0.0, 1000.0],
                "speed": [0.0, 1000.0]
            }
        },
        "extreme_phase_seed_max": {
            "phase_seed": 4294967295
        },
        "extreme_amplitude_large": {
            "amplitude": {
                "breath": 10000.0,
                "sway": -10000.0,
                "float": 0.0
            }
        },
        "extreme_glow_radius": {
            "glow": {
                "radius": 1e10
            }
        }
    }"##;

    let catalog = parse_animations_json_str(test_json, None)
        .expect("parse_animations_json_str must succeed with extreme boundary numbers");

    assert_eq!(catalog["extreme_speed_zero"].speed, 0.0);
    assert_eq!(catalog["extreme_speed_large"].speed, 1e30);
    assert_eq!(catalog["extreme_speed_tiny"].speed, 1e-30);
    assert_eq!(catalog["extreme_particles_max"].particles.rate, u32::MAX);
    assert_eq!(catalog["extreme_phase_seed_max"].phase_seed, u32::MAX);
    assert_eq!(catalog["extreme_amplitude_large"].amplitude.breath, 10000.0);
    assert_eq!(catalog["extreme_amplitude_large"].amplitude.sway, -10000.0);
    assert_eq!(catalog["extreme_glow_radius"].glow.radius, 1e10);

    // 2. instance_phase boundary tests
    let phase_tests = vec![
        (0, 0),
        (u32::MAX, 0),
        (0, u32::MAX),
        (u32::MAX, u32::MAX),
        (123456789, 987654321),
    ];

    for (seed, id) in phase_tests {
        let phase = instance_phase(seed, id);
        assert!(phase.is_finite(), "instance_phase({seed}, {id}) must be finite, got {phase}");
        assert!(phase >= 0.0, "instance_phase({seed}, {id}) must be non-negative, got {phase}");
    }

    // (u32::MAX ^ u32::MAX) == 0, so phase must be 0.0
    assert_eq!(instance_phase(u32::MAX, u32::MAX), 0.0);

    // 3. Huge palette arrays
    let mut huge_palette = Vec::new();
    for i in 0..1000 {
        huge_palette.push(format!("#{:06x}", i));
    }
    let mut map = serde_json::Map::new();
    map.insert(
        "large_palette_cow".to_string(),
        serde_json::json!({
            "palette": huge_palette
        }),
    );
    let large_json = serde_json::to_string(&serde_json::Value::Object(map)).unwrap();
    let large_cat = parse_animations_json_str(&large_json, None).unwrap();
    assert_eq!(large_cat["large_palette_cow"].palette.len(), 1000);
}

// =========================================================================
// TEST 7: Canonical Mascots Recover Authentically While Corrupt Non-Mascots Omitted
// =========================================================================

#[test]
fn test_recovery_canonical_mascots_vs_rejection_of_corrupt_non_canonical() {
    let all_mascots = get_all_mascots();
    assert_eq!(all_mascots.len(), 132);

    let mut obj = serde_json::Map::new();

    // 10 canonical mascots corrupted
    let test_canonicals: Vec<&str> = all_mascots.iter().take(10).map(|m| m.name).collect();
    for name in &test_canonicals {
        obj.insert(
            name.to_string(),
            serde_json::json!({
                "base": "TotallyCorruptedAnim",
                "speed": [1, 2, 3],
                "particles": "invalid_type",
                "palette": "should_be_array"
            }),
        );
    }

    // 10 canonical mascots valid
    let valid_canonicals: Vec<&str> = all_mascots.iter().skip(10).take(10).map(|m| m.name).collect();
    for name in &valid_canonicals {
        obj.insert(
            name.to_string(),
            serde_json::json!({
                "base": "Float",
                "speed": 2.2
            }),
        );
    }

    // 10 non-canonical synthetic mascots corrupted
    let corrupt_synthetics = [
        "mutant_alien_999", "corrupted_zombie_dog", "fake_beast",
        "poison_spider", "broken_robot", "glitch_phantom",
        "chimera_corrupt", "synthetic_demon", "fake_mascot_404", "null_entity",
    ];
    for name in &corrupt_synthetics {
        obj.insert(
            name.to_string(),
            serde_json::json!({
                "base": "BadEnum",
                "speed": null
            }),
        );
    }

    // 10 non-canonical custom mascots valid
    let valid_customs = [
        "custom_hero_1", "custom_hero_2", "custom_hero_3", "custom_hero_4", "custom_hero_5",
        "custom_hero_6", "custom_hero_7", "custom_hero_8", "custom_hero_9", "custom_hero_10",
    ];
    for name in &valid_customs {
        obj.insert(
            name.to_string(),
            serde_json::json!({
                "base": "Breathe",
                "speed": 1.5,
                "palette": ["#111111", "#222222", "#333333", "#444444", "#555555"]
            }),
        );
    }

    let payload = serde_json::to_string(&serde_json::Value::Object(obj)).unwrap();
    let catalog = parse_animations_json_str(&payload, None)
        .expect("parse_animations_json_str must succeed");

    // Expected total in catalog:
    // 10 recovered canonicals + 10 valid canonicals + 0 corrupt synthetics + 10 valid customs = 30
    assert_eq!(
        catalog.len(),
        30,
        "Catalog must contain exactly 30 entries (corrupted non-mascots cleanly omitted)"
    );

    // Verify 10 recovered canonical mascots
    for name in &test_canonicals {
        assert!(catalog.contains_key(*name), "Canonical mascot '{name}' must be recovered");
        let dna = &catalog[*name];
        assert_eq!(dna.base, BaseAnim::Walk, "Recovered mascot '{name}' must have Walk animation");
        assert_eq!(dna.speed, 1.0, "Recovered mascot '{name}' must have default speed 1.0");
        assert_eq!(
            dna.palette,
            get_natural_hex_palette(name),
            "Recovered mascot '{name}' must have authentic biological palette"
        );
    }

    // Verify 10 valid canonical mascots (omitted palette populated with biological palette)
    for name in &valid_canonicals {
        assert!(catalog.contains_key(*name), "Valid canonical '{name}' must be present");
        let dna = &catalog[*name];
        assert_eq!(dna.base, BaseAnim::Float);
        assert_eq!(dna.speed, 2.2);
        assert_eq!(
            dna.palette,
            get_natural_hex_palette(name),
            "Valid canonical '{name}' with omitted palette must receive biological palette"
        );
    }

    // Verify 10 corrupt non-canonical mascots are completely absent
    for name in &corrupt_synthetics {
        assert!(
            !catalog.contains_key(*name),
            "Corrupt synthetic '{name}' must NOT be in catalog"
        );
    }

    // Verify 10 valid customs are present
    for name in &valid_customs {
        assert!(catalog.contains_key(*name), "Valid custom '{name}' must be present");
        let dna = &catalog[*name];
        assert_eq!(dna.base, BaseAnim::Breathe);
        assert_eq!(dna.speed, 1.5);
        assert_eq!(
            dna.palette,
            vec!["#111111", "#222222", "#333333", "#444444", "#555555"]
        );
    }
}

// =========================================================================
// TEST 8: Standalone Embedded Catalog & Biological Palette Invariants
// =========================================================================

#[test]
fn test_standalone_embedded_catalog_integrity() {
    let embedded = load_embedded_animations();
    assert_eq!(embedded.len(), 132, "Embedded animations must contain all 132 mascots");

    let all_mascots = get_all_mascots();
    assert_eq!(all_mascots.len(), 132);

    for mascot in all_mascots {
        assert!(
            embedded.contains_key(mascot.name),
            "Embedded catalog missing mascot '{}'",
            mascot.name
        );
        let dna = &embedded[mascot.name];
        assert_eq!(
            dna.palette.len(),
            5,
            "Mascot '{}' must have 5 palette colors",
            mascot.name
        );
        assert_eq!(
            dna.palette,
            mascot.natural_palette,
            "Mascot '{}' in embedded catalog must match biological palette from biome.rs",
            mascot.name
        );
    }

    let synthesized = synthesize_biome_catalog();
    assert_eq!(synthesized.len(), 132);
    for mascot in all_mascots {
        assert!(synthesized.contains_key(mascot.name));
        let dna = &synthesized[mascot.name];
        assert_eq!(dna.base, BaseAnim::Walk);
        assert_eq!(dna.palette, mascot.natural_palette);
    }
}
