//! Test integrity of all 106 mascots and embedded cow assets.
//!
//! Validates that:
//! 1. Every single mascot listed across the 6 curated categories in `forgum_tui::app::CATEGORIES` (106 mascots)
//!    is embedded at compile-time and resolves independently with distinct art (never falling back to default cow).
//! 2. Standalone environments with zero files on disk can load and resolve all 106 mascots.
//! 3. All 132 embedded mascots in `forgum-platform` expand cleanly and contain valid ASCII art.

use std::collections::HashSet;
use tempfile::tempdir;

#[test]
fn test_all_106_mascots_in_categories_are_embedded_and_distinct() {
    let empty_dir = tempdir().expect("create temporary directory");
    let mut all_category_mascots = Vec::new();
    let mut unique_mascots = HashSet::new();

    for (_cat_name, mascot_list) in forgum_tui::app::CATEGORIES {
        for mascot in *mascot_list {
            all_category_mascots.push(*mascot);
            unique_mascots.insert(*mascot);
        }
    }

    assert_eq!(
        unique_mascots.len(),
        106,
        "CATEGORIES must contain exactly 106 unique mascots"
    );

    // Get the baseline default cow art for comparison
    let (default_art, _) = forgum_engine::cow::load_cow_with_landmarks(
        "default",
        empty_dir.path(),
        "oo",
        "..",
        "test",
    );
    assert!(
        !default_art.is_empty(),
        "Default cow art must not be empty"
    );

    let mut failed_mascots = Vec::new();
    let mut fallback_detected = Vec::new();

    for mascot in &unique_mascots {
        // 1. Check embedded availability in forgum-platform
        let embedded = forgum_platform::embedded_cows::get_embedded_cow(mascot);
        if embedded.is_none() {
            failed_mascots.push(format!("{mascot}: not found in embedded_cows"));
            continue;
        }

        // 2. Check engine cow loading with zero disk dependency
        let (art, _landmarks) = forgum_engine::cow::load_cow_with_landmarks(
            mascot,
            empty_dir.path(),
            "oo",
            "..",
            "test",
        );

        if art.trim().is_empty() {
            failed_mascots.push(format!("{mascot}: rendered empty art"));
        }

        // 3. For any mascot other than 'default', art MUST be distinct from default cow!
        // This directly guards against the bug where installed systems showed default cow for all mascots.
        if *mascot != "default" && art == default_art {
            fallback_detected.push(*mascot);
        }
    }

    assert!(
        failed_mascots.is_empty(),
        "Failed mascot integrity checks: {:?}",
        failed_mascots
    );

    assert!(
        fallback_detected.is_empty(),
        "Mascots incorrectly falling back to default cow: {:?}",
        fallback_detected
    );
}

#[test]
fn test_all_132_embedded_cows_are_valid_and_non_empty() {
    let embedded_names = forgum_platform::embedded_cows::all_embedded_cow_names();
    assert!(
        embedded_names.len() >= 132,
        "Expected at least 132 embedded cows, got {}",
        embedded_names.len()
    );

    for name in embedded_names {
        let content = forgum_platform::embedded_cows::get_embedded_cow(name)
            .unwrap_or_else(|| panic!("embedded cow '{name}' should exist"));
        assert!(
            !content.trim().is_empty(),
            "Embedded cow '{name}' content cannot be empty"
        );

        // Verify template expands properly
        let expanded = forgum_tui::app::ConfigApp::expand_cow_template(content, "oo", "..");
        assert!(
            !expanded.trim().is_empty(),
            "Expanded template for '{name}' cannot be empty"
        );
        assert!(
            expanded.lines().count() >= 2,
            "Mascot '{name}' must have at least 2 lines of ASCII art"
        );
    }
}

#[test]
fn test_cow_name_resolution_without_disk_dependency() {
    let empty_dir = tempdir().expect("create temporary directory");

    for (_cat_name, mascot_list) in forgum_tui::app::CATEGORIES {
        for mascot in *mascot_list {
            let resolved = forgum_engine::cow::resolve_cow_name(mascot, empty_dir.path());
            assert_eq!(
                resolved, *mascot,
                "Mascot '{mascot}' must resolve to itself without disk dependency"
            );
        }
    }
}
