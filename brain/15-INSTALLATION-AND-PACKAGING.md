# Forgum — Installation & Packaging Plan (cross-platform, version-resilient)

> **The "it just installs" blueprint.** Forgum must install on Windows, macOS, and Linux (Debian, Arch/AUR, Fedora, and Nix) through **every channel a user expects** — winget, scoop, MSI, EXE, apt, dnf, pacman/AUR, Homebrew, Nix flake, and a plain interactive shell script — and **all of them must produce the same canonical binary at the same canonical path**, so that a user who switches install methods (or upgrades across versions) never gets a broken, half-installed, or duplicated Forgum.
>
> This document specifies the **single-canonical-path principle**, the **install matrix**, the **interactive installer scripts** per platform, the **package manifests** (and the PRs to upstream them into winget/apt/dnf/brew/pacman), the **shell-hook injection** that happens post-install, the **`forgum doctor` integrity check**, and the **phased/test-gated rollout**. It is the delivery layer for v2/v3; it does not re-spec the engine or AI.
>
> Read after `06-ARCHITECTURE.md`, `10-FINE-TUNED-MASTER-PLAN.md`, and `16-CLI-DESIGN-…`.

---

## 0. The single-canonical-path principle (the constitution of packaging)

Every install method is a **delivery vehicle**, not a destination. They all converge on the same on-disk layout. This is the rule that makes version resilience possible:

