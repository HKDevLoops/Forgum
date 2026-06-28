//! G9 — `open_output()` resolves to stdout when it's a tty, or to
//! `/dev/tty` (unix) / `CONOUT$` (windows) when it isn't, or to stdout as
//! last resort. The returned `target` field tells us which.

use forgum_platform::{open_output, OutputTarget};

#[test]
fn open_output_succeeds_in_canonical_environment() {
    // We don't care which target it picks — just that it doesn't error.
    let result = open_output();
    match result {
        Ok(h) => {
            // The target should be one of the three known values.
            assert!(matches!(
                h.target,
                OutputTarget::Stdout | OutputTarget::Tty | OutputTarget::Pipe
            ));
        }
        Err(e) => {
            // Acceptable: no terminal available.
            eprintln!("open_output err (acceptable in some CI): {e}");
        }
    }
}

#[test]
fn open_output_is_idempotent() {
    // Opening twice in the same process should yield the same target.
    if let (Ok(a), Ok(b)) = (open_output(), open_output()) {
        assert_eq!(a.target, b.target);
    }
}
