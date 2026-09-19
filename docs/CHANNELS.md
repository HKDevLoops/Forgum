# 🌊 Forgum Release Channels & Multi-Channel Architecture

This guide details Forgum's 3-branch release architecture, how to switch release channels, and how Forgum resolves dual installations and cross-package-manager conflicts.

---

## 📑 Table of Contents
1. [The 3-Channel Architecture](#1-the-3-channel-architecture)
2. [Channel Comparison Matrix](#2-channel-comparison-matrix)
3. [Switching Channels via CLI](#3-switching-channels-via-cli)
4. [Cross-Package-Manager Conflict Resolution](#4-cross-package-manager-conflict-resolution)
   - [Scenario A: Standalone Script followed by Scoop/Brew](#scenario-a-standalone-script-followed-by-scoopbrew)
   - [Scenario B: Scoop/Brew followed by Standalone Install](#scenario-b-scoopbrew-followed-by-standalone-install)
5. [Automatic Shadow Installation Detection](#5-automatic-shadow-installation-detection)
6. [Safe Atomic Updates & Rollback](#6-safe-atomic-updates--rollback)

---

## 1. The 3-Channel Architecture

Forgum provides three distinct release branches to suit different stability and development needs:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. dev (Maintainer & Contributor Branch)                   │
│    - PR landing target for all active development.          │
│    - Bleeding-edge features, kinematics, and physics models.│
└──────────────────────────────┬──────────────────────────────┘
                               │ (Automated Daily CI/CD at 02:00 UTC)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. nightly (Early-Access Channel)                           │
│    - Automated pre-release binary rolling builds.           │
│    - Ideal for power users who want fresh updates.          │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Tagged Stable Release)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. main (Public Stable Channel)                             │
│    - Recommended for all general users and production.      │
│    - Distributed via Homebrew, Scoop, Winget, Pacman, etc.  │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Channel Comparison Matrix

| Channel | Recommended Audience | Release Cadence | Target Branch | Update Command |
| :--- | :--- | :--- | :--- | :--- |
| **`main`** (Stable) | All users, default installations | Semantic version tags (`v0.4.1`) | `main` | `forgum update --channel stable` |
| **`nightly`** (Early Access) | Testers, power users, theme creators | Daily rolling release | `nightly` | `forgum update --channel nightly` |
| **`dev`** (Bleeding Edge) | Maintainers & active PR contributors | On git commit | `dev` | `forgum channel set dev` |

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
Version:             0.0.1-alpha.1
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

## 4. Cross-Package-Manager Conflict Resolution

A common problem in developer workstations is installing a tool via an install script, and later running `scoop install forgum` (or `brew install forgum`), leading to multiple versions shadowing each other in `$PATH`.

Forgum handles this with **State Tracking Receipts** and **PATH Shadow Auditing**:

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
     - To use Scoop exclusively: run `forgum uninstall --soft`
     - To use Standalone exclusively: run `scoop uninstall forgum`
     ```

### Scenario B: Scoop/Brew followed by Standalone Install
1. User installed via Scoop (`~/scoop/apps/forgum`).
2. User later runs `./install.ps1` from the repository to try out a nightly or dev build.
3. **Forgum's Resolution**:
   - `install.ps1` checks for active package managers before installing.
   - It prompts:
     `"Existing installation detected via Scoop. Would you like to delegate to Scoop or install a Standalone Channel Build?"`
   - If Standalone is chosen, Forgum records an installation receipt (`.config/forgum/install_receipt.json`) marking the override so `forgum update` knows which binary to manage.

---

## 5. Automatic Shadow Installation Detection

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
