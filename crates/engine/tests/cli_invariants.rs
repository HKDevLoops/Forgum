//! G6 — bounded stdin: 5 MB input → error, non-zero exit (BUG-D4).
//! G7 — malformed JSON → non-zero exit (BUG-D5).
//! G9 — `open_output` works when stdout is piped (BUG-B9/C1).

use std::io::Write;
use std::process::{Command, Stdio};

fn binary_path() -> std::path::PathBuf {
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_forgum") {
        return std::path::PathBuf::from(bin);
    }
    let mut p = std::env::current_exe().unwrap();
    p.pop();
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
        if stdin.write_all(&chunk).is_err() {
            break;
        }
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
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    assert!(output.status.success(), "status command failed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("forgum status"),
        "status output should contain header: {stdout}"
    );
}

#[test]
fn version_command_works() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let output = Command::new(&bin)
        .args(["--version"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("forgum"));
}

#[test]
fn unknown_flag_returns_exit_64() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let output = Command::new(&bin)
        .args(["render", "--bogus-flag"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn");

    // EX_USAGE = 64 per `man sysexits.h`.
    assert_eq!(output.status.code(), Some(64));
}

#[test]
fn bare_engine_renders_thought_bubble_and_cow_when_piped() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, "{}").unwrap();

    let output = Command::new(&bin)
        .env("FORGUM_CONFIG", &cfg_path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn forgum-engine");

    assert!(output.status.success(), "bare forgum-engine must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify thought bubble parentheses exist
    assert!(
        stdout.contains('(') && stdout.contains(')'),
        "Default forgum-engine must render a thought bubble with parentheses: {stdout}"
    );

    // Verify cow mascot exists
    assert!(
        stdout.contains("^__^"),
        "Default forgum-engine must render cow mascot ^__^: {stdout}"
    );

    // Verify thought connection circles 'o' exist
    assert!(
        stdout.contains(" o ") || stdout.contains("o  ") || stdout.contains("  o"),
        "Default forgum-engine must render thought stem circles 'o': {stdout}"
    );
}

#[test]
fn think_subcommand_renders_thought_bubble_with_custom_text() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, "{}").unwrap();

    let custom_thought = "Quantum cows roam the cosmic pastures";
    let output = Command::new(&bin)
        .env("FORGUM_CONFIG", &cfg_path)
        .args(["think", custom_thought])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn forgum-engine think");

    assert!(output.status.success(), "think subcommand must exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify thought text is inside the output (wrapped to stay within cow width)
    assert!(
        stdout.contains("Quantum cows") && stdout.contains("cosmic pastures"),
        "Thought text must appear in output: {stdout}"
    );

    // Verify thought bubble parentheses
    assert!(
        stdout.contains('(') && stdout.contains(')'),
        "Output must have parentheses thought bubble borders: {stdout}"
    );

    // Verify cow mascot and connective circles
    assert!(stdout.contains("^__^"));
    assert!(stdout.contains(" o ") || stdout.contains("o  ") || stdout.contains("  o"));
}

#[test]
fn render_with_text_renders_speech_bubble() {
    let bin = binary_path();
    if !bin.exists() {
        return;
    }

    let tmp = tempfile::tempdir().unwrap();
    let cfg_path = tmp.path().join("config.json");
    std::fs::write(&cfg_path, "{}").unwrap();

    let speech_text = "Classic speech in speech bubble";
    let output = Command::new(&bin)
        .env("FORGUM_CONFIG", &cfg_path)
        .args(["render", "--text", speech_text])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn forgum-engine render --text");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify text appears across bubble lines
    for word in speech_text.split_whitespace() {
        assert!(
            stdout.contains(word),
            "Output missing word '{word}': {stdout}"
        );
    }

    // Verify speech bubble pipe borders |
    assert!(
        stdout.contains('|'),
        "Speech bubble must use vertical pipe borders: {stdout}"
    );

    // Verify speech backslash stem \
    assert!(
        stdout.contains('\\'),
        "Speech bubble must use backslash stem: {stdout}"
    );
}
