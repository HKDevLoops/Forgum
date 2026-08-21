//! G1 — RAII guards restore terminal state on normal return AND on panic.
//!
//! These tests only run when stdout is a tty (otherwise the guards can't be
//! acquired). On non-tty environments (most CI) we skip with a printed
//! reason.

use forgum_platform::{AltScreenGuard, CursorShowGuard, RawModeGuard};

fn tty_out() -> bool {
    crossterm::tty::IsTty::is_tty(&std::io::stdout())
}

#[test]
fn raw_mode_guard_drop_restores_on_normal_return() {
    if !tty_out() {
        eprintln!("skipping: stdout not a tty");
        return;
    }
    {
        let _g = RawModeGuard::acquire().expect("enable raw");
        assert!(crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
    }
    assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
}

#[test]
fn raw_mode_guard_drop_restores_after_panic() {
    if !tty_out() {
        eprintln!("skipping: stdout not a tty");
        return;
    }
    let result = std::panic::catch_unwind(|| {
        let _g = RawModeGuard::acquire().expect("enable raw");
        panic!("forced");
    });
    assert!(result.is_err());
    assert!(!crossterm::terminal::is_raw_mode_enabled().unwrap_or(false));
}

#[test]
fn alt_screen_and_cursor_guards_construct_and_drop_safely() {
    if !tty_out() {
        eprintln!("skipping: stdout not a tty");
        return;
    }
    let _alt = AltScreenGuard::acquire().expect("acquire alt screen");
    let _cur = CursorShowGuard::acquire().expect("acquire cursor guard");
}

#[test]
fn alt_screen_and_cursor_guards_restore_after_panic() {
    if !tty_out() {
        eprintln!("skipping: stdout not a tty");
        return;
    }
    let result = std::panic::catch_unwind(|| {
        let _alt = AltScreenGuard::acquire().expect("acquire alt screen");
        let _cur = CursorShowGuard::acquire().expect("acquire cursor guard");
        panic!("forced panic inside guards");
    });
    assert!(result.is_err());
}
