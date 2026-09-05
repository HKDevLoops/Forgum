//! Strongly-typed error definitions for `forgum-engine`.
//!
//! Provides granular, contextual error variants mapping to standard POSIX/BSD sysexits codes.

use std::path::PathBuf;

/// Engine-level errors returned across simulation, rendering, and runner pipelines.
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Platform error: {0}")]
    Platform(#[from] forgum_platform::PlatformError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Render initialization failure: {0}")]
    RenderInit(String),

    #[error("Failed to load mascot '{name}': {message}")]
    MascotLoad { name: String, message: String },

    #[error("Mascot template file not found at path: {0}")]
    MascotNotFound(PathBuf),

    #[error("Terminal window is too small ({cols}x{rows}). Minimum dimensions are 20x5.")]
    TerminalTooSmall { cols: usize, rows: usize },

    #[error("Control channel disconnected: {0}")]
    ControlDisconnected(String),

    #[error("Daemon communication timeout after {timeout_secs}s")]
    DaemonTimeout { timeout_secs: u64 },

    #[error("Configuration validation error: {0}")]
    Config(String),
}

impl EngineError {
    /// Return POSIX / BSD exit code matching this error variant.
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Platform(p) => p.exit_code(),
            Self::Io(_) | Self::RenderInit(_) => 78, // EX_CONFIG
            Self::MascotLoad { .. } | Self::MascotNotFound(_) | Self::Config(_) => 65, // EX_DATAERR
            Self::TerminalTooSmall { .. } => 64,     // EX_USAGE
            Self::ControlDisconnected(_) | Self::DaemonTimeout { .. } => 71, // EX_OSERR
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_error_exit_codes() {
        assert_eq!(EngineError::RenderInit("fail".into()).exit_code(), 78);
        assert_eq!(
            EngineError::MascotLoad {
                name: "corgi".into(),
                message: "corrupted".into()
            }
            .exit_code(),
            65
        );
        assert_eq!(
            EngineError::TerminalTooSmall { cols: 10, rows: 2 }.exit_code(),
            64
        );
        assert_eq!(
            EngineError::ControlDisconnected("closed".into()).exit_code(),
            71
        );
    }
}
