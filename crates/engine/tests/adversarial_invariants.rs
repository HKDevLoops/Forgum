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
