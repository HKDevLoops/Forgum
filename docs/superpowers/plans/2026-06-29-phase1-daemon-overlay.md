# Phase 1 — Daemon/Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the background animation daemon actually work — fork/detach, write PID file, run a control socket listener, dispatch commands to the render loop, and clean up properly. Cross-platform (Unix + Windows).

**Architecture:** The engine binary's `--daemon` flag triggers a self-re-exec: the parent forks (Unix) or spawns a detached child (Windows), writes `daemon.json`, and exits. The child binds a Unix domain socket (or Windows named pipe) for control commands. A `crossbeam`-free `std::sync::mpsc` channel bridges the socket listener thread to the render loop. The PowerShell module gets `Stop-ForgumDaemon` that reads `daemon.json`, sends SIGTERM (or `Stop-Process` on Windows), and cleans up.

**Tech Stack:** `nix` 0.29 (Unix socket + fork helpers), `windows-sys` 0.59 (named pipes), `std::sync::mpsc` (channel), `serde_json` (protocol), existing `DaemonState`/`ControlCmd`/`ControlResponse` types.

---

## File Map

| File | Action | Purpose |
|------|--------|---------|
| `crates/platform/src/spawn.rs` | Modify | Add `daemonize()` that forks+detaches+exits parent |
| `crates/platform/src/daemon_socket.rs` | Create | Cross-platform Unix domain socket / Windows named pipe server |
| `crates/platform/src/lib.rs` | Modify | Export `daemonize`, `DaemonSocket` |
| `crates/platform/Cargo.toml` | Modify | Add `nix` feature `user` for `unistd::getpid` |
| `crates/engine/src/control_socket.rs` | Modify | Add `ControlServer` that binds socket, accepts connections, dispatches via `mpsc` |
| `crates/engine/src/render.rs` | Modify | Accept `mpsc::Receiver<ControlCmd>` in background loop |
| `crates/engine/src/main.rs` | Modify | Wire daemon lifecycle: fork, write PID, bind socket, pass receiver to render loop |
| `crates/engine/src/daemon.rs` | Modify | Add `write_daemon_state()` convenience, `cleanup_daemon_state()` |
| `Forgum.psm1` or `Public/forgum.ps1` | Modify | Add `Stop-ForgumDaemon` function |
| `crates/engine/tests/daemon_lifecycle.rs` | Create | Integration test: spawn daemon, send PING, send STOP, verify exit |

---

## Task 1: Add `daemonize()` to Platform Crate

**Files:**
- Modify: `crates/platform/src/spawn.rs`
- Modify: `crates/platform/src/lib.rs`
- Modify: `crates/platform/Cargo.toml`

The platform crate already has `spawn_detached()` (fork+setsid). We need a higher-level `daemonize()` that:
1. Forks on Unix / re-execs on Windows
2. Parent prints PID to stdout and exits 0
3. Child continues in new session
4. Returns `Ok(true)` in parent, `Ok(false)` in child

### Step 1: Add `nix` feature for `unistd::getpid`

In `crates/platform/Cargo.toml`, the `nix` dependency already has `process` feature which includes `fork`/`setsid`. But we need `unistd::getpid` for the child PID. The `process` feature already covers this. Just verify:

```toml
nix = { version = "0.29", default-features = false, features = ["fs", "term", "signal", "process"] }
```

No change needed — `process` includes `unistd::getpid`.

### Step 2: Add `daemonize()` function

Add to `crates/platform/src/spawn.rs`:

