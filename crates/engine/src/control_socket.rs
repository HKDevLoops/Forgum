//! Control socket for daemon management.
//!
//! Implements a Unix domain socket (or named pipe on Windows) that accepts
//! newline-delimited JSON commands for controlling a running daemon.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

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

/// A control socket server that accepts connections and dispatches commands.
#[derive(Debug)]
pub struct ControlServer {
    socket_path: PathBuf,
    _thread: thread::JoinHandle<()>,
}

impl ControlServer {
    /// Bind the control socket and start listening.
    ///
    /// Returns the receiver end of the command channel. The render loop
    /// reads commands from this receiver.
    pub fn start(
        socket_path: PathBuf,
    ) -> Result<(Self, mpsc::Receiver<ControlCmd>), Box<dyn std::error::Error>> {
        let (tx, rx) = mpsc::channel();
        let socket = forgum_platform::DaemonSocket::bind(&socket_path)?;

        let thread = thread::spawn(move || {
            Self::accept_loop(socket, tx);
        });

        Ok((
            Self {
                socket_path,
                _thread: thread,
            },
            rx,
        ))
    }

    fn accept_loop(
        socket: forgum_platform::DaemonSocket,
        tx: mpsc::Sender<ControlCmd>,
    ) {
        loop {
            match socket.accept() {
                Ok(Some(mut conn)) => {
                    // Read commands from this connection.
                    loop {
                        match conn.read_line() {
                            Ok(Some(line)) => {
                                let cmd = parse_cmd(&line);
                                let is_stop = matches!(cmd, ControlCmd::Stop);
                                let is_status = matches!(cmd, ControlCmd::Status);
                                let is_ping = matches!(cmd, ControlCmd::Ping);

                                if is_status {
                                    let resp = ControlResponse {
                                        ok: true,
                                        error: None,
                                        status: Some(StatusInfo {
                                            running: true,
                                            paused: false,
                                            effect: "unknown".into(),
                                            fps: 30,
                                            speed: 1.0,
                                        }),
                                    };
                                    let _ = conn.write_response(&encode_response(&resp));
                                    continue;
                                }

                                if is_ping {
                                    let resp = ControlResponse {
                                        ok: true,
                                        error: None,
                                        status: None,
                                    };
                                    let _ = conn.write_response(&encode_response(&resp));
                                    continue;
                                }

                                // Send command to render loop.
                                if tx.send(cmd).is_err() {
                                    return; // render loop dropped
                                }

                                // Send generic OK response.
                                let resp = ControlResponse {
                                    ok: true,
                                    error: None,
                                    status: None,
                                };
                                let _ = conn.write_response(&encode_response(&resp));

                                if is_stop {
                                    return; // stop accept loop
                                }
                            }
                            Ok(None) => break, // client disconnected
                            Err(_) => break,   // read error
                        }
                    }
                }
                Ok(None) => {
                    // No connection pending (non-blocking).
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(_) => {
                    // Accept error — keep trying.
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    /// Path to the socket file.
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        // Clean up the socket file.
        let _ = std::fs::remove_file(&self.socket_path);
    }
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
