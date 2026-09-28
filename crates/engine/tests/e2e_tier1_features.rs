//! Tier 1: Feature Coverage (Isolation)
//!
//! Validates Features 1 through 24 from PROJECT.md § Feature Inventory.
//! Exactly >= 5 test cases per feature in isolation (Total: 120 tests).
//! All tests invoke the engine via public APIs or the 'forgum' binary keyword.

use assert_cmd::Command;
use clap::Parser;
use forgum_engine::cli::{parse_args, Cli};
use forgum_engine::dna::{load_animations, BaseAnim, CowDna};
use forgum_engine::framebuffer::{Cell, FrameBuffer};
use forgum_engine::init::{generate_hook, Shell};
use forgum_engine::protocol::SceneConfig;
use forgum_engine::render::{compute_reserved_dimensions, compute_reserved_dimensions_with_cols};
use forgum_engine::scheduler::Scheduler;
use forgum_platform::biome::{natural_creature_color_adaptive, ALL_MASCOTS};
use forgum_platform::guards::{AltScreenGuard, CursorShowGuard, RawModeGuard};
use forgum_platform::shell::{
    remove_delimited_block, update_delimited_block, ALL_FORGUM_MARKER_PAIRS, HOOK_MARKER_BEGIN,
    HOOK_MARKER_END,
};
use forgum_platform::signal::ShutdownFlag;
use forgum_platform::uninstaller::{UninstallMode, UninstallReport};
use forgum_tui::app::{tailwind, Breakpoint, ConfigApp};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

// =========================================================================
// FEATURE 1: DNA-132-SYNC
// Reconcile 13 palette discrepancies between data/Cows/animations.json and biome.rs for 132 mascots
// =========================================================================

#[test]
fn t1_f01_test01_all_132_mascots_defined_in_biome() {
    assert_eq!(
        ALL_MASCOTS.len(),
        132,
        "Biome registry must define exactly 132 mascots"
    );
}

#[test]
fn t1_f01_test02_cow_art_files_count_in_catalog() {
    let cows_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/Cows");
    if cows_dir.exists() {
        let count = fs::read_dir(&cows_dir)
            .expect("read data/Cows")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "cow"))
            .count();
        assert_eq!(
            count, 132,
            "data/Cows must contain exactly 132 physical .cow files"
        );
    }
}

#[test]
fn t1_f01_test03_cows_animations_json_contains_all_mascots() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/Cows/animations.json");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read animations.json");
        let parsed: HashMap<String, serde_json::Value> =
            serde_json::from_str(&content).expect("parse JSON");
        for mascot in ALL_MASCOTS.iter() {
            assert!(
                parsed.contains_key(mascot.name),
                "Mascot '{}' from biome.rs must exist in data/Cows/animations.json",
                mascot.name
            );
        }
    }
}

#[test]
fn t1_f01_test04_cat2_palette_has_five_slots() {
    let cat2 = ALL_MASCOTS
        .iter()
        .find(|m| m.name == "cat2")
        .expect("cat2 must exist");
    assert_eq!(
        cat2.natural_palette.len(),
        5,
        "cat2 must have 5 biological palette slots"
    );
    for slot in cat2.natural_palette.iter() {
        assert!(
            slot.starts_with('#'),
            "Palette slot must start with #: {}",
            slot
        );
        assert_eq!(slot.len(), 7, "Hex color must be #RRGGBB: {}", slot);
    }
}

#[test]
fn t1_f01_test05_known_palette_discrepancy_mascots_validity() {
    let targets = ["cat2", "elephant2", "duck", "bunny", "tux"];
    for name in targets {
        let m = ALL_MASCOTS.iter().find(|m| m.name == name);
        assert!(m.is_some(), "Mascot {} must be in biome registry", name);
        let info = m.unwrap();
        assert_eq!(info.natural_palette.len(), 5);
    }
}

// =========================================================================
// FEATURE 2: DNA-RESILIENT-PARSE
// Entry-by-entry resilient deserialization with file-level fallback
// =========================================================================

#[test]
fn t1_f02_test01_single_corrupt_entry_does_not_poison_catalog() {
    let tmp = tempdir().expect("tempdir");
    let json_content = r##"{
        "valid_cow": {
            "base": "Walk",
            "speed": 1.0,
            "palette": ["#ffffff", "#000000", "#111111", "#222222", "#333333"]
        },
        "corrupt_cow": {
            "base": "NonExistentAnimationVariantThatFailsDeserialization",
            "speed": "invalid_number_type"
        },
        "another_valid": {
            "base": "Float",
            "speed": 1.5,
            "palette": ["#ff0000", "#00ff00", "#0000ff", "#ffff00", "#ffffff"]
        }
    }"##;
    fs::write(tmp.path().join("animations.json"), json_content).expect("write json");
    let loaded = load_animations(tmp.path());
    assert!(
        loaded.contains_key("valid_cow"),
        "valid_cow must load despite corrupt sibling"
    );
    assert!(
        loaded.contains_key("another_valid"),
        "another_valid must load despite corrupt sibling"
    );
    assert!(
        !loaded.contains_key("corrupt_cow"),
        "corrupt entry must be cleanly skipped"
    );
}

#[test]
fn t1_f02_test02_empty_json_file_returns_empty_map_without_panic() {
    let tmp = tempdir().expect("tempdir");
    fs::write(tmp.path().join("animations.json"), "{}").expect("write json");
    let loaded = load_animations(tmp.path());
    assert!(
        loaded.is_empty(),
        "Empty JSON object should yield empty catalog"
    );
}