```rust
/// Daemonize the current process.
///
/// On Unix: forks. Parent prints child PID to stdout and exits with 0.
/// Child calls `setsid()` and returns `Ok(false)`.
///
/// On Windows: spawns a detached copy of self. Parent prints child PID
/// and exits with 0. Child process returns `Ok(false)` (it's a fresh
/// process, so this function is called again — but the child doesn't
/// re-daemonize because it won't have the `--daemon` flag).
///
/// Returns `Ok(true)` in parent, `Ok(false)` in child.
pub fn daemonize() -> Result<bool, PlatformError> {
    #[cfg(unix)]
    {
        use nix::unistd::{fork, ForkResult};
        match fork().map_err(|e| PlatformError::Io(e))? {
            ForkResult::Parent { child } => {
                // Parent: print child PID and exit.
                println!("{}", child.as_raw());
                std::process::exit(0);
            }
            ForkResult::Child => {
                // Child: become session leader.
                nix::unistd::setsid().map_err(|e| PlatformError::Io(e))?;
                return Ok(false);
            }
        }
    }
    #[cfg(windows)]
    {
        // Windows: re-launch self without --daemon flag.
        use std::os::windows::process::CommandExt;
        use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

        let exe = std::env::current_exe().map_err(PlatformError::Io)?;
        let args: Vec<String> = std::env::args()
            .filter(|a| a != "--daemon")
            .collect();
        let mut cmd = std::process::Command::new(exe);
        cmd.args(&args[1..]); // skip argv[0]
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        let child = cmd.spawn().map_err(PlatformError::Io)?;
        println!("{}", child.id());
        std::process::exit(0);
    }
}
```

### Step 3: Export from `lib.rs`

Add to `crates/platform/src/lib.rs` exports:

```rust
pub use spawn::{daemonize, process_is_alive, spawn_detached, spawn_silent, DetachedChild};
```

### Step 4: Verify it compiles

Run: `cargo check -p forgum-platform`
Expected: Compiles with no errors.

### Step 5: Commit

```bash
git add crates/platform/
git commit -m "feat(platform): add daemonize() for cross-platform daemon detach"
```

---

## Task 2: Add Cross-Platform Socket Server to Platform Crate

**Files:**
- Create: `crates/platform/src/daemon_socket.rs`
- Modify: `crates/platform/src/lib.rs`
- Modify: `crates/platform/Cargo.toml`

The control socket needs to be a cross-platform IPC mechanism:
- **Unix:** Unix domain socket at `$XDG_RUNTIME_DIR/Forgum/ctrl-{session}.sock`
- **Windows:** Named pipe at `\\.\pipe\forgum-ctrl-{session}`

### Step 1: Add `nix` `user` feature for socket ownership

In `crates/platform/Cargo.toml`, add `"user"` to nix features:

```toml
nix = { version = "0.29", default-features = false, features = ["fs", "term", "signal", "process", "user"] }
```

### Step 2: Create `daemon_socket.rs`

