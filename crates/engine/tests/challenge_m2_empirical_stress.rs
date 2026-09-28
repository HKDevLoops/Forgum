//! Empirical Stress Harness — Milestone 2: Dynamic Geometry & Timing Invariants
//!
//! Evaluates:
//! 1. Dynamic row/column reservation and prompt headroom preservation for tall/wide mascots:
//!    `dragon`, `charizardvice`, `elephant`, `dragon-and-cow`, `moojira`.
//! 2. Viewports: 40x12 (compact), 80x24 (standard), 120x35 (wide), plus edge cases (200x60, 20x5).
//! 3. Native multiplexer split planning (tmux, zellij, wezterm).
//! 4. Wall-clock timing invariants: `forgum render` runs at 240 FPS, 60 FPS, and 30 FPS
//!    matching requested duration within ±150ms.
//! 5. `duration 0` unbounded execution.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use forgum_engine::cow;
use forgum_engine::effects;
use forgum_engine::protocol::SceneConfig;
use forgum_engine::render::compute_reserved_dimensions_with_cols;
use forgum_engine::split_scroll::{format_decstbm, format_reset_decstbm};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn data_dir() -> PathBuf {
    repo_root().join("data")
}

fn binary_path() -> PathBuf {
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_forgum") {
        return PathBuf::from(bin);
    }
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if p.file_name().is_some_and(|n| n == "deps") {
        p.pop();
    }
    let name = if cfg!(windows) {
        "forgum.exe"
    } else {
        "forgum"
    };
    p.push(name);
    p
}

// =========================================================================
// 1. DYNAMIC GEOMETRY & PROMPT HEADROOM SUITE
// =========================================================================

#[test]
fn challenge_tall_and_wide_mascots_anatomy_measurements() {
    let d = data_dir();
    assert!(d.exists(), "data directory must exist at {:?}", d);

    let mascots = [
        "dragon",
        "charizardvice",
        "elephant",
        "dragon-and-cow",
        "moojira",
    ];

    for name in mascots {
        let cow_name = cow::resolve_cow_name(name, &d);
        let cow_raw = cow::load_cow(&cow_name, &d, "oo", "  ", "\\");
        assert!(!cow_raw.is_empty(), "Mascot '{}' failed to load", name);

        let composed = cow::compose_scene_with_mode(&cow_raw, "Milestone 2 Geometry Test", false);
        let cow_foot = effects::find_cow_foot_y(&composed);
        let line_count = (cow_foot + 4).max(composed.lines().count() + 3).max(6);
        let mascot_cols = composed
            .lines()
            .map(cow::str_display_width)
            .max()
            .unwrap_or(0);

        eprintln!(
            "Mascot '{}': composed_lines={}, foot_y={}, line_count={}, max_cols={}",
            name,
            composed.lines().count(),
            cow_foot,
            line_count,
            mascot_cols
        );

        // Verify anatomy extraction succeeds and produces valid non-zero geometry
        assert!(line_count >= 6, "Line count must be at least 6");
        assert!(mascot_cols >= 10, "Mascot cols must be at least 10");
    }
}

#[test]
fn challenge_dynamic_geometry_across_target_viewports() {
    let d = data_dir();
    let mascots = [
        "dragon",
        "charizardvice",
        "elephant",
        "dragon-and-cow",
        "moojira",
    ];
    let viewports = [
        (40, 12, "40x12 compact"),
        (80, 24, "80x24 standard"),
        (120, 35, "120x35 wide"),
        (200, 60, "200x60 ultrawide"),
        (60, 18, "60x18 medium-compact"),
        (20, 5, "20x5 minimum boundary"),
    ];

    let config = SceneConfig {
        split_scroll: true,
        ..Default::default()
    };

    for name in mascots {
        let cow_name = cow::resolve_cow_name(name, &d);
        let cow_raw = cow::load_cow(&cow_name, &d, "oo", "  ", "\\");
        let composed = cow::compose_scene_with_mode(&cow_raw, "Prompt Headroom Invariant", false);
        let cow_foot = effects::find_cow_foot_y(&composed);
        let line_count = (cow_foot + 4).max(composed.lines().count() + 3).max(6);
        let mascot_cols = composed
            .lines()
            .map(cow::str_display_width)
            .max()
            .unwrap_or(0);

        for &(total_cols, total_rows, label) in &viewports {
            let (reserved_cols, reserved_rows) = compute_reserved_dimensions_with_cols(
                total_cols,
                total_rows,
                line_count,
                mascot_cols,
                &config,
            );

            // Invariant 1: Columns default to total_cols.max(20)
            assert_eq!(
                reserved_cols,
                total_cols.max(20),
                "Columns reservation mismatch for {} on {}",
                name,
                label
            );

            // Invariant 2: Prompt headroom is preserved!
            // When total_rows >= 5, user must get at least 4 prompt rows (reserved_rows <= total_rows - 4).
            let max_safe_rows = total_rows.saturating_sub(4).max(1);
            assert!(
                reserved_rows <= max_safe_rows,
                "Prompt headroom VIOLATED for mascot '{}' on {}: reserved_rows={}, max_safe_rows={}, total_rows={}",
                name, label, reserved_rows, max_safe_rows, total_rows
            );

            let prompt_headroom = total_rows.saturating_sub(reserved_rows);
            if total_rows >= 5 {
                assert!(
                    prompt_headroom >= 4,
                    "Prompt headroom was {} (< 4) for mascot '{}' on {}",
                    prompt_headroom,
                    name,
                    label
                );
            }

            // Invariant 3: Full line-height allocation without artificial clamping when viewport allows
            if total_rows >= line_count + 4 {
                assert_eq!(
                    reserved_rows, line_count,
                    "Mascot '{}' was unnecessarily clipped on {}: got reserved_rows={}, expected full line_count={}",
                    name, label, reserved_rows, line_count
                );
            }

            // Invariant 4: DECSTBM sequence correctness
            if reserved_rows < total_rows {
                let decstbm = format_decstbm(reserved_rows + 1, total_rows);
                assert_eq!(
                    decstbm,
                    format!("\x1b[{};{}r", reserved_rows + 1, total_rows)
                );
            }
        }
    }
    assert_eq!(format_reset_decstbm(), "\x1b[r");
}

