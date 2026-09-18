//! Structured logging subsystem for Forgum.
//!
//! Features:
//! - Multi-sink logging: human-readable `forgum.log` + structured `forgum.jsonl`.
//! - Standard severity levels: `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`.
//! - High-performance append with ISO 8601 timestamps and thread targets.
//! - Beautiful ANSI table formatter for `forgum logs` and TUI viewer.
//! - Zero platform `#[cfg]` (all log directory discovery via `forgum_platform::log_dir()`).

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

use chrono::Local;
use serde::{Deserialize, Serialize};

/// Log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }

    #[must_use]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "TRACE" => Some(Self::Trace),
            "DEBUG" => Some(Self::Debug),
            "INFO" => Some(Self::Info),
            "WARN" | "WARNING" => Some(Self::Warn),
            "ERROR" | "ERR" => Some(Self::Error),
            _ => None,
        }
    }
}

/// A structured log entry with optional user and developer diagnostic hints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub developer_hint: Option<String>,
}

impl Default for LogEntry {
    fn default() -> Self {
        Self {
            timestamp: String::new(),
            level: "INFO".to_string(),
            target: String::new(),
            message: String::new(),
            user_hint: None,
            developer_hint: None,
        }
    }
}

impl LogEntry {
    #[must_use]
    pub fn new(timestamp: String, level: String, target: String, message: String) -> Self {
        Self {
            timestamp,
            level,
            target,
            message,
            user_hint: None,
            developer_hint: None,
        }
    }
}

use std::sync::atomic::{AtomicU8, Ordering};

/// Global minimum log level threshold. Defaults to LogLevel::Info (2).
static LOG_THRESHOLD: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

/// Global logger lock to prevent write interleaving across threads.
static LOG_LOCK: Mutex<()> = Mutex::new(());

/// Set minimum logging severity threshold.
pub fn set_min_log_level(level: LogLevel) {
    LOG_THRESHOLD.store(level as u8, Ordering::Release);
}

/// Raw minimum log level for zero-overhead macro inspection.
#[inline(always)]
#[must_use]
pub fn min_log_level_raw() -> u8 {
    LOG_THRESHOLD.load(Ordering::Relaxed)
}

/// Get current logging severity threshold.
#[must_use]
pub fn get_min_log_level() -> LogLevel {
    match LOG_THRESHOLD.load(Ordering::Acquire) {
        0 => LogLevel::Trace,
        1 => LogLevel::Debug,
        2 => LogLevel::Info,
        3 => LogLevel::Warn,
        _ => LogLevel::Error,
    }
}

/// Log a message at the given level and target.
pub fn log(level: LogLevel, target: &str, message: &str) {
    log_diagnostic(level, target, message, None, None);
}

