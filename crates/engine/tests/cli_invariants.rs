//! G6 — bounded stdin: 5 MB input → error, non-zero exit (BUG-D4).
//! G7 — malformed JSON → non-zero exit (BUG-D5).
//! G9 — `open_output` works when stdout is piped (BUG-B9/C1).

use std::io::Write;
use std::process::{Command, Stdio};

fn binary_path() -> std::path::PathBuf {
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    if cfg!(windows) {
        p.push("forgum-engine.exe");
    } else {
        p.push("forgum-engine");
    }
    p
}

#[test]
fn huge_stdin_rejected() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let mut child = Command::new(&bin)
        .args(["render", "--text", "x"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    // Write 5 MB of garbage to stdin (exceeds the 4 MB cap).
    let stdin = child.stdin.as_mut().unwrap();
    let chunk = vec![b' '; 64 * 1024];
    for _ in 0..(5 * 1024 / 64 + 1) {
        stdin.write_all(&chunk).unwrap();
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    assert!(!output.status.success(), "BUG-D4: 5 MB stdin was accepted");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("too large") || stderr.contains("InvalidArgument"),
        "BUG-D4: stderr didn't explain rejection: {stderr}"
    );
}

#[test]
fn malformed_json_exits_nonzero() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let mut child = Command::new(&bin)
        .args(["render", "--text", "x"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"{ not json")
        .unwrap();
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    assert!(
        !output.status.success(),
        "BUG-D5: malformed JSON returned exit 0"
    );
}

#[test]
fn piped_stdout_falls_back_gracefully() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    // Run with stdout piped (not a tty). The engine should either open
    // /dev/tty or fall back to stdout. Either way it must exit 0.
    let output = Command::new(&bin)
        .args(["status"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    assert!(output.status.success(), "status command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), "ok");
}

#[test]
fn version_command_works() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let output = Command::new(&bin)
        .args(["--version"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forgum-engine"));
}

#[test]
fn unknown_flag_returns_exit_64() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let output = Command::new(&bin)
        .args(["render", "--bogus-flag"])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    // EX_USAGE = 64 per `man sysexits.h`.
    assert_eq!(output.status.code(), Some(64));
}
