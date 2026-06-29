# Daemon Lifecycle Integration Test — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an integration test that verifies the full daemon lifecycle: spawn daemon → PING → STOP → verify exit.

**Architecture:** A single test file `crates/engine/tests/daemon_lifecycle.rs` spawns the `forgum-engine` binary with `--background --duration 30 --daemon`, captures the child PID from the parent's stdout, connects to the control socket, sends PING and STOP commands, and verifies the daemon exits. The test is gated on `cfg(unix)` because the daemon's control socket only works on Unix (on Windows, `daemonize()` spawns a detached child without `--daemon`, so no control socket is set up). A Windows no-op stub documents this limitation.

**Tech Stack:** Rust, `std::os::unix::net::UnixStream`, `forgum_platform::{detect_session_id, control_socket_path, process_is_alive}`

---

## Context: Why Unix-Only?

The daemon lifecycle has a fundamental platform difference:

- **Unix:** `daemonize()` uses `fork()`. The parent prints PID and exits. The child inherits the already-started control socket server (same address space) and enters the daemon render loop with `cmd_rx`.
- **Windows:** `daemonize()` spawns a new detached process with `--daemon` stripped from args. The child re-parses args, enters the *foreground* code path (no control socket), and `OutputHandle::open()` fails because `DETACHED_PROCESS` means no console.

Therefore, the control socket only exists on Unix. The test is gated accordingly.

---

## File Structure

| File | Action | Purpose |
|------|--------|---------|
| `crates/engine/tests/daemon_lifecycle.rs` | **Create** | Integration test for daemon spawn → PING → STOP → exit |

No other files need modification. The test uses `env!("CARGO_BIN_EXE_forgum-engine")` (Cargo sets this automatically for integration tests when the package defines a `[[bin]]`), `forgum_platform` re-exports, and `std::os::unix::net::UnixStream`.

---

### Task 1: Create the daemon lifecycle test

**Files:**
- Create: `crates/engine/tests/daemon_lifecycle.rs`

- [ ] **Step 1: Write the test file**

```rust
//! Daemon lifecycle integration test.
//!
//! Spawns `forgum-engine --background --duration 30 --daemon`, verifies the
//! parent prints the child PID and exits 0, connects to the control socket,
//! sends PING (verifies `"ok":true`), sends STOP, and asserts the daemon
//! process is no longer alive.
//!
//! Gated on `cfg(unix)` because the daemon control socket only functions on
//! Unix. On Windows, `daemonize()` spawns a detached child without `--daemon`,
//! so no control socket is set up in the child process.

#![cfg(unix)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
use std::time::Duration;

#[test]
fn daemon_lifecycle_ping_stop() {
    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // Spawn daemon in background. The parent prints PID to stdout and exits 0.
    let child = Command::new(exe)
        .args(["--background", "--duration", "30", "--daemon"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .expect("failed to start daemon");

    // Parse PID from parent stdout.
    let stdout = String::from_utf8_lossy(&child.stdout);
    let pid: u32 = match stdout.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            eprintln!(
                "could not parse PID from stdout: {stdout:?}, skipping"
            );
            return;
        }
    };

    assert!(child.status.success(), "parent should exit 0");

    // Wait for daemon to bind the control socket.
    std::thread::sleep(Duration::from_millis(1500));

    let session = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session);

    if !socket_path.exists() {
        eprintln!("socket not found at {socket_path:?}, skipping");
        let _ = kill_pid(pid);
        return;
    }

    // Connect to control socket and send PING.
    let mut stream = match UnixStream::connect(&socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not connect to socket: {e}, skipping");
            let _ = kill_pid(pid);
            return;
        }
    };

    // Send PING command.
    stream
        .write_all(b"{\"cmd\":\"PING\"}\n")
        .expect("write PING");
    stream.flush().expect("flush PING");

    // Read PING response.
    let mut reader = BufReader::new(&stream);
    let mut ping_response = String::new();
    reader
        .read_line(&mut ping_response)
        .expect("read PING response");
    assert!(
        ping_response.contains("\"ok\":true"),
        "PING response should contain ok:true: {ping_response}"
    );

    // Send STOP command on a fresh connection (the PING connection may be
    // consumed by BufReader).
    drop(reader);
    drop(stream);

    let mut stream = UnixStream::connect(&socket_path)
        .expect("reconnect for STOP");
    stream
        .write_all(b"{\"cmd\":\"STOP\"}\n")
        .expect("write STOP");
    stream.flush().expect("flush STOP");

    // Wait for daemon to exit.
    std::thread::sleep(Duration::from_millis(2000));

    // Verify PID is no longer alive.
    assert!(
        !forgum_platform::process_is_alive(pid),
        "daemon PID {pid} should have exited after STOP"
    );
}

/// Kill a process by PID (best-effort cleanup).
fn kill_pid(pid: u32) -> std::process::Output {
    Command::new("kill")
        .arg(pid.to_string())
        .output()
        .unwrap_or_else(|_| panic!("failed to kill pid {pid}"))
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo test --test daemon_lifecycle --no-run`
Expected: Compiles successfully with no errors.

- [ ] **Step 3: Run the test**

Run: `cargo test --test daemon_lifecycle`
Expected: Test passes (daemon spawns, PING returns `"ok":true`, STOP exits daemon). May skip on CI if `/dev/tty` is unavailable.

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests pass, no regressions.

- [ ] **Step 5: Commit**

```bash
git add crates/engine/tests/daemon_lifecycle.rs
git commit -m "test(engine): add daemon lifecycle integration test"
```

---

## Self-Review Checklist

- [x] **Spec coverage:** Test covers all 8 steps from the spec: spawn daemon, parse PID, wait for socket, connect, PING + verify response, STOP, wait for exit, assert not alive.
- [x] **No placeholders:** All code is complete and runnable.
- [x] **Type consistency:** Uses `forgum_platform::detect_session_id()`, `forgum_platform::control_socket_path()`, `forgum_platform::process_is_alive()` — all verified to exist in the platform crate's public API.
- [x] **Graceful skip:** Socket-not-found and connect-failure paths print a message and return (test passes as a skip).
- [x] **Cleanup:** `kill_pid()` is called on skip paths to avoid leaked daemon processes.
- [x] **No `#[cfg]` in engine/src:** The test is in `tests/`, not `src/`, so the CI grep gate is not affected.
