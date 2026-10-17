//! Unix-specific platform helpers.
//!
//! This file is the **only** place in the workspace outside `lib.rs` where
//! `#[cfg(unix)]` is allowed to appear. Everything else uses the
//! `cfg_unix!` / `cfg_windows!` macros re-exported from the crate root.

/// Portable handle-count metric for leak soak tests (G2/D2).
///
/// On Linux this is the open fd count (`/proc/self/fd`); on other Unixes it
/// degrades to `None` (callers fall back to a weaker signal).
#[cfg(target_family = "unix")]
#[must_use]
pub fn handle_count() -> Option<usize> {
    proc_self_fd_count()
}

/// Returns the number of currently open file descriptors for this process,
/// by counting entries in `/proc/self/fd`. Used by the daemon leak soak test
/// as the authoritative "are handles leaking?" signal (G2/D2).
///
/// Returns `None` when `/proc` is unavailable (non-Linux Unixes, or a sandbox
/// that hides it). Callers fall back to a weaker metric in that case.
#[cfg(target_family = "unix")]
#[must_use]
pub fn proc_self_fd_count() -> Option<usize> {
    #[cfg(target_os = "linux")]
    {
        let mut count = 0usize;
        let mut dir = std::fs::read_dir("/proc/self/fd").ok()?;
        while dir.next().is_some() {
            count += 1;
        }
        Some(count)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = std::process::id();
        None
    }
}

/// Returns the current process ID.
#[cfg(target_family = "unix")]
#[must_use]
pub fn current_pid() -> u32 {
    std::process::id()
}

/// Returns the parent process ID by reading `/proc/<pid>/stat`.
/// Returns `None` if `/proc` is unavailable or the entry is malformed.
#[cfg(target_family = "unix")]
pub fn parent_pid() -> Option<u32> {
    #[cfg(target_os = "linux")]
    {
        let me = std::process::id();
        let stat = std::fs::read_to_string(format!("/proc/{me}/stat")).ok()?;
        // The format is: pid (comm) state ppid ...
        // comm may contain spaces and parens; find the LAST `)` to handle that.
        let close_paren = stat.rfind(')')?;
        let after = &stat[close_paren + 1..];
        let mut fields = after.split_whitespace();
        // field 0 = state, field 1 = ppid (but the index is off by one because
        // the leading pid and comm are removed by the closing-paren trick).
        fields.next()?; // state
        fields.next()?.parse().ok()
    }
    #[cfg(target_os = "macos")]
    {
        // macOS doesn't have /proc; use libproc or sysctl. We use a minimal
        // sysctl wrapper for portability.
        use std::process::Command;
        let output = Command::new("/bin/ps")
            .args(["-o", "ppid=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;
        let s = std::str::from_utf8(&output.stdout).ok()?;
        s.trim().parse().ok()
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

/// Read the name of the parent process (e.g. "bash", "zsh", "fish", "pwsh").
/// Used by `forgum init` to detect the current shell.
#[cfg(target_family = "unix")]
pub fn parent_comm() -> Option<String> {
    let ppid = parent_pid()?;
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string(format!("/proc/{ppid}/comm")).ok()?;
        Some(s.trim().to_string())
    }
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("/bin/ps")
            .args(["-o", "comm=", "-p", &ppid.to_string()])
            .output()
            .ok()?;
        let s = std::str::from_utf8(&output.stdout).ok()?;
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = ppid;
        None
    }
}

/// Check if stdin has data available to read without blocking indefinitely.
#[cfg(target_family = "unix")]
#[allow(unsafe_code)]
pub fn stdin_has_data() -> bool {
    use std::io;
    use std::os::unix::io::AsRawFd;

    if crossterm::tty::IsTty::is_tty(&io::stdin()) {
        return false;
    }

    let fd = io::stdin().as_raw_fd();
    let mut pfd = libc::pollfd {
        fd,
        events: libc::POLLIN,
        revents: 0,
    };
    let ret = unsafe { libc::poll(&mut pfd, 1, 0) };
    ret > 0 && (pfd.revents & libc::POLLIN) != 0
}

/// Returns `true` if the parent shell process is currently the active foreground
/// process in the terminal. If an editor (e.g. `nvim`, `vim`, `emacs`, `nano`)
/// or another foreground command is running in the shell, this returns `false`.
#[cfg(target_family = "unix")]
#[allow(unsafe_code)]
#[must_use]
pub fn is_parent_shell_in_foreground(parent_pid: Option<u32>) -> bool {
    let ppid = match parent_pid {
        Some(pid) if pid > 0 => pid as libc::pid_t,
        _ => return true,
    };

    // First check: does the parent shell process still exist?
    let shell_pgid = unsafe { libc::getpgid(ppid) };
    if shell_pgid < 0 {
        // Parent process no longer exists or access denied
        return false;
    }

    // Determine tty fd to check foreground process group
    let tty_fd = if unsafe { libc::isatty(libc::STDIN_FILENO) } == 1 {
        libc::STDIN_FILENO
    } else if unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1 {
        libc::STDOUT_FILENO
    } else if unsafe { libc::isatty(libc::STDERR_FILENO) } == 1 {
        libc::STDERR_FILENO
    } else {
        -1
    };

    let fg_pgid = if tty_fd >= 0 {
        unsafe { libc::tcgetpgrp(tty_fd) }
    } else {
        // Try opening /dev/tty with O_CLOEXEC
        let dev_tty = unsafe { libc::open(c"/dev/tty".as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
        if dev_tty >= 0 {
            let pgrp = unsafe { libc::tcgetpgrp(dev_tty) };
            unsafe { libc::close(dev_tty) };
            pgrp
        } else {
            -1
        }
    };

    if fg_pgid > 0 {
        // If foreground pgrp matches shell's pgrp, the shell itself is in the foreground!
        // When an interactive editor or command runs in the foreground, fg_pgid != shell_pgid.
        return fg_pgid == shell_pgid;
    }

    // Fallback on Linux: check if parent process has running/sleeping child processes
    #[cfg(target_os = "linux")]
    {
        if let Ok(children_str) =
            std::fs::read_to_string(format!("/proc/{ppid}/task/{ppid}/children"))
        {
            for child_pid_str in children_str.split_whitespace() {
                if let Ok(cpid) = child_pid_str.parse::<u32>() {
                    if let Ok(stat) = std::fs::read_to_string(format!("/proc/{cpid}/stat")) {
                        let parts: Vec<&str> = stat.split_whitespace().collect();
                        if parts.len() > 2
                            && (parts[2] == "R" || parts[2] == "S" || parts[2] == "D")
                        {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

#[cfg(test)]
#[cfg(target_family = "unix")]
mod tests {
    use super::*;

    #[test]
    fn current_pid_is_nonzero() {
        assert!(current_pid() > 0);
    }

    #[test]
    fn parent_pid_is_something() {
        // On every Unix test environment we should have a parent PID.
        assert!(parent_pid().is_some());
    }
}