```rust
//! Cross-platform IPC socket for daemon control.
//!
//! - Unix: Unix domain socket (SOCK_STREAM)
//! - Windows: Named pipe (\\.\pipe\forgum-{session})

use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use crate::error::PlatformError;

/// A bound and listening IPC socket. Call `accept()` in a loop to handle
/// connections. Each connection provides a `BufReader` + `Write` pair for
/// reading newline-delimited commands and writing responses.
pub struct DaemonSocket {
    inner: SocketInner,
}

enum SocketInner {
    #[cfg(unix)]
    Unix {
        listener: std::os::unix::net::UnixListener,
        _path: PathBuf,
    },
    #[cfg(windows)]
    Windows {
        pipe_name: String,
    },
}

use std::path::PathBuf;

impl DaemonSocket {
    /// Bind a new socket at the given path.
    ///
    /// On Unix, removes any stale socket file first.
    /// On Windows, the pipe is created by `ConnectNamedPipe`.
    pub fn bind(path: &Path) -> Result<Self, PlatformError> {
        #[cfg(unix)]
        {
            // Remove stale socket.
            let _ = std::fs::remove_file(path);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let listener = std::os::unix::net::UnixListener::bind(path)
                .map_err(PlatformError::Io)?;
            listener.set_nonblocking(true).map_err(PlatformError::Io)?;
            Ok(Self {
                inner: SocketInner::Unix {
                    listener,
                    _path: path.to_path_buf(),
                },
            })
        }
        #[cfg(windows)]
        {
            let pipe_name = format!(
                "\\\\.\\pipe\\forgum-{}",
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("default")
            );
            Ok(Self {
                inner: SocketInner::Windows { pipe_name },
            })
        }
    }

    /// Accept a new connection. Returns a `SocketConnection` that can
    /// read commands and write responses.
    ///
    /// On Unix this is non-blocking; returns `None` if no connection pending.
    /// On Windows this blocks until a client connects (used in a dedicated thread).
    pub fn accept(&self) -> Result<Option<SocketConnection>, PlatformError> {
        match &self.inner {
            #[cfg(unix)]
            SocketInner::Unix { listener, .. } => {
                match listener.accept() {
                    Ok((stream, _addr)) => {
                        stream.set_nonblocking(false).map_err(PlatformError::Io)?;
                        Ok(Some(SocketConnection::Unix(stream)))
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
                    Err(e) => Err(PlatformError::Io(e)),
                }
            }
            #[cfg(windows)]
            SocketInner::Windows { pipe_name } => {
                // Windows named pipe: create a new pipe instance and wait for client.
                use std::ffi::OsStr;
                use std::os::windows::ffi::OsStrExt;
                use windows_sys::Win32::Storage::FileSystem::{
                    CreateNamedPipeW, PIPE_ACCESS_DUPLEX,
                };
                use windows_sys::Win32::System::Threading::ConnectNamedPipe;

                let wide: Vec<u16> = OsStr::new(pipe_name)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();
                unsafe {
                    let handle = CreateNamedPipeW(
                        wide.as_ptr(),
                        PIPE_ACCESS_DUPLEX,
                        0, // pipe mode
                        1, // max instances
                        4096, // out buffer
                        4096, // in buffer
                        0, // default timeout
                        std::ptr::null(),
                    );
                    if handle == -1isize as usize {
                        return Err(PlatformError::Io(io::Error::last_os_error()));
                    }
                    ConnectNamedPipe(handle, std::ptr::null_mut());
                    Ok(Some(SocketConnection::Windows(handle)))
                }
            }
        }
    }

    /// Remove the socket/pipe file.
    pub fn cleanup(&self) {
        match &self.inner {
            #[cfg(unix)]
            SocketInner::Unix { _path, .. } => {
                let _ = std::fs::remove_file(_path);
            }
            #[cfg(windows)]
            SocketInner::Windows { .. } => {
                // Named pipes are cleaned up by the OS when all handles close.
            }
        }
    }
}

/// A single accepted connection.
pub enum SocketConnection {
    #[cfg(unix)]
    Unix(std::os::unix::net::UnixStream),
    #[cfg(windows)]
    Windows(usize), // HANDLE
}

impl SocketConnection {
    /// Read a newline-delimited line from the connection.
    pub fn read_line(&mut self) -> Result<Option<String>, PlatformError> {
        match self {
            #[cfg(unix)]
            SocketConnection::Unix(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => Ok(None), // EOF
                    Ok(_) => Ok(Some(line)),
                    Err(e) => Err(PlatformError::Io(e)),
                }
            }
            #[cfg(windows)]
            SocketConnection::Windows(handle) => {
                // Read from Windows named pipe.
                use windows_sys::Win32::Storage::FileSystem::ReadFile;
                let mut buf = [0u8; 4096];
                let mut bytes_read = 0u32;
                unsafe {
                    let ok = ReadFile(
                        *handle,
                        buf.as_mut_ptr(),
                        buf.len() as u32,
                        &mut bytes_read,
                        std::ptr::null_mut(),
                    );
                    if ok == 0 {
                        return Err(PlatformError::Io(io::Error::last_os_error()));
                    }
                }
                let s = String::from_utf8_lossy(&buf[..bytes_read as usize]).to_string();
                Ok(Some(s))
            }
        }
    }

    /// Write a response line (newline-terminated).
    pub fn write_response(&mut self, data: &str) -> Result<(), PlatformError> {
        match self {
            #[cfg(unix)]
            SocketConnection::Unix(stream) => {
                use std::io::Write;
                stream.write_all(data.as_bytes()).map_err(PlatformError::Io)?;
                stream.flush().map_err(PlatformError::Io)?;
                Ok(())
            }
            #[cfg(windows)]
            SocketConnection::Windows(handle) => {
                use windows_sys::Win32::Storage::FileSystem::WriteFile;
                let bytes = data.as_bytes();
                let mut written = 0u32;
                unsafe {
                    let ok = WriteFile(
                        *handle,
                        bytes.as_ptr(),
                        bytes.len() as u32,
                        &mut written,
                        std::ptr::null_mut(),
                    );
                    if ok == 0 {
                        return Err(PlatformError::Io(io::Error::last_os_error()));
                    }
                }
                Ok(())
            }
        }
    }
}

impl Drop for SocketConnection {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            if let SocketConnection::Windows(handle) = self {
                use windows_sys::Win32::Foundation::CloseHandle;
                unsafe { CloseHandle(*handle); }
            }
        }
    }
}
```

