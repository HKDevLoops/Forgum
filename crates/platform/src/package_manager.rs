//! Package manager detection, tracking, and update dispatch.
//!
//! Detects how Forgum was installed (Scoop, WinGet, Chocolatey, Homebrew,
//! Pacman, APT, Cargo, or direct standalone binary) and manages update commands.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::PlatformError;

/// Known package managers supported by Forgum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PackageManager {
    Scoop,
    Winget,
    Chocolatey,
    Homebrew,
    Pacman,
    Apt,
    Dnf,
    Zypper,
    Nix,
    Apk,
    Xbps,
    Gentoo,
    MacPorts,
    FreeBsdPkg,
    Cargo,
    DirectBinary,
}

impl PackageManager {
    /// Human-readable display name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Scoop => "Scoop",
            Self::Winget => "WinGet",
            Self::Chocolatey => "Chocolatey",
            Self::Homebrew => "Homebrew",
            Self::Pacman => "Pacman",
            Self::Apt => "APT",
            Self::Dnf => "DNF",
            Self::Zypper => "Zypper",
            Self::Nix => "Nix",
            Self::Apk => "APK",
            Self::Xbps => "XBPS",
            Self::Gentoo => "Gentoo / Portage",
            Self::MacPorts => "MacPorts",
            Self::FreeBsdPkg => "FreeBSD pkg",
            Self::Cargo => "Cargo",
            Self::DirectBinary => "Standalone Binary",
        }
    }

    /// Primary command string to perform an upgrade.
    #[must_use]
    pub const fn update_command(&self) -> &'static str {
        match self {
            Self::Scoop => "scoop update forgum",
            Self::Winget => "winget upgrade HKDevLoops.Forgum",
            Self::Chocolatey => "choco upgrade forgum -y",
            Self::Homebrew => "brew upgrade forgum",
            Self::Pacman => "sudo pacman -S forgum",
            Self::Apt => "sudo apt update && sudo apt install --only-upgrade forgum",
            Self::Dnf => "sudo dnf upgrade forgum",
            Self::Zypper => "sudo zypper update forgum",
            Self::Nix => "nix profile upgrade forgum",
            Self::Apk => "sudo apk upgrade forgum",
            Self::Xbps => "sudo xbps-install -Su forgum",
            Self::Gentoo => "sudo emerge --ask --update app-misc/forgum",
            Self::MacPorts => "sudo port upgrade forgum",
            Self::FreeBsdPkg => "sudo pkg upgrade forgum",
            Self::Cargo => "cargo install --force forgum-cli",
            Self::DirectBinary => {
                "gh release download or https://github.com/HKDevLoops/Forgum/releases/latest"
            }
        }
    }

    /// Primary command string to check for available updates without installing.
    #[must_use]
    pub const fn check_command(&self) -> &'static str {
        match self {
            Self::Scoop => "scoop status forgum",
            Self::Winget => "winget list -q forgum",
            Self::Chocolatey => "choco outdated",
            Self::Homebrew => "brew outdated forgum",
            Self::Pacman => "pacman -Qu forgum",
            Self::Apt => "apt list --upgradable forgum",
            Self::Dnf => "dnf check-update forgum",
            Self::Zypper => "zypper list-updates",
            Self::Nix => "nix profile list",
            Self::Apk => "apk version -v",
            Self::Xbps => "xbps-install -un",
            Self::Gentoo => "emerge -pv app-misc/forgum",
            Self::MacPorts => "port outdated forgum",
            Self::FreeBsdPkg => "pkg version -v",
            Self::Cargo => "cargo search forgum-cli",
            Self::DirectBinary => "https://github.com/HKDevLoops/Forgum/releases/latest",
        }
    }

    /// Primary command string to cleanly uninstall via the package manager.
    #[must_use]
    pub const fn uninstall_command(&self) -> &'static str {
        match self {
            Self::Scoop => "scoop uninstall forgum",
            Self::Winget => "winget uninstall HKDevLoops.Forgum",
            Self::Chocolatey => "choco uninstall forgum -y",
            Self::Homebrew => "brew uninstall forgum",
            Self::Pacman => "sudo pacman -Rns forgum",
            Self::Apt => "sudo apt remove --purge forgum",
            Self::Dnf => "sudo dnf remove forgum",
            Self::Zypper => "sudo zypper remove forgum",
            Self::Nix => "nix profile remove forgum",
            Self::Apk => "sudo apk del forgum",
            Self::Xbps => "sudo xbps-remove forgum",
            Self::Gentoo => "sudo emerge --unmerge app-misc/forgum",
            Self::MacPorts => "sudo port uninstall forgum",
            Self::FreeBsdPkg => "sudo pkg delete forgum",
            Self::Cargo => "cargo uninstall forgum-cli",
            Self::DirectBinary => "forgum uninstall",
        }
    }

    /// Test if this package manager executable is installed and available in PATH.
    #[must_use]
    pub fn is_available_on_host(&self) -> bool {
        match self {
            Self::Scoop => is_cmd_available("scoop"),
            Self::Winget => is_cmd_available("winget"),
            Self::Chocolatey => is_cmd_available("choco"),
            Self::Homebrew => is_cmd_available("brew"),
            Self::Pacman => is_cmd_available("pacman"),
            Self::Apt => is_cmd_available("apt") || is_cmd_available("dpkg"),
            Self::Dnf => is_cmd_available("dnf"),
            Self::Zypper => is_cmd_available("zypper"),
            Self::Nix => is_cmd_available("nix") || is_cmd_available("nix-env"),
            Self::Apk => is_cmd_available("apk"),
            Self::Xbps => is_cmd_available("xbps-install"),
            Self::Gentoo => is_cmd_available("emerge"),
            Self::MacPorts => is_cmd_available("port"),
            Self::FreeBsdPkg => is_cmd_available("pkg"),
            Self::Cargo => is_cmd_available("cargo"),
            Self::DirectBinary => true,
        }
    }
}

