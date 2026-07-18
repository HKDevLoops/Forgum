//! Typed errors for `forgum-platform`.
//!
//! All fallible operations in this crate return `Result<T, PlatformError>`.
//! We deliberately use `thiserror` (not `anyhow`) so callers can pattern-match
//! on the failure mode and present a useful message to the user — and so the
//! engine binary can convert specific variants into specific exit codes.

use std::path::PathBuf;

/// All errors that can be returned by `forgum-platform` functions.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("terminal is not a TTY and no fallback (e.g. /dev/tty, CONOUT$) is available")]
    NoTerminal,

    #[error("refusing to operate on path outside the configured root: {0}")]
    PathEscape(PathBuf),

    #[error("config file is not valid UTF-8: {0}")]
    ConfigEncoding(PathBuf),

    #[error("config file is not valid JSON ({path}): {message}")]
    ConfigParse { path: PathBuf, message: String },

    #[error("signal handler registration failed for {signal}: {source}")]
    SignalRegistration {
        signal: &'static str,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to detach child process: {0}")]
    Detach(String),

    #[error("unsupported terminal capability: {0}")]
    Unsupported(&'static str),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),
}

impl PlatformError {
    /// Exit code to use when surfacing this error from the engine binary.
    ///
    /// - I/O, terminal-not-found, path-escape: `78` (`EX_CONFIG` — configuration error).
    /// - Parse / encoding: `65` (`EX_DATAERR` — input data error).
    /// - Signal / detach: `71` (`EX_OSERR` — OS-level error).
    /// - Invalid argument: `64` (`EX_USAGE`).
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Io(_) | Self::NoTerminal | Self::PathEscape(_) => 78,
            Self::ConfigEncoding(_) | Self::ConfigParse { .. } => 65,
            Self::SignalRegistration { .. } | Self::Detach(_) => 71,
            Self::Unsupported(_) | Self::InvalidArgument(_) => 64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_config_class_is_78() {
        assert_eq!(
            PlatformError::Io(std::io::Error::other("x")).exit_code(),
            78
        );
        assert_eq!(PlatformError::NoTerminal.exit_code(), 78);
        assert_eq!(
            PlatformError::PathEscape(PathBuf::from("/tmp/x")).exit_code(),
            78
        );
    }

    #[test]
    fn exit_code_data_class_is_65() {
        assert_eq!(
            PlatformError::ConfigEncoding(PathBuf::from("e")).exit_code(),
            65
        );
        assert_eq!(
            PlatformError::ConfigParse {
                path: PathBuf::from("e"),
                message: "bad".into(),
            }
            .exit_code(),
            65
        );
    }

    #[test]
    fn exit_code_os_class_is_71() {
        assert_eq!(
            PlatformError::SignalRegistration {
                signal: "SIGINT",
                source: std::io::Error::other("x"),
            }
            .exit_code(),
            71
        );
        assert_eq!(PlatformError::Detach("x".into()).exit_code(), 71);
    }

    #[test]
    fn exit_code_usage_class_is_64() {
        assert_eq!(PlatformError::Unsupported("x").exit_code(), 64);
        assert_eq!(PlatformError::InvalidArgument("x".into()).exit_code(), 64);
    }

    #[test]
    fn display_messages_are_descriptive() {
        assert!(format!("{}", PlatformError::InvalidArgument("bad".into()))
            .contains("invalid argument"));
        assert!(format!("{}", PlatformError::NoTerminal).contains("TTY"));
    }

    #[test]
    fn from_io_error_produces_io_variant() {
        let io = std::io::Error::other("boom");
        let e: PlatformError = io.into();
        assert!(matches!(e, PlatformError::Io(_)));
    }
}
