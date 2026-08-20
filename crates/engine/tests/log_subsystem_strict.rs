use forgum_engine::logger::{self, LogEntry, LogLevel};

#[test]
fn log_level_ordering_and_parsing() {
    assert!(LogLevel::Error > LogLevel::Warn);
    assert!(LogLevel::Warn > LogLevel::Info);
    assert!(LogLevel::Info > LogLevel::Debug);
    assert!(LogLevel::Debug > LogLevel::Trace);

    assert_eq!(LogLevel::from_str_loose("trace"), Some(LogLevel::Trace));
    assert_eq!(LogLevel::from_str_loose("DEBUG"), Some(LogLevel::Debug));
    assert_eq!(LogLevel::from_str_loose("info"), Some(LogLevel::Info));
    assert_eq!(LogLevel::from_str_loose("warn"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::from_str_loose("warning"), Some(LogLevel::Warn));
    assert_eq!(LogLevel::from_str_loose("error"), Some(LogLevel::Error));
    assert_eq!(LogLevel::from_str_loose("err"), Some(LogLevel::Error));
    assert_eq!(LogLevel::from_str_loose("invalid"), None);
}

#[test]
fn log_write_and_read_roundtrip() {
    let token = format!("audit-token-{}", std::process::id());
    logger::log(LogLevel::Info, "test::audit", &format!("Msg 1 {}", token));
    logger::log(LogLevel::Warn, "test::audit", &format!("Msg 2 {}", token));
    logger::log(LogLevel::Error, "test::audit", &format!("Msg 3 {}", token));

    let entries = logger::read_recent_logs(500, None).expect("read logs");
    let matching: Vec<_> = entries
        .iter()
        .filter(|e| e.message.contains(&token))
        .collect();
    assert_eq!(matching.len(), 3);
    assert_eq!(matching[0].level, "INFO");
    assert_eq!(matching[1].level, "WARN");
    assert_eq!(matching[2].level, "ERROR");
}

#[test]
fn log_level_filtering() {
    let token = format!("filter-token-{}", std::process::id());
    logger::log(LogLevel::Debug, "test::filter", &format!("Debug {}", token));
    logger::log(LogLevel::Info, "test::filter", &format!("Info {}", token));
    logger::log(LogLevel::Warn, "test::filter", &format!("Warn {}", token));
    logger::log(LogLevel::Error, "test::filter", &format!("Error {}", token));

    let warn_and_above = logger::read_recent_logs(500, Some(LogLevel::Warn)).expect("read logs");
    let matching: Vec<_> = warn_and_above
        .iter()
        .filter(|e| e.message.contains(&token))
        .collect();
    assert_eq!(matching.len(), 2);
    assert_eq!(matching[0].level, "WARN");
    assert_eq!(matching[1].level, "ERROR");
}

#[test]
fn log_table_formatting_strict_alignment() {
    let entries = vec![
        LogEntry {
            timestamp: "2026-08-20T11:00:00.123+05:30".into(),
            level: "INFO".into(),
            target: "engine::core".into(),
            message: "System initialized with 0 anomalies".into(),
        },
        LogEntry {
            timestamp: "2026-08-20T11:00:01.456+05:30".into(),
            level: "ERROR".into(),
            target: "engine::config".into(),
            message: "Fatal format conflict detected between json and yaml".into(),
        },
    ];

    let table = logger::format_log_table(&entries);
    assert!(table.contains("Timestamp"));
    assert!(table.contains("Level"));
    assert!(table.contains("Component"));
    assert!(table.contains("Event Message"));
    assert!(table.contains("engine::core"));
    assert!(table.contains("System initialized with 0 anomalies"));
    assert!(table.contains("engine::config"));
    assert!(table.contains("Fatal format conflict"));
}

#[test]
fn log_multithreaded_stress_test() {
    let mut handles = Vec::new();
    for thread_idx in 0..8 {
        let handle = std::thread::spawn(move || {
            for i in 0..25 {
                logger::log(
                    LogLevel::Info,
                    &format!("thread::{thread_idx}"),
                    &format!("Thread iteration {i} completed without deadlock"),
                );
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    let entries = logger::read_recent_logs(250, None).expect("read logs");
    assert!(!entries.is_empty());
}