/// Helper to test if a CLI tool exists in PATH.
fn is_cmd_available(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        if let Ok(status) = Command::new("where.exe").arg(cmd).output() {
            if status.status.success() {
                return true;
            }
        }
    }
    #[cfg(unix)]
    {
        if let Ok(status) = Command::new("which").arg(cmd).output() {
            if status.status.success() {
                return true;
            }
        }
    }

    // Direct check by invoking --version with timeout/output check
    Command::new(cmd).arg("--version").output().is_ok()
}

/// Detect how the currently executing binary was installed.
#[must_use]
pub fn detect_installation_source() -> PackageManager {
    if let Ok(exe_path) = std::env::current_exe() {
        detect_source_from_path(&exe_path)
    } else {
        PackageManager::DirectBinary
    }
}

/// Detect installation source from an executable path.
#[must_use]
pub fn detect_source_from_path(path: &Path) -> PackageManager {
    let path_str = path.to_string_lossy().to_ascii_lowercase();

    if path_str.contains("scoop") {
        return PackageManager::Scoop;
    }
    if path_str.contains("winget") || path_str.contains("desktopappinstaller") {
        return PackageManager::Winget;
    }
    if path_str.contains("chocolatey")
        || path_str.contains("\\choco\\")
        || path_str.contains("/choco/")
    {
        return PackageManager::Chocolatey;
    }
    if path_str.contains("homebrew")
        || path_str.contains("/cellar/")
        || path_str.contains("linuxbrew")
    {
        return PackageManager::Homebrew;
    }
    if path_str.contains(".cargo")
        || path_str.contains("\\cargo\\bin")
        || path_str.contains("/cargo/bin")
    {
        return PackageManager::Cargo;
    }
    if path_str.contains("/nix/store") || path_str.contains("/nix/var") {
        return PackageManager::Nix;
    }
    if path_str.contains("/opt/local/") || path_str.contains("macports") {
        return PackageManager::MacPorts;
    }

    // Secondary heuristic: if binary is in a system path, check package manager registration
    #[cfg(windows)]
    {
        if PackageManager::Scoop.is_available_on_host() {
            if let Ok(out) = Command::new("scoop").args(["which", "forgum"]).output() {
                if out.status.success() && !out.stdout.is_empty() {
                    return PackageManager::Scoop;
                }
            }
        }
    }

    #[cfg(unix)]
    {
        if PackageManager::Homebrew.is_available_on_host() {
            if let Ok(out) = Command::new("brew").args(["list", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Homebrew;
                }
            }
        }
        if PackageManager::Pacman.is_available_on_host() {
            if let Ok(out) = Command::new("pacman").args(["-Q", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Pacman;
                }
            }
        }
        if PackageManager::Apt.is_available_on_host() {
            if let Ok(out) = Command::new("dpkg").args(["-s", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Apt;
                }
            }
        }
        if PackageManager::Dnf.is_available_on_host() {
            if let Ok(out) = Command::new("rpm").args(["-q", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Dnf;
                }
            }
        }
        if PackageManager::Zypper.is_available_on_host() {
            if let Ok(out) = Command::new("rpm").args(["-q", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Zypper;
                }
            }
        }
        if PackageManager::Apk.is_available_on_host() {
            if let Ok(out) = Command::new("apk").args(["info", "-e", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Apk;
                }
            }
        }
        if PackageManager::Xbps.is_available_on_host() {
            if let Ok(out) = Command::new("xbps-query").args(["forgum"]).output() {
                if out.status.success() {
                    return PackageManager::Xbps;
                }
            }
        }
        if PackageManager::FreeBsdPkg.is_available_on_host() {
            if let Ok(out) = Command::new("pkg").args(["info", "forgum"]).output() {
                if out.status.success() {
                    return PackageManager::FreeBsdPkg;
                }
            }
        }
    }

    PackageManager::DirectBinary
}