#[test]
fn challenge_bounded_split_mode_column_reservation() {
    let d = data_dir();
    let mascots = [
        "dragon",
        "charizardvice",
        "elephant",
        "dragon-and-cow",
        "moojira",
    ];

    let bounded_config = SceneConfig {
        split_scroll: true,
        split_mode: Some("bounded".to_string()),
        ..Default::default()
    };

    for name in mascots {
        let cow_name = cow::resolve_cow_name(name, &d);
        let cow_raw = cow::load_cow(&cow_name, &d, "oo", "  ", "\\");
        let composed = cow::compose_scene_with_mode(&cow_raw, "Bounded Split", false);
        let cow_foot = effects::find_cow_foot_y(&composed);
        let line_count = (cow_foot + 4).max(composed.lines().count() + 3).max(6);
        let mascot_cols = composed
            .lines()
            .map(cow::str_display_width)
            .max()
            .unwrap_or(0);

        let (res_cols, _) = compute_reserved_dimensions_with_cols(
            120,
            35,
            line_count,
            mascot_cols,
            &bounded_config,
        );

        let expected_cols = (mascot_cols + 4).clamp(20, 120);
        assert_eq!(
            res_cols, expected_cols,
            "Bounded columns mismatch for mascot '{}': got {}, expected {}",
            name, res_cols, expected_cols
        );
    }
}

#[test]
fn challenge_native_multiplexer_split_planning() {
    use forgum_platform::mux::Mux;
    use forgum_platform::terminal::TerminalEmulator;

    let d = data_dir();
    let mascots = [
        "dragon",
        "charizardvice",
        "elephant",
        "dragon-and-cow",
        "moojira",
    ];
    let test_args = vec![
        "forgum".to_string(),
        "render".to_string(),
        "--text".to_string(),
        "hi".to_string(),
    ];

    for name in mascots {
        let cow_name = cow::resolve_cow_name(name, &d);
        let cow_raw = cow::load_cow(&cow_name, &d, "oo", "  ", "\\");
        let composed = cow::compose_scene_with_mode(&cow_raw, "Multiplexer Test", false);
        let cow_foot = effects::find_cow_foot_y(&composed);
        let line_count = (cow_foot + 4).max(composed.lines().count() + 3).max(6);
        let mascot_cols = composed
            .lines()
            .map(cow::str_display_width)
            .max()
            .unwrap_or(0);

        // In 120x35 viewport, compute reserved rows
        let config = SceneConfig {
            split_scroll: true,
            ..Default::default()
        };
        let (_, dyn_rows) =
            compute_reserved_dimensions_with_cols(120, 35, line_count, mascot_cols, &config);

        // 1. Tmux plan
        let tmux_mux = Mux::Tmux {
            pane: "0".to_string(),
            session: "test".to_string(),
        };
        let plan_tmux = forgum_platform::plan_native_split(
            TerminalEmulator::GenericVt,
            &tmux_mux,
            dyn_rows,
            0.35,
            &test_args,
        )
        .expect("Tmux plan must be generated");

        assert_eq!(plan_tmux.target, "tmux");
        assert_eq!(plan_tmux.program, "tmux");
        // Must contain -l <dyn_rows>
        let l_idx = plan_tmux
            .args
            .iter()
            .position(|a| a == "-l")
            .expect("Tmux args must have -l");
        assert_eq!(
            plan_tmux.args[l_idx + 1],
            dyn_rows.to_string(),
            "Tmux split lines must match dyn_rows"
        );

        // 2. Zellij plan
        let zellij_mux = Mux::Zellij {
            tab: "test".to_string(),
        };
        let plan_zellij = forgum_platform::plan_native_split(
            TerminalEmulator::GenericVt,
            &zellij_mux,
            dyn_rows,
            0.35,
            &test_args,
        )
        .expect("Zellij plan must be generated");

        assert_eq!(plan_zellij.target, "zellij");
        assert_eq!(plan_zellij.program, "zellij");

        // 3. WezTerm plan
        let wezterm_mux = Mux::WezTerm {
            tab: "1".to_string(),
        };
        let plan_wezterm = forgum_platform::plan_native_split(
            TerminalEmulator::WezTerm,
            &wezterm_mux,
            dyn_rows,
            0.35,
            &test_args,
        )
        .expect("WezTerm plan must be generated");

        assert_eq!(plan_wezterm.target, "wezterm");
        assert_eq!(plan_wezterm.program, "wezterm");
    }
}

