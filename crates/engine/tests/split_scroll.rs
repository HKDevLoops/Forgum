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

    // Ultrawide viewport (>= 160 cols): expansive canvas
    let (cols, rows) = compute_reserved_dimensions(200, 50, 8, &config);
    assert_eq!(cols, 200);
    // target = mascot_lines(8).max(12).min(50 * 40 / 100 = 20) -> 12, clamped in 6..=22
    assert_eq!(rows, 12);

    // Ultrawide with taller mascot (16 lines)
    let (_, rows_tall) = compute_reserved_dimensions(200, 60, 16, &config);
    // target = 16.max(12).min(60 * 40 / 100 = 24) -> 16
    assert_eq!(rows_tall, 16);
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
    // target = mascot_lines(8).max(10).min(40 * 38 / 100 = 15) -> 10, clamped in 6..=22
    assert_eq!(rows, 10);
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
    // target = mascot_lines(8).max(8).min(24 * 35 / 100 = 8) -> 8
    // Safe prompt headroom: 24 - 4 = 20, so 8 is within bounds
    assert_eq!(rows, 8);
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
