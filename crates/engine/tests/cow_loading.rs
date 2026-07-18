//! Verify ALL 106 cow files in data/Cows/ load and expand without panicking.
//! Also verify animations.json loads correctly and DNA profiles are valid.
//!
//! This is the safety net that catches:
//! - Malformed .cow files that crash expand_cow
//! - Missing heredoc markers that cause incorrect parsing
//! - Placeholder substitution failures
//! - DNA profiles that don't match actual cow names

use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("workspace root")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn data_dir() -> PathBuf {
    workspace_root().join("data")
}

fn cows_dir() -> PathBuf {
    data_dir().join("Cows")
}

/// Get all cow base names (without .cow extension).
fn all_cow_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(cows_dir())
        .expect("data/Cows/ directory must exist")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("cow"))
        .filter_map(|e| {
            e.path()
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect();
    names.sort();
    names
}

// ── cow.rs integration tests ────────────────────────────────────────────────

#[test]
fn all_106_cow_files_load_and_expand() {
    let dd = data_dir();
    let names = all_cow_names();
    assert!(
        names.len() >= 100,
        "Expected at least 100 cow files, found {}",
        names.len()
    );

    let mut failures = Vec::new();
    for name in &names {
        let result =
            std::panic::catch_unwind(|| forgum_engine::cow::load_cow(name, &dd, "oo", " ", "\\\\"));
        match result {
            Ok(cow_text) => {
                // Every expanded cow must be non-empty.
                if cow_text.trim().is_empty() {
                    failures.push(format!("{name}.cow: expanded to empty string"));
                }
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                failures.push(format!("{name}.cow: PANIC: {msg}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Failed to load/expand {}/{} cow files:\n{}",
        failures.len(),
        names.len(),
        failures.join("\n")
    );
}

#[test]
fn all_cows_expand_with_various_eye_styles() {
    let dd = data_dir();
    let names = all_cow_names();

    // Test with different eye/tongue styles that users commonly set.
    let eye_styles = ["oo", "@@", "XX", "° °", "· ·"];
    let tongue_styles = [" ", "U", "V", "t"];

    for name in &names {
        for eyes in &eye_styles {
            for tongue in &tongue_styles {
                let cow = forgum_engine::cow::load_cow(name, &dd, eyes, tongue, "\\\\");
                assert!(
                    !cow.trim().is_empty(),
                    "{name}.cow produced empty output with eyes={eyes} tongue={tongue}"
                );
            }
        }
    }
}

#[test]
fn all_cows_have_nonzero_height() {
    let dd = data_dir();
    let names = all_cow_names();

    let mut too_short = Vec::new();
    for name in &names {
        let cow = forgum_engine::cow::load_cow(name, &dd, "oo", " ", "\\\\");
        let line_count = cow.lines().count();
        if line_count < 2 {
            too_short.push(format!("{name}.cow: only {line_count} line(s)"));
        }
    }

    assert!(
        too_short.is_empty(),
        "Cow files with suspiciously few lines:\n{}",
        too_short.join("\n")
    );
}

#[test]
fn all_cows_render_into_framebuffer_without_overflow() {
    use forgum_engine::cow::{compose_scene, render_cow};
    use forgum_engine::framebuffer::FrameBuffer;

    let dd = data_dir();
    let names = all_cow_names();

    for name in &names {
        let cow = forgum_engine::cow::load_cow(name, &dd, "oo", " ", "\\\\");
        let scene = compose_scene(&cow, "test bubble");

        // Render into a small framebuffer — must not panic or overflow.
        let mut fb = FrameBuffer::new(60, 20);
        render_cow(&mut fb, &scene);
        fb.swap();

        // Verify at least one non-space character was written.
        let mut has_content = false;
        for y in 0..fb.height {
            for x in 0..fb.width {
                if fb.get(x, y).ch != ' ' {
                    has_content = true;
                    break;
                }
            }
            if has_content {
                break;
            }
        }
        assert!(
            has_content,
            "{name}.cow: rendered but framebuffer is all spaces"
        );
    }
}

// ── dna.rs integration tests ────────────────────────────────────────────────

#[test]
fn animations_json_loads_all_10_profiles() {
    let dd = data_dir();
    let map = forgum_engine::dna::load_animations(&dd);

    assert_eq!(
        map.len(),
        10,
        "Expected 10 DNA profiles in animations.json, found {}",
        map.len()
    );

    let expected = [
        "dragon", "default", "nyan", "dolphin", "ghost", "koala", "skeleton", "doge", "tux", "cat",
    ];
    for name in &expected {
        assert!(
            map.contains_key(*name),
            "animations.json missing profile for '{name}'"
        );
    }
}

#[test]
fn all_dna_profiles_are_valid() {
    let dd = data_dir();
    let map = forgum_engine::dna::load_animations(&dd);

    for (name, dna) in &map {
        assert!(
            dna.speed > 0.0 && dna.speed <= 10.0,
            "DNA '{name}' has invalid speed: {}",
            dna.speed
        );
        assert!(
            dna.amplitude.breath >= 0.0,
            "DNA '{name}' has negative breath amplitude"
        );
        assert!(
            dna.amplitude.sway >= 0.0,
            "DNA '{name}' has negative sway amplitude"
        );
        assert!(
            dna.amplitude.float >= 0.0,
            "DNA '{name}' has negative float amplitude"
        );
        assert!(
            !dna.easing.base.is_empty(),
            "DNA '{name}' has empty base easing"
        );
    }
}

#[test]
fn dna_profiles_match_cow_file_names() {
    let dd = data_dir();
    let map = forgum_engine::dna::load_animations(&dd);
    let cow_names = all_cow_names();

    // Every DNA profile key must correspond to an actual .cow file.
    let mut orphans = Vec::new();
    for key in map.keys() {
        if !cow_names.contains(key) {
            orphans.push(key.clone());
        }
    }

    assert!(
        orphans.is_empty(),
        "DNA profiles reference cow names that don't exist as .cow files: {:?}",
        orphans
    );
}

#[test]
fn get_dna_returns_correct_profile_for_known_cows() {
    let dd = data_dir();
    let map = forgum_engine::dna::load_animations(&dd);

    // dragon should have Fire particles
    let dragon = forgum_engine::dna::get_dna(&map, "dragon");
    assert_eq!(
        dragon.particles.r#type,
        forgum_engine::dna::ParticleType::Fire
    );
    assert!(dragon.particles.rate > 0, "dragon should emit particles");

    // nyan should have Stars particles
    let nyan = forgum_engine::dna::get_dna(&map, "nyan");
    assert_eq!(
        nyan.particles.r#type,
        forgum_engine::dna::ParticleType::Stars
    );

    // dolphin should have Bubbles particles
    let dolphin = forgum_engine::dna::get_dna(&map, "dolphin");
    assert_eq!(
        dolphin.particles.r#type,
        forgum_engine::dna::ParticleType::Bubbles
    );

    // ghost should have Glitch particles
    let ghost = forgum_engine::dna::get_dna(&map, "ghost");
    assert_eq!(
        ghost.particles.r#type,
        forgum_engine::dna::ParticleType::Glitch
    );
}

#[test]
fn dna_fallback_for_unknown_cow() {
    let map = HashMap::new();
    let dna = forgum_engine::dna::get_dna(&map, "unknown-cow.cow");
    assert_eq!(dna, forgum_engine::dna::CowDna::default());
}

// ── cow file format edge cases ──────────────────────────────────────────────

#[test]
fn cow_files_with_no_heredoc_parse_correctly() {
    // Some .cow files may not have the <<EOC; marker. Verify expand_cow
    // handles this gracefully (treats entire file as body).
    let template = "  no heredoc here\n  just raw lines\n";
    let expanded = forgum_engine::cow::expand_cow(template, "oo", " ", "\\\\");
    assert!(expanded.contains("no heredoc here"));
    assert!(expanded.contains("oo") || expanded.contains("just raw lines"));
}

#[test]
fn cow_files_with_backslashes_do_not_panic() {
    // Cow art is full of backslashes. Verify expand_cow doesn't choke.
    let template = r#"$the_cow = <<EOC;
     $eyes
    /\
   /  \
  /    \
 /______\
EOC;"#;
    let expanded = forgum_engine::cow::expand_cow(template, "@@", "U", "\\\\");
    assert!(expanded.contains("@@"));
    assert!(expanded.contains("/\\"));
}

#[test]
fn cow_files_with_unicode_eyes() {
    let template = r#"$the_cow = <<EOC;
   $eyes
  (oo)
   $thoughts
EOC;"#;
    let expanded = forgum_engine::cow::expand_cow(template, "° °", "U", "~");
    assert!(expanded.contains("° °"));
    assert!(expanded.contains("~"));
}

#[test]
fn expand_cow_preserves_multiline_structure() {
    let template = "line1\nline2\nline3";
    let expanded = forgum_engine::cow::expand_cow(template, "x", "y", "z");
    let lines: Vec<&str> = expanded.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");
}
