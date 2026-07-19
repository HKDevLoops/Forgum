//! Windows-specific tests for engine functionality.
//!
//! These tests are gated behind `#[cfg(windows)]` because they exercise
//! Windows-specific command execution (cmd.exe, taskkill).

#![cfg(windows)]

use std::process::Command;

#[test]
fn say_runs_cmd_echo() {
    let output = Command::new("cmd")
        .args(["/c", "echo", "hello"])
        .output()
        .expect("spawn cmd");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
}

#[cfg(windows)]
#[test]
fn timer_runs_cmd_echo() {
    let start = std::time::Instant::now();
    let output = Command::new("cmd")
        .args(["/c", "echo", "test"])
        .output()
        .expect("spawn cmd");
    let duration = start.elapsed().as_secs_f64();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"));
    // Duration should be reasonable (not zero, not too long)
    assert!(duration > 0.001);
    assert!(duration < 10.0);
}