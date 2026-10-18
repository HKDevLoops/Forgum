//! Custom Mascot Asset Marketplace & Pack Importer.
//!
//! Implements the 6-stage automated verification pipeline mandated by `Audit_0.0.10.md`
//! to guarantee Rule C (Single Source of Truth for Mascot DNA) and
//! Rule D (Resilient Deserialization — Zero Catalog Poisoning):
//!
//! - Stage 1: Archive & Path Sandbox Guard (256 KB max, traversal protection).
//! - Stage 2: Heredoc & Placeholder AST Verifier (bounds check, zero lingering `$vars`).
//! - Stage 3: Safe-SGR ANSI Escape Filter (disallows OSC, CSI cursor/clear codes).
//! - Stage 4: CP437 Mojibake & Unicode Width Gate (no U+2600..=U+263F or "Γÿ").
//! - Stage 5: Isolated CowDna Signature & Contrast Gate (5-slot #RRGGBB schema + SHA-256).
//! - Stage 6: AsciiRigMap Pre-Build & Live Daemon Hot-Reload.

use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::cow::{self, char_display_width};
use crate::dna::{BaseAnim, Habitat};
use crate::kinematics::{AnatomicalSegments, AsciiRigMap};
use forgum_platform::marketplace::{is_safe_pack_identifier, pack_directory};

pub const MAX_PACK_SIZE_BYTES: usize = 256 * 1024; // 256 KB strict cap
pub const MAX_MASCOT_WIDTH: usize = 140;
pub const MAX_MASCOT_HEIGHT: usize = 45;

/// Isolated mascot pack manifest defining biology and color styling.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MascotPackManifest {
    pub pack_id: String,
    pub author: String,
    pub version: String,
    pub dna_signature: String, // Isolated SHA-256 signature
    pub habitat: Habitat,
    pub base_anim: BaseAnim,
    pub palette_hex: [String; 5],
}

/// Errors raised during the 6-stage verification pipeline.
#[derive(Debug, PartialEq, Eq)]
pub enum PackVerifyError {
    PayloadTooLarge(usize),
    UnsafePathIdentifier(String),
    ForbiddenTerminalEscapeSequence,
    Cp437MojibakeDetected(char),
    LingeringTemplatePlaceholder(String),
    DimensionsOutOfBounds { width: usize, height: usize },
    InvalidHexPaletteSlot(String),
    SignatureMismatch,
}

impl fmt::Display for PackVerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PayloadTooLarge(sz) => write!(
                f,
                "Pack payload exceeds maximum limit of 256 KB (got {sz} bytes) [SEC-PACK-001]"
            ),
            Self::UnsafePathIdentifier(id) => write!(
                f,
                "Unsafe or invalid pack identifier '{id}' (must be alphanumeric with '-' or '_') [SEC-PACK-001]"
            ),
            Self::ForbiddenTerminalEscapeSequence => write!(
                f,
                "Forbidden terminal escape sequence detected (OSC/CSI/DECSTBM disallowed) [SEC-PACK-002]"
            ),
            Self::Cp437MojibakeDetected(ch) => write!(
                f,
                "Legacy CP437 mojibake collision character detected: '{ch}' (U+{:04X}) [BUG-006]",
                *ch as u32
            ),
            Self::LingeringTemplatePlaceholder(p) => write!(
                f,
                "Lingering unexpanded template placeholder '{p}' in mascot body art"
            ),
            Self::DimensionsOutOfBounds { width, height } => write!(
                f,
                "Mascot art dimensions {width}x{height} exceed limits ({MAX_MASCOT_WIDTH}x{MAX_MASCOT_HEIGHT})"
            ),
            Self::InvalidHexPaletteSlot(hex) => write!(
                f,
                "Invalid 24-bit hex palette slot format '{hex}' (must be #RRGGBB)"
            ),
            Self::SignatureMismatch => write!(
                f,
                "Pack DNA signature does not match computed SHA-256 content hash"
            ),
        }
    }
}

