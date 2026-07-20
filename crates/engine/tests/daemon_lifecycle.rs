//! Integration test: spawn engine with --daemon, send STOP, verify exit.

#[cfg(unix)]
#[test]
fn daemon_lifecycle_ping_stop() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::process::Command;
    use std::time::{Duration, Instant};

    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // Start daemon in background. The engine --daemon forks: the parent prints
    // the child PID and exits; the child keeps running as the daemon.
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

    // Poll (don't fixed-sleep) for the socket to appear — robust under CI load.
    let session = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session);

    let socket_deadline = Instant::now() + Duration::from_secs(10);
    while !socket_path.exists() {
        if Instant::now() > socket_deadline {
            panic!("socket not found at {socket_path:?} after daemon start");
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    // Send STOP via socket.
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"{\"cmd\":\"STOP\"}\n");
        let _ = stream.flush();
    }

    // Poll for the daemon to actually exit (it may linger briefly while tearing
    // down the socket). Fixed sleeps here are the classic source of flakes.
    let exit_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if !forgum_platform::process_is_alive(pid) {
            break;
        }
        if Instant::now() > exit_deadline {
            panic!("daemon (pid {pid}) still alive 10s after STOP");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}