/// Return list of relevant package managers for this OS and whether they are active.
#[must_use]
pub fn detect_available_package_managers() -> Vec<(PackageManager, bool)> {
    let mut managers = Vec::new();

    #[cfg(windows)]
    {
        managers.push((
            PackageManager::Scoop,
            PackageManager::Scoop.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Winget,
            PackageManager::Winget.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Chocolatey,
            PackageManager::Chocolatey.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Cargo,
            PackageManager::Cargo.is_available_on_host(),
        ));
    }

    #[cfg(unix)]
    {
        managers.push((
            PackageManager::Homebrew,
            PackageManager::Homebrew.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Pacman,
            PackageManager::Pacman.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Apt,
            PackageManager::Apt.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Dnf,
            PackageManager::Dnf.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Zypper,
            PackageManager::Zypper.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Nix,
            PackageManager::Nix.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Apk,
            PackageManager::Apk.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Xbps,
            PackageManager::Xbps.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Gentoo,
            PackageManager::Gentoo.is_available_on_host(),
        ));
        managers.push((
            PackageManager::MacPorts,
            PackageManager::MacPorts.is_available_on_host(),
        ));
        managers.push((
            PackageManager::FreeBsdPkg,
            PackageManager::FreeBsdPkg.is_available_on_host(),
        ));
        managers.push((
            PackageManager::Cargo,
            PackageManager::Cargo.is_available_on_host(),
        ));
    }

    managers
}

/// Execute an update or update-check for the given package manager.
pub fn execute_package_manager_action(
    pm: PackageManager,
    check_only: bool,
) -> Result<String, String> {
    match pm {
        PackageManager::DirectBinary => {
            let version = env!("CARGO_PKG_VERSION");
            Ok(format!(
                "Forgum v{version} is running as a Standalone Binary.\n\
                 Latest releases and pre-compiled binaries are published at:\n\
                 https://github.com/HKDevLoops/Forgum/releases/latest\n\n\
                 To update automatically via a package manager, install via Scoop or WinGet:\n\
                   scoop bucket add hkdevloops https://github.com/HKDevLoops/scoop-bucket\n\
                   scoop install forgum"
            ))
        }
        _ => {
            let cmd_str = if check_only {
                pm.check_command()
            } else {
                pm.update_command()
            };

            let parts: Vec<&str> = cmd_str.split_whitespace().collect();
            if parts.is_empty() {
                return Err("No command specified".to_string());
            }

            let program = parts[0];
            let args = &parts[1..];

            match Command::new(program).args(args).output() {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    if output.status.success() {
                        let combined = if stderr.is_empty() {
                            stdout
                        } else {
                            format!("{stdout}\n{stderr}")
                        };
                        Ok(combined.trim().to_string())
                    } else {
                        Err(format!(
                            "Command '{}' exited with status {}:\n{}\n{}",
                            cmd_str,
                            output.status,
                            stdout.trim(),
                            stderr.trim()
                        ))
                    }
                }
                Err(e) => Err(format!("Failed to execute '{}': {}", cmd_str, e)),
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RELEASE CHANNELS & RECEIPT TRACKING
// ═══════════════════════════════════════════════════════════════════════════

/// Release channel stream for Forgum installations and updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Nightly,
    Dev,
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        Self::Stable
    }
}

impl ReleaseChannel {
    pub const ALL: &[Self] = &[Self::Stable, Self::Nightly, Self::Dev];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Nightly => "nightly",
            Self::Dev => "dev",
        }
    }

    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Stable => "Stable (main releases)",
            Self::Nightly => "Nightly (bleeding edge)",
            Self::Dev => "Dev (local / git build)",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Official release builds with maximum stability and verified features.",
            Self::Nightly => "Automated continuous releases with latest enhancements and early fixes.",
            Self::Dev => "Locally compiled developer build running from source.",
        }
    }
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReleaseChannel {
    type Err = PlatformError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stable" | "main" | "release" | "prod" => Ok(Self::Stable),
            "nightly" | "edge" | "preview" => Ok(Self::Nightly),
            "dev" | "develop" | "local" => Ok(Self::Dev),
            other => Err(PlatformError::InvalidArgument(format!(
                "invalid release channel '{other}'. Valid channels: stable, nightly, dev"
            ))),
        }
    }
}

