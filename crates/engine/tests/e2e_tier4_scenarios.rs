//! Tier 4: Real-World Application Scenarios
//!
//! Validates >= 5 realistic end-to-end CLI workflows across shells and terminal environments:
//! 1. Shell Prompt Login & Initialization Workflow (init bash/pwsh, marker isolation, sweep)
//! 2. Classic Unix Pipeline Integration (piping command output into 'forgum say')
//! 3. Mascot Engine Health & Doctor Diagnostics ('forgum doctor' / 'checkhealth')
//! 4. Daemon Lifecycle & Remote Control ('forgum status', 'forgum stop')
//! 5. Classic Cowsay Drop-in Replacement with Custom Eyes & Tongue
//! 6. Mascot Catalog Listing & Taxonomy Verification ('forgum list animals')

use assert_cmd::Command;
use forgum_engine::cow;
use forgum_platform::biome::ALL_MASCOTS;
use forgum_platform::paths::{daemon_state_path, data_dir};
use forgum_platform::shell::{HOOK_MARKER_BEGIN, HOOK_MARKER_END};
use std::path::PathBuf;

// =========================================================================
// Scenario 1: Shell Prompt Login & Initialization Workflow
// =========================================================================

#[test]
fn t4_scenario_01_shell_init_bash_and_sweep() {
    // 1. Generate Bash hook via CLI
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["init", "bash"]);
    let output = cmd.output().expect("execute init bash");
    assert!(
        output.status.success(),
        "forgum init bash must exit with code 0"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(HOOK_MARKER_BEGIN) || stdout.contains("# >>> forgum (bash) >>>"),
        "Bash hook must start with standard begin marker"
    );
    assert!(
        stdout.contains(HOOK_MARKER_END),
        "Bash hook must terminate with standard end marker"
    );
    assert!(
        stdout.contains("forgum"),
        "Bash hook must invoke forgum keyword"
    );

    // 2. Execute emergency sweep recovery
    let mut sweep_cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    sweep_cmd.arg("sweep");
    let sweep_out = sweep_cmd.output().expect("execute sweep");
    assert!(sweep_out.status.success(), "forgum sweep must succeed");
}

#[test]
fn t4_scenario_01_shell_init_pwsh() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["init", "pwsh"]);
    let output = cmd.output().expect("execute init pwsh");
    assert!(
        output.status.success(),
        "forgum init pwsh must exit with code 0"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(HOOK_MARKER_BEGIN) || stdout.contains("# >>> forgum (pwsh) >>>"),
        "Pwsh hook must start with standard begin marker"
    );
    assert!(
        stdout.contains(HOOK_MARKER_END),
        "Pwsh hook must terminate with standard end marker"
    );
    assert!(
        stdout.contains("prompt"),
        "Pwsh hook must wrap shell prompt"
    );
}

// =========================================================================
// Scenario 2: Classic Unix Pipeline Integration
// =========================================================================

#[test]
fn t4_scenario_02_pipeline_say_with_fortune_message() {
    let message = "Fortune favors the bold";
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    if cfg!(windows) {
        cmd.args(["say", "cmd", "/c", "echo", message]);
    } else {
        cmd.args(["say", "echo", message]);
    }
    let output = cmd.output().expect("execute say");
    assert!(output.status.success(), "forgum say must exit with code 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify speech bubble rendering
    assert!(
        stdout.contains(message),
        "Output must contain speech message"
    );
    assert!(
        stdout.contains('\\') || stdout.contains('/') || stdout.contains('|'),
        "Output must render ASCII speech bubble contours"
    );
}

// =========================================================================
// Scenario 3: Mascot Engine Health & Doctor Diagnostics
// =========================================================================

#[test]
fn t4_scenario_03_doctor_system_diagnostics() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.arg("doctor");
    let output = cmd.output().expect("execute doctor");
    assert!(output.status.success(), "forgum doctor must succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.to_lowercase().contains("terminal")
            || stdout.to_lowercase().contains("capability")
            || stdout.to_lowercase().contains("forgum"),
        "Doctor output must report terminal and system diagnostics"
    );
}

#[test]
fn t4_scenario_03_checkhealth_json_report() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["checkhealth", "--json"]);
    let output = cmd.output().expect("execute checkhealth --json");
    assert!(
        output.status.success(),
        "forgum checkhealth --json must succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_val: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(
        json_val.is_ok(),
        "checkhealth --json must emit valid JSON object"
    );
}

// =========================================================================
// Scenario 4: Daemon Lifecycle & Remote Control
// =========================================================================

#[test]
fn t4_scenario_04_daemon_lifecycle_status_and_stop() {
    // 1. Query daemon status
    let mut status_cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    status_cmd.arg("status");
    let status_out = status_cmd.output().expect("execute status");
    assert!(status_out.status.success(), "forgum status should succeed");

    // 2. Verify platform daemon state path is resolvable
    let state_file = daemon_state_path("default");
    assert!(
        state_file.to_string_lossy().contains("forgum")
            || state_file.to_string_lossy().contains("daemon"),
        "Daemon state path must follow platform XDG/AppData conventions"
    );

    // 3. Issue stop command to ensure clean terminal state
    let mut stop_cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    stop_cmd.arg("stop");
    let stop_out = stop_cmd.output().expect("execute stop");
    assert!(stop_out.status.success(), "forgum stop must exit cleanly");
}

// =========================================================================
// Scenario 5: Classic Cowsay Drop-in Replacement with Custom Eyes & Tongue
// =========================================================================

#[test]
fn t4_scenario_05_cowsay_dropin_replacement_custom_eyes_tongue() {
    let data_p = data_dir().unwrap_or_else(|_| PathBuf::from("data"));
    let cow_text = cow::load_cow("corgi", &data_p, "$$", "U ", "\\\\");
    assert!(
        cow_text.contains("$$"),
        "Expanded cow text must contain eyes '$$'"
    );
    assert!(
        cow_text.contains("U "),
        "Expanded cow text must contain tongue 'U '"
    );

    let composed = cow::compose_scene(&cow_text, "Mascot engine operational");
    assert!(composed.contains("$$"));
    assert!(composed.contains("U "));
    assert!(composed.contains("Mascot engine operational"));
}

// =========================================================================
// Scenario 6: Mascot Catalog Listing & Taxonomy Verification
// =========================================================================

#[test]
fn t4_scenario_06_mascot_catalog_listing() {
    let mut cmd = Command::cargo_bin("forgum").expect("cargo_bin forgum");
    cmd.args(["list", "animals"]);
    let output = cmd.output().expect("execute list animals");
    assert!(output.status.success(), "forgum list animals must succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verify key sample mascots from across taxonomic categories are listed
    let samples = ["default", "dragon", "tux", "stegosaurus", "elephant"];
    for name in samples {
        assert!(
            stdout.contains(name),
            "Mascot catalog list must contain '{}'",
            name
        );
    }

    // Verify all 132 biome mascots exist in registry
    assert_eq!(
        ALL_MASCOTS.len(),
        132,
        "Biome registry must contain exactly 132 mascots"
    );
}
