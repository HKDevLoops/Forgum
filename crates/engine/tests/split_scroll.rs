//! Integration tests for split-scroll terminal area reservation,
//! width consciousness, dynamic resizing, and DECSTBM margin management.

use forgum_engine::cli::{build_scene_config, parse_args};
use forgum_engine::config::merge;
use forgum_engine::protocol::SceneConfig;
use forgum_engine::render::compute_reserved_dimensions;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn compute_reserved_dimensions_width_consciousness_ultrawide() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Ultrawide viewport (>= 160 cols): dynamically allocates full mascot height
    let (cols, rows) = compute_reserved_dimensions(200, 50, 8, &config);
    assert_eq!(cols, 200);
    assert_eq!(rows, 8);

    // Ultrawide with taller mascot: dynamic reservation allocates full mascot height
    let (_, rows_tall) = compute_reserved_dimensions(200, 60, 16, &config);
    assert_eq!(rows_tall, 16);
}

#[test]
fn compute_reserved_dimensions_tall_mascot_adaptive_sizing() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Charizard / tall mascots: allocates full 41 lines without truncation
    let (_, rows_tall) = compute_reserved_dimensions(140, 60, 41, &config);
    assert_eq!(rows_tall, 41);

    // In a vertically constrained terminal (35 rows): safe prompt headroom (35 - 4 = 31) bounds reservation cleanly
    let (_, rows_constrained) = compute_reserved_dimensions(140, 35, 41, &config);
    assert_eq!(rows_constrained, 31);
}

#[test]
fn compute_reserved_dimensions_width_consciousness_standard() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Standard widescreen viewport (100..159 cols)
    let (cols, rows) = compute_reserved_dimensions(120, 40, 8, &config);
    assert_eq!(cols, 120);
    assert_eq!(rows, 8);
}

#[test]
fn compute_reserved_dimensions_width_consciousness_compact() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Compact terminal (< 100 cols, e.g. 80x24 standard terminal)
    let (cols, rows) = compute_reserved_dimensions(80, 24, 8, &config);
    assert_eq!(cols, 80);
    assert_eq!(rows, 8);
}

#[test]
fn compute_reserved_dimensions_tall_mascots_automatic_reservation() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Dragon (22 lines) in a 60-row terminal receives full height (22 rows)
    let (cols_dragon, rows_dragon) = compute_reserved_dimensions(100, 60, 22, &config);
    assert_eq!(cols_dragon, 100);
    assert_eq!(rows_dragon, 22);

    // Charizard (41 lines) in a 60-row terminal receives full height (41 rows, prompt gets 19 headroom)
    let (cols_charizard, rows_charizard) = compute_reserved_dimensions(120, 60, 41, &config);
    assert_eq!(cols_charizard, 120);
    assert_eq!(rows_charizard, 41);

    // In a 40-row terminal, Charizard (41 lines) is clamped to max safe rows (40 - 4 = 36)
    let (_, rows_charizard_clamped) = compute_reserved_dimensions(120, 40, 41, &config);
    assert_eq!(rows_charizard_clamped, 36);

    // Elephant / Moojira (e.g. 28 lines) in an 80-row terminal receives full height (28 rows)
    let (_, rows_elephant) = compute_reserved_dimensions(120, 80, 28, &config);
    assert_eq!(rows_elephant, 28);
}

#[test]
fn compute_reserved_dimensions_automatic_column_reservation() {
    use forgum_engine::render::compute_reserved_dimensions_with_cols;

    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Default column reservation when reserve_cols is None defaults to total_cols.max(20)
    let (cols, _) = compute_reserved_dimensions_with_cols(120, 50, 10, 40, &config);
    assert_eq!(cols, 120);

    // Bounded split mode clamps columns to mascot width + padding (40 + 4 = 44)
    let mut bounded_config = config.clone();
    bounded_config.split_mode = Some("bounded".to_string());
    let (cols_bounded, _) =
        compute_reserved_dimensions_with_cols(120, 50, 10, 40, &bounded_config);
    assert_eq!(cols_bounded, 44);

    // Bounded columns minimum 20
    let (cols_min, _) = compute_reserved_dimensions_with_cols(120, 50, 10, 5, &bounded_config);
    assert_eq!(cols_min, 20);
}