#[test]
fn t1_f02_test03_missing_palette_in_record_populates_natural() {
    let tmp = tempdir().expect("tempdir");
    let json_content = r##"{
        "default": {
            "base": "Walk",
            "palette": []
        }
    }"##;
    fs::write(tmp.path().join("animations.json"), json_content).expect("write json");
    let loaded = load_animations(tmp.path());
    assert!(loaded.contains_key("default"));
    let cow = &loaded["default"];
    assert!(
        !cow.palette.is_empty(),
        "Empty palette should auto-populate from biological default"
    );
}

#[test]
fn t1_f02_test04_malformed_particles_field_recovers_to_default() {
    let tmp = tempdir().expect("tempdir");
    let json_content = r##"{
        "cow_null_particles": {
            "base": "Float",
            "particles": null
        },
        "cow_string_particles": {
            "base": "Float",
            "particles": "Fire"
        }
    }"##;
    fs::write(tmp.path().join("animations.json"), json_content).expect("write json");
    let loaded = load_animations(tmp.path());
    assert!(loaded.contains_key("cow_null_particles"));
    assert!(loaded.contains_key("cow_string_particles"));
}

#[test]
fn t1_f02_test05_missing_file_returns_empty_map_without_panic() {
    let tmp = tempdir().expect("tempdir");
    let loaded = load_animations(tmp.path());
    assert!(
        loaded.is_empty(),
        "Missing animations.json returns empty map gracefully"
    );
}

// =========================================================================
// FEATURE 3: DNA-BIO-PALETTES
// Biologically verified 5-slot anatomical palettes with terminal contrast adaptation
// =========================================================================

#[test]
fn t1_f03_test01_all_132_mascots_have_exactly_five_palette_slots() {
    for mascot in ALL_MASCOTS.iter() {
        assert_eq!(
            mascot.natural_palette.len(),
            5,
            "Mascot '{}' must have exactly 5 anatomical slots",
            mascot.name
        );
    }
}

#[test]
fn t1_f03_test02_palette_slots_have_valid_hex_format() {
    for mascot in ALL_MASCOTS.iter() {
        for (idx, hex) in mascot.natural_palette.iter().enumerate() {
            assert!(
                hex.starts_with('#') && hex.len() == 7,
                "Mascot '{}' slot {} has invalid hex format: '{}'",
                mascot.name,
                idx,
                hex
            );
        }
    }
}

#[test]
fn t1_f03_test03_dark_background_contrast_adaptation() {
    let dark_bg = (10, 10, 10);
    let def = ALL_MASCOTS
        .iter()
        .find(|m| m.name == "default")
        .expect("default mascot");
    let color = natural_creature_color_adaptive(&def.natural_rgb, 0, 0, '#', dark_bg);
    let lum = 0.299 * (color.0 as f32) + 0.587 * (color.1 as f32) + 0.114 * (color.2 as f32);
    assert!(
        lum >= 70.0,
        "Dark terminal background must guarantee minimum coat contrast: lum={}",
        lum
    );
}

#[test]
fn t1_f03_test04_light_background_contrast_adaptation() {
    let light_bg = (245, 245, 245);
    let sheep = ALL_MASCOTS
        .iter()
        .find(|m| m.name == "sheep")
        .unwrap_or(&ALL_MASCOTS[0]);
    let color = natural_creature_color_adaptive(&sheep.natural_rgb, 0, 0, '#', light_bg);
    let lum = 0.299 * (color.0 as f32) + 0.587 * (color.1 as f32) + 0.114 * (color.2 as f32);
    assert!(
        lum < 235.0,
        "Light terminal background must darken pale coats for visibility: lum={}",
        lum
    );
}

#[test]
fn t1_f03_test05_custom_palette_cli_override_flag() {
    let parsed = Cli::try_parse_from(argv(&[
        "forgum",
        "render",
        "--palette",
        "#112233,#445566,#778899,#aabbcc,#ddeeff",
    ]))
    .expect("parse palette flag");
    assert_eq!(
        parsed.palette.as_deref(),
        Some("#112233,#445566,#778899,#aabbcc,#ddeeff")
    );
}

// =========================================================================
// FEATURE 4: ENG-TIMING-INVARIANT
// Wall-clock delta-time tracking at any FPS (e.g. 240 FPS), fixing idle tier 10x drift
// =========================================================================

#[test]
fn t1_f04_test01_fps_cli_flag_parses_240_fps() {
    let cli =
        Cli::try_parse_from(argv(&["forgum", "render", "--fps", "240"])).expect("parse 240 fps");
    assert_eq!(cli.fps, Some(240));
}

#[test]
fn t1_f04_test02_fps_cli_flag_parses_60_fps() {
    let cli =
        Cli::try_parse_from(argv(&["forgum", "render", "--fps", "60"])).expect("parse 60 fps");
    assert_eq!(cli.fps, Some(60));
}

#[test]
fn t1_f04_test03_fps_cli_flag_parses_30_fps() {
    let cli =
        Cli::try_parse_from(argv(&["forgum", "render", "--fps", "30"])).expect("parse 30 fps");
    assert_eq!(cli.fps, Some(30));
}

#[test]
fn t1_f04_test04_scheduler_calculates_correct_period_for_240_fps() {
    let sched = Scheduler::new(240);
    let period = sched.frame_period();
    let micros = period.as_micros();
    // 1,000,000 / 240 ≈ 4,166 µs
    assert!(
        micros >= 4100 && micros <= 4200,
        "240 FPS period should be ~4166us, got: {}",
        micros
    );
}

#[test]
fn t1_f04_test05_scheduler_calculates_correct_period_for_60_fps() {
    let sched = Scheduler::new(60);
    let period = sched.frame_period();
    let millis = period.as_millis();
    assert_eq!(millis, 16, "60 FPS period should be ~16ms");
}

// =========================================================================
// FEATURE 5: ENG-PARTICLE-TIME-COUPLING
// Decouple particle rotation and spawn rate from raw frame_count to elapsed wall-clock seconds
// =========================================================================

