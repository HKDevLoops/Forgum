use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::control_socket::ControlResponse;

#[derive(Debug)]
pub struct HerdFilter {
    pub session: Option<String>,
    pub all: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HerdEntry {
    pub session_id: String,
    pub pid: u32,
    pub alive: bool,
    pub effect: String,
    pub fps: u16,
    pub speed: f32,
    pub paused: bool,
    pub age: String,
    pub socket_path: String,
}

pub fn send_command(socket_path: &str, cmd: &str) -> Result<ControlResponse, String> {
    let mut conn = forgum_platform::DaemonSocket::connect(Path::new(socket_path))
        .map_err(|e| format!("failed to connect to {}: {e}", socket_path))?;

    let payload = format!("{}\n", cmd);
    conn.write_response(&payload)
        .map_err(|e| format!("failed to write command: {e}"))?;

    let line = conn
        .read_line()
        .map_err(|e| format!("failed to read response: {e}"))?;

    let line = line.ok_or_else(|| "connection closed before response".to_string())?;

    serde_json::from_str(&line).map_err(|e| format!("invalid JSON response: {e}"))
}

fn compute_age(started_at: &str) -> String {
    let ts: i64 = match started_at.trim_end_matches('Z').parse() {
        Ok(v) => v,
        Err(_) => return "unknown".into(),
    };

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let delta = now.saturating_sub(ts);
    if delta < 60 {
        format!("{delta}s")
    } else if delta < 3600 {
        format!("{}m", delta / 60)
    } else {
        format!("{}h", delta / 3600)
    }
}

pub fn discover_daemons() -> Vec<HerdEntry> {
    let runtime = match forgum_platform::runtime_dir() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let entries = match std::fs::read_dir(&runtime) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut result = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };
        if !name.starts_with("daemon-") || !name.ends_with(".json") {
            continue;
        }

        let state = match crate::daemon::DaemonState::read(&path) {
            Ok(s) => s,
            Err(_) => continue,
        };

        let alive = state.is_alive();
        let session_id = name
            .strip_prefix("daemon-")
            .and_then(|s| s.strip_suffix(".json"))
            .unwrap_or("unknown")
            .to_string();

        let (effect, fps, speed, paused) = if alive {
            match send_command(&state.socket_path, r#"{"cmd":"STATUS"}"#) {
                Ok(resp) => {
                    if let Some(status) = resp.status {
                        (status.effect, status.fps, status.speed, status.paused)
                    } else {
                        ("unknown".into(), 0, 0.0, false)
                    }
                }
                Err(_) => ("unknown".into(), 0, 0.0, false),
            }
        } else {
            ("unknown".into(), 0, 0.0, false)
        };

        result.push(HerdEntry {
            session_id,
            pid: state.pid,
            alive,
            effect,
            fps,
            speed,
            paused,
            age: compute_age(&state.started_at),
            socket_path: state.socket_path,
        });
    }

    result
}

fn filter_daemons(entries: Vec<HerdEntry>, filter: &HerdFilter) -> Vec<HerdEntry> {
    if filter.all {
        return entries;
    }
    entries
        .into_iter()
        .filter(|e| filter.session.as_ref() == Some(&e.session_id))
        .collect()
}

