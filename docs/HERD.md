# 🐑 Forgum Herd Management & Daemon Swarm Control

> **Subcommand:** `forgum herd`  
> **Ecosystem:** Terminal Animation Daemons, Split-Shell Overlays, and Multiplexer Fleets  
> **Platform Support:** Windows (Named Pipes `\\.\pipe\forgum-daemon-*`), Linux & macOS (Unix Domain Sockets `$XDG_RUNTIME_DIR/forgum/`)

---

## 1. Executive Architectural Overview

The `forgum herd` subcommand is the central fleet manager and IPC control hub for all active Forgum animation daemons running across your machine. When running in persistent background mode (`--background`, `-b`), split-scroll mode (`--split-scroll`), or shell prompt integration (`auto_render_on_prompt`), each Forgum instance registers a lightweight daemon record in the user runtime directory and binds to a zero-overhead IPC control socket.

`forgum herd` discovers, queries, and directs these running instances in real-time without requiring process restarts, killing terminals, or corrupting shell buffers.

```
                     ┌───────────────────────────┐
                     │     forgum herd CLI       │
                     │  (census / stop / effect) │
                     └─────────────┬─────────────┘
                                   │
              ┌────────────────────┼────────────────────┐
              │ IPC Command Stream │ (JSON over Socket) │
              ▼                    ▼                    ▼
     ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
     │ Daemon 1 (tmux) │  │ Daemon 2 (pwsh) │  │ Daemon 3 (zsh)  │
     │ PID: 10452      │  │ PID: 18420      │  │ PID: 24190      │
     │ Mascot: dragon  │  │ Mascot: tux     │  │ Mascot: vader   │
     │ Effect: breathe │  │ Effect: walk    │  │ Effect: glitch  │
     └─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## 2. Command Synopsis & Targeting Modes

```bash
forgum herd <SUBCOMMAND> [OPTIONS]
```

### Subcommands Matrix

| Subcommand | Arguments | Description |
| :--- | :--- | :--- |
| `census` | *(None)* | Comprehensive swarm inspection: lists PID, session ID, active effect, FPS, speed, status, and age. |
| `list` | *(None)* | Alias for `census`. |
| `stop` | `[--session <ID>] [--all]` | Gracefully terminates target daemons and removes socket handles. |
| `effect` | `<NAME> [--session <ID>] [--all]` | Dynamically hot-swaps active animation effect (`walk`, `breathe`, `float`, `fly`, `glitch`, etc.). |
| `cow` | `<NAME> [--session <ID>] [--all]` | Dynamically hot-swaps animal mascot model (`dragon`, `tux`, `vader`, `corgi`, etc.). |
| `eyes` | `<GLYPHS> [--session <ID>] [--all]` | Updates eye characters in real time (e.g. `oo`, `$$`, `@@`, `xx`, `^^`). |
| `tongue` | `<GLYPH> [--session <ID>] [--all]` | Updates tongue glyph in real time (e.g. `U `, `\|\|`, `  `). |
| `color` | `<MODE> [--session <ID>] [--all]` | Hot-swaps color palette mode (`natural`, `rainbow`, `aurora`, `matrix`, `fire`, `pastel`). |
| `speed` | `<VALUE> [--session <ID>] [--all]` | Adjusts physics update rate multiplier (e.g. `0.5` for half-speed, `2.0` for double-speed). |
| `pause` | `[--session <ID>] [--all]` | Suspends simulation clock and animation updates (drops CPU usage to 0.0%). |
| `resume` | `[--session <ID>] [--all]` | Unfreezes simulation clock and restores active animation loops. |
| `quiet` | *(None)* | Low-power mode: sets all active daemons to 0.1 FPS to conserve laptop battery / CPU cycles. |
| `follow` | `[--pane <ID>]` | Focus-follow mode: gives 100% frame rate to active pane while dropping background panes to 0.1 FPS. |

### Targeting Filters
- `--all`: Broadcasts the instruction to **every** discovered daemon across all terminal windows, multiplexer panes, and background tabs.
- `--session <ID>`: Directs the instruction strictly to the daemon owning the specified session ID (e.g. `shell-30452`).

---

## 3. 22 Real-World CLI & Scripting Examples

### A. Everyday Interactive Command Line One-Liners

#### Example 1: Inspect All Running Daemons (Swarm Census)
Query all background mascots running across your terminal sessions:
```bash
forgum herd census
```
*Sample Output:*
```
PID      SESSION          EFFECT       FPS    SPEED    STATUS   AGE   
----------------------------------------------------------------------
14028    shell-24810      walk         60     1.0      running  12m   
18920    shell-31044      float        30     1.0      running  45s   
24188    shell-10822      glitch       60     0.5      paused   1h    
```

#### Example 2: Instantly Stop All Daemons
Terminate every running background animation instance and clean up all IPC sockets:
```bash
forgum herd stop --all
```

#### Example 3: Hot-Swap Mascot to Dragon Everywhere
Change every open terminal's mascot to `dragon` without restarting shells:
```bash
forgum herd cow dragon --all
```

#### Example 4: Broadcast Matrix Color Mode across Fleet
Apply the green digital rain color style to all active mascots:
```bash
forgum herd color matrix --all
```

#### Example 5: Activate Low-Power "Quiet Mode"
When running intensive compilation or battery saving, drop all mascots to 0.1 FPS:
```bash
forgum herd quiet
```

#### Example 6: Pause and Resume Swarm During Focused Debugging
Freeze all animations during sensitive console reading:
```bash
# Freeze rendering:
forgum herd pause --all

