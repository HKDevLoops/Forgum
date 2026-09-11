# Anonymous Motivation & Curiosity Telemetry in Forgum

## 1. Philosophy & Core Purpose

Forgum is an independent, free, and open-source project built with passion for the terminal and ASCII art community.

The telemetry system inside Forgum was created for **one reason only: developer motivation and curious delight**. As creators of open-source tools, knowing that real human beings across the world are trying, installing, and enjoying the software inspires continuous maintenance, bugfixes, and artistic development.

> [!IMPORTANT]
> **Zero Surveillance Guarantee**:
> Forgum's telemetry is strictly **NOT** designed to monitor, profile, track, or harvest data from users.
> - **ZERO Personal Data**: No usernames, no hostnames, no machine GUIDs, no MAC addresses.
> - **ZERO IP Logging**: CounterAPI receives only an anonymous hit incrementing a public counter.
> - **ZERO Background Daemons**: No background processes or surveillance daemons are ever spawned.
> - **ZERO Blocking**: Network pings are dispatched on a detached background thread with a strict 1.0-second timeout. If you are offline, behind a corporate firewall, or on an air-gapped machine, Forgum proceeds immediately with zero warnings or lag.

---

## 2. The Three Anonymous Aggregate Counters

Forgum tracks only **three aggregate public counters**, displayed live on the project [README.md](../README.md):

| Counter | Metric Key | Trigger Event | Purpose |
|---|---|---|---|
| **Users Tried** | `users_tried` | Launched when the installation wizard opens or when a user tries Forgum | Tells the author how many developers explored or tested the project, even if they decided not to install or declined telemetry. |
| **Installations** | `users_installed` | Triggered upon successful completion of installation with user consent | Celebrates every user who adopted Forgum into their daily shell workflow. |
| **Active Users** | `active_users` | Daily anonymous heartbeat (throttled to at most once per 24 hours) | Shows how many people actively enjoy Forgum as part of their prompt experience each day. |

---

## 3. Transparent User Consent in the Setup Wizard

When you launch the Celestial Installation Wizard (`install.ps1`, `install.sh`, or `forgum install`), you are greeted by an explicit, transparent permission screen:

```
  ✦ MOTIVATION & CURIOSITY TELEMETRY NOTICE ✦
  Forgum includes an anonymous aggregate counter purely for developer motivation.
  
  [Y] Allow Motivation Counter (+1 to GitHub README badge)
  [N] Decline (Keep strictly offline & private)
```

### What happens when you choose "No" (Decline)?
- If you select **[N] Decline**, the telemetry preference is saved as disabled (`false`).
- Only the anonymous `users_tried` counter is registered once (letting us know someone explored Forgum while respecting their desire for total offline privacy).
- **No** installation counter is incremented.
- **No** daily active pulses are ever dispatched.
- Forgum operates in 100% offline mode.

---

## 4. How to Opt-Out Anytime

If you previously consented and wish to disable telemetry, you can do so instantly via any of the following methods:

### Method A: Forgum Configuration Command
```bash
forgum config set telemetry false
```

### Method B: Environment Variable Override
Set `FORGUM_TELEMETRY=0` in your shell profile:
```bash
# In ~/.bashrc, ~/.zshrc, or config.fish:
export FORGUM_TELEMETRY=0
```
```powershell
# In PowerShell profile:
$env:FORGUM_TELEMETRY = "0"
```

### Method C: Headless Installation Flag
```bash
./install.sh --headless --telemetry decline
```
```powershell
./install.ps1 -Headless -Telemetry decline
```

---

## 5. Live Badges

The aggregate counters are publicly visible on shields.io badges:

- **Users Tried**:  
  `https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.counterapi.dev%2Fv1%2Fforgum%2Fusers_tried&query=%24.count&label=Users%20Tried&color=blueviolet&style=for-the-badge&logo=starship`
- **Installations**:  
  `https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.counterapi.dev%2Fv1%2Fforgum%2Fusers_installed&query=%24.count&label=Installations&color=00F2FE&style=for-the-badge&logo=spacex`
- **Active Users**:  
  `https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fapi.counterapi.dev%2Fv1%2Fforgum%2Factive_users&query=%24.count&label=Active%20Users&color=F59E0B&style=for-the-badge&logo=sparkles`
