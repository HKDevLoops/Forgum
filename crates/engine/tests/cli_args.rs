use clap::Parser;
use forgum_engine::cli::{build_scene_config, parse_args, Cli, Command, Commands};
use forgum_engine::protocol::SceneConfig;

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

#[test]
fn version_flag_panics_or_exits() {
    let r = parse_args(argv(&["forgum-engine", "--version"]));
    assert!(r.is_err());
    let err = r.unwrap_err();
    assert_eq!(err.exit_code, 0);
}

#[test]
fn help_flag_exit_code_zero() {
    let r = parse_args(argv(&["forgum-engine", "--help"]));
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().exit_code, 0);
}

#[test]
fn unknown_flag_exit_code_64() {
    let r = parse_args(argv(&["forgum-engine", "--not-a-real-flag"]));
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().exit_code, 64);
}

#[test]
fn global_flags_parse() {
    let cli = Cli::try_parse_from(argv(&[
        "forgum-engine",
        "--reduce-motion",
        "--text-only",
        "--eyes",
        "$$",
        "--tongue",
        "U",
        "--fps",
        "45",
        "--duration",
        "10",
    ]))
    .unwrap();
    assert!(cli.reduce_motion);
    assert!(cli.text_only);
    assert_eq!(cli.eyes.as_deref(), Some("$$"));
    assert_eq!(cli.tongue.as_deref(), Some("U"));
    assert_eq!(cli.fps, Some(45));
    assert_eq!(cli.duration, Some(10));
}

#[test]
fn cow_text_effect_flags() {
    let cli = Cli::try_parse_from(argv(&[
        "forgum-engine",
        "--cow",
        "tux",
        "--text",
        "hi",
        "--effect",
        "aurora",
    ]))
    .unwrap();
    assert_eq!(cli.cow.as_deref(), Some("tux"));
    assert_eq!(cli.text.as_deref(), Some("hi"));
    assert_eq!(cli.effect.as_deref(), Some("aurora"));
}

#[test]
fn background_flag_sets_overlay_field() {
    let cli = Cli::try_parse_from(argv(&["forgum-engine", "--background"])).unwrap();
    assert!(cli.background);
}

#[test]
fn build_scene_uses_cli_eyes_tongue() {
    let (a, _) = parse_args(argv(&[
        "forgum-engine",
        "render",
        "--eyes",
        "$$",
        "--tongue",
        "U",
        "--fps",
        "50",
        "--duration",
        "7",
    ]))
    .unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.eyes, "$$");
    assert_eq!(cfg.tongue, "U");
    assert_eq!(cfg.fps, 50);
    assert_eq!(cfg.duration, 7);
}

#[test]
fn build_scene_text_only_does_not_change_config() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, r#"{}"#).unwrap();
    let (a, _) = parse_args(argv(&[
        "forgum-engine",
        "render",
        "--config",
        cfg_path.to_str().unwrap(),
        "--text-only",
    ]))
    .unwrap();
    assert!(a.text_only);
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg, SceneConfig::default());
}

#[test]
fn file_overlay_wins_for_set_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, r#"{"cow":"base","fps":15,"text":"hi"}"#).unwrap();
    let file_path = tmp.path().join("scene.json");
    // Overlay sets its own fps=60; config's lower-priority fps must lose.
    std::fs::write(
        &file_path,
        r#"{"cow":"override","effect":"aurora","fps":60}"#,
    )
    .unwrap();

    let (a, _) = parse_args(argv(&[
        "forgum-engine",
        "render",
        "--config",
        cfg_path.to_str().unwrap(),
        "--file",
        file_path.to_str().unwrap(),
    ]))
    .unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.cow, "override"); // file overlay wins
    assert_eq!(cfg.effect, "aurora"); // file overlay wins
    assert_eq!(cfg.fps, 60); // file overlay's fps wins over config
    assert_eq!(cfg.text, "hi"); // not in overlay -> from config
}

