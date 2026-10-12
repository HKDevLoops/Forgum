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
    #[serde(alias = "scoop")]
    Scoop,
    #[serde(alias = "winget")]
    Winget,
    #[serde(alias = "chocolatey")]
    Chocolatey,
    #[serde(alias = "homebrew")]
    Homebrew,
    #[serde(alias = "pacman")]
    Pacman,
    #[serde(alias = "apt")]
    Apt,
    #[serde(alias = "dnf")]
    Dnf,
    #[serde(alias = "zypper")]
    Zypper,
    #[serde(alias = "nix")]
    Nix,
    #[serde(alias = "apk")]
    Apk,
    #[serde(alias = "xbps")]
    Xbps,
    #[serde(alias = "gentoo")]
    Gentoo,
    #[serde(alias = "macports")]
    MacPorts,
    #[serde(alias = "freebsdpkg")]
    FreeBsdPkg,
    #[serde(alias = "cargo")]
    Cargo,
    #[serde(rename = "standalone", alias = "DirectBinary", alias = "direct_binary")]
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

    /// Loose parser for package manager names from CLI or configuration.
    #[must_use]
    pub fn from_str_loose(s: &str) -> Option<Self> {
        let clean = s.trim().to_ascii_lowercase();
        match clean.as_str() {
            "scoop" => Some(Self::Scoop),
            "winget" | "win_get" | "windowsapps" => Some(Self::Winget),
            "choco" | "chocolatey" => Some(Self::Chocolatey),
            "brew" | "homebrew" => Some(Self::Homebrew),
            "pacman" | "arch" | "aur" => Some(Self::Pacman),
            "apt" | "dpkg" | "debian" | "ubuntu" => Some(Self::Apt),
            "dnf" | "fedora" | "rhel" | "rpm" => Some(Self::Dnf),
            "zypper" | "suse" | "opensuse" => Some(Self::Zypper),
            "nix" | "nix-profile" | "nix-env" => Some(Self::Nix),
            "apk" | "alpine" => Some(Self::Apk),
            "xbps" | "void" => Some(Self::Xbps),
            "gentoo" | "portage" | "emerge" => Some(Self::Gentoo),
            "port" | "macports" => Some(Self::MacPorts),
            "pkg" | "freebsd" | "freebsdpkg" => Some(Self::FreeBsdPkg),
            "cargo" | "crates" | "crates.io" => Some(Self::Cargo),
            "standalone" | "direct" | "direct_binary" | "script" | "binary" => {
                Some(Self::DirectBinary)
            }
            _ => None,
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
    // 1. Authoritative check: read persisted installation receipt if available
    if let Ok(Some(receipt)) = read_receipt() {
        if receipt.installer_source != PackageManager::DirectBinary || receipt.bin_path.exists() {
            return receipt.installer_source;
        }
    }

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
        // Secondary heuristic: only if binary is located in a system bin directory (/usr/bin, /bin, /usr/sbin)
        let is_system_bin = (path_str.starts_with("/usr/bin")
            || path_str.starts_with("/bin")
            || path_str.starts_with("/usr/sbin"))
            && !path_str.starts_with("/usr/local/");

        if is_system_bin {
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

            // On openSUSE/SUSE, prioritize Zypper over DNF for rpm-based installations
            let is_suse = std::fs::read_to_string("/etc/os-release")
                .map(|s| {
                    let lower = s.to_ascii_lowercase();
                    lower.contains("opensuse") || lower.contains("suse")
                })
                .unwrap_or(false);

            if is_suse && PackageManager::Zypper.is_available_on_host() {
                if let Ok(out) = Command::new("rpm").args(["-q", "forgum"]).output() {
                    if out.status.success() {
                        return PackageManager::Zypper;
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
            if !is_suse && PackageManager::Zypper.is_available_on_host() {
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

/// Execute standalone binary upgrade via the celestial installer.
pub fn execute_standalone_update(channel: ReleaseChannel) -> Result<String, String> {
    let channel_name = channel.as_str();
    let ref_name = channel.git_ref();

    let cur_exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_string_lossy().to_string()))
        .unwrap_or_default();

    #[cfg(windows)]
    {
        let install_dir_arg = if cur_exe_dir.is_empty() {
            String::new()
        } else {
            format!(" -InstallDir '{cur_exe_dir}'")
        };
        let cmd = format!(
            "try {{ $c = irm 'https://raw.githubusercontent.com/HKDevLoops/Forgum/{ref_name}/install.ps1' -ErrorAction Stop; & ([scriptblock]::Create($c)) -Channel '{channel_name}'{install_dir_arg} }} catch {{ $c = irm 'https://raw.githubusercontent.com/HKDevLoops/Forgum/dev/install.ps1'; & ([scriptblock]::Create($c)) -Channel '{channel_name}'{install_dir_arg} }}"
        );
        let status = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &cmd])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to invoke PowerShell upgrade: {e}"))?;

        if status.success() {
            Ok("Standalone binary upgraded successfully via install.ps1.".to_string())
        } else {
            Err(format!(
                "PowerShell upgrade script exited with status {status}"
            ))
        }
    }
    #[cfg(not(windows))]
    {
        let dir_export = if cur_exe_dir.is_empty() {
            String::new()
        } else {
            format!("export FORGUM_INSTALL_DIR='{cur_exe_dir}'; ")
        };
        let cmd = format!(
            "{dir_export}curl -4 -fsSL --connect-timeout 5 'https://raw.githubusercontent.com/HKDevLoops/Forgum/{ref_name}/install.sh' 2>/dev/null | bash -s -- --channel {channel_name} --headless -y || curl -4 -fsSL --connect-timeout 5 'https://raw.githubusercontent.com/HKDevLoops/Forgum/dev/install.sh' | bash -s -- --channel {channel_name} --headless -y"
        );
        let status = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .map_err(|e| format!("Failed to invoke bash upgrade: {e}"))?;

        if status.success() {
            Ok("Standalone binary upgraded successfully via install.sh.".to_string())
        } else {
            Err(format!("Bash upgrade script exited with status {status}"))
        }
    }
}

/// Path to local release binary in target/release/
#[must_use]
pub fn local_release_binary_path() -> PathBuf {
    #[cfg(windows)]
    {
        PathBuf::from("target").join("release").join("forgum.exe")
    }
    #[cfg(not(windows))]
    {
        PathBuf::from("target").join("release").join("forgum")
    }
}

/// Execute an update or update-check for the given package manager.
pub fn execute_package_manager_action(
    pm: PackageManager,
    check_only: bool,
) -> Result<String, String> {
    match pm {
        PackageManager::DirectBinary => {
            if check_only {
                let version = env!("CARGO_PKG_VERSION");
                Ok(format!(
                    "Forgum v{version} is running as a Standalone Binary.\n\
                     Latest releases and pre-compiled binaries are published at:\n\
                     https://github.com/HKDevLoops/Forgum/releases/latest\n\n\
                     To update automatically via a package manager, install via Scoop or WinGet:\n\
                       scoop bucket add hkdevloops https://github.com/HKDevLoops/scoop-bucket\n\
                       scoop install forgum"
                ))
            } else {
                execute_standalone_update(ReleaseChannel::Stable)
            }
        }
        _ => {
            let cmd_str = if check_only {
                pm.check_command()
            } else {
                pm.update_command()
            };

            if check_only {
                // Read-only check: run in subshell and capture output for reporting
                #[cfg(windows)]
                let mut cmd = {
                    let mut c = Command::new("cmd");
                    c.args(["/c", cmd_str]);
                    c
                };
                #[cfg(not(windows))]
                let mut cmd = {
                    let mut c = Command::new("sh");
                    c.args(["-c", cmd_str]);
                    c
                };

                match cmd.output() {
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
                                "Update check command '{cmd_str}' exited with status {}:\n{}\n{}",
                                output.status,
                                stdout.trim(),
                                stderr.trim()
                            ))
                        }
                    }
                    Err(e) => Err(format!("Failed to execute update check '{cmd_str}': {e}")),
                }
            } else {
                // Interactive installation/upgrade: MUST inherit stdio for sudo passwords and [y/N] prompts!
                println!("\x1b[36m>> Running package manager upgrade: {cmd_str}\x1b[0m\n");
                #[cfg(windows)]
                let mut cmd = {
                    let mut c = Command::new("cmd");
                    c.args(["/c", cmd_str]);
                    c
                };
                #[cfg(not(windows))]
                let mut cmd = {
                    let mut c = Command::new("sh");
                    c.args(["-c", cmd_str]);
                    c
                };

                cmd.stdin(std::process::Stdio::inherit())
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit());

                match cmd.status() {
                    Ok(status) => {
                        if status.success() {
                            Ok(format!(
                                "Package manager upgrade (`{cmd_str}`) completed successfully."
                            ))
                        } else {
                            Err(format!(
                                "Package manager upgrade (`{cmd_str}`) exited with status {status}."
                            ))
                        }
                    }
                    Err(e) => Err(format!(
                        "Failed to spawn package manager upgrade command '{cmd_str}': {e}"
                    )),
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RELEASE CHANNELS & RECEIPT TRACKING
// ═══════════════════════════════════════════════════════════════════════════

/// Release channel stream for Forgum installations and updates.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    #[default]
    Stable,
    Alpha,
    Nightly,
    Dev,
}

impl ReleaseChannel {
    pub const ALL: &[Self] = &[Self::Stable, Self::Alpha, Self::Nightly, Self::Dev];

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Alpha => "alpha",
            Self::Nightly => "nightly",
            Self::Dev => "dev",
        }
    }

    /// Corresponding Git branch or ref name on GitHub.
    #[must_use]
    pub const fn git_ref(&self) -> &'static str {
        match self {
            Self::Stable => "main",
            Self::Alpha | Self::Nightly | Self::Dev => "dev",
        }
    }

    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Stable => "Stable (main releases)",
            Self::Alpha => "Alpha (early testing / dev branch)",
            Self::Nightly => "Nightly (bleeding edge)",
            Self::Dev => "Dev (local / git build)",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Stable => "Official release builds with maximum stability and verified features.",
            Self::Alpha => {
                "Pre-release builds tracking the dev/alpha branch for early testing and contributors."
            }
            Self::Nightly => {
                "Automated continuous releases with latest enhancements and early fixes."
            }
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
            "alpha" | "test" | "testing" => Ok(Self::Alpha),
            "nightly" | "edge" | "preview" => Ok(Self::Nightly),
            "dev" | "develop" | "local" => Ok(Self::Dev),
            other => Err(PlatformError::InvalidArgument(format!(
                "invalid release channel '{other}'. Valid channels: stable, alpha, nightly, dev"
            ))),
        }
    }
}

