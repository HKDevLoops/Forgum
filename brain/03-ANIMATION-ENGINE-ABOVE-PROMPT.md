# Forgum — Animation Engine Above the Prompt

> **The flagship feature.** A Rust ANSI framebuffer renderer that paints animated ASCII cows/effects into a **reserved overlay region at the top of the terminal**, while the user's shell prompt below stays 100 % interactive — typing, history, completion, job control all keep working. No raw mode, no hidden cursor, no stolen keystrokes.

This document is the **authoritative design**. It supersedes the incomplete implementation in `engine/src/main.rs:render_loop_background` (which has BUG-B1/B2/B3/B4 — see `01-BUGS-AND-ISSUES.md` §2) and realizes `docs/PLAN-2026-06-20-BACKGROUND-ENGINE.md`.

---

## 1. The mental model

```
┌──────────────────────────────────────────────────────┐ row 0
│  ┌────────────────────────────────────────────────┐  │
│  │  OVERLAY REGION  (rows 0 .. ob_y1-1)           │  │  ← engine writes ONLY here
│  │  aurora/ember/plasma + cow + particles         │  │
│  │  differential, cursor-save/restore each frame  │  │
│  └────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────┤ row ob_y1  (prompt guard band)
│  user@host:~/proj$ ls -la _                          │  ← engine NEVER touches this
│  total 42                                             │
└──────────────────────────────────────────────────────┘ row rows-1
```

**Three invariants, enforced in code, verified by tests:**

1. **Region invariant** — the engine never writes a byte to any row `>= ob_y1`. `ob_y1 = min(overlay_height, rows - PROMPT_GUARD)` where `PROMPT_GUARD = 3` (one prompt line + one scroll-back line + breathing room).
2. **Cursor invariant** — the user's cursor position before and after every frame is byte-identical. Achieved with `ESC7` (save) / `ESC8` (restore) wrapped around the render, plus a `Drop` guard that restores on panic.
3. **Input invariant** — the engine never reads from the TTY input stream in background mode. It cannot steal, swallow, or interpret keystrokes. Exit is via duration, signal, or a control socket.

---

## 2. Why the current code is broken (executive summary)

| Bug | Effect | Fix section |
|-----|--------|-------------|
| BUG-B1 | `event::poll/read` on `/dev/tty` steals the shell's keystrokes; Enter exits the engine | §6.1 |
| BUG-B2 | `duration=0` runs 150 frames (~5 s) instead of infinite | §7.1 |
| BUG-B3 | No `Resize` handling → resize corrupts the display | §8 |
| BUG-B4 | Cleanup loop uses stale `ob_y1` → scrolls the prompt away on exit after resize | §9.2 |
| BUG-B7 | Daemon not detached → dies with the shell | §10 |
| BUG-B9 | `console_out` dead → piped stdout gets no animation | §3 |
| BUG-E1 | `Cell::dirty` in `PartialEq` → 60 fps forever even for static cows | §11.1 |

Everything below is the fix.

---

## 3. Output handle resolution (where the bytes go)

A single function decides the output sink. It removes the dead `console_out` and the `cmd /c` workaround in one stroke.

```rust
// forgum-platform/src/output.rs
pub fn open_render_output() -> RenderOutput {
    if io::stdout().is_terminal() {
        return RenderOutput::Stdout(io::BufWriter::new(io::stdout()));
    }
    #[cfg(unix)]
    {
        if let Ok(f) = fs::OpenOptions::new().write(true).open("/dev/tty") {
            return RenderOutput::Tty(io::BufWriter::new(f));
        }
    }
    #[cfg(windows)]
    {
        if let Some(f) = open_console_out() {  // CONOUT$
            return RenderOutput::Tty(io::BufWriter::new(f));
        }
    }
    RenderOutput::Pipe(io::BufWriter::new(io::stdout())) // no animation possible
}

pub enum RenderOutput { Stdout(BufWriter<Stdout>), Tty(BufWriter<File>), Pipe(BufWriter<Stdout>) }
```