#[test]
fn file_overlay_without_fps_uses_default_not_config() {
    // Precedence sentinel is 0 (keep-base); omitting a field deserializes to the
    // default (30), which merge treats as an explicit value. So a config's fps
    // is NOT preserved when an overlay simply omits it. This documents the
    // engine's current merge semantics (only `0` means "keep base").
    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, r#"{"cow":"base","fps":15,"text":"hi"}"#).unwrap();
    let file_path = tmp.path().join("scene.json");
    std::fs::write(&file_path, r#"{"cow":"override","effect":"aurora"}"#).unwrap();

    let (a, _) = parse_args(argv(&[
        "forgum-engine",
        "render",
        "--config",
        cfg_path.to_str().unwrap(),
        "--file",
        file_path.to_str().unwrap(),
    ]))
    .unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.cow, "override");
    assert_eq!(cfg.effect, "aurora");
    assert_eq!(cfg.fps, 30); // default wins (overlay omitted fps)
    assert_eq!(cfg.text, "hi");
}

#[test]
fn demo_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "demo"])).unwrap();
    assert_eq!(a.command, Command::Demo);
    assert!(matches!(cmd, Some(Commands::Demo)));
}

#[test]
fn showcase_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "showcase"])).unwrap();
    assert_eq!(a.command, Command::Showcase);
    assert!(matches!(cmd, Some(Commands::Showcase)));
}

#[test]
fn say_subcommand_captures_cmd() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "say", "echo", "hello"])).unwrap();
    assert_eq!(a.command, Command::Say);
    assert!(
        matches!(cmd, Some(Commands::Say { cmd }) if cmd == vec!["echo".to_string(), "hello".to_string()])
    );
}

#[test]
fn timer_subcommand_captures_cmd() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "timer", "sleep", "1"])).unwrap();
    assert_eq!(a.command, Command::Timer);
    assert!(
        matches!(cmd, Some(Commands::Timer { cmd }) if cmd == vec!["sleep".to_string(), "1".to_string()])
    );
}

#[test]
fn battle_subcommand_defaults() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "battle"])).unwrap();
    assert_eq!(a.command, Command::Battle);
    assert!(
        matches!(cmd, Some(Commands::Battle { name1, name2 }) if name1 == "Alice" && name2 == "Bob")
    );
}

#[test]
fn battle_subcommand_custom_names() {
    let (_a, cmd) = parse_args(argv(&[
        "forgum-engine",
        "battle",
        "--name1",
        "Zoe",
        "--name2",
        "Yan",
    ]))
    .unwrap();
    assert!(
        matches!(cmd, Some(Commands::Battle { name1, name2 }) if name1 == "Zoe" && name2 == "Yan")
    );
}

#[test]
fn theme_list_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "theme", "list"])).unwrap();
    assert_eq!(a.command, Command::Theme);
    assert!(matches!(cmd, Some(Commands::Theme { sub: _ })));
}

#[test]
fn herd_list_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "herd", "list"])).unwrap();
    assert_eq!(a.command, Command::Herd);
    assert!(matches!(cmd, Some(Commands::Herd { sub: _ })));
}

#[test]
fn remote_who_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "remote", "who"])).unwrap();
    assert_eq!(a.command, Command::Remote);
    assert!(matches!(cmd, Some(Commands::Remote { sub: _ })));
}

#[test]
fn tmux_wezterm_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "tmux", "wez-term"])).unwrap();
    assert_eq!(a.command, Command::Tmux);
    assert!(matches!(cmd, Some(Commands::Tmux { sub: _ })));
}

#[test]
fn init_pwsh_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "init", "pwsh"])).unwrap();
    assert_eq!(a.command, Command::Init);
    assert!(matches!(
        cmd,
        Some(Commands::Init {
            shell: forgum_engine::cli::ShellArg::Pwsh,
            ..
        })
    ));
}

#[test]
fn status_line_max_len_flag() {
    let cli =
        Cli::try_parse_from(argv(&["forgum-engine", "status-line", "--max-len", "120"])).unwrap();
    match cli.command {
        Some(Commands::StatusLine { max_len }) => assert_eq!(max_len, 120),
        _ => panic!("expected StatusLine"),
    }
}

#[test]
fn default_command_is_render() {
    let (a, cmd) = parse_args(argv(&["forgum-engine"])).unwrap();
    assert_eq!(a.command, Command::Render);
    assert!(cmd.is_none());
}

#[test]
fn think_flag_sets_think_field_and_config() {
    let (a, _) = parse_args(argv(&["forgum-engine", "render", "--think"])).unwrap();
    assert!(a.think);
    let cfg = build_scene_config(&a).unwrap();
    assert!(cfg.think);
}