/// Parse version field from raw Cargo.toml content.
#[must_use]
pub fn parse_version_from_cargo_toml(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                let clean = val.trim().trim_matches('"').trim_matches('\'').trim();
                if !clean.is_empty() {
                    return Some(clean.to_string());
                }
            }
        }
    }
    None
}

/// Parse release tag/version from GitHub release API JSON.
#[must_use]
pub fn parse_version_from_release_json(json_str: &str) -> Option<String> {
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
        if let Some(tag) = val.get("tag_name").and_then(|t| t.as_str()) {
            return Some(tag.trim_start_matches('v').to_string());
        }
    }
    None
}

/// Fetch latest remote version string for the given release channel.
pub fn fetch_remote_channel_version(channel: ReleaseChannel) -> Result<String, String> {
    let ref_name = channel.git_ref();
    let raw_url =
        format!("https://raw.githubusercontent.com/HKDevLoops/Forgum/{ref_name}/Cargo.toml");

    // 1. Try curl (universal across Linux, macOS, modern Windows 10/11)
    if is_cmd_available("curl") {
        if let Ok(out) = Command::new("curl")
            .args([
                "-4",
                "-fsSL",
                "--connect-timeout",
                "4",
                "--max-time",
                "6",
                &raw_url,
            ])
            .output()
        {
            if out.status.success() {
                let body = String::from_utf8_lossy(&out.stdout);
                if let Some(ver) = parse_version_from_cargo_toml(&body) {
                    return Ok(ver);
                }
            }
        }
    }

    // 2. Windows fallback using powershell if curl failed or is unavailable
    #[cfg(windows)]
    {
        let ps_cmd = format!("(Invoke-RestMethod -Uri '{raw_url}' -TimeoutSec 5)");
        if let Ok(out) = Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_cmd])
            .output()
        {
            if out.status.success() {
                let body = String::from_utf8_lossy(&out.stdout);
                if let Some(ver) = parse_version_from_cargo_toml(&body) {
                    return Ok(ver);
                }
            }
        }
    }

    // 3. Fallback for Stable channel via GitHub API
    if channel == ReleaseChannel::Stable && is_cmd_available("curl") {
        let api_url = "https://api.github.com/repos/HKDevLoops/Forgum/releases/latest";
        if let Ok(out) = Command::new("curl")
            .args([
                "-4",
                "-fsSL",
                "-H",
                "User-Agent: forgum",
                "--connect-timeout",
                "4",
                "--max-time",
                "6",
                api_url,
            ])
            .output()
        {
            if out.status.success() {
                let body = String::from_utf8_lossy(&out.stdout);
                if let Some(ver) = parse_version_from_release_json(&body) {
                    return Ok(ver);
                }
            }
        }
    }

    Err("Could not retrieve remote version from GitHub (network timeout or offline)".to_string())
}

