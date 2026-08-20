use forgum_engine::checkhealth::{self, HealthStatus};

#[test]
fn checkhealth_report_structure_and_counts() {
    let report = checkhealth::run_health_check(None);

    assert_eq!(
        report.sections.len(),
        7,
        "Report must contain 7 diagnostic sections"
    );
    assert!(report.total_count >= 7, "Total checks must be at least 7");
    assert_eq!(
        report.ok_count + report.warn_count + report.error_count,
        report.total_count,
        "Sum of ok, warn, and error counts must equal total_count"
    );

    let section_names: Vec<_> = report.sections.iter().map(|s| s.name.as_str()).collect();
    assert!(section_names.contains(&"System & Platform Environment"));
    assert!(section_names.contains(&"Configuration & Multi-Format Subsystem"));
    assert!(section_names.contains(&"Terminal Capabilities & Color Protocols"));
    assert!(section_names.contains(&"Pasture Assets & 2D Compound Signatures"));
    assert!(section_names.contains(&"Shell Integration & Prompt Hooks"));
    assert!(section_names.contains(&"Daemon & IPC Subsystem"));
    assert!(section_names.contains(&"Structured Logging & Traceability"));
}

#[test]
fn checkhealth_ansi_formatting_contains_key_elements() {
    let report = checkhealth::run_health_check(None);
    let ansi = report.format_ansi();

    assert!(ansi.contains("forgum checkhealth"));
    assert!(ansi.contains("Health Summary:"));
    assert!(ansi.contains("System & Platform Environment"));
    assert!(ansi.contains("Configuration & Multi-Format Subsystem"));
    assert!(ansi.contains("Pasture Assets & 2D Compound Signatures"));
}

#[test]
fn checkhealth_json_serialization_roundtrip() {
    let report = checkhealth::run_health_check(None);
    let json_str = serde_json::to_string_pretty(&report).expect("serialize report");
    assert!(!json_str.is_empty());

    let deserialized: checkhealth::HealthReport =
        serde_json::from_str(&json_str).expect("deserialize report");
    assert_eq!(deserialized.version, report.version);
    assert_eq!(deserialized.total_count, report.total_count);
    assert_eq!(deserialized.sections.len(), report.sections.len());
}

#[test]
fn checkhealth_status_badges_are_styled() {
    assert!(HealthStatus::Ok.badge().contains("[OK]"));
    assert!(HealthStatus::Warn.badge().contains("[WARNING]"));
    assert!(HealthStatus::Error.badge().contains("[ERROR]"));
    assert!(HealthStatus::Info.badge().contains("[INFO]"));
}