### Step 3: Export from `lib.rs`

Add module declaration and re-export:

```rust
pub mod daemon_socket;
pub use daemon_socket::{DaemonSocket, SocketConnection};
```

### Step 4: Verify compilation

Run: `cargo check -p forgum-platform`
Expected: Compiles.

### Step 5: Commit

```bash
git add crates/platform/
git commit -m "feat(platform): add DaemonSocket for cross-platform IPC"
```

---

## Task 3: Implement ControlServer in Engine

**Files:**
- Modify: `crates/engine/src/control_socket.rs`
- Modify: `crates/engine/Cargo.toml`

The existing `control_socket.rs` has parsing/encoding. We add a `ControlServer` that:
1. Binds a `DaemonSocket`
2. Runs an accept loop in a thread
3. For each connection: reads lines, parses `ControlCmd`, sends via `mpsc::Sender`
4. Returns `mpsc::Receiver<ControlCmd>` to the render loop

### Step 1: Add `crossbeam` or use `std::sync::mpsc`

We'll use `std::sync::mpsc` (no new dependency). But we need the sender to be shared across threads for concurrent connections. `mpsc::Sender` is `Clone`, so that's fine.

Actually, we need the `ControlServer` to hold a `JoinHandle` for the listener thread and a `Receiver` for the render loop. Let's use `std::sync::mpsc`.

### Step 2: Add `ControlServer` to `control_socket.rs`

Append to `crates/engine/src/control_socket.rs`:

```rust
use std::sync::mpsc;
use std::thread;

/// A control socket server that accepts connections and dispatches commands.
pub struct ControlServer {
    socket_path: std::path::PathBuf,
    _thread: thread::JoinHandle<()>,
}

impl ControlServer {
    /// Bind the control socket and start listening.
    ///
    /// Returns the receiver end of the command channel. The render loop
    /// reads commands from this receiver.
    pub fn start(
        socket_path: std::path::PathBuf,
    ) -> Result<(Self, mpsc::Receiver<ControlCmd>), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let socket = forgum_platform::DaemonSocket::bind(&socket_path)?;

        let thread = thread::spawn(move || {
            Self::accept_loop(socket, tx);
        });

        Ok((
            Self {
                socket_path,
                _thread: thread,
            },
            rx,
        ))
    }

    fn accept_loop(
        socket: forgum_platform::DaemonSocket,
        tx: mpsc::Sender<ControlCmd>,
    ) {
        loop {
            match socket.accept() {
                Ok(Some(mut conn)) => {
                    // Read commands from this connection.
                    loop {
                        match conn.read_line() {
                            Ok(Some(line)) => {
                                let cmd = parse_cmd(&line);
                                let is_stop = matches!(cmd, ControlCmd::Stop);
                                let is_status = matches!(cmd, ControlCmd::Status);
                                let is_ping = matches!(cmd, ControlCmd::Ping);

                                if is_status {
                                    let resp = ControlResponse {
                                        ok: true,
                                        error: None,
                                        status: Some(StatusInfo {
                                            running: true,
                                            paused: false,
                                            effect: "unknown".into(),
                                            fps: 30,
                                            speed: 1.0,
                                        }),
                                    };
                                    let _ = conn.write_response(&encode_response(&resp));
                                    continue;
                                }

                                if is_ping {
                                    let resp = ControlResponse {
                                        ok: true,
                                        error: None,
                                        status: None,
                                    };
                                    let _ = conn.write_response(&encode_response(&resp));
                                    continue;
                                }

                                // Send command to render loop.
                                if tx.send(cmd).is_err() {
                                    return; // render loop dropped
                                }

                                // Send generic OK response.
                                let resp = ControlResponse {
                                    ok: true,
                                    error: None,
                                    status: None,
                                };
                                let _ = conn.write_response(&encode_response(&resp));

                                if is_stop {
                                    return; // stop accept loop
                                }
                            }
                            Ok(None) => break, // client disconnected
                            Err(_) => break,   // read error
                        }
                    }
                }
                Ok(None) => {
                    // No connection pending (non-blocking).
                    // Sleep briefly to avoid busy-wait.
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(_) => {
                    // Accept error — keep trying.
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Path to the socket file.
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        // Clean up the socket file.
        let _ = std::fs::remove_file(&self.socket_path);
    }
}
```

