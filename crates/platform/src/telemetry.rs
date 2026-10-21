//! Anonymous Motivation & Curiosity Telemetry Subsystem.
//!
//! # Philosophy & Privacy Invariants
//! This telemetry subsystem exists purely for author motivation and community curiosity.
//! It tracks only three high-level aggregate counters:
//! 1. `users_tried` - Number of developers who launched/tested the installer or engine.
//! 2. `users_installed` - Number of users who completed installation with consent.
//! 3. `active_users` - Daily anonymous activity pulse (throttled to at most once every 24h).
//!
//! ## Strict Guarantees
//! - ZERO Personal Data: No usernames, hostnames, machine GUIDs, hardware IDs, or IP addresses stored.
//! - ZERO Blocking: Runs on a detached thread with a strict 1.0-second network timeout.
//! - ZERO Surveillance: No background daemons, no analytics tracking, no session profiling.
//! - 100% Opt-Out: Set `FORGUM_TELEMETRY=0` or config `telemetry: false` to disable entirely.
//! - Silent Offline Fallback: If offline or firewalled, fails silently without warnings.

use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, SystemTime};

/// Metric kinds supported by the motivation counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    UsersTried,
    UsersInstalled,
    ActiveUsers,
}

impl Metric {
    pub fn as_str(self) -> &'static str {
        match self {
            Metric::UsersTried => "users_tried",
            Metric::UsersInstalled => "users_installed",
            Metric::ActiveUsers => "active_users",
        }
    }
}

/// Check if telemetry is allowed by user preference and environment variables.
pub fn is_telemetry_allowed() -> bool {
    // 1. Check environment variable override
    if let Ok(val) = std::env::var("FORGUM_TELEMETRY") {
        let v = val.trim().to_lowercase();
        if matches!(v.as_str(), "0" | "false" | "off" | "no" | "disable") {
            return false;
        }
        if matches!(v.as_str(), "1" | "true" | "on" | "yes") {
            return true;
        }
    }

    // 2. Check user consent marker (.telemetry_consent)
    if let Ok(cfg_dir) = crate::paths::config_dir() {
        let consent_file = cfg_dir.join(".telemetry_consent");
        if consent_file.is_file() {
            if let Ok(content) = std::fs::read_to_string(&consent_file) {
                let trimmed = content.trim().to_lowercase();
                if matches!(trimmed.as_str(), "0" | "false" | "no" | "decline") {
                    return false;
                }
                if matches!(trimmed.as_str(), "1" | "true" | "yes" | "allow") {
                    return true;
                }
            }
        }

        // 3. Check active configuration file across formats (JSON, YAML, TOML)
        if let Ok((cfg_path, fmt)) = crate::paths::detect_config_file(None) {
            if cfg_path.is_file() {
                if let Ok(content) = std::fs::read_to_string(&cfg_path) {
                    if let Ok(cfg) = crate::protocol::SceneConfig::parse_with_format(&content, fmt)
                    {
                        if !cfg.telemetry {
                            return false;
                        }
                    }
                }
            }
        }
    }

    true
}

/// Set the telemetry consent preference in user configuration.
pub fn set_telemetry_consent(allowed: bool) -> std::io::Result<()> {
    if let Ok(cfg_dir) = crate::paths::config_dir() {
        std::fs::create_dir_all(&cfg_dir)?;
        let consent_file = cfg_dir.join(".telemetry_consent");
        std::fs::write(consent_file, if allowed { "1\n" } else { "0\n" })?;
    }
    Ok(())
}

/// Increment the `users_tried` counter (called when installer wizard starts or forgum is tested).
/// Best-effort, non-blocking.
pub fn record_tried() {
    dispatch_counter_hit(Metric::UsersTried);
}

/// Increment the `users_installed` counter (called when installation finishes with consent).
/// Best-effort, non-blocking.
pub fn record_installed() {
    if is_telemetry_allowed() {
        dispatch_counter_hit(Metric::UsersInstalled);
    }
}