/// Installation receipt recording installer source, channel, version, binary path, and timestamp.
/// Persisted in `.config/forgum/install_receipt.json`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Receipt {
    pub installer_source: PackageManager,
    pub channel: ReleaseChannel,
    pub version: String,
    pub bin_path: PathBuf,
    pub timestamp: u64,
}

impl Receipt {
    /// Return the canonical path to the installation receipt file.
    pub fn file_path() -> Result<PathBuf, PlatformError> {
        let dir = crate::paths::config_dir()?;
        Ok(dir.join("install_receipt.json"))
    }

    /// Read the installation receipt from disk if it exists.
    pub fn read() -> Result<Option<Self>, PlatformError> {
        let path = Self::file_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        match serde_json::from_str::<Self>(&content) {
            Ok(receipt) => Ok(Some(receipt)),
            Err(_) => Ok(None),
        }
    }

    /// Write this receipt to disk.
    pub fn write(&self) -> Result<(), PlatformError> {
        let path = Self::file_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| PlatformError::InvalidArgument(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Record an installation receipt for the current execution environment.
    pub fn record(
        channel: ReleaseChannel,
        source_override: Option<PackageManager>,
    ) -> Result<Self, PlatformError> {
        let bin_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("forgum"));
        let source = source_override.unwrap_or_else(|| detect_source_from_path(&bin_path));
        let version = env!("CARGO_PKG_VERSION").to_string();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let receipt = Self {
            installer_source: source,
            channel,
            version,
            bin_path,
            timestamp,
        };
        receipt.write()?;
        Ok(receipt)
    }
}

pub fn read_receipt() -> Result<Option<Receipt>, PlatformError> {
    Receipt::read()
}

pub fn write_receipt(receipt: &Receipt) -> Result<(), PlatformError> {
    receipt.write()
}

pub fn record_receipt(
    channel: ReleaseChannel,
    source: Option<PackageManager>,
) -> Result<Receipt, PlatformError> {
    Receipt::record(channel, source)
}

// ═══════════════════════════════════════════════════════════════════════════
// CROSS-PACKAGE-MANAGER SHADOW DETECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Represents an installation of Forgum detected on the host system.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ShadowInstallation {
    pub path: PathBuf,
    pub source: PackageManager,
    pub is_active: bool,
    pub version: Option<String>,
}

impl ShadowInstallation {
    /// Format a concise single-line description of this installation.
    #[must_use]
    pub fn display_line(&self) -> String {
        let tag = if self.is_active {
            "\x1b[1;32m[Active]\x1b[0m"
        } else {
            "\x1b[1;33m[Shadow]\x1b[0m"
        };
        let ver = self
            .version
            .as_deref()
            .map(|v| format!("v{v}"))
            .unwrap_or_else(|| "unknown version".to_string());
        format!("{tag} {} ({}, {ver})", self.path.display(), self.source.name())
    }
}

