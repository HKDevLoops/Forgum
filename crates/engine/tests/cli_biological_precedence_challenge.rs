use forgum_engine::cli::{build_scene_config, parse_args};
use forgum_platform::biome;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn challenge_all_132_mascots_cli_animal_resolves_authentic_palette() {
    let mascots = biome::get_all_mascots();
    assert_eq!(mascots.len(), 132, "Expected exactly 132 mascots in biome");

    for m in mascots {
        let expected_palette = m.natural_palette.join(",");

        // 1. Test via `forgum --animal <name>`
        let (args1, _) = parse_args(argv(&["forgum", "--animal", m.name])).unwrap();
        let cfg1 = build_scene_config(&args1).unwrap();
        assert_eq!(
            cfg1.cow, m.name,
            "Mascot name mismatch for --animal {}",
            m.name
        );
        assert_eq!(
            cfg1.palette.as_deref(),
            Some(expected_palette.as_str()),
            "Biological palette mismatch for --animal {}",
            m.name
        );
        assert_eq!(
            cfg1.color_mode, "natural",
            "Color mode must be natural for --animal {}",
            m.name
        );

        // 2. Test via `forgum --cow <name>`
        let (args2, _) = parse_args(argv(&["forgum", "--cow", m.name])).unwrap();
        let cfg2 = build_scene_config(&args2).unwrap();
        assert_eq!(
            cfg2.cow, m.name,
            "Mascot name mismatch for --cow {}",
            m.name
        );
        assert_eq!(
            cfg2.palette.as_deref(),
            Some(expected_palette.as_str()),
            "Biological palette mismatch for --cow {}",
            m.name
        );

        // 3. Test via `forgum -c <name>`
        let (args3, _) = parse_args(argv(&["forgum", "-c", m.name])).unwrap();
        let cfg3 = build_scene_config(&args3).unwrap();
        assert_eq!(
            cfg3.cow, m.name,
            "Mascot name mismatch for -c {}",
            m.name
        );
        assert_eq!(
            cfg3.palette.as_deref(),
            Some(expected_palette.as_str()),
            "Biological palette mismatch for -c {}",
            m.name
        );

        // 4. Test via `forgum render --animal <name>`
        let (args4, _) = parse_args(argv(&["forgum", "render", "--animal", m.name])).unwrap();
        let cfg4 = build_scene_config(&args4).unwrap();
        assert_eq!(
            cfg4.palette.as_deref(),
            Some(expected_palette.as_str()),
            "Biological palette mismatch for render --animal {}",
            m.name
        );
    }
}

#[test]
fn challenge_explicit_palette_overrides_biological_palette() {
    let custom_palette = "#112233,#445566,#778899";

    // 1. Passing explicit --palette with --animal
    let (args, _) = parse_args(argv(&[
        "forgum",
        "--animal",
        "dragon",
        "--palette",
        custom_palette,
    ]))
    .unwrap();
    let cfg = build_scene_config(&args).unwrap();

    assert_eq!(cfg.cow, "dragon");
    assert_eq!(
        cfg.palette.as_deref(),
        Some(custom_palette),
        "Explicit --palette must strictly override biological palette"
    );
    assert_eq!(
        cfg.color_mode, "custom",
        "Color mode must switch to custom when explicit palette is provided without explicit color-mode"
    );

    // 2. Passing explicit --palette with --cow
    let (args2, _) = parse_args(argv(&[
        "forgum",
        "--cow",
        "elephant",
        "--palette",
        custom_palette,
    ]))
    .unwrap();
    let cfg2 = build_scene_config(&args2).unwrap();
    assert_eq!(cfg2.palette.as_deref(), Some(custom_palette));
    assert_eq!(cfg2.color_mode, "custom");
}

#[test]
fn challenge_color_mode_and_palette_combinations() {
    let custom_palette = "#abcdef,#fedcba";

    // Scenario 1: --palette + --color-mode rainbow -> palette preserved, color-mode stays rainbow
    let (a1, _) = parse_args(argv(&[
        "forgum",
        "--animal",
        "tux",
        "--palette",
        custom_palette,
        "--color-mode",
        "rainbow",
    ]))
    .unwrap();
    let cfg1 = build_scene_config(&a1).unwrap();
    assert_eq!(cfg1.palette.as_deref(), Some(custom_palette));
    assert_eq!(cfg1.color_mode, "rainbow");

    // Scenario 2: --color-mode rainbow without --palette
    let (a2, _) = parse_args(argv(&["forgum", "--animal", "tux", "--color-mode", "rainbow"])).unwrap();
    let cfg2 = build_scene_config(&a2).unwrap();
    assert_eq!(cfg2.color_mode, "rainbow");

    // Scenario 3: Aliases for natural color mode (animal, animal_natural, default)
    for alias in &["animal", "animal_natural", "default"] {
        let (a, _) = parse_args(argv(&["forgum", "--animal", "corgi", "--color-mode", alias])).unwrap();
        let cfg = build_scene_config(&a).unwrap();
        assert_eq!(cfg.color_mode, "natural", "Alias {} must normalize to natural", alias);
        let corgi_palette = biome::get_natural_hex_palette("corgi").join(",");
        assert_eq!(cfg.palette.as_deref(), Some(corgi_palette.as_str()));
    }

    // Scenario 4: Other color modes (solid, none)
    let (a_solid, _) = parse_args(argv(&["forgum", "--animal", "corgi", "--color-mode", "solid"])).unwrap();
    let cfg_solid = build_scene_config(&a_solid).unwrap();
    assert_eq!(cfg_solid.color_mode, "solid");

    let (a_none, _) = parse_args(argv(&["forgum", "--animal", "corgi", "--color-mode", "none"])).unwrap();
    let cfg_none = build_scene_config(&a_none).unwrap();
    assert_eq!(cfg_none.color_mode, "none");
}

