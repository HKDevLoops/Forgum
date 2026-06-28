//! Scene reader — bounded stdin or `--file`.
//!
//! **The fix for BUG-D4**: stdin is capped at 4 MB. Anything larger returns
//! [`PlatformError::InvalidArgument`] with a clear message, and `main()`
//! exits with code 65 (`EX_DATAERR`).
//!
//! **The fix for BUG-D5**: any parse failure exits non-zero. Callers can
//! detect failure via `$LASTEXITCODE` (PowerShell) or `$?` (bash).

use std::fs;
use std::io::{self, Read};
use std::path::Path;

use forgum_platform::PlatformError;

use crate::protocol::SceneConfig;

/// Maximum input size. 4 MB is generous for a cow file plus a fortune plus
/// scene metadata; anything bigger is almost certainly an attack or a bug.
pub const MAX_INPUT: usize = 4 * 1024 * 1024;

/// Read a scene config from `--file` or stdin (in that order). When neither
/// is set, returns `SceneConfig::default()`.
///
/// If `cleanup` is true and the file was read successfully, it is deleted
/// after parsing (BUG-D2 fix).
pub fn read_scene(file: Option<&Path>, cleanup: bool) -> Result<SceneConfig, PlatformError> {
    let raw = if let Some(p) = file {
        let bytes = fs::read(p)?;
        if bytes.len() >= MAX_INPUT {
            return Err(PlatformError::InvalidArgument(format!(
                "--file too large: {} bytes (cap {})",
                bytes.len(),
                MAX_INPUT
            )));
        }
        if cleanup {
            let _ = fs::remove_file(p);
        }
        bytes
    } else if atty_stdin() {
        return Ok(SceneConfig::default());
    } else {
        let mut buf = Vec::with_capacity(8 * 1024);
        let mut handle = io::stdin().take(MAX_INPUT as u64);
        handle.read_to_end(&mut buf)?;
        if buf.len() >= MAX_INPUT {
            return Err(PlatformError::InvalidArgument(format!(
                "stdin too large (>= {} bytes)",
                MAX_INPUT
            )));
        }
        buf
    };

    if raw.iter().all(|b| b.is_ascii_whitespace()) {
        return Ok(SceneConfig::default());
    }

    serde_json::from_slice::<SceneConfig>(&raw).map_err(|e| PlatformError::ConfigParse {
        path: file.map_or_else(
            || std::path::PathBuf::from("<stdin>"),
            std::path::Path::to_path_buf,
        ),
        message: e.to_string(),
    })
}

fn atty_stdin() -> bool {
    crossterm::tty::IsTty::is_tty(&io::stdin())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_valid_json() {
        let json = r#"{"cow":"tux","text":"hi"}"#;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("scene.json");
        std::fs::write(&path, json).unwrap();
        let scene = read_scene(Some(&path), false).unwrap();
        assert_eq!(scene.cow, "tux");
        assert_eq!(scene.text, "hi");
    }

    #[test]
    fn cleanup_removes_file() {
        let json = r#"{"cow":"tux"}"#;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("scene.json");
        std::fs::write(&path, json).unwrap();
        let _scene = read_scene(Some(&path), true).unwrap();
        assert!(!path.exists(), "file should be removed after read");
    }

    #[test]
    fn no_cleanup_preserves_file() {
        let json = r#"{"cow":"tux"}"#;
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("scene.json");
        std::fs::write(&path, json).unwrap();
        let _scene = read_scene(Some(&path), false).unwrap();
        assert!(path.exists(), "file should still exist when cleanup=false");
    }

    #[test]
    fn parse_invalid_json_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("bad.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(b"{ not valid json").unwrap();
        let r = read_scene(Some(&path), false);
        assert!(matches!(r, Err(PlatformError::ConfigParse { .. })));
    }

    #[test]
    fn huge_file_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("huge.json");
        let f = std::fs::File::create(&path).unwrap();
        f.set_len(MAX_INPUT as u64).unwrap();
        let r = read_scene(Some(&path), false);
        assert!(matches!(r, Err(PlatformError::InvalidArgument(_))));
    }

    #[test]
    fn whitespace_only_is_default() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("ws.json");
        std::fs::write(&path, "   \n\n\t ").unwrap();
        let scene = read_scene(Some(&path), false).unwrap();
        assert_eq!(scene, SceneConfig::default());
    }

    #[test]
    fn missing_file_errors() {
        let r = read_scene(Some(Path::new("/nonexistent/forgum-test/file.json")), false);
        assert!(matches!(r, Err(PlatformError::Io(_))));
    }
}
