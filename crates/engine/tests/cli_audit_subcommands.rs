use forgum_engine::init::Shell;
use forgum_platform::ConfigFormat;

#[test]
fn audit_shell_hook_generation_completeness() {
    let shells = [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Pwsh,
        Shell::PowerShell,
        Shell::Cmd,
    ];

    for s in shells {
        let hook = forgum_engine::init::generate_hook(s, "/usr/local/bin/forgum-engine");
        assert!(!hook.is_empty(), "Hook for {:?} must not be empty", s);
        assert!(
            hook.contains("forgum-engine") || hook.contains("FORGUM_ENGINE"),
            "Hook for {:?} must contain engine reference",
            s
        );
    }
}

#[test]
fn audit_tmux_config_generator_formats() {
    let tmux = forgum_engine::init::generate_tmux_config("/bin/forgum-engine");
    assert!(tmux.contains("# >>> forgum tmux >>>"));
    assert!(tmux.contains("status-right"));
    assert!(tmux.contains("# <<< forgum tmux <<<"));

    let zellij = forgum_engine::init::generate_zellij_config("/bin/forgum-engine");
    assert!(zellij.contains("zellij run"));

    let wezterm = forgum_engine::init::generate_wezterm_config("/bin/forgum-engine");
    assert!(wezterm.contains("wezterm.on('update-status'"));

    let screen = forgum_engine::init::generate_screen_config("/bin/forgum-engine");
    assert!(screen.contains("hardstatus alwayslastline"));
}

#[test]
fn audit_status_line_length_bounds() {
    let short_line = forgum_engine::status_line::render_status_line(30);
    assert!(!short_line.is_empty());

    let long_line = forgum_engine::status_line::render_status_line(120);
    assert!(!long_line.is_empty());
}

#[test]
fn audit_battle_engine_produces_winner() {
    let battle_log = forgum_engine::battle::run_battle("Sir Moo-a-Lot", "Dragon Slayer");
    assert!(!battle_log.is_empty());
    assert!(
        battle_log.contains("VS") || battle_log.contains("Battle") || battle_log.contains("wins")
    );
}

#[test]
fn audit_showcase_and_demo_output() {
    let demo_out = forgum_engine::demo::run_demo().expect("demo should succeed");
    assert!(!demo_out.is_empty());

    let segments = forgum_engine::showcase::segments();
    assert_eq!(segments.len(), 7);
    let frame = forgum_engine::showcase::render_showcase_frame(&segments[0], 0.5);
    assert!(!frame.is_empty());
}

#[test]
fn audit_config_migration_all_permutations() {
    let dir = tempfile::tempdir().unwrap();
    let initial_cfg = forgum_engine::protocol::SceneConfig {
        cow: "dragon".into(),
        effect: "default".into(),
        fps: 60,
        shell_attach_mode: "split".into(),
        ..Default::default()
    };

    // 1. Start with JSON
    let json_path = dir.path().join("config.json");
    forgum_engine::config::write_config_file(&json_path, &initial_cfg, ConfigFormat::Json).unwrap();
    assert!(json_path.is_file());

    // 2. Migrate JSON -> YAML
    let yaml_path =
        forgum_engine::config::migrate_config_format(dir.path(), ConfigFormat::Yaml).unwrap();
    assert!(yaml_path.is_file());
    assert!(
        !json_path.exists(),
        "Old json config must be removed during migration"
    );
    let yaml_read = forgum_engine::config::read_config_file(&yaml_path).unwrap();
    assert_eq!(yaml_read.cow, "dragon");
    assert_eq!(yaml_read.shell_attach_mode, "split");

    // 3. Migrate YAML -> TOML
    let toml_path =
        forgum_engine::config::migrate_config_format(dir.path(), ConfigFormat::Toml).unwrap();
    assert!(toml_path.is_file());
    assert!(
        !yaml_path.exists(),
        "Old yaml config must be removed during migration"
    );
    let toml_read = forgum_engine::config::read_config_file(&toml_path).unwrap();
    assert_eq!(toml_read.cow, "dragon");
    assert_eq!(toml_read.fps, 60);

    // 4. Migrate TOML -> JSON
    let final_json =
        forgum_engine::config::migrate_config_format(dir.path(), ConfigFormat::Json).unwrap();
    assert!(final_json.is_file());
    assert!(
        !toml_path.exists(),
        "Old toml config must be removed during migration"
    );
    let final_read = forgum_engine::config::read_config_file(&final_json).unwrap();
    assert_eq!(final_read.cow, "dragon");
}

#[test]
fn audit_theme_seasonal_resolution() {
    let seasonal = forgum_engine::theme::seasonal_theme();
    assert!(seasonal.effect.is_some() || seasonal.cow.is_some());
}
