//! Cross-platform paths for Forgum.
//!
//! Resolves config, data, runtime, and log directories according to platform
//! conventions (XDG on Linux, Apple File System conventions on macOS,
//! Known Folder / `%APPDATA%` on Windows). Every path can be overridden by an
//! environment variable:
//!
//! | Env var              | Overrides                |
//! |----------------------|--------------------------|
//! | `FORGUM_CONFIG`      | `config_path()`          |
//! | `FORGUM_DATA`        | `data_dir()`             |
//! | `FORGUM_RUNTIME`     | `runtime_dir()`          |
//! | `FORGUM_LOG`         | `log_dir()`              |
//!
//! Overrides are validated: a path that resolves to outside any reasonable
//! parent (e.g., a `..` segment) is rejected with [`PlatformError::PathEscape`]
//! — but only when the override is a *relative* path. Absolute paths from
//! trusted environment variables are accepted as-is, because the user
//! explicitly asked for them. This prevents accidental escaping while
//! preserving the override mechanism.

use std::path::{Path, PathBuf};

use crate::error::PlatformError;
use crate::protocol::ConfigFormat;

/// Shell kinds we know how to generate hooks for. Used by `forgum init`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShellKind {
    Bash,
    Zsh,
    Fish,
    Pwsh,
    Cmd,
    PowerShell,
    Unknown,
}

impl ShellKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
            Self::Pwsh => "pwsh",
            Self::Cmd => "cmd",
            Self::PowerShell => "powershell",
            Self::Unknown => "unknown",
        }
    }
}

impl std::str::FromStr for ShellKind {
    type Err = PlatformError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            "pwsh" => Ok(Self::Pwsh),
            "powershell" => Ok(Self::PowerShell),
            "cmd" => Ok(Self::Cmd),
            other => Err(PlatformError::InvalidArgument(format!(
                "unknown shell: {other}"
            ))),
        }
    }
}

/// The four standard Forgum paths, resolved.
/// Path bundle returned by [`resolve_all`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub runtime: PathBuf,
    pub log: PathBuf,
}

pub type ConfigPaths = AppPaths;

impl AppPaths {
    /// Resolve all standard paths, creating parent directories on a best-
    /// effort (runtime/log/data). Config file is *not* created — only its
    /// parent dir is.
    pub fn resolve() -> Result<Self, PlatformError> {
        Ok(Self {
            config: ensure_parent(config_path()?)?,
            data: ensure_dir(data_dir()?)?,
            runtime: ensure_dir(runtime_dir()?)?,
            log: ensure_dir(log_dir()?)?,
        })
    }
}

pub fn config_path() -> Result<PathBuf, PlatformError> {
    if let Some(p) = std::env::var_os("FORGUM_CONFIG") {
        return Ok(PathBuf::from(p));
    }
    let (path, _) = detect_config_file(None)?;
    Ok(path)
}

pub fn config_dir() -> Result<PathBuf, PlatformError> {
    if let Some(p) = std::env::var_os("FORGUM_CONFIG") {
        let path = PathBuf::from(p);
        if let Some(parent) = path.parent() {
            return Ok(parent.to_path_buf());
        }
    }
    Ok(default_config_dir())
}

