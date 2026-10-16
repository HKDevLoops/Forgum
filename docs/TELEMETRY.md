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

## 5. Local Anonymous Statistics (`stats.json`)

Starting in `0.0.8-beta`, Forgum records an **isolated, local-only tally** stored on your machine in:
- **Linux/macOS**: `~/.config/forgum/stats.json`
- **Windows**: `%APPDATA%\forgum\stats.json`

This file is 100% private and never transmitted anywhere. It provides an offline summary of your local client interactions:
```json
{
  "users_tried_count": 1,
  "is_installed": true,
  "active_pulses_count": 14,
  "last_pulse_unix": 1729080000,
  "first_seen_unix": 1727800000
}
```

---

## 6. Custom Telemetry Endpoints (`FORGUM_TELEMETRY_ENDPOINT`)

You can redirect anonymous pulses to your own private proxy or metrics aggregator using the `FORGUM_TELEMETRY_ENDPOINT` environment variable.

Supported patterns:
- **Interpolated**: `export FORGUM_TELEMETRY_ENDPOINT="https://telemetry.yourdomain.com/v1/pulse/{metric}"`
- **Path appended**: `export FORGUM_TELEMETRY_ENDPOINT="https://telemetry.yourdomain.com/v1/pulse"` (automatically appends `/{metric}`)

---

## 7. Recommended Modern Stats Architectures for Open-Source CLI Tools

Collecting usage metrics for an open-source binary terminal tool without violating user privacy or relying on fragile hobby mirrors requires careful architecture. The 3 best industry-standard patterns:

### Tier 1: Canonical GitHub Releases Analytics (Immediate Best Practice)
- **Mechanism**: GitHub natively increments `download_count` for all release assets (`.tar.gz`, `.zip`, `.deb`, `.rpm`, `.exe`).
- **Shields.io Badge**: `https://img.shields.io/github/downloads/HKDevLoops/Forgum/total?style=for-the-badge&logo=github&color=blueviolet&label=Downloads`
- **Why it is superior**: 100% uptime guaranteed by GitHub CDN, zero external servers, completely tamper-resistant against script spam, and never blocked by browser or DNS adblockers (Pi-hole, NextDNS).

### Tier 2: Serverless Cloudflare Worker + Edge KV (For Active Daily Heartbeats)
If real-time daily active user pulses are desired, deploy a zero-cost Cloudflare Worker (100,000 free requests/day):
- **Worker Code**:
  ```javascript
  export default {
    async fetch(request, env) {
      const url = new URL(request.url);
      const metric = url.pathname.split("/").pop();
      if (!["users_tried", "users_installed", "active_users"].includes(metric)) {
        return new Response("Invalid metric", { status: 400 });
      }
      // Increment counter in Cloudflare KV without logging IP
      const current = (await env.STATS_KV.get(metric)) || 0;
      const updated = Number(current) + 1;
      await env.STATS_KV.put(metric, updated.toString());

      return new Response(JSON.stringify({ schemaVersion: 1, label: metric, message: updated.toString(), color: "blue" }), {
        headers: { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" }
      });
    }
  };
  ```
- **Shields.io Endpoint**: `https://img.shields.io/endpoint?url=https%3A%2F%2Fyour-worker.workers.dev%2Factive_users`

### Tier 3: GitOps Daily Aggregator via GitHub Actions (The Private Metrics Cache)
A GitHub Actions cron job runs daily at `00:00 UTC`, queries GitHub Releases API and crates.io API, aggregates download totals, and commits an immutable `metrics/stats.json` snapshot to GitHub Pages or repository. Badges consume the cached snapshot with 100% reliability.

---

## 8. Live Badges

The aggregate counters and official downloads are publicly visible on shields.io badges:

- **GitHub Total Downloads**:  
  `https://img.shields.io/github/downloads/HKDevLoops/Forgum/total?style=for-the-badge&logo=github&color=blueviolet&label=Downloads`
- **Latest Release**:  
  `https://img.shields.io/github/v/release/HKDevLoops/Forgum?include_prereleases&style=for-the-badge&logo=rust&color=00F2FE&label=Release`
- **Users Explored**:  
  `https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fcountapi.mileshilliard.com%2Fapi%2Fv1%2Fget%2Fhkdevloops_forgum_users_tried&query=%24.value&label=Users%20Tried&color=F59E0B&style=for-the-badge&logo=starship`
- **Privacy Guarantee**:  
  `https://img.shields.io/badge/Telemetry-100%25%20Transparent%20%26%20Private-success?style=for-the-badge&logo=shield`