| Platform | Canonical binary | Canonical config | Canonical data | Canonical cache |
|----------|------------------|-------------------|-----------------|-----------------|
| Windows | `%LOCALAPPDATA%\Programs\forgum\forgum.exe` | `%APPDATA%\forgum\config.toml` | `%APPDATA%\forgum\` | `%LOCALAPPDATA%\forgum\cache\` |
| macOS | `/opt/forgum/bin/forgum` (Homebrew-style) or `~/.local/bin/forgum` (user) | `~/.config/forgum/config.toml` | `~/.local/share/forgum/` | `~/.cache/forgum/` |
| Linux | `/usr/bin/forgum` (system) or `~/.local/bin/forgum` (user) | `~/.config/forgum/config.toml` | `~/.local/share/forgum/` | `~/.cache/forgum/` |

**Rules derived from this:**

1. **One binary wins.** If two install methods place a `forgum` binary on `PATH`, the one at the canonical path is authoritative; `forgum doctor` detects and warns about "shadowing" binaries and offers to remove the losers. No silent dual-installs.
2. **Config is shared, never per-install-method.** Whether you installed via winget or scoop, the config lives in `%APPDATA%\forgum\`. Uninstalling one method never deletes config/data/cache — only the binary + the method's own metadata.
3. **The binary is self-locating.** `forgum` knows its own canonical path (via `std::env::current_exe` + the install manifest) and refuses to run if it finds itself at a non-canonical path *and* a canonical one exists — instead it prints *"a canonical Forgum exists at X; this copy at Y is stale; run `forgum doctor`."* This prevents the "I installed it twice and the wrong one runs" bug class.
4. **Version migration is automatic.** On startup, `forgum` reads `config.toml`'s `schema_version`; if older than the binary's, it runs the migration ladder (§7) before anything else renders. A backup `config.toml.bak.{ts}` is written first.
5. **Uninstall is surgical.** `forgum uninstall` (or the package manager's uninstall) removes *only* what that method installed. Config/data/cache survive unless the user passes `--purge`. This is tested per method.

---

## 1. The install matrix

| Platform | Method | Manifest type | Maintainer | Automation | Phase |
|----------|--------|---------------|------------|------------|-------|
| Windows | **winget** | `.yaml` in `winget-pkgs` | upstream PR | CI publishes to a release; PR updates winget-pkgs | P1 |
| Windows | **scoop** | `forgum.json` bucket | our `scoop-forgum` bucket | CI pushes JSON on release | P1 |
| Windows | **MSI** | WiX `.wxs` → `.msi` | us | CI builds + signs | P2 |
| Windows | **EXE** | self-extracting (cargo-wix or Inno Setup) | us | CI builds | P2 |
| Windows | **interactive `install.ps1`** | PowerShell script | us | in-repo `install.ps1` | P0 |
| Linux (Debian) | **.deb** | `cargo-deb` | us | CI builds per arch | P1 |
| Linux (Debian) | **apt (PPA / repo)** | apt repo | our `.deb` repo + key | CI publishes to repo | P3 |
| Linux (Fedora) | **.rpm** | `cargo-generate-rpm` | us | CI builds per arch | P1 |
| Linux (Fedora) | **dnf (COPR)** | COPR repo | our COPR project | CI pushes SRPM | P3 |
| Linux (Arch) | **AUR** | `PKGBUILD` (`-bin` + `-git`) | AUR maintainer | CI updates AUR SSH push | P2 |
| Linux (Arch) | **pacman (repo)** | custom repo | our `[forgum]` repo | CI publishes | P3 |
| Linux (any) | **Nix** | `flake.nix` + `forgum.nix` | us, in-repo + nixpkgs PR | nixpkgs PR after stability | P2 |
| Linux (any) | **interactive `install.sh`** | bash script | us | in-repo `install.sh` | P0 |
| macOS | **Homebrew** | formula in `homebrew-forgum` tap | our tap + homebrew-core PR | CI pushes formula on release | P1 |
| macOS | **interactive `install.sh`** | (shared with Linux) | us | in-repo | P0 |
| macOS | **.pkg** | `pkgbuild` | us | CI builds + signs | P3 |
| All | **cargo install forgum** | crates.io | us | CI publishes crate | P0 |
| All | **build from source** | `git clone && cargo build --release` | us | README instructions | P0 |

**P0** = ships with v2.0 release. **P1** = within 2 weeks of v2.0. **P2** = within 4 weeks. **P3** = "nice to have" / community-sourced.

---

## 2. The interactive installer scripts (P0 — ships first)

The interactive installer is the **recommended path** for users who don't use a package manager. It is a thin shell/PowerShell wrapper that **downloads the right prebuilt binary for the platform+arch**, places it at the canonical path, runs `forgum init` (the TUI first-run, see `17-TUI-CONFIG-MENU.md`), and injects the shell hook. It is *not* a build-from-source script.

### 2.1 Design principle: the script delegates to the binary

The shell script does as little as possible. It:
1. Detects OS + arch + shell.
2. Downloads the matching tarball/zip from the GitHub release (with SHA-256 verification).
3. Extracts to the canonical path (or `~/.local/bin` if no sudo / no admin).
4. Runs `forgum init --first-run` (the TUI) to configure + inject the shell hook.
5. Runs `forgum doctor` to verify.

Everything "interactive" (toggle menus, shell selection, theme pick) is done by **`forgum init`'s TUI**, not by the shell script. This means the interactive experience is identical whether you installed via winget, brew, or `install.sh` — the script is just a fetcher. This is deliberate: one TUI to maintain, not three.

### 2.2 Linux + macOS: `install.sh` (shared, POSIX-ish bash)

```bash
#!/usr/bin/env bash
# forgum/install.sh — interactive installer for Linux + macOS
set -euo pipefail

FORGUM_VERSION="${FORGUM_VERSION:-latest}"
FORGUM_INSTALL_DIR="${FORGUM_INSTALL_DIR:-}"   # auto: /usr/local/bin (sudo) or ~/.local/bin
FORGUM_FORCE="${FORGUM_FORCE:-0}"