`render_loop_background` uses `open_render_output()` and writes through a `&mut dyn Write`. The `is_terminal_stdout()` early-return now only fires for the `Pipe` variant (print `cow_text` and bail).

---

## 4. Region math (the prompt guard)

```rust
const PROMPT_GUARD: u16 = 3;          // never render into the bottom 3 rows
const MIN_OVERLAY:   u16 = 5;         // refuse to animate below this height

fn compute_overlay_bounds(cols: u16, rows: u16, requested: u16) -> Rect {
    let y1 = requested.min(rows.saturating_sub(PROMPT_GUARD)).max(MIN_OVERLAY.min(rows));
    Rect { x0: 0, y0: 0, x1: cols, y1 }   // y1 is exclusive top-bound; engine writes rows [0, y1)
}
```

- On a 24-row terminal with `overlay_height=10`: `ob_y1 = min(10, 21) = 10`. Prompt lives in rows 10..23.
- On a 6-row terminal: `ob_y1 = min(10, 3) = 3`. Engine animates rows 0..2, prompt in 3..5. Degraded but safe.
- On a 4-row terminal: `rows - PROMPT_GUARD = 1`, `max(MIN_OVERLAY.min(4)) = max(4) = 4` → engine takes the whole screen but **falls back to static print** instead (see §11.4). We never corrupt a 4-row terminal.

`ob_y1` is a **`Cell`/`mut`** tracked across resizes — never a `let` that goes stale (BUG-B4).

---

## 5. The render cycle (per frame)

```rust
loop {
    // 0. shutdown check (signal flag / control socket / duration)
    if !running.load(Relaxed) || (max_frames > 0 && frame_count >= max_frames) { break; }

    // 1. timing
    let now = Instant::now();
    let dt = (now - last_frame).as_secs_f32().min(0.1);
    last_frame = now;

    // 2. optional resize (SIGWINCH-driven, not event-poll-driven — see §8)
    if resize_pending.swap(false, Relaxed) { handle_resize(&mut fb, &mut region, &mut effect, &mut ob_y1); }

    // 3. simulate
    effect.update(dt * speed);

    // 4. composite into back buffer (cleared first)
    fb.clear();
    let clip = region.get(overlay_id).map(|r| r.bounds).unwrap_or(full_clip);
    effect.render(&mut fb, clip);

    // 5. diff + coalesced write, wrapped in cursor save/restore
    fb.compute_damage();
    if scheduler.should_render(fb.damage_count()) {
        write!(out, "\x1b7")?;                       // ESC 7 = save cursor (DECSC)
        let n = fb.render_region(&mut out, clip)?;   // coalesced runs (§11.2)
        write!(out, "\x1b8")?;                       // ESC 8 = restore cursor (DECRC)
        out.flush()?;
        scheduler.adapt(n);
    } else {
        scheduler.adapt(0);
    }

    frame_count += 1;
    scheduler.wait_if_needed();
}
```

### 5.1 Why `ESC 7`/`ESC 8` and not crossterm `cursor::SavePosition`?

`cursor::SavePosition` emits `ESC[s` (SCP) which some terminals (older xterm, GNU screen) treat as a single-level save. `ESC 7` (DECSC) saves **cursor + attributes + charset** and is universally supported, including the prompt's own save/restore. We use `ESC 7`/`ESC 8` and **never** nest engine saves on top of a shell save that hasn't been restored. The render cycle is strictly: save → write overlay → restore → the shell sees its cursor exactly where it was.

### 5.2 What about the user's in-progress typed text?

The user is mid-typing: `user@host:~/proj$ ls -la foo|_` (cursor at `|`). The engine saves the cursor (at `foo|`), writes a frame to rows 0..ob_y1 (the top), restores the cursor (back to `foo|`). The typed text and cursor are untouched because they live in rows `>= ob_y1`. The only visible effect: the top of the terminal updates. This is the whole point of the design.

