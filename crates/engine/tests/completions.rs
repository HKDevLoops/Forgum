use clap::CommandFactory;
use forgum_engine::cli::Cli;
use forgum_engine::completions::{generate_completion_script, generate_completions};
use forgum_engine::init::Shell;

#[test]
fn completions_bash_writes() {
    let mut cmd = Cli::command();
    let r = generate_completions(Shell::Bash, &mut cmd);
    assert!(r.is_ok());
}

#[test]
fn completions_zsh_writes() {
    let mut cmd = Cli::command();
    assert!(generate_completions(Shell::Zsh, &mut cmd).is_ok());
}

#[test]
fn completions_fish_writes() {
    let mut cmd = Cli::command();
    assert!(generate_completions(Shell::Fish, &mut cmd).is_ok());
}

#[test]
fn completions_pwsh_writes() {
    let mut cmd = Cli::command();
    assert!(generate_completions(Shell::Pwsh, &mut cmd).is_ok());
}

#[test]
fn completions_powershell_writes() {
    let mut cmd = Cli::command();
    assert!(generate_completions(Shell::PowerShell, &mut cmd).is_ok());
}

#[test]
fn completions_all_15_shells_write() {
    for &shell in Shell::ALL {
        let mut cmd = Cli::command();
        assert!(
            generate_completions(shell, &mut cmd).is_ok(),
            "generate_completions failed for {shell}"
        );
    }
}

#[test]
fn every_shell_generates_valid_nonempty_completion_script() {
    for &shell in Shell::ALL {
        let mut cmd = Cli::command();
        let script = generate_completion_script(shell, &mut cmd);
        assert!(
            !script.is_empty(),
            "completion script for {shell} must not be empty"
        );
        assert!(
            script.contains("forgum"),
            "completion script for {shell} must reference 'forgum'"
        );
    }
}

#[test]
fn shell_specific_completion_syntax_and_keywords() {
    let mut cmd = Cli::command();

    // Bash
    let bash = generate_completion_script(Shell::Bash, &mut cmd);
    assert!(bash.contains("_forgum"), "bash must define _forgum");
    assert!(
        bash.contains("_forgum_custom_complete"),
        "bash must have custom completion helpers"
    );

    // Zsh
    let zsh = generate_completion_script(Shell::Zsh, &mut cmd);
    assert!(zsh.contains("_forgum"), "zsh must define _forgum");
    assert!(
        zsh.contains("_forgum_effects"),
        "zsh must define _forgum_effects candidates"
    );

    // Fish
    let fish = generate_completion_script(Shell::Fish, &mut cmd);
    assert!(
        fish.contains("complete -c forgum"),
        "fish must register complete -c forgum"
    );
    assert!(
        fish.contains("__fish_forgum_no_subcommand"),
        "fish must have subcommand isolation guard"
    );

    // Pwsh & Windows PowerShell
    let pwsh = generate_completion_script(Shell::Pwsh, &mut cmd);
    assert!(
        pwsh.contains("Register-ArgumentCompleter"),
        "pwsh must call Register-ArgumentCompleter"
    );
    assert!(
        pwsh.contains("CompletionResult"),
        "pwsh must construct CompletionResult items"
    );

    let ps = generate_completion_script(Shell::PowerShell, &mut cmd);
    assert!(
        ps.contains("Register-ArgumentCompleter"),
        "powershell must call Register-ArgumentCompleter"
    );
    assert!(
        ps.contains("CompletionResult"),
        "powershell must construct CompletionResult items"
    );

    // Elvish
    let elvish = generate_completion_script(Shell::Elvish, &mut cmd);
    assert!(
        elvish.contains("forgum"),
        "elvish script must contain forgum"
    );

    // Nushell
    let nushell = generate_completion_script(Shell::Nushell, &mut cmd);
    assert!(
        nushell.contains("export extern \"forgum\""),
        "nushell must export extern 'forgum'"
    );
    assert!(
        nushell.contains("def \"nu_forgum_effects\""),
        "nushell must define nu_forgum_effects"
    );
    assert!(
        nushell.contains("def \"nu_forgum_animals\""),
        "nushell must define nu_forgum_animals"
    );

    // Verify Nushell bracket, brace, and paren balancing
    let open_brace = nushell.chars().filter(|&c| c == '{').count();
    let close_brace = nushell.chars().filter(|&c| c == '}').count();
    assert_eq!(
        open_brace, close_brace,
        "nushell braces must balance: open={open_brace}, close={close_brace}"
    );

    let open_bracket = nushell.chars().filter(|&c| c == '[').count();
    let close_bracket = nushell.chars().filter(|&c| c == ']').count();
    assert_eq!(
        open_bracket, close_bracket,
        "nushell brackets must balance: open={open_bracket}, close={close_bracket}"
    );

    let open_paren = nushell.chars().filter(|&c| c == '(').count();
    let close_paren = nushell.chars().filter(|&c| c == ')').count();
    assert_eq!(
        open_paren, close_paren,
        "nushell parens must balance: open={open_paren}, close={close_paren}"
    );

    // Carapace
    let carapace = generate_completion_script(Shell::Carapace, &mut cmd);
    assert!(
        carapace.contains("name: forgum"),
        "carapace must declare name: forgum"
    );
    assert!(
        carapace.contains("completion:"),
        "carapace must define completion flag candidates"
    );

    // Xonsh
    let xonsh = generate_completion_script(Shell::Xonsh, &mut cmd);
    assert!(
        xonsh.contains("@contextual_command_completer_for(\"forgum\")"),
        "xonsh must bind contextual command completer"
    );

    // Tcsh
    let tcsh = generate_completion_script(Shell::Tcsh, &mut cmd);
    assert!(
        tcsh.contains("complete forgum"),
        "tcsh must define complete forgum"
    );

    // Ksh
    let ksh = generate_completion_script(Shell::Ksh, &mut cmd);
    assert!(ksh.contains("_forgum"), "ksh must generate bash/ksh spec");

    // Ion
    let ion = generate_completion_script(Shell::Ion, &mut cmd);
    assert!(
        ion.contains("complete -c forgum"),
        "ion must define complete -c forgum"
    );

    // Oil
    let oil = generate_completion_script(Shell::Oil, &mut cmd);
    assert!(
        oil.contains("_forgum"),
        "oil must generate bash-compatible spec"
    );

    // Yash
    let yash = generate_completion_script(Shell::Yash, &mut cmd);
    assert!(
        yash.contains("completion/forgum"),
        "yash must define completion/forgum function"
    );

    // Cmd
    let cmd_script = generate_completion_script(Shell::Cmd, &mut cmd);
    assert!(
        cmd_script.contains("cmd.exe does not support shell tab completion scripts"),
        "cmd must return descriptive explanation"
    );
}