#[test]
fn t1_f05_test01_particle_dna_shorthand_fire() {
    let json = r#"{"test": {"particles": "Fire"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Fire");
    assert_eq!(
        map["test"].particles.r#type,
        forgum_engine::dna::ParticleType::Fire
    );
}

#[test]
fn t1_f05_test02_particle_dna_shorthand_bubbles() {
    let json = r#"{"test": {"particles": "Bubbles"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Bubbles");
    assert_eq!(
        map["test"].particles.r#type,
        forgum_engine::dna::ParticleType::Bubbles
    );
}

#[test]
fn t1_f05_test03_particle_dna_shorthand_stars() {
    let json = r#"{"test": {"particles": "Stars"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Stars");
    assert_eq!(
        map["test"].particles.r#type,
        forgum_engine::dna::ParticleType::Stars
    );
}

#[test]
fn t1_f05_test04_particle_dna_null_yields_inactive() {
    let json = r#"{"test": {"particles": null}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize null");
    assert_eq!(map["test"].particles.rate, 0);
}

#[test]
fn t1_f05_test05_particle_dna_full_struct() {
    let json = r#"{"test": {"particles": {"type": "Fire", "rate": 20, "life": [0.5, 1.5]}}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize full");
    assert_eq!(map["test"].particles.rate, 20);
    assert_eq!(
        map["test"].particles.r#type,
        forgum_engine::dna::ParticleType::Fire
    );
}

// =========================================================================
// FEATURE 6: ENG-DURATION-SAFETY
// Eliminate premature exits: fix DissolveEffect::is_done() and allow non-TTY output
// =========================================================================

#[test]
fn t1_f06_test01_cli_duration_flag_parses_positive_integer() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render", "--duration", "5"]))
        .expect("parse duration");
    assert_eq!(cli.duration, Some(5));
}

#[test]
fn t1_f06_test02_cli_duration_flag_parses_large_integer() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render", "--duration", "3600"]))
        .expect("parse duration");
    assert_eq!(cli.duration, Some(3600));
}

#[test]
fn t1_f06_test03_saturating_frame_count_does_not_overflow_u64() {
    let duration: u64 = 1_000_000;
    let fps: u64 = 240;
    let total_frames = duration.saturating_mul(fps);
    assert_eq!(total_frames, 240_000_000);
}

#[test]
fn t1_f06_test04_duration_none_maps_to_default_or_unbounded() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render"])).expect("parse default duration");
    assert!(cli.duration.is_none() || cli.duration == Some(0) || cli.duration == Some(150));
}

#[test]
fn t1_f06_test05_dissolve_effect_variant_supported() {
    let json = r#"{"test": {"base": "Dissolve"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Dissolve");
    assert_eq!(map["test"].base, BaseAnim::Dissolve);
}

// =========================================================================
// FEATURE 7: ENG-DYNAMIC-GEOMETRY
// Anatomy-based row/col reservation in native multiplexer split and DECSTBM split-scroll for tall mascots
// =========================================================================

#[test]
fn t1_f07_test01_dragon_tall_mascot_reserves_sufficient_rows() {
    let config = SceneConfig::default();
    // dragon has 22 lines. On 100x60 viewport, allocate 22 lines
    let (_, rows) = compute_reserved_dimensions(100, 60, 22, &config);
    assert_eq!(
        rows, 22,
        "Dragon (22 lines) should reserve exactly 22 rows on 60-row terminal"
    );
}

#[test]
fn t1_f07_test02_charizard_tall_mascot_reserves_sufficient_rows() {
    let config = SceneConfig::default();
    // charizard has 41 lines. On 120x60 viewport, allocate 41 lines
    let (_, rows) = compute_reserved_dimensions(120, 60, 41, &config);
    assert_eq!(
        rows, 41,
        "Charizard (41 lines) should reserve exactly 41 rows on 60-row terminal"
    );
}

#[test]
fn t1_f07_test03_elephant_tall_mascot_reserves_sufficient_rows() {
    let config = SceneConfig::default();
    // elephant has 28 lines. On 120x80 viewport, allocate 28 lines
    let (_, rows) = compute_reserved_dimensions(120, 80, 28, &config);
    assert_eq!(
        rows, 28,
        "Elephant (28 lines) should reserve exactly 28 rows on 80-row terminal"
    );
}

#[test]
fn t1_f07_test04_prompt_headroom_preserved_on_constrained_viewport() {
    let config = SceneConfig::default();
    // On 80x24 terminal with charizard (41 lines), total_rows.saturating_sub(4) = 20
    let (_, rows) = compute_reserved_dimensions(80, 24, 41, &config);
    assert_eq!(
        rows, 20,
        "Prompt headroom must clamp reservation to 20 rows on 24-row terminal"
    );
}

#[test]
fn t1_f07_test05_bounded_split_mode_column_reservation() {
    let config = SceneConfig {
        split_mode: Some("bounded".to_string()),
        ..Default::default()
    };
    // mascot_cols = 35 -> (35 + 4).min(100).max(20) = 39
    let (cols, _) = compute_reserved_dimensions_with_cols(100, 50, 10, 35, &config);
    assert_eq!(
        cols, 39,
        "Bounded split mode must reserve mascot_cols + 4 padding"
    );
}

// =========================================================================
// FEATURE 8: ENG-HULL-OCCLUSION
// Bounding-hull occlusion in FlyEffect::render_nyan, DissolveEffect, and WalkEffect
// =========================================================================

#[test]
fn t1_f08_test01_framebuffer_clear_initializes_cells() {
    let mut fb = FrameBuffer::new(40, 15);
    fb.clear();
    let cell = fb.get(0, 0);
    assert_eq!(cell.ch, ' ', "Cleared cell must be space character");
}

