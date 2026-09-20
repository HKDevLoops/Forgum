# 🚀 Forgum Installation & Preflight Diagnostic Manual

This manual covers all installation workflows, preflight dependency checks, package manager compatibility, and uninstallation procedures.

---

## 📑 Table of Contents
1. [Quick Start (One-Liner Installers)](#1-quick-start-one-liner-installers)
2. [Interactive Preflight TUI Wizard](#2-interactive-preflight-tui-wizard)
3. [Package Manager Distribution Matrix](#3-package-manager-distribution-matrix)
4. [Recommended Dependencies & Host Capabilities](#4-recommended-dependencies--host-capabilities)
5. [Channel Selection (Stable, Nightly, Dev)](#5-channel-selection-stable-nightly-dev)
6. [Preflight Verification & Diagnostics](#6-preflight-verification--diagnostics)
7. [Clean Uninstallation (Soft vs Purge)](#7-clean-uninstallation-soft-vs-purge)

---

## 1. Quick Start (One-Liner Installers)

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/HKDevLoops/Forgum/main/install.ps1 | iex
```

### Linux, macOS, and BSD (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/HKDevLoops/Forgum/main/install.sh | bash
```

Both installers automatically detect your OS, CPU architecture, terminal capabilities, and package manager ecosystem.

---

## 2. Interactive Preflight TUI Wizard

When launched in an interactive terminal, the installer executes the **Celestial Preflight Setup Wizard**:

1. **Terminal Capabilities Probe**:
   - Checks 24-bit TrueColor support (`COLORTERM=truecolor` / `24bit`).
   - Checks UTF-8 Unicode glyph rendering.
   - Checks DECSTBM hardware split-scrolling support.
2. **Package Manager Audit**:
   - Detects whether Scoop, Homebrew, Winget, or Cargo are present.
   - Warns if existing installations exist to prevent `$PATH` shadowing.
3. **Dependency Diagnostics**:
   - Checks for `ffmpeg` (recommended for video/GIF recording with `forgum record`).
   - Suggests one-command installation if missing.
4. **Shell Integration**:
   - Scans active shells on your system (Bash, Zsh, Fish, Pwsh, PowerShell, Nu, Elvish, Cmd).
   - Allows selecting which shells should receive prompt and startup hooks.
5. **Motivation Telemetry Consent**:
   - Explicit opt-in/opt-out for transparent usage metrics.

---

## 3. Package Manager Distribution Matrix

If you prefer using your operating system's native package manager:

| Package Manager | Platform | Install Command | Update Command |
| :--- | :--- | :--- | :--- |
| **Scoop** | Windows | `scoop install forgum` | `scoop update forgum` |
| **Homebrew** | macOS / Linux | `brew install forgum` | `brew upgrade forgum` |
| **WinGet** | Windows | `winget install HKDevLoops.Forgum` | `winget upgrade HKDevLoops.Forgum` |
| **Chocolatey** | Windows | `choco install forgum` | `choco upgrade forgum` |
| **Arch (AUR / Pacman)** | Linux | `yay -S forgum` | `yay -Syu forgum` |
| **openSUSE (Zypper / RPM)** | Linux | `sudo zypper in forgum` | `sudo zypper update forgum` |
| **Debian / Ubuntu (APT)** | Linux | `sudo dpkg -i forgum.deb` | `sudo apt update && sudo apt install --only-upgrade forgum` |
| **Fedora / RHEL (DNF)** | Linux | `sudo rpm -i forgum.rpm` | `sudo dnf upgrade forgum` |
| **Alpine (APK)** | Linux | `sudo apk add forgum` | `sudo apk upgrade forgum` |
| **FreeBSD (pkg)** | FreeBSD | `sudo pkg install forgum` | `sudo pkg upgrade forgum` |
| **Nix** | Cross-platform | `nix profile install .#forgum` | `nix profile upgrade forgum` |
| **Cargo** | Cross-platform | `cargo install forgum-cli` | `cargo install --force forgum-cli` |

---

## 4. Recommended Dependencies & Host Capabilities

While Forgum runs standalone without mandatory external dependencies, certain advanced features benefit from optional tools:

### A. Terminal TrueColor Support
- **Recommended**: Windows Terminal, WezTerm, Alacritty, Kitty, Ghostty, iTerm2.
- **Diagnostics**: Run `forgum doctor` to verify color depth and escape sequence support.

### B. FFmpeg (Optional: High-FPS Video/GIF Capture)
- Enables smooth frame-accurate terminal animation recording:
  ```bash
  # Windows (via Scoop)
  scoop install ffmpeg
  # macOS (via Homebrew)
  brew install ffmpeg
  # Ubuntu / Debian
  sudo apt install ffmpeg
  ```

---

## 5. Channel Selection (Stable, Nightly, Dev)

You can select your release channel during initial install or anytime later:

```bash
# Install stable channel (recommended)
./install.sh --channel stable

# Install nightly early-access channel
./install.sh --channel nightly

# Switch channels anytime via CLI
forgum update --channel nightly
forgum update --channel stable
```
See [Release Channels Guide](./CHANNELS.md) for full details.

---

## 6. Preflight Verification & Diagnostics

After installation, verify that Forgum is active and functioning smoothly:

```bash
# Check version and keyword integrity
forgum --version

# Run full system diagnostics
forgum doctor

# Run comprehensive health check
forgum checkhealth
```

---

## 7. Clean Uninstallation (Soft vs Purge)

Forgum honors zero-vendor lock-in with non-destructive, reversible uninstallation:

### Interactive Uninstallation Wizard
```bash
forgum uninstall --tui
```

### Soft Uninstallation (Default)
Removes binaries and shell hooks while preserving user configurations and custom `.cow` files:
```bash
forgum uninstall --method soft -y
```

### Purge Uninstallation (Clean Slate)
Completely removes all binaries, shell hooks, config folders, cache, and state:
```bash
forgum uninstall --method purge -y
```