/// Helper to strip Windows verbatim `\\?\` prefix if present.
fn strip_verbatim(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(stripped) = s.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

/// Scans PATH and known platform directories to detect all installed Forgum binaries
/// across package managers, identifying the active binary and any shadow installations.
#[must_use]
pub fn detect_shadow_installations() -> Vec<ShadowInstallation> {
    let mut detected: Vec<ShadowInstallation> = Vec::new();
    let mut seen_canonical: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    let current_exe = std::env::current_exe().ok();
    let current_canon = current_exe.as_ref().and_then(|p| std::fs::canonicalize(p).ok());

    let mut candidate_paths: Vec<PathBuf> = Vec::new();

    // 1. Current executing binary takes priority
    if let Some(ref exe) = current_exe {
        candidate_paths.push(exe.clone());
    }

    // 2. Scan PATH environment variable
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            #[cfg(windows)]
            {
                let names = ["forgum.exe", "forgum.cmd", "forgum.bat", "forgum.ps1", "forgum"];
                for name in names {
                    let candidate = dir.join(name);
                    if candidate.is_file() {
                        candidate_paths.push(candidate);
                    }
                }
            }
            #[cfg(unix)]
            {
                let candidate = dir.join("forgum");
                if candidate.is_file() {
                    candidate_paths.push(candidate);
                }
            }
        }
    }

    // 3. Scan standard well-known locations
    #[cfg(windows)]
    {
        if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
            let p = PathBuf::from(&local_app_data).join("Forgum").join("forgum.exe");
            if p.is_file() {
                candidate_paths.push(p);
            }
            let winget_p = PathBuf::from(&local_app_data)
                .join("Microsoft")
                .join("WindowsApps")
                .join("forgum.exe");
            if winget_p.is_file() {
                candidate_paths.push(winget_p);
            }
        }
        if let Some(userprofile) = std::env::var_os("USERPROFILE") {
            let scoop_shim = PathBuf::from(&userprofile)
                .join("scoop")
                .join("shims")
                .join("forgum.exe");
            if scoop_shim.is_file() {
                candidate_paths.push(scoop_shim);
            }
            let scoop_app = PathBuf::from(&userprofile)
                .join("scoop")
                .join("apps")
                .join("forgum")
                .join("current")
                .join("forgum.exe");
            if scoop_app.is_file() {
                candidate_paths.push(scoop_app);
            }
            let cargo_bin = PathBuf::from(&userprofile)
                .join(".cargo")
                .join("bin")
                .join("forgum.exe");
            if cargo_bin.is_file() {
                candidate_paths.push(cargo_bin);
            }
        }
        let choco_bin = PathBuf::from(r"C:\ProgramData\chocolatey\bin\forgum.exe");
        if choco_bin.is_file() {
            candidate_paths.push(choco_bin);
        }
    }

    #[cfg(unix)]
    {
        if let Some(home) = std::env::var_os("HOME") {
            let home_path = PathBuf::from(home);
            let local_bin = home_path.join(".local").join("bin").join("forgum");
            if local_bin.is_file() {
                candidate_paths.push(local_bin);
            }
            let cargo_bin = home_path.join(".cargo").join("bin").join("forgum");
            if cargo_bin.is_file() {
                candidate_paths.push(cargo_bin);
            }
            let nix_profile = home_path.join(".nix-profile").join("bin").join("forgum");
            if nix_profile.is_file() {
                candidate_paths.push(nix_profile);
            }
        }
        let std_paths = [
            "/usr/local/bin/forgum",
            "/usr/bin/forgum",
            "/opt/homebrew/bin/forgum",
            "/usr/local/opt/forgum/bin/forgum",
        ];
        for p_str in std_paths {
            let p = PathBuf::from(p_str);
            if p.is_file() {
                candidate_paths.push(p);
            }
        }
    }

    let mut has_active = false;

    for candidate in candidate_paths {
        let canon = std::fs::canonicalize(&candidate)
            .map(|p| strip_verbatim(&p))
            .unwrap_or_else(|_| strip_verbatim(&candidate));

        if !seen_canonical.insert(canon.clone()) {
            continue;
        }

        let clean_display = strip_verbatim(&candidate);
        let source = detect_source_from_path(&clean_display);

        // Determine if this candidate is the active binary
        let is_active = if let (Some(cur_c), Some(cur_e)) = (&current_canon, &current_exe) {
            let cur_clean_c = strip_verbatim(cur_c);
            let cur_clean_e = strip_verbatim(cur_e);
            canon == cur_clean_c || clean_display == cur_clean_e
        } else {
            false
        };

        if is_active {
            has_active = true;
        }

        let version = if is_active {
            Some(env!("CARGO_PKG_VERSION").to_string())
        } else {
            query_binary_version(&candidate)
        };

        detected.push(ShadowInstallation {
            path: clean_display,
            source,
            is_active,
            version,
        });
    }

    // If current_exe wasn't matched, mark first found binary in PATH as active
    if !has_active && !detected.is_empty() {
        detected[0].is_active = true;
    }

    detected
}