/// Log a structured message with explicit user remediation hint and developer context.
pub fn log_diagnostic(
    level: LogLevel,
    target: &str,
    message: &str,
    user_hint: Option<&str>,
    developer_hint: Option<&str>,
) {
    // O(1) atomic threshold check: return immediately with zero heap allocation
    if (level as u8) < LOG_THRESHOLD.load(Ordering::Relaxed) {
        return;
    }

    let now = Local::now();
    let timestamp_human = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let timestamp_iso = now.to_rfc3339();

    let entry = LogEntry {
        timestamp: timestamp_iso,
        level: level.as_str().to_string(),
        target: target.to_string(),
        message: message.to_string(),
        user_hint: user_hint.map(|s| s.to_string()),
        developer_hint: developer_hint.map(|s| s.to_string()),
    };

    let Ok(log_dir) = forgum_platform::log_dir() else {
        return;
    };

    let Ok(_guard) = LOG_LOCK.lock() else {
        return;
    };

    if !log_dir.is_dir() {
        let _ = fs::create_dir_all(&log_dir);
    }

    const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

    // 1. Text log: [2026-08-20 11:00:00.123] [INFO] [target] Message (Hint: ...)
    let text_path = log_dir.join("forgum.log");
    if let Ok(meta) = fs::metadata(&text_path) {
        if meta.len() > MAX_LOG_SIZE {
            let _ = fs::rename(&text_path, log_dir.join("forgum.log.1"));
        }
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&text_path)
    {
        let mut line = format!(
            "[{}] [{:<5}] [{}] {}",
            timestamp_human,
            level.as_str(),
            target,
            message
        );
        if let Some(ref uh) = entry.user_hint {
            line.push_str(&format!(" [USER: {uh}]"));
        }
        if let Some(ref dh) = entry.developer_hint {
            line.push_str(&format!(" [DEV: {dh}]"));
        }
        let _ = writeln!(file, "{line}");
    }

    // 2. JSONL log: {"timestamp":"...","level":"...","target":"...","message":"...", ...}
    let jsonl_path = log_dir.join("forgum.jsonl");
    if let Ok(meta) = fs::metadata(&jsonl_path) {
        if meta.len() > MAX_LOG_SIZE {
            let _ = fs::rename(&jsonl_path, log_dir.join("forgum.jsonl.1"));
        }
    }
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&jsonl_path)
    {
        if let Ok(json) = serde_json::to_string(&entry) {
            let _ = writeln!(file, "{json}");
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($target:expr, $($arg:tt)*) => {
        if ($crate::logger::LogLevel::Info as u8) >= $crate::logger::min_log_level_raw() {
            $crate::logger::log($crate::logger::LogLevel::Info, $target, &format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)*) => {
        if ($crate::logger::LogLevel::Warn as u8) >= $crate::logger::min_log_level_raw() {
            $crate::logger::log($crate::logger::LogLevel::Warn, $target, &format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)*) => {
        if ($crate::logger::LogLevel::Error as u8) >= $crate::logger::min_log_level_raw() {
            $crate::logger::log($crate::logger::LogLevel::Error, $target, &format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)*) => {
        if ($crate::logger::LogLevel::Debug as u8) >= $crate::logger::min_log_level_raw() {
            $crate::logger::log($crate::logger::LogLevel::Debug, $target, &format!($($arg)*))
        }
    };
}

#[macro_export]
macro_rules! log_diag {
    ($lvl:expr, $target:expr, $msg:expr, $user_hint:expr, $dev_hint:expr) => {
        if ($lvl as u8) >= $crate::logger::min_log_level_raw() {
            $crate::logger::log_diagnostic($lvl, $target, $msg, Some($user_hint), Some($dev_hint))
        }
    };
}

/// Read recent log entries from the JSONL log file.
pub fn read_recent_logs(
    max_lines: usize,
    min_level: Option<LogLevel>,
) -> Result<Vec<LogEntry>, std::io::Error> {
    let log_dir = match forgum_platform::log_dir() {
        Ok(d) => d,
        Err(_) => return Ok(Vec::new()),
    };
    let jsonl_path = log_dir.join("forgum.jsonl");
    if !jsonl_path.is_file() {
        return Ok(Vec::new());
    }

    let file = fs::File::open(jsonl_path)?;
    let reader = BufReader::new(file);
    let mut entries: Vec<LogEntry> = Vec::new();

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
            if let Some(min_lvl) = min_level {
                if let Some(lvl) = LogLevel::from_str_loose(&entry.level) {
                    if lvl < min_lvl {
                        continue;
                    }
                }
            }
            entries.push(entry);
        }
    }

    if entries.len() > max_lines && max_lines > 0 {
        let start = entries.len() - max_lines;
        entries = entries.split_off(start);
    }

    Ok(entries)
}

/// Get the active log file paths.
pub fn get_log_paths() -> Option<(std::path::PathBuf, std::path::PathBuf, std::path::PathBuf)> {
    let dir = forgum_platform::log_dir().ok()?;
    let text = dir.join("forgum.log");
    let jsonl = dir.join("forgum.jsonl");
    Some((dir, text, jsonl))
}

/// Clear / truncate existing log files.
pub fn clear_logs() -> Result<(), std::io::Error> {
    let log_dir = match forgum_platform::log_dir() {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };
    let Ok(_guard) = LOG_LOCK.lock() else {
        return Ok(());
    };

    let text_path = log_dir.join("forgum.log");
    let jsonl_path = log_dir.join("forgum.jsonl");
    if text_path.is_file() {
        fs::write(&text_path, "")?;
    }
    if jsonl_path.is_file() {
        fs::write(&jsonl_path, "")?;
    }
    Ok(())
}

/// Read the raw text log file in its entirety.
pub fn read_raw_log() -> Result<String, std::io::Error> {
    let log_dir = match forgum_platform::log_dir() {
        Ok(d) => d,
        Err(e) => return Err(std::io::Error::other(e.to_string())),
    };
    let text_path = log_dir.join("forgum.log");
    if !text_path.is_file() {
        return Ok(String::new());
    }
    fs::read_to_string(text_path)
}

/// Launch the user's default system editor or file explorer to inspect logs.
pub fn open_log_in_system(open_directory: bool) -> Result<std::path::PathBuf, std::io::Error> {
    let (dir, text, _) = get_log_paths().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Unable to resolve forgum log directory",
        )
    })?;

    // Ensure directory exists
    let _ = fs::create_dir_all(&dir);

    let target_path = if open_directory || !text.is_file() {
        dir
    } else {
        text
    };

    forgum_platform::open_in_system_viewer(&target_path)?;

    Ok(target_path)
}