---

## 6. Input handling (the keystroke-theft fix)

### 6.1 Background mode reads NO input

The current `event::poll`/`event::read` in the background loop is **deleted entirely**. Background mode has no keyboard input path. This single change fixes BUG-B1 and BUG-B6.

### 6.2 Exit mechanisms (in priority order)

| Mechanism | How | Latency |
|-----------|-----|---------|
| **Duration timeout** | `max_frames` reached (§7) | exact |
| **Signal** | `SIGTERM`/`SIGHUP`/`SIGINT` → `AtomicBool` (§6.3) | < 1 frame |
| **Control socket** | `STOP` command on a Unix socket / named pipe (§6.4) | < 50 ms |
| **Sentinel file** | existence of `$runtime_dir/stop` | next frame |
| **Parent shell exit** | daemon detached (§10) → survives; or `prctl(PR_SET_PDEATHSIG, SIGHUP)` on Linux to die with parent if desired | immediate |

### 6.3 Signal handler

```rust
// forgum-platform/src/signal.rs
pub struct ShutdownFlag(Arc<AtomicBool>);
impl ShutdownFlag {
    pub fn install() -> io::Result<Self> {
        let flag = Arc::new(AtomicBool::new(false));
        #[cfg(unix)] {
            signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&flag))?;
            signal_hook::flag::register(signal_hook::consts::SIGINT,  Arc::clone(&flag))?;
            signal_hook::flag::register(signal_hook::consts::SIGHUP,  Arc::clone(&flag))?;
            // SIGWINCH is separate (resize, not shutdown)
            signal_hook::flag::register(signal_hook::consts::SIGWINCH, Arc::clone(&resize_flag))?;
        }
        #[cfg(windows)] { install_console_ctrl_handler(Arc::clone(&flag))?; }
        Ok(Self(flag))
    }
    pub fn pending(&self) -> bool { self.0.load(Ordering::Relaxed) }
}
```

The render loop checks `shutdown.pending()` at the top of every iteration. Cost: one relaxed atomic load per frame. Effectively free.

### 6.4 Control socket (the "cool" exit + live control)

```
$XDG_RUNTIME_DIR/forgum/<session>.sock   (mode 0600, per-user)
```

The daemon listens on this socket with a 0 ms poll (non-blocking `accept`) once per frame, or better, in a **dedicated thread** that pushes commands onto an `mpsc` channel. Supported commands:

| Command | Action |
|---------|--------|
| `STOP` | graceful shutdown (clears overlay, exits 0) |
| `PAUSE` / `RESUME` | toggle `running` without exiting |
| `EFFECT <name>` | hot-swap the effect at runtime |
| `SPEED <f32>` | live speed multiplier |
| `COW <file>` | reload cow, re-render |
| `STATUS` | reply with JSON `{fps, frame, effect, pid}` |
| `PING` | reply `PONG` (health check) |

This is what `forgum daemon stop`, `forgum daemon effect aurora`, `forgum daemon speed 1.5` use. It replaces `StopDaemon`'s blanket `kill` (BUG-D3) with precise, per-session control. The socket path is keyed on the tmux pane / shell PID so two terminals don't collide.

```rust
// per-frame, non-blocking:
if let Ok(cmd) = control_rx.try_recv() { apply_control(cmd, &mut effect, &mut running); }
```

---

## 7. Duration & looping semantics (fixed)

### 7.1 The rule

| `duration` (seconds) | `max_frames` | Behavior |
|----------------------|--------------|----------|
| `0` | `0` | **infinite** — runs until signal/socket/STOP |
| `N > 0` | `N * fps` (u64, saturating) | runs exactly `N` seconds, then cleans up & exits |
| absent | `0` | infinite (safe default for background) |