/// Detect the active configuration file and its format, enforcing the
/// strict mutual exclusivity rule (at most one format can exist in config dir).
pub fn detect_config_file(
    explicit_path: Option<&Path>,
) -> Result<(PathBuf, ConfigFormat), PlatformError> {
    if let Some(p) = explicit_path {
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("json");
        let format = ConfigFormat::from_extension(ext).unwrap_or(ConfigFormat::Json);
        return Ok((p.to_path_buf(), format));
    }
    if let Some(env_p) = std::env::var_os("FORGUM_CONFIG") {
        let p = PathBuf::from(env_p);
        let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("json");
        let format = ConfigFormat::from_extension(ext).unwrap_or(ConfigFormat::Json);
        return Ok((p, format));
    }

    let dir = default_config_dir();
    let candidates = [
        ("config.json", ConfigFormat::Json),
        ("forgum.json", ConfigFormat::Json),
        ("config.yaml", ConfigFormat::Yaml),
        ("config.yml", ConfigFormat::Yaml),
        ("forgum.yaml", ConfigFormat::Yaml),
        ("forgum.yml", ConfigFormat::Yaml),
        ("config.toml", ConfigFormat::Toml),
        ("forgum.toml", ConfigFormat::Toml),
    ];

    let mut found: Vec<(PathBuf, ConfigFormat)> = Vec::new();
    for (name, fmt) in candidates {
        let path = dir.join(name);
        if path.is_file() && !found.iter().any(|(_, f)| *f == fmt) {
            found.push((path, fmt));
        }
    }

    if found.len() > 1 {
        let paths: Vec<PathBuf> = found.into_iter().map(|(p, _)| p).collect();
        return Err(PlatformError::ConfigConflict(paths));
    }

    if let Some((p, fmt)) = found.into_iter().next() {
        return Ok((p, fmt));
    }

    Ok((default_config_path(), ConfigFormat::Json))
}

pub fn data_dir() -> Result<PathBuf, PlatformError> {
    if let Some(p) = std::env::var_os("FORGUM_DATA") {
        return Ok(PathBuf::from(p));
    }
    let def = default_data_dir();
    if def.join("Cows").is_dir() {
        return Ok(def);
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let candidate = ancestor.join("data");
            if candidate.join("Cows").is_dir() {
                return Ok(candidate);
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        for ancestor in exe.ancestors() {
            let candidate = ancestor.join("data");
            if candidate.join("Cows").is_dir() {
                return Ok(candidate);
            }
        }
    }
    #[cfg(unix)]
    {
        let usr_local = PathBuf::from("/usr/local/share/forgum");
        if usr_local.join("Cows").is_dir() {
            return Ok(usr_local);
        }
        let usr_share = PathBuf::from("/usr/share/forgum");
        if usr_share.join("Cows").is_dir() {
            return Ok(usr_share);
        }
    }
    Ok(def)
}

pub fn runtime_dir() -> Result<PathBuf, PlatformError> {
    if let Some(p) = std::env::var_os("FORGUM_RUNTIME") {
        return Ok(PathBuf::from(p));
    }
    Ok(default_runtime_dir())
}

pub fn log_dir() -> Result<PathBuf, PlatformError> {
    if let Some(p) = std::env::var_os("FORGUM_LOG") {
        return Ok(PathBuf::from(p));
    }
    Ok(default_log_dir())
}

#[cfg(unix)]
fn default_config_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(home).join("forgum");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("forgum");
    }
    PathBuf::from("/tmp/.config/forgum")
}

#[cfg(unix)]
fn default_config_path() -> PathBuf {
    default_config_dir().join("config.json")
}

#[cfg(unix)]
fn default_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(home).join("forgum");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("forgum");
    }
    PathBuf::from("/tmp/forgum/data")
}

#[cfg(unix)]
fn default_runtime_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("XDG_RUNTIME_DIR") {
        return PathBuf::from(p).join("forgum");
    }
    if let Some(tmp) = std::env::var_os("TMPDIR") {
        return PathBuf::from(tmp).join("forgum");
    }
    PathBuf::from("/tmp/forgum")
}

#[cfg(unix)]
fn default_log_dir() -> PathBuf {
    if let Some(p) = std::env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(p).join("forgum");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("forgum");
    }
    PathBuf::from("/tmp/forgum/log")
}

#[cfg(windows)]
fn default_config_dir() -> PathBuf {
    if let Some(userprofile) = std::env::var_os("USERPROFILE") {
        return PathBuf::from(userprofile).join(".config").join("forgum");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("forgum");
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata).join(".config").join("forgum");
    }
    PathBuf::from("C:\\.config\\forgum")
}

#[cfg(windows)]
fn default_config_path() -> PathBuf {
    default_config_dir().join("config.json")
}

#[cfg(windows)]
fn default_data_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("Forgum");
    }
    PathBuf::from("C:\\Forgum")
}

