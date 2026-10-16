//! Reactive Living Pasture shell telemetry bus.
//!
//! Receives ambient shell hook events (command exit codes, elapsed execution time,
//! Git repository status) and updates active background daemons in real-time.

use std::process::ExitCode;

use forgum_platform::{control_socket_path, daemon_state_path, detect_session_id};

use crate::control_socket::ControlCmd;
use crate::remote::send_control_cmd;

/// Report shell command execution telemetry to the active daemon.
pub fn report_shell_telemetry(
    session: Option<&str>,
    exit_code: i32,
    elapsed_ms: Option<u64>,
    _command: Option<&str>,
) -> ExitCode {
    let session = session
        .map(String::from)
        .unwrap_or_else(detect_session_id);
    let state_file = daemon_state_path(&session);

    // If no daemon state file exists for this session, exit immediately with zero latency.
    if !state_file.exists() {
        return ExitCode::SUCCESS;
    }

    let socket_path = control_socket_path(&session);
    let socket_str = match socket_path.to_str() {
        Some(s) => s,
        None => return ExitCode::SUCCESS,
    };

    if exit_code != 0 {
        // Shift mascot eyes to "xx" on command failure
        let _ = send_control_cmd(socket_str, &ControlCmd::Eyes("xx".to_string()));

        // If execution took significant time, show duration in speech/thought bubble
        if let Some(ms) = elapsed_ms {
            if ms >= 1000 {
                let secs = ms as f64 / 1000.0;
                let _ = send_control_cmd(
                    socket_str,
                    &ControlCmd::Think(format!("Failed after {secs:.1}s")),
                );
            }
        }
    } else {
        // Restore happy/normal eyes on success
        let _ = send_control_cmd(socket_str, &ControlCmd::Eyes("oo".to_string()));

        // Check if git in current directory has uncommitted changes
        if is_current_dir_git_dirty() {
            let _ = send_control_cmd(
                socket_str,
                &ControlCmd::Think("git repo has uncommitted changes 🌿".to_string()),
            );
        }
    }

    ExitCode::SUCCESS
}

/// Fast check whether the current working directory is inside a dirty Git repository.
pub fn is_current_dir_git_dirty() -> bool {
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match out {
        Ok(o) if o.status.success() => {
            let s = String::from_utf8_lossy(&o.stdout);
            !s.trim().is_empty()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_telemetry_no_daemon_runs_cleanly() {
        let code = report_shell_telemetry(None, 0, None, None);
        assert_eq!(code, ExitCode::SUCCESS);

        let code_fail = report_shell_telemetry(None, 1, Some(1500), Some("cargo build"));
        assert_eq!(code_fail, ExitCode::SUCCESS);
    }

    #[test]
    fn test_git_dirty_check_does_not_panic() {
        let _ = is_current_dir_git_dirty();
    }
}