```rust
let max_frames: u64 = match config.duration.unwrap_or(0) {
    0 => 0,
    n => (n as u64).saturating_mul(fps as u64),
};
// loop guard:
if max_frames > 0 && frame_count >= max_frames { break; }
```

This fixes BUG-B2 and BUG-D7. The shell hooks send `duration: 0` for "live wallpaper" mode; `forgum run --duration 12` sends `duration: 12` for a one-shot.

### 7.2 Effect "done" short-circuit

One-shot effects (Shatter, Dissolve) set `active=false` when all particles die. Without a guard, the overlay stays blank for the rest of `duration`. Add:

```rust
pub trait Effect {
    fn update(&mut self, dt: f32);
    fn render(&self, fb: &mut FrameBuffer, clip: Rect);
    fn on_resize(&mut self, cols: usize, rows: usize);
    fn is_done(&self) -> bool { false }   // default: never
}
// in the loop:
if effect.is_done() && frame_count > MIN_FRAMES { break; }   // BUG-E4
```

---

## 8. Resize handling (the missing arm)

Background mode does **not** use crossterm's event loop (that's what stole keystrokes). Instead, a `SIGWINCH` handler (Unix) / `WINDOW_BUFFER_SIZE_EVENT` thread (Windows) sets a `resize_pending: AtomicBool`. The render loop handles it synchronously:

```rust
fn handle_resize(fb: &mut FrameBuffer, region: &mut RegionAllocator,
                 effect: &mut dyn Effect, ob_y1: &mut u16) {
    // 1. clear the OLD overlay on-screen so stale pixels don't linger (BUG-F2)
    let (oc, or) = (fb.width as u16, *ob_y1);
    write!(out, "\x1b7").ok();
    for y in 0..or {
        write!(out, "\x1b[{};1H{}", y + 1, " ".repeat(oc as usize)).ok();
    }
    write!(out, "\x1b8").ok();
    out.flush().ok();

    // 2. re-query real size
    let (nc, nr) = get_terminal_size();
    fb.resize(nc as usize, nr as usize);
    region.resize_canvas(Rect::new(0, 0, nc, nr));

    // 3. recompute overlay bounds
    *ob_y1 = compute_overlay_bounds(nc, nr, requested_overlay_height).y1;
    region.resize_region(overlay_id, Rect::new(0, 0, nc, *ob_y1));

    // 4. let the effect recenter
    effect.on_resize(nc as usize, nr as usize);

    // 5. force a full repaint next frame
    fb.invalidate();   // sets front = empty so compute_damage marks everything dirty
}
```

This fixes BUG-B3, BUG-B4, BUG-F2 in one place. The on-screen clear-before-resize is the key insight the original code missed.

---

## 9. Clean exit (the prompt-safety guarantee)

### 9.1 RAII guards

```rust
struct OverlayGuard<'a> { out: &'a mut dyn Write, ob_y1: u16, cols: u16 }
impl Drop for OverlayGuard<'_> {
    fn drop(&mut self) {
        // clear the overlay region, restore cursor, reset color
        let _ = write!(self.out, "\x1b7");
        for y in 0..self.ob_y1 {
            let _ = write!(self.out, "\x1b[{};1H{}", y + 1, " ".repeat(self.cols as usize));
        }
        let _ = write!(self.out, "\x1b8\x1b[0m");
        let _ = self.out.flush();
    }
}
```

Created **after** the first successful frame. On any return path (normal exit, signal, panic-unwind, `?` error), `Drop` runs → overlay cleared, cursor restored, color reset. The user's prompt is byte-identical to before the animation.

### 9.2 Stale-safe cleanup (post-resize)

The guard captures `ob_y1` and `cols` from a `Cell` that's updated by `handle_resize`, so it always clears the **current** region, never a stale one (BUG-B4).

### 9.3 The SIGKILL escape hatch

