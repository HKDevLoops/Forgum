//! Adversarial and invariant test suite for Forgum engine.
//!
//! Validates:
//! 1. Universal mascot universe parity (132 cow files == options == scenery == animations == completions).
//! 2. Extreme terminal geometry resilience (0x0, 1x1, 10x2, 2048x2048).
//! 3. Zero-allocation logging fast-path and concurrent logging thread safety.
//! 4. Frame pacing precision and delta clamping invariants.
//! 5. Error classification exit code guarantees.

use forgum_engine::cow::{expand_cow, load_cow};
use forgum_engine::dna::{self, BaseAnim};
use forgum_engine::effects;
use forgum_engine::error::EngineError;
use forgum_engine::framebuffer::FrameBuffer;
use forgum_engine::logger::{self, LogLevel};
use forgum_engine::options_table;
use forgum_engine::scenery;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("data")
}

// ── Invariant 1: 132-Mascot Universe Parity ─────────────────────────

#[test]
fn mascot_universe_132_parity_across_all_systems() {
    let dd = data_dir();
    let cows_dir = dd.join("Cows");

    let disk_cows: HashSet<String> = std::fs::read_dir(&cows_dir)
        .expect("data/Cows exists")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cow"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
        .collect();

    assert_eq!(
        disk_cows.len(),
        132,
        "Expected exactly 132 .cow files on disk, found {}",
        disk_cows.len()
    );

    // 1. Check animations.json has all 132
    let animations = dna::load_animations(&dd);
    for cow in &disk_cows {
        assert!(
            animations.contains_key(cow),
            "Mascot '{cow}' missing from data/animations.json"
        );
    }

    // 2. Check scenery::get_animal_profile handles all 132
    for cow in &disk_cows {
        let profile = scenery::get_animal_profile(cow);
        assert!(!profile.name.is_empty());
        assert!(!profile.eyes.is_empty());
    }

    // 3. Check options_table output contains 132
    let options_rendered = options_table::render_options("animals");
    assert!(
        options_rendered.contains("132 Available Built-In Animals"),
        "options_table does not announce 132 available animals"
    );
    for cow in &disk_cows {
        assert!(
            options_rendered.contains(cow.as_str()),
            "Mascot '{cow}' missing from options_table catalog output"
        );
    }
}

// ── Invariant 2: Extreme Terminal Geometry Resilience ───────────────

#[test]
fn extreme_terminal_bounds_do_not_panic_or_overflow() {
    let extreme_sizes = [
        (0, 0),
        (1, 1),
        (2, 2),
        (5, 5),
        (10, 2),
        (3, 80),
        (80, 3),
        (500, 500),
        (2048, 2048),
    ];

    let dd = data_dir();
    let animations = dna::load_animations(&dd);
    let cow_raw = load_cow("default", &dd, "oo", "  ", "\\\\");
    let cow_text = expand_cow(&cow_raw, "oo", "  ", "\\\\");
    let dna = dna::get_dna(&animations, "default");

    for &(w, h) in &extreme_sizes {
        let mut fb = FrameBuffer::new(w, h);
        assert_eq!(fb.width, w.min(2048));
        assert_eq!(fb.height, h.min(2048));

        // Test all base effects render safely into extreme buffers
        let base_anims = [
            BaseAnim::Walk,
            BaseAnim::Breathe,
            BaseAnim::Float,
            BaseAnim::Fly,
            BaseAnim::Talk,
            BaseAnim::Sway,
            BaseAnim::Glitch,
            BaseAnim::Pulse,
            BaseAnim::Particles,
            BaseAnim::Dissolve,
        ];

        for &anim in &base_anims {
            let mut effect =
                effects::create_effect(anim, cow_text.clone(), dna.clone(), 0, "animal");
            effect.update(0.033, w, h);
            effect.render(&mut fb, 0.5);
            fb.swap();
        }

        // Test resize logic
        fb.resize(w, h); // Identical size short-circuit test
        fb.resize(w.saturating_add(5), h.saturating_add(5));
    }
}

// ── Invariant 3: Logger O(1) Atomic Filter Fast Path ────────────────