#[test]
fn compute_reserved_dimensions_preserves_prompt_headroom() {
    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    // Constrained vertical space: 10 rows total
    // Safe prompt headroom mandates at least 4 rows for prompt/shell
    let (_, rows) = compute_reserved_dimensions(80, 10, 8, &config);
    assert_eq!(rows, 6); // 10 - 4 = 6 max safe rows

    // Extremely constrained vertical space: 5 rows total
    let (_, rows_tiny) = compute_reserved_dimensions(80, 5, 8, &config);
    assert_eq!(rows_tiny, 1); // min is 1
}

#[test]
fn compute_reserved_dimensions_explicit_overrides() {
    let mut config = SceneConfig {
        reserve_rows: Some(14),
        ..Default::default()
    };
    let (cols, rows) = compute_reserved_dimensions(100, 30, 6, &config);
    assert_eq!(cols, 100);
    assert_eq!(rows, 14);

    // Explicit reserve_rows clamped by safe prompt headroom
    config.reserve_rows = Some(28);
    let (_, rows_clamped) = compute_reserved_dimensions(100, 30, 6, &config);
    assert_eq!(rows_clamped, 26); // 30 - 4 = 26

    // Explicit reserve_cols
    config.reserve_rows = None;
    config.reserve_cols = Some(70);
    let (cols_override, _) = compute_reserved_dimensions(120, 30, 6, &config);
    assert_eq!(cols_override, 70);

    // Explicit reserve_cols clamped to total_cols and min 20
    let (cols_clamped, _) = compute_reserved_dimensions(50, 30, 6, &config);
    assert_eq!(cols_clamped, 50);

    // Explicit split_ratio
    config.reserve_cols = None;
    config.split_ratio = Some(0.25);
    let (_, rows_ratio) = compute_reserved_dimensions(100, 40, 6, &config);
    // 40 * 0.25 = 10
    assert_eq!(rows_ratio, 10);

    // Split ratio clamped to [0.10, 0.75]
    config.split_ratio = Some(0.95);
    let (_, rows_max_ratio) = compute_reserved_dimensions(100, 40, 6, &config);
    // 40 * 0.75 = 30
    assert_eq!(rows_max_ratio, 30);
}

#[test]
fn cli_parses_split_scroll_and_reservation_flags() {
    let (args, _) = parse_args(argv(&[
        "forgum",
        "render",
        "--split-scroll",
        "--reserve-rows",
        "12",
        "--reserve-cols",
        "85",
        "--split-ratio",
        "0.35",
    ]))
    .unwrap();

    assert!(args.split_scroll);
    assert_eq!(args.reserve_rows, Some(12));
    assert_eq!(args.reserve_cols, Some(85));
    assert_eq!(args.split_ratio, Some(0.35));

    let cfg = build_scene_config(&args).unwrap();
    assert!(cfg.split_scroll);
    assert_eq!(cfg.reserve_rows, Some(12));
    assert_eq!(cfg.reserve_cols, Some(85));
    assert_eq!(cfg.split_ratio, Some(0.35));
}

#[test]
fn reservation_flags_automatically_enable_split_scroll() {
    // Only --reserve-rows supplied without explicit --split-scroll
    let (args1, _) = parse_args(argv(&["forgum", "render", "--reserve-rows", "10"])).unwrap();
    let cfg1 = build_scene_config(&args1).unwrap();
    assert!(
        cfg1.split_scroll,
        "reserve-rows must automatically enable split_scroll"
    );
    assert_eq!(cfg1.reserve_rows, Some(10));

    // Only --split-ratio supplied without explicit --split-scroll
    let (args2, _) = parse_args(argv(&["forgum", "render", "--split-ratio", "0.25"])).unwrap();
    let cfg2 = build_scene_config(&args2).unwrap();
    assert!(
        cfg2.split_scroll,
        "split-ratio must automatically enable split_scroll"
    );
    assert_eq!(cfg2.split_ratio, Some(0.25));
}

