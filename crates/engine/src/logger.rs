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

/// A structured log entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// Global logger lock to prevent write interleaving across threads.
static LOG_LOCK: Mutex<()> = Mutex::new(());

/// Log a message at the given level and target.
pub fn log(level: LogLevel, target: &str, message: &str) {
    let now = Local::now();
    let timestamp_human = now.format("%Y-%m-%d %H:%M:%S%.3f").to_string();
    let timestamp_iso = now.to_rfc3339();

    let entry = LogEntry {
        timestamp: timestamp_iso,
        level: level.as_str().to_string(),
        target: target.to_string(),
        message: message.to_string(),
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

    // 1. Text log: [2026-08-20 11:00:00.123] [INFO] [target] Message
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
        let _ = writeln!(
            file,
            "[{}] [{:<5}] [{}] {}",
            timestamp_human,
            level.as_str(),
            target,
            message
        );
    }

    // 2. JSONL log: {"timestamp":"...","level":"...","target":"...","message":"..."}
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
        $crate::logger::log($crate::logger::LogLevel::Info, $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Warn, $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Error, $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::log($crate::logger::LogLevel::Debug, $target, &format!($($arg)*))
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
    if text_path.is_file() {
        let _ = fs::write(&text_path, b"");
    }
    let jsonl_path = log_dir.join("forgum.jsonl");
    if jsonl_path.is_file() {
        let _ = fs::write(&jsonl_path, b"");
    }
    Ok(())
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
        "\x1b[90mTotal logs: {} | Logs directory: {}\x1b[0m\n",
        entries.len(),
        forgum_platform::log_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    ));
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
            },
            LogEntry {
                timestamp: "2026-08-20T11:00:01.456Z".into(),
                level: "ERROR".into(),
                target: "engine::config".into(),
                message: "Multiple conflicting configs found".into(),
            },
        ];
        let table = format_log_table(&entries);
        assert!(table.contains("engine::core"));
        assert!(table.contains("Multiple conflicting"));
    }
}
