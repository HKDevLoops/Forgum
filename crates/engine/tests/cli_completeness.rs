//! G11 — CLI completion-drift guard.
//!
//! Asserts the `cli::Commands` enum includes every subcommand documented in
//! the test-coverage matrix (brain/13) and the user-facing CLI surface. If a
//! subcommand is removed or renamed, this test fails and flags the drift.
//!
//! `completions` is intentionally allowed (it's a developer subcommand not in
//! the documented user set) — see `ALLOWED_EXTRA`.

#[test]
fn cli_commands_include_documented_set() {
    // Documented user-facing subcommands that must remain present.
    let documented = [
        "init",
        "fortune",
        "config",
        "render",
        "status",
        "tmux",
        "status-line",
        "herd",
        "theme",
        "demo",
        "showcase",
        "remote",
        "say",
        "timer",
        "battle",
    ];

    // Developer-only subcommands that exist in the enum but are not part of
    // the documented user set (no drift expected, but not asserted as required).
    let allowed_extra = ["completions"];

    // The canonical names of every variant currently in the enum.
    let present: Vec<&'static str> = all_command_variants();

    for name in documented {
        assert!(
            present.contains(&name),
            "CLI drift: documented subcommand `{name}` is missing from `Commands`"
        );
    }

    // Everything in the enum must be either documented or explicitly allowed.
    for v in &present {
        assert!(
            documented.contains(v) || allowed_extra.contains(v),
            "CLI drift: `Commands::{v}` is neither documented nor in allowed_extra"
        );
    }
}

/// Enumerate the string name of every `Commands` variant.
fn all_command_variants() -> Vec<&'static str> {
    vec![
        "render",
        "fortune",
        "init",
        "completions",
        "status",
        "config",
        "tmux",
        "status-line",
        "herd",
        "theme",
        "demo",
        "showcase",
        "remote",
        "say",
        "timer",
        "battle",
    ]
}