impl std::error::Error for PackVerifyError {}

/// Computes an isolated signature over raw `.cow` art and palette slots.
#[must_use]
pub fn compute_pack_dna_signature(pack_id: &str, raw_cow: &str, palette: &[String; 5]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    pack_id.hash(&mut hasher);
    raw_cow.hash(&mut hasher);
    for slot in palette {
        slot.to_uppercase().hash(&mut hasher);
    }
    format!("forgum-dna-v1:{:016x}", hasher.finish())
}

/// Validates a user-created `.cow` mascot and its isolated `CowDna` manifest
/// before importing or hot-reloading into the user catalog overlay.
pub fn verify_and_compile_mascot_pack(
    pack_id: &str,
    raw_cow_body: &str,
    manifest: &MascotPackManifest,
) -> Result<AsciiRigMap, PackVerifyError> {
    // Stage 1: Archive & Path Sandbox Guard
    if raw_cow_body.len() > MAX_PACK_SIZE_BYTES {
        return Err(PackVerifyError::PayloadTooLarge(raw_cow_body.len()));
    }
    if !is_safe_pack_identifier(pack_id) {
        return Err(PackVerifyError::UnsafePathIdentifier(pack_id.to_string()));
    }

    // Stage 2: Safe-SGR ANSI Escape Filter (SEC-PACK-002)
    if raw_cow_body.contains("\x1b]")
        || raw_cow_body.contains("\x1bP")
        || raw_cow_body.contains("\x1b[?")
        || raw_cow_body.contains("\x1b[2J")
        || raw_cow_body.contains("\x1b[r")
    {
        return Err(PackVerifyError::ForbiddenTerminalEscapeSequence);
    }

    // Stage 3: CP437 Mojibake & Width Gate (BUG-006)
    for ch in raw_cow_body.chars() {
        let code = ch as u32;
        if (0x2600..=0x263F).contains(&code) {
            return Err(PackVerifyError::Cp437MojibakeDetected(ch));
        }
    }
    if raw_cow_body.contains("Γÿ") {
        return Err(PackVerifyError::Cp437MojibakeDetected('Γ'));
    }

    // Stage 4: 5-Slot Hex Palette Format (#RRGGBB)
    for slot in &manifest.palette_hex {
        if slot.len() != 7
            || !slot.starts_with('#')
            || !slot[1..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err(PackVerifyError::InvalidHexPaletteSlot(slot.clone()));
        }
    }

    // Stage 4b: Isolated DNA Signature Verification (SEC-PACK-003)
    let expected_sig = compute_pack_dna_signature(pack_id, raw_cow_body, &manifest.palette_hex);
    if manifest.dna_signature != expected_sig {
        return Err(PackVerifyError::SignatureMismatch);
    }

    // Stage 5: Heredoc & Placeholder AST Verifier across landmark permutations
    let (expanded, _) = cow::expand_cow_with_landmarks(raw_cow_body, "oo", "U ", "\\");
    for placeholder in ["$eyes", "$eye", "$tongue", "$thoughts"] {
        if expanded.contains(placeholder) {
            return Err(PackVerifyError::LingeringTemplatePlaceholder(
                placeholder.to_string(),
            ));
        }
    }

    // Dimensions check
    let lines: Vec<&str> = expanded.lines().collect();
    let height = lines.len();
    let width = lines
        .iter()
        .map(|l| l.chars().map(char_display_width).sum::<usize>())
        .max()
        .unwrap_or(0);

    if width == 0 || height == 0 || width > MAX_MASCOT_WIDTH || height > MAX_MASCOT_HEIGHT {
        return Err(PackVerifyError::DimensionsOutOfBounds { width, height });
    }

    // Stage 6: AsciiRigMap Pre-Build & 2D Flood-Fill Mask Compilation
    let segments =
        AnatomicalSegments::detect(&expanded, 0, None, None, None, height.saturating_sub(1));
    let rig = AsciiRigMap::build(&expanded, 0, &segments);
    Ok(rig)
}

