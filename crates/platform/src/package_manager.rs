//! Package manager detection, tracking, and update dispatch.
//!
//! Detects how Forgum was installed (Scoop, WinGet, Chocolatey, Homebrew,
//! Pacman, APT, Cargo, or direct standalone binary) and manages update commands.

use std::path::Path;
use std::process::Command;

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
}