#[test]
fn completions_marker_block_is_cleanly_uninstalled() {
    let begin = forgum_platform::COMPLETIONS_MARKER_BEGIN;
    let end = forgum_platform::COMPLETIONS_MARKER_END;

    let user_rc = format!(
        "export PATH=/usr/bin\n{begin}\n# Generated by forgum completions — do not edit by hand\nsource ~/.config/forgum/completions/forgum.bash\n{end}\nalias ll='ls -l'\n"
    );

    let (cleaned, removed) = forgum_platform::remove_all_forgum_blocks(user_rc);
    assert!(removed, "completions block must be recognized and removed");
    assert_eq!(
        cleaned, "export PATH=/usr/bin\nalias ll='ls -l'\n",
        "clean uninstallation must leave user config intact"
    );
}

#[test]
fn completions_contain_split_scroll_and_reservation_flags() {
    let mut cmd = Cli::command();

    // Fish
    let fish = generate_completion_script(Shell::Fish, &mut cmd);
    assert!(
        fish.contains("-l split-scroll"),
        "fish must complete -l split-scroll"
    );
    assert!(
        fish.contains("-l reserve-rows"),
        "fish must complete -l reserve-rows"
    );
    assert!(
        fish.contains("-l reserve-cols"),
        "fish must complete -l reserve-cols"
    );
    assert!(
        fish.contains("-l split-ratio"),
        "fish must complete -l split-ratio"
    );

    // PowerShell / Pwsh
    let pwsh = generate_completion_script(Shell::Pwsh, &mut cmd);
    assert!(
        pwsh.contains("'--split-scroll'"),
        "pwsh must contain --split-scroll flag"
    );
    assert!(
        pwsh.contains("'--reserve-rows'"),
        "pwsh must contain --reserve-rows flag"
    );
    assert!(
        pwsh.contains("'--reserve-cols'"),
        "pwsh must contain --reserve-cols flag"
    );
    assert!(
        pwsh.contains("'--split-ratio'"),
        "pwsh must contain --split-ratio flag"
    );

    let ps = generate_completion_script(Shell::PowerShell, &mut cmd);
    assert!(
        ps.contains("'--reserve-rows'"),
        "powershell must contain --reserve-rows flag"
    );

    // Nushell
    let nushell = generate_completion_script(Shell::Nushell, &mut cmd);
    assert!(
        nushell.contains("--split-scroll"),
        "nushell must complete --split-scroll"
    );
    assert!(
        nushell.contains("--reserve-rows"),
        "nushell must complete --reserve-rows"
    );
    assert!(
        nushell.contains("--reserve-cols"),
        "nushell must complete --reserve-cols"
    );
    assert!(
        nushell.contains("--split-ratio"),
        "nushell must complete --split-ratio"
    );

    // Carapace
    let carapace = generate_completion_script(Shell::Carapace, &mut cmd);
    assert!(
        carapace.contains("--split-scroll:"),
        "carapace must define --split-scroll"
    );
    assert!(
        carapace.contains("--reserve-rows=:"),
        "carapace must define --reserve-rows"
    );
    assert!(
        carapace.contains("--reserve-cols=:"),
        "carapace must define --reserve-cols"
    );
    assert!(
        carapace.contains("--split-ratio=:"),
        "carapace must define --split-ratio"
    );

    // Xonsh
    let xonsh = generate_completion_script(Shell::Xonsh, &mut cmd);
    assert!(
        xonsh.contains("\"--split-scroll\""),
        "xonsh must contain --split-scroll"
    );
    assert!(
        xonsh.contains("\"--reserve-rows\""),
        "xonsh must contain --reserve-rows"
    );
    assert!(
        xonsh.contains("\"--reserve-cols\""),
        "xonsh must contain --reserve-cols"
    );
    assert!(
        xonsh.contains("\"--split-ratio\""),
        "xonsh must contain --split-ratio"
    );
}
