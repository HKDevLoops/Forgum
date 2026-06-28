# Forgum — Multiplexer Integration (tmux / zellij / screen / wezterm / remote / herder)

> **The vision:** Forgum isn't just "a fortune in the status bar." It's a **living terminal companion** that rides along inside your multiplexer — animating above the prompt in any pane, synced across shared remote sessions, and herdable as a fleet of daemons you control from one command. This doc makes it *much* cooler.

The current `wiki/tmux-Integration.md` only covers a static fortune in `status-right` (which spawns `pwsh` every 5 min — slow). We supersede it.

---

## 1. Multiplexer detection (the foundation)

```rust
pub enum Mux { None, Tmux { pane: String, session: String }, Zellij { tab: String },
               Screen { window: String }, WezTerm { tab: String } }

pub fn detect_mux() -> Mux {
    if let Ok(v) = env::var("TMUX") {                                  // "/tmp/tmux-1000/default,1234,0"
        let mut it = v.split(','); let session = it.nth(1).unwrap_or("").to_string();
        let pane = env::var("TMUX_PANE").unwrap_or_default();          // "%5"
        return Mmux::Tmux { pane, session };
    }
    if env::var("ZELLIJ").is_ok()      { return Mux::Zellij { tab: env::var("ZELLIJ_TAB_NAME").unwrap_or_default() }; }
    if env::var("STY").is_ok()         { return Mux::Screen { window: env::var("WINDOW").unwrap_or_default() }; }
    if env::var("WEZTERM_UNIX_SOCKET").is_ok() { return Mux::WezTerm { tab: env::var("WEZTERM_TAB_ID").unwrap_or_default() }; }
    Mux::None
}
```

**Why it matters:**
- **Per-pane daemons** — the control socket + PID file are keyed on `$TMUX_PANE` (not the shell PID) so two panes animate independently and `forgum daemon stop` only kills *this* pane's daemon (BUG-D3 fixed properly).
- **Passthrough awareness** — tmux < 3.3 needs `allow-passthrough off` (no alt screen inside panes); tmux ≥ 3.3 + zellij/screen have different escape rules.
- **Resize source** — inside a multiplexer, `SIGWINCH` fires on pane resize; the engine handles it (see `03-…` §8). No event-poll needed.

---

## 2. tmux — the flagship integration

### 2.1 The four integration surfaces

| Surface | What it does | How |
|---------|--------------|-----|
| **A. Pane overlay** | animation above the prompt *inside* a tmux pane | background daemon per `$TMUX_PANE` (the core feature from `03-…`) |
| **B. Status line** | live mini-animation / fortune in `status-right` | `forgum-engine status-line` one-shot, or a 1 fps daemon feeding `#(…)` |
| **C. Popup cow** | a cow bursts out of a tmux popup (`display-popup`) on demand | `forgum tmux popup --effect shatter` |
| **D. Pane hooks** | auto-start/stop animations on pane focus / window switch | tmux `set-hook` calls `forgum daemon start/stop` |

### 2.2 Surface A — pane overlay (already designed)

Works out of the box once `03-…` is implemented. tmux panes are independent PTYs, so the engine's overlay sits above the pane's prompt with no special handling. The only tmux-specific notes:
- **Passthrough:** if the user has `set -g allow-passthrough on` and the engine ever uses the alternate screen (foreground mode), tmux 3.3+ forwards the sequences. For background mode (the default), no passthrough is needed — the overlay writes directly to the pane's PTY.
- **Pane-aware daemon state:** `runtime_dir/forgum/pane-${TMUX_PANE}.json` holds `{pid, ob_y1, effect}`. `forgum daemon stop` targets this. `forgum daemon stop --all` iterates.

### 2.3 Surface B — the living status line

The old wiki approach spawns `pwsh` every 5 min (200–800 ms each). Replace with a **native engine one-shot** that runs in < 5 ms:

```bash
# ~/.tmux.conf
set -g status-right-length 80
set -g status-interval 5
set -g status-right "#(forgum-engine status-line --format tmux --max-len 70)"
```

`forgum-engine status-line`:
1. reads the config (effect, cow, lolcat),
2. picks a fortune (cached per `status-interval` via a mtime check on a cache file),
3. renders a **single-frame** mini-cow + fortune with ANSI truecolor,
4. prints it as a single line (tmux passes ANSI through to the status line).

