use forgum_engine::dna::load_animations;
use std::path::Path;

#[test]
fn prominent_cows_have_dna() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    let anims = load_animations(&data_dir);
    for cow in [
        "default",
        "dragon",
        "nyan",
        "dolphin",
        "ghost",
        "koala",
        "skeleton",
        "doge",
        "tux",
        "cat",
        "moofasa",
        "meow",
        "knight",
        "owl",
        "fox",
        "daemon",
        "hellokitty",
        "stegosaurus",
        "tortoise",
        "happy-whale",
    ] {
        assert!(anims.contains_key(cow), "missing DNA for cow: {cow}");
    }
}

#[test]
fn test_resilient_deserialization_prevents_catalog_poisoning() {
    use forgum_engine::dna::{BaseAnim, CowDna};
    use std::collections::HashMap;

    // Synthetic catalog containing valid entries alongside corrupted poison entries
    let synthetic_json = r#"{
        "valid_cow_1": {
            "base": "Walk",
            "particles": "Fire",
            "speed": 1.5
        },
        "poison_cow_corrupt_type": {
            "base": "NonExistentAnimationVariant123",
            "speed": "not_a_number"
        },
        "valid_cow_2": {
            "base": "Float",
            "particles": null,
            "speed": 0.8
        },
        "poison_cow_malformed_particles": {
            "base": "Breathe",
            "particles": 99999
        },
        "valid_cow_3": {
            "base": "Fly",
            "particles": "Bubbles"
        }
    }"#;

    let raw_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(synthetic_json).expect("Top-level map deserializes");

    let mut catalog = HashMap::new();
    for (name, val) in raw_map {
        if let Ok(dna) = serde_json::from_value::<CowDna>(val) {
            catalog.insert(name, dna);
        }
    }

    // Rule D Verification: Corrupt entries are isolated and discarded; valid entries load 100%
    assert!(
        catalog.contains_key("valid_cow_1"),
        "valid_cow_1 must be recovered"
    );
    assert!(
        catalog.contains_key("valid_cow_2"),
        "valid_cow_2 must be recovered"
    );
    assert!(
        catalog.contains_key("valid_cow_3"),
        "valid_cow_3 must be recovered"
    );

    assert_eq!(catalog.get("valid_cow_1").unwrap().base, BaseAnim::Walk);
    assert_eq!(catalog.get("valid_cow_2").unwrap().base, BaseAnim::Float);
    assert_eq!(catalog.get("valid_cow_3").unwrap().base, BaseAnim::Fly);

    assert!(
        !catalog.contains_key("poison_cow_corrupt_type"),
        "Poison record 1 must be filtered"
    );
    assert!(
        !catalog.contains_key("poison_cow_malformed_particles"),
        "Poison record 2 must be filtered"
    );
    assert_eq!(
        catalog.len(),
        3,
        "Catalog must contain exactly the 3 valid mascots"
    );
}