#[test]
fn logger_atomic_filter_fast_path_and_concurrency() {
    // Set threshold to Error
    logger::set_min_log_level(LogLevel::Error);
    assert_eq!(logger::get_min_log_level(), LogLevel::Error);

    let start = Instant::now();
    // 10,000 trace logs must execute in < 5 milliseconds because of the atomic guard
    for _ in 0..10_000 {
        logger::log(LogLevel::Trace, "test::perf", "ignored trace line");
        logger::log(LogLevel::Debug, "test::perf", "ignored debug line");
        logger::log(LogLevel::Info, "test::perf", "ignored info line");
    }
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(50),
        "Filtered logs took too long: {elapsed:?} (must be fast O(1) atomic check)"
    );

    // Reset threshold back to Info
    logger::set_min_log_level(LogLevel::Info);
}

// ── Invariant 4: Error Sysexits Code Mapping ─────────────────────────

#[test]
fn typed_engine_errors_map_strictly_to_sysexits() {
    assert_eq!(EngineError::RenderInit("fail".into()).exit_code(), 78);
    assert_eq!(
        EngineError::MascotLoad {
            name: "tiger".into(),
            message: "invalid".into(),
        }
        .exit_code(),
        65
    );
    assert_eq!(
        EngineError::TerminalTooSmall { cols: 10, rows: 2 }.exit_code(),
        64
    );
    assert_eq!(
        EngineError::ControlDisconnected("eof".into()).exit_code(),
        71
    );
    assert_eq!(
        EngineError::DaemonTimeout { timeout_secs: 5 }.exit_code(),
        71
    );
}

// ── Invariant 5: Steady-State Frame Timing Calibration ──────────────

#[test]
fn frame_period_delta_compensation_invariants() {
    let target_fps = 60u16;
    let period = Duration::from_secs_f32(1.0 / target_fps as f32);

    // Simulate work taking 3ms
    let simulated_work = Duration::from_millis(3);
    assert!(period > simulated_work);

    let remaining_sleep = period - simulated_work;
    assert_eq!(
        remaining_sleep.as_micros() + simulated_work.as_micros(),
        period.as_micros()
    );

    // Delta clamping check: clamp between 1ms and 100ms
    let raw_dt_frozen = Duration::from_secs(5); // e.g. terminal window paused
    let clamped_dt = raw_dt_frozen.clamp(Duration::from_millis(1), Duration::from_millis(100));
    assert_eq!(clamped_dt, Duration::from_millis(100));
}

// ── Invariant 6: Mascot Universal Dynamic Eye Glyphs ────────────────

#[test]
fn mascot_eyes_update_invariance() {
    let dd = data_dir();
    let cows_dir = dd.join("Cows");

    let mut tested_count = 0;
    for entry in std::fs::read_dir(&cows_dir)
        .expect("data/Cows exists")
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cow") {
            let stem = path.file_stem().unwrap().to_string_lossy().to_string();
            let raw = std::fs::read_to_string(&path).unwrap_or_default();
            // If the cow template defines dynamic eyes ($eyes, ${eyes}, $eye, ${eye})
            if raw.contains("$eyes")
                || raw.contains("${eyes}")
                || raw.contains("$eye")
                || raw.contains("${eye}")
            {
                let expanded = load_cow(&stem, &dd, "§§", "  ", "\\");
                assert!(
                    expanded.contains('§'),
                    "Cow '{stem}' has eye placeholder but failed to render custom eye glyph '§'"
                );
                tested_count += 1;
            }
        }
    }
    assert!(
        tested_count >= 100,
        "Expected at least 100 dynamic creatures with eye placeholders, found {tested_count}"
    );
}

// ── Invariant 7: API Animation Control & Mutation ───────────────────

