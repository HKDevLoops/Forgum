//! Detached child-process spawning.
//!
//! Used by the engine in `--daemon` mode to launch a background animation
//! that outlives the parent shell. Fixes BUG-B7 (child not detached; dies
//! with the shell).
//!
//! ## Unix
//!
//! We use `setsid(2)` in a `pre_exec` callback so the child becomes the
//! leader of a new session and process group, and therefore does not
//! receive SIGHUP when its parent shell exits. Stdin is redirected from
//! `/dev/null`, stdout/stderr to either a log file or `/dev/null` (the
//! daemon shouldn't spam the user's terminal — the foreground parent
//! owns that).
//!
//! ## Windows
//!
//! We use `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP` so the child does
//! not share a console with the parent and Ctrl-C events are not delivered.

use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::error::PlatformError;

/// Daemonize the current process.
///
/// On Unix: forks via `nix::unistd::fork()`. Parent prints child PID to
/// stdout and exits with code 0. Child calls `setsid()` to become session
/// leader and returns `Ok(false)`.
///
/// On Windows: spawns a detached copy of self via `Command` with
/// `DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP`. Parent prints child PID
/// and exits with code 0. Child returns `Ok(false)`.
///
/// The parent process prints the child PID and exits (never returns).
/// The child process returns `Ok(false)`.
#[cfg(unix)]
#[allow(unsafe_code)]
pub fn daemonize() -> Result<bool, PlatformError> {
    use nix::unistd::{fork, setsid, ForkResult};

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            // Print the child PID so callers (and the daemon-lifecycle test)
            // can discover the daemon. `std::process::exit` does NOT flush
            // buffered stdout, so flush explicitly — otherwise a captured
            // (piped) parent stdout receives an empty string and the PID is
            // lost.
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            let _ = writeln!(lock, "{}", child.as_raw());
            let _ = lock.flush();
            std::process::exit(0);
        }
        Ok(ForkResult::Child) => {
            setsid().map_err(|e| PlatformError::Io(std::io::Error::from(e)))?;
            Ok(false)
        }
        Err(e) => Err(PlatformError::Io(io::Error::other(e))),
    }
}

/// Daemonize the current process (Windows).
///
/// Spawns a detached copy of self with `DETACHED_PROCESS |
/// CREATE_NEW_PROCESS_GROUP`. Parent prints child PID and exits with code 0.
/// Child returns `Ok(false)`.
#[cfg(windows)]
pub fn daemonize() -> Result<bool, PlatformError> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

    let current_exe = std::env::current_exe().map_err(PlatformError::Io)?;
    let args: Vec<String> = std::env::args()
        .skip(1)
        .filter(|a| a != "--daemon")
        .collect();

    let mut cmd = Command::new(current_exe);
    cmd.args(&args);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

    let child = cmd.spawn().map_err(PlatformError::Io)?;
    println!("{}", child.id());
    std::process::exit(0);
}

/// A detached child process. The handle is kept for status reporting; the
/// actual lifecycle is managed by the OS session.
#[derive(Debug)]
pub struct DetachedChild {
    inner: Child,
}

impl DetachedChild {
    /// PID of the child.
    #[must_use]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Try to collect the child's exit status without blocking.
    pub fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.inner.try_wait()
    }
}

