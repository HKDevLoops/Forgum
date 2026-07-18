//! Integration test: spawn engine with --daemon, send STOP, verify exit.

#[cfg(unix)]
#[test]
fn daemon_lifecycle_ping_stop() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::process::Command;
    use std::time::Duration;
    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // Start daemon in background.
    let child = Command::new(exe)
        .args(["--background", "--duration", "30", "--daemon"])
        .output()
        .expect("failed to start daemon");

    // The parent prints PID and exits. Capture it.
    let stdout = String::from_utf8_lossy(&child.stdout);
    let pid: u32 = stdout
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("could not parse PID from stdout: {stdout:?}"));

    // Give daemon time to bind socket.
    std::thread::sleep(Duration::from_millis(1000));

    // Check if socket exists.
    let session = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session);

    assert!(
        socket_path.exists(),
        "socket not found at {socket_path:?} after daemon start"
    );

    // Send STOP via socket.
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"{\"cmd\":\"STOP\"}\n");
        let _ = stream.flush();
    }

    // Wait for daemon to exit.
    std::thread::sleep(Duration::from_millis(1000));

    // Verify PID is no longer alive.
    assert!(
        !forgum_platform::process_is_alive(pid),
        "daemon should have exited after STOP"
    );
}