/// Helper to query the version of an external Forgum binary.
fn query_binary_version(path: &Path) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("forgum") {
            if let Some(ver) = trimmed.split_whitespace().nth(1) {
                return Some(ver.to_string());
            }
        }
    }
    None
}

// ═══════════════════════════════════════════════════════════════════════════
// ATOMIC SELF-UPDATE & ROLLBACK
// ═══════════════════════════════════════════════════════════════════════════

/// Safely and atomically replaces `target_exe` with `new_binary_bytes`.
///
/// Implements safe staging, `.bak` backup, and automatic rollback on failure:
/// 1. Writes new bytes to a temporary file in the same directory (`forgum.update.<pid>.tmp`).
/// 2. Renames existing `target_exe` to `target_exe.bak` (works on Windows even while running!).
/// 3. Moves temporary file to `target_exe`.
/// 4. If step 3 fails, rolls back `target_exe.bak` -> `target_exe`.
pub fn atomic_replace_binary(target_exe: &Path, new_binary_bytes: &[u8]) -> Result<(), PlatformError> {
    let parent = target_exe.parent().ok_or_else(|| {
        PlatformError::InvalidArgument(format!(
            "target executable has no parent dir: {}",
            target_exe.display()
        ))
    })?;

    let pid = std::process::id();
    let temp_name = format!("forgum.update.{pid}.tmp");
    let temp_path = parent.join(temp_name);

    let bak_path = target_exe.with_extension("bak");

    // 1. Write the new binary to temporary path
    std::fs::write(&temp_path, new_binary_bytes)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o755);
        let _ = std::fs::set_permissions(&temp_path, perms);
    }

    // 2. If target exists, backup to .bak
    let had_target = target_exe.exists();
    if had_target {
        if bak_path.exists() {
            let _ = std::fs::remove_file(&bak_path);
        }
        if let Err(e) = std::fs::rename(target_exe, &bak_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(PlatformError::Io(e));
        }
    }

    // 3. Move temp file to target_exe
    if let Err(e) = std::fs::rename(&temp_path, target_exe) {
        // Rollback
        if had_target && bak_path.exists() {
            let _ = std::fs::rename(&bak_path, target_exe);
        }
        let _ = std::fs::remove_file(&temp_path);
        return Err(PlatformError::Io(e));
    }

    // 4. Best-effort cleanup of backup
    let _ = std::fs::remove_file(&bak_path);

    Ok(())
}

