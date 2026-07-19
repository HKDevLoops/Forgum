//! Integration test: shell hook generation for every `Shell` variant.

use forgum_engine::init::{generate_hook, generate_tmux_config, Shell};

const ENGINE: &str = "/usr/local/bin/forgum-engine";

#[test]
fn shell_parse_all_variants() {
    assert_eq!(Shell::parse("bash"), Some(Shell::Bash));
    assert_eq!(Shell::parse("zsh"), Some(Shell::Zsh));
    assert_eq!(Shell::parse("fish"), Some(Shell::Fish));
    assert_eq!(Shell::parse("pwsh"), Some(Shell::Pwsh));
    assert_eq!(Shell::parse("powershell"), Some(Shell::PowerShell));
    assert_eq!(Shell::parse("cmd"), Some(Shell::Cmd));
    assert_eq!(Shell::parse("nonsense"), None);
}

#[test]
fn every_shell_hook_is_nonempty_and_contains_engine_path() {
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Pwsh,
        Shell::Cmd,
        Shell::PowerShell,
    ] {
        let hook = generate_hook(shell, ENGINE);
        assert!(!hook.is_empty(), "hook for {shell} must not be empty");
        assert!(
            hook.contains(ENGINE),
            "hook for {shell} must embed the engine path"
        );
    }
}

#[test]
fn every_shell_hook_has_header_and_footer() {
    for shell in [
        Shell::Bash,
        Shell::Zsh,
        Shell::Fish,
        Shell::Pwsh,
        Shell::Cmd,
        Shell::PowerShell,
    ] {
        let hook = generate_hook(shell, ENGINE);
        assert!(hook.contains(">>> forgum"), "shell {shell} missing header");
        assert!(hook.contains("<<< forgum"), "shell {shell} missing footer");
    }
}

#[test]
fn bash_hook_uses_prompt_command() {
    let hook = generate_hook(Shell::Bash, ENGINE);
    assert!(hook.contains("function forgum") || hook.contains("forgum()"));
    assert!(hook.contains("PROMPT_COMMAND"));
    assert!(hook.contains("--background"));
    assert!(hook.contains("--duration 0"));
}

#[test]
fn zsh_hook_uses_precmd_functions() {
    let hook = generate_hook(Shell::Zsh, ENGINE);
    assert!(hook.contains("precmd_functions"));
    assert!(hook.contains("forgum()"));
}

#[test]
fn fish_hook_uses_function_and_event() {
    let hook = generate_hook(Shell::Fish, ENGINE);
    assert!(hook.contains("function forgum"));
    assert!(hook.contains("function __forgum_sweep"));
    assert!(hook.contains("--on-event fish_prompt"));
}

#[test]
fn pwsh_hook_wraps_prompt() {
    let hook = generate_hook(Shell::Pwsh, ENGINE);
    assert!(hook.contains("function forgum"));
    assert!(hook.contains("__ForgumPromptBackup"));
    assert!(hook.contains("function global:prompt"));
}

#[test]
fn cmd_hook_contains_prompt_and_sweep() {
    let hook = generate_hook(Shell::Cmd, ENGINE);
    assert!(hook.contains(">>> forgum"));
    assert!(hook.contains("<<< forgum"));
    assert!(hook.contains("prompt"));
    assert!(hook.contains("sweep"));
    assert!(hook.contains(ENGINE));
    assert!(hook.contains("cmd"));
}

#[test]
fn powershell_hook_wraps_prompt() {
    let hook = generate_hook(Shell::PowerShell, ENGINE);
    assert!(hook.contains("function forgum"));
    assert!(hook.contains("__ForgumPromptBackup"));
}

#[test]
fn shell_display_and_config_path() {
    assert_eq!(Shell::Bash.to_string(), "bash");
    assert_eq!(Shell::Zsh.to_string(), "zsh");
    assert_eq!(Shell::Fish.to_string(), "fish");
    assert_eq!(Shell::Pwsh.to_string(), "pwsh");
    assert_eq!(Shell::Cmd.to_string(), "cmd");
    assert_eq!(Shell::PowerShell.to_string(), "powershell");
    assert!(Shell::Pwsh.default_config_path().contains("APPDATA"));
    assert!(Shell::Cmd.default_config_path().contains("APPDATA"));
    assert!(Shell::PowerShell.default_config_path().contains("APPDATA"));
}

#[test]
fn tmux_config_contains_expected_markers() {
    let cfg = generate_tmux_config(ENGINE);
    assert!(cfg.contains(">>> forgum tmux >>>"));
    assert!(cfg.contains("<<< forgum tmux <<<"));
    assert!(cfg.contains("status-interval 5"));
    assert!(cfg.contains("status-line --max-len 70"));
    assert!(cfg.contains("pane-focus-in"));
    assert!(cfg.contains(ENGINE));
}