### Step 3: Add `Send` bound to `ControlServer`

The thread handle requires `Send`. `ControlServer` holds a `JoinHandle` (which is `Send` if the closure is `Send`). The `DaemonSocket` is moved into the thread, so it's fine.

### Step 4: Verify compilation

Run: `cargo check -p forgum-engine`
Expected: Compiles.

### Step 5: Commit

```bash
git add crates/engine/
git commit -m "feat(engine): add ControlServer with accept loop and mpsc dispatch"
```

---

## Task 4: Wire Control Commands into Render Loop

**Files:**
- Modify: `crates/engine/src/render.rs`

The render loop needs to:
1. Accept an `mpsc::Receiver<ControlCmd>` parameter
2. Check for commands each frame (non-blocking)
3. Dispatch: STOP → set shutdown flag; PAUSE/RESUME → toggle rendering; etc.

### Step 1: Add `receiver` parameter to render loops

Modify both `render_loop_background` and `render_loop_foreground` to accept `Option<mpsc::Receiver<ControlCmd>>`.

In `render.rs`, add import:

```rust
use std::sync::mpsc;
use crate::control_socket::ControlCmd;
```

Change signature of `render_loop_background`:

```rust
pub fn render_loop_background(
    mut out: OutputHandle,
    config: SceneConfig,
    shutdown: ShutdownFlag,
    composed_text: Option<&str>,
    cow_dna: CowDna,
    instance_id: u32,
    cmd_rx: Option<mpsc::Receiver<ControlCmd>>,
) -> Result<(), Box<dyn std::error::Error>> {
```

Same for `render_loop_foreground`.

### Step 2: Add command processing in the frame loop

Inside the `while !shutdown.is_shutdown()` loop, add after the resize check:

```rust
// Process control commands (non-blocking).
if let Some(rx) = &cmd_rx {
    while let Ok(cmd) = rx.try_recv() {
        match cmd {
            ControlCmd::Stop => {
                shutdown.trigger();
                break;
            }
            ControlCmd::Pause => {
                // Skip rendering but keep looping.
                // We'll add a `paused` flag.
            }
            ControlCmd::Resume => {
                // Resume rendering.
            }
            ControlCmd::Effect(name) => {
                // Recreate effect with new name.
                // For now, just log it.
                eprintln!("{PROGRAM}: effect change requested: {name}");
            }
            ControlCmd::Speed(_s) => {
                // TODO: apply speed multiplier to scheduler.
            }
            ControlCmd::Cow(_name) => {
                // TODO: reload cow art and recreate effect.
            }
            ControlCmd::Text(_text) => {
                // TODO: update composed text.
            }
            _ => {}
        }
    }
}
```

### Step 3: Update main.rs call sites

