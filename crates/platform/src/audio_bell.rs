//! Terminal audio bell cue and synchronization subsystem.
//!
//! Provides non-blocking, mascot-specific auditory cues synchronized with terminal output.
//! Enforces strict 5-point silence guards (`AUDIO-GUARD-001..005`):
//! - `--silent` / `config.silent`
//! - `NO_COLOR` / `FORGUM_SILENT` / `FORGUM_NO_BELL`
//! - `TERM=dumb`
//! - Non-TTY / Piped stdout (`OutputTarget::Pipe`)
//! - Foreground editor detection (`!is_parent_shell_in_foreground()`)
//! - Minimum cooldown throttle (`MIN_CHIME_COOLDOWN_MS`)

use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Minimum cooldown period between consecutive chimes to prevent chime storms.
pub const MIN_CHIME_COOLDOWN_MS: u64 = 2500;

static LAST_CHIME_EPOCH_MS: AtomicU64 = AtomicU64::new(0);

/// Categorized terminal chime events for mascot and command feedback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChimeEvent {
    /// Single subtle bell on command success or timer completion.
    CommandSuccess,
    /// Double warning bell on non-zero exit code or error.
    CommandFailure,
    /// Impact bell cue during battle/joust collisions.
    BattleImpact,
    /// Subtle mascot greeting prompt alert.
    MascotGreet,
}

/// Evaluates all 5 silence guards (`AUDIO-GUARD-001..005`) to determine whether a bell cue is permitted:
/// 1. Explicit CLI `--silent` flag or config file `silent = true`.
/// 2. Non-interactive pipe or redirected stdout (`!is_tty`).
/// 3. Foreground full-screen editor active (`vim`, `nvim`, `emacs`, etc.).
/// 4. Standard silence environment variables (`NO_COLOR`, `FORGUM_SILENT`, `FORGUM_NO_BELL`, `TERM=dumb`).
/// 5. Cooldown gate preventing rapid audio spam.
#[must_use]
pub fn is_chime_permitted(
    cli_silent: bool,
    config_silent: bool,
    is_tty: bool,
    shell_in_foreground: bool,
) -> bool {
    if cli_silent || config_silent || !is_tty || !shell_in_foreground {
        return false;
    }

    if std::env::var_os("NO_COLOR").is_some()
        || std::env::var_os("FORGUM_SILENT").is_some()
        || std::env::var_os("FORGUM_NO_BELL").is_some()
        || std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false)
    {
        return false;
    }

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64;

    let prev_ms = LAST_CHIME_EPOCH_MS.load(Ordering::Relaxed);
    if now_ms.saturating_sub(prev_ms) < MIN_CHIME_COOLDOWN_MS {
        return false;
    }

    LAST_CHIME_EPOCH_MS.store(now_ms, Ordering::Relaxed);
    true
}

/// Emits a non-blocking, terminal-safe bell cue synchronized with the TTY flush.
///
/// If any silence guard evaluates to false, zero bytes are written to `tty_out`.
pub fn emit_synchronized_chime<W: Write>(
    tty_out: &mut W,
    event: ChimeEvent,
    cli_silent: bool,
    config_silent: bool,
    is_tty: bool,
    shell_in_foreground: bool,
) -> std::io::Result<()> {
    if !is_chime_permitted(cli_silent, config_silent, is_tty, shell_in_foreground) {
        return Ok(());
    }

    match event {
        ChimeEvent::CommandSuccess | ChimeEvent::BattleImpact | ChimeEvent::MascotGreet => {
            tty_out.write_all(b"\x07")?;
            tty_out.flush()?;
        }
        ChimeEvent::CommandFailure => {
            tty_out.write_all(b"\x07\x07")?;
            tty_out.flush()?;
        }
    }
    Ok(())
}

/// Force-reset the chime cooldown epoch for deterministic testing.
pub fn reset_chime_cooldown_for_test() {
    LAST_CHIME_EPOCH_MS.store(0, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_guards_cli_and_config() {
        reset_chime_cooldown_for_test();
        assert!(!is_chime_permitted(true, false, true, true));
        assert!(!is_chime_permitted(false, true, true, true));
    }

    #[test]
    fn test_silence_guards_tty_and_editor() {
        reset_chime_cooldown_for_test();
        assert!(!is_chime_permitted(false, false, false, true));
        assert!(!is_chime_permitted(false, false, true, false));
    }

    #[test]
    fn test_chime_permitted_when_clean() {
        reset_chime_cooldown_for_test();
        // Clear any interfering env vars for testing
        let _guard = std::env::var_os("NO_COLOR");
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("FORGUM_SILENT");
        std::env::remove_var("FORGUM_NO_BELL");

        let permitted = is_chime_permitted(false, false, true, true);
        assert!(permitted);

        // Immediate subsequent call must be blocked by cooldown
        let cooldown_blocked = !is_chime_permitted(false, false, true, true);
        assert!(cooldown_blocked);
    }

    #[test]
    fn test_emit_synchronized_chime_writes_expected_bytes() {
        reset_chime_cooldown_for_test();
        std::env::remove_var("NO_COLOR");
        std::env::remove_var("FORGUM_SILENT");
        std::env::remove_var("FORGUM_NO_BELL");

        let mut buf = Vec::new();
        emit_synchronized_chime(
            &mut buf,
            ChimeEvent::CommandSuccess,
            false,
            false,
            true,
            true,
        )
        .expect("emit chime");
        assert_eq!(buf, b"\x07");

        reset_chime_cooldown_for_test();
        let mut fail_buf = Vec::new();
        emit_synchronized_chime(
            &mut fail_buf,
            ChimeEvent::CommandFailure,
            false,
            false,
            true,
            true,
        )
        .expect("emit fail chime");
        assert_eq!(fail_buf, b"\x07\x07");
    }

    #[test]
    fn test_piped_output_emits_zero_bytes() {
        reset_chime_cooldown_for_test();
        let mut buf = Vec::new();
        emit_synchronized_chime(
            &mut buf,
            ChimeEvent::CommandSuccess,
            false,
            false,
            false, // is_tty = false (piped)
            true,
        )
        .expect("emit chime");
        assert!(
            buf.is_empty(),
            "Piped output must never receive 0x07 bell bytes!"
        );
    }
}
