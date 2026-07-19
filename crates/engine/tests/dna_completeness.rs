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