# Resume rendering when done:
forgum herd resume --all
```

---

### B. Multiplexer Fleets (tmux, Zellij, WezTerm)

#### Example 7: Tmux Active Window Follower
Ensure only the currently focused tmux pane animates at full speed, while all background splits idle:
```bash
# Set in ~/.tmux.conf or run interactively:
tmux set-hook -g pane-focus-in "run-shell 'forgum herd follow --pane #{pane_id}'"
```

#### Example 8: Night Mode Circadian Dimming via Cron / Systemd
A shell one-liner triggered at 8:00 PM to dim terminal mascots to pastel colors and half speed:
```bash
forgum herd color pastel --all && forgum herd speed 0.5 --all
```

#### Example 9: Targeted Control by Session ID
Change only a specific split pane without disturbing other terminals:
```bash
# Discover target session ID:
forgum herd list

# Command only session shell-24810:
forgum herd effect glitch --session shell-24810
forgum herd eyes '$$' --session shell-24810
```

#### Example 10: Zellij Tab Switch Hook
Hook into Zellij multiplexer tab switches in `config.kdl` to quiet inactive tabs:
```bash
zellij action run -- forgum herd quiet
```

---

### C. PowerShell Automation Scripts (Windows 11 / Windows Server)

#### Example 11: Build Failure & Success Alarm Script
Automatically signal long `dotnet build` or `cargo build` results using Forgum daemon expressions:
```powershell
# build_watcher.ps1
Write-Host "Compiling production binaries..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    # Victory: Celebration mode with rainbow cow
    forgum herd cow default --all
    forgum herd effect walk --all
    forgum herd color rainbow --all
    forgum herd eyes "^^" --all
    Write-Host "Build succeeded! Mascots celebrating." -ForegroundColor Green
} else {
    # Failure: Inferno dragon with dead eyes
    forgum herd cow dragon --all
    forgum herd effect glitch --all
    forgum herd color fire --all
    forgum herd eyes "xx" --all
    forgum herd tongue "U " --all
    Write-Warning "Build failed! Dragon summoned."
}
```

#### Example 12: High CPU Throttle Guard (PowerShell Daemon)
Monitors system CPU utilization; if CPU exceeds 80%, pauses Forgum animations to prioritize user tasks:
```powershell
# cpu_governor.ps1
while ($true) {
    $cpu = (Get-Counter '\Processor(_Total)\% Processor Time').CounterSamples[0].CookedValue
    if ($cpu -gt 80.0) {
        # High CPU load: throttle or pause
        forgum herd quiet
    } elseif ($cpu -lt 40.0) {
        # Normal CPU: resume standard speed
        forgum herd speed 1.0 --all
    }
    Start-Sleep -Seconds 5
}
```

#### Example 13: Git Dirty Repository Watcher
Changes mascot eyes when working in a dirty git repository:
```powershell
# git_watch.ps1
function Check-GitPasture {
    if (git rev-parse --is-inside-work-tree 2>$null) {
        $status = git status --porcelain
        if ($status) {
            # Uncommitted changes: alert eyes
            forgum herd eyes "@@" --all
        } else {
            # Pristine working tree: peaceful eyes
            forgum herd eyes "==" --all
        }
    }
}
```

#### Example 14: Automated Test Suite Progress Announcer
Updates mascot kinematics based on test suite execution phases:
```powershell
# test_runner.ps1
forgum herd effect breathe --all
forgum herd eyes ".." --all
Write-Host "Running unit tests..."