#[test]
fn think_subcommand_parses_thought_text() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "think", "I", "ponder", "deeply"])).unwrap();
    assert_eq!(a.command, Command::Think);
    assert!(a.think);
    assert_eq!(a.text.as_deref(), Some("I ponder deeply"));
    assert!(matches!(cmd, Some(Commands::Think { .. })));
    let cfg = build_scene_config(&a).unwrap();
    assert!(cfg.think);
    assert_eq!(cfg.text, "I ponder deeply");
}

#[test]
fn think_subcommand_without_args_sets_think() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "think"])).unwrap();
    assert_eq!(a.command, Command::Think);
    assert!(a.think);
    assert!(a.text.is_none());
    assert!(matches!(cmd, Some(Commands::Think { .. })));
    let cfg = build_scene_config(&a).unwrap();
    assert!(cfg.think);
}

#[test]
fn animal_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--animal", "list"])).unwrap();
    assert_eq!(a.cow.as_deref(), Some("list"));
}

#[test]
fn effect_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--effect", "list"])).unwrap();
    assert_eq!(a.effect.as_deref(), Some("list"));
}

#[test]
fn mountain_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--mountain", "list"])).unwrap();
    assert_eq!(a.mountain.as_deref(), Some("list"));
}

#[test]
fn road_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--road", "list"])).unwrap();
    assert_eq!(a.road.as_deref(), Some("list"));
}

#[test]
fn env_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--env", "list"])).unwrap();
    assert_eq!(a.environment.as_deref(), Some("list"));
}

#[test]
fn color_mode_list_arg_parses_list_value() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--color-mode", "list"])).unwrap();
    assert_eq!(a.color_mode.as_deref(), Some("list"));
}

#[test]
fn completions_list_and_default_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "completions", "list"])).unwrap();
    assert_eq!(a.command, Command::Completions);
    assert!(matches!(
        cmd,
        Some(Commands::Completions {
            shell: forgum_engine::cli::ShellArg::List
        })
    ));

    let (a_def, cmd_def) = parse_args(argv(&["forgum-engine", "completions"])).unwrap();
    assert_eq!(a_def.command, Command::Completions);
    assert!(matches!(
        cmd_def,
        Some(Commands::Completions {
            shell: forgum_engine::cli::ShellArg::List
        })
    ));
}

#[test]
fn init_list_and_default_subcommand() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "init", "list"])).unwrap();
    assert_eq!(a.command, Command::Init);
    assert!(matches!(
        cmd,
        Some(Commands::Init {
            shell: forgum_engine::cli::ShellArg::List,
            ..
        })
    ));

    let (a_def, cmd_def) = parse_args(argv(&["forgum-engine", "init"])).unwrap();
    assert_eq!(a_def.command, Command::Init);
    assert!(matches!(
        cmd_def,
        Some(Commands::Init {
            shell: forgum_engine::cli::ShellArg::List,
            ..
        })
    ));
}

#[test]
fn config_list_flag_and_key_query() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "config", "--list"])).unwrap();
    assert_eq!(a.command, Command::Config);
    assert!(matches!(cmd, Some(Commands::Config { list: true, .. })));

    let (a2, cmd2) = parse_args(argv(&["forgum-engine", "config", "list"])).unwrap();
    assert_eq!(a2.command, Command::Config);
    assert!(matches!(
        cmd2,
        Some(Commands::Config {
            key: Some(ref k),
            ..
        }) if k == "list"
    ));
}

#[test]
fn tmux_list_subcommand_parses() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "tmux", "list"])).unwrap();
    assert_eq!(a.command, Command::Tmux);
    assert!(matches!(
        cmd,
        Some(Commands::Tmux {
            sub: forgum_engine::cli::TmuxSub::List
        })
    ));
}

#[test]
fn remote_list_subcommand_parses() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "remote", "list"])).unwrap();
    assert_eq!(a.command, Command::Remote);
    assert!(matches!(
        cmd,
        Some(Commands::Remote {
            sub: forgum_engine::cli::RemoteSub::List
        })
    ));
}

#[test]
fn list_subcommand_and_options_aliases() {
    let (a, cmd) = parse_args(argv(&["forgum-engine", "list", "effects"])).unwrap();
    assert_eq!(a.command, Command::List);
    assert!(matches!(
        cmd,
        Some(Commands::List { ref category }) if category == "effects"
    ));

    let (a2, cmd2) = parse_args(argv(&["forgum-engine", "options", "scenery"])).unwrap();
    assert_eq!(a2.command, Command::List);
    assert!(matches!(
        cmd2,
        Some(Commands::List { ref category }) if category == "scenery"
    ));
}