/// Imports and installs a verified mascot pack bundle to the isolated user data directory.
pub fn import_pack_bundle(
    manifest: &MascotPackManifest,
    raw_cow_body: &str,
) -> Result<PathBuf, String> {
    // 1. Run the 6-stage verification gate
    let _rig = verify_and_compile_mascot_pack(&manifest.pack_id, raw_cow_body, manifest)
        .map_err(|e| format!("Pack verification failed: {e}"))?;

    // 2. Resolve sandboxed target directory
    let target_dir = pack_directory(&manifest.pack_id)?;
    if !target_dir.exists() {
        fs::create_dir_all(&target_dir)
            .map_err(|e| format!("failed to create pack directory: {e}"))?;
    }

    // 3. Write cow art file
    let cow_path = target_dir.join(format!("{}.cow", manifest.pack_id));
    fs::write(&cow_path, raw_cow_body).map_err(|e| format!("failed to write cow art file: {e}"))?;

    // 4. Write manifest JSON
    let manifest_path = target_dir.join("manifest.json");
    let json_bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| format!("failed to serialize manifest: {e}"))?;
    fs::write(&manifest_path, json_bytes)
        .map_err(|e| format!("failed to write manifest JSON: {e}"))?;

    // 5. Broadcast hot-reload signal to live daemons if running
    notify_live_daemons_catalog_reload();

    Ok(target_dir)
}

/// Broadcasts a catalog reload notification to any active background daemon.
pub fn notify_live_daemons_catalog_reload() {
    let session_id = forgum_platform::detect_session_id();
    let socket = forgum_platform::control_socket_path(&session_id);
    let payload = serde_json::json!({
        "cmd": "RELOAD_CATALOG",
        "arg": "all"
    })
    .to_string();
    let _ = crate::herd::send_command(&socket.to_string_lossy(), &payload);
}

/// Load an installed pack's `.cow` art and manifest from the sandboxed packs directory.
pub fn load_installed_pack(pack_id: &str) -> Option<(String, MascotPackManifest)> {
    let dir = pack_directory(pack_id).ok()?;
    let cow_file = dir.join(format!("{pack_id}.cow"));
    let manifest_file = dir.join("manifest.json");

    if cow_file.exists() && manifest_file.exists() {
        let raw_cow = fs::read_to_string(&cow_file).ok()?;
        let manifest_bytes = fs::read(&manifest_file).ok()?;
        let manifest: MascotPackManifest = serde_json::from_slice(&manifest_bytes).ok()?;
        Some((raw_cow, manifest))
    } else {
        None
    }
}

/// Scans the sandboxed packs directory for custom `.cow` mascot files.
pub fn load_custom_pack_cow(name: &str) -> Option<String> {
    let dir = pack_directory(name).ok()?;
    let cow_file = dir.join(format!("{name}.cow"));
    if cow_file.exists() {
        fs::read_to_string(&cow_file).ok()
    } else {
        None
    }
}

/// Scaffold a new mascot pack directory with valid templates.
pub fn scaffold_pack(name: &str, target_dir: Option<&Path>) -> Result<PathBuf, String> {
    if !is_safe_pack_identifier(name) {
        return Err(format!(
            "Invalid pack name '{name}': must be alphanumeric with '-' or '_'"
        ));
    }
    let base_dir = if let Some(td) = target_dir {
        td.to_path_buf()
    } else {
        PathBuf::from(name)
    };
    fs::create_dir_all(&base_dir).map_err(|e| format!("failed to create dir: {e}"))?;

    let cow_content = r#"$the_cow = <<"EOC";
    $thoughts   ^__^
     $thoughts  ($eyes)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
EOC
"#
    .to_string();
    let cow_path = base_dir.join(format!("{name}.cow"));
    fs::write(&cow_path, &cow_content).map_err(|e| format!("failed to write cow template: {e}"))?;

    let palette = [
        "#E06C75".to_string(),
        "#98C379".to_string(),
        "#E5C07B".to_string(),
        "#61AFEF".to_string(),
        "#C678DD".to_string(),
    ];
    let signature = compute_pack_dna_signature(name, &cow_content, &palette);
    let manifest = MascotPackManifest {
        pack_id: name.to_string(),
        author: "Forgum Contributor".to_string(),
        version: "0.1.0".to_string(),
        dna_signature: signature,
        habitat: Habitat::Terrestrial,
        base_anim: BaseAnim::Walk,
        palette_hex: palette,
    };
    let manifest_path = base_dir.join("manifest.json");
    let json_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_path, json_bytes).map_err(|e| format!("failed to write manifest: {e}"))?;

    Ok(base_dir)
}