#[test]
fn challenge_mascot_aliases_resolve_biological_palettes() {
    let aliases = [
        ("rabbit", "bunny"),
        ("beavis", "beavis.zen"),
        ("nyancat", "nyan"),
        ("nyan-cat", "nyan"),
        ("nyan_cat", "nyan"),
        ("shiba", "doge"),
        ("shibu", "doge"),
        ("shiba-inu", "doge"),
        ("kitten", "cat"),
        ("kittens", "cat"),
        ("cow", "default"),
        ("cowsay", "default"),
    ];

    for (alias, canonical) in aliases {
        let (a, _) = parse_args(argv(&["forgum", "--animal", alias])).unwrap();
        let cfg = build_scene_config(&a).unwrap();
        let expected = biome::get_natural_hex_palette(canonical).join(",");
        assert_eq!(
            cfg.palette.as_deref(),
            Some(expected.as_str()),
            "Alias '{}' did not resolve to canonical '{}' palette",
            alias,
            canonical
        );
    }
}

#[test]
fn challenge_config_file_and_cli_overrides_precedence() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");

    // Config file defines cow = "dragon" with custom palette
    std::fs::write(
        &cfg_path,
        r##"{"cow": "dragon", "palette": "#111111,#222222", "color_mode": "custom"}"##,
    )
    .unwrap();

    // Case 1: CLI overrides animal with `--animal tux`, no --palette passed
    // CLI animal must override cow AND adopt tux's biological palette!
    let (a1, _) = parse_args(argv(&[
        "forgum",
        "--config",
        cfg_path.to_str().unwrap(),
        "--animal",
        "tux",
    ]))
    .unwrap();
    let cfg1 = build_scene_config(&a1).unwrap();
    assert_eq!(cfg1.cow, "tux");
    let tux_palette = biome::get_natural_hex_palette("tux").join(",");
    assert_eq!(
        cfg1.palette.as_deref(),
        Some(tux_palette.as_str()),
        "Selecting a new animal on CLI must adopt its authentic biological palette"
    );

    // Case 2: CLI overrides animal with `--animal tux` AND provides `--palette`
    // Explicit CLI palette must override!
    let (a2, _) = parse_args(argv(&[
        "forgum",
        "--config",
        cfg_path.to_str().unwrap(),
        "--animal",
        "tux",
        "--palette",
        "#999999,#aaaaaa",
    ]))
    .unwrap();
    let cfg2 = build_scene_config(&a2).unwrap();
    assert_eq!(cfg2.cow, "tux");
    assert_eq!(cfg2.palette.as_deref(), Some("#999999,#aaaaaa"));
    assert_eq!(cfg2.color_mode, "custom");

    // Case 3: CLI does NOT pass --animal or --palette; inherits dragon and custom palette from config
    let (a3, _) = parse_args(argv(&[
        "forgum",
        "--config",
        cfg_path.to_str().unwrap(),
    ]))
    .unwrap();
    let cfg3 = build_scene_config(&a3).unwrap();
    assert_eq!(cfg3.cow, "dragon");
    assert_eq!(
        cfg3.palette.as_deref(),
        Some("#111111,#222222"),
        "Unmodified config palette must be preserved when no CLI animal/palette passed"
    );
}

#[test]
fn challenge_default_cow_without_arguments_resolves_biological_palette() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_cfg = tmp.path().join("empty_config.json");
    std::fs::write(&empty_cfg, "{}").unwrap();
    let (a, _) = parse_args(argv(&[
        "forgum",
        "--config",
        empty_cfg.to_str().unwrap(),
        "render",
    ]))
    .unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.cow, "default");
    assert_eq!(cfg.color_mode, "natural");
    let default_palette = biome::get_natural_hex_palette("default").join(",");
    assert_eq!(
        cfg.palette.as_deref(),
        Some(default_palette.as_str()),
        "Default cow without arguments must resolve to biological Holstein cow palette"
    );
}