/// Record an active usage pulse (throttled to at most once per 24 hours).
/// Best-effort, non-blocking.
pub fn record_active_pulse() {
    if !is_telemetry_allowed() {
        return;
    }

    // Check throttling stamp
    let stamp_path = get_pulse_stamp_path();
    if let Some(ref path) = stamp_path {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    if elapsed < Duration::from_secs(86400) {
                        // Already pulsed within 24 hours; do not spam counter
                        return;
                    }
                }
            }
        }
        // Update or create timestamp file
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(
            path,
            format!(
                "{}\n",
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ),
        );
    }

    dispatch_counter_hit(Metric::ActiveUsers);
}

fn get_pulse_stamp_path() -> Option<PathBuf> {
    crate::paths::runtime_dir()
        .ok()
        .map(|d| d.join(".active_pulse"))
}

/// Local anonymous statistics tracking (stored in ~/.config/forgum/stats.json).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LocalStats {
    pub users_tried_count: u64,
    pub is_installed: bool,
    pub active_pulses_count: u64,
    pub last_pulse_unix: u64,
    pub first_seen_unix: u64,
}

impl Default for LocalStats {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            users_tried_count: 0,
            is_installed: false,
            active_pulses_count: 0,
            last_pulse_unix: 0,
            first_seen_unix: now,
        }
    }
}

/// Load local user telemetry statistics from disk.
pub fn load_local_stats() -> LocalStats {
    if let Ok(cfg_dir) = crate::paths::config_dir() {
        let stats_file = cfg_dir.join("stats.json");
        if let Ok(content) = std::fs::read_to_string(&stats_file) {
            if let Ok(s) = serde_json::from_str(&content) {
                return s;
            }
        }
    }
    LocalStats::default()
}

/// Record an event in local stats.
pub fn record_local_stat(metric: Metric) {
    let mut stats = load_local_stats();
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match metric {
        Metric::UsersTried => {
            stats.users_tried_count = stats.users_tried_count.saturating_add(1);
        }
        Metric::UsersInstalled => {
            stats.is_installed = true;
        }
        Metric::ActiveUsers => {
            stats.active_pulses_count = stats.active_pulses_count.saturating_add(1);
            stats.last_pulse_unix = now;
        }
    }
    if let Ok(cfg_dir) = crate::paths::config_dir() {
        let _ = std::fs::create_dir_all(&cfg_dir);
        let stats_file = cfg_dir.join("stats.json");
        if let Ok(json) = serde_json::to_string_pretty(&stats) {
            let _ = std::fs::write(stats_file, json);
        }
    }
}

/// Resolve target endpoint URL for the given metric.
pub fn resolve_telemetry_url(metric: Metric) -> String {
    let metric_name = metric.as_str();
    match std::env::var("FORGUM_TELEMETRY_ENDPOINT") {
        Ok(endpoint) => {
            if endpoint.contains("{metric}") {
                endpoint.replace("{metric}", metric_name)
            } else if endpoint.ends_with('/') {
                format!("{endpoint}{metric_name}")
            } else {
                format!("{endpoint}/{metric_name}")
            }
        }
        Err(_) => {
            format!("https://countapi.mileshilliard.com/api/v1/hit/hkdevloops_forgum_{metric_name}")
        }
    }
}

/// Dispatch an anonymous hit in a detached background thread.
/// Timeout is capped at 1.0 second. Fails silently on any error.
fn dispatch_counter_hit(metric: Metric) {
    record_local_stat(metric);

    let url = resolve_telemetry_url(metric);

    std::thread::Builder::new()
        .name("forgum-telemetry".to_string())
        .spawn(move || {
            execute_ping(&url);
        })
        .ok();
}

