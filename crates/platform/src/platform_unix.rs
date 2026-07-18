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