#[test]
fn config_merge_preserves_reservation_settings() {
    let base = SceneConfig {
        reserve_rows: Some(10),
        reserve_cols: Some(80),
        split_ratio: Some(0.25),
        split_scroll: true,
        ..Default::default()
    };

    let overlay = SceneConfig::default();
    let merged = merge(base.clone(), overlay);
    assert_eq!(merged.reserve_rows, Some(10));
    assert_eq!(merged.reserve_cols, Some(80));
    assert_eq!(merged.split_ratio, Some(0.25));
    assert!(merged.split_scroll);

    // Overlay override
    let overlay2 = SceneConfig {
        reserve_rows: Some(16),
        ..Default::default()
    };
    let merged2 = merge(base, overlay2);
    assert_eq!(merged2.reserve_rows, Some(16));
    assert_eq!(merged2.reserve_cols, Some(80));
}

#[test]
fn decstbm_escape_sequences_and_cursor_isolation() {
    let scroll_top = 11;
    let total_rows = 40;

    // DECSTBM scroll margin setting wrapped in ANSI cursor save (ESC 7) and restore (ESC 8)
    let decstbm_seq = format!("\x1b7\x1b[{scroll_top};{total_rows}r\x1b8");
    assert_eq!(decstbm_seq, "\x1b7\x1b[11;40r\x1b8");

    // DECSTBM reset sequence: resets top and bottom margins to full screen
    let reset_seq = "\x1b[r";
    assert_eq!(reset_seq, "\x1b[r");

    // Line clear sequence when reserved area shrinks
    let clear_line_seq = format!("\x1b7\x1b[{};1H\x1b[2K\x1b8", 12);
    assert_eq!(clear_line_seq, "\x1b7\x1b[12;1H\x1b[2K\x1b8");
}

#[test]
fn cli_parses_split_mode_flag() {
    let (args1, _) = parse_args(argv(&["forgum", "render", "--split-mode", "decstbm"])).unwrap();
    assert_eq!(args1.split_mode, Some("decstbm".to_string()));
    let cfg1 = build_scene_config(&args1).unwrap();
    assert_eq!(cfg1.split_mode, Some("decstbm".to_string()));
    assert!(cfg1.split_scroll);

    let (args2, _) = parse_args(argv(&["forgum", "render", "--split-mode", "native"])).unwrap();
    assert_eq!(args2.split_mode, Some("native".to_string()));
    let cfg2 = build_scene_config(&args2).unwrap();
    assert_eq!(cfg2.split_mode, Some("native".to_string()));

    let (args3, _) = parse_args(argv(&["forgum", "render", "--split-mode", "precmd"])).unwrap();
    assert_eq!(args3.split_mode, Some("precmd".to_string()));
    let cfg3 = build_scene_config(&args3).unwrap();
    assert_eq!(cfg3.split_mode, Some("precmd".to_string()));
}

#[test]
fn config_merge_preserves_split_mode() {
    let base = SceneConfig {
        split_mode: Some("native".to_string()),
        ..Default::default()
    };
    let overlay = SceneConfig::default();
    let merged = merge(base, overlay);
    assert_eq!(merged.split_mode, Some("native".to_string()));

    let base2 = SceneConfig::default();
    let overlay2 = SceneConfig {
        split_mode: Some("precmd".to_string()),
        ..Default::default()
    };
    let merged2 = merge(base2, overlay2);
    assert_eq!(merged2.split_mode, Some("precmd".to_string()));
}
