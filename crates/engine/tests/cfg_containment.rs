//! G8 — zero platform-targeting `#[cfg]` in the engine source tree.
//! All `cfg(unix)`/`cfg(windows)`/`cfg(target_os = ...)` branching is
//! delegated to `forgum-platform`.
//!
//! We walk the engine `src/` directory and assert no file contains
//! `#[cfg(<platform-target>)]`. Test-only gating (`#[cfg(test)]`) is allowed
//! because it doesn't affect the released binary.

use std::fs;
use std::path::Path;

#[test]
fn engine_source_has_zero_platform_cfg_attributes() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert!(src_dir.is_dir(), "src/ missing at {}", src_dir.display());

    let mut hits = Vec::new();
    walk(&src_dir, &mut hits);

    assert!(
        hits.is_empty(),
        "BUG-arch regression: engine/src/ must contain zero platform-targeting \
         `#[cfg(...)]` attributes (all platform branching lives in forgum-platform). \
         Offenders:\n  {}",
        hits.join("\n  ")
    );
}

/// Platform-targeting cfg attributes that belong only in forgum-platform.
const PLATFORM_CFG_PATTERNS: &[&str] = &[
    "#[cfg(unix)]",
    "cfg!(unix)",
    "#[cfg(windows)]",
    "cfg!(windows)",
    "#[cfg(target_os",
    "cfg!(target_os",
    "#[cfg(target_family",
    "cfg!(target_family",
    "#[cfg(target_arch",
    "cfg!(target_arch",
    "#[cfg(target_endian",
    "cfg!(target_endian",
    "#[cfg(target_pointer_width",
    "cfg!(target_pointer_width",
    "#[cfg(target_env",
    "cfg!(target_env",
    "#[cfg(target_vendor",
    "cfg!(target_vendor",
];

fn is_platform_cfg(line: &str) -> bool {
    PLATFORM_CFG_PATTERNS.iter().any(|p| line.contains(p))
}

/// Check if a line is part of a test function (has #[test] attribute nearby).
/// We allow platform cfg on test functions since they only affect test compilation.
fn is_test_function_context(text: &str, line_idx: usize) -> bool {
    // Look backwards up to 10 lines for #[test]
    let start = line_idx.saturating_sub(10);
    for i in start..=line_idx {
        if i < text.lines().count() {
            let line = text.lines().nth(i).unwrap_or("");
            if line.trim().starts_with("#[test]") {
                return true;
            }
        }
    }
    false
}

fn walk(dir: &Path, hits: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, hits);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(text) = fs::read_to_string(&path) {
                for (i, line) in text.lines().enumerate() {
                    if is_platform_cfg(line) && !is_test_function_context(&text, i) {
                        let rel = path
                            .strip_prefix(env!("CARGO_MANIFEST_DIR"))
                            .unwrap_or(&path);
                        hits.push(format!("{}:{}: {}", rel.display(), i + 1, line.trim()));
                    }
                }
            }
        }
    }
}