`SIGKILL` skips `Drop`. If a user `kill -9`s the daemon, the overlay stays. Mitigation: the shell hook's `PROMPT_COMMAND` / `precmd` hook runs `forgum daemon sweep` which:
1. checks the PID file is alive; if not,
2. clears the top `ob_y1` rows (stored alongside the PID) and resets color.

```bash
# bash hook (simplified)
__forgum_precmd() {
  if [ -f "$FORGUM_RUNTIME/daemon.json" ]; then
    pid=$(jq -r .pid "$FORGUM_RUNTIME/daemon.json")
    if ! kill -0 "$pid" 2>/dev/null; then
      # daemon died ungracefully — sweep
      rows=$(jq -r .ob_y1 "$FORGUM_RUNTIME/daemon.json")
      printf '\x1b7'; for y in $(seq 1 "$rows"); do printf '\x1b[%d;1H%*s' "$y" "$COLUMNS" ''; done; printf '\x1b8\x1b[0m'
      rm -f "$FORGUM_RUNTIME/daemon.json"
    fi
  fi
}
PROMPT_COMMAND="__forgum_precmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

This is the safety net that makes the feature bulletproof even against `kill -9`.

---

## 10. Daemon spawn (truly detached, PID-tracked)

```rust
fn spawn_daemon(config: SceneConfig) -> io::Result<u32> {
    let exe = env::current_exe()?;
    let mut cmd = Command::new(exe);
    cmd.arg("--daemon").stdin(Stdio::null())
       .stdout(Stdio::null()).stderr(log_file?);

    #[cfg(unix)] {
        use std::os::unix::process::CommandExt;
        unsafe { cmd.pre_exec(|| { libc::setsid(); libc::umask(0o077); Ok(()) }); }
    }
    #[cfg(windows)] {
        use std::os::windows::process::CommandExt;
        const DETACHED: u32 = 0x00000008; const NEW_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED | NEW_GROUP);
    }

    let child = cmd.spawn()?;
    let pid = child.id();
    // write pid + ob_y1 + cols to runtime_dir/daemon.json for StopDaemon & sweep
    write_daemon_state(pid, config.overlay_height, config.cols)?;
    // don't wait — detach
    std::mem::forget(child);
    Ok(pid)
}
```

- `setsid()` detaches from the controlling terminal's process group → survives shell exit (BUG-B7).
- `stdin(Stdio::null())` → the daemon never touches the shell's input.
- `stdout/stderr → log file` → no stray bytes on the user's terminal.
- `daemon.json` → `StopDaemon`, `sweep`, and `forgum daemon status` all read it.

---

## 11. Rendering correctness & performance

### 11.1 Fix damage tracking (BUG-E1)

Remove `dirty` from `Cell`, or exclude it from `PartialEq`:
```rust
impl PartialEq for Cell {
    fn eq(&self, o: &Self) -> bool {
        self.ch == o.ch && self.fg == o.fg && self.bg == o.bg && self.alpha == o.alpha
    }
}
```
Now a static cow produces `damage_count == 0` after the first frame → the scheduler idles to 5 fps → CPU drops to ~0. The battery thanks you.

### 11.2 Coalesced run rendering (BUG-F1)

Instead of `MoveTo` per cell, walk each row, find contiguous dirty runs with the same color, emit one `MoveTo` + a `print!` of the whole run:
```rust
for y in 0..h {
    let mut x = 0;
    while x < w {
        let i = y * w + x;
        if front[i] == back[i] { x += 1; continue; }      // not dirty
        let run_start = x;
        let mut run = String::new();
        let mut fg = back[i].fg; let mut bg = back[i].bg;
        queue!(out, MoveTo(x as u16, y as u16), SetFg(fg), SetBg(bg))?;
        while x < w && front[y*w+x] != back[y*w+x]
              && back[y*w+x].fg == fg && back[y*w+x].bg == bg {
            run.push(back[y*w+x].ch);
            x += 1;
        }
        queue!(out, Print(run))?;
    }
}
```
Typical full repaint drops from ~12 800 escape sequences to ~a few hundred. SSH users rejoice.

### 11.3 Real `dt` (BUG-B8)

`Instant::now()` diff, clamped to 0.1 s. Effects become framerate-independent.

### 11.4 Tiny-terminal guard

```rust
if cols < 20 || rows < 5 {
    print!("{}", config.cow_text);   // static fallback
    return Ok(());
}
```
No animation on a 1×1 terminal (BUG-E7's spiritual cousin).

### 11.5 Adaptive scheduler (already good, now it can actually idle)

With BUG-E1 fixed, the scheduler's idle tier (5 fps) finally engages for static content. For active content it stays at the target fps. The `should_render(damage)` skip + `adapt(written)` feedback are kept. Add: on `damage_count == 0` for 60 consecutive frames, drop to **1 fps** "heartbeat" that re-checks size + control socket but doesn't render.

---

## 12. Interaction with the foreground (full-screen) mode

Foreground mode is the simpler case (owns the alternate screen). Only two differences from background:

1. **Full canvas** — `ob_y1 = rows` (BUG-B5). No prompt guard.
2. **Raw mode + alt screen + cursor hide** — guarded by RAII; input via crossterm events is legitimate here because the user isn't typing at a shell.
3. **Exit keys** — `q`/`Esc`/`Enter`/`Ctrl+C` work (raw mode delivers them as key events).

Foreground is what `forgum run --no-background` uses. It must also install the signal handler (BUG-T1) so SIGHUP doesn't strand the terminal.

---

## 13. Verification plan (how we prove it works)

These tests live in `08-TESTING-STRATEGY.md` §3 (E2E tier). The overlay-specific ones:

1. **Prompt-row assertion** — capture 5 s of ANSI from a background render under `xvfb-run tmux`; assert no `MoveTo` with `y >= ob_y1` ever appears in the stream. Catches BUG-B1/B3/B4.
2. **Cursor-save/restore balance** — assert every `ESC 7` has a matching `ESC 8` before the next frame's `ESC 7`. Catches cursor leaks.
3. **Static-cow idle** — render a `static` effect for 60 s; assert avg CPU < 1 %. Catches BUG-E1.
4. **Resize mid-animation** — send `SIGWINCH` after 2 s; assert the overlay reflows and no bytes land below `ob_y1` (post-resize). Catches BUG-B3/B4/F2.
5. **Signal exit cleanliness** — `kill -TERM` the daemon; capture the tail of the stream; assert it ends with overlay-clear + `ESC[0m`. Catches BUG-T1/B4.
6. **Keystroke pass-through** — under a pty, type `echo hi\n` into the shell while a background animation runs; assert the shell receives and executes `echo hi`. Catches BUG-B1 definitively.
7. **`kill -9` sweep** — `kill -9` the daemon, then trigger `precmd`; assert the overlay is cleared. Catches the §9.3 net.
8. **Duration=0 infinite** — render with `duration:0`, assert still alive after 10 s. Catches BUG-B2.

---

## 14. The end-state API surface

```bash
# Live wallpaper (infinite, background, above prompt)
forgum daemon start --effect aurora --cow dragon
forgum daemon effect ember          # hot-swap
forgum daemon speed 1.5
forgum daemon pause / resume
forgum daemon status
forgum daemon stop

# One-shot (12 s, background)
forgum run --effect shatter --cow tux --duration 12

# Full-screen interactive (q to quit)
forgum run --effect plasma --cow default --no-background
```

All of these work, leave the terminal pristine, and don't steal a single keystroke.

---

**Next:** `04-PROMPT-INTEGRATION.md` — the shell-hook side: how bash/zsh/fish/pwsh integrate so `forgum` "just works" with correct cows, eyes, lolcat, and config — and never breaks the prompt.