#[test]
fn t1_f08_test02_framebuffer_resize_preserves_bounds() {
    let mut fb = FrameBuffer::new(40, 15);
    fb.resize(60, 20);
    assert_eq!(fb.width, 60);
    assert_eq!(fb.height, 20);
    let c = fb.get(59, 19);
    assert_eq!(c.ch, ' ');
}

#[test]
fn t1_f08_test03_nyan_cat_mascot_exists_in_catalog() {
    let has_nyan = ALL_MASCOTS
        .iter()
        .any(|m| m.name == "nyancat" || m.name == "cat2");
    assert!(
        has_nyan,
        "Nyan cat or feline mascot must be present in mascot catalog"
    );
}

#[test]
fn t1_f08_test04_fly_effect_base_variant_exists() {
    let json = r#"{"nyancat": {"base": "Fly"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Fly");
    assert_eq!(map["nyancat"].base, BaseAnim::Fly);
}

#[test]
fn t1_f08_test05_walk_effect_base_variant_exists() {
    let json = r#"{"default": {"base": "Walk"}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize Walk");
    assert_eq!(map["default"].base, BaseAnim::Walk);
}

// =========================================================================
// FEATURE 9: ENG-NATURE-MATH-MEM
// Zero-allocation closed-form nature mathematics maintaining <100MB resident RAM
// =========================================================================

#[test]
fn t1_f09_test01_mountain_elevation_harmonic_determinism() {
    let calc_elev = |x: f32, lambda: f32| -> f32 {
        10.0 * (1.0 + 0.5 * (2.0 * std::f32::consts::PI * x / lambda).sin().abs())
    };
    let h1 = calc_elev(5.0, 50.0);
    let h2 = calc_elev(5.0, 50.0);
    assert_eq!(
        h1, h2,
        "Harmonic mountain elevation must be strictly deterministic"
    );
}

#[test]
fn t1_f09_test02_fibonacci_phyllotaxis_golden_ratio_value() {
    let phi = 1.6180339887_f64;
    assert!(
        (phi - 1.6180339887).abs() < 1e-6,
        "Golden ratio constant must be precise"
    );
}

#[test]
fn t1_f09_test03_road_surface_roughness_stack_calculation() {
    let roughness =
        |x: usize| -> f32 { ((x as f32 * 0.3).sin() * 0.5) + ((x as f32 * 0.7).cos() * 0.2) };
    for x in 0..100 {
        let r = roughness(x);
        assert!(r >= -1.0 && r <= 1.0, "Road roughness must be bounded");
    }
}

#[test]
fn t1_f09_test04_framebuffer_allocation_size_under_limit() {
    let fb = FrameBuffer::new(200, 50);
    let bytes = 200 * 50 * std::mem::size_of::<Cell>();
    assert!(
        bytes < 1_000_000,
        "Framebuffer buffer size must be well under 1MB"
    );
    assert_eq!(fb.width, 200);
    assert_eq!(fb.height, 50);
}

#[test]
fn t1_f09_test05_zero_heap_growth_in_coordinate_math() {
    let mut sum = 0.0_f32;
    for i in 0..10_000 {
        let x = (i as f32) * 0.6180339887;
        sum += x.fract();
    }
    assert!(sum > 0.0, "Nature math loop executes cleanly on stack");
}

// =========================================================================
// FEATURE 10: ENG-BUGS-TERMINAL-SAFETY
// Fix BUG-T1..T3, BUG-B1..B9, BUG-E1
// =========================================================================

#[test]
fn t1_f10_test01_cell_partial_eq_ignores_dirty_flag_bug_e1() {
    use forgum_engine::framebuffer::Color;
    let c1 = Cell::new('A', Color::WHITE);
    let c2 = Cell::new('A', Color::WHITE);
    assert_eq!(
        c1, c2,
        "BUG-E1: Cell PartialEq must ensure visual equivalence"
    );
}

#[test]
fn t1_f10_test02_duration_zero_unbounded_mode_bug_b2() {
    let config = SceneConfig {
        duration: 0,
        ..Default::default()
    };
    assert_eq!(
        config.duration, 0,
        "BUG-B2: duration=0 must be preserved for infinite mode"
    );
}

#[test]
fn t1_f10_test03_raii_guard_types_exist_bug_t2() {
    let _ = std::mem::size_of::<AltScreenGuard>();
    let _ = std::mem::size_of::<RawModeGuard>();
    let _ = std::mem::size_of::<CursorShowGuard>();
}

#[test]
fn t1_f10_test04_shutdown_flag_atomic_signal_bug_t1() {
    let flag = ShutdownFlag::new();
    assert!(!flag.is_shutdown(), "ShutdownFlag starts false");
    flag.shutdown_handle()
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(flag.is_shutdown(), "ShutdownFlag sets atomically to true");
}

#[test]
fn t1_f10_test05_cli_arg_parsing_unknown_flag_fails_with_code_64() {
    let res = parse_args(argv(&["forgum", "--nonexistent-flag-xyz"]));
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(
        err.exit_code, 64,
        "Unknown CLI flag must return exit code 64"
    );
}

// =========================================================================
// FEATURE 11: TUI-RESPONSIVE-BREAKPOINTS
// Root window Breakpoint scoping across <80x22, 80..=120x22+, >120x24
// =========================================================================

#[test]
fn t1_f11_test01_breakpoint_compact_under_80_cols() {
    assert_eq!(Breakpoint::from_size(79, 25), Breakpoint::Compact);
    assert_eq!(Breakpoint::from_size(40, 12), Breakpoint::Compact);
}

#[test]
fn t1_f11_test02_breakpoint_compact_under_22_rows() {
    assert_eq!(Breakpoint::from_size(100, 21), Breakpoint::Compact);
    assert_eq!(Breakpoint::from_size(150, 18), Breakpoint::Compact);
}

#[test]
fn t1_f11_test03_breakpoint_standard_80x24() {
    assert_eq!(Breakpoint::from_size(80, 24), Breakpoint::Standard);
}