cargo test --no-run
if ($LASTEXITCODE -eq 0) {
    forgum herd effect float --all
    forgum herd eyes "^^" --all
    Write-Host "All tests compiled cleanly!" -ForegroundColor Green
} else {
    forgum herd effect glitch --all
    forgum herd eyes "XX" --all
}
```

#### Example 15: Battery Saver Hook (Windows Laptop on Battery)
Detects when the laptop unplugged from AC power and throttles daemon frame pacing:
```powershell
# battery_saver.ps1
$battery = Get-CimInstance -ClassName Win32_Battery
if ($battery.BatteryStatus -eq 1) { # 1 = Discharging (On Battery)
    Write-Host "Running on battery: Activating quiet mode..."
    forgum herd quiet
} else {
    Write-Host "Running on AC Power: Restoring full 60 FPS..."
    forgum herd speed 1.0 --all
}
```

---

### D. Bash & Zsh DevOps / CI Scripts (Linux & macOS)

#### Example 16: Deployment Status Daemon (Bash)
Reports Kubernetes or Docker deployment phases visually to developers' open terminals:
```bash
#!/usr/bin/env bash
# deploy.sh

set -e
echo "Initiating Kubernetes rollout..."
forgum herd cow robot --all
forgum herd effect walk --all
forgum herd color cyber --all

if kubectl rollout status deployment/web-api --timeout=60s; then
    echo "Deployment healthy!"
    forgum herd cow tux --all
    forgum herd effect float --all
    forgum herd eyes "^^" --all
else
    echo "Deployment degraded!"
    forgum herd cow dragon --all
    forgum herd effect glitch --all
    forgum herd color fire --all
    exit 1
fi
```

#### Example 17: Production SSH Warning Lock
When an SSH session connects to a production host, change local terminal mascots to alert mode:
```bash
# Add to ~/.bashrc or ~/.zshrc:
ssh_prod_guard() {
    local target="$1"
    if [[ "$target" == *"prod"* ]]; then
        forgum herd color fire --all
        forgum herd eyes "!!" --all
        echo "🚨 ATTENTION: Connected to PRODUCTION environment ($target)"
    fi
    command ssh "$@"
    # Reset when disconnected
    forgum herd color natural --all
    forgum herd eyes "oo" --all
}
alias ssh=ssh_prod_guard
```

#### Example 18: Long Command Completion Chime (Bash Preexec/Precmd)
Calculates duration of previous command; if duration exceeded 30 seconds, wakes the mascot with eyes wide open:
```bash
# Add to ~/.bashrc:
__forgum_preexec() {
    FORGUM_CMD_START=$(date +%s)
}
__forgum_precmd() {
    if [ -n "$FORGUM_CMD_START" ]; then
        local elapsed=$(( $(date +%s) - FORGUM_CMD_START ))
        if [ "$elapsed" -ge 30 ]; then
            forgum herd eyes "$$" --all
            forgum herd effect talk --all
        fi
        unset FORGUM_CMD_START
    fi
}
trap '__forgum_preexec' DEBUG
PROMPT_COMMAND="__forgum_precmd; $PROMPT_COMMAND"
```

#### Example 19: Clean Orphaned Daemons Garbage Collector
Cron script to clean up stale daemon sockets and dead PID entries:
```bash
#!/usr/bin/env bash
# gc_pasture.sh