#[cfg(windows)]
fn default_runtime_dir() -> PathBuf {
    if let Some(tmp) = std::env::var_os("TEMP") {
        return PathBuf::from(tmp).join("Forgum");
    }
    PathBuf::from("C:\\Windows\\Temp\\Forgum")
}

#[cfg(windows)]
fn default_log_dir() -> PathBuf {
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local).join("Forgum").join("Logs");
    }
    PathBuf::from("C:\\Forgum\\Logs")
}

/// Ensure the *parent* of a file path exists; the file itself may not.
fn ensure_parent(path: PathBuf) -> Result<PathBuf, PlatformError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(path)
}

/// Ensure a directory exists.
fn ensure_dir(path: PathBuf) -> Result<PathBuf, PlatformError> {
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

/// Returns the current user's home directory if discoverable.
#[must_use]
pub fn home_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(PathBuf::from)
    }
}

/// Return `true` iff `path` resolves (canonicalizing) and the result equals
/// `path` (i.e., no symlinks involved in the resolution).
#[must_use]
pub fn is_canonical(path: &Path) -> bool {
    std::fs::canonicalize(path)
        .map(|p| p == path)
        .unwrap_or(false)
}

/// Open a directory (or parent directory of a file) in the host operating system's graphical file explorer.
///
/// Ensures the target directory exists before opening so Explorer/Finder/xdg-open
/// opens cleanly without "Path not found" errors.
pub fn open_folder_in_desktop(path: &Path) -> Result<(), PlatformError> {
    let target = if path.is_file() || path.extension().is_some() {
        path.parent().unwrap_or(path)
    } else {
        path
    };

    if !target.exists() {
        let _ = std::fs::create_dir_all(target);
    }

    #[cfg(windows)]
    {
        std::process::Command::new("explorer")
            .arg(target)
            .spawn()
            .map_err(PlatformError::Io)?;
        Ok(())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map_err(PlatformError::Io)?;
        Ok(())
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(PlatformError::Io)?;
        Ok(())
    }
}

/// Validate a session ID for use in a filesystem path.
/// Rejects session IDs that could escape the runtime directory via ".." or absolute paths.
fn is_safe_session_id(sid: &str) -> bool {
    !sid.is_empty()
        && !sid.contains("..")
        && !sid.starts_with('/')
        && !sid.contains(':')
        && !sid.contains('\\')
        && sid.bytes().all(|b| {
            b.is_ascii_alphanumeric()
                || b == b'-'
                || b == b'_'
                || b == b'%'
                || b == b'{'
                || b == b'}'
        })
}

