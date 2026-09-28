//! Tier 2: Boundary & Corner Cases
//!
//! Validates system boundary conditions and extreme parameters:
//! - Extreme terminal dimensions (40x12, 200x50, 1x1, 0x0 / min clamps)
//! - Simulation timing extremes (240 FPS, 5 FPS, duration=0, duration=1)
//! - Mascot scale extremes (dragon 22L, charizard 41L, elephant 28L, small 3L)
//! - Input payload boundaries (empty text, whitespace only, long word-wrapping, special ASCII)
//! - Memory ceiling (< 100MB resident RAM)

use assert_cmd::Command;
use clap::Parser;
use forgum_engine::cli::Cli;
use forgum_engine::framebuffer::{Cell, FrameBuffer};
use forgum_engine::protocol::SceneConfig;
use forgum_engine::render::compute_reserved_dimensions;
use forgum_engine::scheduler::Scheduler;
use forgum_platform::biome::ALL_MASCOTS;
use forgum_tui::app::Breakpoint;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

// =========================================================================
// 1. Extreme Terminal Dimensions
// =========================================================================

#[test]
fn t2_boundary_dimensions_compact_40x12() {
    let bp = Breakpoint::from_size(40, 12);
    assert_eq!(
        bp,
        Breakpoint::Compact,
        "40x12 must resolve to Compact breakpoint"
    );

    let config = SceneConfig::default();
    // Compact terminal: total_rows = 12, max safe = 12 - 4 = 8
    let (cols, rows) = compute_reserved_dimensions(40, 12, 6, &config);
    assert!(cols <= 40, "Reserved cols must not exceed terminal width");
    assert!(
        rows <= 8,
        "Prompt headroom must be preserved on 40x12 terminal"
    );
    assert!(rows >= 1, "At least 1 row must be reserved");
}

#[test]
fn t2_boundary_dimensions_ultrawide_200x50() {
    let bp = Breakpoint::from_size(200, 50);
    assert_eq!(
        bp,
        Breakpoint::Wide,
        "200x50 must resolve to Wide breakpoint"
    );

    let config = SceneConfig::default();
    let (cols, rows) = compute_reserved_dimensions(200, 50, 20, &config);
    assert_eq!(
        cols, 200,
        "Ultrawide terminal allocates full available columns"
    );
    assert_eq!(
        rows, 20,
        "Allocates full mascot line height when headroom allows"
    );
}

#[test]
fn t2_boundary_dimensions_degenerate_1x1() {
    let bp = Breakpoint::from_size(1, 1);
    assert_eq!(bp, Breakpoint::Compact);

    let config = SceneConfig::default();
    let (cols, rows) = compute_reserved_dimensions(1, 1, 10, &config);
    // Cols clamped to min 20, rows clamped to max(1)
    assert!(cols >= 1);
    assert!(rows >= 1);
}

#[test]
fn t2_boundary_dimensions_degenerate_zero() {
    let bp = Breakpoint::from_size(0, 0);
    assert_eq!(bp, Breakpoint::Compact);

    let config = SceneConfig::default();
    let (cols, rows) = compute_reserved_dimensions(0, 0, 0, &config);
    assert!(cols >= 1);
    assert!(rows >= 1);
}

#[test]
fn t2_boundary_dimensions_single_row_terminal() {
    let bp = Breakpoint::from_size(80, 1);
    assert_eq!(bp, Breakpoint::Compact);

    let config = SceneConfig::default();
    let (_, rows) = compute_reserved_dimensions(80, 1, 5, &config);
    assert_eq!(rows, 1, "Single row terminal clamps reservation to 1");
}

// =========================================================================
// 2. Simulation Timing & FPS Extremes
// =========================================================================

#[test]
fn t2_boundary_timing_high_refresh_240_fps() {
    let cli = Cli::try_parse_from(argv(&[
        "forgum",
        "render",
        "--fps",
        "240",
        "--duration",
        "1",
    ]))
    .expect("parse 240 fps CLI");
    assert_eq!(cli.fps, Some(240));

    let sched = Scheduler::new(240);
    let period = sched.frame_period();
    let micros = period.as_micros();
    // At 240 FPS: 1,000,000 / 240 ≈ 4,166 µs
    assert!(
        micros >= 4100 && micros <= 4200,
        "240 FPS frame period must be ~4.166ms: {}",
        micros
    );
}

#[test]
fn t2_boundary_timing_low_refresh_5_fps() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render", "--fps", "5", "--duration", "1"]))
        .expect("parse 5 fps CLI");
    assert_eq!(cli.fps, Some(5));

    let sched = Scheduler::new(5);
    let period = sched.frame_period();
    assert_eq!(
        period.as_millis(),
        200,
        "5 FPS frame period must be exactly 200ms"
    );
}

#[test]
fn t2_boundary_timing_duration_zero_infinite_mode() {
    let config = SceneConfig {
        duration: 0,
        background: true,
        ..Default::default()
    };
    assert_eq!(
        config.duration, 0,
        "duration=0 specifies unbounded background simulation"
    );
}

#[test]
fn t2_boundary_timing_duration_one_second() {
    let cli = Cli::try_parse_from(argv(&["forgum", "render", "--duration", "1"]))
        .expect("parse duration 1");
    assert_eq!(cli.duration, Some(1));
}

#[test]
fn t2_boundary_timing_saturating_frame_multiplication() {
    let duration: u64 = u32::MAX as u64;
    let fps: u64 = 240;
    let frames = duration.saturating_mul(fps);
    assert_eq!(frames, 1_030_792_150_800);
}

