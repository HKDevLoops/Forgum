# Phase 6 — Herder Design Spec

> Fleet manager for controlling multiple Forgum daemons across panes/sessions.

---

## Goal

Provide `forgum herd` CLI to discover, list, and control all running Forgum daemons on a host. Support theme bundles for coordinated effect+cow+palette changes.

---

## Deliverables

| # | Feature | Description |
|---|---------|-------------|
| H1 | `herd list` | Scan daemon state files, query each via control socket, display table |
| H2 | `herd stop/effect/speed/pause/resume` | Fan out commands to all or filtered daemons |
| H3 | `herd quiet` | Drop all daemons to 1 fps (idle) |
| H4 | `theme apply/list/rotate` | Theme bundles (effect+cow+palette) |

---

## H1: Daemon Discovery

Scan `runtime_dir/daemon-*.json` for all daemon state files. For each:
1. Read `DaemonState` (pid, socket_path, started_at, cols, ob_y1)
2. Check if PID is alive via `process_is_alive()`
3. If alive, connect to control socket, send `STATUS` command, parse response
4. Build `HerdEntry` with state + status info

```rust
pub struct HerdEntry {
    pub session_id: String,
    pub pid: u32,
    pub alive: bool,
    pub effect: String,
    pub fps: u16,
    pub speed: f32,
    pub paused: bool,
    pub age: String,
    pub socket_path: String,
}
```

---

## H2: Command Fan-out

For `stop/effect/speed/pause/resume`, iterate over discovered daemons (optionally filtered by session_id), connect to each control socket, send the command, collect results.

Filtering flags:
- `--session <id>` — only daemons in this session
- `--all` — all daemons (default)

---

## H3: Quiet Mode

`forgum herd quiet` sends `SPEED 0.1` to all daemons (drops them to ~1 fps for battery savings).

---

## H4: Themes

Themes are JSON files in `~/.config/Forgum/themes/`:

```json
{
  "effect": "aurora",
  "cow": "tux",
  "eyes": "@@",
  "tongue": "U"
}
```

`forgum theme list` — list available themes
`forgum theme apply <name>` — send EFFECT+COW commands to all daemons