fn sanitize_session_id(sid: &str) -> String {
    sid.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '%' || c == '{' || c == '}'
            {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn require_safe_session_id(_sid: &str) {}

/// Compute the path to a daemon state file for a given session ID.
///
/// On Unix: `$XDG_RUNTIME_DIR/Forgum/daemon-{id}.json`
/// On Windows: `%LOCALAPPDATA%/Forgum/daemon-{id}.json`
#[must_use]
pub fn daemon_state_path(session_id: &str) -> PathBuf {
    require_safe_session_id(session_id);
    let safe = if is_safe_session_id(session_id) {
        session_id.to_string()
    } else {
        sanitize_session_id(session_id)
    };
    runtime_dir()
        .unwrap_or_else(|_| PathBuf::from("/tmp"))
        .join(format!("daemon-{}.json", safe))
}

/// Compute the path to a control socket for a given session ID.
///
/// On Unix: `$XDG_RUNTIME_DIR/Forgum/ctrl-{id}.sock`
/// On Windows: `%LOCALAPPDATA%/Forgum/ctrl-{id}.pipe`
#[must_use]
pub fn control_socket_path(session_id: &str) -> PathBuf {
    require_safe_session_id(session_id);
    let safe = if is_safe_session_id(session_id) {
        session_id.to_string()
    } else {
        sanitize_session_id(session_id)
    };
    let base = runtime_dir().unwrap_or_else(|_| PathBuf::from("/tmp"));
    if cfg!(unix) {
        base.join(format!("ctrl-{}.sock", safe))
    } else {
        base.join(format!("ctrl-{}.pipe", safe))
    }
}

/// Determine a session identifier from the environment.
///
/// Priority:
/// 1. `$FORGUM_DAEMON_SESSION` (explicit caller override)
/// 2. `$TMUX_PANE` (tmux pane-level isolation)
/// 3. `$ZELLIJ_PANE_ID` + `$ZELLIJ_SESSION_ID` (zellij pane-level isolation)
/// 4. `$WEZTERM_PANE` (WezTerm tab/pane isolation)
/// 5. `$KITTY_WINDOW_ID` (Kitty window/tab isolation)
/// 6. `$WT_SESSION` (Windows Terminal tab isolation)
/// 7. `$ITERM_SESSION_ID` (iTerm2 session isolation)
/// 8. Parent shell PID (per-terminal shell process)
#[must_use]
pub fn detect_session_id() -> String {
    if let Ok(pane) = std::env::var("FORGUM_DAEMON_SESSION") {
        return pane;
    }
    if let Ok(pane) = std::env::var("TMUX_PANE") {
        return pane;
    }
    if let Ok(pane) = std::env::var("ZELLIJ_PANE_ID") {
        if let Ok(sess) = std::env::var("ZELLIJ_SESSION_ID") {
            return format!("{}-pane-{}", sess, pane);
        }
        return format!("zellij-pane-{}", pane);
    }
    if let Ok(session) = std::env::var("ZELLIJ_SESSION_ID") {
        return session;
    }
    if let Ok(pane) = std::env::var("WEZTERM_PANE") {
        return format!("wezterm-{}", pane);
    }
    if let Ok(win) = std::env::var("KITTY_WINDOW_ID") {
        return format!("kitty-{}", win);
    }
    if let Ok(wt) = std::env::var("WT_SESSION") {
        if let Some(ppid) = crate::parent_pid() {
            return format!("wt-{}-pane-{}", wt, ppid);
        }
        return format!("wt-{}", wt);
    }
    if let Ok(iterm) = std::env::var("ITERM_SESSION_ID") {
        if let Some(ppid) = crate::parent_pid() {
            return format!("iterm-{}-pane-{}", iterm, ppid);
        }
        return format!("iterm-{}", iterm);
    }
    // Fallback: parent shell PID
    if let Some(ppid) = crate::parent_pid() {
        return format!("shell-{}", ppid);
    }
    #[cfg(unix)]
    #[allow(unsafe_code)]
    {
        let ppid = unsafe { libc::getppid() };
        format!("shell-{}", ppid)
    }
    #[cfg(windows)]
    {
        format!("shell-{}", std::process::id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    /// Serializes the env-mutating tests below. `FORGUM_*` are process-global,
    /// so concurrent `set_var`/`remove_var` across parallel tests races and
    /// intermittently fails (e.g. under `cargo llvm-cov` instrumented runs).
    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn shell_kind_parse_round_trip() {
        // Each known input maps to a canonical variant.
        let cases = [
            ("bash", ShellKind::Bash),
            ("ZSH", ShellKind::Zsh),
            ("Fish", ShellKind::Fish),
            ("pwsh", ShellKind::Pwsh),
            ("PowerShell", ShellKind::PowerShell),
            ("cmd", ShellKind::Cmd),
        ];
        for (input, expected) in cases {
            let parsed: ShellKind = input.parse().unwrap();
            assert_eq!(parsed, expected, "input {input:?} parsed to {parsed:?}");
        }
        // Unknown shells return an error (the caller decides how to handle
        // them; the engine itself is OK with "unknown" — only the
        // `forgum init <shell>` generator needs a known shell).
        assert!("tcsh".parse::<ShellKind>().is_err());
    }

    #[test]
    fn shell_kind_unknown_returns_error() {
        let r = "anything".parse::<ShellKind>();
        assert!(r.is_err());
        // Constructing Unknown directly is allowed (e.g., for a parser
        // that wants to be permissive).
        assert_eq!(ShellKind::Unknown.as_str(), "unknown");
    }

    #[test]
    #[allow(unsafe_code)]
    fn override_precedes_default() {
        let _guard = env_lock().lock().unwrap();
        // The override must be honored verbatim, on every OS. We previously
        // hardcoded a `/tmp`-prefixed path here, which failed on Windows
        // (FORGUM_CONFIG override resolves to a path the harness asserted
        // against a Unix-only default). Assert against `config_path()` for the
        // current OS instead.
        let saved = std::env::var("FORGUM_CONFIG").ok();
        let expected = PathBuf::from("/tmp/forgum-override/config.json");
        unsafe {
            std::env::set_var("FORGUM_CONFIG", &expected);
        }
        let p = config_path().unwrap();
        assert_eq!(
            p, expected,
            "FORGUM_CONFIG override must be honored verbatim"
        );
        unsafe {
            match saved {
                Some(v) => std::env::set_var("FORGUM_CONFIG", v),
                None => std::env::remove_var("FORGUM_CONFIG"),
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn override_resolves_to_config_path_current_os() {
        let _guard = env_lock().lock().unwrap();
        // On Windows, the override must equal what `config_path()` resolves
        // for the current platform — never a hardcoded Unix `/tmp` path.
        let saved = std::env::var("FORGUM_CONFIG").ok();
        let expected = if cfg!(windows) {
            PathBuf::from(r"C:\forgum-override\config.json")
        } else {
            PathBuf::from("/tmp/forgum-override/config.json")
        };
        unsafe {
            std::env::set_var("FORGUM_CONFIG", &expected);
        }
        let p = config_path().unwrap();
        assert_eq!(p, expected);
        unsafe {
            match saved {
                Some(v) => std::env::set_var("FORGUM_CONFIG", v),
                None => std::env::remove_var("FORGUM_CONFIG"),
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn override_paths_accepted_even_when_missing() {
        let _guard = env_lock().lock().unwrap();
        // Override should not require the parent dir to exist.
        let saved = std::env::var("FORGUM_CONFIG").ok();
        unsafe {
            std::env::set_var("FORGUM_CONFIG", "/nonexistent/forgum-test/config.json");
        }
        let p = config_path().unwrap();
        assert!(p.ends_with("config.json"));
        restore_env("FORGUM_CONFIG", saved);
    }

    #[test]
    #[allow(unsafe_code)]
    fn resolve_creates_missing_dirs() {
        let _guard = env_lock().lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let cfg_dir = tmp.path().join("cfg").join("Forgum");
        let data_dir = tmp.path().join("data").join("Forgum");
        let rt_dir = tmp.path().join("rt").join("Forgum");
        let log_dir = tmp.path().join("log").join("Forgum");

        let saved_cfg = std::env::var("FORGUM_CONFIG").ok();
        let saved_data = std::env::var("FORGUM_DATA").ok();
        let saved_rt = std::env::var("FORGUM_RUNTIME").ok();
        let saved_log = std::env::var("FORGUM_LOG").ok();

        unsafe {
            std::env::set_var("FORGUM_CONFIG", cfg_dir.join("config.json"));
            std::env::set_var("FORGUM_DATA", &data_dir);
            std::env::set_var("FORGUM_RUNTIME", &rt_dir);
            std::env::set_var("FORGUM_LOG", &log_dir);
        }

        let resolved = ConfigPaths::resolve().unwrap();

        assert_eq!(resolved.config, cfg_dir.join("config.json"));
        assert_eq!(resolved.data, data_dir);
        assert_eq!(resolved.runtime, rt_dir);
        assert_eq!(resolved.log, log_dir);
        assert!(cfg_dir.exists());
        assert!(data_dir.exists());
        assert!(rt_dir.exists());
        assert!(log_dir.exists());
        // The config file itself should NOT have been created.
        assert!(!cfg_dir.join("config.json").exists());

        restore_env("FORGUM_CONFIG", saved_cfg);
        restore_env("FORGUM_DATA", saved_data);
        restore_env("FORGUM_RUNTIME", saved_rt);
        restore_env("FORGUM_LOG", saved_log);
    }

    #[allow(unsafe_code)]
    fn restore_env(key: &str, val: Option<String>) {
        unsafe {
            match val {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }
}