# 1. Detect OS + arch
OS="$(uname -s)"; ARCH="$(uname -m)"
case "$OS/$ARCH" in
  Linux/x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
  Linux/aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
  Linux/*musl*)  TARGET="x86_64-unknown-linux-musl" ;;
  Darwin/x86_64) TARGET="x86_64-apple-darwin" ;;
  Darwin/arm64)  TARGET="aarch64-apple-darwin" ;;
  *) echo "Unsupported: $OS/$ARCH"; exit 1 ;;
esac

# 2. Pick install dir (interactive if not set)
if [[ -z "$FORGUM_INSTALL_DIR" ]]; then
  if [[ -w /usr/local/bin ]]; then FORGUM_INSTALL_DIR="/usr/local/bin"
  elif command -v sudo >/dev/null && sudo -n true 2>/dev/null; then FORGUM_INSTALL_DIR="/usr/local/bin"
  else FORGUM_INSTALL_DIR="$HOME/.local/bin"; fi
fi

# 3. Resolve version + download URL
#    (queries GitHub API if "latest", prints the resolved version)
URL="https://github.com/harish2222/Forgum/releases/download/${VER}/forgum-${VER}-${TARGET}.tar.gz"
SHA_URL="$URL.sha256"

# 4. Download, verify SHA-256, extract
#    (curl + sha256sum -c; refuses to proceed on mismatch)

# 5. Place binary at canonical path
install -m 0755 forgum "$FORGUM_INSTALL_DIR/forgum"

# 6. Delegate to the binary for interactive first-run + shell hook
exec "$FORGUM_INSTALL_DIR/forgum" init --first-run
```

**The script is idempotent:** re-running it upgrades in place (it detects an existing install, asks "upgrade from X to Y?", overwrites the binary, runs `forgum doctor`).

**Flags it accepts** (for scripted installs, per the user's "args for scripting" requirement):
`--version <v>`, `--install-dir <path>`, `--force`, `--no-shell-hook`, `--yes` (non-interactive, all defaults), `--unstable` (track prereleases). All `--flag` forms have a `-s` short form documented in `--help`.

### 2.3 Windows: `install.ps1`

```powershell
# forgum/install.ps1 — interactive installer for Windows
#Requires -Version 5.1
[CmdletBinding()]
param(
  [string]$Version = "latest",
  [string]$InstallDir = "",
  [switch]$Force,
  [switch]$NoShellHook,
  [switch]$Yes,            # non-interactive
  [switch]$Unstable
)
$ErrorActionPreference = "Stop"

# 1. Detect arch
$arch = if ([Environment]::Is64BitOperatingSystem) {
  if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "aarch64-pc-windows-msvc" }
  else { "x86_64-pc-windows-msvc" }
} else { "x86-pc-windows-msvc" }

# 2. Canonical path: %LOCALAPPDATA%\Programs\forgum
if (-not $InstallDir) { $InstallDir = "$env:LOCALAPPDATA\Programs\forgum" }

# 3. Download + verify SHA-256 (Invoke-WebRequest + Get-FileHash)

# 4. Extract + place forgum.exe

# 5. Add to PATH (user-level, idempotent via [Environment]::SetEnvironmentVariable)

# 6. Delegate: forgum init --first-run  (the TUI, in Windows Terminal / conhost)
& "$InstallDir\forgum.exe" init --first-run
```

**PowerShell execution policy:** the script is signed (code-signing cert, P2) and the README shows `Set-ExecutionPolicy -Scope CurrentUser RemoteSigned` + `iex (irm https://forgum.dev/install.ps1)` as the one-liner. A `irm | iex` pipe is also offered for users who prefer it.

### 2.4 The `--first-run` TUI delegation

`forgum init --first-run` launches the full interactive TUI (see `17-…`): shell detection, theme pick, cow pick, AI on/off, shell-hook injection preview, "apply." The scripts above are deliberately boring; **all the magic is in `forgum init`**, which is the same on every platform. This is the key insight: **one interactive experience, many delivery vehicles.**

---

## 3. Package manifests (P1–P3)

All manifests are **generated from `Cargo.toml`** by CI (principle #10 of v2: one version source). A CI job asserts byte-stability of the generated manifests against committed templates.

### 3.1 winget manifest (Windows, P1)

Path in upstream `microsoft/winget-pkgs`: `manifests/f/Forgum/Forgum/<version>/`.

```yaml: Forgum.Forgum.installer.yaml
PackageIdentifier: Forgum.Forgum
PackageVersion: 2.0.0
InstallerLocale: en-US
InstallerType: wix          # MSI (P2); zip fallback for P1
Installers:
  - Architecture: x64
    InstallerType: wix
    InstallerUrl: https://github.com/harish2222/Forgum/releases/download/v2.0.0/forgum-2.0.0-x86_64-pc-windows-msvc.msi
    InstallerSha256: <sha256>
    Scope: user
    InstallerSwitches:
      Custom: ADDSHELLHOOK=1     # runs `forgum init --first-run --yes` post-install
  - Architecture: arm64
    InstallerType: wix
    InstallerUrl: ...aarch64...msi
    InstallerSha256: <sha256>
ManifestType: installer
ManifestVersion: 1.4.0
```

Plus `defaultLocale`, `locale`, `version` sibling manifests (the 4-file winget schema). The PR to `winget-pkgs` is opened by CI on every release via a bot account; a maintainer approves. **Review criteria winget enforces:** valid SHA-256, correct URL reachability, no admin requirement for user-scope, installer runs silently with `/quiet`.

### 3.2 scoop bucket (Windows, P1)

Our bucket repo `forgum/scoop-forgum` contains `bucket/forgum.json`:

```json
{
  "version": "2.0.0",
  "description": "Cross-platform cowsay + fortune + lolcat with a Rust animation engine",
  "homepage": "https://github.com/harish2222/Forgum",
  "license": "MIT",
  "architecture": {
    "64bit": {
      "url": "https://github.com/harish2222/Forgum/releases/download/v2.0.0/forgum-2.0.0-x86_64-pc-windows-msvc.zip",
      "hash": "<sha256>"
    },
    "arm64": {
      "url": "...aarch64...zip",
      "hash": "<sha256>"
    }
  },
  "bin": "forgum.exe",
  "post_install": "forgum init --first-run --yes",
  "persist": "config"
}
```

Install: `scoop bucket add forgum https://github.com/forgum/scoop-forgum && scoop install forgum`. The `persist` key keeps config across upgrades — scoop manages the symlink to `%APPDATA%\forgum`.

### 3.3 MSI (Windows, P2) — WiX

`cargo wix` generates the `.msi` from a `wix/main.wxs` template. Features: per-user install (no admin), adds `forgum.exe` to PATH, runs `forgum init --first-run` via a custom action, writes an uninstall entry visible in "Add/Remove Programs." Signed with an EV cert (P2). The MSI is the canonical artifact winget references.

### 3.4 .deb (Debian/Ubuntu, P1) — cargo-deb

`Cargo.toml` `[package.metadata.deb]` section:

```toml
[package.metadata.deb]
maintainer = "Forgum maintainers <maintainers@forgum.dev>"
license-file = ["LICENSE", "0"]
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
  ["target/release/forgum", "usr/bin/", "755"],
  ["man/forgum.1", "usr/share/man/man1/", "644"],
  ["completions/forgum.bash", "usr/share/bash-completion/completions/forgum", "644"],
  ["completions/forgum.zsh", "usr/share/zsh/vendor-completions/_forgum", "644"],
  ["completions/forgum.fish", "usr/share/fish/vendor_completions.d/forgum.fish", "644"],
]
```

CI builds `forgum_2.0.0_amd64.deb` + `_arm64.deb`. Postinst runs `forgum init --first-run --yes --no-shell-hook` (system install; the user runs `forgum init` themselves to pick their shell). The apt repo (P3) is a simple reprepro-managed repo at `apt.forgum.dev` with a GPG-signed key.

### 3.5 .rpm (Fedora, P1) — cargo-generate-rpm

```toml
[package.metadata.generate-rpm]
assets = [
  { source = "target/release/forgum", dest = "/usr/bin/forgum", mode = "755" },
  { source = "man/forgum.1", dest = "/usr/share/man/man1/forgum.1", mode = "644" },
  # completions...
]
```

Produces `forgum-2.0.0-1.x86_64.rpm` + `aarch64.rpm`. COPR (P3) builds from the SRPM.

### 3.6 AUR (Arch, P2)

Two packages, maintained by a community member (or us):
- `forgum-bin` — fetches the prebuilt binary from the GitHub release (PKGBUILD with `source=...releases/.../forgum-...tar.gz`).
- `forgum-git` — builds from `main` (for contributors).

PKGBUILD follows Arch standards: `pkgname`, `pkgver`, `sha256sums`, `package()` installs to `/usr/bin`, completions to `/usr/share/{bash-completion,zsh,fish}/…`. The AUR is updated via SSH push from CI on release.

### 3.7 Nix (any Linux, P2)

In-repo `flake.nix`:

```nix
{
  description = "Forgum — cross-platform cowsay+fortune+lolcat with a Rust engine";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in {
        packages.forgum = pkgs.rustPlatform.buildRustPackage {
          pname = "forgum"; version = "2.0.0";
          src = ./.; cargoLock.lockFile = ./Cargo.lock;
          buildInputs = with pkgs; [ /* openssl, etc. */ ];
        };
        packages.default = self.packages.${system}.forgum;
        devShells.default = pkgs.mkShell { buildInputs = with pkgs; [ rustc cargo just ]; };
      });
}
```

Usage: `nix run github:harish2222/Forgum`. After stability, a PR to `nixpkgs` adds `forgum` to `pkgs/by-name/fo/forgum/`. Nix users get reproducible builds and per-user profiles — no PATH conflicts because Nix's wrapper handles it.

### 3.8 Homebrew (macOS, P1)

Our tap `forgum/homebrew-forgum` contains `Formula/forgum.rb`:

```ruby
class Forgum < Formula
  desc "Cross-platform cowsay + fortune + lolcat with a Rust animation engine"
  homepage "https://github.com/harish2222/Forgum"
  url "https://github.com/harish2222/Forgum/releases/download/v2.0.0/forgum-2.0.0-aarch64-apple-darwin.tar.gz"
  sha256 "<sha256>"
  version "2.0.0"
  license "MIT"

  def install
    bin.install "forgum"
    # completions + manpage
  end

  def post_install
    # don't auto-run TUI; user runs `forgum init` themselves
    ohai "Run `forgum init` to configure your shell."
  end

  test do
    assert_match "forgum 2.0.0", shell_output("#{bin}/forgum --version")
  end