**Cooler variant — a 1 fps "ticker" daemon:**
```bash
set -g status-right "#(forgum-engine status-line --live --fps 1)"
```
The engine runs a 1 fps background loop (the §11.5 heartbeat scheduler) that updates a shared status file; tmux's `#(cat /tmp/forgum/status.tmux)` reads it every `status-interval`. This gives a **slowly color-cycling fortune** in the status bar for ~0 CPU.

### 2.4 Surface C — the popup cow (tmux 3.2+)

tmux `display-popup` opens a floating overlay. Forgum can claim it for a dramatic one-shot:

```bash
# bind a key to summon a shattering cow
bind C-f run-shell 'forgum-engine render --effect shatter --cow dragon \
    --text "$(forgum-engine fortune)" --no-background --duration 4 \
    --popup'
```

`--popup` mode:
1. prints the tmux `display-popup -E -w 60 -h 20 -x C -y C --` wrapper sequence,
2. runs the foreground render loop inside the popup (full-screen, raw mode, q-to-quit),
3. on exit, the popup closes and the pane is untouched.

Result: `Ctrl-f` summons a 4-second shattering dragon fortune in the center of the tmux window. Pure dopamine.

### 2.5 Surface D — tmux hooks (auto start/stop)

```bash
# ~/.tmux.conf
set-hook -g pane-focus-in    'run-shell "forgum daemon start --effect aurora --cow default 2>/dev/null"'
set-hook -g pane-focus-out   'run-shell "forgum daemon stop 2>/dev/null"'
set-hook -g window-layout-changed 'run-shell "forgum daemon resize 2>/dev/null"'
```

Now the focused pane breathes aurora; switching panes hands the animation off. `forgum daemon resize` sends `SIGWINCH` to the pane's daemon so it reflows without restarting.

### 2.6 The `forgum tmux` subcommand

A convenience layer over raw tmux commands:

```
forgum tmux install          # writes the §2.2–2.5 config block to ~/.tmux.conf (idempotent)
forgum tmux popup [--effect E] [--cow C] [--duration N]
forgum tmux status [on|off|live]
forgum tmux focus-aware [on|off]
forgum tmux sync             # §4 — synchronize effects across panes
```

---

## 3. zellij

zellij is the modern Rust multiplexer. Integration is simpler than tmux in some ways (no passthrough quirks) and harder in others (less hook surface).

| Feature | zellij support |
|---------|----------------|
| Pane overlay (Surface A) | ✅ works — zellij panes are PTYs like tmux |
| Status bar (Surface B) | ⚠️ zellij's status bar isn't ANSI-customizable; instead use a **plugin** (WASM) or a `tab` name that the engine updates via `zellij action rename-tab` |
| Popup (Surface C) | ⚠️ no native popup; use `zellij action new-pane --floating` as the popup |
| Hooks (Surface D) | ⚠️ no `pane-focus-in` equivalent yet; poll via `ZELLIJ_PANE_ID` env in the `precmd` sweep |

```bash
# ~/.config/zellij/config.kdl
keybinds {
    shared { bind "Ctrl f" { Run "forgum-engine" "render" "--effect" "shatter" "--duration" "4" "--no-background"; }; }
}
```

The `forgum zellij install` subcommand wires up a floating-pane popup + tab-name status.

---

## 4. GNU screen

screen is the grandfather. Its alternate-screen handling is historically flaky, so:

- **Force background-overlay mode** in screen (never alt screen). The engine's `render_loop_background` is the default; in screen we disable `--no-background`.
- Status line: screen's `backtick` + `caption`/`hardstatus`:
  ```
  hardstatus string "%{=b kw}%-w%{=b kW}%n %t%{-}%+w %=%{=b rw}`forgum-engine status-line --max-len 50`"
  backtick 0 0 5 forgum-engine status-line --max-len 50
  ```
- No popup, no hooks. Documented as "best effort."

---

## 5. wezterm

wezterm is both a terminal and a multiplexer (tabs, panes, domains). Integration via its Lua config:

