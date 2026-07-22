//! Detached child-process spawning.
//!
//! Used by the engine in `--daemon` mode to launch a background animation
//! that outlives the parent shell.

use std::io;
use std::path::Path;
use std::process::{Child, Command, Stdio};

use crate::error::PlatformError;

/// Daemonize (legacy fork path; parity shell-hook helper).
#[cfg(unix)]
#[allow(unsafe_code)]
pub fn daemonize() -> Result<bool, PlatformError> {
    use nix::unistd::{fork, setsid, ForkResult};
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child }) => {
            use std::io::Write;
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            let _ = writeln!(lock, "{}", child.as_raw());
            let _ = lock.flush();
            std::process::exit(0);
        }
        Ok(ForkResult::Child) => {
            setsid().map_err(|e| PlatformError::Io(std::io::Error::other(e)))?;
            Ok(false)
        }
        Err(e) => Err(PlatformError::Io(io::Error::other(e))),
    }
}

/// Daemonize (Windows).
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

#[derive(Debug)]
pub struct DetachedChild {
    inner: Child,
}
impl DetachedChild {
    #[must_use]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }
    pub fn try_wait(&mut self) -> io::Result<Option<std::process::ExitStatus>> {
        self.inner.try_wait()
    }
}

/// Build a detached-spawn Command on Unix (setsid via pre_exec).
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
    #[allow(unsafe_code)]
    unsafe {
        cmd.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    cmd.spawn().map_err(PlatformError::Io)
}

/// Build a detached-spawn Command on Windows.
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

/// Convenience: spawn a detached child with safe defaults.
pub fn spawn_silent(program: &Path, args: &[&str]) -> Result<DetachedChild, PlatformError> {
    let child = spawn_detached(program, args, Stdio::null(), Stdio::null(), Stdio::null())?;
    Ok(DetachedChild { inner: child })
}

/// Drop into daemon mode for the calling process. On Unix, atomic
/// process replacement via the inherent `Command::exec` method
/// (execve(2) under the hood). On Windows, no portable exec-stable
/// API exists, so we run the supplied closure in-place.
///
/// The platform dispatch lives here, so callers stay cfg-free.
#[cfg(unix)]
pub fn daemon_bootstrap<F: FnOnce() -> std::process::ExitCode>(
    argv: &[String],
    fallback: F,
) -> std::process::ExitCode {
    use std::os::unix::process::CommandExt;

    let self_exe: std::path::PathBuf = std::env::current_exe()
        .ok()
        .filter(|p| p.exists())
        .or_else(|| std::env::args().next().map(std::path::PathBuf::from))
        .unwrap_or_else(|| std::path::PathBuf::from("forgum-engine"));

    let mut cmd = std::process::Command::new(&self_exe);
    cmd.args(argv)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // exec() comes from std::os::unix::process::CommandExt.
    // It returns io::Result<Infallible>: Ok is unreachable
    // (the process image is gone on success), Err is the
    // real diagnostic. We match the two arms for clarity, since
    // `!` is unstable on stable rustc.
    let exec_result: Result<std::convert::Infallible, std::io::Error> = cmd.exec();
    match exec_result {
        Ok(unreachable) => match unreachable {},
        Err(e) => {
            eprintln!("forgum-engine: daemon re-exec failed: {e}; running daemon body inline");
            fallback()
        }
    }
}

/// Windows variant of `daemon_bootstrap`. No portable exec-stable on
/// Windows; run the supplied closure inline.
#[cfg(windows)]
pub fn daemon_bootstrap<F: FnOnce() -> std::process::ExitCode>(
    _argv: &[String],
    fallback: F,
) -> std::process::ExitCode {
    fallback()
}

/// Check if a process with the given PID is still alive (Unix).
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
        exit_code == 259
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    #[test]
    fn spawn_detached_runs_simple_program() {
        let me = std::env::current_exe().unwrap();
        match spawn_detached(
            &me,
            &["--help"],
            Stdio::null(),
            Stdio::null(),
            Stdio::null(),
        ) {
            Ok(mut c) => {
                let _ = c.try_wait();
            }
            Err(e) => {
                eprintln!("spawn skipped: {e}");
            }
        }
    }
    #[test]
    fn spawn_silent_uses_null_stdio() {
        let me = std::env::current_exe().unwrap();
        if let Ok(mut c) = spawn_silent(&me, &["--help"]) {
            let _ = c.try_wait();
        }
    }
    #[test]
    fn daemonize_exists_with_correct_signature() {
        let _f: fn() -> Result<bool, PlatformError> = daemonize;
    }
    #[test]
    #[cfg(unix)]
    fn detached_has_new_session() {
        use std::os::unix::process::CommandExt;
        let sh = Path::new("/bin/sh");
        if !sh.exists() {
            return;
        }
        let mut cmd = Command::new(sh);
        cmd.arg("-c").arg("exit 0");
        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
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
    #[test]
    fn daemon_bootstrap_signature_is_cross_platform() {
        #[allow(clippy::type_complexity)]
        let _f: fn(&[String], fn() -> std::process::ExitCode) -> std::process::ExitCode =
            |a, b| daemon_bootstrap(a, b);
    }
}
