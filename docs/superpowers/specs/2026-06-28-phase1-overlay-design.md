# Phase 1 — Make the Overlay Actually Work

> Design spec for Phase 1 deliverables. Builds on Phase 0's RAII guards,
> signal handlers, and input-free background loop.

---

## Goal

Background animation renders above the prompt, survives resize, exits cleanly,
doesn't die with the shell, and can be managed via a control socket.

---

## Tasks (scoped to this implementation)

| # | Task | Bug | Deliverable |
|---|------|-----|-------------|
| 1.1 | SIGWINCH resize handler | B3 | `AtomicBool` resize flag; signal handler sets it; render loop checks & resizes framebuffer |
| 1.2 | Daemon detach + PID file | B7, D3 | `setsid`/`DETACHED_PROCESS`; writes `daemon.json` with `{pid, ob_y1, cols, socket_path}` |
| 1.3 | Control socket | new | Unix domain socket; commands: `STOP`, `PAUSE`, `RESUME`, `EFFECT <name>`, `SPEED <f32>`, `COW <name>`, `STATUS`, `PING` |
| 1.4 | Per-session StopDaemon | D3 | reads `daemon.json`, sends SIGTERM, cleans up |
| 1.5 | Temp file cleanup | D2 | engine removes `--file` after reading |
| 1.6 | Tiny-terminal guard | E7 | `cols < 20 \|\| rows < 5` → static print fallback, no animation |

---

## 1.1 SIGWINCH Resize Handler

### Signal handler (platform crate)

Add `SIGWINCH` to the signal handler in `signal.rs`. When received, set an
`AtomicBool` flag (`resize_pending`). The render loop checks this flag each
frame and re-probes terminal size.

```rust
// In SignalGuard::install():
let resize_flag = shutdown.resize_flag(); // Arc<AtomicBool>
// signal-hook registers SIGWINCH → resize_flag.store(true, Relaxed);
```

### Render loop integration

```rust
// In render_loop_background / render_loop_foreground:
if resize_pending.swap(false, Ordering::Relaxed) {
    let caps = TerminalCapabilities::probe();
    let (new_cols, new_rows) = (caps.width.max(1), caps.height.max(1));
    fb.resize(usize::from(new_cols), usize::from(new_rows));
    // recompute overlay bounds: ob_y1 = min(overlay_height, rows - PROMPT_GUARD)
}
```

### FrameBuffer resize

Add `fb.resize(new_cols, new_rows)` that reallocates the cell buffer and
preserves existing content where possible.

---

## 1.2 Daemon Detach + PID File

### Detach mechanism

On Unix: `unsafe { libc::setsid() }` after fork, before exec.
On Windows: `CREATE_NO_WINDOW | DETACHED_PROCESS` creation flags.

### daemon.json

Written to `$XDG_RUNTIME_DIR/forgum/daemon-{pane_id}.json` (Linux) or
`%LOCALAPPDATA%/Forgum/daemon-{pane_id}.json` (Windows).

```json
{
  "pid": 12345,
  "ob_y1": 10,
  "cols": 80,
  "socket_path": "/tmp/forgum-ctrl-{pane_id}.sock",
  "started_at": "2026-06-28T21:00:00Z"
}
```

### CLI addition

`--daemon` flag on the engine binary triggers detach:
1. Fork (Unix) or CreateProcess with DETACHED_PROCESS (Windows)
2. Parent exits with 0
3. Child writes daemon.json
4. Child runs render loop

---

## 1.3 Control Socket

### Protocol

Unix domain socket (Windows named pipe). Newline-delimited JSON:

Request:  `{ "cmd": "STOP" }\n`
Response: `{ "ok": true }\n` or `{ "ok": false, "error": "..." }\n`

### Commands

| Command | Effect |
|---------|--------|
| `STOP` | Set shutdown flag, clean exit |
| `PAUSE` | Stop rendering (keep loop alive) |
| `RESUME` | Resume rendering |
| `EFFECT <name>` | Change effect |
| `SPEED <f32>` | Set speed multiplier |
| `COW <name>` | Change cow |
| `STATUS` | Return `{ "running": true, "effect": "...", "fps": 30, ... }` |
| `PING` | Return `{ "ok": true }` |

### Implementation

Add `control_socket.rs` to engine:
- `ControlServer::bind(path)` — creates socket, starts accept thread
- `ControlMsg` enum — parsed commands
- `crossbeam::channel::unbounded<ControlMsg>` — fed to render loop

---

## 1.4 Per-session StopDaemon

### PowerShell: `Stop-ForgumDaemon`

```powershell
function Stop-ForgumDaemon {
    param([string] $DaemonJsonPath)
    $daemon = Get-Content -Raw $DaemonJsonPath | ConvertFrom-Json
    if (Get-Process -Id $daemon.pid -ErrorAction SilentlyContinue) {
        Stop-Process -Id $daemon.pid -Force
    }
    Remove-Item $DaemonJsonPath -Force -ErrorAction SilentlyContinue
}
```

### Precmd sweep (shell hooks)

On each prompt, check if daemon PID is alive. If not, clear the overlay
(emit `ESC[?25h ESC[0m ESC[?1049l`).

---

## 1.5 Temp File Cleanup

In `read_scene()`, after parsing the JSON from `--file`, call:
```rust
if let Some(path) = file_path {
    let _ = std::fs::remove_file(path);
}
```

---

## 1.6 Tiny-terminal Guard

```rust
const MIN_COLS: u16 = 20;
const MIN_ROWS: u16 = 5;

if cols < MIN_COLS || rows < MIN_ROWS {
    // Print cow_text as plain text (no animation)
    // Return early from render loop
}
```

---

## Test Gates

1. **Resize survival**: background animation survives 3 resizes in 5 seconds
2. **Daemon survives shell exit**: launch daemon, close parent shell, daemon still running
3. **Control socket STOP**: send STOP via socket, daemon exits cleanly
4. **Per-session kill**: two daemons, kill one, other keeps running
5. **Temp file cleanup**: after render, `--file` no longer exists
6. **Tiny terminal**: cols=10 → static print, no crash
7. **All Phase 0 tests still pass**