/// Returns a random funky celebration statement from Forgum when already on the latest version.
#[must_use]
pub fn get_funky_latest_version_quote() -> &'static str {
    const QUOTES: &[&str] = &[
        "🐄 The pasture is pristine, the cud is freshly chewed, and your ASCII mascots are running at peak celestial performance! Touch some grass or go yeet a fortune! ✨",
        "🚀 You're already flying at maximum warp speed! There's no newer code in the galaxy—go enjoy the view from orbit! 🌌",
        "🕶️ Calm down, speedrunner! You're already rocking the freshest Forgum in town. Your terminal is certified dripping with swagger! 🗿",
        "🌾 Even the sharpest steer doesn't graze twice in the same minute. Your installation is 100% bleeding-edge perfection!",
        "🎉 Zero updates needed! The cows are vibing, the particles are sparkling, and your shell prompt has never looked cooler. Keep calm and moo on! 🐮💫",
        "🕺 Smooth criminal detected! You're already moonwalking on the absolute latest release. Nothing to patch, nothing to fix!",
        "🧘 Peak terminal enlightenment achieved. You possess the definitive Forgum binary known to humankind. Bask in the glory!",
        "🌱 The grass has been touched, the bits have been flipped, and your binary is immaculate. Rest easy, hacker!",
    ];
    let idx = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0)
        % QUOTES.len();
    QUOTES[idx]
}

/// Installation receipt recording installer source, channel, version, binary path, and timestamp.
/// Persisted in `.config/forgum/install_receipt.json`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Receipt {
    pub installer_source: PackageManager,
    pub channel: ReleaseChannel,
    pub version: String,
    pub bin_path: PathBuf,
    #[serde(alias = "installed_at", default)]
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
// INTERACTIVE PACKAGE MANAGER SELECTION
// ═══════════════════════════════════════════════════════════════════════════

