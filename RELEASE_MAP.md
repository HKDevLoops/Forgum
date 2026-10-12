# Forgum Universal Release & Distribution Map (RELEASE_MAP.md)

This document provides the definitive topological graph and execution matrix for releasing Forgum versions. It maps out **what to do**, **how to do it**, **where changes occur**, and **what new capabilities are introduced**.

---

## 1. Release Architecture & Pipeline Flowchart

```mermaid
flowchart TD
    subgraph S1["Stage 1: Pre-Flight Verification"]
        A1["Codebase Audit & Formatting\ncargo fmt --check"] --> A2["Zero-Warning Lint Gate\ncargo clippy --workspace --all-targets -- -D warnings"]
        A2 --> A3["Test Suite Verification\ncargo test -p forgum-platform\ncargo test -p forgum-engine\ncargo test -p forgum-tui"]
        A3 --> A4["Adversarial Invariant Audit\n(132 mascots, zero CP437, <100MB RAM)"]
    end

    subgraph S2["Stage 2: Multi-Manifest & Installer Synchronization"]
        B1["Cargo Workspace\nCargo.toml: [workspace.package] version\ncrates/platform/src/lib.rs: html_root_url"]
        B2["Windows Package Managers\nScoop: packaging/scoop/forgum.json\nWinget: packaging/winget/HKDevLoops.Forgum.yaml\nChocolatey: packaging/choco/forgum.nuspec\nChoco Helper: packaging/choco/tools/chocolateyinstall.ps1\nWiX MSI: packaging/windows/forgum.wxs"]
        B3["Linux & BSD Package Managers\nDebian: packaging/deb/DEBIAN/control\nRPM: packaging/rpm/forgum.spec\nPacman: packaging/pacman/PKGBUILD\nNix: packaging/nix/flake.nix & package.nix\nGentoo: packaging/gentoo/forgum-0.0.6.ebuild\nGuix: packaging/guix/forgum.scm\nHomebrew: packaging/homebrew/forgum.rb"]
        B4["Universal Installers & PowerShell Module\nPowerShell Module: Forgum.psd1 (ModuleVersion)\nWindows Installer: install.ps1 ($Version fallback)\nUnix/macOS Installer: install.sh (resolve_version fallback)"]
    end

    subgraph S3["Stage 3: Documentation, Guides & Ledgers Sync"]
        C1["Primary Repository Documentation\nREADME.md: Header version badge & TUI Configurator preview\nCHANGELOG.md: Keep-a-Changelog release section\nARCHITECTURE_MAP.md: Milestone taxonomy & capabilities"]
        C2["Sub-Guides & Samples\ndocs/samples/README.md: SceneConfig schema validator\ndocs/CHANNELS.md: Channel matrix & status output\ndocs/GUIDE.md: Diagnostic checkhealth sample output"]
        C3["Packaging & Uninstallation Documentation\npackaging/windows/README.md: Portable zip asset name\npackaging/gentoo/README.md: Ebuild filename & testing steps\npackaging/UNINSTALL.md: Overlay ebuild unmerge command"]
        C4["Defect Ledger & Flight Protocol\nFIX_LOGS.md: Target Release header & defect registers\nRELEASE_SOP.md: Release SOP stage examples & tag command\nRELEASE_MAP.md: Definitive distribution graph & change matrix"]
    end

    subgraph S4["Stage 4: Git Branch Promotion & Tagging"]
        D1["Atomic Commit on dev\nchore(release): bump version to 0.0.6-beta"] --> D2["Push dev to origin\ngit push origin dev"]
        D2 --> D3["Fast-Forward main\ngit checkout main && git merge --ff-only dev"]
        D3 --> D4["Push main to origin\ngit push origin main"]
        D4 --> D5["Checkout dev\ngit checkout dev"]
        D5 --> D6["Create Annotated Release Tag\ngit tag -a v0.0.6-beta -m 'Release v0.0.6-beta'"]
        D6 --> D7["Push Release Tag\ngit push origin v0.0.6-beta"]
    end

    subgraph S5["Stage 5: CI/CD Pipeline Monitoring"]
        E1["GitHub Actions Trigger\n.github/workflows/release.yml"] --> E2["Build 8 Target Triples\n(Linux gnu/musl, macOS darwin, Windows msvc)"]
        E2 --> E3["Package Distributions\n(.rpm, .deb, .msi, .nupkg, .zip, .tar.gz)"]
        E3 --> E4["Cryptographic Checksums\nsha256sums.txt generation"]
        E4 --> E5["Publish GitHub Release Assets\ngh release view v0.0.6-beta"]
    end

    S1 --> S2
    S2 --> S3
    S3 --> S4
    S4 --> S5
```

---

## 2. Release Manifest & File Change Matrix

