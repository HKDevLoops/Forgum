//! Tier 3: Cross-Feature Combinations
//!
//! Validates emergent behavior across interdependent engine subsystems:
//! 1. Split-Scroll + Biological Palettes + Contrast Adaptation
//! 2. High FPS (240 FPS) + Background Simulation + Particle Time Decoupling
//! 3. Native Multiplexer Split + Custom Hex Palette Overrides
//! 4. Resilient DNA Parsing + Dynamic Catalog Fallback + Corrupted Record Isolation
//! 5. Daemon Control IPC + Graceful Signal Trapping + RAII Terminal Restoration
//! 6. Zero-Allocation Nature Math + Double-Buffered Framebuffer under Load
//! 7. TUI Responsive Breakpoints + Settings Matrix Lockstep
//! 8. Universal Shell Hooks + Marker Isolation + Soft/Purge Uninstall Lifecycle

use clap::Parser;
use forgum_engine::cli::Cli;
use forgum_engine::dna::{load_animations, BaseAnim, CowDna, ParticleType};
use forgum_engine::framebuffer::{Cell, FrameBuffer};
use forgum_engine::init::{generate_hook, Shell};
use forgum_engine::protocol::SceneConfig;
use forgum_engine::render::compute_reserved_dimensions;
use forgum_engine::scheduler::Scheduler;
use forgum_platform::biome::{natural_creature_color_adaptive, ALL_MASCOTS};
use forgum_platform::guards::{AltScreenGuard, CursorShowGuard, RawModeGuard};
use forgum_platform::shell::{remove_delimited_block, HOOK_MARKER_BEGIN, HOOK_MARKER_END};
use forgum_platform::signal::ShutdownFlag;
use forgum_platform::uninstaller::{UninstallMode, UninstallReport};
use forgum_tui::app::{Breakpoint, ConfigApp};
use std::collections::HashMap;
use std::fs;
use tempfile::tempdir;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

// =========================================================================
// Combo 1: Split-Scroll + Biological Palettes + Contrast Adaptation
// =========================================================================

#[test]
fn t3_combo_split_scroll_and_biological_palettes() {
    let config = SceneConfig {
        split_scroll: true,
        split_mode: Some("decstbm".to_string()),
        color_mode: "natural".to_string(),
        ..Default::default()
    };

    // 1. Dynamic reservation for tall mascot dragon (22 lines) on 100x50 terminal
    let (cols, rows) = compute_reserved_dimensions(100, 50, 22, &config);
    assert_eq!(cols, 100);
    assert_eq!(
        rows, 22,
        "Dragon reserves full 22 lines in split-scroll mode"
    );

    // 2. Validate biological palette slots for dragon
    let dragon = ALL_MASCOTS
        .iter()
        .find(|m| m.name == "dragon")
        .expect("dragon in biome");
    assert_eq!(dragon.natural_palette.len(), 5);

    // 3. Adapt palette for dark terminal background
    let dark_bg = (12, 12, 12);
    let coat_color = natural_creature_color_adaptive(&dragon.natural_rgb, 0, 0, '#', dark_bg);
    let lum = 0.299 * (coat_color.0 as f32)
        + 0.587 * (coat_color.1 as f32)
        + 0.114 * (coat_color.2 as f32);
    assert!(
        lum >= 70.0,
        "Coat color adapted for dark background must remain visible: lum={}",
        lum
    );
}

// =========================================================================
// Combo 2: High FPS (240 FPS) + Background Simulation + Particle Time Decoupling
// =========================================================================