/// Filter entries by a case-insensitive query across message, target, and diagnostic hints.
pub fn filter_entries(entries: &[LogEntry], query: &str) -> Vec<LogEntry> {
    if query.trim().is_empty() {
        return entries.to_vec();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| {
            e.message.to_lowercase().contains(&q)
                || e.target.to_lowercase().contains(&q)
                || e.level.to_lowercase().contains(&q)
                || e.user_hint
                    .as_ref()
                    .is_some_and(|h| h.to_lowercase().contains(&q))
                || e.developer_hint
                    .as_ref()
                    .is_some_and(|h| h.to_lowercase().contains(&q))
        })
        .cloned()
        .collect()
}

/// Format log entries into a high-visibility, dark-humored ANSI table.
pub fn format_log_table(entries: &[LogEntry]) -> String {
    if entries.is_empty() {
        return "\x1b[90m(No logs found in this pasture. The cows are suspiciously quiet.)\x1b[0m\n".to_string();
    }

    let mut out = String::new();
    out.push_str("\x1b[1;36m┌──────────────────────────┬───────┬──────────────────────┬────────────────────────────────────────────────────────┐\x1b[0m\n");
    out.push_str("\x1b[1;36m│\x1b[0m \x1b[1mTimestamp\x1b[0m                \x1b[1;36m│\x1b[0m \x1b[1mLevel\x1b[0m \x1b[1;36m│\x1b[0m \x1b[1mComponent\x1b[0m            \x1b[1;36m│\x1b[0m \x1b[1mEvent Message\x1b[0m                                          \x1b[1;36m│\x1b[0m\n");
    out.push_str("\x1b[1;36m├──────────────────────────┼───────┼──────────────────────┼────────────────────────────────────────────────────────┤\x1b[0m\n");

    for entry in entries {
        let level_color = match entry.level.as_str() {
            "ERROR" => "\x1b[1;31m", // bold red
            "WARN" => "\x1b[1;33m",  // bold yellow
            "INFO" => "\x1b[1;32m",  // bold green
            "DEBUG" => "\x1b[1;34m", // bold blue
            _ => "\x1b[90m",         // gray
        };

        // Format timestamp cleanly
        let ts = if entry.timestamp.len() > 23 {
            &entry.timestamp[..23]
        } else {
            &entry.timestamp
        };

        let target_display = if entry.target.len() > 20 {
            &entry.target[..20]
        } else {
            &entry.target
        };

        let msg_display = if entry.message.len() > 54 {
            format!("{}...", &entry.message[..51])
        } else {
            entry.message.clone()
        };

        out.push_str(&format!(
            "\x1b[1;36m│\x1b[0m {:<24} \x1b[1;36m│\x1b[0m {}{:<5}\x1b[0m \x1b[1;36m│\x1b[0m {:<20} \x1b[1;36m│\x1b[0m {:<54} \x1b[1;36m│\x1b[0m\n",
            ts,
            level_color,
            entry.level,
            target_display,
            msg_display
        ));
    }

    out.push_str("\x1b[1;36m└──────────────────────────┴───────┴──────────────────────┴────────────────────────────────────────────────────────┘\x1b[0m\n");
    out.push_str(&format!(
        "\x1b[90mTotal logs: {} | Log directory: {}\x1b[0m\n",
        entries.len(),
        forgum_platform::log_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    ));
    out.push_str("\x1b[90mAccess Commands: 'forgum logs -w' (stream) | 'forgum logs -o' (open in editor) | 'forgum logs -D' (triage bugs)\x1b[0m\n");
    out
}