/// Rollback an update by restoring the `.bak` file if it exists.
pub fn rollback_binary(target_exe: &Path) -> Result<(), PlatformError> {
    let bak_path = target_exe.with_extension("bak");
    if !bak_path.exists() {
        return Err(PlatformError::InvalidArgument(format!(
            "no backup binary found at {}",
            bak_path.display()
        )));
    }
    if target_exe.exists() {
        let _ = std::fs::remove_file(target_exe);
    }
    std::fs::rename(&bak_path, target_exe)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detect_from_known_paths() {
        assert_eq!(
            detect_source_from_path(&PathBuf::from(
                "C:\\Users\\user\\scoop\\apps\\forgum\\current\\forgum.exe"
            )),
            PackageManager::Scoop
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("C:\\Users\\user\\scoop\\shims\\forgum.exe")),
            PackageManager::Scoop
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from(
                "C:\\Program Files\\WindowsApps\\Microsoft.Winget.Source_8wekyb3d8bbwe\\forgum.exe"
            )),
            PackageManager::Winget
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from(
                "C:\\ProgramData\\chocolatey\\bin\\forgum.exe"
            )),
            PackageManager::Chocolatey
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("/opt/homebrew/bin/forgum")),
            PackageManager::Homebrew
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("/home/user/.cargo/bin/forgum")),
            PackageManager::Cargo
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("C:\\Users\\user\\.cargo\\bin\\forgum.exe")),
            PackageManager::Cargo
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("/usr/local/bin/my_standalone_forgum")),
            PackageManager::DirectBinary
        );
    }

    #[test]
    fn package_manager_metadata() {
        assert_eq!(PackageManager::Scoop.name(), "Scoop");
        assert!(PackageManager::Scoop
            .update_command()
            .contains("scoop update"));
        assert!(PackageManager::Winget
            .update_command()
            .contains("winget upgrade"));
        assert!(PackageManager::Dnf.update_command().contains("dnf upgrade"));
        assert!(PackageManager::Nix
            .update_command()
            .contains("nix profile upgrade"));
        assert_eq!(
            PackageManager::Scoop.uninstall_command(),
            "scoop uninstall forgum"
        );
        assert_eq!(
            PackageManager::Homebrew.uninstall_command(),
            "brew uninstall forgum"
        );
        assert_eq!(
            PackageManager::Apt.uninstall_command(),
            "sudo apt remove --purge forgum"
        );
    }

    #[test]
    fn detect_from_nix_and_macports_paths() {
        assert_eq!(
            detect_source_from_path(&PathBuf::from("/nix/store/abc-forgum-0.4.0/bin/forgum")),
            PackageManager::Nix
        );
        assert_eq!(
            detect_source_from_path(&PathBuf::from("/opt/local/bin/forgum")),
            PackageManager::MacPorts
        );
    }

    #[test]
    fn release_channel_parse_and_display() {
        assert_eq!(ReleaseChannel::default(), ReleaseChannel::Stable);
        assert_eq!(ReleaseChannel::Stable.as_str(), "stable");
        assert_eq!(ReleaseChannel::Nightly.as_str(), "nightly");
        assert_eq!(ReleaseChannel::Dev.as_str(), "dev");

        assert_eq!("stable".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Stable);
        assert_eq!("main".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Stable);
        assert_eq!("nightly".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Nightly);
        assert_eq!("preview".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Nightly);
        assert_eq!("dev".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Dev);
        assert_eq!("local".parse::<ReleaseChannel>().unwrap(), ReleaseChannel::Dev);

        assert!("invalid_channel".parse::<ReleaseChannel>().is_err());
    }

    #[test]
    fn receipt_serialization_and_write() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let receipt_file = temp_dir.path().join("install_receipt.json");

        let receipt = Receipt {
            installer_source: PackageManager::DirectBinary,
            channel: ReleaseChannel::Nightly,
            version: "0.4.0".to_string(),
            bin_path: temp_dir.path().join("forgum"),
            timestamp: 1700000000,
        };

        let json = serde_json::to_string_pretty(&receipt).expect("json");
        std::fs::write(&receipt_file, &json).expect("write");

        let content = std::fs::read_to_string(&receipt_file).expect("read");
        let parsed: Receipt = serde_json::from_str(&content).expect("parse");

        assert_eq!(parsed.channel, ReleaseChannel::Nightly);
        assert_eq!(parsed.installer_source, PackageManager::DirectBinary);
        assert_eq!(parsed.version, "0.4.0");
    }

    #[test]
    fn atomic_replace_and_rollback() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let exe_path = temp_dir.path().join("forgum_test.exe");

        // Initial binary write
        std::fs::write(&exe_path, b"ORIGINAL_BINARY_V1").expect("write original");
        assert_eq!(std::fs::read(&exe_path).unwrap(), b"ORIGINAL_BINARY_V1");

        // Atomic replace with V2
        atomic_replace_binary(&exe_path, b"NEW_BINARY_V2").expect("replace");
        assert_eq!(std::fs::read(&exe_path).unwrap(), b"NEW_BINARY_V2");

        // Manually test rollback by creating a .bak and rolling back
        let bak_path = exe_path.with_extension("bak");
        std::fs::write(&bak_path, b"BACKUP_BINARY").expect("write backup");
        rollback_binary(&exe_path).expect("rollback");
        assert_eq!(std::fs::read(&exe_path).unwrap(), b"BACKUP_BINARY");
    }

    #[test]
    fn shadow_detection_runs_cleanly() {
        let shadows = detect_shadow_installations();
        // Should not panic, and returns a list where at least one might be active
        for s in &shadows {
            let line = s.display_line();
            assert!(!line.is_empty());
        }
    }
}