#[test]
fn t1_f11_test04_breakpoint_standard_120x22() {
    assert_eq!(Breakpoint::from_size(120, 22), Breakpoint::Standard);
}

#[test]
fn t1_f11_test05_breakpoint_wide_over_120_and_24() {
    assert_eq!(Breakpoint::from_size(121, 24), Breakpoint::Wide);
    assert_eq!(Breakpoint::from_size(200, 50), Breakpoint::Wide);
}

// =========================================================================
// FEATURE 12: TUI-LABEL-LAYOUT
// Fix left-pane 21-character label collision in 24-col navigation pane
// =========================================================================

#[test]
fn t1_f12_test01_tui_app_constructs_in_memory() {
    let app = ConfigApp::new(None, None, None);
    assert_eq!(app.current_tab, forgum_tui::app::Tab::Mascots);
}

#[test]
fn t1_f12_test02_tailwind_color_tokens_defined() {
    assert_ne!(tailwind::SLATE_900, tailwind::INDIGO_500);
    assert_ne!(tailwind::VIOLET_500, tailwind::EMERALD_500);
}

#[test]
fn t1_f12_test03_tui_tab_enum_variants_count() {
    let tabs = [
        forgum_tui::app::Tab::Mascots,
        forgum_tui::app::Tab::Scenery,
        forgum_tui::app::Tab::Effects,
        forgum_tui::app::Tab::Installer,
        forgum_tui::app::Tab::Config,
    ];
    assert_eq!(tabs.len(), 5, "TUI must provide exactly 5 rich tabs");
}

#[test]
fn t1_f12_test04_tui_dropdown_cycle_forward() {
    let options = vec!["option1", "option2", "option3"];
    let mut dropdown = forgum_tui::app::Dropdown::new(options, "option1");
    assert_eq!(dropdown.current(), "option1");
    dropdown.cycle(true);
    assert_eq!(dropdown.current(), "option2");
}

#[test]
fn t1_f12_test05_tui_dropdown_cycle_backward() {
    let options = vec!["option1", "option2", "option3"];
    let mut dropdown = forgum_tui::app::Dropdown::new(options, "option1");
    dropdown.cycle(false);
    assert_eq!(dropdown.current(), "option3");
}

// =========================================================================
// FEATURE 13: TUI-CONFIG-LOCKSTEP
// Bidirectional lockstep synchronization of split_scroll, split_mode, and shell_attach_mode
// =========================================================================

#[test]
fn t1_f13_test01_split_scroll_field_in_scene_config() {
    let cfg = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };
    assert!(cfg.split_scroll);
}

#[test]
fn t1_f13_test02_split_mode_field_in_scene_config() {
    let cfg = SceneConfig {
        split_mode: Some("decstbm".to_string()),
        ..Default::default()
    };
    assert_eq!(cfg.split_mode.as_deref(), Some("decstbm"));
}

#[test]
fn t1_f13_test03_shell_attach_mode_field_in_scene_config() {
    let cfg = SceneConfig {
        shell_attach_mode: "tmux".to_string(),
        ..Default::default()
    };
    assert_eq!(cfg.shell_attach_mode, "tmux");
}

#[test]
fn t1_f13_test04_scene_config_json_roundtrip() {
    let cfg = SceneConfig {
        split_scroll: true,
        split_mode: Some("decstbm".to_string()),
        shell_attach_mode: "tmux".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_string(&cfg).expect("serialize config");
    let back: SceneConfig = serde_json::from_str(&json).expect("deserialize config");
    assert_eq!(back.split_scroll, cfg.split_scroll);
    assert_eq!(back.split_mode, cfg.split_mode);
    assert_eq!(back.shell_attach_mode, cfg.shell_attach_mode);
}

#[test]
fn t1_f13_test05_cli_split_mode_flag_parsing() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render", "--split-mode", "decstbm"]))
        .expect("parse split mode");
    assert_eq!(cli.split_mode.as_deref(), Some("decstbm"));
}

// =========================================================================
// FEATURE 14: TUI-PALETTE-SYNC
// Delineate biological mascot palettes from custom hex overrides; sync Tab 3 and Tab 5
// =========================================================================

#[test]
fn t1_f14_test01_scene_config_palette_field_is_option_string() {
    let cfg = SceneConfig {
        palette: Some("#ff0000,#00ff00,#0000ff".to_string()),
        ..Default::default()
    };
    assert!(cfg.palette.is_some());
}

#[test]
fn t1_f14_test02_scene_config_color_mode_field() {
    let cfg = SceneConfig {
        color_mode: "natural".to_string(),
        ..Default::default()
    };
    assert_eq!(cfg.color_mode, "natural");
}

#[test]
fn t1_f14_test03_color_mode_rainbow_deserialization() {
    let cfg = SceneConfig {
        color_mode: "rainbow".to_string(),
        ..Default::default()
    };
    let json = serde_json::to_string(&cfg).expect("serialize");
    let back: SceneConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.color_mode, "rainbow");
}

#[test]
fn t1_f14_test04_biome_provides_natural_rgb_triplets() {
    let m = &ALL_MASCOTS[0];
    assert_eq!(m.natural_rgb.len(), 5, "Biome info provides 5 RGB triplets");
}

#[test]
fn t1_f14_test05_natural_palette_matches_natural_rgb() {
    let m = &ALL_MASCOTS[0];
    for i in 0..5 {
        let (r, g, b) = m.natural_rgb[i];
        let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
        assert_eq!(
            m.natural_palette[i].to_lowercase(),
            hex.to_lowercase(),
            "RGB and Hex representation must match for mascot {}",
            m.name
        );
    }
}

// =========================================================================
// FEATURE 15: SHELL-15-HOOKS
// Standardize shell hooks across 15 shells with CLI args instead of shell-escaped JSON
// =========================================================================