/// Comprehensive diagnostic triage report generated from engine logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub total_logs: usize,
    pub errors_count: usize,
    pub warnings_count: usize,
    pub info_count: usize,
    pub debug_count: usize,
    pub active_issues: Vec<IssueSummary>,
    pub uprising_bugs: Vec<BugAnomaly>,
    pub user_action_items: Vec<String>,
    pub developer_action_items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    pub severity: String,
    pub component: String,
    pub description: String,
    pub occurrences: usize,
    pub user_remediation: String,
    pub developer_context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugAnomaly {
    pub pattern: String,
    pub subsystem: String,
    pub frequency: usize,
    pub threat_level: String,
    pub suggested_fix: String,
}

/// Analyze a sequence of log entries to detect active issues, uprising bugs,
/// user-facing remediations, and developer focus targets.
pub fn diagnose_logs(entries: &[LogEntry]) -> DiagnosticReport {
    let mut errors_count = 0;
    let mut warnings_count = 0;
    let mut info_count = 0;
    let mut debug_count = 0;

    let mut cow_missing_count = 0;
    let mut dna_err_count = 0;
    let mut terminal_err_count = 0;
    let mut config_err_count = 0;
    let mut render_lag_count = 0;

    let mut active_issues = Vec::new();
    let mut user_action_items = Vec::new();
    let mut developer_action_items = Vec::new();

    for entry in entries {
        match entry.level.as_str() {
            "ERROR" => errors_count += 1,
            "WARN" => warnings_count += 1,
            "INFO" => info_count += 1,
            _ => debug_count += 1,
        }

        let is_anomaly_candidate = entry.level == "ERROR" || entry.level == "WARN";
        let msg_lower = entry.message.to_ascii_lowercase();

        if is_anomaly_candidate {
            if msg_lower.contains("missing cow")
                || msg_lower.contains("cannot find cow")
                || msg_lower.contains("fallback cow")
            {
                cow_missing_count += 1;
            }
            if msg_lower.contains("deserializ")
                || msg_lower.contains("invalid dna")
                || msg_lower.contains("schema error")
                || msg_lower.contains("corrupt animation")
            {
                dna_err_count += 1;
            }
            if msg_lower.contains("margin clamp")
                || msg_lower.contains("decstbm clamp")
                || msg_lower.contains("margin overflow")
            {
                terminal_err_count += 1;
            }
            if msg_lower.contains("conflict")
                || msg_lower.contains("config format conflict")
                || msg_lower.contains("invalid configuration")
            {
                config_err_count += 1;
            }
            if msg_lower.contains("frame drop")
                || msg_lower.contains("deadline miss")
                || msg_lower.contains("render stall")
            {
                render_lag_count += 1;
            }
        }

        // Collect custom diagnostic hints directly from entries
        if let (Some(ref uh), Some(ref dh)) = (&entry.user_hint, &entry.developer_hint) {
            active_issues.push(IssueSummary {
                severity: entry.level.clone(),
                component: entry.target.clone(),
                description: entry.message.clone(),
                occurrences: 1,
                user_remediation: uh.clone(),
                developer_context: dh.clone(),
            });
            if !user_action_items.contains(uh) {
                user_action_items.push(uh.clone());
            }
            if !developer_action_items.contains(dh) {
                developer_action_items.push(dh.clone());
            }
        }
    }

    let mut uprising_bugs = Vec::new();

    if cow_missing_count > 0 {
        uprising_bugs.push(BugAnomaly {
            pattern: "Mascot Art Lookup Failure".to_string(),
            subsystem: "cow.rs / MascotRegistry".to_string(),
            frequency: cow_missing_count,
            threat_level: if cow_missing_count > 3 { "HIGH".into() } else { "MEDIUM".into() },
            suggested_fix: "Check data/Cows/ catalog and cow name resolution fallback in crates/engine/src/cow.rs".into(),
        });
        user_action_items.push("Some requested mascots are missing. Run `forgum list animals` to view all available mascots.".into());
        developer_action_items.push(
            "Audit `cow.rs::resolve_cow_name` and verify `.cow` files exist in `data/Cows/`."
                .into(),
        );
    }

    if dna_err_count > 0 {
        uprising_bugs.push(BugAnomaly {
            pattern: "Animation DNA Schema Discrepancy".to_string(),
            subsystem: "dna.rs / AnimationsCatalog".to_string(),
            frequency: dna_err_count,
            threat_level: "HIGH".into(),
            suggested_fix: "Validate `animations.json` syntax and verify `BaseAnim` enum mappings in `crates/engine/src/dna.rs`".into(),
        });
        user_action_items.push("One or more creature animations had syntax errors and defaulted. Run `forgum doctor` to check integrity.".into());
        developer_action_items.push("Check `dna.rs::load_animations` deserializer logs and ensure `data/Cows/animations.json` matches `CowDna` struct.".into());
    }

    if terminal_err_count > 0 {
        uprising_bugs.push(BugAnomaly {
            pattern: "DECSTBM Scroll Margin Protocol Clamping".to_string(),
            subsystem: "engine_core.rs / TerminalProtocol".to_string(),
            frequency: terminal_err_count,
            threat_level: "MEDIUM".into(),
            suggested_fix: "Ensure overlay_rows is clamped against total terminal rows - 3 in `engine_core.rs:1053`".into(),
        });
        user_action_items.push("Terminal window is too small for split-scroll banner mode. Expand your terminal height.".into());
        developer_action_items.push(
            "Review DECSTBM scroll margin emission in `crates/engine/src/engine_core.rs`.".into(),
        );
    }

    if config_err_count > 0 {
        uprising_bugs.push(BugAnomaly {
            pattern: "Config Format Conflict or Mutual Exclusivity Violation".to_string(),
            subsystem: "paths.rs / Configuration".to_string(),
            frequency: config_err_count,
            threat_level: "MEDIUM".into(),
            suggested_fix:
                "Verify `detect_config_file` in `paths.rs` and cleanup logic in `tui/src/lib.rs`"
                    .into(),
        });
        user_action_items.push("Multiple config file formats exist in ~/.config/forgum/. Run `forgum config --tui` to consolidate into one format.".into());
        developer_action_items.push(
            "Ensure safe mutual exclusivity in `crates/platform/src/paths.rs::detect_config_file`."
                .into(),
        );
    }

    if render_lag_count > 0 {
        uprising_bugs.push(BugAnomaly {
            pattern: "SIM Thread Render Deadline Miss".to_string(),
            subsystem: "engine_core.rs / Scheduler".to_string(),
            frequency: render_lag_count,
            threat_level: "LOW".into(),
            suggested_fix: "Profile frame time in `SimState::tick` and consider lowering target FPS for high latency environments".into(),
        });
        user_action_items.push(
            "Terminal output is lagging. Run `forgum config set fps 30` to reduce GPU/CPU load."
                .into(),
        );
        developer_action_items.push(
            "Inspect SIM state framebuffer rendering pass in `crates/engine/src/engine_core.rs`."
                .into(),
        );
    }

    DiagnosticReport {
        total_logs: entries.len(),
        errors_count,
        warnings_count,
        info_count,
        debug_count,
        active_issues,
        uprising_bugs,
        user_action_items,
        developer_action_items,
    }
}