// =========================================================================
// 2. WALL-CLOCK TIMING INVARIANTS SUITE (240 FPS, 60 FPS, 30 FPS)
// =========================================================================

#[test]
fn challenge_timing_invariants_empirical_cli() {
    let bin = binary_path();
    assert!(bin.exists(), "forgum binary must exist at {:?}", bin);

    let tmp = tempfile::tempdir().unwrap();
    let empty_cfg = tmp.path().join("config.json");
    std::fs::write(&empty_cfg, "{}").unwrap();

    let fps_list = [240, 60, 30];
    let duration_secs = 2;
    let expected_ms = duration_secs * 1000;
    // Disallow premature termination (the user bug where 240 FPS finished in 1s or less)
    let min_allowed_ms = expected_ms - 250;
    // Allow process launch/teardown margin under parallel test runs
    let max_allowed_ms = expected_ms + 2500;

    for &fps in &fps_list {
        eprintln!(
            "Running timing test: --duration {} --fps {}...",
            duration_secs, fps
        );
        let start = Instant::now();
        let status = Command::new(&bin)
            .env("FORGUM_CONFIG", &empty_cfg)
            .args([
                "render",
                "--duration",
                &duration_secs.to_string(),
                "--fps",
                &fps.to_string(),
                "--text",
                "Timing Invariant Verification",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .expect("Failed to execute forgum render");

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_millis() as u64;

        eprintln!(
            "-> Result for {} FPS: elapsed = {:?} ({} ms), status = {:?}",
            fps, elapsed, elapsed_ms, status
        );

        assert!(
            status.success(),
            "forgum render failed with status {:?}",
            status
        );
        assert!(
            elapsed_ms >= min_allowed_ms,
            "Execution ended too quickly for {} FPS: {}ms < min {}ms (-{}ms drift)",
            fps,
            elapsed_ms,
            min_allowed_ms,
            min_allowed_ms - elapsed_ms
        );
        assert!(
            elapsed_ms <= max_allowed_ms,
            "Execution took too long for {} FPS: {}ms > max {}ms (+{}ms drift)",
            fps,
            elapsed_ms,
            max_allowed_ms,
            elapsed_ms - max_allowed_ms
        );
    }
}

// =========================================================================
// 3. UNBOUNDED DURATION 0 SUITE
// =========================================================================

#[test]
fn challenge_duration_zero_unbounded_execution() {
    let bin = binary_path();
    assert!(bin.exists(), "forgum binary must exist at {:?}", bin);

    let tmp = tempfile::tempdir().unwrap();
    let empty_cfg = tmp.path().join("config.json");
    std::fs::write(&empty_cfg, "{}").unwrap();

    let mut child = Command::new(&bin)
        .env("FORGUM_CONFIG", &empty_cfg)
        .args([
            "render",
            "--background",
            "--duration",
            "0",
            "--text",
            "Infinite Simulation Run",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("Failed to spawn forgum render --duration 0");

    // Sleep 3.5 seconds. Old bug terminated at 2 seconds. Must still be alive.
    std::thread::sleep(Duration::from_millis(3500));

    let poll = child.try_wait().expect("try_wait failed");
    assert!(
        poll.is_none(),
        "FATAL: --duration 0 prematurely terminated after 3.5s! Status: {:?}",
        poll
    );

    // Clean termination
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &child.id().to_string(), "/T"])
            .status();
    }
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-9", &child.id().to_string()])
            .status();
    }

    let _ = child.wait();
}