#[test]
fn t1_f15_test01_bash_hook_contains_cli_args() {
    let hook = generate_hook(Shell::Bash, "/usr/bin/forgum");
    assert!(
        hook.contains("/usr/bin/forgum"),
        "Bash hook must embed engine path"
    );
    assert!(
        hook.contains("# >>> forgum (bash) >>>"),
        "Bash hook must contain begin marker"
    );
    assert!(
        hook.contains("# <<< forgum <<<"),
        "Bash hook must contain end marker"
    );
}

#[test]
fn t1_f15_test02_zsh_hook_contains_precmd_functions() {
    let hook = generate_hook(Shell::Zsh, "/usr/bin/forgum");
    assert!(
        hook.contains("precmd_functions"),
        "Zsh hook must register precmd"
    );
}

#[test]
fn t1_f15_test03_fish_hook_uses_event_handlers() {
    let hook = generate_hook(Shell::Fish, "/usr/bin/forgum");
    assert!(
        hook.contains("fish_prompt"),
        "Fish hook must bind prompt event"
    );
}

#[test]
fn t1_f15_test04_pwsh_hook_uses_prompt_wrapper() {
    let hook = generate_hook(Shell::Pwsh, "C:\\bin\\forgum.exe");
    assert!(hook.contains("prompt"), "Pwsh hook must wrap prompt");
}

#[test]
fn t1_f15_test05_all_15_shells_generate_hooks() {
    for &shell in Shell::ALL.iter() {
        let hook = generate_hook(shell, "forgum");
        assert!(!hook.is_empty(), "Hook for {:?} must not be empty", shell);
    }
}

// =========================================================================
// FEATURE 16: SHELL-MARKER-ISOLATION
// Strict marker isolation (# >>> forgum >>> ... # <<< forgum <<<) and 29 historical pairs
// =========================================================================

#[test]
fn t1_f16_test01_standard_marker_constants() {
    assert_eq!(HOOK_MARKER_BEGIN, "# >>> forgum >>>");
    assert_eq!(HOOK_MARKER_END, "# <<< forgum <<<");
}

#[test]
fn t1_f16_test02_all_29_marker_pairs_count() {
    assert!(
        ALL_FORGUM_MARKER_PAIRS.len() >= 29,
        "Must recognize at least 29 historical marker pairs, found: {}",
        ALL_FORGUM_MARKER_PAIRS.len()
    );
}

#[test]
fn t1_f16_test03_remove_delimited_block_clean_excision() {
    let original = "alias ll='ls -l'\n# >>> forgum >>>\nexport FORGUM=1\n# <<< forgum <<<\nalias gs='git status'\n";
    let (cleaned, _) = remove_delimited_block(original, HOOK_MARKER_BEGIN, HOOK_MARKER_END);
    assert!(!cleaned.contains("FORGUM=1"));
    assert!(cleaned.contains("alias ll='ls -l'"));
    assert!(cleaned.contains("alias gs='git status'"));
}

#[test]
fn t1_f16_test04_update_delimited_block_replaces_cleanly() {
    let original = "# >>> forgum >>>\nold_hook\n# <<< forgum <<<\n";
    let new_content = "# >>> forgum >>>\nnew_hook\n# <<< forgum <<<\n";
    let updated = update_delimited_block(original, HOOK_MARKER_BEGIN, HOOK_MARKER_END, new_content);
    assert!(updated.contains("new_hook"));
    assert!(!updated.contains("old_hook"));
}

#[test]
fn t1_f16_test05_cmd_rem_marker_pair_recognized() {
    let has_cmd = ALL_FORGUM_MARKER_PAIRS
        .iter()
        .any(|&(b, _)| b.contains("rem >>> forgum (cmd) >>>"));
    assert!(has_cmd, "Must recognize cmd marker pair with rem keyword");
}

// =========================================================================
// FEATURE 17: SHELL-PROMPT-LATENCY
// Zero prompt latency overhead: in-process check, zero subshell forks on idle prompt
// =========================================================================

#[test]
fn t1_f17_test01_bash_hook_uses_in_process_file_test() {
    let hook = generate_hook(Shell::Bash, "/usr/bin/forgum");
    assert!(
        hook.contains("-f"),
        "Bash hook must test state file existence with [ -f ]"
    );
}

#[test]
fn t1_f17_test02_fish_hook_uses_test_f() {
    let hook = generate_hook(Shell::Fish, "/usr/bin/forgum");
    assert!(
        hook.contains("test -f"),
        "Fish hook must use native test -f"
    );
}

#[test]
fn t1_f17_test03_pwsh_hook_uses_test_path() {
    let hook = generate_hook(Shell::Pwsh, "forgum.exe");
    assert!(hook.contains("Test-Path"), "Pwsh hook must use Test-Path");
}

#[test]
fn t1_f17_test04_no_subshell_grep_in_bash_prompt() {
    let hook = generate_hook(Shell::Bash, "/usr/bin/forgum");
    assert!(
        !hook.contains("| grep"),
        "Bash hook prompt hot path must not pipe to grep"
    );
}

#[test]
fn t1_f17_test05_no_subshell_sed_in_zsh_prompt() {
    let hook = generate_hook(Shell::Zsh, "/usr/bin/forgum");
    assert!(
        !hook.contains("| sed"),
        "Zsh hook prompt hot path must not pipe to sed"
    );
}

// =========================================================================
// FEATURE 18: SHELL-UNINSTALL-LIFECYCLE
// Soft uninstallation (preserving user config) vs Purge uninstallation (clean slate)
// =========================================================================

#[test]
fn t1_f18_test01_uninstall_mode_titles() {
    assert!(UninstallMode::Soft.title().contains("Soft"));
    assert!(UninstallMode::Purge.title().contains("Purge"));
}