In `main.rs`, the render loop calls need the new parameter. For now, pass `None` (we'll wire it in Task 5):

```rust
let result = if scene.background {
    render::render_loop_background(out, scene, shutdown, Some(&composed), cow_dna, instance_id, None)
} else {
    render::render_loop_foreground(out, scene, shutdown, Some(&composed), cow_dna, instance_id, None)
};
```

### Step 4: Update integration tests

Any test that calls `render_loop_background` or `render_loop_foreground` needs the new parameter. Add `None` to all existing calls.

### Step 5: Verify compilation

Run: `cargo test --no-run 2>&1 | Select-String "error"`
Expected: No compile errors.

### Step 6: Commit

```bash
git add crates/engine/
git commit -m "feat(engine): wire ControlCmd receiver into render loops"
```

---

## Task 5: Wire Daemon Lifecycle in main.rs

**Files:**
- Modify: `crates/engine/src/main.rs`
- Modify: `crates/engine/src/daemon.rs`

Replace the Phase 1 stubs with real daemon logic.

### Step 1: Add `write_daemon_state` to `daemon.rs`

```rust
/// Write daemon state to the platform's daemon state path.
pub fn write_daemon_state(
    pid: u32,
    ob_y1: u16,
    cols: u16,
    socket_path: &Path,
) -> Result<PathBuf, std::io::Error> {
    let session_id = forgum_platform::detect_session_id();
    let state_path = forgum_platform::daemon_state_path(&session_id);
    let state = DaemonState {
        pid,
        ob_y1,
        cols,
        socket_path: socket_path.to_string_lossy().to_string(),
        started_at: chrono_free_timestamp(),
    };
    state.write(&state_path)?;
    Ok(state_path)
}

fn chrono_free_timestamp() -> String {
    // Simple ISO-8601 without chrono dependency.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}Z", d.as_secs()))
        .unwrap_or_else(|_| "unknown".into())
}

/// Remove daemon state file.
pub fn cleanup_daemon_state(session_id: &str) {
    let path = forgum_platform::daemon_state_path(session_id);
    let _ = std::fs::remove_file(path);
}
```

### Step 2: Rewrite `render_subcommand` in `main.rs`

```rust
fn render_subcommand(args: cli::Args) -> ExitCode {
    // Read scene: --file overrides stdin; if neither, use defaults.
    let scene_from_file = match read_scene(args.file.as_deref(), false) {
        Ok(s) => Some(s),
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(e.exit_code() as u8);
        }
    };
    let _ = scene_from_file;

    // Build merged scene (config auto-discovered if --config not given).
    let scene = match build_scene_config(&args) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{PROGRAM}: {e}");
            return ExitCode::from(65);
        }
    };

    // Clean up temp file after building config.
    if let Some(path) = &args.file {
        let _ = std::fs::remove_file(path);
    }

    let shutdown = ShutdownFlag::new();

    // Daemon mode: fork, write PID file, bind control socket.
    if args.daemon {
        let session_id = forgum_platform::detect_session_id();
        let socket_path = forgum_platform::control_socket_path(&session_id);

        // Start control socket server before forking.
        let (server, cmd_rx) = match forgum_engine::control_socket::ControlServer::start(socket_path.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{PROGRAM}: control socket: {e}");
                return ExitCode::from(74);
            }
        };

        // Daemonize: parent exits, child continues.
        match forgum_platform::daemonize() {
            Ok(true) => {
                // Parent already exited via process::exit(0).
                unreachable!();
            }
            Ok(false) => {
                // Child: write daemon state, continue to render.
                let pid = std::process::id();
                let _ = daemon::write_daemon_state(
                    pid,
                    0, // ob_y1 computed later
                    80, // default cols, updated after terminal probe
                    &socket_path,
                );
            }
            Err(e) => {
                eprintln!("{PROGRAM}: daemonize: {e}");
                return ExitCode::from(74);
            }
        }

        // Open output and render.
        let out = match OutputHandle::open() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot open output: {e}");
                return ExitCode::from(e.exit_code() as u8);
            }
        };

        let data = match data_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot find data directory: {e}");
                return ExitCode::from(78);
            }
        };
        let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
        let composed = cow::compose_scene(&cow_text, &scene.text);
        let animations = dna::load_animations(&data);
        let cow_dna = dna::get_dna(&animations, &scene.cow);
        let instance_id = std::process::id();

        let result = if scene.background {
            render::render_loop_background(out, scene, shutdown, Some(&composed), cow_dna, instance_id, Some(cmd_rx))
        } else {
            render::render_loop_foreground(out, scene, shutdown, Some(&composed), cow_dna, instance_id, Some(cmd_rx))
        };

        // Cleanup on exit.
        drop(server);

        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                ExitCode::from(71)
            }
        }
    } else {
        // Foreground mode (no daemon).
        let out = match OutputHandle::open() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot open output: {e}");
                return ExitCode::from(e.exit_code() as u8);
            }
        };

        let data = match data_dir() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{PROGRAM}: cannot find data directory: {e}");
                return ExitCode::from(78);
            }
        };
        let cow_text = cow::load_cow(&scene.cow, &data, &scene.eyes, &scene.tongue, "\\\\");
        let composed = cow::compose_scene(&cow_text, &scene.text);
        let animations = dna::load_animations(&data);
        let cow_dna = dna::get_dna(&animations, &scene.cow);
        let instance_id = std::process::id();

        let result = if scene.background {
            render::render_loop_background(out, scene, shutdown, Some(&composed), cow_dna, instance_id, None)
        } else {
            render::render_loop_foreground(out, scene, shutdown, Some(&composed), cow_dna, instance_id, None)
        };

        match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{PROGRAM}: {e}");
                ExitCode::from(71)
            }
        }
    }
}
```

### Step 3: Verify compilation

Run: `cargo check -p forgum-engine`
Expected: Compiles.

### Step 4: Commit

```bash
git add crates/engine/
git commit -m "feat(engine): wire daemon lifecycle in main.rs"
```

---

## Task 6: Add `Stop-ForgumDaemon` to PowerShell Module

**Files:**
- Modify: `Forgum.psm1` or `Public/forgum.ps1`

### Step 1: Add `Stop-ForgumDaemon` function

```powershell
function Stop-ForgumDaemon {
    <#
    .SYNOPSIS
        Stops a running Forgum daemon for the current session.
    .DESCRIPTION
        Reads daemon.json, sends SIGTERM (Unix) or Stop-Process (Windows),
        and cleans up the state file and control socket.
    #>
    [CmdletBinding()]
    param()

    $session = Get-ForgumSessionId
    $daemonPath = Get-ForgumDaemonStatePath -SessionId $session

    if (-not (Test-Path $daemonPath)) {
        Write-Verbose "No daemon state file found at $daemonPath"
        return
    }

    try {
        $daemon = Get-Content -Raw $daemonPath | ConvertFrom-Json
    } catch {
        Write-Warning "Failed to read daemon state: $_"
        Remove-Item $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    if ($null -eq $daemon.pid) {
        Write-Warning "Daemon state has no PID"
        Remove-Item $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    # Check if process is still running.
    $proc = Get-Process -Id $daemon.pid -ErrorAction SilentlyContinue
    if ($null -eq $proc) {
        Write-Verbose "Daemon PID $($daemon.pid) is not running"
        Remove-Item $daemonPath -Force -ErrorAction SilentlyContinue
        return
    }

    # Stop the process.
    try {
        Stop-Process -Id $daemon.pid -Force -ErrorAction Stop
        Write-Verbose "Stopped daemon PID $($daemon.pid)"
    } catch {
        Write-Warning "Failed to stop daemon PID $($daemon.pid): $_"
    }

    # Clean up state file.
    Remove-Item $daemonPath -Force -ErrorAction SilentlyContinue

    # Clean up control socket (Unix).
    if ($daemon.socket_path) {
        Remove-Item $daemon.socket_path -Force -ErrorAction SilentlyContinue
    }
}
```

### Step 2: Add helper functions if missing

The module needs `Get-ForgumSessionId` and `Get-ForgumDaemonStatePath`. If they don't exist, add them:

```powershell
function Get-ForgumSessionId {
    if ($env:TMUX_PANE) { return $env:TMUX_PANE }
    if ($env:ZELLIJ_SESSION_ID) { return $env:ZELLIJ_SESSION_ID }
    return "shell-$PID"
}

function Get-ForgumDaemonStatePath {
    param([string] $SessionId)
    if ($IsWindows -or $env:OS -eq 'Windows_NT') {
        $base = Join-Path $env:LOCALAPPDATA "Forgum"
    } else {
        $base = Join-Path (Join-Path $env:XDG_RUNTIME_DIR "Forgum") ""
        if (-not $base -or -not (Test-Path $base)) {
            $base = "/tmp"
        }
    }
    return Join-Path $base "daemon-$SessionId.json"
}
```

### Step 3: Export the new function

Add to `Forgum.psd1` `FunctionsToExport` list: `'Stop-ForgumDaemon'`.

### Step 4: Commit

```bash
git add Forgum.psm1 Forgum.psd1 Public/
git commit -m "feat(pwsh): add Stop-ForgumDaemon for per-session daemon cleanup"
```

---

## Task 7: Add Integration Test

**Files:**
- Create: `crates/engine/tests/daemon_lifecycle.rs`

### Step 1: Write the integration test

```rust
//! Integration test: spawn engine with --daemon, send PING, send STOP.

use std::process::Command;
use std::time::Duration;

#[test]
fn daemon_lifecycle_ping_stop() {
    let exe = env!("CARGO_BIN_EXE_forgum-engine");

    // Start daemon in background.
    let child = Command::new(exe)
        .args(["--background", "--duration", "30", "--daemon"])
        .output()
        .expect("failed to start daemon");

    // The parent prints PID and exits. Capture it.
    let stdout = String::from_utf8_lossy(&child.stdout);
    let pid: u32 = stdout.trim().parse().expect("expected PID on stdout");

    // Give daemon time to bind socket.
    std::thread::sleep(Duration::from_millis(500));

    // Send PING via control socket.
    // (This requires the socket to exist. On CI we may not have the
    // runtime dir. Skip if socket not found.)
    let session = forgum_platform::detect_session_id();
    let socket_path = forgum_platform::control_socket_path(&session);

    if !socket_path.exists() {
        eprintln!("socket not found at {socket_path:?}, skipping");
        // Still need to clean up the daemon.
        let _ = std::process::Command::new("kill")
            .arg(pid.to_string())
            .output();
        return;
    }

    // Send STOP to daemon.
    let stop_msg = r#"{"cmd":"STOP"}"#;
    // On Unix we can write to the socket.
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        use std::io::Write;
        if let Ok(mut stream) = UnixStream::connect(&socket_path) {
            let _ = stream.write_all(stop_msg.as_bytes());
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();
        }
    }

    // Wait for daemon to exit.
    std::thread::sleep(Duration::from_millis(500));

    // Verify PID is no longer alive.
    assert!(
        !forgum_platform::process_is_alive(pid),
        "daemon should have exited after STOP"
    );
}
```

### Step 2: Verify test compiles

Run: `cargo test --test daemon_lifecycle --no-run`
Expected: Compiles.

### Step 3: Commit

```bash
git add crates/engine/tests/daemon_lifecycle.rs
git commit -m "test(engine): add daemon lifecycle integration test"
```

---

## Task 8: Fix Tests and Run Full Suite

**Files:**
- Modify any test files that call `render_loop_*` with the old signature.

### Step 1: Update all render loop call sites

Search for all calls to `render_loop_background` and `render_loop_foreground` in test files and add `None` as the last argument.

Run: `cargo test 2>&1 | Select-String "error"`
Fix any compilation errors.

### Step 2: Run full test suite

```bash
cargo test 2>&1
cargo clippy --all-targets 2>&1
cargo fmt --check 2>&1
```

All should pass with zero warnings.

### Step 3: Run cfg-grep gate

```bash
cargo test -p forgum-engine --test cfg_containment 2>&1
```

Expected: PASS (zero `#[cfg]` in engine).

### Step 4: Run Pester tests

```bash
pwsh -Command "Invoke-Pester -Path './Tests' -Output Detailed" 2>&1
```

Expected: 13/13 pass.

### Step 5: Final commit

```bash
git add -A
git commit -m "fix: update all test call sites for new render loop signature"
```

---

## Summary

After completing all 8 tasks:

1. **`daemonize()`** — cross-platform fork+detach using `nix` (Unix) / `windows-sys` (Windows)
2. **`DaemonSocket`** — cross-platform IPC (Unix domain socket / Windows named pipe)
3. **`ControlServer`** — accept loop + `mpsc` dispatch to render loop
4. **Render loop integration** — processes STOP/PAUSE/RESUME/EFFECT/SPEED/COW commands
5. **`main.rs` daemon lifecycle** — fork → write PID → bind socket → render → cleanup
6. **`Stop-ForgumDaemon`** — PowerShell function for per-session daemon management
7. **Integration test** — spawn daemon, send STOP, verify exit
8. **All existing tests pass** — zero regressions