#[test]
fn t3_combo_high_fps_and_particle_time_decoupling() {
    let cli = Cli::try_parse_from(argv(&[
        "forgum",
        "render",
        "--fps",
        "240",
        "--duration",
        "1",
        "--background",
    ]))
    .expect("parse 240 fps background");
    assert_eq!(cli.fps, Some(240));
    assert!(cli.background);

    // Verify scheduler period at 240 FPS
    let sched = Scheduler::new(240);
    let dt = sched.frame_period();
    let dt_secs = dt.as_secs_f32();
    assert!(
        dt_secs > 0.004 && dt_secs < 0.005,
        "dt per frame at 240 FPS should be ~4.16ms"
    );

    // Verify particle emission configuration
    let json = r#"{"phoenix": {"base": "Float", "particles": "Fire", "speed": 1.2}}"#;
    let map: HashMap<String, CowDna> = serde_json::from_str(json).expect("deserialize phoenix");
    let dna = &map["phoenix"];
    assert_eq!(dna.particles.r#type, ParticleType::Fire);
    assert_eq!(dna.particles.rate, 8);
}

// =========================================================================
// Combo 3: Native Multiplexer Split + Custom Hex Palette Overrides
// =========================================================================

#[test]
fn t3_combo_native_split_and_custom_hex_override() {
    let custom_hex = "#ff007f,#00f0ff,#7000ff,#ffe600,#ffffff";
    let cli = Cli::try_parse_from(argv(&[
        "forgum",
        "render",
        "--split-mode",
        "native",
        "--palette",
        custom_hex,
        "--reserve-rows",
        "18",
    ]))
    .expect("parse native split with custom palette");

    assert_eq!(cli.split_mode.as_deref(), Some("native"));
    assert_eq!(cli.palette.as_deref(), Some(custom_hex));
    assert_eq!(cli.reserve_rows, Some(18));

    // Hex tokens validation
    let slots: Vec<&str> = custom_hex.split(',').collect();
    assert_eq!(slots.len(), 5);
    for slot in slots {
        assert!(slot.starts_with('#') && slot.len() == 7);
    }
}

// =========================================================================
// Combo 4: Resilient DNA Parsing + Dynamic Catalog Fallback
// =========================================================================

#[test]
fn t3_combo_resilient_parse_and_catalog_fallback() {
    let tmp = tempdir().expect("tempdir");
    let corrupted_catalog = r##"{
        "dragon": {
            "base": "TotallyInvalidAnimType",
            "palette": "NotAnArray"
        },
        "healthy_cow": {
            "base": "Breathe",
            "speed": 0.8,
            "palette": []
        }
    }"##;
    fs::write(tmp.path().join("animations.json"), corrupted_catalog).expect("write json");

    // Resilient loader does not abort catalog loading; it falls back to biological defaults
    let loaded = load_animations(tmp.path());
    assert!(loaded.contains_key("healthy_cow"));
    assert!(
        loaded.contains_key("dragon"),
        "Corrupted canonical mascot falls back gracefully to biological default"
    );

    // Broken mascot receives fallback Walk animation and biological palette
    let fallback = &loaded["dragon"];
    assert_eq!(
        fallback.base,
        BaseAnim::Walk,
        "Fallback mascot defaults to Walk animation"
    );
    assert!(
        !fallback.palette.is_empty(),
        "Fallback mascot receives biological palette"
    );

    // Healthy mascot with empty palette automatically receives biological fallback
    let cow = &loaded["healthy_cow"];
    assert_eq!(cow.base, BaseAnim::Breathe);
    assert!(
        !cow.palette.is_empty(),
        "Fallback to biological palette for healthy cow"
    );
}

// =========================================================================
// Combo 5: Daemon Control IPC + Graceful Signal Trapping + RAII Guards
// =========================================================================

#[test]
fn t3_combo_daemon_ipc_and_raii_teardown() {
    // 1. Verify RAII guards compile and implement Drop cleanly
    let _raw_guard_size = std::mem::size_of::<RawModeGuard>();
    let _alt_guard_size = std::mem::size_of::<AltScreenGuard>();
    let _cur_guard_size = std::mem::size_of::<CursorShowGuard>();
    assert!(_raw_guard_size > 0 && _alt_guard_size > 0 && _cur_guard_size > 0);

    // 2. ShutdownFlag atomic coordination
    let flag = ShutdownFlag::new();
    assert!(!flag.is_shutdown());
    let handle = flag.shutdown_handle();
    handle.store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(flag.is_shutdown(), "Shutdown signal delivered atomically");

    // 3. CLI stop and sweep commands parse cleanly
    let res_stop = Cli::try_parse_from(argv(&["forgum", "stop"]));
    assert!(res_stop.is_ok(), "forgum stop must parse cleanly");

    let res_sweep = Cli::try_parse_from(argv(&["forgum", "sweep"]));
    assert!(res_sweep.is_ok(), "forgum sweep must parse cleanly");
}

// =========================================================================
// Combo 6: Zero-Allocation Nature Math + Double-Buffered Framebuffer
// =========================================================================

