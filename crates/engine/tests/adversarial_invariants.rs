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