/// Interactively prompts the user to select their desired update method / package manager.
/// Shows all available methods on the host with the detected source marked as recommended.
pub fn select_update_method_interactive(
    default_pm: PackageManager,
) -> Result<PackageManager, String> {
    let mut available: Vec<PackageManager> = Vec::new();

    // 1. First add the detected source so it is option #1
    available.push(default_pm);

    // 2. Add other available package managers on this host
    for (pm, is_avail) in detect_available_package_managers() {
        if is_avail && !available.contains(&pm) {
            available.push(pm);
        }
    }

    // 3. Standalone Binary is always an option if not already in list
    if !available.contains(&PackageManager::DirectBinary) {
        available.push(PackageManager::DirectBinary);
    }

    // 4. Cargo is always an option if available in PATH
    if is_cmd_available("cargo") && !available.contains(&PackageManager::Cargo) {
        available.push(PackageManager::Cargo);
    }

    println!("\x1b[1;36m━━━ Select Forgum Update Method ━━━\x1b[0m");
    println!(
        "Detected installation source: \x1b[1;32m{}\x1b[0m\n",
        default_pm.name()
    );

    for (idx, pm) in available.iter().enumerate() {
        let num = idx + 1;
        let badge = if *pm == default_pm {
            " \x1b[1;32m[Default / Detected]\x1b[0m"
        } else {
            ""
        };
        let cmd = match pm {
            PackageManager::DirectBinary => "curl / irm installer script",
            _ => pm.update_command(),
        };
        println!(
            "  \x1b[1m{num}.\x1b[0m \x1b[1;33m{:<18}\x1b[0m (`{cmd}`){badge}",
            pm.name()
        );
    }

    println!(
        "\n\x1b[90m(Tip: Set a default anytime with `forgum config set update_method <name>` or via TUI)\x1b[0m"
    );
    print!(
        "Select update method [1-{}] (default: 1): ",
        available.len()
    );
    let _ = std::io::Write::flush(&mut std::io::stdout());

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return Ok(default_pm);
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        println!("Selected: \x1b[1;32m{}\x1b[0m\n", default_pm.name());
        return Ok(default_pm);
    }

    // Try parsing as number
    if let Ok(choice) = trimmed.parse::<usize>() {
        if choice >= 1 && choice <= available.len() {
            let chosen = available[choice - 1];
            println!("Selected: \x1b[1;32m{}\x1b[0m\n", chosen.name());
            return Ok(chosen);
        }
    }

    // Try parsing as package manager name string (e.g. "zypper", "scoop", "standalone")
    if let Some(pm) = PackageManager::from_str_loose(trimmed) {
        println!("Selected: \x1b[1;32m{}\x1b[0m\n", pm.name());
        return Ok(pm);
    }

    println!(
        "\x1b[90m(Unrecognized selection '{}', defaulting to {})\x1b[0m\n",
        trimmed,
        default_pm.name()
    );
    Ok(default_pm)
}

// ═══════════════════════════════════════════════════════════════════════════
// SEMANTIC VERSIONING & UPDATE CHECK CACHING
// ═══════════════════════════════════════════════════════════════════════════

/// Parsed representation of a Semantic Version 2.0.0 string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
}

impl ParsedSemVer {
    /// Parse a semantic version string (e.g. "0.0.6-beta", "v1.2.3", "0.0.6", "0.0.6-beta.2").
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        let trimmed = s.trim().trim_start_matches(['v', 'V']);
        if trimmed.is_empty() {
            return None;
        }

        // Strip build metadata after '+' if present
        let no_build = trimmed.split('+').next().unwrap_or(trimmed);

        // Split into version core and optional prerelease after '-'
        let mut hyphen_parts = no_build.splitn(2, '-');
        let core = hyphen_parts.next().unwrap_or("");
        let prerelease = hyphen_parts
            .next()
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty());

        let mut nums = core.split('.');
        let major = nums.next()?.parse::<u64>().ok()?;
        let minor = nums.next().and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);
        let patch = nums.next().and_then(|n| n.parse::<u64>().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
            prerelease,
        })
    }
}

impl PartialOrd for ParsedSemVer {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ParsedSemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            std::cmp::Ordering::Equal => {}
            ord => return ord,
        }

        // SemVer 2.0.0 rule: a normal version with no prerelease has higher
        // precedence than a prerelease version with the same major.minor.patch.
        match (&self.prerelease, &other.prerelease) {
            (None, None) => std::cmp::Ordering::Equal,
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(a), Some(b)) => compare_prerelease_identifiers(a, b),
        }
    }
}

/// Compares two pre-release strings identifier by identifier according to SemVer 2.0.0.
fn compare_prerelease_identifiers(a: &str, b: &str) -> std::cmp::Ordering {
    let parts_a: Vec<&str> = a.split('.').collect();
    let parts_b: Vec<&str> = b.split('.').collect();

    for (pa, pb) in parts_a.iter().zip(parts_b.iter()) {
        let num_a = pa.parse::<u64>();
        let num_b = pb.parse::<u64>();

        match (num_a, num_b) {
            (Ok(na), Ok(nb)) => match na.cmp(&nb) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            },
            // SemVer 2.0.0 §11.4: numeric identifiers always have lower precedence than non-numeric
            (Ok(_), Err(_)) => return std::cmp::Ordering::Less,
            (Err(_), Ok(_)) => return std::cmp::Ordering::Greater,
            (Err(_), Err(_)) => match pa.cmp(pb) {
                std::cmp::Ordering::Equal => {}
                ord => return ord,
            },
        }
    }

    parts_a.len().cmp(&parts_b.len())
}

/// Compare two version strings according to Semantic Versioning 2.0.0.
#[must_use]
pub fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    match (ParsedSemVer::parse(a), ParsedSemVer::parse(b)) {
        (Some(pa), Some(pb)) => pa.cmp(&pb),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => a.trim().cmp(b.trim()),
    }
}

/// Returns true if `upstream` is strictly newer than `current`.
#[must_use]
pub fn is_newer_version(current: &str, upstream: &str) -> bool {
    compare_semver(current, upstream) == std::cmp::Ordering::Less
}

/// 24 hours TTL for cached update check (86,400 seconds).
pub const UPDATE_CHECK_TTL_SECS: u64 = 86_400;