#[test]
fn options_table_renders_all_categories() {
    let categories = [
        "all",
        "animals",
        "effects",
        "mountains",
        "roads",
        "environments",
        "colors",
        "shells",
        "config",
        "multiplexers",
        "eyes",
        "tongue",
        "scenery",
    ];

    for cat in categories {
        let table = forgum_engine::options_table::render_options(cat);
        assert!(
            !table.is_empty(),
            "Table for category '{cat}' must not be empty"
        );
        assert!(
            table.contains('┌') && table.contains('┐'),
            "Table for category '{cat}' must contain box borders"
        );
    }
}

#[test]
fn top_level_list_flag_and_category() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--list"])).unwrap();
    assert_eq!(a.list.as_deref(), Some("all"));

    let (a2, _) = parse_args(argv(&["forgum-engine", "--list", "animals"])).unwrap();
    assert_eq!(a2.list.as_deref(), Some("animals"));
}

#[test]
fn typo_suggestions_produced_for_invalid_options() {
    let err = parse_args(argv(&["forgum-engine", "renderr"])).unwrap_err();
    assert!(err.message.contains("Did you mean 'render'?"));

    let err_opt = parse_args(argv(&["forgum-engine", "--animall"])).unwrap_err();
    assert!(err_opt.message.contains("Did you mean '--animal'?"));

    let candidates = &["walk", "run", "fly", "float", "ember", "aurora"];
    assert_eq!(
        forgum_engine::cli::find_closest_match("walkk", candidates),
        Some("walk")
    );
    assert_eq!(
        forgum_engine::cli::find_closest_match("flly", candidates),
        Some("fly")
    );
}

#[test]
fn animal_signature_defaults_apply_automatically() {
    let (a, _) = parse_args(argv(&["forgum-engine", "--animal", "nyan"])).unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.cow, "nyan");
    assert_eq!(cfg.environment.as_deref(), Some("space"));
    assert_eq!(cfg.road.as_deref(), Some("grid"));
    assert_eq!(cfg.mountain.as_deref(), Some("crater"));
    assert_eq!(cfg.effect, "fly");
    assert!(cfg.palette.as_ref().unwrap().contains("#ff0033"));
}

#[test]
fn dynamic_scenario_adaptation_adapts_road_mountain_and_motion() {
    // When environment is overridden to ocean for walking animal (tux):
    // road adapts to seabed, mountain to seamount, motion adapts to float (swimming)
    let (a, _) = parse_args(argv(&[
        "forgum-engine",
        "--animal",
        "tux",
        "--env",
        "ocean",
    ]))
    .unwrap();
    let cfg = build_scene_config(&a).unwrap();
    assert_eq!(cfg.cow, "tux");
    assert_eq!(cfg.environment.as_deref(), Some("ocean"));
    assert_eq!(cfg.road.as_deref(), Some("seabed"));
    assert_eq!(cfg.mountain.as_deref(), Some("seamount"));
    assert_eq!(cfg.effect, "float");
}

#[test]
fn install_subcommand_parses_cleanly() {
    let (a, cmd) = parse_args(argv(&["forgum", "install"])).unwrap();
    assert_eq!(a.command, Command::Install);
    assert!(matches!(cmd, Some(Commands::Install)));

    let (a_wizard, _) = parse_args(argv(&["forgum", "setup"])).unwrap();
    assert_eq!(a_wizard.command, Command::Install);
}

#[test]
fn update_subcommand_parses_cleanly() {
    let (a, cmd) = parse_args(argv(&["forgum", "update"])).unwrap();
    assert_eq!(a.command, Command::Update);
    assert!(matches!(cmd, Some(Commands::Update { check: false })));

    let (a_check, cmd_check) = parse_args(argv(&["forgum", "update", "--check"])).unwrap();
    assert_eq!(a_check.command, Command::Update);
    assert!(matches!(cmd_check, Some(Commands::Update { check: true })));

    let (a_upgrade, _) = parse_args(argv(&["forgum", "upgrade"])).unwrap();
    assert_eq!(a_upgrade.command, Command::Update);
}
