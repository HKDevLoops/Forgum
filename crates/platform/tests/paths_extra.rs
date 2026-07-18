use forgum_platform::{config_path, control_socket_path, daemon_state_path, detect_session_id, is_canonical, ConfigPaths, ShellKind};
use std::path::Path;

#[test]
fn shell_kind_as_str_values() {
    assert_eq!(ShellKind::Bash.as_str(), "bash");
    assert_eq!(ShellKind::Zsh.as_str(), "zsh");
    assert_eq!(ShellKind::Fish.as_str(), "fish");
    assert_eq!(ShellKind::Pwsh.as_str(), "pwsh");
    assert_eq!(ShellKind::Unknown.as_str(), "unknown");
}

#[test]
fn daemon_state_path_includes_session() {
    let p = daemon_state_path("abc123");
    let name = p.file_name().and_then(|n| n.to_str()).unwrap();
    assert_eq!(name, "daemon-abc123.json");
}

#[test]
fn control_socket_path_includes_session() {
    let p = control_socket_path("sess9");
    let name = p.file_name().and_then(|n| n.to_str()).unwrap();
    assert!(name.starts_with("ctrl-sess9."));
}

#[test]
fn config_path_uses_env_override() {
    std::env::set_var("FORGUM_CONFIG", "/tmp/over/config.json");
    let p = config_path().unwrap();
    assert_eq!(p, Path::new("/tmp/over/config.json"));
    std::env::remove_var("FORGUM_CONFIG");
}

#[test]
fn resolve_returns_four_paths() {
    let tmp = tempfile::tempdir().unwrap();
    std::env::set_var("FORGUM_CONFIG", tmp.path().join("config.json"));
    std::env::set_var("FORGUM_DATA", tmp.path().join("data"));
    std::env::set_var("FORGUM_RUNTIME", tmp.path().join("runtime"));
    std::env::set_var("FORGUM_LOG", tmp.path().join("log"));
    let paths = ConfigPaths::resolve().unwrap();
    assert!(paths.config.ends_with("config.json"));
    assert!(paths.data.exists());
    assert!(paths.runtime.exists());
    assert!(paths.log.exists());
    std::env::remove_var("FORGUM_CONFIG");
    std::env::remove_var("FORGUM_DATA");
    std::env::remove_var("FORGUM_RUNTIME");
    std::env::remove_var("FORGUM_LOG");
}

#[test]
fn detect_session_id_returns_nonempty() {
    let id = detect_session_id();
    assert!(!id.is_empty());
}

#[test]
fn is_canonical_true_for_existing_real_file() {
    let tmp = tempfile::tempdir().unwrap();
    let f = tmp.path().join("real.txt");
    std::fs::write(&f, b"x").unwrap();
    assert!(is_canonical(&f));
}

#[test]
fn is_canonical_false_for_nonexistent() {
    let tmp = tempfile::tempdir().unwrap();
    let f = tmp.path().join("nope.txt");
    assert!(!is_canonical(&f));
}