| Component / Channel | File Path | Field / Variable | Old Value (`0.0.4-beta`) | New Value (`0.0.6-beta`) |
| :--- | :--- | :--- | :--- | :--- |
| **Cargo Workspace** | `Cargo.toml` | `[workspace.package].version` | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Platform Docs** | `crates/platform/src/lib.rs` | `#![doc(html_root_url = ...)]` | `.../0.0.3-beta` | `.../0.0.6-beta` |
| **Primary README** | `README.md` | Line 31 (Version badge) & Line 811 (TUI header) | `0.0.4-beta` | `0.0.6-beta` |
| **Changelog** | `CHANGELOG.md` | Release header & release date | `## [0.0.4-beta]` | `## [0.0.6-beta] - 2026-10-12` |
| **Defect Ledger** | `FIX_LOGS.md` | Line 3 (`Target Release`) & defect entries | `v0.0.4-beta` | `v0.0.6-beta` (BUG-027 through BUG-030, FEATURE-029, FEATURE-030) |
| **Architecture Map** | `ARCHITECTURE_MAP.md` | Section 1.2 header & overview | `(0.0.4-beta)` | `(0.0.6-beta)` |
| **Release SOP** | `RELEASE_SOP.md` | Lines 13, 67, 72–88, 96, 119, 152 | `0.0.4-beta` | `0.0.6-beta` |
| **PowerShell Module** | `Forgum.psd1` | Line 3 (`ModuleVersion`) | `'0.0.3'` | `'0.0.6'` |
| **Windows Installer** | `install.ps1` | Line 213 (`$Version = ...` fallback) | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Unix / macOS Installer** | `install.sh` | Line 17 (env comment) & Line 553 (`echo ...` fallback) | `0.0.4-beta` | `0.0.6-beta` |
| **Scoop Manifest** | `packaging/scoop/forgum.json` | `"version"` & download URLs | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Winget Manifest** | `packaging/winget/HKDevLoops.Forgum.yaml` | `PackageVersion` & URLs | `0.0.4-beta` | `0.0.6-beta` |
| **Chocolatey Nuspec** | `packaging/choco/forgum.nuspec` | `<version>` | `0.0.4-beta` | `0.0.6-beta` |
| **Chocolatey Helper** | `packaging/choco/tools/chocolateyinstall.ps1` | Line 6 (`$version = ...`) | `'0.0.4-beta'` | `'0.0.6-beta'` |
| **Homebrew Formula** | `packaging/homebrew/forgum.rb` | `version` & release URL | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Debian (APT)** | `packaging/deb/DEBIAN/control` | `Version:` | `0.0.4-beta` | `0.0.6-beta` |
| **Red Hat (RPM)** | `packaging/rpm/forgum.spec` | `Version:` & changelog block | `0.0.4` | `0.0.6` |
| **Arch Linux** | `packaging/pacman/PKGBUILD` | `pkgver` (normalized pacman format) | `0.0.4.beta` | `0.0.6.beta` |
| **Nix Flake** | `packaging/nix/flake.nix` | `version = ...` | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Nix Package** | `packaging/nix/package.nix` | `version = ...` | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **GNU Guix** | `packaging/guix/forgum.scm` | `(version "...")` | `"0.0.4-beta"` | `"0.0.6-beta"` |
| **Windows MSI (WiX)** | `packaging/windows/forgum.wxs` | `Version="..."` | `0.0.4` | `0.0.6` |
| **Gentoo Ebuild** | `packaging/gentoo/forgum-0.0.6.ebuild` | Ebuild filename | `forgum-0.0.4.ebuild` | `forgum-0.0.6.ebuild` |
| **Gentoo Packaging Guide** | `packaging/gentoo/README.md` | Lines 13, 27, 32 (ebuild testing commands) | `forgum-0.0.3.ebuild` | `forgum-0.0.6.ebuild` |
| **Windows Packaging Guide** | `packaging/windows/README.md` | Line 37 (portable zip asset reference) | `forgum-0.0.3-beta` | `forgum-0.0.6-beta` |
| **Uninstall Guide** | `packaging/UNINSTALL.md` | Line 146 (Gentoo overlay unmerge command) | `forgum-0.0.3.ebuild` | `forgum-0.0.6.ebuild` |
| **Sample Configs Guide** | `docs/samples/README.md` | Line 7 (SceneConfig schema version validator) | `v0.0.4-beta` | `v0.0.6-beta` |
| **Release Channels Guide** | `docs/CHANNELS.md` | Lines 51 & 73 (Stable channel tag & CLI output) | `0.0.3-beta` | `0.0.6-beta` |
| **Diagnostics & User Guide** | `docs/GUIDE.md` | Line 263 (`checkhealth` sample output header) | `v0.0.3-beta` | `v0.0.6-beta` |
| **Distribution Map** | `RELEASE_MAP.md` | Flowchart, sync matrix & capability registers | Full 0.0.4 matrix | Complete 29-target 0.0.6 matrix |

---

