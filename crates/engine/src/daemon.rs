//! Daemon lifecycle: detach, PID file, per-session management.
//!
//! When `--background` is passed, the engine forks into a detached daemon
//! and writes a `daemon.json` file that the shell's `precmd` sweep and
//! `Stop-ForgumDaemon` use for cleanup.

use std::fs;
use std::path::{Path, PathBuf};

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
        if let Ok(meta) = std::fs::symlink_metadata(path) {
            if meta.file_type().is_symlink() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "refusing to write to a symlink",
                ));
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;
        std::io::Write::write_all(&mut std::io::BufWriter::new(file), json.as_bytes())
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
    if let Ok(rt) = forgum_platform::runtime_dir() {
        let _ = state.write(&rt.join("daemon.json"));
    }
    Ok(state_path)
}

fn chrono_free_timestamp() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("{}Z", d.as_secs()))
        .unwrap_or_else(|_| "unknown".into())
}

/// Remove daemon state file.
pub fn cleanup_daemon_state(session_id: &str) {
    let path = forgum_platform::daemon_state_path(session_id);
    let _ = std::fs::remove_file(path);
    if let Ok(rt) = forgum_platform::runtime_dir() {
        let _ = std::fs::remove_file(rt.join("daemon.json"));
    }
}

/// Stop any active daemon running in the current session and reset terminal margins.
pub fn stop_session_daemon_and_reset_margins() {
    let session_id = forgum_platform::detect_session_id();
    let path = forgum_platform::daemon_state_path(&session_id);
    let mut ob_y1: u16 = 0;
    let mut had_daemon = false;
    if path.exists() {
        if let Ok(state) = DaemonState::read(&path) {
            ob_y1 = state.ob_y1;
            had_daemon = true;
            if state.is_alive() {
                let _ = crate::herd::send_command(&state.socket_path, r#"{"cmd":"STOP"}"#);
                std::thread::sleep(std::time::Duration::from_millis(60));
                if state.is_alive() {
                    let _ = forgum_platform::kill_process(state.pid);
                }
            }
        }
        cleanup_daemon_state(&session_id);
    } else if let Ok(rt) = forgum_platform::runtime_dir() {
        let generic = rt.join("daemon.json");
        if generic.exists() {
            if let Ok(state) = DaemonState::read(&generic) {
                ob_y1 = state.ob_y1;
                had_daemon = true;
                if state.is_alive() {
                    let _ = crate::herd::send_command(&state.socket_path, r#"{"cmd":"STOP"}"#);
                    std::thread::sleep(std::time::Duration::from_millis(60));
                    if state.is_alive() {
                        let _ = forgum_platform::kill_process(state.pid);
                    }
                }
            }
            let _ = std::fs::remove_file(&generic);
        }
    }

    if had_daemon {
        let (_, rows) = forgum_platform::terminal_size();
        let mut clean_buf = Vec::new();
        if ob_y1 > 0 {
            for y in 1..=(ob_y1 as usize).min(60) {
                clean_buf.extend_from_slice(format!("\x1b[{y};1H\x1b[2K").as_bytes());
            }
        }
        clean_buf.extend_from_slice(b"\x1b[r\x1b[?6l\x1b[?69l\x1b[0m\x1b[?25h");
        let target_row = (ob_y1 + 1).min(rows.max(1));
        clean_buf.extend_from_slice(format!("\x1b[{target_row};1H\x1b[J").as_bytes());
        let _ = std::io::Write::write_all(&mut std::io::stdout(), &clean_buf);
        let _ = std::io::Write::flush(&mut std::io::stdout());
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
