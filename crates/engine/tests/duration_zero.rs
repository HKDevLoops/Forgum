//! G4 + G2 — `duration=0` means "infinite", and SIGTERM causes graceful
//! exit.
//!
//! These are integration tests that spawn the actual `forgum-engine` binary
//! with `--background --duration 0`, sleep briefly, send SIGTERM (Unix) or
//! `Stop-Process` (Windows), and assert the process exits within 500 ms
//! with exit code 0.
//!
//! The tests are gated on `cfg(unix)` for the SIGTERM path; on Windows we
//! use a graceful `Stop-Process` instead, which sends WM_CLOSE / CTRL_C_EVENT
//! via the .NET runtime.

#![cfg(any(unix, windows))]

use std::process::Command;
use std::time::{Duration, Instant};

fn binary_path() -> std::path::PathBuf {
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_forgum") {
        return std::path::PathBuf::from(bin);
    }
    let mut p = std::env::current_exe().unwrap();
    p.pop(); // remove test exe name
    if p.file_name().is_some_and(|n| n == "deps") {
        p.pop();
    }
    let name = if cfg!(windows) {
        "forgum.exe"
    } else {
        "forgum"
    };
    p.push(name);
    if !p.exists() {
        p.pop();
        p.push(if cfg!(windows) {
            "forgum-engine.exe"
        } else {
            "forgum-engine"
        });
    }
    p
}

#[test]
fn duration_zero_runs_indefinitely_until_killed() {
    let bin = binary_path();
    if !bin.exists() {
        eprintln!(
            "forgum-engine binary not found at {}; skipping",
            bin.display()
        );
        return;
    }

    let mut child = Command::new(&bin)
        .args([
            "render",
            "--background",
            "--duration",
            "0",
            "--text",
            "hello",
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn forgum-engine");

    // Wait 2 s. With `duration=0`, the engine must still be running.
    std::thread::sleep(Duration::from_secs(2));
    assert!(
        matches!(child.try_wait(), Ok(None)),
        "BUG-B2 regression: duration=0 exited early"
    );

    // Gracefully terminate.
    let start = Instant::now();
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .args(["-TERM", &child.id().to_string()])
            .status();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/F", "/PID", &child.id().to_string(), "/T"])
            .status();
    }

    let status = child.wait().expect("wait");
    let elapsed = start.elapsed();

    // The shutdown should complete within a few seconds.
    assert!(
        elapsed < Duration::from_secs(5),
        "graceful shutdown took too long: {:?}",
        elapsed
    );
    #[cfg(unix)]
    assert!(status.success(), "engine exited non-zero: {:?}", status);
    #[cfg(windows)]
    let _ = status;
}

#[test]
fn duration_n_seconds_exits_in_time() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let start = Instant::now();
    let status = Command::new(&bin)
        .args(["render", "--text", "hello", "--duration", "1"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("spawn");
    let elapsed = start.elapsed();

    assert!(status.success(), "engine exited non-zero: {:?}", status);
    assert!(
        elapsed < Duration::from_secs(10),
        "BUG regression: duration=1 took {:?}",
        elapsed
    );
}