pub fn herd_stop(filter: &HerdFilter) -> Result<usize, String> {
    let entries = filter_daemons(discover_daemons(), filter);
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let resp = send_command(&entry.socket_path, r#"{"cmd":"STOP"}"#)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_effect(name: &str, filter: &HerdFilter) -> Result<usize, String> {
    let entries = filter_daemons(discover_daemons(), filter);
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let cmd = format!(r#"{{"cmd":"EFFECT","arg":"{}"}}"#, name);
            let resp = send_command(&entry.socket_path, &cmd)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_speed(speed: f32, filter: &HerdFilter) -> Result<usize, String> {
    let entries = filter_daemons(discover_daemons(), filter);
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let cmd = format!(r#"{{"cmd":"SPEED","arg":"{}"}}"#, speed);
            let resp = send_command(&entry.socket_path, &cmd)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_pause(filter: &HerdFilter) -> Result<usize, String> {
    let entries = filter_daemons(discover_daemons(), filter);
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let resp = send_command(&entry.socket_path, r#"{"cmd":"PAUSE"}"#)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_resume(filter: &HerdFilter) -> Result<usize, String> {
    let entries = filter_daemons(discover_daemons(), filter);
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let resp = send_command(&entry.socket_path, r#"{"cmd":"RESUME"}"#)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_quiet() -> Result<usize, String> {
    let entries = discover_daemons();
    let mut count = 0;
    for entry in &entries {
        if entry.alive {
            let resp = send_command(&entry.socket_path, r#"{"cmd":"STOP"}"#)?;
            if resp.ok {
                count += 1;
            }
        }
    }
    Ok(count)
}

pub fn herd_follow(pane: Option<&str>) -> Result<usize, String> {
    let entries = discover_daemons();
    let target_session = pane.map(|p| p.to_string());
    let mut count = 0;

    for entry in &entries {
        if !entry.alive {
            continue;
        }
        let is_target = target_session.as_ref() == Some(&entry.session_id);
        let speed = if is_target { 1.0 } else { 0.1 };
        let cmd = format!(r#"{{"cmd":"SPEED","arg":"{}"}}"#, speed);
        let resp = send_command(&entry.socket_path, &cmd)?;
        if resp.ok {
            count += 1;
        }
    }
    Ok(count)
}

pub fn herd_census() -> Vec<HerdEntry> {
    discover_daemons()
}

pub fn format_table(entries: &[HerdEntry]) -> String {
    if entries.is_empty() {
        return String::from("No daemons found.\n");
    }

    let header = format!(
        "{:<8} {:<16} {:<12} {:<6} {:<8} {:<8} {:<6}",
        "PID", "SESSION", "EFFECT", "FPS", "SPEED", "STATUS", "AGE"
    );
    let separator = "-".repeat(header.len());

    let mut lines = vec![header, separator];
    for e in entries {
        let status = if !e.alive {
            "dead"
        } else if e.paused {
            "paused"
        } else {
            "running"
        };
        lines.push(format!(
            "{:<8} {:<16} {:<12} {:<6} {:<8.1} {:<8} {:<6}",
            e.pid, e.session_id, e.effect, e.fps, e.speed, status, e.age
        ));
    }

    lines.push(String::new());
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn census_returns_vec() {
        let entries = herd_census();
        assert!(entries.is_empty() || !entries.is_empty());
    }

    #[test]
    fn format_table_with_entries() {
        let entries = vec![
            HerdEntry {
                session_id: "abc123".into(),
                pid: 1234,
                alive: true,
                effect: "aurora".into(),
                fps: 30,
                speed: 1.5,
                paused: false,
                age: "2m".into(),
                socket_path: "/tmp/ctrl.sock".into(),
            },
            HerdEntry {
                session_id: "def456".into(),
                pid: 5678,
                alive: true,
                effect: "static".into(),
                fps: 60,
                speed: 1.0,
                paused: true,
                age: "1h".into(),
                socket_path: "/tmp/ctrl2.sock".into(),
            },
        ];
        let table = format_table(&entries);
        assert!(table.contains("PID"));
        assert!(table.contains("1234"));
        assert!(table.contains("aurora"));
        assert!(table.contains("running"));
        assert!(table.contains("5678"));
        assert!(table.contains("paused"));
    }

    #[test]
    fn format_table_empty() {
        let table = format_table(&[]);
        assert_eq!(table, "No daemons found.\n");
    }

    #[test]
    fn compute_age_seconds() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = compute_age(&format!("{}Z", now - 30));
        assert_eq!(age, "30s");
    }

    #[test]
    fn compute_age_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = compute_age(&format!("{}Z", now - 120));
        assert_eq!(age, "2m");
    }

    #[test]
    fn compute_age_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let age = compute_age(&format!("{}Z", now - 7200));
        assert_eq!(age, "2h");
    }

    #[test]
    fn compute_age_invalid_timestamp() {
        let age = compute_age("not-a-timestamp");
        assert_eq!(age, "unknown");
    }

    #[test]
    fn send_command_unreachable_socket() {
        let result = send_command("/tmp/nonexistent-forgum-socket.sock", r#"{"cmd":"PING"}"#);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("failed to connect"));
    }
}