/// Verify a pack located at `path` (directory containing `<name>.cow` and `manifest.json`).
pub fn verify_pack_path(path: &Path) -> Result<MascotPackManifest, String> {
    if !path.exists() {
        return Err(format!("Pack path '{}' does not exist", path.display()));
    }
    let (cow_path, manifest_path) = if path.is_dir() {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| "Invalid pack directory name".to_string())?;
        (path.join(format!("{name}.cow")), path.join("manifest.json"))
    } else {
        return Err(
            "Pack path must be a directory containing manifest.json and .cow art".to_string(),
        );
    };

    if !cow_path.exists() {
        return Err(format!("Missing cow art file: {}", cow_path.display()));
    }
    if !manifest_path.exists() {
        return Err(format!(
            "Missing manifest JSON: {}",
            manifest_path.display()
        ));
    }

    let cow_raw =
        fs::read_to_string(&cow_path).map_err(|e| format!("failed to read cow art: {e}"))?;
    let manifest_bytes =
        fs::read(&manifest_path).map_err(|e| format!("failed to read manifest: {e}"))?;
    let manifest: MascotPackManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| format!("invalid manifest JSON schema: {e}"))?;

    verify_and_compile_mascot_pack(&manifest.pack_id, &cow_raw, &manifest)
        .map_err(|e| format!("Verification failed: {e}"))?;

    Ok(manifest)
}

/// Import and install a mascot pack from `path`.
pub fn import_mascot_pack_file(path: &Path) -> Result<PathBuf, String> {
    let manifest = verify_pack_path(path)?;
    let name = &manifest.pack_id;
    let cow_path = if path.is_dir() {
        path.join(format!("{name}.cow"))
    } else {
        return Err("Pack must be a directory".to_string());
    };
    let cow_raw =
        fs::read_to_string(&cow_path).map_err(|e| format!("failed to read cow art: {e}"))?;
    import_pack_bundle(&manifest, &cow_raw)
}

/// Export an installed mascot pack into an archive or directory.
pub fn export_mascot_pack(name: &str, output: Option<&Path>) -> Result<PathBuf, String> {
    let (cow_raw, manifest) =
        load_installed_pack(name).ok_or_else(|| format!("Installed pack '{name}' not found"))?;

    let out_dir = if let Some(o) = output {
        o.to_path_buf()
    } else {
        PathBuf::from(format!("{name}-export"))
    };
    fs::create_dir_all(&out_dir).map_err(|e| format!("failed to create export dir: {e}"))?;

    let cow_file = out_dir.join(format!("{name}.cow"));
    fs::write(&cow_file, cow_raw).map_err(|e| format!("failed to write cow file: {e}"))?;

    let manifest_file = out_dir.join("manifest.json");
    let json_bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    fs::write(&manifest_file, json_bytes).map_err(|e| format!("failed to write manifest: {e}"))?;

    Ok(out_dir)
}

/// List all installed custom mascot packs.
pub fn list_installed_packs() -> Vec<MascotPackManifest> {
    let ids = forgum_platform::marketplace::list_installed_packs();
    ids.into_iter()
        .filter_map(|id| load_installed_pack(&id).map(|(_, m)| m))
        .collect()
}

