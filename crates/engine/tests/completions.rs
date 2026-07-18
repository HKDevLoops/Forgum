use clap::CommandFactory;
use forgum_engine::cli::Cli;
use forgum_engine::completions::generate_completions;
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
