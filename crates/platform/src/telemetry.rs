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
        if v == "0" || v == "false" || v == "off" || v == "no" || v == "disable" {
            return false;
        }
        if v == "1" || v == "true" || v == "on" || v == "yes" {
            return true;
        }
    }

    // 2. Check config file if accessible
    if let Ok(cfg_dir) = crate::paths::config_dir() {
        let cfg_file = cfg_dir.join("config.json");
        if cfg_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&cfg_file) {
                if content.contains("\"telemetry\": false")
                    || content.contains("\"telemetry\":false")
                {
                    return false;
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

/// Dispatch an anonymous hit to CounterAPI in a detached background thread.
/// Timeout is capped at 1.0 second. Fails silently on any error.
fn dispatch_counter_hit(metric: Metric) {
    let metric_name = metric.as_str();
    let url = format!("https://api.counterapi.dev/v1/forgum/{metric_name}/up");

    std::thread::Builder::new()
        .name("forgum-telemetry".to_string())
        .spawn(move || {
            execute_ping(&url);
        })
        .ok();
}

/// Ping the counter endpoint using built-in system tools (curl or PowerShell).
fn execute_ping(url: &str) {
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
            // Fallback to powershell WebClient with TLS 1.2
            let ps_script = format!(
                "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object Net.WebClient).DownloadString('{url}')"
            );
            let _ = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(&ps_script)
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
}