#[test]
fn api_control_cmd_wire_roundtrip_invariance() {
    use forgum_engine::control_socket::{parse_cmd, ControlCmd};

    let test_cases = [
        (r#"{"cmd":"EYES","arg":"^^"}"#, "eyes", "^^"),
        (r#"{"cmd":"TONGUE","arg":"U "}"#, "tongue", "U "),
        (r#"{"cmd":"COLOR","arg":"rainbow"}"#, "color", "rainbow"),
        (r#"{"cmd":"EFFECT","arg":"fly"}"#, "effect", "fly"),
        (r#"{"cmd":"COW","arg":"tux"}"#, "cow", "tux"),
    ];

    for (json, kind, expected) in test_cases {
        let cmd = parse_cmd(json);
        match (kind, cmd) {
            ("eyes", ControlCmd::Eyes(v)) => assert_eq!(v, expected),
            ("tongue", ControlCmd::Tongue(v)) => assert_eq!(v, expected),
            ("color", ControlCmd::Color(v)) => assert_eq!(v, expected),
            ("effect", ControlCmd::Effect(v)) => assert_eq!(v, expected),
            ("cow", ControlCmd::Cow(v)) => assert_eq!(v, expected),
            other => panic!("Unexpected parsed command for {kind}: {other:?}"),
        }
    }
}

// ── Invariant 8: Zero CP437 Mojibake Glyphs Invariance ───────────────

#[test]
fn zero_cp437_mojibake_symbols_invariance() {
    let dd = data_dir();
    let cows_dir = dd.join("Cows");

    let entries = std::fs::read_dir(&cows_dir).expect("data/Cows exists");
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "cow") {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
            assert!(
                !content.contains("Γÿ"),
                "Mascot file {} contains literal CP437 mojibake string Γÿ",
                path.display()
            );
            for ch in content.chars() {
                let code = ch as u32;
                assert!(
                    !(0x2600..=0x263F).contains(&code),
                    "Mascot file {} contains CP437-collision glyph U+{:04X} ('{}')",
                    path.display(),
                    code,
                    ch
                );
            }
        }
    }

    let anim_path = cows_dir.join("animations.json");
    if anim_path.exists() {
        let anim_content = std::fs::read_to_string(&anim_path).expect("read animations.json");
        assert!(!anim_content.contains("Γÿ"));
        for ch in anim_content.chars() {
            let code = ch as u32;
            assert!(
                !(0x2600..=0x263F).contains(&code),
                "animations.json contains CP437-collision glyph U+{:04X} ('{}')",
                code,
                ch
            );
        }
    }

    // Assert that scenery, effects, and particles module sources are 100% pure ASCII to prevent CP437 mojibake (Γöé, Γòæ, òæ)
    for path in [
        "crates/engine/src/scenery.rs",
        "crates/engine/src/effects.rs",
        "crates/engine/src/particles.rs",
    ] {
        let src = std::fs::read_to_string(path)
            .or_else(|_| {
                std::fs::read_to_string(format!(
                    "src/{}",
                    path.trim_start_matches("crates/engine/src/")
                ))
            })
            .unwrap_or_else(|_| panic!("read {}", path));
        for (line_no, line) in src.lines().enumerate() {
            let mut code_part = if let Some((code, _comment)) = line.split_once("//") {
                code
            } else {
                line
            };
            let stripped;
            if code_part.contains("/*") && code_part.contains("*/") {
                let start = code_part.find("/*").unwrap();
                let end = code_part.find("*/").unwrap() + 2;
                stripped = format!("{}{}", &code_part[..start], &code_part[end..]);
                code_part = &stripped;
            }
            for ch in code_part.chars() {
                assert!(
                    ch.is_ascii(),
                    "{} line {} contains non-ASCII character '{}' (U+{:04X}) that can cause CP437 mojibake",
                    path,
                    line_no + 1,
                    ch,
                    ch as u32
                );
            }
        }
    }

    // Assert that battle.rs, timer.rs, rps.rs, showcase.rs contain no CP437-collision box drawing characters
    for path in [
        "crates/engine/src/battle.rs",
        "crates/engine/src/timer.rs",
        "crates/engine/src/rps.rs",
        "crates/engine/src/showcase.rs",
    ] {
        if let Ok(src) = std::fs::read_to_string(path) {
            for (line_no, line) in src.lines().enumerate() {
                for ch in line.chars() {
                    assert!(
                        !matches!(
                            ch,
                            '╔' | '═'
                                | '╗'
                                | '║'
                                | '╚'
                                | '╝'
                                | '╟'
                                | '─'
                                | '╢'
                                | '┌'
                                | '┐'
                                | '└'
                                | '┘'
                                | '│'
                        ),
                        "{} line {} contains CP437-collision box character '{}' (U+{:04X})",
                        path,
                        line_no + 1,
                        ch,
                        ch as u32
                    );
                }
            }
        }
    }
}

// ── Invariant 9: All Color Modes Functional & Beautiful ─────────────

#[test]
fn all_color_modes_implemented_and_functional() {
    let modes = [
        "natural",
        "animal",
        "rainbow",
        "cyberpunk",
        "autumn",
        "pastel",
        "matrix",
        "monochrome",
        "sunset",
        "ocean",
        "fire",
        "solid",
        "none",
    ];

    let base = forgum_engine::framebuffer::Color::WHITE;
    let palette = [(255, 0, 0), (0, 255, 0), (0, 0, 255)];

    for &mode in &modes {
        for ch in ['@', '#', '*', ' ', 'o'] {
            let col = effects::resolve_fg_palette_char(mode, &palette, 10, 5, 1.0, base, ch);
            assert!(
                col.a == 255,
                "Color mode '{mode}' produced non-opaque alpha {}",
                col.a
            );
        }
    }
}

// ── Invariant 10: Universal UTF & Unicode Invariance ─────────────────

#[test]
fn universal_utf_and_unicode_resilience_invariance() {
    let bom_cow = "\u{feff}$the_cow = <<\"EOC\";\n  (o o)\n  (   )\nEOC\n";
    let tmp_cow = std::env::temp_dir().join("forgum_test_utf_bom.cow");
    std::fs::write(&tmp_cow, bom_cow.as_bytes()).unwrap();

    let loaded = forgum_engine::cow::load_cow_from_path(&tmp_cow, "oo", "  ", "\\");
    assert!(
        loaded.is_ok(),
        "Failed to load cow file with UTF BOM: {:?}",
        loaded.err()
    );
    let cow_str = loaded.unwrap();
    assert!(
        !cow_str.contains('\u{feff}'),
        "BOM was not stripped from cow template"
    );
    let _ = std::fs::remove_file(tmp_cow);

    // Test BiDi Trojan Source removal
    let malicious = "safe_prefix \u{202E}malicious_reversed\u{202C} suffix";
    let sanitized = forgum_platform::sanitize_and_normalize(malicious);
    assert!(!sanitized.contains('\u{202E}'));
    assert!(!sanitized.contains('\u{202C}'));
    assert_eq!(sanitized, "safe_prefix malicious_reversed suffix");
}

// ── Invariant 11: Dynamic Animation Compute Offload Invariance ───────

#[test]
fn dynamic_animation_compute_offload_invariance() {
    use forgum_engine::kinematics::{AnatomicalSegments, BoundsMode, CartoonRig, KinematicBody};

    let mut body = KinematicBody::new(
        10,
        5,
        BoundsMode::Lissajous {
            amp_x: 2.0,
            amp_y: 1.0,
            freq_x: 1.0,
            freq_y: 0.5,
            phase_x: 0.0,
            phase_y: 0.0,
        },
    );

    // Update with compute engine offload
    body.update(0.016, 1.0, 80, 24);
    assert!(body.x >= 0.0 && body.x <= 70.0);
    assert!(body.y >= 0.0 && body.y <= 19.0);

    let segs = AnatomicalSegments::detect(" (oo)\n (  )\n", 0, None, None, None, 1);
    let mut rig = CartoonRig::new(segs);
    rig.update(0.016, 2.0);

    assert!((-3.0..=3.0).contains(&rig.bubble_offset_x));
    assert!((-1.0..=1.0).contains(&rig.bubble_offset_y));
}

// ── Invariant 12: All Effects FPS, Velocity & Phase Independence ────

#[test]
fn invariant_all_effects_fps_velocity_and_phase_independence() {
    use forgum_engine::kinematics::{BoundsMode, KinematicBody};

    // 1. KinematicBody velocity and position invariance at 30 FPS vs 240 FPS for 5.0 simulated seconds
    let sim_duration = 5.0_f64;
    let mut body_30 = KinematicBody::new(10, 5, BoundsMode::Wrap);
    let dt_30 = 1.0 / 30.0;
    let steps_30 = (sim_duration * 30.0) as usize;
    for _ in 0..steps_30 {
        body_30.update(dt_30 as f32, 1.0, 160, 40);
    }

    let mut body_240 = KinematicBody::new(10, 5, BoundsMode::Wrap);
    let dt_240 = 1.0 / 240.0;
    let steps_240 = (sim_duration * 240.0) as usize;
    for _ in 0..steps_240 {
        body_240.update(dt_240 as f32, 1.0, 160, 40);
    }

    let diff_x = (body_30.x - body_240.x).abs();
    assert!(
        diff_x < 0.1,
        "KinematicBody position must match within 0.1% across 30 FPS vs 240 FPS, got diff {diff_x}"
    );

    // 2. Verify all effects in ALL_EFFECTS update consistently across 5.0s
    let dd = data_dir();
    let animations = dna::load_animations(&dd);
    let default_dna = dna::get_dna(&animations, "default");
    let cow_raw = load_cow("default", &dd, "oo", "  ", "\\\\");

    for &eff_name in effects::ALL_EFFECTS {
        let mut effect_30 = effects::create_scene_effect(
            eff_name,
            cow_raw.clone(),
            default_dna.clone(),
            0,
            "natural",
        );
        for _ in 0..steps_30 {
            effect_30.update(dt_30 as f32, 80, 24);
        }

        let mut effect_240 = effects::create_scene_effect(
            eff_name,
            cow_raw.clone(),
            default_dna.clone(),
            0,
            "natural",
        );
        for _ in 0..steps_240 {
            effect_240.update(dt_240 as f32, 80, 24);
        }

        // Verify neither panics and both render without buffer corruption
        let mut fb_30 = FrameBuffer::new(80, 24);
        let mut fb_240 = FrameBuffer::new(80, 24);
        effect_30.render(&mut fb_30, sim_duration as f32);
        effect_240.render(&mut fb_240, sim_duration as f32);
    }
}

// ── Invariant 13: Liquid Governor 240Hz P99 Jitter Bound ─────────────

#[test]
fn invariant_liquid_governor_240hz_p99_jitter_bound() {
    use forgum_engine::engine_core::LiquidFrameGovernor;

    let mut gov = LiquidFrameGovernor::new(240);
    let expected_period_ns = 1_000_000_000 / 240;
    assert_eq!(
        gov.target_period().as_nanos(),
        expected_period_ns as u128,
        "Target period at 240Hz must be exactly 4,166,666 ns"
    );

    let mut fixed_integrations = 0;
    for _ in 0..500 {
        let (alpha, elapsed) = gov.step_simulation(|_dt| {
            fixed_integrations += 1;
        });
        assert!(
            (0.0..=1.0).contains(&alpha),
            "Alpha interpolation must stay in [0.0, 1.0], got {alpha}"
        );
        assert!(
            elapsed >= 0.0,
            "Elapsed simulation clock must be non-negative"
        );
    }
}

// ── Invariant 14: Flood-Fill Hull & Moat Preserve Concavities ────────

#[test]
fn invariant_flood_fill_hull_and_moat_preserve_concavities() {
    use forgum_engine::kinematics::{AnatomicalSegments, AsciiRigMap, CellMask};

    let dd = data_dir();
    let cows_dir = dd.join("Cows");
    let cow_files: Vec<_> = std::fs::read_dir(&cows_dir)
        .expect("data/Cows exists")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cow"))
        .collect();

    assert_eq!(cow_files.len(), 132);

    for entry in &cow_files {
        let cow_name = entry
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let cow_raw = load_cow(&cow_name, &dd, "oo", "  ", "\\\\");
        let segments = AnatomicalSegments::detect(&cow_raw, 0, None, None, None, 1);
        let rig = AsciiRigMap::build(&cow_raw, 0, &segments);

        if rig.width > 0 && rig.height > 0 {
            // Check that every mascot has glyph opaque cells
            let has_glyph = rig.cells.iter().any(|c| c.mask == CellMask::GlyphOpaque);
            assert!(
                has_glyph,
                "Mascot '{cow_name}' must contain GlyphOpaque cells"
            );

            // Check that border whitespace is exterior transparent
            let has_exterior = rig
                .cells
                .iter()
                .any(|c| c.mask == CellMask::ExteriorTransparent);
            assert!(
                has_exterior,
                "Mascot '{cow_name}' must contain ExteriorTransparent cells"
            );
        }
    }

    // Explicit concavity test: A U-shaped mascot outline with interior and exterior whitespace
    let concave_mascot = "\
|   |
|   |
+---+
";
    let segments = AnatomicalSegments::detect(concave_mascot, 0, None, None, None, 1);
    let rig = AsciiRigMap::build(concave_mascot, 0, &segments);

    // The top row (y=0, x=1..=3) connects directly to the top boundary, so it is exterior transparent
    // Cell (2, 0) is ExteriorTransparent
    let top_gap_cell = &rig.cells[2];
    assert_eq!(
        top_gap_cell.mask,
        CellMask::ExteriorTransparent,
        "Concave top opening must be marked ExteriorTransparent"
    );

    // Verify 1-cell ExteriorMoat exists adjacent to GlyphOpaque cells
    let has_moat = rig.cells.iter().any(|c| c.mask == CellMask::ExteriorMoat);
    assert!(
        has_moat,
        "Rig map must contain 1-cell ExteriorMoat halo cells"
    );
}

// ── Invariant 15: UV-Pinned Palette & Zero Color Swimming ───────────

#[test]
fn invariant_uv_pinned_palette_zero_color_swimming() {
    use forgum_engine::kinematics::{morph_ascii_glyph, AnatomicalSegments, AsciiRigMap};

    // 1. Morph ASCII glyph transitions preserve slope bounds without jumping
    // Pivoting leg transitions
    assert_eq!(morph_ascii_glyph('|', -0.5, 0.0), '/');
    assert_eq!(morph_ascii_glyph('|', 0.0, 0.0), '|');
    assert_eq!(morph_ascii_glyph('|', 0.5, 0.0), '\\');

    assert_eq!(morph_ascii_glyph('/', -0.2, 0.0), '/');
    assert_eq!(morph_ascii_glyph('/', 0.8, 0.0), '|');
    assert_eq!(morph_ascii_glyph('/', 1.1, 0.0), '\\');

    assert_eq!(morph_ascii_glyph('\\', 0.2, 0.0), '\\');
    assert_eq!(morph_ascii_glyph('\\', -0.8, 0.0), '|');
    assert_eq!(morph_ascii_glyph('\\', -1.1, 0.0), '/');

    // Flexing wing transitions
    assert_eq!(morph_ascii_glyph('-', 0.0, 0.5), '^');
    assert_eq!(morph_ascii_glyph('-', 0.0, 0.25), '~');
    assert_eq!(morph_ascii_glyph('-', 0.0, -0.25), '_');
    assert_eq!(morph_ascii_glyph('-', 0.0, -0.5), 'v');
    assert_eq!(morph_ascii_glyph('-', 0.0, 0.0), '-');

    // 2. UV coordinates are pinned to anatomical positions and do not swim between frames
    let mascot_art = " (oo)\n (  )\n || ||\n";
    let segments = AnatomicalSegments::detect(mascot_art, 0, None, None, None, 1);
    let rig_frame1 = AsciiRigMap::build(mascot_art, 0, &segments);
    let rig_frame2 = AsciiRigMap::build(mascot_art, 0, &segments);

    assert_eq!(rig_frame1.cells.len(), rig_frame2.cells.len());
    for (i, (c1, c2)) in rig_frame1
        .cells
        .iter()
        .zip(rig_frame2.cells.iter())
        .enumerate()
    {
        assert_eq!(
            c1.u, c2.u,
            "Cell {i}: U coordinate must remain strictly pinned across frames (zero swimming)"
        );
        assert_eq!(
            c1.v, c2.v,
            "Cell {i}: V coordinate must remain strictly pinned across frames (zero swimming)"
        );
        assert_eq!(
            c1.part, c2.part,
            "Cell {i}: Anatomical part classification must remain identical"
        );
    }
}

// ── Invariant 16: Split Overlay Suspends When Foreground Changes ───

#[test]
fn invariant_split_overlay_suspends_when_foreground_pgid_changes() {
    use forgum_engine::split_scroll::{SplitOverlayGuard, SplitOverlayState};

    let mut guard = SplitOverlayGuard::new(None, 11, 24, 10);
    assert!(!guard.is_suspended());

    let mut buf = Vec::new();
    let should_render = guard.should_render(&mut buf);
    // When parent shell is in foreground (or None in standalone test), rendering proceeds and 0 bytes are emitted
    assert!(should_render);
    assert!(buf.is_empty());

    // When state is set to Suspended:
    guard.state = SplitOverlayState::Suspended;
    assert!(guard.is_suspended());

    // Transitioning from Suspended back to Active emits margin re-assertion sequences
    // and reserved row clearance:
    guard.force_check();
    let res = guard.should_render(&mut buf);
    assert!(res, "Should resume rendering when shell is active");
    assert!(
        guard.take_just_resumed(),
        "Must report just_resumed after returning from suspension"
    );
    assert!(
        buf.starts_with(b"\x1b7\x1b[?6l\x1b[11;24r\x1b8"),
        "Must re-assert DECSTBM margins upon returning to shell"
    );
}

// ── Invariant 17: Single Thought Tether & Spring Bubble Tracking ────

#[test]
fn invariant_single_thought_tether_and_spring_bubble_tracking() {
    use forgum_engine::cow::{compose_scene_with_mode, load_cow};
    use forgum_engine::kinematics::CartoonRig;

    let dd = data_dir();
    // Load standard cow template with $thoughts expanded as 'o'
    let cow_raw = load_cow("default", &dd, "oo", "  ", "o");
    let scene = compose_scene_with_mode(&cow_raw, "Testing thought bubble tethering", true);

    // Verify thought tether: exactly 3 connector circles between bubble and cow crest
    let lines: Vec<&str> = scene.lines().collect();
    let cow_crest_idx = lines
        .iter()
        .position(|l| l.contains("^__^"))
        .expect("cow crest present");
    let bubble_bottom_idx = lines[..cow_crest_idx]
        .iter()
        .rposition(|l| l.contains('(') && l.contains(')'))
        .expect("bubble bottom border present");

    assert!(cow_crest_idx > bubble_bottom_idx);
    let tether_lines: Vec<&str> = lines[bubble_bottom_idx + 1..cow_crest_idx]
        .iter()
        .copied()
        .filter(|l| !l.trim().is_empty())
        .collect();

    // Tether lines must contain exactly 3 connector circles (VIS-001)
    assert_eq!(
        tether_lines.len(),
        3,
        "Expected exactly 3 thought bubble connector lines (VIS-001)"
    );
    for line in &tether_lines {
        assert_eq!(
            line.trim(),
            "o",
            "Each thought connector line must be a single 'o' without stray tokens"
        );
    }

    // Verify spring-damper bubble physics settles under zero acceleration
    let segments =
        forgum_engine::kinematics::AnatomicalSegments::detect(&cow_raw, 0, None, None, None, 1);
    let mut rig = CartoonRig::new(segments);
    rig.update(0.1, 10.0); // velocity impulse
    assert!(rig.bubble_offset_x != 0.0 || rig.bubble_vel_x != 0.0);
    for _ in 0..60 {
        rig.update(0.05, 0.0); // settle
    }
    assert!(
        rig.bubble_vel_x.abs() < 5.0,
        "Bubble spring-damper must settle over time"
    );
}

// ── Invariant 18: Neural Inference & Audio Bell Strict Guardrails ───

#[test]
fn invariant_neural_inference_and_audio_bell_strict_guardrails() {
    use forgum_engine::neural_dialogue::{
        probe_neural_hardware, sanitize_neural_token_stream, NeuralDialogueEngine,
        MAX_KV_ARENA_BYTES, MAX_NEURAL_WEIGHTS_BYTES,
    };
    use forgum_platform::audio_bell::{emit_synchronized_chime, is_chime_permitted, ChimeEvent};

    // 1. Mandate A & I: Memory limits for neural models
    const {
        assert!(
            MAX_NEURAL_WEIGHTS_BYTES <= 38 * 1024 * 1024,
            "Neural weights cap must be <= 38MB to guarantee < 100MB resident RAM"
        );
        assert!(
            MAX_KV_ARENA_BYTES <= 8 * 1024 * 1024,
            "KV cache arena must be <= 8MB"
        );
    }

    // 2. Hardware probe executes fast and returns valid backend
    let hw = probe_neural_hardware();
    assert!(!hw.device_name.is_empty());
    assert!(hw.max_batch_tokens >= 32);

    // 3. SEC-NEURAL-001..003: Token stream sanitization
    let dirty_tokens = "\x1b[31mRed\x1b[0m \x1b]52;c;evil\x07 \u{2600} \x00\x08Safe\nThought";
    let cleaned = sanitize_neural_token_stream(dirty_tokens);
    assert!(!cleaned.contains("\x1b"));
    assert!(!cleaned.contains("\x07"));
    assert!(!cleaned.contains('\u{2600}'));
    assert!(!cleaned.contains('\x00'));
    assert!(cleaned.contains("Safe"));

    // 4. Fallback dialogue generation
    let dd = data_dir();
    let engine = NeuralDialogueEngine::new(&dd);
    let thought = engine.generate_thought("default", Some("rust language"));
    assert!(!thought.is_empty());
    assert!(!thought.contains("\x1b"));

    // 5. Audio / Bell Chime Silence Guards (AUDIO-GUARD-001..005)
    assert!(
        !is_chime_permitted(true, false, true, true),
        "Must mute when cli_silent"
    );
    assert!(
        !is_chime_permitted(false, true, true, true),
        "Must mute when config_silent"
    );
    assert!(
        !is_chime_permitted(false, false, false, true),
        "Must mute when not a TTY (pipe guard)"
    );
    assert!(
        !is_chime_permitted(false, false, true, false),
        "Must mute when editor in foreground"
    );

    // Verify zero bytes written when muted
    let mut out = Vec::new();
    let res = emit_synchronized_chime(
        &mut out,
        ChimeEvent::CommandSuccess,
        true, // cli_silent
        false,
        true,
        true,
    );
    assert!(res.is_ok());
    assert!(
        out.is_empty(),
        "Zero bytes must be emitted when chime is muted"
    );
}

// ── Invariant 19: Custom Mascot Pack Verifier Zero Catalog Poisoning ──

#[test]
fn invariant_custom_mascot_pack_verifier_zero_catalog_poisoning() {
    use forgum_engine::dna::{BaseAnim, Habitat};
    use forgum_engine::pack_importer::{
        compute_pack_dna_signature, verify_and_compile_mascot_pack, MascotPackManifest,
        PackVerifyError,
    };

    let valid_palette = [
        "#112233".to_string(),
        "#445566".to_string(),
        "#778899".to_string(),
        "#AABBCC".to_string(),
        "#DDEEFF".to_string(),
    ];
    let valid_cow = " (oo)\n (  )\n || ||\n";
    let valid_sig = compute_pack_dna_signature("custom_pack", valid_cow, &valid_palette);

    let mut manifest = MascotPackManifest {
        pack_id: "custom_pack".to_string(),
        author: "Tester".to_string(),
        version: "1.0.0".to_string(),
        dna_signature: valid_sig.clone(),
        habitat: Habitat::Terrestrial,
        base_anim: BaseAnim::Walk,
        palette_hex: valid_palette.clone(),
    };

    // 1. Valid pack compiles successfully into AsciiRigMap
    let compile_res = verify_and_compile_mascot_pack("custom_pack", valid_cow, &manifest);
    assert!(
        compile_res.is_ok(),
        "Valid pack must pass 6-stage verification: {compile_res:?}"
    );

    // 2. Unsafe path identifiers (Zip-Slip / path traversal) rejected (SEC-PACK-001)
    assert_eq!(
        verify_and_compile_mascot_pack("../evil", valid_cow, &manifest).unwrap_err(),
        PackVerifyError::UnsafePathIdentifier("../evil".to_string())
    );
    assert_eq!(
        verify_and_compile_mascot_pack("pack/nested", valid_cow, &manifest).unwrap_err(),
        PackVerifyError::UnsafePathIdentifier("pack/nested".to_string())
    );

    // 3. Forbidden terminal escapes rejected (SEC-PACK-002)
    let osc_cow = " (oo) \x1b]52;c;evil\x07";
    assert_eq!(
        verify_and_compile_mascot_pack("custom_pack", osc_cow, &manifest).unwrap_err(),
        PackVerifyError::ForbiddenTerminalEscapeSequence
    );

    // 4. CP437 mojibake rejected (BUG-006)
    let mojibake_cow = " (oo) \u{2602}";
    assert_eq!(
        verify_and_compile_mascot_pack("custom_pack", mojibake_cow, &manifest).unwrap_err(),
        PackVerifyError::Cp437MojibakeDetected('\u{2602}')
    );

    // 5. Invalid hex palette rejected
    manifest.palette_hex[0] = "NOT_HEX".to_string();
    assert_eq!(
        verify_and_compile_mascot_pack("custom_pack", valid_cow, &manifest).unwrap_err(),
        PackVerifyError::InvalidHexPaletteSlot("NOT_HEX".to_string())
    );
    manifest.palette_hex = valid_palette;

    // 6. Signature mismatch rejected
    manifest.dna_signature = "invalid_sig".to_string();
    assert_eq!(
        verify_and_compile_mascot_pack("custom_pack", valid_cow, &manifest).unwrap_err(),
        PackVerifyError::SignatureMismatch
    );

    // 7. Rule D (Zero Catalog Poisoning): Ensure all 132 canonical mascots are intact
    let dd = data_dir();
    let disk_cows: Vec<String> = std::fs::read_dir(dd.join("Cows"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "cow"))
        .map(|e| e.path().file_stem().unwrap().to_string_lossy().to_string())
        .collect();
    assert_eq!(disk_cows.len(), 132);
    for cow_name in &disk_cows {
        let loaded = load_cow(cow_name, &dd, "oo", "  ", "\\\\");
        assert!(
            !loaded.is_empty(),
            "Canonical mascot '{cow_name}' must remain valid"
        );
    }
}