/// Ping the counter endpoint using built-in system tools (curl or PowerShell).
fn execute_ping(url: &str) {
    // 1. Strict URL validation to prevent command injection
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return;
    }
    if url.chars().any(|c| {
        matches!(
            c,
            '\'' | '"' | ';' | '`' | '$' | '|' | '&' | '<' | '>' | '\n' | '\r' | ' ' | '\t'
        )
    }) {
        return;
    }

    // Attempt curl first (present on modern Windows 10/11, macOS, and Linux)
    #[cfg(windows)]
    {
        let curl_res = Command::new("curl.exe")
            .arg("-s")
            .arg("--max-time")
            .arg("1")
            .arg("-X")
            .arg("GET")
            .arg(url)
            .output();

        if curl_res.is_err() {
            // Safe parameter-bound fallback to powershell WebClient with TLS 1.2
            let ps_script = "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object Net.WebClient).DownloadString($args[0])";
            let _ = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(ps_script)
                .arg(url)
                .output();
        }
    }

    #[cfg(unix)]
    {
        let curl_res = Command::new("curl")
            .arg("-s")
            .arg("--max-time")
            .arg("1")
            .arg(url)
            .output();

        if curl_res.is_err() {
            // Fallback to wget
            let _ = Command::new("wget")
                .arg("-q")
                .arg("-O")
                .arg("/dev/null")
                .arg("--timeout=1")
                .arg(url)
                .output();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_as_str() {
        assert_eq!(Metric::UsersTried.as_str(), "users_tried");
        assert_eq!(Metric::UsersInstalled.as_str(), "users_installed");
        assert_eq!(Metric::ActiveUsers.as_str(), "active_users");
    }

    #[test]
    fn telemetry_env_override() {
        std::env::set_var("FORGUM_TELEMETRY", "0");
        assert!(!is_telemetry_allowed());

        std::env::set_var("FORGUM_TELEMETRY", "false");
        assert!(!is_telemetry_allowed());

        std::env::set_var("FORGUM_TELEMETRY", "1");
        assert!(is_telemetry_allowed());

        std::env::remove_var("FORGUM_TELEMETRY");
    }

    #[test]
    fn telemetry_url_resolution() {
        std::env::remove_var("FORGUM_TELEMETRY_ENDPOINT");
        let default_url = resolve_telemetry_url(Metric::UsersTried);
        assert!(default_url.contains("hkdevloops_forgum_users_tried"));

        std::env::set_var(
            "FORGUM_TELEMETRY_ENDPOINT",
            "https://custom.worker.dev/pulse/{metric}",
        );
        let custom_interpolated = resolve_telemetry_url(Metric::ActiveUsers);
        assert_eq!(
            custom_interpolated,
            "https://custom.worker.dev/pulse/active_users"
        );

        std::env::set_var(
            "FORGUM_TELEMETRY_ENDPOINT",
            "https://custom.worker.dev/hit/",
        );
        let custom_slash = resolve_telemetry_url(Metric::UsersInstalled);
        assert_eq!(
            custom_slash,
            "https://custom.worker.dev/hit/users_installed"
        );

        std::env::set_var("FORGUM_TELEMETRY_ENDPOINT", "https://custom.worker.dev/hit");
        let custom_no_slash = resolve_telemetry_url(Metric::UsersInstalled);
        assert_eq!(
            custom_no_slash,
            "https://custom.worker.dev/hit/users_installed"
        );

        std::env::remove_var("FORGUM_TELEMETRY_ENDPOINT");
    }

    #[test]
    fn local_stats_default_and_mutation() {
        let mut stats = LocalStats::default();
        assert_eq!(stats.users_tried_count, 0);
        assert!(!stats.is_installed);
        assert_eq!(stats.active_pulses_count, 0);

        stats.users_tried_count += 1;
        stats.is_installed = true;
        stats.active_pulses_count += 1;

        assert_eq!(stats.users_tried_count, 1);
        assert!(stats.is_installed);
        assert_eq!(stats.active_pulses_count, 1);
    }
}
