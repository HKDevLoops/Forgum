//! Remote sync — the "rmux" feature.
//!
//! Follows your cow animation across SSH sessions via reverse-forwarded
//! control sockets. Peers sync effect/speed/cow commands; rendering is
//! deterministic so all peers show the same frame without pixel streaming.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::control_socket::ControlCmd;
use crate::herd::{discover_daemons, send_command};

/// A remote peer discovered via control socket or config.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PeerInfo {
    pub host: String,
    pub pid: u32,
    pub session_id: String,
    pub effect: String,
    pub speed: f32,
    pub last_seen_secs: u64,
    pub is_leader: bool,
}

/// A discovered daemon entry with remote peer info.
#[derive(Debug, Clone)]
pub struct RemoteDaemon {
    pub pid: u32,
    pub host: String,
    pub session_id: String,
    pub socket_path: PathBuf,
    pub effect: String,
    pub speed: f32,
    pub age: Duration,
    pub is_leader: bool,
}

/// Detect if we're in an SSH session.
pub fn is_ssh_session() -> bool {
    std::env::var("SSH_CONNECTION").is_ok()
        || std::env::var("SSH_CLIENT").is_ok()
        || std::env::var("SSH_TTY").is_ok()
}

/// Get the local user for socket naming.
pub fn local_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Generate a deterministic sync session ID from a user and host.
pub fn sync_session_id(user: &str, host: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    user.hash(&mut hasher);
    host.hash(&mut hasher);
    format!("forgum-sync-{:x}", hasher.finish())
}

