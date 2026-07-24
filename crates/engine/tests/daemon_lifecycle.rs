//! Integration test: spawn engine with --daemon, send STOP, verify exit.

#[cfg(unix)]
#[test]
fn daemon_lifecycle_ping_stop() {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::os::unix::net::UnixStream;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // We deliberately do NOT use `Command::output()` here. It blocks
    // until the engine parent's stdout AND stderr pipes are closed —
    // which only happens when the parent engine exits. On the cross
    // aarch64 lane the parent's `posix_spawn` is rejected by QEMU
    // user-mode, the fallback runs the daemon in-process for 30s, and
    // `.output()` blocks for the full duration. By the time it returns
    // the test's 15s polling deadline has already panicked.
    //
    // Instead we `spawn()` directly with `Stdio::piped()` and drain both
    // pipes on background threads, then poll the state file WHILE the
    // engine parent is still alive. This decouples liveness polling from
    // the parent-blocking `.output()` call.

    let mut child = Command::new(exe)
        .args(["--background", "--duration", "30", "--daemon"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn daemon parent");

    let stdout = child.stdout.take().expect("stdout pipe");
    let stderr = child.stderr.take().expect("stderr pipe");

    // Drain both pipes so the parent never blocks on a full pipe buffer.
    // We don't care about the contents here; the test asserts only via
    // the state file and control socket.
    let stdout_thread = thread::spawn(move || {
        let mut s = String::new();
        let _ = BufReader::new(stdout).read_to_string(&mut s);
        s
    });
    let stderr_thread = thread::spawn(move || {
        let s = String::new();
        BufReader::new(stderr)
            .lines()
            .take(200)
            .filter_map(Result::ok)
            .for_each(|l| eprintln!("[engine stderr] {l}"));
        s
    });

    // Build the state-file and socket paths from the test process's PID.
    // The engine parent's `detect_session_id()` honors `FORGUM_DAEMON_SESSION`
    // (we don't set it; config defaults hold), falling through to
    // `shell-<ppid>` on Unix, where ppid == THIS test process's PID.
    // Therefore the spawned daemon writes `daemon-shell-{test_pid}.json`
    // — the exact path we poll for below.
    let session = format!("shell-{}", std::process::id());
    let state_path = forgum_platform::daemon_state_path(&session);
    let socket_path = forgum_platform::control_socket_path(&session);

    // Poll for the daemon to come up: ready iff the state file exists with
    // a valid `pid` AND the control socket is bound.
    let mut pid: Option<u32> = None;
    let ready_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(text) = std::fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(p) = state.get("pid").and_then(|v| v.as_u64()) {
                    pid = Some(p as u32);
                }
            }
        }
        if pid.is_some() && socket_path.exists() {
            break;
        }
        if Instant::now() > ready_deadline {
            let _ = child.kill();
            panic!(
                "daemon not ready within 15s (state={:?}, socket_exists={})",
                pid,
                socket_path.exists()
            );
        }
        thread::sleep(Duration::from_millis(50));
    }
    let pid = pid.expect("daemon state file had no pid");

    // Send STOP via the control socket.
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"{\"cmd\":\"STOP\"}\n");
        let _ = stream.flush();
    }

    // Poll for daemon exit. On the spawn-Ok lane (Linux/macOS/Windows
    // native) the daemon survives parent exit; on the cross-aarch64
    // lane where posix_spawn fell through to in-process fallback, the
    // daemon is THIS process so it dies on child.kill().
    let exit_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if !forgum_platform::process_is_alive(pid) {
            break;
        }
        if Instant::now() > exit_deadline {
            // If the test process IS the daemon (fallback path), kill
            // it via its own parent. Otherwise we'd hang on `try_wait`.
            let _ = child.kill();
            panic!("daemon (pid {pid}) still alive 15s after STOP");
        }
        // Even on fallback, the daemon must respond to STOP via its
        // control socket without us polling its own liveness twice.
        thread::sleep(Duration::from_millis(50));
    }

    let _ = child.wait();
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();
}
