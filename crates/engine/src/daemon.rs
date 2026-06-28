//! Daemon lifecycle: detach, PID file, per-session management.
//!
//! When `--background` is passed, the engine forks into a detached daemon
//! and writes a `daemon.json` file that the shell's `precmd` sweep and
//! `Stop-ForgumDaemon` use for cleanup.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// State persisted to `daemon.json` so the shell and `Stop-ForgumDaemon`
/// can find and manage the running daemon.
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonState {
    /// Process ID of the daemon.
    pub pid: u32,
    /// Overlay bottom row (exclusive) — for cleanup.
    pub ob_y1: u16,
    /// Terminal columns at launch.
    pub cols: u16,
    /// Path to the control socket.
    pub socket_path: String,
    /// ISO-8601 timestamp of when the daemon started.
    pub started_at: String,
}

impl DaemonState {
    /// Write the state to the given path.
    pub fn write(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Read state from a file.
    pub fn read(path: &Path) -> std::io::Result<Self> {
        let data = fs::read_to_string(path)?;
        serde_json::from_str(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Check if the daemon PID is still alive (delegates to platform crate).
    pub fn is_alive(&self) -> bool {
        forgum_platform::process_is_alive(self.pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn daemon_state_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("daemon.json");
        let state = DaemonState {
            pid: 12345,
            ob_y1: 10,
            cols: 80,
            socket_path: "/tmp/ctrl.sock".to_string(),
            started_at: "2026-06-28T21:00:00Z".to_string(),
        };
        state.write(&path).unwrap();
        let loaded = DaemonState::read(&path).unwrap();
        assert_eq!(loaded.pid, 12345);
        assert_eq!(loaded.ob_y1, 10);
        assert_eq!(loaded.cols, 80);
    }

    #[test]
    fn daemon_state_path_has_forgum_dir() {
        let path = forgum_platform::daemon_state_path("test-session");
        assert!(path.to_string_lossy().contains("daemon-test-session"));
    }

    #[test]
    fn control_socket_path_has_forgum_dir() {
        let path = forgum_platform::control_socket_path("test-session");
        assert!(path.to_string_lossy().contains("ctrl-test-session"));
    }

    #[test]
    fn session_id_not_empty() {
        let id = forgum_platform::detect_session_id();
        assert!(!id.is_empty());
    }
}
