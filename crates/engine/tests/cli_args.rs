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
    let (a, _) = parse_args(argv(&["forgum-engine", "render", "--text-only"])).unwrap();
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
