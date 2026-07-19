//! Shell completion generation using clap_complete.
//!
//! `forgum-engine completions <shell>` emits the completion script to stdout.

use std::io;

use clap_complete::{generate, shells};

use crate::init::Shell;

/// Generate shell completions and write to stdout.
pub fn generate_completions(shell: Shell, cmd: &mut clap::Command) -> io::Result<()> {
    let bin_name = "forgum-engine";

    match shell {
        Shell::Bash => {
            generate(shells::Bash, cmd, bin_name, &mut io::stdout());
        }
        Shell::Zsh => {
            generate(shells::Zsh, cmd, bin_name, &mut io::stdout());
        }
        Shell::Fish => {
            generate(shells::Fish, cmd, bin_name, &mut io::stdout());
        }
        Shell::Pwsh | Shell::PowerShell => {
            generate(shells::PowerShell, cmd, bin_name, &mut io::stdout());
        }
        Shell::Cmd => {
            // cmd.exe has no clap_complete backend; emit nothing.
        }
    }

    Ok(())
}