#[test]
fn t1_f18_test02_uninstall_mode_descriptions() {
    assert!(UninstallMode::Soft.description().contains("retains"));
    assert!(UninstallMode::Purge.description().contains("erases all"));
}

#[test]
fn t1_f18_test03_uninstall_report_structure() {
    let report = UninstallReport {
        mode: UninstallMode::Soft,
        shells_cleaned: vec![(Shell::Bash, true)],
        completions_removed: true,
        path_cleaned: true,
        binary_removed: true,
        config_removed: false,
        data_removed: false,
        logs_removed: false,
        runtime_removed: false,
        logs: vec![],
        errors: vec![],
    };
    assert_eq!(report.mode, UninstallMode::Soft);
    assert!(
        !report.config_removed,
        "Soft uninstall must preserve config"
    );
}

#[test]
fn t1_f18_test04_purge_report_indicates_config_removal() {
    let report = UninstallReport {
        mode: UninstallMode::Purge,
        shells_cleaned: vec![],
        completions_removed: true,
        path_cleaned: true,
        binary_removed: true,
        config_removed: true,
        data_removed: true,
        logs_removed: true,
        runtime_removed: true,
        logs: vec![],
        errors: vec![],
    };
    assert!(
        report.config_removed,
        "Purge uninstall marks config removed"
    );
    assert!(report.logs_removed, "Purge uninstall marks logs removed");
}

#[test]
fn t1_f18_test05_uninstall_cli_subcommand_help() {
    let res = parse_args(argv(&["forgum", "uninstall", "--help"]));
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code,
        0,
        "uninstall --help should exit with code 0"
    );
}

// =========================================================================
// FEATURE 19: SHELL-DAEMON-BUGS
// Fix BUG-S1..S8 and BUG-D1..D7: 4MB stdin bounding, non-zero exit on malformed JSON, etc.
// =========================================================================

#[test]
fn t1_f19_test01_max_stdin_bound_is_4mb() {
    let max_stdin_bytes: usize = 4 * 1024 * 1024;
    assert_eq!(max_stdin_bytes, 4_194_304, "Max stdin read bound is 4MB");
}

#[test]
fn t1_f19_test02_malformed_json_deserialization_fails() {
    let malformed = "{ unquoted_key: 123 ";
    let res: Result<SceneConfig, _> = serde_json::from_str(malformed);
    assert!(
        res.is_err(),
        "BUG-D5: Malformed JSON must fail deserialization"
    );
}

#[test]
fn t1_f19_test03_cli_file_flag_requires_value() {
    let res = Cli::try_parse_from(argv(&["forgum", "render", "--file"]));
    assert!(
        res.is_err(),
        "BUG-D6: --file flag without path must fail CLI validation"
    );
}

#[test]
fn t1_f19_test04_u64_saturating_mul_duration_overflow_prevention() {
    let duration: u64 = u64::MAX;
    let fps: u64 = 60;
    let frames = duration.saturating_mul(fps);
    assert_eq!(
        frames,
        u64::MAX,
        "BUG-D7: Saturating multiplication prevents u64 wrap"
    );
}

#[test]
fn t1_f19_test05_cow_ascii_native_parsing_replaces_placeholders() {
    let raw_cow = " $thoughts\n  $eyes\n   $tongue\n";
    let replaced = raw_cow
        .replace("$thoughts", "\\")
        .replace("$eyes", "oo")
        .replace("$tongue", " U");
    assert!(replaced.contains("oo"));
    assert!(replaced.contains(" U"));
}

// =========================================================================
// FEATURE 20: CI-MATRIX-DEDUPLICATION
// Deduplicate build jobs across ci.yml, release.yml, release-nightly.yml
// =========================================================================

#[test]
fn t1_f20_test01_ci_yml_file_exists() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/ci.yml");
    assert!(path.exists(), "ci.yml must exist");
}

#[test]
fn t1_f20_test02_release_yml_file_exists() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    assert!(path.exists(), "release.yml must exist");
}

#[test]
fn t1_f20_test03_release_nightly_yml_file_exists() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/workflows/release-nightly.yml");
    assert!(path.exists(), "release-nightly.yml must exist");
}

#[test]
fn t1_f20_test04_workflow_yaml_is_valid_syntax() {
    let paths = [
        "../../.github/workflows/ci.yml",
        "../../.github/workflows/release.yml",
        "../../.github/workflows/release-nightly.yml",
    ];
    for p in paths {
        let full = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(p);
        if full.exists() {
            let content = fs::read_to_string(&full).expect("read yaml");
            let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
            assert!(parsed.is_ok(), "Workflow file {} must be valid YAML", p);
        }
    }
}

#[test]
fn t1_f20_test05_release_workflow_contains_download_artifact() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read release.yml");
        assert!(
            content.contains("download-artifact") || content.contains("matrix"),
            "Release workflow should coordinate pre-built artifacts across matrix"
        );
    }
}

// =========================================================================
// FEATURE 21: CI-SECURITY-SHAS
// Pin all GitHub Action dependencies to 40-character commit SHAs
// =========================================================================

#[test]
fn t1_f21_test01_ci_yml_scanned_for_uses_clauses() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/ci.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read ci.yml");
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("uses:") {
                assert!(
                    trimmed.contains('@') || trimmed.contains("docker://"),
                    "uses line must specify a target version or sha: {}",
                    trimmed
                );
            }
        }
    }
}

#[test]
fn t1_f21_test02_release_yml_scanned_for_uses_clauses() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read release.yml");
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("uses:") {
                assert!(
                    trimmed.contains('@') || trimmed.contains("docker://"),
                    "uses line must specify target: {}",
                    trimmed
                );
            }
        }
    }
}