## 3. What New Capabilities Are Bundled in `0.0.6-beta`

1. **CP437 Mojibake Elimination (BUG-021 & BUG-023)**:
   - Dynamic particle pools, instinct signature emitters, talk vocals, and glitch effects sanitized to 100% pure ASCII.
   - Invariant tests asserting zero CP437 box-drawing collisions (`Γöé`, `Γòæ`, `òæ`).

2. **Universal UTF & Unicode Compatibility Engine (BUG-024)**:
   - Module `crates/platform/src/encoding.rs` transparently decodes UTF-8, UTF-8 with BOM (`0xEF 0xBB 0xBF`), UTF-16 LE/BE, and UTF-32 LE/BE.
   - Sanitizes Trojan Source BiDi overrides (CVE-2021-42574) and normalizes CRLF endings to LF.
   - Wired into mascot `.cow` loaders, fortunes, configuration files, and piped stdin.

3. **NPU Priority in Auto Acceleration Mode (FEATURE-025)**:
   - In `AccelerationMode::Auto`, hardware resolution prioritizes detected NPUs (`Intel AI Boost`, `AMD XDNA`, `Apple Neural Engine`, `Qualcomm Hexagon`) over discrete GPUs.
   - Offloads rendering math completely from CPU while sparing discrete GPU compute queues.

4. **Dynamic Animation & Kinematics GPU/NPU Offload (FEATURE-026)**:
   - Offloaded cartoon spring-damper dialogue bubble inertia (`compute_dynamic_bubble_inertia`), Lissajous trajectory math (`compute_dynamic_kinematics_step`), and underwater bubble turbulence to `GpuComputeEngine`.
   - Operates with zero heap allocations per frame in the render loop, strictly preserving the <100MB resident RAM ceiling.

5. **Full Stylized Color Theme Modes**:
   - Added OKLCH palette gradients for curated color modes: `cyberpunk`, `autumn`, `pastel`, `matrix`, `monochrome`, `sunset`, `ocean`, and `fire`.
   - Enforced Mandate K biological coat morph palette lock when `Color Mode != "natural"`.

6. **High-Entropy SplitMix64 PRNG & Dynamic Incommensurate Particle Showering (FEATURE-029)**:
   - 64-bit SplitMix64 PRNG with Golden Ratio Weyl constant ($\Phi = 0x9E3779B97F4A7C15$), eliminating degenerate 0 fixed-point collapses.
   - Box-Muller Gaussian normal sampling (`rand_normal`) and incommensurate dual harmonic turbulence ($\omega_2 = \omega_1 \cdot 1.6180339887$).
   - Per-particle lateral aerodynamic micro-turbulence in `eval_particle_chunk`.

7. **Elimination of Alphabetic Leg Glyphs in Richard Williams Gait (BUG-028)**:
   - Replaced alphabetic ASCII characters (`'J'`, `'L'`, `'d'`, `'b'`) in `articulated_leg_glyph` with authentic limb strokes (`'/'`, `'\\'`, `'|'`, `')'`, `'('`).

8. **Multi-Pane & Split-Mode Alignment Calibration (BUG-027)**:
   - Formally documented alignment calibration for multi-pane multiplexer environments in defect ledger for scheduled deep focus.

9. **openSUSE WSL Shell Hook Integer Parsing Elimination (BUG-030)**:
   - Replaced brittle `awk` field splits with portable POSIX `sed` regex for unquoted JSON integer parsing in daemon state files.
   - Guarded integer evaluations with `${rows:-0}` parameter expansion, completely eliminating `-bash: [: : integer expression expected` on login, clear, and prompts.

10. **Remote Channel Up-to-Date Synchronization & Brutal Mascot Disgrace Radar (FEATURE-030)**:
    - Added release channel version checking against GitHub raw CDN: skips redundant upgrades with `Already on latest version of Forgum` and celebratory mascot quotes.
    - Added the Mascot Disgrace Radar to taunt consecutive user syntax errors (>2 failures) with brutal meme disgrace statements and escalating ASCII mascot reaction faces.

---

## 4. Execution Commands Quick Reference

```powershell
# 1. Quality Gates
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p forgum-platform
cargo test -p forgum-engine --test adversarial_invariants
cargo test -p forgum-tui

# 2. Git Promotion & Tagging
git add -A
git commit -m "chore(release): bump version to 0.0.6-beta with Unicode compatibility, NPU priority, and dynamic kinematics offload"
git push origin dev
git checkout main
git merge --ff-only dev
git push origin main
git checkout dev
git tag -a v0.0.6-beta -m "Release v0.0.6-beta: CP437 ASCII sanitization, universal Unicode compatibility engine, NPU auto priority, dynamic kinematics GPU compute offload, and full color themes"
git push origin v0.0.6-beta

# 3. CI/CD Monitoring
gh run list --workflow=release.yml --limit 3
gh run watch <run-id>
```
