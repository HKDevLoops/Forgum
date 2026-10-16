# 🌊 Forgum Release Channels & Multi-Channel Architecture

This guide details Forgum's 4-channel release architecture, how to switch release channels, and how Forgum resolves dual installations and cross-package-manager conflicts.

---

## 📑 Table of Contents
1. [The 4-Channel Architecture](#1-the-4-channel-architecture)
2. [Channel Comparison Matrix](#2-channel-comparison-matrix)
3. [Switching Channels via CLI](#3-switching-channels-via-cli)
4. [Cross-Package-Manager Conflict Resolution & Migration](#4-cross-package-manager-conflict-resolution--migration)
   - [Scenario A: Standalone Script followed by Scoop/Brew](#scenario-a-standalone-script-followed-by-scoopbrew)
   - [Scenario B: Scoop/Brew followed by Standalone Install](#scenario-b-scoopbrew-followed-by-standalone-install)
   - [Automated Migration Reconciliation (`forgum update --method <pm>`)](#automated-migration-reconciliation-forgum-update---method-pm)
5. [Automatic Shadow Detection & Duplicate Pruning (`forgum prune`)](#5-automatic-shadow-detection--duplicate-pruning-forgum-prune)
   - [The WSL Host Executable Exemption](#the-wsl-host-executable-exemption)
6. [Safe Atomic Updates & Rollback](#6-safe-atomic-updates--rollback)
7. [Cryptographic Verification & GitHub Artifact Attestation](#7-cryptographic-verification--github-artifact-attestation)

---

## 1. The 4-Channel Architecture

Forgum provides four distinct release channels to suit different stability and development needs:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. dev (Local & Contributor Channel)                        │
│    - PR landing target for all active development.          │
│    - Bleeding-edge features, kinematics, and physics models.│
└──────────────────────────────┬──────────────────────────────┘
                               │ (Automated Daily CI/CD at 02:00 UTC)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. nightly (Bleeding Edge Rolling Channel)                  │
│    - Automated pre-release binary rolling builds.           │
│    - Ideal for power users who want daily fresh updates.    │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Candidate Validation)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. alpha (Early Testing & Preview Channel)                  │
│    - Pre-release testing builds before tagging on main.     │
│    - Validates stability, animations, and GPU acceleration. │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Tagged Release)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. stable (Public Stable Channel - main branch)             │
│    - Recommended for all general users and production.      │
│    - Distributed via Homebrew, Scoop, Winget, Pacman, etc.  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Channel Comparison Matrix

| Channel | Recommended Audience | Release Cadence | Target Branch | Update Command |
| :--- | :--- | :--- | :--- | :--- |
| **`stable`** (Main) | All users, default installations | Semantic version tags (`v0.0.9-beta`) | `main` | `forgum update --channel stable` |
| **`alpha`** (Preview) | Early testing, canary validations | Pre-release release candidates | `dev` | `forgum update --channel alpha` |
| **`nightly`** (Bleeding Edge) | Testers, power users, theme creators | Daily rolling release | `nightly` | `forgum update --channel nightly` |
| **`dev`** (Local/Git) | Maintainers & active PR contributors | On git commit | `dev` | `forgum channel set dev` |

---

## 3. Switching Channels via CLI

You can inspect and switch release channels dynamically from within Forgum:

### View Current Channel & Installation Source
```bash
forgum channel
# or
forgum status
```
*Sample Output:*
```text
━━━ Forgum Release Channel ━━━
Active Channel:      stable (main)
Installation Source: Standalone Binary
Binary Location:     C:\Users\haris\AppData\Local\Forgum\forgum.exe
Version:             0.0.9-beta
```

### Switch to Nightly Channel
```bash
forgum update --channel nightly
```
Forgum safely fetches the rolling pre-release binary, validates executable integrity, and atomically replaces the active binary.

### Switch to Stable Channel
```bash
forgum update --channel stable
```

---

## 4. Cross-Package-Manager Conflict Resolution & Migration

A common problem in developer workstations is installing a tool via an install script, and later running `scoop install forgum` (or `brew install forgum`), leading to multiple versions shadowing each other in `$PATH`.

Forgum handles this with **State Tracking Receipts**, **PATH Shadow Auditing**, and **Automated Migration Reconciliation**:

### Scenario A: Standalone Script followed by Scoop/Brew
1. User installs via `install.ps1`, placing `forgum.exe` in `%LOCALAPPDATA%\Forgum`.
2. User later runs `scoop install forgum`, which creates a shim in `~/scoop/shims/forgum.exe`.
3. If `%LOCALAPPDATA%\Forgum` precedes `~/scoop/shims` in `PATH`, running `forgum` executes the standalone binary rather than the Scoop build!
4. **Forgum's Resolution**:
   - `forgum doctor` and `forgum update` execute multi-path shadow detection.
   - Upon detecting a shadow, Forgum prints:
     ```text
     ⚠️ Dual Installation Warning:
     Active Binary:   C:\Users\user\AppData\Local\Forgum\forgum.exe (Standalone)
     Shadowed Binary: C:\Users\user\scoop\shims\forgum.exe (Scoop)

     Suggested Remediation:
     - To use Scoop exclusively: run `forgum update --method scoop` or `forgum prune`
     - To remove duplicates safely: run `forgum prune`
     ```

### Scenario B: Scoop/Brew followed by Standalone Install
1. User installed via Scoop (`~/scoop/apps/forgum`).
2. User later runs `./install.ps1` from the repository to try out a nightly or dev build.
3. **Forgum's Resolution**:
   - `install.ps1` checks for active package managers before installing.
   - It prompts:
     `"Existing installation detected via Scoop. Would you like to delegate to Scoop or install a Standalone Channel Build?"`
   - If Standalone is chosen, Forgum records an installation receipt (`.config/forgum/install_receipt.json`) marking the override so `forgum update` knows which binary to manage.

### Automated Migration Reconciliation (`forgum update --method <pm>`)
When migrating from a standalone install to a package manager:
```bash
forgum update --method scoop      # Windows Scoop
forgum update --method winget     # Windows WinGet
forgum update --method brew       # macOS / Linux Homebrew
forgum update --method pacman     # Arch Linux
```
Forgum safely backs up the active standalone binary to a timestamped backup directory (`forgum.exe.pre-reconcile-*`), updates the installation receipt to mark the target package manager, and prompts the user so the package manager installation becomes immediately authoritative without path conflicts.

---

## 5. Automatic Shadow Detection & Duplicate Pruning (`forgum prune`)

If multiple Forgum binaries exist across PATH directories, running `forgum prune` detects and cleans inactive/shadowed binaries:

```bash
# Preview shadowed binaries without making changes:
forgum prune --dry-run

# Interactively remove shadowed binaries:
forgum prune

# Non-interactive prune for automation:
forgum prune --yes
```

### The WSL Host Executable Exemption
Under Windows Subsystem for Linux (WSL), the Linux environment inherits Windows host executables (such as `/mnt/c/Users/.../forgum.exe`) into `$PATH` via WSL interop.
Forgum detects when it is running inside WSL via `/proc/version` and kernel telemetry, and **intentionally ignores Windows host binaries (`/mnt/*` and `.exe`) during duplicate detection**. This prevents false positive shadow warnings while ensuring native Linux binaries are properly validated.

Run the diagnostic suite at any time to verify your system integrity:
```bash
forgum doctor
```
Or check the comprehensive JSON report:
```bash
forgum checkhealth --json
```

---

## 6. Safe Atomic Updates & Rollback

Forgum's update engine implements a three-phase atomic swap:

1. **Verification**: The update binary is downloaded to a temporary file (`forgum.tmp`). Its headers, architecture, and `--version` output are verified before touching the host system.
2. **Atomic Swap**: The active binary is moved to `forgum.bak`, and `forgum.tmp` is promoted to `forgum`.
3. **Automatic Rollback**: If the new binary fails validation or panics on first boot, Forgum automatically restores `forgum.bak` back to `forgum`, preventing broken terminal states.

---

## 7. Cryptographic Verification & GitHub Artifact Attestation

Every release asset uploaded to GitHub Releases includes:
- **`SHA256SUMS`**: Authoritative master checksum manifest containing SHA-256 digests for all 20+ platform targets, tarballs, zips, installers, and package manager bundles.
- **`*.sha256`**: Per-artifact checksum files for easy single-file validation.
- **GitHub Artifact Attestation**: Sigstore/SLSA build provenance generated via GitHub Actions `actions/attest-build-provenance@v2`.

Verify downloads:
```bash
# SHA-256 verification:
sha256sum -c forgum-x86_64-unknown-linux-gnu.tar.gz.sha256

# GitHub provenance attestation:
gh attestation verify forgum --owner HKDevLoops
```