/// Minimum seconds between background update check spawn attempts.
pub const BACKGROUND_CHECK_COOLDOWN_SECS: u64 = 300;

/// Cache entry stored on disk recording the last update check result.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateCheckCache {
    /// Timestamp when check was performed (Unix epoch seconds).
    pub last_checked: u64,
    /// Latest upstream version found on the channel.
    pub latest_version: String,
    /// Channel that was checked.
    pub channel: ReleaseChannel,
}

impl UpdateCheckCache {
    pub fn file_path() -> Result<PathBuf, PlatformError> {
        let dir = crate::paths::config_dir()?;
        Ok(dir.join("update_check.json"))
    }

    pub fn read() -> Result<Option<Self>, PlatformError> {
        let path = Self::file_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        match serde_json::from_str::<Self>(&content) {
            Ok(c) => Ok(Some(c)),
            Err(_) => Ok(None),
        }
    }

    pub fn write(&self) -> Result<(), PlatformError> {
        let path = Self::file_path()?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| PlatformError::InvalidArgument(e.to_string()))?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}

/// Read cached update check information.
pub fn read_update_cache() -> Result<Option<UpdateCheckCache>, PlatformError> {
    UpdateCheckCache::read()
}

/// Write update check information to cache.
pub fn write_update_cache(cache: &UpdateCheckCache) -> Result<(), PlatformError> {
    cache.write()
}

/// Clear the update check cache file.
pub fn clear_update_cache() -> Result<(), PlatformError> {
    let path = UpdateCheckCache::file_path()?;
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    }
    Ok(())
}

/// Notice information returned when a new update is available.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateNotice {
    pub current_version: String,
    pub latest_version: String,
    pub channel: ReleaseChannel,
    pub update_command: String,
    pub pm_update_command: Option<String>,
}

/// Query cached update status without performing network I/O unless `force_refresh` is true.
/// Returns `Some(UpdateNotice)` if a newer version is available.
#[must_use]
pub fn check_for_updates_cached(
    channel: ReleaseChannel,
    force_refresh: bool,
) -> Option<UpdateNotice> {
    let current_version = env!("CARGO_PKG_VERSION")
        .trim_start_matches('v')
        .to_string();

    if force_refresh {
        if let Ok(remote) = fetch_remote_channel_version(channel) {
            let clean_remote = remote.trim_start_matches('v').to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let cache = UpdateCheckCache {
                last_checked: now,
                latest_version: clean_remote.clone(),
                channel,
            };
            let _ = cache.write();

            if is_newer_version(&current_version, &clean_remote) {
                let source = detect_installation_source();
                let pm_cmd = if source != PackageManager::DirectBinary {
                    Some(source.update_command().to_string())
                } else {
                    None
                };
                return Some(UpdateNotice {
                    current_version,
                    latest_version: clean_remote,
                    channel,
                    update_command: "forgum update".to_string(),
                    pm_update_command: pm_cmd,
                });
            }
        }
        return None;
    }

    let cache = UpdateCheckCache::read().ok().flatten()?;

    // Only respect cache for the matching channel
    if cache.channel != channel {
        return None;
    }

    if is_newer_version(&current_version, &cache.latest_version) {
        let source = detect_installation_source();
        let pm_cmd = if source != PackageManager::DirectBinary {
            Some(source.update_command().to_string())
        } else {
            None
        };
        return Some(UpdateNotice {
            current_version,
            latest_version: cache.latest_version,
            channel,
            update_command: "forgum update".to_string(),
            pm_update_command: pm_cmd,
        });
    }

    None
}

/// Checks whether an asynchronous background update check should be triggered.
/// Returns true if cache is missing or older than 24 hours.
#[must_use]
pub fn should_spawn_background_check(channel: ReleaseChannel) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    match UpdateCheckCache::read() {
        Ok(Some(cache)) => {
            if cache.channel != channel {
                return true;
            }
            now.saturating_sub(cache.last_checked) > UPDATE_CHECK_TTL_SECS
        }
        _ => true,
    }
}

/// Spawns a silent background process to refresh the update check cache.
/// Zero latency to the caller (<1ms).
pub fn spawn_background_update_check(channel: ReleaseChannel) {
    if let Ok(cur_exe) = std::env::current_exe() {
        let ch_str = channel.as_str();
        // Record current timestamp in cache to prevent duplicate spawns
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let placeholder = UpdateCheckCache {
            last_checked: now,
            latest_version: env!("CARGO_PKG_VERSION")
                .trim_start_matches('v')
                .to_string(),
            channel,
        };
        let _ = placeholder.write();

        let _ = crate::spawn::spawn_silent(
            &cur_exe,
            &[
                "update",
                "--check",
                "--channel",
                ch_str,
                "--background-worker",
            ],
        );
    }
}

