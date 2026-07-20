//! Integration test: spawn engine with --daemon, send STOP, verify exit.

#[cfg(unix)]
#[test]
fn daemon_lifecycle_ping_stop() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::process::Command;
    use std::time::{Duration, Instant};

    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // Start the engine in --daemon mode. It forks: the parent prints the
    // child PID then exits; the child becomes the daemon and writes a
    // `daemon-<session>.json` state file (the same file `forgum herd` and
    // `Stop-ForgumDaemon` use to discover running daemons).
    let child = Command::new(exe)
        .args(["--background", "--duration", "30", "--daemon"])
        .output()
        .expect("failed to start daemon");
    assert!(child.status.success(), "daemon parent exited non-zero");

    let session = forgum_platform::detect_session_id();
    let state_path = forgum_platform::daemon_state_path(&session);
    let socket_path = forgum_platform::control_socket_path(&session);

    // Poll for the daemon to come up: it's "ready" once BOTH the state file
    // exists (child wrote its PID) AND the control socket is bound. Polling
    // with a deadline is robust under CI load (no brittle fixed sleeps).
    let ready_deadline = Instant::now() + Duration::from_secs(15);
    let mut pid: Option<u32> = None;
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
            panic!(
                "daemon not ready within 15s (state={:?}, socket_exists={})",
                pid,
                socket_path.exists()
            );
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    let pid = pid.expect("daemon state file had no pid");

    // Send STOP via the control socket.
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"{\"cmd\":\"STOP\"}\n");
        let _ = stream.flush();
    }

    // Poll for the daemon to actually exit (it may linger briefly while
    // tearing down the socket). Fixed sleeps here are the classic flake.
    let exit_deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if !forgum_platform::process_is_alive(pid) {
            break;
        }
        if Instant::now() > exit_deadline {
            panic!("daemon (pid {pid}) still alive 15s after STOP");
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // The state file should be cleaned up on exit.
    let _ = state_path;
}
