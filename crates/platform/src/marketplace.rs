//! Mascot pack marketplace discovery, sandboxing, and overlay storage.
//!
//! Enforces:
//! - Stage 1 Sandbox Guard: Disallows path traversal (`..`, leading `/` or `\`, drive letters).
//! - Packs stored in an isolated overlay: `AppPaths::data_dir()/packs/<sanitized_id>/`.
//! - Resilient Deserialization (Rule D): Pack loading errors never corrupt the 132 canonical mascots.

use std::fs;
use std::path::PathBuf;

/// Validates that a pack ID contains exclusively safe alphanumeric or dash/underscore characters.
#[must_use]
pub fn is_safe_pack_identifier(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Sanitizes a user-supplied pack identifier into a safe filename.
#[must_use]
pub fn sanitize_pack_id(id: &str) -> String {
    let clean: String = id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if clean.is_empty() {
        "pack_custom".to_string()
    } else {
        clean
    }
}

/// Directory path where custom user mascot packs are isolated.
pub fn packs_dir() -> std::io::Result<PathBuf> {
    let base = crate::paths::data_dir()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::NotFound, e.to_string()))?;
    let dir = base.join("packs");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    Ok(dir)
}

/// Returns the sandboxed directory path for a specific mascot pack.
pub fn pack_directory(pack_id: &str) -> Result<PathBuf, String> {
    if !is_safe_pack_identifier(pack_id) {
        return Err(format!(
            "Unsafe or invalid pack identifier '{pack_id}' (SEC-PACK-001)"
        ));
    }
    let p_dir = packs_dir().map_err(|e| format!("cannot resolve packs dir: {e}"))?;
    let target = p_dir.join(pack_id);
    Ok(target)
}

/// List all valid installed mascot pack identifiers.
#[must_use]
pub fn list_installed_packs() -> Vec<String> {
    let Ok(dir) = packs_dir() else {
        return Vec::new();
    };
    let mut packs = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if is_safe_pack_identifier(name) {
                        packs.push(name.to_string());
                    }
                }
            }
        }
    }
    packs.sort();
    packs
}

/// Remove an installed mascot pack from the local data directory.
pub fn remove_installed_pack(pack_id: &str) -> Result<bool, String> {
    let target = pack_directory(pack_id)?;
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|e| format!("failed to remove pack directory {}: {e}", target.display()))?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_pack_identifier_rejects_traversal() {
        assert!(is_safe_pack_identifier("cyber-dragon"));
        assert!(is_safe_pack_identifier("cool_doge_v2"));
        assert!(!is_safe_pack_identifier("../etc/passwd"));
        assert!(!is_safe_pack_identifier("C:\\Windows"));
        assert!(!is_safe_pack_identifier("/root/pack"));
        assert!(!is_safe_pack_identifier("has space"));
        assert!(!is_safe_pack_identifier(""));
    }

    #[test]
    fn test_sanitize_pack_id() {
        assert_eq!(sanitize_pack_id("my-pack!"), "my-pack_");
        assert_eq!(sanitize_pack_id(""), "pack_custom");
    }
}