# List census and stop any unresponsive daemons:
ACTIVE_COUNT=$(forgum herd census | grep -c "running" || true)
echo "Active daemons detected: $ACTIVE_COUNT"

# Sweeps dead records:
forgum doctor > /dev/null 2>&1
echo "Pasture swept cleanly."
```

#### Example 20: Random Mascot Rotation Cron Job
Rotates animal mascots randomly across all developer terminal windows every 15 minutes:
```bash
# Add to crontab via `crontab -e`:
# */15 * * * * forgum herd cow random --all >/dev/null 2>&1
forgum herd cow random --all
forgum herd effect random --all
```

---

### E. Modern Shells: Fish & Nushell

#### Example 21: Fish Shell Prompt Event Integration
Updates eyes to dollar signs when entering a git repository with stash entries:
```fish
# ~/.config/fish/functions/fish_prompt.fish
function __check_forgum_git --on-event fish_prompt
    if git rev-parse --is-inside-work-tree >/dev/null 2>&1
        set -l stashes (git stash list | count)
        if test $stashes -gt 0
            forgum herd eyes '$$' --all
        else
            forgum herd eyes 'oo' --all
        end
    end
end
```

#### Example 22: Nushell Pipeline Swarm Filter
Using Nushell's structured pipelines to filter and pause specific high-framerate daemons:
```nu
# nu script
forgum herd census
| lines
| skip 2
| parse "{pid} {session} {effect} {fps} {speed} {status} {age}"
| where status == "running"
| each { |d|
    if ($d.fps | into int) > 120 {
        forgum herd speed 0.5 --session $d.session
    }
}
```

---

## 4. IPC Control Protocol & Wire Format

Communication between the `forgum herd` CLI client and running daemons occurs via newline-delimited JSON payloads over standard OS local sockets:

### A. Request Wire Payload
```json
{
  "cmd": "COMMAND_NAME",
  "arg": "OPTIONAL_ARGUMENT_STRING"
}
```

Supported Commands:
- `{"cmd": "STATUS"}`: Queries current frame rate, effect name, speed, and pause status.
- `{"cmd": "EFFECT", "arg": "glitch"}`: Hot-swaps current animation effect.
- `{"cmd": "COW", "arg": "dragon"}`: Reloads mascot ASCII art from asset bank.
- `{"cmd": "EYES", "arg": "^^"}`: Replaces eye characters in live frame buffer.
- `{"cmd": "TONGUE", "arg": "U "}`: Replaces tongue character in live frame buffer.
- `{"cmd": "COLOR", "arg": "matrix"}`: Switches active color shader.
- `{"cmd": "SPEED", "arg": "1.5"}`: Modulates physics delta-time scaling.
- `{"cmd": "PAUSE"}`: Sets pause flag; skips simulation update iterations.
- `{"cmd": "RESUME"}`: Clears pause flag; resumes simulation loop.
- `{"cmd": "STOP"}`: Instructs daemon process to cleanly exit and unbind socket.

### B. Daemon Response Wire Payload
```json
{
  "ok": true,
  "status": {
    "effect": "walk",
    "fps": 60,
    "speed": 1.0,
    "paused": false
  }
}
```

---

## 5. Troubleshooting & Diagnostics

- **"No daemons found"**: Ensure your Forgum instance was launched with `--background` (`-b`), `--split-scroll`, or via an initialized shell prompt hook (`auto_render_on_prompt: true`). One-shot banner runs (`--banner` with short duration) exit immediately and do not remain as daemons.
- **Permission Denied on Windows Named Pipe**: Ensure both the terminal running the daemon and the `forgum herd` command are running within the same user security token / integrity level.
- **Stale Dead Daemons in Census**: Run `forgum doctor` or `forgum herd stop --all` to trigger the automatic runtime directory sweep and remove stale socket files.