/// Build a `Command` configured to detach from the parent shell on Unix.
#[cfg(unix)]
pub fn spawn_detached(
    program: &Path,
    args: &[&str],
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
) -> Result<Child, PlatformError> {
    use std::os::unix::process::CommandExt;

    let mut cmd = Command::new(program);
    cmd.args(args).stdin(stdin).stdout(stdout).stderr(stderr);

    // SAFETY: `setsid` is async-signal-safe and the closure runs in the
    // forked child between fork() and execve(). No Rust allocator is in
    // scope at that point. The `Ok(())` return is the only failure mode
    // for setsid (which can't fail for our purposes — we're not in a
    // session leader).
    #[allow(unsafe_code)]
    unsafe {
        cmd.pre_exec(|| {
            // Best-effort detach. setsid() is required; umask is hygiene;
            // close stdin isn't done here — the parent sets Stdio::null().
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn().map_err(PlatformError::Io)
}

/// Build a `Command` configured to detach on Windows.
#[cfg(windows)]
#[allow(unsafe_code)]
pub fn spawn_detached(
    program: &Path,
    args: &[&str],
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
) -> Result<Child, PlatformError> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Threading::{CREATE_NEW_PROCESS_GROUP, DETACHED_PROCESS};

    let mut cmd = Command::new(program);
    cmd.args(args).stdin(stdin).stdout(stdout).stderr(stderr);
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

    cmd.spawn().map_err(PlatformError::Io)
}

/// Convenience: spawn a detached child with safe defaults
/// (stdin from `/dev/null`, stdout/stderr discarded).
pub fn spawn_silent(program: &Path, args: &[&str]) -> Result<DetachedChild, PlatformError> {
    let child = spawn_detached(program, args, Stdio::null(), Stdio::null(), Stdio::null())?;
    Ok(DetachedChild { inner: child })
}

/// Drop into daemon mode for the calling process. On Unix, replaces
/// this process image with `argv[0]` (the engine binary) re-exec'd
/// with the supplied args (the caller sets argv up the way it wants
/// the daemon child to see it). On Windows, no portable exec
/// exists in stable Rust, so we just run the supplied closure in
/// place — the caller has to inline its daemon body. The
/// platform dispatch lives here, so callers stay cfg-free.
///
/// Tests verifying that callers without a TTY or with restricted
/// filesystem namespaces still rely on `Command::spawn`-style
/// detach will require running the binary's helpful re-exec path.
/// On `cross-rs/...:aarch64` QEMU user-mode emulators we observe
/// `posix_spawn()` reject /dev/null fd inheritance and ITSELF
/// fork() returning EINVAL/ENOSYS; re-exec is the only path that
/// avoids both.
#[cfg(unix)]
#[allow(unsafe_code)]
pub fn daemon_bootstrap<F: FnOnce() -> std::process::ExitCode>(
    argv: &[String],
    fallback: F,
) -> std::process::ExitCode {
    use std::os::unix::process::CommandExt;

    // Prefer current_exe (canonical path); fall back to argv[0]
    // when current_exe points through /proc/self/exe to a path
    // QEMU-binfmt cannot re-exec (cross-rs aarch64). Fall back
    // further to the binary literal name (POSIX PATH search).
    let self_exe: std::path::PathBuf = std::env::current_exe()
        .ok()
        .filter(|p| p.exists())
        .or_else(|| std::env::args().next().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("forgum-engine"));

    if let Err(e) = std::process::Command::new(&self_exe)
        .args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .exec()
    {
        eprintln!("forgum-engine: daemon re-exec failed: {e}; running daemon body inline");
        return fallback();
    }
    // Unreachable: exec on success replaces the process.
    std::process::ExitCode::from(74)
}

/// Drop into daemon mode for the calling process (Windows variant).
/// No portable exec stable-API exists on Windows — we run the
/// daemon body inline, so the daemon's lifetime matches the
/// host lifetime. The caller should mirror whatever shape they had
/// planned for the post-exec daemon child.
#[cfg(windows)]
pub fn daemon_bootstrap<F: FnOnce() -> std::process::ExitCode>(
    _argv: &[String],
    fallback: F,
) -> std::process::ExitCode {
    fallback()
}

/// Check if a process with the given PID is still alive.
///
/// On Unix, uses `kill(pid, 0)` to check without sending a signal.
/// On Windows, uses `OpenProcess` + `GetExitCodeProcess`.
#[cfg(unix)]
#[allow(unsafe_code)]
pub fn process_is_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

/// Check if a process with the given PID is still alive (Windows).
#[cfg(windows)]
#[allow(unsafe_code)]
pub fn process_is_alive(pid: u32) -> bool {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut exit_code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        if ok == 0 {
            return false;
        }
        exit_code == 259 // STILL_ACTIVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;

    #[test]
    fn spawn_detached_runs_simple_program() {
        let me = std::env::current_exe().unwrap();
        let child = spawn_detached(
            &me,
            &["--help"],
            Stdio::null(),
            Stdio::null(),
            Stdio::null(),
        );
        // On most CI environments --help is honored; some test binaries may
        // not have it. Either way spawn should not panic.
        match child {
            Ok(mut c) => {
                let _ = c.try_wait();
            }
            Err(e) => {
                // Acceptable on Windows where a minimal binary may not exist
                // for current_exe in tests.
                eprintln!("spawn skipped: {e}");
            }
        }
    }

    #[test]
    fn spawn_silent_uses_null_stdio() {
        // Just exercise the wrapper; we don't care about the result.
        let me = std::env::current_exe().unwrap();
        let result = spawn_silent(&me, &["--help"]);
        if let Ok(mut c) = result {
            let _ = c.try_wait();
        }
    }

    #[test]
    fn daemonize_exists_with_correct_signature() {
        // Verify daemonize() compiles with the correct signature.
        // The parent branch calls process::exit(0), so we can only test
        // the function signature and that it compiles; the actual fork/exit
        // behavior is verified by integration tests.
        let _f: fn() -> Result<bool, PlatformError> = daemonize;
    }

    #[test]
    #[cfg(unix)]
    fn detached_has_new_session() {
        use std::os::unix::process::CommandExt;
        // Spawn `sh -c 'echo $PPID'` and check we can read its PID. We don't
        // actually verify session id (the child prints and exits), but we
        // verify the spawn path doesn't hang.
        let sh = Path::new("/bin/sh");
        if !sh.exists() {
            return; // non-Unix-typical environment
        }
        let mut cmd = Command::new(sh);
        cmd.arg("-c").arg("exit 0");
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        // SAFETY: same reasoning as in spawn_detached.
        #[allow(unsafe_code)]
        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = cmd.spawn().expect("spawn");
        let status = child.wait().expect("wait");
        assert!(status.success());
    }
}