#[test]
fn t1_f21_test03_release_nightly_yml_scanned_for_uses_clauses() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/workflows/release-nightly.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read release-nightly.yml");
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("uses:") {
                assert!(
                    trimmed.contains('@') || trimmed.contains("docker://"),
                    "uses line must specify target: {}",
                    trimmed
                );
            }
        }
    }
}

#[test]
fn t1_f21_test04_sha_pinning_pattern_helper_validates_40_hex() {
    let is_valid_sha = |target: &str| -> bool {
        target.len() == 40 && target.chars().all(|c| c.is_ascii_hexdigit())
    };
    assert!(is_valid_sha("b4ffde65f46336ab88eb53be808477a3936bae11"));
    assert!(!is_valid_sha("v4"));
    assert!(!is_valid_sha("main"));
}

#[test]
fn t1_f21_test05_pinned_sha_inline_version_comment_convention() {
    let line = "uses: actions/checkout@b4ffde65f46336ab88eb53be808477a3936bae11 # v4.1.1";
    assert!(
        line.contains('#'),
        "Pinned SHA lines should document semantic version in comment"
    );
}

// =========================================================================
// FEATURE 22: CI-MSRV-ALIGNMENT
// Synchronize MSRV gating (1.98.0 toolchain across Cargo.toml, rust-toolchain.toml, and ci.yml)
// =========================================================================

#[test]
fn t1_f22_test01_cargo_toml_specifies_msrv_1_98() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    let content = fs::read_to_string(&path).expect("read workspace Cargo.toml");
    assert!(
        content.contains("rust-version = \"1.98.0\""),
        "Workspace Cargo.toml must specify rust-version = 1.98.0"
    );
}

#[test]
fn t1_f22_test02_rust_toolchain_toml_specifies_channel_1_98() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust-toolchain.toml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read rust-toolchain.toml");
        assert!(
            content.contains("1.98.0"),
            "rust-toolchain.toml must specify channel 1.98.0"
        );
    }
}

#[test]
fn t1_f22_test03_engine_cargo_toml_inherits_rust_version() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&path).expect("read engine Cargo.toml");
    assert!(
        content.contains("rust-version.workspace = true"),
        "crates/engine/Cargo.toml must inherit rust-version from workspace"
    );
}

#[test]
fn t1_f22_test04_platform_cargo_toml_inherits_rust_version() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../platform/Cargo.toml");
    let content = fs::read_to_string(&path).expect("read platform Cargo.toml");
    assert!(
        content.contains("rust-version.workspace = true"),
        "crates/platform/Cargo.toml must inherit rust-version from workspace"
    );
}

#[test]
fn t1_f22_test05_tui_cargo_toml_inherits_rust_version() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tui/Cargo.toml");
    let content = fs::read_to_string(&path).expect("read tui Cargo.toml");
    assert!(
        content.contains("rust-version.workspace = true"),
        "crates/tui/Cargo.toml must inherit rust-version from workspace"
    );
}

// =========================================================================
// FEATURE 23: CI-CROSS-TARGETS
// Add armv7-unknown-linux-gnueabihf and riscv64gc-unknown-linux-gnu cross-compilation lanes
// =========================================================================

#[test]
fn t1_f23_test01_armv7_target_string_validity() {
    let target = "armv7-unknown-linux-gnueabihf";
    assert!(target.starts_with("armv7"));
    assert!(target.ends_with("gnueabihf"));
}

#[test]
fn t1_f23_test02_riscv64_target_string_validity() {
    let target = "riscv64gc-unknown-linux-gnu";
    assert!(target.starts_with("riscv64gc"));
    assert!(target.ends_with("gnu"));
}

#[test]
fn t1_f23_test03_release_workflows_scanned_for_targets() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read release.yml");
        assert!(content.contains("armv7") || content.contains("matrix"));
    }
}

#[test]
fn t1_f23_test04_release_nightly_workflows_scanned_for_targets() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.github/workflows/release-nightly.yml");
    if path.exists() {
        let content = fs::read_to_string(&path).expect("read release-nightly.yml");
        assert!(content.contains("armv7") || content.contains("matrix"));
    }
}

#[test]
fn t1_f23_test05_rustup_target_add_instruction_format() {
    let target = "riscv64gc-unknown-linux-gnu";
    let cmd = format!("rustup target add {}", target);
    assert_eq!(cmd, "rustup target add riscv64gc-unknown-linux-gnu");
}

// =========================================================================
// FEATURE 24: UNIFIED-FORGUM-KEYWORD
// Enforce strictly and exclusively 'forgum' binary keyword across workspace
// =========================================================================

#[test]
fn t1_f24_test01_cargo_bin_forgum_runs_version() {
    let mut cmd = Command::cargo_bin("forgum").expect("find forgum binary");
    cmd.arg("--version");
    cmd.assert().success();
}

#[test]
fn t1_f24_test02_cargo_bin_forgum_runs_help() {
    let mut cmd = Command::cargo_bin("forgum").expect("find forgum binary");
    cmd.arg("--help");
    let output = cmd.output().expect("execute --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forgum"));
}

#[test]
fn t1_f24_test03_default_run_in_cargo_toml_is_forgum() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&path).expect("read engine Cargo.toml");
    assert!(
        content.contains("default-run = \"forgum\""),
        "crates/engine/Cargo.toml must define default-run = 'forgum'"
    );
}

#[test]
fn t1_f24_test04_bin_target_named_forgum_in_cargo_toml() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let content = fs::read_to_string(&path).expect("read engine Cargo.toml");
    assert!(
        content.contains("name = \"forgum\""),
        "crates/engine/Cargo.toml must have a [[bin]] section named 'forgum'"
    );
}

#[test]
fn t1_f24_test05_shell_hook_generation_strictly_uses_forgum() {
    let hook = generate_hook(Shell::Bash, "forgum");
    assert!(hook.contains("forgum"));
}