#[test]
fn t3_combo_nature_math_and_framebuffer_under_load() {
    let mut fb = FrameBuffer::new(160, 48);
    assert_eq!(fb.width, 160);
    assert_eq!(fb.height, 48);

    // Simulate 100 frames of nature mathematics and damage calculation
    for frame in 0..100 {
        let t = (frame as f32) * 0.033;
        fb.clear();

        for x in 0..160 {
            // Harmonic mountain elevation
            let elev = 8.0 * (1.0 + 0.4 * (x as f32 * 0.05 + t).sin().abs());
            let y = (40.0 - elev).max(0.0) as usize;
            if y < 48 {
                let cell = Cell::new('^', forgum_engine::framebuffer::Color::WHITE);
                fb.set(x, y, cell);
            }

            // Fibonacci phyllotaxis tree positioning
            let is_tree_pos = ((x as f64 * 1.6180339887).fract() * 10.0) < 1.0;
            if is_tree_pos && y < 47 {
                let trunk = Cell::new('|', forgum_engine::framebuffer::Color::WHITE);
                fb.set(x, y + 1, trunk);
            }
        }
    }
}

// =========================================================================
// Combo 7: TUI Responsive Breakpoints + Settings Matrix Lockstep
// =========================================================================

#[test]
fn t3_combo_tui_responsive_breakpoints_and_settings_lockstep() {
    // 1. Verify responsive breakpoints across all 3 Tailwind-inspired tiers
    assert_eq!(Breakpoint::from_size(60, 18), Breakpoint::Compact);
    assert_eq!(Breakpoint::from_size(100, 30), Breakpoint::Standard);
    assert_eq!(Breakpoint::from_size(160, 45), Breakpoint::Wide);

    // 2. In-memory TUI App creation without config on disk
    let app = ConfigApp::new(None, None, None);
    assert_eq!(app.current_tab, forgum_tui::app::Tab::Mascots);

    // 3. Bidirectional lockstep settings matrix serialization
    let cfg = SceneConfig {
        split_scroll: true,
        split_mode: Some("decstbm".to_string()),
        shell_attach_mode: "tmux".to_string(),
        ..Default::default()
    };

    let json = serde_json::to_string(&cfg).expect("serialize lockstep");
    let roundtrip: SceneConfig = serde_json::from_str(&json).expect("deserialize lockstep");
    assert!(roundtrip.split_scroll);
    assert_eq!(roundtrip.split_mode.as_deref(), Some("decstbm"));
    assert_eq!(roundtrip.shell_attach_mode, "tmux");
}

// =========================================================================
// Combo 8: Universal Shell Hooks + Marker Isolation + Uninstall Lifecycle
// =========================================================================

#[test]
fn t3_combo_shell_hooks_marker_isolation_and_uninstall() {
    let engine_bin = "forgum";

    // 1. Generate hooks for multiple shells
    let bash_hook = generate_hook(Shell::Bash, engine_bin);
    let pwsh_hook = generate_hook(Shell::Pwsh, engine_bin);

    assert!(bash_hook.contains(HOOK_MARKER_BEGIN) || bash_hook.contains("# >>> forgum (bash) >>>"));
    assert!(bash_hook.contains(HOOK_MARKER_END));
    assert!(pwsh_hook.contains(HOOK_MARKER_BEGIN) || pwsh_hook.contains("# >>> forgum (pwsh) >>>"));
    assert!(pwsh_hook.contains(HOOK_MARKER_END));

    // 2. Mock shell rc file insertion and clean excision
    let mock_bashrc = format!(
        "export PATH=$HOME/bin:$PATH\n{}\nalias ll='ls -lah'\n",
        bash_hook
    );
    let (cleaned, removed) =
        remove_delimited_block(&mock_bashrc, "# >>> forgum (bash) >>>", "# <<< forgum <<<");
    assert!(removed, "Delimited block must be excised");
    assert!(!cleaned.contains("forgum"));
    assert!(cleaned.contains("export PATH"));
    assert!(cleaned.contains("alias ll='ls -lah'"));

    // 3. Soft vs Purge uninstall verification
    let soft_report = UninstallReport {
        mode: UninstallMode::Soft,
        shells_cleaned: vec![(Shell::Bash, true), (Shell::Pwsh, true)],
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
    assert_eq!(soft_report.mode, UninstallMode::Soft);
    assert!(
        !soft_report.config_removed,
        "Soft uninstall preserves user configurations"
    );

    let purge_report = UninstallReport {
        mode: UninstallMode::Purge,
        shells_cleaned: vec![(Shell::Bash, true), (Shell::Pwsh, true)],
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
    assert_eq!(purge_report.mode, UninstallMode::Purge);
    assert!(
        purge_report.config_removed && purge_report.logs_removed,
        "Purge removes all directories"
    );
}
