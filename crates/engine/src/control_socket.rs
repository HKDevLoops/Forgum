//! Control socket for daemon management.
//!
//! Implements a Unix domain socket (or named pipe on Windows) that accepts
//! newline-delimited JSON commands for controlling a running daemon.

use serde::{Deserialize, Serialize};

/// A command received over the control socket.
#[derive(Debug, Clone)]
pub enum ControlCmd {
    Stop,
    Pause,
    Resume,
    Effect(String),
    Speed(f32),
    Cow(String),
    Text(String),
    Status,
    Ping,
    Unknown(String),
}

/// Response to send back over the socket.
#[derive(Debug, Serialize)]
pub struct ControlResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<StatusInfo>,
}

#[derive(Debug, Serialize)]
pub struct StatusInfo {
    pub running: bool,
    pub paused: bool,
    pub effect: String,
    pub fps: u16,
    pub speed: f32,
}

/// Request JSON from the socket.
#[derive(Debug, Deserialize)]
struct ControlRequest {
    cmd: String,
    #[serde(default)]
    arg: Option<String>,
}

/// Parse a raw line from the control socket into a `ControlCmd`.
pub fn parse_cmd(line: &str) -> ControlCmd {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return ControlCmd::Unknown("empty".into());
    }
    match serde_json::from_str::<ControlRequest>(trimmed) {
        Ok(req) => match req.cmd.to_uppercase().as_str() {
            "STOP" => ControlCmd::Stop,
            "PAUSE" => ControlCmd::Pause,
            "RESUME" => ControlCmd::Resume,
            "EFFECT" => ControlCmd::Effect(req.arg.unwrap_or_default()),
            "SPEED" => {
                let v = req.arg.and_then(|s| s.parse::<f32>().ok()).unwrap_or(1.0);
                ControlCmd::Speed(v)
            }
            "COW" => ControlCmd::Cow(req.arg.unwrap_or_default()),
            "TEXT" => ControlCmd::Text(req.arg.unwrap_or_default()),
            "STATUS" => ControlCmd::Status,
            "PING" => ControlCmd::Ping,
            other => ControlCmd::Unknown(other.to_string()),
        },
        Err(_) => ControlCmd::Unknown(trimmed.to_string()),
    }
}

/// Encode a response as a newline-terminated JSON string.
pub fn encode_response(resp: &ControlResponse) -> String {
    let mut json = serde_json::to_string(resp).unwrap_or_else(|_| r#"{"ok":false}"#.into());
    json.push('\n');
    json
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stop() {
        assert!(matches!(parse_cmd(r#"{"cmd":"STOP"}"#), ControlCmd::Stop));
    }

    #[test]
    fn parse_effect_with_arg() {
        let cmd = parse_cmd(r#"{"cmd":"EFFECT","arg":"aurora"}"#);
        assert!(matches!(cmd, ControlCmd::Effect(ref s) if s == "aurora"));
    }

    #[test]
    fn parse_speed() {
        let cmd = parse_cmd(r#"{"cmd":"SPEED","arg":"2.5"}"#);
        assert!(matches!(cmd, ControlCmd::Speed(2.5)));
    }

    #[test]
    fn parse_ping() {
        assert!(matches!(parse_cmd(r#"{"cmd":"PING"}"#), ControlCmd::Ping));
    }

    #[test]
    fn parse_empty_is_unknown() {
        assert!(matches!(parse_cmd(""), ControlCmd::Unknown(_)));
    }

    #[test]
    fn encode_ok_response() {
        let resp = ControlResponse {
            ok: true,
            error: None,
            status: None,
        };
        let s = encode_response(&resp);
        assert!(s.contains(r#""ok":true"#));
        assert!(s.ends_with('\n'));
    }
}