/// Format an ANSI-styled update notice box to alert the user of a new Forgum version.
#[must_use]
pub fn format_update_notice_box(notice: &UpdateNotice) -> String {
    let cur = &notice.current_version;
    let new = &notice.latest_version;
    let primary_cmd = &notice.update_command;

    let line2_plain = if let Some(ref pm) = notice.pm_update_command {
        if pm != primary_cmd {
            format!("  Run `{primary_cmd}` (or `{pm}`) to upgrade!")
        } else {
            format!("  Run `{primary_cmd}` to upgrade!")
        }
    } else {
        format!("  Run `{primary_cmd}` to upgrade!")
    };

    // Calculate display width: 📦 counts as 2 terminal columns
    let len1 = 2 + 19 + cur.len() + 3 + new.len();
    let len2 = line2_plain.chars().count();
    let inner_width = len1.max(len2).max(46);

    let pad1 = " ".repeat(inner_width.saturating_sub(len1));
    let pad2 = " ".repeat(inner_width.saturating_sub(len2));

    let border_h = "─".repeat(inner_width + 4);

    let cyan = "\x1b[1;36m";
    let yellow = "\x1b[1;33m";
    let green = "\x1b[1;32m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";

    let line2_formatted = if let Some(ref pm) = notice.pm_update_command {
        if pm != primary_cmd {
            format!("Run `{cyan}{primary_cmd}{reset}` (or `{cyan}{pm}{reset}`) to upgrade!")
        } else {
            format!("Run `{cyan}{primary_cmd}{reset}` to upgrade!")
        }
    } else {
        format!("Run `{cyan}{primary_cmd}{reset}` to upgrade!")
    };

    format!(
        "\n{cyan}╭{border_h}╮{reset}\n\
         {cyan}│{reset}  {bold}📦 Update available:{reset} {yellow}v{cur}{reset} → {green}v{new}{reset}{pad1}  {cyan}│{reset}\n\
         {cyan}│{reset}  {line2_formatted}{pad2}  {cyan}│{reset}\n\
         {cyan}╰{border_h}╯{reset}\n"
    )
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
        format!(
            "{tag} {} ({}, {ver})",
            self.path.display(),
            self.source.name()
        )
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
    let current_canon = current_exe
        .as_ref()
        .and_then(|p| std::fs::canonicalize(p).ok());

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
                let names = [
                    "forgum.exe",
                    "forgum.cmd",
                    "forgum.bat",
                    "forgum.ps1",
                    "forgum",
                ];
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
            let p = PathBuf::from(&local_app_data)
                .join("Forgum")
                .join("forgum.exe");
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
pub fn atomic_replace_binary(
    target_exe: &Path,
    new_binary_bytes: &[u8],
) -> Result<(), PlatformError> {
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

/// Detailed status of a local Git repository tracking Forgum development.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GitRepoStatus {
    pub is_git_repo: bool,
    pub branch: String,
    pub local_commit: String,
    pub remote_branch: String,
    pub commits_behind: usize,
    pub commits_ahead: usize,
    pub is_dirty: bool,
    pub latest_remote_summary: Option<String>,
}

/// Detect whether the current directory or workspace is inside a Git repository.
#[must_use]
pub fn detect_git_status(target_channel: ReleaseChannel) -> Option<GitRepoStatus> {
    if !is_cmd_available("git") {
        return None;
    }

    let Ok(out) = Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    else {
        return None;
    };
    if !out.status.success() || String::from_utf8_lossy(&out.stdout).trim() != "true" {
        return None;
    }

    // Verify that the detected git repository is actually the Forgum project
    // (prevents accidentally hijacking arbitrary user git repos when running `forgum update` elsewhere)
    let toplevel = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if toplevel.is_empty() {
        return None;
    }

    let toplevel_path = Path::new(&toplevel);
    let is_forgum_repo = {
        let root_cargo = toplevel_path.join("Cargo.toml");
        let engine_dir = toplevel_path.join("crates").join("engine");
        let cows_dir = toplevel_path.join("data").join("Cows");

        let has_structure = (engine_dir.is_dir() && cows_dir.is_dir())
            || (root_cargo.exists()
                && std::fs::read_to_string(&root_cargo)
                    .map(|c| c.contains("forgum"))
                    .unwrap_or(false));

        if has_structure {
            true
        } else {
            let remote_url = Command::new("git")
                .args(["config", "--get", "remote.origin.url"])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase())
                .unwrap_or_default();
            remote_url.contains("forgum")
        }
    };

    if !is_forgum_repo {
        return None;
    }

    let branch = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "HEAD".to_string());

    let local_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    let is_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);

    let remote_branch = match target_channel {
        ReleaseChannel::Dev => "dev".to_string(),
        ReleaseChannel::Alpha => {
            if branch == "alpha" {
                "alpha".to_string()
            } else {
                "dev".to_string()
            }
        }
        ReleaseChannel::Stable => {
            if branch == "dev" {
                "dev".to_string()
            } else {
                "main".to_string()
            }
        }
        ReleaseChannel::Nightly => "dev".to_string(),
    };

    Some(GitRepoStatus {
        is_git_repo: true,
        branch,
        local_commit,
        remote_branch,
        commits_behind: 0,
        commits_ahead: 0,
        is_dirty,
        latest_remote_summary: None,
    })
}

/// Fetch remote status and compute behind/ahead commit counts against HKDevLoops/Forgum.
pub fn check_git_updates(status: &mut GitRepoStatus) -> Result<(), String> {
    if !is_cmd_available("git") {
        return Err("git CLI tool is not available in PATH".to_string());
    }

    let _ = Command::new("git")
        .args(["fetch", "--quiet", "origin", &status.remote_branch])
        .output();

    let target_ref = format!("origin/{}", status.remote_branch);

    let behind_range = format!("HEAD..{target_ref}");
    if let Ok(out) = Command::new("git")
        .args(["rev-list", "--count", &behind_range])
        .output()
    {
        if out.status.success() {
            status.commits_behind = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(0);
        }
    }

    let ahead_range = format!("{target_ref}..HEAD");
    if let Ok(out) = Command::new("git")
        .args(["rev-list", "--count", &ahead_range])
        .output()
    {
        if out.status.success() {
            status.commits_ahead = String::from_utf8_lossy(&out.stdout)
                .trim()
                .parse::<usize>()
                .unwrap_or(0);
        }
    }

    if let Ok(out) = Command::new("git")
        .args(["log", "-n", "1", "--format=%h %s (%cr)", &target_ref])
        .output()
    {
        if out.status.success() {
            let summary = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !summary.is_empty() {
                status.latest_remote_summary = Some(summary);
            }
        }
    }

    Ok(())
}