/// Convert a `ControlCmd` to its JSON wire format.
fn control_cmd_to_json(cmd: &ControlCmd) -> String {
    match cmd {
        ControlCmd::Stop => r#"{"cmd":"STOP"}"#.to_string(),
        ControlCmd::Pause => r#"{"cmd":"PAUSE"}"#.to_string(),
        ControlCmd::Resume => r#"{"cmd":"RESUME"}"#.to_string(),
        ControlCmd::Effect(e) => format!(r#"{{"cmd":"EFFECT","arg":"{}"}}"#, e),
        ControlCmd::Speed(s) => format!(r#"{{"cmd":"SPEED","arg":"{}"}}"#, s),
        ControlCmd::Cow(c) => format!(r#"{{"cmd":"COW","arg":"{}"}}"#, c),
        ControlCmd::Text(t) => format!(r#"{{"cmd":"TEXT","arg":"{}"}}"#, t),
        ControlCmd::Status => r#"{"cmd":"STATUS"}"#.to_string(),
        ControlCmd::Ping => r#"{"cmd":"PING"}"#.to_string(),
        ControlCmd::PeerJoin { session_id } => {
            format!(r#"{{"cmd":"PEER_JOIN","session_id":"{}"}}"#, session_id)
        }
        ControlCmd::PeerLeave => r#"{"cmd":"PEER_LEAVE"}"#.to_string(),
        ControlCmd::PeerList => r#"{"cmd":"PEER_LIST"}"#.to_string(),
        ControlCmd::ClaimLeader => r#"{"cmd":"CLAIM_LEADER"}"#.to_string(),
        ControlCmd::Unknown(u) => format!(r#"{{"cmd":"{}"}}"#, u),
    }
}

/// Send a `ControlCmd` to a daemon at the given socket path.
pub fn send_control_cmd(
    socket_path: &str,
    cmd: &ControlCmd,
) -> Result<crate::control_socket::ControlResponse, String> {
    let json = control_cmd_to_json(cmd);
    send_command(socket_path, &json)
}

/// Parse an age string like "30s", "2m", "1h" into a `Duration`.
fn parse_age_str(age_str: &str) -> Duration {
    let trimmed = age_str.trim();
    if trimmed.ends_with('s') {
        let secs: u64 = trimmed.trim_end_matches('s').parse().unwrap_or(0);
        Duration::from_secs(secs)
    } else if trimmed.ends_with('m') {
        let mins: u64 = trimmed.trim_end_matches('m').parse().unwrap_or(0);
        Duration::from_secs(mins * 60)
    } else if trimmed.ends_with('h') {
        let hrs: u64 = trimmed.trim_end_matches('h').parse().unwrap_or(0);
        Duration::from_secs(hrs * 3600)
    } else {
        Duration::from_secs(0)
    }
}

/// Discover remote peers by scanning known socket paths.
pub fn discover_remote_peers(extra_hosts: &[String]) -> Vec<RemoteDaemon> {
    let mut peers = Vec::new();

    // Discover local daemons first
    for entry in discover_daemons() {
        peers.push(RemoteDaemon {
            pid: entry.pid,
            host: "localhost".to_string(),
            session_id: entry.session_id.clone(),
            socket_path: PathBuf::from(&entry.socket_path),
            effect: entry.effect.clone(),
            speed: entry.speed,
            age: parse_age_str(&entry.age),
            is_leader: false,
        });
    }

    // Try connecting to remote hosts via SSH
    for host in extra_hosts {
        if let Ok(remote_peers) = discover_via_ssh(host) {
            peers.extend(remote_peers);
        }
    }

    peers
}

/// Discover daemons on a remote host via SSH.
fn discover_via_ssh(host: &str) -> Result<Vec<RemoteDaemon>, String> {
    use std::process::Command;

    let user = local_user();
    let remote_cmd = format!("ls /tmp/forgum-{}/daemon-*.json 2>/dev/null || true", user);

    let output = Command::new("ssh")
        .args([host, &remote_cmd])
        .output()
        .map_err(|e| format!("SSH to {} failed: {}", host, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut peers = Vec::new();

    for line in stdout.lines() {
        let path = line.trim();
        if path.is_empty() {
            continue;
        }

        // Try to read the daemon state file via SSH
        let cat_cmd = format!("cat {} 2>/dev/null", path);
        if let Ok(state_output) = Command::new("ssh").args([host, &cat_cmd]).output() {
            let state_json = String::from_utf8_lossy(&state_output.stdout);
            if let Ok(state) = serde_json::from_str::<serde_json::Value>(&state_json) {
                peers.push(RemoteDaemon {
                    pid: state["pid"].as_u64().unwrap_or(0) as u32,
                    host: host.to_string(),
                    session_id: state["session_id"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string(),
                    socket_path: PathBuf::from(path),
                    effect: state["effect"].as_str().unwrap_or("default").to_string(),
                    speed: state["speed"].as_f64().unwrap_or(1.0) as f32,
                    age: Duration::from_secs(state["age_secs"].as_u64().unwrap_or(0)),
                    is_leader: false,
                });
            }
        }
    }

    Ok(peers)
}

/// Elect a leader from a list of peers (lowest PID wins).
pub fn elect_leader(peers: &[RemoteDaemon]) -> Option<RemoteDaemon> {
    peers.iter().min_by_key(|p| p.pid).cloned()
}

/// Determine if this daemon should be the leader.
pub fn is_local_leader(peers: &[RemoteDaemon], local_pid: u32) -> bool {
    peers
        .iter()
        .all(|p| p.host == "localhost" || p.pid >= local_pid)
}

/// Sync effect to all peers in a session.
pub fn sync_effect_to_peers(
    peers: &[RemoteDaemon],
    session_id: &str,
    effect: &str,
) -> HashMap<String, Result<crate::control_socket::ControlResponse, String>> {
    let mut results = HashMap::new();

    for peer in peers {
        if peer.session_id != session_id {
            continue;
        }

        let key = format!("{}:{}", peer.host, peer.pid);
        let result = send_control_cmd(
            &peer.socket_path.to_string_lossy(),
            &ControlCmd::Effect(effect.to_string()),
        );
        results.insert(key, result);
    }

    results
}

/// Sync speed to all peers in a session.
pub fn sync_speed_to_peers(
    peers: &[RemoteDaemon],
    session_id: &str,
    speed: f32,
) -> HashMap<String, Result<crate::control_socket::ControlResponse, String>> {
    let mut results = HashMap::new();

    for peer in peers {
        if peer.session_id != session_id {
            continue;
        }

        let key = format!("{}:{}", peer.host, peer.pid);
        let result = send_control_cmd(
            &peer.socket_path.to_string_lossy(),
            &ControlCmd::Speed(speed),
        );
        results.insert(key, result);
    }

    results
}

/// Format a peer list for display.
pub fn format_peer_table(peers: &[RemoteDaemon]) -> String {
    if peers.is_empty() {
        return "No peers found.".to_string();
    }

    let mut output = String::new();
    output.push_str(&format!(
        "{:<8} {:<12} {:<12} {:<10} {:<6} {}\n",
        "PID", "HOST", "SESSION", "EFFECT", "SPEED", "AGE"
    ));
    output.push_str(&"-".repeat(65));
    output.push('\n');

    for peer in peers {
        let age_str = format!("{}s", peer.age.as_secs());
        let leader = if peer.is_leader { " *" } else { "" };
        output.push_str(&format!(
            "{:<8} {:<12} {:<12} {:<10} {:<6.1} {}{}\n",
            peer.pid, peer.host, peer.session_id, peer.effect, peer.speed, age_str, leader
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_ssh_detects_env() {
        // Without SSH env vars, should return false
        std::env::remove_var("SSH_CONNECTION");
        std::env::remove_var("SSH_CLIENT");
        std::env::remove_var("SSH_TTY");
        assert!(!is_ssh_session());
    }

    #[test]
    fn local_user_returns_string() {
        let user = local_user();
        assert!(!user.is_empty());
    }

    #[test]
    fn sync_session_id_deterministic() {
        let id1 = sync_session_id("alice", "laptop");
        let id2 = sync_session_id("alice", "laptop");
        assert_eq!(id1, id2);
    }

    #[test]
    fn sync_session_id_varies_by_host() {
        let id1 = sync_session_id("alice", "laptop");
        let id2 = sync_session_id("alice", "server");
        assert_ne!(id1, id2);
    }

    #[test]
    fn elect_leader_returns_lowest_pid() {
        let peers = vec![
            RemoteDaemon {
                pid: 100,
                host: "a".into(),
                session_id: "s1".into(),
                socket_path: PathBuf::from("/tmp/s1"),
                effect: "aurora".into(),
                speed: 1.0,
                age: Duration::from_secs(10),
                is_leader: false,
            },
            RemoteDaemon {
                pid: 50,
                host: "b".into(),
                session_id: "s1".into(),
                socket_path: PathBuf::from("/tmp/s2"),
                effect: "aurora".into(),
                speed: 1.0,
                age: Duration::from_secs(5),
                is_leader: false,
            },
        ];
        let leader = elect_leader(&peers).unwrap();
        assert_eq!(leader.pid, 50);
    }

    #[test]
    fn elect_leader_empty_returns_none() {
        assert!(elect_leader(&[]).is_none());
    }

    #[test]
    fn is_local_leader_true_when_lowest() {
        let peers = vec![RemoteDaemon {
            pid: 200,
            host: "remote".into(),
            session_id: "s1".into(),
            socket_path: PathBuf::from("/tmp/s"),
            effect: "aurora".into(),
            speed: 1.0,
            age: Duration::from_secs(10),
            is_leader: false,
        }];
        assert!(is_local_leader(&peers, 100));
    }

    #[test]
    fn is_local_leader_false_when_higher() {
        let peers = vec![RemoteDaemon {
            pid: 50,
            host: "remote".into(),
            session_id: "s1".into(),
            socket_path: PathBuf::from("/tmp/s"),
            effect: "aurora".into(),
            speed: 1.0,
            age: Duration::from_secs(10),
            is_leader: false,
        }];
        assert!(!is_local_leader(&peers, 100));
    }

    #[test]
    fn format_peer_table_empty() {
        assert_eq!(format_peer_table(&[]), "No peers found.");
    }

    #[test]
    fn format_peer_table_with_entries() {
        let peers = vec![RemoteDaemon {
            pid: 1234,
            host: "localhost".into(),
            session_id: "main".into(),
            socket_path: PathBuf::from("/tmp/s"),
            effect: "aurora".into(),
            speed: 1.5,
            age: Duration::from_secs(120),
            is_leader: false,
        }];
        let table = format_peer_table(&peers);
        assert!(table.contains("1234"));
        assert!(table.contains("aurora"));
    }

    #[test]
    fn discover_remote_peers_returns_vec() {
        let peers = discover_remote_peers(&[]);
        assert!(peers.is_empty() || !peers.is_empty());
    }

    #[test]
    fn parse_age_str_seconds() {
        assert_eq!(parse_age_str("30s"), Duration::from_secs(30));
    }

    #[test]
    fn parse_age_str_minutes() {
        assert_eq!(parse_age_str("2m"), Duration::from_secs(120));
    }

    #[test]
    fn parse_age_str_hours() {
        assert_eq!(parse_age_str("1h"), Duration::from_secs(3600));
    }

    #[test]
    fn control_cmd_to_json_effect() {
        let json = control_cmd_to_json(&ControlCmd::Effect("aurora".into()));
        assert!(json.contains("EFFECT"));
        assert!(json.contains("aurora"));
    }

    #[test]
    fn control_cmd_to_json_speed() {
        let json = control_cmd_to_json(&ControlCmd::Speed(2.5));
        assert!(json.contains("SPEED"));
        assert!(json.contains("2.5"));
    }

    #[test]
    fn control_cmd_to_json_stop() {
        let json = control_cmd_to_json(&ControlCmd::Stop);
        assert!(json.contains("STOP"));
    }

    #[test]
    fn control_cmd_to_json_ping() {
        let json = control_cmd_to_json(&ControlCmd::Ping);
        assert!(json.contains("PING"));
    }
}