/// Format the diagnostic report into an executive-grade, dual User/Developer ANSI dashboard.
pub fn format_diagnostic_report(report: &DiagnosticReport) -> String {
    let mut out = String::new();

    let health_badge = if report.errors_count > 0 {
        "\x1b[1;41;37m CRITICAL ISSUES DETECTED \x1b[0m"
    } else if report.warnings_count > 0 || !report.uprising_bugs.is_empty() {
        "\x1b[1;43;30m WARNINGS / ANOMALIES DETECTED \x1b[0m"
    } else {
        "\x1b[1;42;30m HEALTHY & OPTIMAL \x1b[0m"
    };

    out.push_str(&format!(
        "\x1b[1;36m┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓\x1b[0m\n\
         \x1b[1;36m┃\x1b[0m  \x1b[1mFORGUM INTELLIGENT LOG DIAGNOSTICS & BUG TRIAGE\x1b[0m               {}  \x1b[1;36m┃\x1b[0m\n\
         \x1b[1;36m┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫\x1b[0m\n",
        health_badge
    ));

    out.push_str(&format!(
        "\x1b[1;36m┃\x1b[0m Total Events: \x1b[1m{:<5}\x1b[0m │ Errors: \x1b[1;31m{:<3}\x1b[0m │ Warnings: \x1b[1;33m{:<3}\x1b[0m │ Info: \x1b[1;32m{:<5}\x1b[0m │ Debug: \x1b[1;34m{:<5}\x1b[0m \x1b[1;36m┃\x1b[0m\n\
         \x1b[1;36m┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫\x1b[0m\n",
        report.total_logs, report.errors_count, report.warnings_count, report.info_count, report.debug_count
    ));

    // Section 1: User Guidance
    out.push_str("\x1b[1;36m┃\x1b[0m \x1b[1;32m▶ FOR THE USER: What's Going Wrong & How to Fix It\x1b[0m                                     \x1b[1;36m┃\x1b[0m\n");
    if report.user_action_items.is_empty() {
        out.push_str("\x1b[1;36m┃\x1b[0m  \x1b[90m✓ Everything is running smoothly. No corrective actions required.\x1b[0m                    \x1b[1;36m┃\x1b[0m\n");
    } else {
        for (i, action) in report.user_action_items.iter().enumerate() {
            let num = format!("{}. ", i + 1);
            let display = if action.len() > 80 {
                &action[..80]
            } else {
                action.as_str()
            };
            out.push_str(&format!(
                "\x1b[1;36m┃\x1b[0m  \x1b[1m{}{:<81}\x1b[0m\x1b[1;36m┃\x1b[0m\n",
                num, display
            ));
        }
    }

    out.push_str("\x1b[1;36m┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫\x1b[0m\n");

    // Section 2: Developer Radar (Uprising Bugs)
    out.push_str("\x1b[1;36m┃\x1b[0m \x1b[1;35m▶ FOR DEVELOPERS: Bug Radar & Uprising Subsystem Anomaly Triage\x1b[0m                         \x1b[1;36m┃\x1b[0m\n");
    if report.uprising_bugs.is_empty() && report.developer_action_items.is_empty() {
        out.push_str("\x1b[1;36m┃\x1b[0m  \x1b[90m✓ Zero anomalies detected in animation, render, and platform subsystems.\x1b[0m            \x1b[1;36m┃\x1b[0m\n");
    } else {
        for bug in &report.uprising_bugs {
            let threat_color = match bug.threat_level.as_str() {
                "HIGH" => "\x1b[1;31m",
                "MEDIUM" => "\x1b[1;33m",
                _ => "\x1b[1;34m",
            };
            out.push_str(&format!(
                "\x1b[1;36m┃\x1b[0m  • \x1b[1m{}\x1b[0m (Subsystem: \x1b[36m{}\x1b[0m | Threat: {}{}\x1b[0m | Count: {})\x1b[1;36m\n",
                bug.pattern, bug.subsystem, threat_color, bug.threat_level, bug.frequency
            ));
            out.push_str(&format!(
                "\x1b[1;36m┃\x1b[0m    \x1b[90m↳ Code Fix: {}\x1b[0m\x1b[1;36m\n",
                bug.suggested_fix
            ));
        }
        for (i, dev_hint) in report.developer_action_items.iter().enumerate() {
            let num = format!("  [{}] ", i + 1);
            out.push_str(&format!(
                "\x1b[1;36m┃\x1b[0m  \x1b[33m{}{}\x1b[0m\n",
                num, dev_hint
            ));
        }
    }

    out.push_str("\x1b[1;36m┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛\x1b[0m\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_level_ordering() {
        assert!(LogLevel::Error > LogLevel::Warn);
        assert!(LogLevel::Warn > LogLevel::Info);
        assert!(LogLevel::Info > LogLevel::Debug);
        assert!(LogLevel::Debug > LogLevel::Trace);
    }

    #[test]
    fn format_log_table_renders_entries() {
        let entries = vec![
            LogEntry {
                timestamp: "2026-08-20T11:00:00.123Z".into(),
                level: "INFO".into(),
                target: "engine::core".into(),
                message: "Daemon initialized in background overlay".into(),
                user_hint: None,
                developer_hint: None,
            },
            LogEntry {
                timestamp: "2026-08-20T11:00:01.456Z".into(),
                level: "ERROR".into(),
                target: "engine::config".into(),
                message: "Multiple conflicting configs found".into(),
                user_hint: None,
                developer_hint: None,
            },
        ];
        let table = format_log_table(&entries);
        assert!(table.contains("engine::core"));
        assert!(table.contains("Multiple conflicting"));
    }

    #[test]
    fn diagnose_logs_detects_uprising_bugs_and_remediations() {
        let entries = vec![
            LogEntry {
                timestamp: "2026-08-20T11:00:00.123Z".into(),
                level: "WARN".into(),
                target: "engine::dna".into(),
                message: "Animations.json deserialization failed for corrupted entry".into(),
                user_hint: None,
                developer_hint: None,
            },
            LogEntry {
                timestamp: "2026-08-20T11:00:01.456Z".into(),
                level: "ERROR".into(),
                target: "engine::cow".into(),
                message: "Missing cow template file: pegasus.cow not found".into(),
                user_hint: Some("Verify animal name with `forgum list animals`".into()),
                developer_hint: Some("Check `cow.rs::load_cow`".into()),
            },
        ];

        let diag = diagnose_logs(&entries);
        assert_eq!(diag.total_logs, 2);
        assert_eq!(diag.errors_count, 1);
        assert_eq!(diag.warnings_count, 1);
        assert!(!diag.uprising_bugs.is_empty());
        assert!(!diag.user_action_items.is_empty());
        assert!(!diag.developer_action_items.is_empty());

        let report = format_diagnostic_report(&diag);
        assert!(report.contains("FORGUM INTELLIGENT LOG DIAGNOSTICS"));
        assert!(report.contains("FOR THE USER"));
        assert!(report.contains("FOR DEVELOPERS"));
        assert!(report.contains("Mascot Art Lookup Failure"));
    }
}