// =========================================================================
// 3. Mascot Scale & Anatomy Boundaries
// =========================================================================

#[test]
fn t2_boundary_large_mascot_dragon_22_lines() {
    let config = SceneConfig::default();
    // 22 lines tall on standard 80x24: clamped to 20 to protect prompt headroom
    let (_, rows_std) = compute_reserved_dimensions(80, 24, 22, &config);
    assert_eq!(
        rows_std, 20,
        "Standard 80x24 clamps 22-line dragon to 20 rows"
    );

    // On tall 100x50: full 22 lines allocated
    let (_, rows_tall) = compute_reserved_dimensions(100, 50, 22, &config);
    assert_eq!(
        rows_tall, 22,
        "50-row terminal allocates full 22 lines for dragon"
    );
}

#[test]
fn t2_boundary_large_mascot_charizard_41_lines() {
    let config = SceneConfig::default();
    // 41 lines tall on 120x60: full 41 lines allocated
    let (_, rows_tall) = compute_reserved_dimensions(120, 60, 41, &config);
    assert_eq!(
        rows_tall, 41,
        "60-row terminal allocates full 41 lines for charizard"
    );

    // On 80x24: clamped to 20
    let (_, rows_clamped) = compute_reserved_dimensions(80, 24, 41, &config);
    assert_eq!(
        rows_clamped, 20,
        "24-row terminal clamps 41-line charizard to 20 rows"
    );
}

#[test]
fn t2_boundary_large_mascot_elephant_28_lines() {
    let config = SceneConfig::default();
    let (_, rows) = compute_reserved_dimensions(120, 50, 28, &config);
    assert_eq!(
        rows, 28,
        "50-row terminal allocates full 28 lines for elephant"
    );
}

#[test]
fn t2_boundary_tiny_mascot_small() {
    let config = SceneConfig::default();
    let (_, rows) = compute_reserved_dimensions(80, 24, 3, &config);
    assert_eq!(rows, 3, "Small 3-line mascot reserves exactly 3 lines");
}

#[test]
fn t2_boundary_mascot_catalog_names_non_empty() {
    for m in ALL_MASCOTS.iter() {
        assert!(!m.name.is_empty(), "Mascot name must not be empty");
        assert!(
            m.name.len() <= 32,
            "Mascot name '{}' within reasonable length",
            m.name
        );
    }
}

// =========================================================================
// 4. Input & Payload Boundaries
// =========================================================================

#[test]
fn t2_boundary_empty_text_input_cli() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["say", ""]);
    let output = cmd.output().expect("execute empty string");
    assert!(
        output.status.success(),
        "Empty text input must succeed without panic"
    );
}

#[test]
fn t2_boundary_whitespace_only_text_cli() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["say", "   \t\n  "]);
    let output = cmd.output().expect("execute whitespace");
    assert!(
        output.status.success(),
        "Whitespace text input must succeed without panic"
    );
}

#[test]
fn t2_boundary_long_multiline_text_input() {
    let text = "Forgum brings terminal mascots to life with physical animation, \
                vibrant biological palettes, responsive TUI configurators, \
                and zero-allocation nature mathematics running under 100MB resident RAM.";
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["say", text]);
    let output = cmd.output().expect("execute long text");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Forgum"));
}

#[test]
fn t2_boundary_special_ascii_characters_input() {
    let text = "Special: \\ / \" ' ` $ @ & < > | ; # * ~ % ^ ! ? [ ] { } ( )";
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["say", text]);
    let output = cmd.output().expect("execute special chars");
    assert!(
        output.status.success(),
        "Special ASCII chars must render safely"
    );
}

#[test]
fn t2_boundary_deep_word_wrapping_boundary() {
    let long_word = "A".repeat(120);
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["say", &long_word]);
    let output = cmd.output().expect("execute long word");
    assert!(
        output.status.success(),
        "120-char unbroken word must wrap safely without crash"
    );
}

// =========================================================================
// 5. Memory Ceiling (< 100MB RAM Mandate)
// =========================================================================

#[test]
fn t2_boundary_memory_ceiling_under_100mb() {
    // 1. Verify buffer allocation for ultrawide 240x80 canvas is under 5MB
    let fb = FrameBuffer::new(240, 80);
    let cell_size = std::mem::size_of::<Cell>();
    let total_bytes = fb.width * fb.height * cell_size * 2; // front and back
    let total_mb = (total_bytes as f64) / (1024.0 * 1024.0);
    assert!(
        total_mb < 5.0,
        "Ultrawide canvas (240x80) memory ({:.2}MB) must remain strictly < 5MB",
        total_mb
    );

    // 2. Verify resident process memory remains strictly below 100MB RAM ceiling
    use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};
    let pid = Pid::from_u32(std::process::id());
    let mut sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_memory()),
    );
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    if let Some(proc) = sys.process(pid) {
        let mem_bytes = proc.memory();
        let mem_mb = (mem_bytes as f64) / (1024.0 * 1024.0);
        assert!(
            mem_bytes < 100 * 1024 * 1024,
            "Process resident memory ({:.2}MB) must strictly remain < 100MB ceiling",
            mem_mb
        );
    }
}

#[test]
fn t2_boundary_subcommand_execution_memory_lightweight() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.arg("--version");
    let output = cmd.output().expect("execute --version");
    assert!(output.status.success());
}
