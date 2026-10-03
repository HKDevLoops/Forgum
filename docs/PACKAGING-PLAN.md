# 📦 Forgum Master Packaging & Release Deployment Plan

> **The Definitive Specification for Multi-Platform Packaging, Distribution Lanes, CI/CD Automation, and Version Governance for Forgum.**

---

## 📑 Table of Contents

1. [Universal Distribution Strategy & Target Matrix](#1-universal-distribution-strategy--target-matrix)
   - [Target Operating Systems & Architectures](#target-operating-systems--architectures)
   - [Rust Target Tuples & Toolchain Specification](#rust-target-tuples--toolchain-specification)
   - [Artifact Naming Conventions & Release Assets](#artifact-naming-conventions--release-assets)
2. [Package Managers Matrix & Concrete Manifestos](#2-package-managers-matrix--concrete-manifestos)
   - [Homebrew (macOS & Linux)](#-homebrew-macos--linux)
   - [Scoop (Windows)](#-scoop-windows)
   - [WinGet (Windows Package Manager)](#-winget-windows-package-manager)
   - [Chocolatey (Windows)](#-chocolatey-windows)
   - [Arch Linux User Repository (AUR)](#-arch-linux-user-repository-aur)
   - [Debian & Ubuntu (APT / dpkg)](#-debian--ubuntu-apt--dpkg)
   - [Fedora, RHEL & openSUSE (RPM)](#-fedora-rhel--opensuse-rpm)
   - [Nix & NixOS (Flake / nixpkgs)](#-nix--nixos-flake--nixpkgs)
   - [Alpine Linux (APK)](#-alpine-linux-apk)
   - [Crates.io (Cargo / Rust Ecosystem)](#-cratesio-cargo--rust-ecosystem)
3. [Automated CI/CD Release & Publishing Pipeline](#3-automated-cicd-release--publishing-pipeline)
   - [Pipeline Architecture Overview](#pipeline-architecture-overview)
   - [Complete GitHub Actions Workflow (`package-release.yml`)](#complete-github-actions-workflow-package-releaseyml)
   - [Downstream Package Manager Synchronization](#downstream-package-manager-synchronization)
4. [Release Governance, Verification & Rollback Protocols](#4-release-governance-verification--rollback-protocols)
   - [Stable v1.0.0 Production Readiness Criteria](#stable-v100-production-readiness-criteria)
   - [Semantic Versioning (SemVer) Contract](#semantic-versioning-semver-contract)
   - [Pre-Flight Health Checks & Verification Gate](#pre-flight-health-checks--verification-gate)
   - [Emergency Hotfix & Rollback Playbook](#emergency-hotfix--rollback-playbook)

---

## 1. Universal Distribution Strategy & Target Matrix

Forgum is engineered as a zero-dependency, ultra-low-overhead, native binary application. To achieve the mandate of universal accessibility without requiring end users to maintain Rust toolchains, Forgum distributes pre-compiled, cryptographically signed binaries across every tier-1 and tier-2 platform architecture.

### Target Operating Systems & Architectures

| Operating System | Architecture | ABI / Runtime | Distribution Tier | Primary Packaging Lane |
| :--- | :--- | :--- | :--- | :--- |
| **Linux** | `x86_64` (amd64) | glibc (>= 2.31) | Tier 1 (Core) | APT, RPM, Pacman, Tarball |
| **Linux** | `x86_64` (amd64) | musl (Static) | Tier 1 (Core) | Alpine APK, Static Tarball |
| **Linux** | `aarch64` (arm64) | glibc / musl | Tier 1 (Core) | APT, RPM, Pacman, Tarball |
| **Linux** | `armv7l` (armhf) | glibc / musl | Tier 2 (Embedded) | Raspberry Pi OS, Tarball |
| **Linux** | `riscv64gc` | glibc | Tier 2 (Next-Gen) | Debian Ports, Tarball |
| **Linux** | `i686` (32-bit x86) | glibc | Tier 3 (Legacy) | Legacy Debian / Tarball |
| **macOS** | `aarch64` (Apple Silicon) | Darwin (>= 11.0 Big Sur) | Tier 1 (Core) | Homebrew, Tarball |
| **macOS** | `x86_64` (Intel) | Darwin (>= 10.15 Catalina) | Tier 1 (Core) | Homebrew, Tarball |
| **Windows** | `x86_64` (x64) | MSVC (Windows 10/11) | Tier 1 (Core) | WinGet, Scoop, Choco, MSI, Zip |
| **Windows** | `aarch64` (ARM64) | MSVC (Surface / WoA) | Tier 1 (Core) | WinGet, Scoop, Choco, Zip |
| **Windows** | `i686` (x86 32-bit) | MSVC | Tier 3 (Legacy) | Scoop, Zip |
| **FreeBSD** | `x86_64` | Native ELF | Tier 2 (Community) | FreeBSD Ports / `pkg` |

### Rust Target Tuples & Toolchain Specification

All official release binaries are compiled with strict Link-Time Optimization (`lto = "thin"` or `"fat"`), symbol stripping (`strip = "debuginfo"`), and codegen units set to `1` for maximum execution efficiency and minimal binary footprint (< 15MB uncompressed, < 5MB compressed).

```toml
# Release compilation profile (Cargo.toml)
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = "debuginfo"
panic = "unwind"
```

| Target Tuple | Compiler / Cross Engine | Linker Flags / C Runtime |
| :--- | :--- | :--- |
| `x86_64-unknown-linux-gnu` | `rustc` on Ubuntu 20.04 | `-C target-feature=+crt-static` (off) / glibc 2.31 |
| `x86_64-unknown-linux-musl` | `cross` / Alpine Linux | `-C target-feature=+crt-static` (fully static) |
| `aarch64-unknown-linux-gnu` | `cross` (`cross-rs`) | GCC aarch64 cross-toolchain |
| `aarch64-unknown-linux-musl` | `cross` (`cross-rs`) | musl-gcc aarch64 static |
| `armv7-unknown-linux-gnueabihf` | `cross` (`cross-rs`) | arm-linux-gnueabihf-gcc |
| `riscv64gc-unknown-linux-gnu` | `cross` (`cross-rs`) | riscv64-linux-gnu-gcc |
| `x86_64-apple-darwin` | Xcode Toolchain on macOS-13 | MacOSX10.15.sdk |
| `aarch64-apple-darwin` | Xcode Toolchain on macOS-14 | MacOSX11.0.sdk |
| `x86_64-pc-windows-msvc` | MSVC 2022 on Windows Server 2022 | `link.exe` / Universal CRT |
| `aarch64-pc-windows-msvc` | MSVC 2022 Cross-ARM64 | `link.exe` / Universal CRT |

### Artifact Naming Conventions & Release Assets

Every tagged release (`vX.Y.Z`) publishes standardized, deterministic archive artifacts to GitHub Releases:

```text
forgum-v{VERSION}-{TARGET}.tar.gz       # Linux, macOS, BSD
forgum-v{VERSION}-{TARGET}.tar.gz.sha256
forgum-v{VERSION}-{TARGET}.zip          # Windows
forgum-v{VERSION}-{TARGET}.zip.sha256
forgum-v{VERSION}-windows-x64.msi       # Windows WiX Installer
forgum-v{VERSION}-windows-x64.msi.sha256
forgum_{VERSION}_amd64.deb              # Debian / Ubuntu
forgum_{VERSION}_arm64.deb              # Debian / Ubuntu ARM
forgum-{VERSION}-1.x86_64.rpm           # Fedora / RHEL / openSUSE
forgum-{VERSION}-1.aarch64.rpm          # Fedora / RHEL ARM
SHA256SUMS.txt                          # Universal checksum ledger
SHA256SUMS.txt.sig                      # Cosign / Minisign cryptographic signature
```

---

## 2. Package Managers Matrix & Concrete Manifestos

### 🍺 Homebrew (macOS & Linux)

Forgum operates two official Homebrew delivery channels:
1. **Official Custom Tap** (`HKDevLoops/homebrew-tap`): Instant distribution for stable, alpha, and beta releases.
2. **Upstream Homebrew-Core** (`homebrew/homebrew-core`): Upstream distribution conforming strictly to source-build standards.

#### Official Tap Formula (`Formula/forgum.rb`)
File path: `https://github.com/HKDevLoops/homebrew-tap/blob/main/Formula/forgum.rb`

```ruby
class Forgum < Formula
  desc "Cross-platform ANSI animation mascot and shell integration engine"
  homepage "https://github.com/HKDevLoops/Forgum"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000001"
    end
    on_intel do
      url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000002"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000003"
    end
    on_intel do
      url "https://github.com/HKDevLoops/Forgum/releases/download/v#{version}/forgum-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "0000000000000000000000000000000000000000000000000000000000000004"
    end
  end

  def install
    bin.install "forgum"

    # Generate and link auto-completions for all shells
    generate_completions_from_executable(bin/"forgum", "completions", shells: [:bash, :zsh, :fish])
  end

  def caveats
    <<~EOS
      ✨ To integrate Forgum into your active shell prompt:
        Bash:       eval "$(forgum init bash)"
        Zsh:        eval "$(forgum init zsh)"
        Fish:       forgum init fish | source
        PowerShell: forgum init pwsh | Out-String | Invoke-Expression

      Run 'forgum checkhealth' to verify your terminal environment.
    EOS
  end

  test do
    assert_match "forgum #{version}", shell_output("#{bin}/forgum --version")
    assert_match "ok", shell_output("#{bin}/forgum status")
  end
end
```

#### Upstream Homebrew-Core Formula (`forgum.rb`)
In compliance with Homebrew-core acceptance policies, upstream builds directly from the GitHub tagged source tarball using Cargo:

```ruby
class Forgum < Formula
  desc "Cross-platform ANSI animation mascot and shell integration engine"
  homepage "https://github.com/HKDevLoops/Forgum"
  url "https://github.com/HKDevLoops/Forgum/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
  license "MIT"
  head "https://github.com/HKDevLoops/Forgum.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/engine")
    generate_completions_from_executable(bin/"forgum", "completions", shells: [:bash, :zsh, :fish])
  end

  test do
    assert_match "forgum #{version}", shell_output("#{bin}/forgum --version")
  end
end
```

---

### 🍨 Scoop (Windows)

Scoop provides portable, user-space binary management without administrator rights.

#### Manifest (`HKDevLoops/scoop-bucket/forgum.json`)
File location in bucket repo: `bucket/forgum.json`

```json
{
    "version": "1.0.0",
    "description": "Cross-platform ANSI animation mascot and shell integration engine",
    "homepage": "https://github.com/HKDevLoops/Forgum",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/HKDevLoops/Forgum/releases/download/v1.0.0/forgum-v1.0.0-windows-x64.zip",
            "hash": "PLACEHOLDER_HASH_X64",
            "bin": "forgum.exe"
        },
        "arm64": {
            "url": "https://github.com/HKDevLoops/Forgum/releases/download/v1.0.0/forgum-v1.0.0-windows-arm64.zip",
            "hash": "PLACEHOLDER_HASH_ARM64",
            "bin": "forgum.exe"
        },
        "32bit": {
            "url": "https://github.com/HKDevLoops/Forgum/releases/download/v1.0.0/forgum-v1.0.0-windows-x86.zip",
            "hash": "PLACEHOLDER_HASH_X86",
            "bin": "forgum.exe"
        }
    },
    "checkver": {
        "github": "https://github.com/HKDevLoops/Forgum"
    },
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-v$version-windows-x64.zip"
            },
            "arm64": {
                "url": "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-v$version-windows-arm64.zip"
            },
            "32bit": {
                "url": "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-v$version-windows-x86.zip"
            }
        },
        "hash": {
            "url": "$url.sha256"
        }
    },
    "post_install": [
        "Write-Host '✦ Forgum installed successfully. Run \"forgum checkhealth\" or \"forgum init pwsh | iex\" to activate.' -ForegroundColor Cyan"
    ]
}
```

---

### 🪟 WinGet (Windows Package Manager)

WinGet uses multi-part manifests conforming to the Microsoft Community Repository standards.

#### Package Version Manifest (`HKDevLoops.Forgum.yaml`)
Path: `manifests/h/HKDevLoops/Forgum/1.0.0/HKDevLoops.Forgum.yaml`

```yaml
# yaml-language-server: $schema=https://aka.ms/winget-manifest.version.1.6.0.schema.json
PackageIdentifier: HKDevLoops.Forgum
PackageVersion: 1.0.0
DefaultLocale: en-US
ManifestType: version
ManifestVersion: 1.6.0
```

#### Installer Manifest (`HKDevLoops.Forgum.installer.yaml`)
Path: `manifests/h/HKDevLoops/Forgum/1.0.0/HKDevLoops.Forgum.installer.yaml`

```yaml
# yaml-language-server: $schema=https://aka.ms/winget-manifest.installer.1.6.0.schema.json
PackageIdentifier: HKDevLoops.Forgum
PackageVersion: 1.0.0
Platform:
  - Windows.Desktop
MinimumOSVersion: 10.0.17763.0
InstallerType: zip
NestedInstallerType: portable
NestedInstallerFiles:
  - RelativeFilePath: forgum.exe
    PortableCommandAlias: forgum
Commands:
  - forgum
Installers:
  - Architecture: x64
    InstallerUrl: https://github.com/HKDevLoops/Forgum/releases/download/v1.0.0/forgum-v1.0.0-windows-x64.zip
    InstallerSha256: PLACEHOLDER_HASH_X64
  - Architecture: arm64
    InstallerUrl: https://github.com/HKDevLoops/Forgum/releases/download/v1.0.0/forgum-v1.0.0-windows-arm64.zip
    InstallerSha256: PLACEHOLDER_HASH_ARM64
ManifestType: installer
ManifestVersion: 1.6.0
```

#### Locale Manifest (`HKDevLoops.Forgum.locale.en-US.yaml`)
Path: `manifests/h/HKDevLoops/Forgum/1.0.0/HKDevLoops.Forgum.locale.en-US.yaml`

```yaml
# yaml-language-server: $schema=https://aka.ms/winget-manifest.defaultLocale.1.6.0.schema.json
PackageIdentifier: HKDevLoops.Forgum
PackageVersion: 1.0.0
PackageLocale: en-US
Publisher: HKDevLoops
PublisherUrl: https://github.com/HKDevLoops
PackageName: Forgum
PackageUrl: https://github.com/HKDevLoops/Forgum
License: MIT
LicenseUrl: https://github.com/HKDevLoops/Forgum/blob/main/LICENSE
ShortDescription: Cross-platform ANSI animation mascot and shell integration engine.
Description: |
  Forgum is an ultra-lightweight, high-performance ANSI animation engine and mascot system.
  It renders cowsay, fortunes, and custom mascots with physics-based effects, procedural nature horizons,
  and split-terminal shell prompt integration across PowerShell, Bash, Zsh, Fish, and Nushell.
Moniker: forgum
Tags:
  - cowsay
  - fortune
  - lolcat
  - animation
  - terminal
  - cli
  - powershell
ManifestType: defaultLocale
ManifestVersion: 1.6.0
```

---

### 🍫 Chocolatey (Windows)

#### Specification File (`forgum.nuspec`)
Path: `packaging/choco/forgum.nuspec`

```xml
<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">
  <metadata>
    <id>forgum</id>
    <version>1.0.0</version>
    <title>Forgum</title>
    <authors>harish2222, HKDevLoops</authors>
    <owners>HKDevLoops</owners>
    <projectUrl>https://github.com/HKDevLoops/Forgum</projectUrl>
    <licenseUrl>https://github.com/HKDevLoops/Forgum/blob/main/LICENSE</licenseUrl>
    <requireLicenseAcceptance>false</requireLicenseAcceptance>
    <projectSourceUrl>https://github.com/HKDevLoops/Forgum</projectSourceUrl>
    <packageSourceUrl>https://github.com/HKDevLoops/Forgum/tree/main/packaging/choco</packageSourceUrl>
    <docsUrl>https://github.com/HKDevLoops/Forgum/blob/main/README.md</docsUrl>
    <bugTrackerUrl>https://github.com/HKDevLoops/Forgum/issues</bugTrackerUrl>
    <tags>forgum cowsay fortune lolcat terminal cli rust powershell animation</tags>
    <summary>Modern terminal mascot, ANSI animation engine, and shell integration tool.</summary>
    <description>
      Forgum is an ultra-lightweight, high-performance ANSI animation engine that renders cowsay,
      fortune quotes, and custom mascots with physical ANSI effects, procedural nature horizons,
      and split-shell integration above your prompt.
    </description>
    <releaseNotes>https://github.com/HKDevLoops/Forgum/releases/tag/v1.0.0</releaseNotes>
  </metadata>
  <files>
    <file src="tools\**" target="tools" />
  </files>
</package>
```

#### Install Script (`tools/chocolateyInstall.ps1`)

```powershell
$ErrorActionPreference = 'Stop'
$packageName = 'forgum'
$version     = '1.0.0'
$toolsDir    = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"

$url64       = "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-v$version-windows-x64.zip"
$urlArm64    = "https://github.com/HKDevLoops/Forgum/releases/download/v$version/forgum-v$version-windows-arm64.zip"

$hash64      = 'PLACEHOLDER_HASH_X64'
$hashArm64   = 'PLACEHOLDER_HASH_ARM64'

$isArm = $env:PROCESSOR_ARCHITECTURE -eq 'ARM64'
$url   = if ($isArm) { $urlArm64 } else { $url64 }
$hash  = if ($isArm) { $hashArm64 } else { $hash64 }

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  url           = $url
  checksum      = $hash
  checksumType  = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Ensure shim points to forgum.exe in toolsDir
$target = Join-Path $toolsDir 'forgum.exe'
Install-BinFile -Name 'forgum' -Path $target
```

#### Uninstall Script (`tools/chocolateyUninstall.ps1`)

```powershell
$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$binFile  = Join-Path $toolsDir 'forgum.exe'

# Execute forgum uninstaller soft de-orbit to cleanly detach hooks
if (Test-Path -LiteralPath $binFile) {
    & $binFile uninstall --method soft --yes
}
```

---

### 🏹 Arch Linux User Repository (AUR)

Forgum maintains two distinct AUR packages:
1. `forgum-bin`: Pre-compiled binary package for fast installation.
2. `forgum`: Source-built package leveraging Cargo in a clean chroot.

#### Binary Package (`forgum-bin/PKGBUILD`)

```bash
# Maintainer: harish2222 (HKDevLoops) <harish2222@users.noreply.github.com>
pkgname=forgum-bin
pkgver=1.0.0
pkgrel=1
pkgdesc="Cross-platform ANSI animation mascot and shell integration engine (prebuilt binary)"
arch=('x86_64' 'aarch64')
url="https://github.com/HKDevLoops/Forgum"
license=('MIT')
provides=('forgum')
conflicts=('forgum')
depends=('glibc')

source_x86_64=("https://github.com/HKDevLoops/Forgum/releases/download/v${pkgver}/forgum-v${pkgver}-x86_64-unknown-linux-gnu.tar.gz")
sha256sums_x86_64=('PLACEHOLDER_HASH_X86_64')

source_aarch64=("https://github.com/HKDevLoops/Forgum/releases/download/v${pkgver}/forgum-v${pkgver}-aarch64-unknown-linux-gnu.tar.gz")
sha256sums_aarch64=('PLACEHOLDER_HASH_AARCH64')

package() {
  install -Dm755 "${srcdir}/forgum" "${pkgdir}/usr/bin/forgum"

  # Shell completions
  install -d "${pkgdir}/usr/share/bash-completion/completions"
  "${pkgdir}/usr/bin/forgum" completions bash > "${pkgdir}/usr/share/bash-completion/completions/forgum"

  install -d "${pkgdir}/usr/share/zsh/site-functions"
  "${pkgdir}/usr/bin/forgum" completions zsh > "${pkgdir}/usr/share/zsh/site-functions/_forgum"

  install -d "${pkgdir}/usr/share/fish/vendor_completions.d"
  "${pkgdir}/usr/bin/forgum" completions fish > "${pkgdir}/usr/share/fish/vendor_completions.d/forgum.fish"

  # License
  install -Dm644 "${srcdir}/LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE" 2>/dev/null || true
}
```

#### Source Package (`forgum/PKGBUILD`)

```bash
# Maintainer: harish2222 (HKDevLoops) <harish2222@users.noreply.github.com>
pkgname=forgum
pkgver=1.0.0
pkgrel=1
pkgdesc="Cross-platform ANSI animation mascot and shell integration engine"
arch=('x86_64' 'aarch64' 'armv7h' 'riscv64')
url="https://github.com/HKDevLoops/Forgum"
license=('MIT')
depends=('gcc-libs')
makedepends=('cargo')
source=("${pkgname}-${pkgver}.tar.gz::https://github.com/HKDevLoops/Forgum/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('PLACEHOLDER_SOURCE_TARBALL_HASH')

prepare() {
  cd "Forgum-${pkgver}"
  export RUSTUP_TOOLCHAIN=stable
  cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
  cd "Forgum-${pkgver}"
  export RUSTUP_TOOLCHAIN=stable
  export CARGO_TARGET_DIR=target
  cargo build --frozen --release --bin forgum
}

check() {
  cd "Forgum-${pkgver}"
  export RUSTUP_TOOLCHAIN=stable
  cargo test --frozen -p forgum-platform
}

package() {
  cd "Forgum-${pkgver}"
  install -Dm755 "target/release/forgum" "${pkgdir}/usr/bin/forgum"
  install -Dm644 "LICENSE" "${pkgdir}/usr/share/licenses/${pkgname}/LICENSE"

  # Completions
  install -d "${pkgdir}/usr/share/bash-completion/completions"
  "${pkgdir}/usr/bin/forgum" completions bash > "${pkgdir}/usr/share/bash-completion/completions/forgum"

  install -d "${pkgdir}/usr/share/zsh/site-functions"
  "${pkgdir}/usr/bin/forgum" completions zsh > "${pkgdir}/usr/share/zsh/site-functions/_forgum"

  install -d "${pkgdir}/usr/share/fish/vendor_completions.d"
  "${pkgdir}/usr/bin/forgum" completions fish > "${pkgdir}/usr/share/fish/vendor_completions.d/forgum.fish"
}
```

---

### 🐧 Debian & Ubuntu (APT / dpkg)

#### Binary Control File (`packaging/deb/DEBIAN/control`)

```control
Package: forgum
Version: 1.0.0
Architecture: amd64
Maintainer: harish2222 (HKDevLoops) <harish2222@users.noreply.github.com>
Homepage: https://github.com/HKDevLoops/Forgum
Description: Cross-platform ANSI animation mascot and shell integration engine
 Forgum is a high-performance terminal mascot engine rendering cowsay,
 fortunes, and procedural nature biomes with 60 FPS TrueColor animations.
Section: utils
Priority: optional
Depends: libc6 (>= 2.31)
```

#### Post-Installation Hook (`packaging/deb/DEBIAN/postinst`)

```bash
#!/bin/sh
set -e

if [ "$1" = "configure" ]; then
    # Generate system-wide shell completions if directories exist
    if [ -x /usr/bin/forgum ]; then
        if [ -d /usr/share/bash-completion/completions ]; then
            /usr/bin/forgum completions bash > /usr/share/bash-completion/completions/forgum 2>/dev/null || true
        fi
        if [ -d /usr/share/zsh/vendor-completions ]; then
            /usr/bin/forgum completions zsh > /usr/share/zsh/vendor-completions/_forgum 2>/dev/null || true
        fi
        if [ -d /usr/share/fish/vendor_completions.d ]; then
            /usr/bin/forgum completions fish > /usr/share/fish/vendor_completions.d/forgum.fish 2>/dev/null || true
        fi
    fi
fi

exit 0
```

#### APT Repository Hosting Strategy (GitHub Pages + Aptly)
Forgum hosts an automated, signed APT repository directly via GitHub Pages at `https://hkdevloops.github.io/forgum-apt`:
1. CI invokes `reprepro` or `aptly` to ingest newly built `.deb` packages across `amd64` and `arm64`.
2. Packages are signed with a dedicated release GPG subkey: `secring.gpg`.
3. InRelease, Release, and Release.gpg metadata are generated and deployed to the `gh-pages` branch.
4. Users configure the repository via:
   ```bash
   curl -fsSL https://hkdevloops.github.io/forgum-apt/KEY.gpg | sudo gpg --dearmor -o /etc/apt/keyrings/forgum.gpg
   echo "deb [signed-by=/etc/apt/keyrings/forgum.gpg] https://hkdevloops.github.io/forgum-apt stable main" | sudo tee /etc/apt/sources.list.d/forgum.list
   sudo apt update && sudo apt install forgum
   ```

---

### 🎩 Fedora, RHEL & openSUSE (RPM)

#### RPM Spec Specification (`packaging/rpm/forgum.spec`)

```spec
Name:           forgum
Version:        1.0.0
Release:        1%{?dist}
Summary:        Cross-platform ANSI animation mascot and shell integration engine
License:        MIT
URL:            https://github.com/HKDevLoops/Forgum
Source0:        https://github.com/HKDevLoops/Forgum/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo >= 1.70.0
BuildRequires:  rust >= 1.70.0

%description
Forgum is an ultra-lightweight, high-performance terminal mascot engine rendering
cowsay, fortunes, and procedural nature biomes with TrueColor ANSI effects and
split-terminal shell prompt multitasking.

%prep
%autosetup -n Forgum-%{version}

%build
cargo build --release --locked --bin forgum

%install
rm -rf %{buildroot}
install -d %{buildroot}%{_bindir}
install -m 755 target/release/forgum %{buildroot}%{_bindir}/forgum

# Shell completions
install -d %{buildroot}%{_datadir}/bash-completion/completions
target/release/forgum completions bash > %{buildroot}%{_datadir}/bash-completion/completions/forgum

install -d %{buildroot}%{_datadir}/zsh/site-functions
target/release/forgum completions zsh > %{buildroot}%{_datadir}/zsh/site-functions/_forgum

install -d %{buildroot}%{_datadir}/fish/vendor_completions.d
target/release/forgum completions fish > %{buildroot}%{_datadir}/fish/vendor_completions.d/forgum.fish

%check
target/release/forgum --version

%files
%license LICENSE
%doc README.md
%{_bindir}/forgum
%{_datadir}/bash-completion/completions/forgum
%{_datadir}/zsh/site-functions/_forgum
%{_datadir}/fish/vendor_completions.d/forgum.fish

%changelog
* Fri Oct 02 2026 harish2222 <harish2222@users.noreply.github.com> - 1.0.0-1
- Official 1.0.0 production release.
```

#### Fedora COPR & openSUSE Build Service (OBS) Integration
- **Fedora COPR**: Managed under project `hkdevloops/forgum`. Webhook triggers automated `srpm` builds on release tag for Fedora 39, 40, 41, Rawhide, and EPEL 9/10.
- **OBS**: Linked repository `home:hkdevloops:forgum` automatically mirrors to openSUSE Tumbleweed, Leap 15.5/15.6, and SUSE Linux Enterprise.

---

### ❄️ Nix & NixOS (Flake / nixpkgs)

#### Upstream Nixpkgs Derivation (`pkgs/by-name/fo/forgum/package.nix`)

```nix
{ lib
, rustPlatform
, fetchFromGitHub
, installShellFiles
}:

rustPlatform.buildRustPackage rec {
  pname = "forgum";
  version = "1.0.0";

  src = fetchFromGitHub {
    owner = "HKDevLoops";
    repo = "Forgum";
    rev = "v${version}";
    hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
  };

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ installShellFiles ];

  cargoBuildFlags = [ "--bin" "forgum" ];
  cargoTestFlags = [ "-p" "forgum-platform" ];

  postInstall = ''
    installShellCompletion --cmd forgum \
      --bash <($out/bin/forgum completions bash) \
      --zsh <($out/bin/forgum completions zsh) \
      --fish <($out/bin/forgum completions fish)
  '';

  meta = with lib; {
    description = "Cross-platform ANSI animation mascot and shell integration engine";
    homepage = "https://github.com/HKDevLoops/Forgum";
    changelog = "https://github.com/HKDevLoops/Forgum/releases/tag/v${version}";
    license = licenses.mit;
    maintainers = [ ];
    mainProgram = "forgum";
    platforms = platforms.unix ++ platforms.windows;
  };
}
```

#### Flake Definition (`flake.nix`)
Path: `packaging/nix/flake.nix`
Supports immediate execution via `nix run github:HKDevLoops/Forgum` or inclusion into NixOS system configurations.

---

### 🏔️ Alpine Linux (APK)

#### Alpine Package Build (`community/forgum/APKBUILD`)

```sh
# Contributor: harish2222 <harish2222@users.noreply.github.com>
# Maintainer: harish2222 <harish2222@users.noreply.github.com>
pkgname=forgum
pkgver=1.0.0
pkgrel=0
pkgdesc="Cross-platform ANSI animation mascot and shell integration engine"
url="https://github.com/HKDevLoops/Forgum"
arch="all"
license="MIT"
makedepends="cargo rust"
subpackages="$pkgname-doc $pkgname-bash-completion $pkgname-zsh-completion $pkgname-fish-completion"
source="$pkgname-$pkgver.tar.gz::https://github.com/HKDevLoops/Forgum/archive/refs/tags/v$pkgver.tar.gz"
builddir="$srcdir/Forgum-$pkgver"

prepare() {
    default_prepare
    cargo fetch --locked --target "$CTARGET"
}

build() {
    cargo build --frozen --release --bin forgum
}

check() {
    target/release/forgum --version
}

package() {
    install -Dm755 target/release/forgum "$pkgdir"/usr/bin/forgum
    install -Dm644 LICENSE "$pkgdir"/usr/share/licenses/$pkgname/LICENSE

    # Completions
    target/release/forgum completions bash > forgum.bash
    install -Dm644 forgum.bash "$pkgdir"/usr/share/bash-completion/completions/forgum

    target/release/forgum completions zsh > forgum.zsh
    install -Dm644 forgum.zsh "$pkgdir"/usr/share/zsh/site-functions/_forgum

    target/release/forgum completions fish > forgum.fish
    install -Dm644 forgum.fish "$pkgdir"/usr/share/fish/vendor_completions.d/forgum.fish
}
```

---

### 🦀 Crates.io (Cargo / Rust Ecosystem)

Forgum's architecture separates core platform primitives from the engine and TUI widgets:
1. `forgum-platform` (`crates/platform`): OS hooks, DECSTBM, cross-platform terminal handles.
2. `forgum-tui` (`crates/tui`): TrueColor ANSI widgets and menu UI.
3. `forgum` (`crates/engine`): Animation loops, command handlers, and the `forgum` binary.

#### Publishing Dependency Order
Due to Cargo inter-crate dependencies, publishing must occur in topological sequence:
```bash
# 1. Publish platform abstraction crate
cd crates/platform && cargo publish --locked

# 2. Wait 30 seconds for Crates.io index propagation
sleep 30

# 3. Publish TUI crate
cd ../tui && cargo publish --locked

# 4. Wait 30 seconds
sleep 30

# 5. Publish Engine binary crate
cd ../engine && cargo publish --locked
```

#### Crate Exclusion Manifest (`crates/engine/Cargo.toml`)
To comply with Crates.io 10MB package limits and avoid publishing non-essential assets:

```toml
[package]
name = "forgum"
version = "1.0.0"
edition = "2021"
license = "MIT"
description = "Forgum ANSI animation mascot and shell integration engine"
repository = "https://github.com/HKDevLoops/Forgum"
readme = "../../README.md"
keywords = ["cowsay", "fortune", "lolcat", "animation", "terminal"]
categories = ["command-line-interface", "graphics"]
default-run = "forgum"

exclude = [
    "../../test-renders/**",
    "../../recordings/**",
    "../../packaging/**",
    "../../docs/**",
    "../../.github/**",
    "../../tests/**"
]
```

---

## 3. Automated CI/CD Release & Publishing Pipeline

### Pipeline Architecture Overview

The release pipeline executes in four concurrent and sequential phases:

```text
 ┌────────────────────────────────────────────────────────┐
 │            Trigger: Push Git Tag (v*.*.*)              │
 └───────────────────────────┬────────────────────────────┘
                             │
 ┌───────────────────────────▼────────────────────────────┐
 │  Phase 1: Multi-Arch Build Matrix (10 targets)         │
 │  • Linux (GNU/musl, x86_64, aarch64, armv7, riscv64)   │
 │  • macOS (Intel, Apple Silicon)                        │
 │  • Windows (x64, ARM64, x86)                           │
 └───────────────────────────┬────────────────────────────┘
                             │
 ┌───────────────────────────▼────────────────────────────┐
 │  Phase 2: Packaging & Checksum Aggregation             │
 │  • Generate .tar.gz, .zip, .deb, .rpm, .msi            │
 │  • Compute SHA256SUMS.txt & Cosign Signature           │
 │  • Draft & Publish GitHub Release with Assets          │
 └───────────────────────────┬────────────────────────────┘
                             │
 ┌───────────────────────────▼────────────────────────────┐
 │  Phase 3: Package Manager Automation Dispatch          │
 │  • Scoop Bucket Auto-PR via commit push                │
 │  • WinGet Auto-PR via wingetcreate                     │
 │  • Arch Linux AUR Auto-Push via SSH key                │
 │  • Homebrew Tap Formula Update                         │
 │  • Crates.io Publish in topological sequence           │
 └────────────────────────────────────────────────────────┘
```

### Complete GitHub Actions Workflow (`package-release.yml`)
File path: `.github/workflows/package-release.yml`

```yaml
name: Release & Packaging Pipeline

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write
  packages: write
  id-token: write

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  # =========================================================================
  # 1. Build Binaries Matrix across 10 Target Architectures
  # =========================================================================
  build-binaries:
    name: Build (${{ matrix.target }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        include:
          # Linux GNU & Musl (x86_64)
          - os: ubuntu-20.04
            target: x86_64-unknown-linux-gnu
            use_cross: false
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            use_cross: true

          # Linux ARM & RISC-V
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            use_cross: true
          - os: ubuntu-latest
            target: aarch64-unknown-linux-musl
            use_cross: true
          - os: ubuntu-latest
            target: armv7-unknown-linux-gnueabihf
            use_cross: true
          - os: ubuntu-latest
            target: riscv64gc-unknown-linux-gnu
            use_cross: true

          # macOS (Apple Silicon & Intel)
          - os: macos-14
            target: aarch64-apple-darwin
            use_cross: false
          - os: macos-13
            target: x86_64-apple-darwin
            use_cross: false

          # Windows (x64, ARM64)
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            use_cross: false
          - os: windows-latest
            target: aarch64-pc-windows-msvc
            use_cross: false

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Rust Toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross (if needed)
        if: matrix.use_cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Compile Release Binary
        run: |
          if [ "${{ matrix.use_cross }}" = "true" ]; then
            cross build --release --locked --target ${{ matrix.target }} --bin forgum
          else
            cargo build --release --locked --target ${{ matrix.target }} --bin forgum
          fi
        shell: bash

      - name: Package Compressed Archive
        shell: bash
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          STAGING="forgum-v${VERSION}-${{ matrix.target }}"
          mkdir -p "$STAGING"

          if [ "${{ runner.os }}" = "Windows" ]; then
            cp "target/${{ matrix.target }}/release/forgum.exe" "$STAGING/"
            cp LICENSE README.md "$STAGING/"
            7z a "${STAGING}.zip" "$STAGING"
            echo "ASSET=${STAGING}.zip" >> $GITHUB_ENV
          else
            cp "target/${{ matrix.target }}/release/forgum" "$STAGING/"
            cp LICENSE README.md "$STAGING/"
            tar -czvf "${STAGING}.tar.gz" "$STAGING"
            echo "ASSET=${STAGING}.tar.gz" >> $GITHUB_ENV
          fi

      - name: Upload Build Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ env.ASSET }}
          path: ${{ env.ASSET }}

  # =========================================================================
  # 2. Native System Packages (DEB, RPM, MSI)
  # =========================================================================
  build-packages:
    name: Build Native Installers
    needs: [build-binaries]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Debian & RPM Packaging Utilities
        run: sudo apt-get update && sudo apt-get install -y dpkg-dev rpm

      - name: Download x86_64 Linux Binary
        uses: actions/download-artifact@v4
        with:
          pattern: forgum-*-x86_64-unknown-linux-gnu.tar.gz
          merge-multiple: true

      - name: Package DEB & RPM
        shell: bash
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          tar -xzf forgum-v${VERSION}-x86_64-unknown-linux-gnu.tar.gz
          mkdir -p target/release
          cp forgum-v${VERSION}-x86_64-unknown-linux-gnu/forgum target/release/

          # Build deb
          ./packaging/deb/build-deb.sh "$VERSION"

          # Build rpm
          ./packaging/rpm/build-rpm.sh "$VERSION"

      - name: Upload Linux Installers
        uses: actions/upload-artifact@v4
        with:
          name: linux-installers
          path: |
            *.deb
            rpmbuild/RPMS/*/*.rpm

  # =========================================================================
  # 3. Checksums, Signatures & GitHub Release Creation
  # =========================================================================
  publish-release:
    name: Publish GitHub Release
    needs: [build-binaries, build-packages]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all release artifacts
        uses: actions/download-artifact@v4
        with:
          path: dist
          merge-multiple: true

      - name: Generate SHA256SUMS.txt
        working-directory: dist
        run: |
          sha256sum * > SHA256SUMS.txt
          cat SHA256SUMS.txt

      - name: Install Cosign
        uses: sigstore/cosign-installer@v3.5.0

      - name: Sign Checksums with Cosign
        working-directory: dist
        run: cosign sign-blob --yes --output-signature SHA256SUMS.txt.sig SHA256SUMS.txt

      - name: Create Official GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: dist/*
          generate_release_notes: true
          draft: false
          prerelease: false

  # =========================================================================
  # 4. Multi-Channel Distribution Lane Dispatch
  # =========================================================================
  dispatch-package-managers:
    name: Package Manager Auto-Update Dispatch
    needs: [publish-release]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download Release Checksums
        uses: actions/download-artifact@v4
        with:
          name: dist
          path: dist

      - name: Update Scoop Bucket Manifest
        env:
          SCOOP_BUCKET_PAT: ${{ secrets.SCOOP_BUCKET_PAT }}
        run: |
          VERSION="${GITHUB_REF_NAME#v}"
          HASH_X64=$(grep "forgum-v${VERSION}-windows-x64.zip" dist/SHA256SUMS.txt | awk '{print $1}')
          HASH_ARM64=$(grep "forgum-v${VERSION}-windows-arm64.zip" dist/SHA256SUMS.txt | awk '{print $1}')

          git clone https://x-access-token:${SCOOP_BUCKET_PAT}@github.com/HKDevLoops/scoop-bucket.git scoop-repo
          cd scoop-repo
          sed -i "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" bucket/forgum.json
          sed -i "s/PLACEHOLDER_HASH_X64/${HASH_X64}/" bucket/forgum.json
          sed -i "s/PLACEHOLDER_HASH_ARM64/${HASH_ARM64}/" bucket/forgum.json
          git config user.name "forgum-bot"
          git config user.email "bot@hkdevloops.com"
          git commit -am "chore(forgum): bump to v${VERSION}"
          git push origin main

      - name: Dispatch Winget Package Submission
        uses: vedantmgoyal2009/winget-releaser@v2
        with:
          identifier: HKDevLoops.Forgum
          version: ${{ github.ref_name }}
          token: ${{ secrets.WINGET_PAT }}

      - name: Push to Arch User Repository (AUR)
        env:
          AUR_KEY: ${{ secrets.AUR_SSH_KEY }}
        run: |
          mkdir -p ~/.ssh
          echo "$AUR_KEY" > ~/.ssh/id_rsa
          chmod 600 ~/.ssh/id_rsa
          ssh-keyscan -H aur.archlinux.org >> ~/.ssh/known_hosts

          git clone ssh://aur@aur.archlinux.org/forgum-bin.git aur-bin
          cd aur-bin
          VERSION="${GITHUB_REF_NAME#v}"
          sed -i "s/^pkgver=.*/pkgver=${VERSION}/" PKGBUILD
          sed -i "s/^pkgrel=.*/pkgrel=1/" PKGBUILD
          makepkg --printsrcinfo > .SRCINFO
          git config user.name "harish2222"
          git config user.email "harish2222@users.noreply.github.com"
          git commit -am "chore: release v${VERSION}"
          git push origin master

      - name: Update Homebrew Tap Formula
        uses: mislav/bump-homebrew-formula-action@v3
        with:
          formula-name: forgum
          homebrew-tap: HKDevLoops/homebrew-tap
          base-branch: main
          download-url: https://github.com/HKDevLoops/Forgum.git
        env:
          COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_PAT }}

      - name: Publish to Crates.io
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          cargo publish --locked -p forgum-platform
          sleep 30
          cargo publish --locked -p forgum-tui
          sleep 30
          cargo publish --locked -p forgum
```

---

## 4. Release Governance, Verification & Rollback Protocols

### Stable v1.0.0 Production Readiness Criteria

A release candidate is approved for promotion to `v1.0.0` (Stable) only when all ten criteria pass without exception:

| # | Readiness Verification Category | Strict Acceptance Ceiling | Automated Check Command |
| :- | :--- | :--- | :--- |
| **1** | **Strict Memory Ceiling** | Resident RAM < 100MB; typical < 15MB | `cargo bench --bench engine_benches` |
| **2** | **Zero-Heap Render Loops** | Stack primitives only during frame paint | Valgrind / Heaptrack audit |
| **3** | **DECSTBM Multi-Tasking** | Split-scroll buffer integrity in all viewports | `cargo test test_split_scroll` |
| **4** | **Shell Hook Latency** | Invocation startup overhead < 2.0ms | Hyperfine benchmark (`forgum init bash`) |
| **5** | **15-Shell Completeness** | Valid hook generation across all 15 shells | CI test matrix (`cargo test shell`) |
| **6** | **Mascot Art Integrity** | All 106 `.cow` files render without overflow | `forgum list animals --validate` |
| **7** | **Zero Unsafe Code** | Workspace denies `unsafe_code` without bypass | `cargo clippy -- -D unsafe_code` |
| **8** | **Cross-Platform Seams** | `#[cfg]` strictly confined to `forgum-platform` | CI Grep Gate (`rg '#\[cfg' crates/engine/`) |
| **9** | **Clean De-Orbit Guarantee**| Soft and Purge modes leave zero orphan hooks | Container uninstallation test |
| **10**| **Telemetry Consent Gate** | 100% offline compliance when declined | Air-gapped container test |

### Semantic Versioning (SemVer) Contract

Forgum strictly adheres to **Semantic Versioning 2.0.0** (`MAJOR.MINOR.PATCH`):

- **MAJOR (Breaking Changes)**:
  - Removal or renaming of CLI flags or subcommands.
  - Breaking schema changes to `~/.config/forgum/config.json`.
  - Incompatible changes to `forgum init <shell>` hook contracts requiring manual user RC edits.
- **MINOR (New Features & Enhancements)**:
  - Introduction of new animal mascots, animation effects, or procedural biomes.
  - New configuration options that default to backwards-compatible behaviors.
  - Additions to the shell integration matrix.
- **PATCH (Bug Fixes & Maintenance)**:
  - Terminal rendering glitch fixes and ANSI escape optimizations.
  - Package manager manifest updates and CI stabilization.
  - Documentation refinements.

### Pre-Flight Health Checks & Verification Gate

Before tagging any release:

```bash
# 1. Run local unified diagnostic health check
cargo run --bin forgum -- checkhealth

# 2. Verify all animal mascots and ANSI palettes
cargo run --bin forgum -- list animals
cargo run --bin forgum -- list effects
cargo run --bin forgum -- list scenery

# 3. Verify clean container builds
docker run --rm -v $(pwd):/workspace:ro ubuntu:22.04 bash -c \
  "apt-get update -qq && apt-get install -y -qq curl ca-certificates && bash /workspace/install.sh --headless && forgum doctor"

# 4. Verify uninstallation idempotency
./uninstall.sh --method purge --yes
```

### Emergency Hotfix & Rollback Playbook

In the critical event that a release causes breaking terminal corruption or regressions:

1. **GitHub Release Yank**:
   Mark the release as "Pre-release" on GitHub Releases or delete the tag to halt automated downloads.
2. **Crates.io Yank**:
   ```bash
   cargo yank --version 1.0.0 forgum
   ```
3. **Package Manager Rollback Instructions**:
   - **Scoop**: Run `scoop reset forgum@<previous-version>` or revert commit in `HKDevLoops/scoop-bucket`.
   - **Homebrew**: Push immediate formula rollback commit restoring the previous release tag and hashes.
   - **WinGet**: Submit fast-track PR with `wingetcreate` pointing to previous known stable release.
   - **Arch AUR**: Roll back git commit on `forgum-bin` and push to master.
4. **Emergency Patch Tag**:
   Publish `v1.0.1` immediately containing the targeted regression fix following the standard CI pipeline.
