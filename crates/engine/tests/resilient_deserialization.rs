//! Integration tests for resilient deserialization and multi-tier file fallbacks.
//!
//! Validates:
//! 1. Single corrupt mascot entry recovers via CowDna::from_biome without dropping from catalog,
//!    while all other 131 mascots load authentically.
//! 2. File fallback to Cows/animations.json when animations.json is missing or has syntax errors.
//! 3. Fallback to compile-time embedded defaults when disk files have syntax errors.
//! 4. CowDna::default() and CowDna::from_biome provide biologically accurate palettes.
//! 5. Full 132-mascot catalog synthesis from biome.rs.

use forgum_engine::dna::{
    load_animations, parse_animations_json_str, synthesize_biome_catalog, BaseAnim, CowDna,
};
use forgum_platform::biome::{get_all_mascots, get_natural_hex_palette};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn test_resilient_deserialization_single_corrupt_entry_preserves_131_mascots() {
    let cows_json_path = repo_root().join("data").join("Cows").join("animations.json");
    let content = fs::read_to_string(&cows_json_path).expect("read baseline animations.json");

    let mut raw_map: serde_json::Value =
        serde_json::from_str(&content).expect("parse valid baseline JSON");

    let corrupt_target = "dragon";
    let other_mascots: Vec<String> = {
        let obj = raw_map.as_object_mut().expect("top-level JSON object");
        assert_eq!(obj.len(), 132, "Baseline catalog must have 132 mascots");
        assert!(obj.contains_key(corrupt_target));

        // Poison the target record with invalid types and unknown variants
        obj.insert(
            corrupt_target.to_string(),
            serde_json::json!({
                "base": "CompletelyInvalidAnimationVariant999",
                "speed": "super_sonic_speed_invalid_type",
                "particles": 999999,
                "amplitude": "invalid_amplitude_string"
            }),
        );

        obj.keys()
            .filter(|&k| k != corrupt_target)
            .cloned()
            .collect()
    };

    let mutated_json = serde_json::to_string_pretty(&raw_map).expect("serialize mutated JSON");

    let catalog = parse_animations_json_str(&mutated_json, None)
        .expect("Catalog parsing must not abort on corrupted record");

    // Must still contain all 132 mascots
    assert_eq!(catalog.len(), 132, "Catalog size must remain 132");

    // Verify all 131 uncorrupted mascots loaded authentically
    for mascot in &other_mascots {
        assert!(
            catalog.contains_key(mascot),
            "Mascot '{mascot}' must load successfully"
        );
        let dna = &catalog[mascot];
        assert!(dna.speed > 0.0, "Speed must be > 0 for '{mascot}'");
        assert_eq!(dna.palette.len(), 5, "Palette must have 5 colors for '{mascot}'");
    }

    // Verify corrupted mascot recovered via biological fallback from biome.rs
    let fallback = catalog.get(corrupt_target).expect("Corrupt mascot must have fallback record");
    assert_eq!(fallback.base, BaseAnim::Walk, "Corrupted mascot must fall back to BaseAnim::Walk");
    assert_eq!(fallback.speed, 1.0, "Corrupted mascot must fall back to speed 1.0");

    let expected_palette = get_natural_hex_palette(corrupt_target);
    assert_eq!(
        fallback.palette,
        expected_palette.iter().map(|&s| s.to_string()).collect::<Vec<_>>(),
        "Corrupted mascot must receive authentic biological palette from biome.rs"
    );
}

#[test]
fn test_file_fallback_to_cows_animations_on_missing_file() {
    let tmp = tempdir().expect("tempdir");
    let cows_dir = tmp.path().join("Cows");
    fs::create_dir_all(&cows_dir).expect("create Cows dir");

    let baseline = repo_root().join("data").join("Cows").join("animations.json");
    fs::copy(&baseline, cows_dir.join("animations.json")).expect("copy baseline animations.json");

    // In tmp.path(), animations.json is absent, but Cows/animations.json exists
    let catalog = load_animations(tmp.path());
    assert_eq!(
        catalog.len(),
        132,
        "Must fall back to Cows/animations.json when root animations.json is missing"
    );
    assert!(catalog.contains_key("dragon"));
    assert!(catalog.contains_key("cat2"));
    assert!(catalog.contains_key("tux"));
}

#[test]
fn test_file_fallback_to_cows_animations_on_syntax_error() {
    let tmp = tempdir().expect("tempdir");
    let cows_dir = tmp.path().join("Cows");
    fs::create_dir_all(&cows_dir).expect("create Cows dir");

    let baseline = repo_root().join("data").join("Cows").join("animations.json");
    fs::copy(&baseline, cows_dir.join("animations.json")).expect("copy baseline animations.json");

    // Write malformed JSON to root animations.json
    fs::write(
        tmp.path().join("animations.json"),
        b"{\"dragon\": { truncated json syntax error ...",
    )
    .expect("write corrupt JSON");

    let catalog = load_animations(tmp.path());
    assert_eq!(
        catalog.len(),
        132,
        "Must fall back to Cows/animations.json when root animations.json is malformed"
    );
    assert!(catalog.contains_key("dragon"));
}

#[test]
fn test_fallback_to_embedded_defaults_when_disk_syntax_error() {
    let tmp = tempdir().expect("tempdir");
    // Write syntax error to animations.json and no Cows directory
    fs::write(
        tmp.path().join("animations.json"),
        b"{\"syntax_error\": [corrupted...",
    )
    .expect("write corrupt JSON");

    let catalog = load_animations(tmp.path());
    assert_eq!(
        catalog.len(),
        132,
        "Must fall back to embedded animations.json when disk files have syntax errors"
    );
    assert!(catalog.contains_key("dragon"));
    assert!(catalog.contains_key("cat"));
    assert!(catalog.contains_key("default"));
}

#[test]
fn test_cow_dna_default_has_nonempty_biological_palette() {
    let def = CowDna::default();
    assert_eq!(def.base, BaseAnim::Walk);
    assert_eq!(def.speed, 1.0);
    assert_eq!(
        def.palette.len(),
        5,
        "CowDna::default() palette must not be empty"
    );
    let expected = get_natural_hex_palette("default");
    assert_eq!(def.palette, expected);
}

#[test]
fn test_cow_dna_from_biome_covers_all_132_mascots() {
    for mascot in get_all_mascots() {
        let dna = CowDna::from_biome(mascot.name);
        assert_eq!(dna.base, BaseAnim::Walk);
        assert_eq!(dna.speed, 1.0);
        assert_eq!(dna.palette.len(), 5);
        assert_eq!(dna.palette, mascot.natural_palette);
    }
}

#[test]
fn test_synthesize_biome_catalog_has_all_132_mascots() {
    let catalog = synthesize_biome_catalog();
    assert_eq!(catalog.len(), 132);
    for mascot in get_all_mascots() {
        assert!(catalog.contains_key(mascot.name));
        let dna = &catalog[mascot.name];
        assert_eq!(dna.palette.len(), 5);
        assert_eq!(dna.palette, mascot.natural_palette);
    }
}