```lua
-- ~/.wezterm.lua
local wezterm = require 'wezterm'
return {
  status_update_interval = 1000,
  wezterm.on('update-status', function(window, pane)
    local handle = io.popen('forgum-engine status-line --format wezterm --max-len 60 2>/dev/null')
    local status = handle:read('*a') or ''; handle:close()
    return { { Text = status } }
  end),
  keys = {
    { key='f', mods='CTRL', action=wezterm.action.SpawnCommandInNewTab {
      args={'forgum-engine','render','--effect','shatter','--duration','4','--no-background'} } },
  },
}
```
wezterm's GPU rendering makes the 60 fps overlay buttery. The `forgum wezterm install` subcommand writes this block.

---

## 6. "rmux" — remote / shared sessions

The user asked for "rmux." This is the **remote/shared-session** story: a Forgum animation that **follows you** across an SSH session, or is **synced** to everyone attached to a shared tmux session (wemux-style).

### 6.1 The follow-me animation (per-user, cross-host)

A Forgum daemon runs on your **local** laptop. When you SSH to a server, the remote shell's `forgum` hook detects `$SSH_CONNECTION` and, instead of starting a remote daemon, connects back to your local daemon's control socket via SSH reverse forwarding:

```bash
# ~/.ssh/config
Host *
  RemoteForward /tmp/forgum-%u.sock /tmp/forgum-%u.sock   # forward the control socket
```

Now `forgum daemon effect ember` typed **on the server** reaches your **local** daemon via the forwarded socket, and the animation plays on your local terminal above the SSH prompt. The remote host needs no Forgum install (only the hook, which is a 5-line bash function). This is the "my cow follows me across SSH" feature.

### 6.2 The shared-session herd (wemux-style)