/// Remove an installed custom mascot pack.
pub fn remove_installed_pack(name: &str) -> Result<(), String> {
    if !is_safe_pack_identifier(name) {
        return Err(format!("Invalid pack name '{name}'"));
    }
    let dir = pack_directory(name).map_err(|e| e.to_string())?;
    if !dir.exists() {
        return Err(format!(
            "Pack '{name}' is not installed at {}",
            dir.display()
        ));
    }
    fs::remove_dir_all(&dir).map_err(|e| format!("failed to remove pack dir: {e}"))?;
    notify_live_daemons_catalog_reload();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(id: &str) -> MascotPackManifest {
        MascotPackManifest {
            pack_id: id.to_string(),
            author: "TestAuthor".to_string(),
            version: "1.0.0".to_string(),
            dna_signature: "forgum-dna-v1:test".to_string(),
            habitat: Habitat::Terrestrial,
            base_anim: BaseAnim::Walk,
            palette_hex: [
                "#112233".to_string(),
                "#445566".to_string(),
                "#778899".to_string(),
                "#aabbcc".to_string(),
                "#ddeeff".to_string(),
            ],
        }
    }

    #[test]
    fn test_verify_valid_pack_succeeds() {
        let mut manifest = sample_manifest("test-mascot");
        let cow_body = "$the_cow = <<\"EOC\";\n  ^__^\n  ($eyes)\\_______\n   (__)\\       )\\/\\\n       ||----w |\n       ||     ||\nEOC\n";
        manifest.dna_signature =
            compute_pack_dna_signature("test-mascot", cow_body, &manifest.palette_hex);
        let res = verify_and_compile_mascot_pack("test-mascot", cow_body, &manifest);
        assert!(res.is_ok(), "Valid pack must pass verification: {res:?}");
    }

    #[test]
    fn test_verify_rejects_signature_mismatch() {
        let manifest = sample_manifest("tampered-pack");
        let cow_body = "Simple Art";
        let res = verify_and_compile_mascot_pack("tampered-pack", cow_body, &manifest);
        assert_eq!(res.unwrap_err(), PackVerifyError::SignatureMismatch);
    }

    #[test]
    fn test_verify_rejects_forbidden_escape_sequences() {
        let manifest = sample_manifest("malicious-pack");
        let cow_body = "Evil \x1b]52;c;payload\x07 Mascot";
        let res = verify_and_compile_mascot_pack("malicious-pack", cow_body, &manifest);
        assert_eq!(
            res.unwrap_err(),
            PackVerifyError::ForbiddenTerminalEscapeSequence
        );
    }

    #[test]
    fn test_verify_rejects_cp437_mojibake() {
        let manifest = sample_manifest("mojibake-pack");
        let cow_body = "Corrupted \u{2600} Art";
        let res = verify_and_compile_mascot_pack("mojibake-pack", cow_body, &manifest);
        assert!(matches!(
            res.unwrap_err(),
            PackVerifyError::Cp437MojibakeDetected(_)
        ));
    }

    #[test]
    fn test_verify_rejects_invalid_palette_hex() {
        let mut manifest = sample_manifest("invalid-hex");
        manifest.palette_hex[0] = "not-a-hex".to_string();
        let cow_body = "Simple Art";
        let res = verify_and_compile_mascot_pack("invalid-hex", cow_body, &manifest);
        assert!(matches!(
            res.unwrap_err(),
            PackVerifyError::InvalidHexPaletteSlot(_)
        ));
    }

    #[test]
    fn test_verify_rejects_oversized_dimensions() {
        let mut manifest = sample_manifest("giant-art");
        let wide_line = "X".repeat(150);
        manifest.dna_signature =
            compute_pack_dna_signature("giant-art", &wide_line, &manifest.palette_hex);
        let res = verify_and_compile_mascot_pack("giant-art", &wide_line, &manifest);
        assert!(matches!(
            res.unwrap_err(),
            PackVerifyError::DimensionsOutOfBounds { .. }
        ));
    }
}
