//! Integration test: `DaemonState` round-trip, alive check, and status fields.

use std::path::Path;

use forgum_engine::daemon::DaemonState;
use tempfile::tempdir;

fn sample_state() -> DaemonState {
    DaemonState {
        pid: 4242,
        ob_y1: 12,
        cols: 100,
        socket_path: "/tmp/forgum-ctrl.sock".to_string(),
        started_at: "2026-07-18T12:00:00Z".to_string(),
    }
}

#[test]
fn daemon_state_write_then_read_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("daemon.json");

    let state = sample_state();
    state.write(&path).unwrap();
    assert!(path.exists(), "daemon state file must be written");

    let loaded = DaemonState::read(&path).unwrap();
    assert_eq!(loaded.pid, state.pid);
    assert_eq!(loaded.ob_y1, state.ob_y1);
    assert_eq!(loaded.cols, state.cols);
    assert_eq!(loaded.socket_path, state.socket_path);
    assert_eq!(loaded.started_at, state.started_at);
}

#[test]
fn daemon_state_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path: &Path = &dir.path().join("nested/deep/daemon.json");

    sample_state().write(path).unwrap();
    let loaded = DaemonState::read(path).unwrap();
    assert_eq!(loaded.pid, 4242);
}

#[test]
fn daemon_state_read_missing_file_errors() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does-not-exist.json");
    let result = DaemonState::read(&path);
    assert!(result.is_err(), "reading a missing file must error");
}

#[test]
fn daemon_state_is_alive_returns_bool() {
    // Any PID (alive or not) must yield a bool without panicking.
    let state = sample_state();
    let _alive: bool = state.is_alive();
}

#[test]
fn daemon_state_fields_reflect_input() {
    let state = DaemonState {
        pid: 7,
        ob_y1: 3,
        cols: 80,
        socket_path: "/run/forgum.sock".to_string(),
        started_at: "now".to_string(),
    };
    let dir = tempdir().unwrap();
    let path = dir.path().join("d.json");
    state.write(&path).unwrap();
    let loaded = DaemonState::read(&path).unwrap();
    assert_eq!(loaded.pid, 7);
    assert_eq!(loaded.cols, 80);
    assert!(loaded.socket_path.ends_with(".sock"));
}