For pair-programming on a shared tmux session (`wemux`/`tmux -L pair`), every attached client sees the **same** animation because they share the same PTY. The daemon keys its state on the tmux **server socket** (`$TMUX`'s first field), not the pane, so all clients see one synchronized effect. `forgum daemon effect plasma` (run by anyone) changes it for everyone. Nice for demo sessions.

### 6.3 `forgum remote` subcommand

```
forgum remote attach <host>          # attach this terminal's overlay to a remote daemon
forgum remote sync   <session-id>    # broadcast effect changes to all peers in <session-id>
forgum remote who                   # list active peers
```

Implementation: a tiny RPC over the control socket (JSON lines). Each daemon can act as a peer; one is elected "leader" (lowest PID wins) and its effect choice is replicated. Lag-tolerant: effects are deterministic given `(name, seed, start_time)`, so peers render the same frame without streaming pixels — they only sync the occasional `EFFECT`/`SPEED` command. ~10 bytes/sec of sync traffic.

---

## 7. "herdr" — the daemon herder

The user asked for "herdr." This is Forgum's **fleet manager** for the (surprisingly common) situation where you have background daemons in multiple panes/windows/tabs/hosts and want to control them as one.

### 7.1 The problem

With per-pane daemons (§2.2), a developer with 6 tmux panes has 6 independent Forgum daemons. `forgum daemon stop` kills one. Killing all six means `for p in …; forgum daemon stop …`. Changing the effect everywhere means repeating it 6 times. Enter the herder.

### 7.2 `forgum herd` — the fleet CLI

```
forgum herd list                      # all daemons on this host (+ remote peers)
#  PID    PANE    SESSION   EFFECT     FPS   CPU   AGE
#  12345  %3      main      aurora     60    0.4%  2m
#  12346  %5      main      ember      30    0.2%  1m
#  12347  %1      logs      matrix     5     0.0%  5m  (idle)
#  12348  -       editor    aurora     60    0.4%  30s [remote: dev-laptop]

forgum herd stop  [--all | --pane %5 | --session main | --host dev-laptop]
forgum herd effect aurora  [--all]     # set effect on the whole herd
forgum herd speed 1.5        [--all]
forgum herd pause / resume   [--all]
forgum herd theme cyberpunk            # apply a named theme bundle (effect+cows+palette)
forgum herd quiet                      # drop every daemon to idle (1 fps)
forgum herd follow <pane>              # only the focused pane animates; others idle (see §2.5)
forgum herd census                     # health check; restarts dead daemons
```

### 7.3 How it works

- **Discovery:** scan `runtime_dir/forgum/*.json` (one per pane/session). Each file has `{pid, pane, session, host, socket, effect, fps, age}`.
- **Control:** send commands to each daemon's control socket (§6.4 of `03-…`). Parallel, non-blocking. Replies aggregated into the `list` table.
- **Remote peers:** the `remote sync` mesh (§6.3) propagates `herd` commands to other hosts.
- **`census`:** a watchdog — if a daemon's PID is dead but its state file lingers, the herder sweeps it (runs the §9.3 sweep) and, if `--auto-restart` is set, respawns it.

### 7.4 The herder as a tmux companion

`forgum herd` integrates with tmux so the herd view itself is a tmux **popup**:

```bash
bind H display-popup -E -w 90 -h 20 'forgum herd list --watch'
```

`--watch` refreshes the table every second inside the popup. Press `e` to change effect on all, `q` to quit. A live dashboard for your cow fleet. (This is the "much more cooler" the user asked for.)

---

## 8. Themes & "cool" bundles

Coordinate effect + cow + palette across the herd with one command:

```jsonc
// ~/.config/Forgum/themes/cyberpunk.json
{
  "effect": "glitch",
  "cow": "tux",
  "eyes": "@@",
  "palette": { "fg": [0,255,200], "bg": [10,10,30] },
  "lolcat": { "enabled": true, "spread": 4.0 }
}
```

```
forgum theme apply cyberpunk [--all]
forgum theme list
forgum theme random           # roll a random theme every N minutes (--rotate 5)
```

`--rotate N` schedules a theme change every N minutes via the herder — your terminal's mood shifts through the day. Pair with `forgum herd follow` for a workspace that breathes.

---

## 9. tmux-specific escape safety

tmux strips/rewrites some sequences. Rules the engine follows inside tmux:

1. **No `ESC[?1049h` (alt screen) in background mode** — would hijack the pane. Foreground `--popup` mode uses it inside a popup, which is fine.
2. **Cursor save/restore uses `ESC 7`/`ESC 8`** (DECSC) — tmux preserves these per-pane. Avoid `ESC[s` (SCP) which tmux sometimes collapses.
3. **Truecolor (`ESC[38;2;r;g;bt m`) is passed through** by tmux ≥ 2.6. For older tmux, the engine downgrades to 256-color (detect via `$TMUX` + `tmux -V`).
4. **No `rmcup`/`smcup` quirks** — we don't rely on them in background mode.
5. **Pane resize = `SIGWINCH`** — handled by the §8 resize path of the engine, no tmux-specific code needed.

---

## 10. The "cool" demo script

A single command that showcases everything:

```bash
forgum demo
```

runs:
1. `forgum herd quiet` (calm the fleet),
2. `forgum tmux popup --effect shatter --cow dragon --text "Forgum, online." --duration 3`,
3. `forgum herd effect aurora --all`,
4. `forgum theme rotate 5` (mood cycling every 5 min),
5. `forgum herd follow` (only the focused pane animates).

A 3-second dramatic reveal, then a calm, focus-aware, mood-cycling workspace. This is the demo that sells the project.

---

## 11. Implementation phases (multiplexer slice)

| Phase | Deliverable | Depends on |
|-------|-------------|------------|
| M1 | `detect_mux()` + per-pane daemon state files | `03-…` §10 |
| M2 | `forgum tmux install` + status-line one-shot | M1 |
| M3 | `forgum tmux popup` (Surface C) | M1, `--popup` render mode |
| M4 | tmux `set-hook` focus-aware (Surface D) | M2 |
| M5 | `forgum herd list/stop/effect/speed` | M1, control socket (`03-…` §6.4) |
| M6 | `forgum herd follow` + `--watch` popup dashboard | M5 |
| M7 | `forgum theme apply/list/rotate` | M5 |
| M8 | zellij + wezterm + screen installers | M2 (port the config templates) |
| M9 | `forgum remote attach/sync` (rmux) | M5, control-socket RPC |
| M10 | `forgum herd census` watchdog + auto-restart | M5, §9.3 sweep |
| M11 | `forgum demo` showcase | M3, M6, M7 |

M1–M4 is the tmux MVP (~1 week after the engine core). M5–M7 is the herder (~1 week). M8–M11 is the long tail.

---

**Next:** `06-ARCHITECTURE.md` — the consolidated target architecture tying the engine, platform crate, shell hooks, and herder together.