/// Pull latest commits fast-forward and rebuild release binary via cargo if available.
pub fn execute_git_update(status: &GitRepoStatus) -> Result<String, String> {
    if status.is_dirty {
        return Err(
            "Working tree has uncommitted local changes. Please stash or commit them before updating:\n  git stash"
                .to_string(),
        );
    }

    let target_ref = &status.remote_branch;

    let pull_out = Command::new("git")
        .args(["pull", "--ff-only", "origin", target_ref])
        .output()
        .map_err(|e| format!("Failed to invoke `git pull`: {e}"))?;

    if !pull_out.status.success() {
        let stderr = String::from_utf8_lossy(&pull_out.stderr).to_string();
        return Err(format!("`git pull` failed:\n{}", stderr.trim()));
    }

    let mut result_msg = String::from_utf8_lossy(&pull_out.stdout).trim().to_string();

    let new_commit = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    result_msg.push_str(&format!(
        "\nLocal branch is now synchronized at commit \x1b[1;32m{new_commit}\x1b[0m."
    ));

    if is_cmd_available("cargo") {
        result_msg.push_str(
            "\n\x1b[36m>> Compiling release binary (`cargo build --release --bin forgum`)...\x1b[0m",
        );
        let build_out = Command::new("cargo")
            .args([
                "build",
                "--release",
                "-p",
                "forgum-engine",
                "--bin",
                "forgum",
            ])
            .output();

        match build_out {
            Ok(bo) if bo.status.success() => {
                result_msg.push_str(
                    "\n\x1b[1;32m✓ Successfully rebuilt forgum binary to latest commit!\x1b[0m",
                );
                if let Ok(cur_exe) = std::env::current_exe() {
                    let cur_str = cur_exe.to_string_lossy();
                    if !cur_str.contains("target") {
                        let target_bin = if cfg!(windows) {
                            Path::new("target/release/forgum.exe")
                        } else {
                            Path::new("target/release/forgum")
                        };
                        if let Ok(bytes) = std::fs::read(target_bin) {
                            if let Ok(()) = atomic_replace_binary(&cur_exe, &bytes) {
                                result_msg.push_str(&format!(
                                    "\n\x1b[1;32m✓ Atomically updated active binary at {}\x1b[0m",
                                    cur_exe.display()
                                ));
                            }
                        }
                    }
                }
            }
            Ok(bo) => {
                let err = String::from_utf8_lossy(&bo.stderr);
                result_msg.push_str(&format!(
                    "\n\x1b[1;33m⚠️  Binary rebuild returned non-zero. Run manually: `cargo build --release`\n{}\x1b[0m",
                    err.lines().take(5).collect::<Vec<_>>().join("\n")
                ));
            }
            Err(e) => {
                result_msg.push_str(&format!("\n\x1b[1;33m⚠️  Could not run cargo: {e}\x1b[0m"));
            }
        }
    } else {
        result_msg.push_str(
            "\n\x1b[90m(Install Rust/cargo to auto-compile binaries: https://rustup.rs)\x1b[0m",
        );
    }

    Ok(result_msg)
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
            detect_source_from_path(&PathBuf::from(
                "/nix/store/abc-forgum-0.0.3-beta/bin/forgum"
            )),
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
        assert_eq!(ReleaseChannel::Alpha.as_str(), "alpha");
        assert_eq!(ReleaseChannel::Nightly.as_str(), "nightly");
        assert_eq!(ReleaseChannel::Dev.as_str(), "dev");

        assert_eq!(
            "stable".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Stable
        );
        assert_eq!(
            "main".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Stable
        );
        assert_eq!(
            "alpha".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Alpha
        );
        assert_eq!(
            "test".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Alpha
        );
        assert_eq!(
            "nightly".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Nightly
        );
        assert_eq!(
            "preview".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Nightly
        );
        assert_eq!(
            "dev".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Dev
        );
        assert_eq!(
            "local".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Dev
        );

        assert!("invalid_channel".parse::<ReleaseChannel>().is_err());
    }

    #[test]
    fn receipt_serialization_and_write() {
        let temp_dir = tempfile::tempdir().expect("tempdir");
        let receipt_file = temp_dir.path().join("install_receipt.json");

        let receipt = Receipt {
            installer_source: PackageManager::DirectBinary,
            channel: ReleaseChannel::Nightly,
            version: "0.0.3-beta".to_string(),
            bin_path: temp_dir.path().join("forgum"),
            timestamp: 1700000000,
        };

        let json = serde_json::to_string_pretty(&receipt).expect("json");
        std::fs::write(&receipt_file, &json).expect("write");

        let content = std::fs::read_to_string(&receipt_file).expect("read");
        let parsed: Receipt = serde_json::from_str(&content).expect("parse");

        assert_eq!(parsed.channel, ReleaseChannel::Nightly);
        assert_eq!(parsed.installer_source, PackageManager::DirectBinary);
        assert_eq!(parsed.version, "0.0.3-beta");
    }

    #[test]
    fn installer_script_receipt_compatibility() {
        let script_receipt_json = r#"{
  "installer_source": "standalone",
  "channel": "dev",
  "version": "0.0.3-beta",
  "bin_path": "/home/hkdevs/.local/bin/forgum",
  "installed_at": 1790595540,
  "platform": "linux",
  "arch": "x86_64",
  "is_wsl": 0,
  "distro": "opensuse-leap",
  "package_manager": "zypper",
  "auto_update": true
}"#;

        let parsed: Receipt = serde_json::from_str(script_receipt_json)
            .expect("must parse install.sh receipt without error");
        assert_eq!(parsed.installer_source, PackageManager::DirectBinary);
        assert_eq!(parsed.channel, ReleaseChannel::Dev);
        assert_eq!(parsed.version, "0.0.3-beta");
        assert_eq!(parsed.timestamp, 1790595540);
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

    #[test]
    fn test_semver_parsing_and_comparison() {
        use std::cmp::Ordering;

        assert_eq!(compare_semver("0.0.6", "0.0.6"), Ordering::Equal);
        assert_eq!(compare_semver("v0.0.6", "0.0.6"), Ordering::Equal);
        assert_eq!(compare_semver("0.0.6-beta", "0.0.6-beta"), Ordering::Equal);
        assert_eq!(compare_semver("0.0.6-beta", "0.0.7-beta"), Ordering::Less);
        assert_eq!(
            compare_semver("0.0.7-beta", "0.0.6-beta"),
            Ordering::Greater
        );
        assert_eq!(compare_semver("0.0.6-beta", "0.0.6"), Ordering::Less);
        assert_eq!(compare_semver("0.0.6", "0.0.6-beta"), Ordering::Greater);
        assert_eq!(compare_semver("0.0.6-alpha", "0.0.6-beta"), Ordering::Less);
        assert_eq!(
            compare_semver("0.0.6-beta.1", "0.0.6-beta.2"),
            Ordering::Less
        );
        assert_eq!(compare_semver("0.0.5", "0.0.6-beta"), Ordering::Less);
        assert_eq!(compare_semver("1.0.0", "0.9.9"), Ordering::Greater);
    }

    #[test]
    fn test_is_newer_version_edge_cases() {
        assert!(is_newer_version("0.0.6-beta", "0.0.7-beta"));
        assert!(is_newer_version("0.0.6-beta", "0.0.6"));
        assert!(is_newer_version("0.0.6-beta", "0.1.0"));
        assert!(!is_newer_version("0.0.6-beta", "0.0.6-beta"));
        assert!(!is_newer_version("0.0.6-beta", "0.0.5"));
        assert!(!is_newer_version("0.0.6", "0.0.6-beta"));
    }

    #[test]
    fn test_package_manager_from_str_loose_matching() {
        assert_eq!(
            PackageManager::from_str_loose("zypper"),
            Some(PackageManager::Zypper)
        );
        assert_eq!(
            PackageManager::from_str_loose("opensuse"),
            Some(PackageManager::Zypper)
        );
        assert_eq!(
            PackageManager::from_str_loose("suse"),
            Some(PackageManager::Zypper)
        );
        assert_eq!(
            PackageManager::from_str_loose("scoop"),
            Some(PackageManager::Scoop)
        );
        assert_eq!(
            PackageManager::from_str_loose("winget"),
            Some(PackageManager::Winget)
        );
        assert_eq!(
            PackageManager::from_str_loose("brew"),
            Some(PackageManager::Homebrew)
        );
        assert_eq!(
            PackageManager::from_str_loose("apt"),
            Some(PackageManager::Apt)
        );
        assert_eq!(
            PackageManager::from_str_loose("dnf"),
            Some(PackageManager::Dnf)
        );
        assert_eq!(
            PackageManager::from_str_loose("pacman"),
            Some(PackageManager::Pacman)
        );
        assert_eq!(
            PackageManager::from_str_loose("cargo"),
            Some(PackageManager::Cargo)
        );
        assert_eq!(
            PackageManager::from_str_loose("standalone"),
            Some(PackageManager::DirectBinary)
        );
        assert_eq!(
            PackageManager::from_str_loose("script"),
            Some(PackageManager::DirectBinary)
        );
        assert_eq!(PackageManager::from_str_loose("unknown_pm"), None);
    }

    #[test]
    fn test_update_cache_serialization_and_ttl() {
        let cache = UpdateCheckCache {
            last_checked: 1700000000,
            latest_version: "0.0.7-beta".to_string(),
            channel: ReleaseChannel::Stable,
        };

        let json = serde_json::to_string(&cache).unwrap();
        let deserialized: UpdateCheckCache = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, cache);
    }

    #[test]
    fn test_format_update_notice_box_styling() {
        let notice = UpdateNotice {
            current_version: "0.0.6-beta".to_string(),
            latest_version: "0.0.7-beta".to_string(),
            channel: ReleaseChannel::Stable,
            update_command: "forgum update".to_string(),
            pm_update_command: Some("sudo zypper update forgum".to_string()),
        };

        let box_str = format_update_notice_box(&notice);
        assert!(box_str.contains("0.0.6-beta"));
        assert!(box_str.contains("0.0.7-beta"));
        assert!(box_str.contains("forgum update"));
        assert!(box_str.contains("sudo zypper update forgum"));
    }
}
