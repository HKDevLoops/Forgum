//! High-level terminal chime integration and event dispatch for the Forgum engine.

use std::io;

pub use forgum_platform::audio_bell::{
    emit_synchronized_chime, is_chime_permitted, reset_chime_cooldown_for_test, ChimeEvent,
    MIN_CHIME_COOLDOWN_MS,
};

/// Dispatches an auditory chime event if `--chime` is active and all silence guards pass.
pub fn trigger_chime(
    event: ChimeEvent,
    cli_chime: bool,
    cli_silent: bool,
    config_silent: bool,
) -> io::Result<()> {
    // Chime is an opt-in auditory cue unless configured or explicitly requested via --chime
    if !cli_chime {
        return Ok(());
    }

    let caps = forgum_platform::terminal::detect_capabilities();
    let parent_pid = forgum_platform::parent_pid();
    let shell_in_fg = forgum_platform::is_parent_shell_in_foreground(parent_pid);

    let mut stdout = io::stdout().lock();
    emit_synchronized_chime(
        &mut stdout,
        event,
        cli_silent,
        config_silent,
        caps.is_tty,
        shell_in_fg,
    )
}

/// Convenience hook for timer command completion.
pub fn maybe_chime_timer(success: bool, cli_chime: bool, cli_silent: bool) {
    let event = if success {
        ChimeEvent::CommandSuccess
    } else {
        ChimeEvent::CommandFailure
    };
    let _ = trigger_chime(event, cli_chime, cli_silent, false);
}

/// Convenience hook for mascot battle/joust impacts.
pub fn maybe_chime_battle_impact(cli_chime: bool, cli_silent: bool) {
    let _ = trigger_chime(ChimeEvent::BattleImpact, cli_chime, cli_silent, false);
}

/// Convenience hook for subtle mascot greeting prompts.
pub fn maybe_chime_greet(cli_chime: bool, cli_silent: bool) {
    let _ = trigger_chime(ChimeEvent::MascotGreet, cli_chime, cli_silent, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chime_requires_cli_or_config_optin() {
        reset_chime_cooldown_for_test();
        // If cli_chime is false, trigger_chime must immediately return Ok without writing to stdout
        let res = trigger_chime(ChimeEvent::CommandSuccess, false, false, false);
        assert!(res.is_ok());
    }

    #[test]
    fn test_chime_timer_convenience_does_not_panic() {
        reset_chime_cooldown_for_test();
        maybe_chime_timer(true, false, false);
        maybe_chime_timer(false, false, false);
        maybe_chime_battle_impact(false, false);
        maybe_chime_greet(false, false);
    }
}