end
```

Install: `brew tap forgum/forgum && brew install forgum`. After 30 days of stability + ≥1000 downloads, a PR to `homebrew-core` (the PR requires the formula be in `homebrew/core` format, no `tap` reference, with audit passing).

---

## 4. Shell-hook injection (post-install)

Every install path ends by running (or prompting the user to run) `forgum init`, which injects a **managed block** into the user's shell rc file. The block is delimited and idempotent:

```bash
# >>> forgum >>>
# This block is managed by `forgum init`. Do not edit by hand; run `forgum init` to change.
if command -v forgum >/dev/null 2>&1; then
  forgum init --sh bash --silent   # emits the precmd hook + completions
fi
# <<< forgum <<<
```

`forgum init --sh bash --silent` emits the hook lines to stdout; the block sources them. The block:
- is **re-entrant**: re-running `forgum init` replaces the block contents, never duplicates.
- is **removable**: `forgum init --uninstall` strips the block.
- **detects the shell** (bash/zsh/fish/pwsh) and emits the right syntax — see `04-PROMPT-INTEGRATION.md` for the hook design (the v2 fix for BUG-S1..S4).
- **sweeps dead daemons** on every precmd (the v2 daemon-lifecycle fix).

`forgum init` offers (in the TUI) the choice of "which shell rc to inject into" when multiple are detected (e.g., a user with both `.bashrc` and `.zshrc`).

---

## 5. `forgum doctor` — install integrity

```bash
forgum doctor
```

runs a battery of checks and prints a green/red report:

| Check | What it verifies |
|-------|------------------|
| binary canonical | `current_exe` matches the canonical path for this platform |
| binary shadowing | no other `forgum` earlier on `PATH` |
| version | binary version == manifest version |
| config schema | `config.toml.schema_version` is current (or migratable) |
| config valid | TOML parses, all keys known, no orphan values |
| shell hook | the managed block is present + syntactically valid in the detected rc file(s) |
| shell hook sweep | no stale blocks from old versions |
| completions | installed for the detected shell(s) |
| data dir | `~/.local/share/forgum` (or equivalent) exists + writable |
| cache dir | exists + writable + under size cap |
| models (AI) | if AI on, required models present + SHA-256 verified |
| cows | cow library present, count matches manifest, no corrupt `.cow` files |
| tmux | if installed, the `forgum` tmux plugin is loadable |
| GPU | renderer capability probe result (kitty-graphics/wgpu/ANSI) |

`forgum doctor --fix` auto-resolves everything it safely can (re-inject shell hook, migrate config, prune stale cache). Anything it can't, it prints an actionable message. This is the **single command** support can ask users to run.

---

## 6. Version resilience — the upgrade matrix

The hard problem: a user installed v1.1.2 via scoop, then v2.0.0 via winget, then ran `cargo install forgum`. What happens?

**Resolution rules (in priority order):**

1. **Canonical path wins.** If a binary exists at the canonical path (`%LOCALAPPDATA%\Programs\forgum\forgum.exe`), it is the truth. Others are "shadow sources."
2. **Newer version wins** *only if* no canonical-path binary exists. `forgum doctor` reports which source is active.
3. **`forgum upgrade`** (the subcommand) upgrades *the active method's* binary in place — it does not install a second copy. If you installed via winget, `forgum upgrade` runs `winget upgrade Forgum.Forgum`. If via brew, `brew upgrade`. If via `install.sh`, it re-runs the fetcher. The active method is recorded in `~/.config/forgum/install-source.json`:
   ```json
   { "method": "winget", "version": "2.0.0", "installed_at": "...", "canonical_path": "..." }
   ```
4. **Switching methods** is explicit: `forgum install --switch-to scoop` uninstalls the winget copy (via `winget uninstall`) and installs scoop. Config/data/cache are untouched (they're at canonical paths, not per-method).
5. **Config migration** runs on every startup regardless of method: `schema_version` mismatch → migrate → backup → continue. A v1.1.2 config becomes a v2.0.0 config transparently (the v1 `effect` key is migrated to the v2 `animation.mode` key, fixing BUG-S2 retroactively).
6. **Downgrade is supported but warned.** `forgum install --version 1.1.2` works; on startup the v1.1.2 binary sees a v2.0.0 config, prints *"config is from a newer Forgum (2.0.0); some settings will be ignored; run `forgum doctor`,"* and runs with the subset it understands. No crash, no data loss.

**The test for this:** CI runs a matrix of "install method A v2.0.0, then method B v2.1.0, then method C v1.1.2" across all 3 platforms and asserts (a) exactly one canonical binary at the end, (b) config valid, (c) `forgum doctor` green or green-with-warnings, (d) `forgum` runs.

---

## 7. Config migration ladder

```rust
// crates/config/src/migrate.rs
pub fn migrate(cfg: &mut Value, from: u32, to: u32) -> Result<()> {
    let mut cur = from;
    while cur < to {
        match cur {
            1 => migrate_v1_to_v2(cfg)?,    // v1.1.2 → v2.0.0: effect→animation.mode, etc.
            2 => migrate_v2_to_v2_1(cfg)?,  // adding AI keys with safe defaults
            3 => migrate_v2_1_to_v3(cfg)?,  // adding ai.features map
            _ => return Err(unknown_schema(cur)),
        }
        cur += 1;
    }
    Ok(())
}
```

Each step is pure, deterministic, tested against golden old-config fixtures. A backup `config.toml.bak.{from}_to_{to}` is written before the first step. `forgum config migrate --dry-run` shows the diff. This is the mechanism that lets any version read any older config.

---

## 8. The package-manager PRs (P1–P3)

| Target | Repo | What the PR contains | Review criteria | Maintainer |
|--------|------|----------------------|-----------------|------------|
| winget | `microsoft/winget-pkgs` | 4 YAML manifests (installer/locale/defaultLocale/version) under `manifests/f/Forgum/Forgum/<ver>/` | valid SHA-256, reachable URL, silent install, correct `PackageIdentifier` | bot PR, human approval |
| scoop | our `scoop-forgum` bucket (no upstream PR needed) | `bucket/forgum.json` | n/a (our bucket) | us |
| apt | our `apt.forgum.dev` repo (no upstream PR; PPA optional) | `.deb` + `Release`/`InRelease` (signed) | GPG-signed, `apt-cache policy` clean | us |
| dnf | Fedora COPR `@forgum/forgum` | SRPM | builds in mock, passes `fedora-review` | us (COPR account) |
| brew | `Homebrew/homebrew-core` (after tap stability) | `Formula/f/forgum.rb` | `brew audit --strict` passes, ≥30 days in tap, ≥1000 downloads | community + us |
| pacman | AUR `forgum-bin` / `forgum-git` | PKGBUILD + .SRCINFO | builds in clean chroot, `namcap` clean | AUR user (us) |
| nixpkgs | `NixOS/nixpkgs` | `pkgs/by-name/fo/forgum/package.nix` | builds on `x86_64-linux` + `aarch64-darwin`, `nixpkgs-fmt` clean | us, after flake stability |

**The release CI** opens/updates these PRs automatically on every `v*` tag. Each PR body includes: version, changelog excerpt, SHA-256 table, manual test checklist, link to the release artifacts. A human merges after the package repo's CI passes.

---

## 9. Phased rollout + test gates

| Phase | Scope | Test gate (CI must be green) |
|-------|-------|------------------------------|
| P0 | `install.sh`, `install.ps1`, `cargo install`, build-from-source README, `forgum init --first-run` TUI, `forgum doctor`, shell-hook injection, config migration v1→v2 | install scripts run clean on 3 OSes × 2 arches; `forgum doctor` green post-install; migration golden tests pass |
| P1 | winget PR, scoop bucket, .deb, .rpm, Homebrew tap, `forgum upgrade` per-method | each package installs on a clean VM; `forgum doctor` green; upgrade matrix test (§6) passes |
| P2 | MSI (signed), EXE, AUR (`-bin` + `-git`), Nix flake, `forgum install --switch-to` | MSI installs silently + uninstall is clean; AUR builds in chroot; `nix run` works |
| P3 | apt repo, COPR, pacman custom repo, homebrew-core PR, nixpkgs PR, .pkg | repo publishes signed; `brew audit` passes; nixpkgs builds on 2 systems |

**Every phase's gate includes the cross-method upgrade matrix test** (§6) — that's the test that catches the "works in isolation, breaks on upgrade" class of bugs.

---

## 10. The one-paragraph summary

Forgum installs on Windows (winget, scoop, MSI, EXE, `install.ps1`), macOS (Homebrew, `install.sh`, .pkg), and Linux (.deb/apt, .rpm/dnf, AUR/pacman, Nix flake, `install.sh`, `cargo install`) — and **every method converges on the same canonical binary path and the same shared config/data/cache layout**, so mixing methods or upgrading across versions never breaks. The interactive install scripts are thin fetchers that delegate the actual configuration to `forgum init --first-run`'s TUI (see `17-…`), giving one interactive experience across all platforms. A managed, idempotent shell-hook block is injected post-install; `forgum doctor` verifies integrity; `forgum upgrade` upgrades via the active method; config migration runs automatically on every startup. Package-manager PRs (winget, apt, dnf, brew, pacman, nixpkgs) are opened by release CI and merged after each repo's review. The whole thing is phased P0→P3, each gate including the cross-method upgrade-matrix test that proves version resilience.
