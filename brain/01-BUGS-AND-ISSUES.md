# Forgum — Bugs & Issues (Comprehensive Catalog with Fixes)

> **Audience:** Senior Rust/PowerShell engineers.
> **Scope:** Every confirmed defect in `repo/` (Forgum v1.1.2, engine v0.3.0) across the Rust engine, the PowerShell module, the shell hooks, the installers, and the CI surface.
> **Sources:** (A) The repo's own `docs/AUDIT-2026-06-20.md` (32 items, partially fixed), (B) a fresh 45-finding deep audit of the Rust engine + shell-integration layer (`ANALYSIS-1`), (C) manual review of `engine/src/main.rs`, `Cargo.toml`, `handle_init`, the shell completion scripts, and `install.sh`.
> **Convention:** Each entry has a stable ID, severity, file:line, root cause, cross-platform impact, and a concrete fix. Where the repo audit already applied a fix, the entry is marked **[PARTIALLY-FIXED]** with what remains.

---

## Severity legend

| Level | Meaning |
|-------|---------|
| 🔴 **CRITICAL** | Breaks a flagship feature, corrupts the terminal, leaks resources, or makes the tool unusable on a supported platform. Fix before any release. |
| 🟠 **HIGH** | Major UX/correctness defect or platform-specific failure. Fix in the next minor. |
| 🟡 **MEDIUM** | Degraded experience, perf cliff, or latent correctness risk. Fix in the next patch. |
| 🟢 **LOW** | Polish, dead code, missing validation. Fix opportunistically. |

---

## Table of contents

- [1. Terminal-state safety & abnormal exit](#1-terminal-state-safety--abnormal-exit)
- [2. Background overlay engine (the "above-the-prompt" flagship)](#2-background-overlay-engine-the-above-the-prompt-flagship)
- [3. Shell hooks (bash / zsh / fish / pwsh)](#3-shell-hooks-bash--zsh--fish--pwsh)
- [4. JSON protocol & daemon lifecycle](#4-json-protocol--daemon-lifecycle)
- [5. Effects & particle system](#5-effects--particle-system)
- [6. Framebuffer, scheduler & rendering perf](#6-framebuffer-scheduler--rendering-perf)
- [7. Cross-platform correctness](#7-cross-platform-correctness)
- [8. PowerShell module internals](#8-powershell-module-internals)
- [9. Security & input validation](#9-security--input-validation)
- [10. Installers, package managers & completions](#10-installers-package-managers--completions)
- [11. Consolidated fix priority matrix](#11-consolidated-fix-priority-matrix)

---

## 1. Terminal-state safety & abnormal exit

### 🔴 BUG-T1 — No signal handlers; terminal left corrupted on SIGHUP/SIGTERM/SIGINT
- **Files:** `engine/src/main.rs` (no signal hook anywhere), `engine/Cargo.toml` (no `signal-hook`).
- **Symptom:** Closing the terminal window, SSH disconnect, or `kill <pid>` leaves the foreground shell in **raw mode + alternate screen + hidden cursor**. The background overlay leaves stale cow art on screen. Users must `reset` / `stty sane`.
- **Root cause:** No `SIGINT/SIGHUP/SIGTERM` handlers. Default signal disposition terminates immediately; Rust `Drop` impls do **not** run on signal-termination (only on `panic` unwind). `SIGKILL` is uncatchable, but `SIGHUP/SIGTERM/SIGINT` are all catchable.
- **Cross-platform:** Unix: SIGHUP on terminal close, SIGTERM on `kill`. Windows: no SIGHUP, but `SetConsoleCtrlHandler` for Ctrl-Break / console-close is unhandled.
- **Fix:**
  1. Add `signal-hook = "0.3"` (Unix) + `windows-sys` console handler (Windows).
  2. Install a flag-based handler:
     ```rust
     use signal_hook::{consts::{SIGINT, SIGTERM, SIGHUP}, flag};
     let running = Arc::new(AtomicBool::new(true));
     flag::register(SIGINT,  Arc::clone(&running))?;
     flag::register(SIGTERM, Arc::clone(&running))?;
     #[cfg(unix)] flag::register(SIGHUP, Arc::clone(&running))?;
     ```
  3. Check `!running.load(Ordering::Relaxed)` at the top of the render loop → break → run cleanup.
  4. On Windows, `SetConsoleCtrlHandler` sets the same `AtomicBool`.

### 🔴 BUG-T2 — `Terminal::Drop` is a no-op; raw mode left on after panic
- **Files:** `engine/src/terminal.rs:64-70`, `engine/src/main.rs:141`.
- **Symptom:** A panic inside any effect (array OOB, divide-by-zero) leaves the terminal in raw mode with the alternate screen active and cursor hidden.
- **Root cause:** `Terminal::detect()` sets `raw_mode_enabled: false`. `enable_raw_mode()` is called as a **free function** in `main.rs:141`, never through a `Terminal` method that would flip the flag. The `Drop` impl checks `if self.raw_mode_enabled` — always false — so it never calls `disable_raw_mode()`. The guard is dead code.
- **Fix:** Use RAII guards, not a flag on a struct you don't control:
  ```rust
  struct RawModeGuard;
  impl Drop for RawModeGuard {
      fn drop(&mut self) { let _ = crossterm::terminal::disable_raw_mode(); }
  }
  // in render_loop_foreground:
  crossterm::terminal::enable_raw_mode()?;
  let _raw_guard = RawModeGuard;                       // restores on panic/return
  execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
  let _alt_guard = AltScreenGuard;                     // LeaveAlternateScreen on drop
  let _cursor_guard = CursorShowGuard;                 // cursor::Show on drop
  ```
  This guarantees cleanup even on `panic = "unwind"`. For `panic = "abort"`, combine with the signal handler above.

### 🟠 BUG-T3 — `InvokeEngine.ps1` kills the engine with `Kill()` (SIGKILL) — no cleanup
- **Files:** `Private/Animation/InvokeEngine.ps1:72-76`.
- **Symptom:** The 30s watchdog calls `$proc.Kill()`. On Unix that's `SIGKILL`; the engine's cleanup (disable raw mode, leave alt screen, clear overlay, show cursor) never runs → corrupted terminal.
- **Root cause:** Ungraceful kill with no `SIGTERM` first. Also the 30s timeout is shorter than the default `duration: 150` (150 s), so it *always* fires for foreground animations.
- **Fix:**
  1. Graceful-first: Unix → `kill -TERM $proc.Id`; Windows → `$proc.CloseMainWindow()` or generate `CTRL_C_EVENT`.
  2. Wait 2 s, then `Kill()` only as last resort.
  3. Raise timeout to `duration + 5s`.
  4. After any kill, force-restore terminal:
     ```powershell
     [Console]::Out.Write("`e[?25h`e[0m`e[?1049l`e[?25h")  # show cursor, reset, leave alt screen
     ```

---

## 2. Background overlay engine (the "above-the-prompt" flagship)

> This is the feature the user most cares about. It is currently **non-functional in several ways**.

### 🔴 BUG-B1 — Background loop steals keystrokes from the shell
- **Files:** `engine/src/main.rs:278-291`.
- **Symptom:** While a background animation runs, characters the user types at the prompt are swallowed, or pressing **Enter** to submit a command instead **exits the engine**. The shell feels broken.
- **Root cause:** The background loop calls `event::poll(0)` + `event::read()` every frame. On Unix, crossterm opens `/dev/tty` for event reads (stdin is the JSON pipe) — `/dev/tty` is the shell's controlling terminal, so the engine and shell race for the same keystroke stream. On Windows, `ReadConsoleInput` drains the console input queue the shell needs. The `KeyCode::Enter` match arm kills the animation when the user just wanted to run a command.
- **Fix:** **Remove `event::poll`/`event::read` entirely from `render_loop_background`.** Background mode must never read input (the plan already says "never enter raw mode"). Exit only via:
  - `max_frames` (duration timeout),
  - signal (SIGTERM/SIGHUP — BUG-T1),
  - a control socket: a Unix domain socket / named pipe at `$XDG_RUNTIME_DIR/forgum-<session>.sock` that accepts `STOP`/`PAUSE`/`RESUME`/`EFFECT <name>`,
  - or a sentinel file (`~/.forgum/stop`).
  This also fixes BUG-B6 (Ctrl+C doesn't work in background).

### 🔴 BUG-B2 — `duration=0` runs 150 frames, not infinite (spec violation)
- **Files:** `engine/src/main.rs:158-162` (fg), `:267-271` (bg).
- **Symptom:** The plan (`PLAN-2026-06-20-BACKGROUND-ENGINE.md` §6) says `duration=0` → "loop forever until q/Esc/Enter/Ctrl+C." The code maps `0 → 150 frames` (~5 s at 30 fps), so background animations die almost immediately.
- **Root cause:**
  ```rust
  let max_frames = if config.duration.unwrap_or(0) == 0 { 150 } else { duration * fps };
  ```
- **Fix:**
  ```rust
  let max_frames = match config.duration.unwrap_or(0) {
      0 => 0,                          // 0 => infinite
      n => (n as u64).saturating_mul(fps as u64),
  };
  // loop:
  if max_frames > 0 && frame_count >= max_frames { break; }
  ```
  (The `max_frames > 0` guard already exists at lines 217/322, so this Just Works once the mapping is fixed.)

### 🔴 BUG-B3 — Background loop has **no `Event::Resize` handling**
- **Files:** `engine/src/main.rs:278-291` (only `Event::Key`, catch-all `_ => {}`).
- **Symptom:** Resizing the terminal (or a tmux pane) during a background animation leaves the framebuffer at the old size; the engine writes `MoveTo` beyond new bounds → terminal scrolls / wraps / corrupts. Cow art stays at stale offsets.
- **Fix:** Add a `Resize` arm mirroring the foreground loop, and update the captured `cols`/`rows`/`ob_y1` (make them `let mut`):
  ```rust
  Event::Resize(nc, nr) => {
      fb.resize(nc as usize, nr as usize);
      region_alloc.resize_canvas(Rect::new(0,0,nc,nr));
      let nob_y1 = overlay_height.min(nr.saturating_sub(3)).max(1);
      region_alloc.resize_region(overlay_id, Rect::new(0,0,nc,nob_y1));
      effect.on_resize(nc as usize, nr as usize);
      cols = nc; rows = nr; ob_y1 = nob_y1;
  }
  ```

### 🟠 BUG-B4 — Background cleanup uses stale `ob_y1` → corrupts prompt on resize
- **Files:** `engine/src/main.rs:262` (let binding), `:331-348` (cleanup loop).
- **Symptom:** After a resize, the cleanup loop writes spaces to rows `0..ob_y1` with the *old* `ob_y1`. If the terminal shrank below `ob_y1`, `MoveTo(0, y>=rows)` scrolls the prompt away.
- **Fix:** Re-query size before cleanup and clamp:
  ```rust
  let (_, cur_rows) = get_terminal_size();
  let clean_y1 = ob_y1.min(cur_rows);
  for y in 0..clean_y1 { /* clear */ }
  ```
  (Depends on BUG-B3 so `ob_y1` is tracked correctly.)

### 🟠 BUG-B5 — Foreground mode clips the bottom 3 rows (copy-paste from background)
- **Files:** `engine/src/main.rs:153-155`, `engine/src/terminal.rs:49-61`.
- **Symptom:** Full-screen foreground mode (alternate screen, no prompt) still reserves 3 rows → blank band at the bottom. On a 3-row terminal, nothing renders.
- **Root cause:** `render_loop_foreground` calls `term.overlay_bounds()` which returns `y1 = rows - 3`. Foreground mode owns the whole alternate screen; it should use the full canvas.
- **Fix:** In foreground, allocate `Rect::new(0, 0, cols, rows)` directly. Don't call `overlay_bounds()`.

### 🟡 BUG-B6 — Ctrl+C doesn't work in background (no raw mode → no key event)
- **Files:** `engine/src/main.rs:284`.
- **Symptom:** Ctrl+C goes to the shell, not the engine; user can't stop the overlay.
- **Fix:** Removed by BUG-B1 (no input reads in background). Exit via signal handler (BUG-T1) + control socket.

### 🟡 BUG-B7 — Daemon child not truly detached; dies with parent shell
- **Files:** `engine/src/main.rs:489-503`.
- **Symptom:** Background animations vanish when the shell exits or the SSH session closes.
- **Root cause:** `Command::new(exe).arg("--daemon").spawn()` — no `pre_exec(setsid)` on Unix, no `DETACHED_PROCESS`/`CREATE_NEW_PROCESS_GROUP` on Windows. The child stays in the parent's process group and receives SIGHUP.
- **Fix (Unix):**
  ```rust
  #[cfg(unix)]
  use std::os::unix::process::CommandExt;
  unsafe { child.pre_exec(|| { libc::setsid(); Ok(()) }); }
  ```
  Also `stdin(Stdio::null())` after writing the JSON (not `piped()` left dangling), `stdout`/`stderr` to the tty or a log file. Write the child PID to `$XDG_RUNTIME_DIR/forgum-<session>.pid` so `StopDaemon` can target it (BUG-D3).
- **Fix (Windows):** use `CommandExt::creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)`.

### 🟡 BUG-B8 — `dt` hardcoded to 0.016 → animation speed varies with framerate
- **Files:** `engine/src/main.rs:167` (fg), `:276` (bg).
- **Symptom:** When the scheduler drops to idle (5 fps), effects advance 12× slower. On a slow box at 20 fps, effects run slower than intended. Aurora/Plasma/physics all drift.
- **Fix:** Measure real dt:
  ```rust
  let mut last = Instant::now();
  // each frame:
  let now = Instant::now();
  let dt = (now - last).as_secs_f32().min(0.1);  // clamp after stalls
  last = now;
  effect.update(dt);
  ```

### 🟡 BUG-B9 — `console_out` (CONOUT$) is dead code; Windows piped-stdout gets no animation
- **Files:** `engine/src/main.rs:39-68` (defined), `:138,252` (always `None`).
- **Symptom:** On Windows, when PowerShell pipes JSON (`$json | forgum-engine`), `is_terminal_stdout()` is false → engine prints `cow_text` and exits. The `open_console_out()` helper that would open `CONOUT$` is never called. (Unix equivalent: open `/dev/tty`.)
- **Fix:** Call it when stdout isn't a tty:
  ```rust
  let mut console_out = if !is_terminal_stdout() {
      #[cfg(windows)] { open_console_out() }
      #[cfg(unix)]    { open_tty_out() }     // open("/dev/tty", O_WRONLY)
  } else { None };
  ```
  Then the `if let Some(ref mut con) = console_out` branches actually fire. This also lets `InvokeEngine.ps1` drop its `cmd /c --file` workaround (BUG-C1).

---

## 3. Shell hooks (bash / zsh / fish / pwsh)

> The shell-hook layer is **the most broken part of Forgum**. Bash/zsh/fish users effectively get nothing.

### 🔴 BUG-S1 — Bash/zsh hooks produce **invalid JSON** (backslashes not escaped)
- **Files:** `Private/GetForgumShellHook.ps1:34-36`, `engine/src/main.rs:388-391`.
- **Symptom:** `forgum` in bash/zsh fails with `Failed to parse config` for almost every cow (default, tux, dragon… all contain backslashes). No animation renders.
- **Root cause:** The hook escapes newlines and quotes but **not backslashes**. Cow art is full of `\` (legs `\)`, `\\___`…). In JSON, `\)` is an invalid escape.
- **Fix:** Escape backslashes **first** (so the `\` inserted by later escapes isn't re-escaped):
  ```bash
  json_cow="${cow//\\/\\\\}"          # \ -> \\   (MUST be first)
  json_cow="${json_cow//$'\n'/\\n}"   # newline -> \n
  json_cow="${json_cow//\"/\\\"}"     # " -> \"
  ```
  **Better:** stop hand-rolling JSON. Pipe through `jq -Rs .` (produces a correctly-escaped JSON string) or `python3 -c 'import json,sys;print(json.dumps(sys.stdin.read()))'`. Detect availability at hook-generation time and embed the right strategy.

### 🔴 BUG-S2 — Fish hook produces invalid JSON (`'\n'` is literal, not newline)
- **Files:** `Private/GetForgumShellHook.ps1:60`, `engine/src/main.rs:405-406`.
- **Symptom:** `forgum` in fish always fails JSON parse.
- **Root cause:** Fish single-quoted strings don't interpret escapes — `'\n'` is the literal two chars `\` `n`. So `string replace -a '\n' '\\n'` replaces nothing; real newlines stay in the JSON string (forbidden).
- **Fix:** Use `--regex` so `\n` matches an actual newline, and escape backslashes first:
  ```fish
  set json_cow (string replace -a --regex '\\' '\\\\' "$cow")   # first
  set json_cow (string replace -a --regex '\n'  '\\n'  "$json_cow")
  set json_cow (string replace -a --regex '"'   '\\"'  "$json_cow")
  ```
  Or `echo "$cow" | jq -Rs .` if jq is present.

### 🔴 BUG-S3 — Hooks read the wrong config key → always fall back to `random`
- **Files:** `Private/GetForgumShellHook.ps1:21` (`$conf.effect`), `engine/src/main.rs:383` (grep for `"effect"`).
- **Symptom:** User sets `animation.mode: aurora`; hooks ignore it and use `random`.
- **Root cause:** `default-config.json` nests the effect under `animation.mode`. There is **no** top-level `effect`. Both the PowerShell hook and the Rust `grep` look for the wrong key.
- **Fix:** Read `animation.mode`:
  - Rust bash hook: `jq -r '.animation.mode // "random"' "$config"` (preferred), or grep `"mode"` inside the `"animation"` block.
  - PowerShell hook: `$conf.animation.mode`.
  - **Best:** add a `--config <path>` flag to `forgum-engine` and let the **engine** read the config in Rust (single source of truth). The hook then only assembles `cow_text` and passes `--config`.

### 🔴 BUG-S4 — Hooks call system `cowsay`, not Forgum's 109-cow library
- **Files:** `GetForgumShellHook.ps1:33,46,59`, `engine/src/main.rs:388,404,426`.
- **Symptom:** From bash/zsh/fish you can't pick a cow, set eyes/tongue, use lolcat, or use any Forgum feature — the hook shells out to the system `cowsay` binary (which may not be installed).
- **Fix:** Two viable strategies:
  1. **Engine-native cow rendering** (recommended): teach `forgum-engine` to read a `cow_file` field, load `Data/Cows/<file>.cow` (bundle the cow dir next to the binary via `include_dir!` or an install-time copy), expand `$eyes/$tongue/$thoughts`, wrap the fortune in a speech bubble, and render. The hook becomes:
     ```bash
     forgum() {
       local text="$*"; [ -z "$text" ] && text="$(forgum-fortune)"
       printf '{"cow_file":"%s","text":"%s","background":true,"duration":0}' \
              "$(forgum-config .cow.file)" "$(jq -Rs . <<<"$text")" \
         | forgum-engine
     }
     ```
  2. **PowerShell fallback**: `pwsh -NoProfile -Command "Import-Module Forgum; forgum -Text '$*' -CowFile tux -Json"` where `-Json` makes `forgum` emit the engine JSON payload. Slower (pwsh spawn per call) but reuses the full pipeline.

### 🟠 BUG-S5 — `forgum-engine` not guaranteed on PATH
- **Files:** `GetForgumShellHook.ps1:36`, `engine/src/main.rs:391,407,426`.
- **Symptom:** After install, `forgum` in bash prints `forgum-engine: command not found`.
- **Fix:**
  - Unix installer: `ln -s "$MODULE_DIR/bin/forgum-engine" /usr/local/bin/forgum-engine` (or `$HOME/.local/bin`).
  - Windows installer: add `bin/` to user PATH via `setx PATH`.
  - At hook-generation time, **bake the resolved absolute path** into the hook: `| "/opt/forgum/bin/forgum-engine"`.

### 🟠 BUG-S6 — Two hook generators produce inconsistent hooks
- **Files:** `Private/GetForgumShellHook.ps1` vs `engine/src/main.rs:373-433`.
- **Symptom:** `forgum init bash` (PowerShell) and `forgum-engine init bash` (Rust) emit **different** hooks with different behavior.
- **Fix:** Single source of truth. Keep `handle_init` in Rust (ships with the engine, runs without PowerShell), delete the PowerShell generator or make it `forgum-engine init $shell`. Fix BUG-S1/S2/S3/S4 in the Rust generator.

### 🟡 BUG-S7 — `GetForgumShellHook.ps1` bakes the effect at generation time
- **Files:** `GetForgumShellHook.ps1:39,52,65` (`.Replace('EFFECT_PLACEHOLDER', $effect)`).
- **Symptom:** Config changes after `forgum init` are ignored until the hook is regenerated.
- **Fix:** Generate hooks that read the config at runtime (the Rust `handle_init` already attempts this — fix its key per BUG-S3 and standardize on it).

### 🟡 BUG-S8 — `GetShell` misidentifies WSL / Git Bash / Cygwin as `pwsh`
- **Files:** `Private/CrossPlatform.ps1:21-24`.
- **Symptom:** A bash user on WSL runs `forgum init` and gets a **PowerShell** hook.
- **Fix:** Check `$env:SHELL` / parent process first, **even on Windows**:
  ```powershell
  function GetShell {
    $sh = $env:SHELL
    if ($sh) { return (Split-Path $sh -Leaf) }   # bash/zsh/fish, incl. WSL/Git Bash
    if ($IsWindows -or -not $IsCoreCLR) { return 'pwsh' }
    # /proc/$$/comm on Linux, etc.
  }
  ```

---

## 4. JSON protocol & daemon lifecycle

### 🔴 BUG-D1 — `StartDaemon` launches `--daemon` with no config → exits immediately
- **Files:** `Private/StartDaemon.ps1:14-15`.
- **Symptom:** `forgum daemon start` prints "started" but nothing runs.
- **Root cause:** `Start-Process $binary -ArgumentList '--daemon' -NoNewWindow` gives the engine no JSON (no `--file`, no stdin pipe). `main()` reads stdin → either blocks on the console or hits EOF → "No input provided" → exits.
- **Fix:** Write JSON to a temp file and pass `--file`:
  ```powershell
  $cfg = @{ effect='random'; cow_file='default'; background=$true; duration=0 } | ConvertTo-Json -Compress
  $tmp = [System.IO.Path]::GetTempFileName() + '.json'
  $cfg | Out-File $tmp -Encoding utf8
  Start-Process -FilePath $binary -ArgumentList @('--daemon','--file',"`"$tmp`"") -NoNewWindow -PassThru | …
  ```
  Have the engine `remove_file` the `--file` after reading (also fixes BUG-D2).

### 🟠 BUG-D2 — `InvokeEngine.ps1` leaks temp files in the background path
- **Files:** `Private/Animation/InvokeEngine.ps1:33-34` (create), `:39-50` (no `Remove-Item`).
- **Symptom:** Every background invocation orphans a `forgum_engine_*.json` in `$TMPDIR`. Thousands accumulate.
- **Fix:**
  - Engine: `std::fs::remove_file(path)?` right after `read_to_string` in `main.rs:458`.
  - PowerShell: `try/finally` with a delayed `Remove-Item`, and a startup sweep `Remove-Item $env:TEMP/forgum_engine_*.json -EA SilentlyContinue`.

### 🟡 BUG-D3 — `StopDaemon` kills **all** `forgum-engine` processes; no per-session targeting
- **Files:** `Private/StopDaemon.ps1:13-15`. *(Repo audit item #22.)*
- **Symptom:** Two terminals running background animations → `StopDaemon` nukes both. `-Force` skips graceful shutdown.
- **Fix:**
  - Daemon writes its PID to `$XDG_RUNTIME_DIR/forgum-$SESSIONID.pid` (Unix) / `%TEMP%\forgum-<guid>.pid` (Windows).
  - `StopDaemon` reads the PID, sends `SIGTERM` (Unix) / `CloseMainWindow` (Windows), waits 2 s, then `Kill`.
  - Remove `-Force` as default.

### 🟡 BUG-D4 — JSON stdin read has no size cap → OOM on huge payload
- **Files:** `engine/src/main.rs:461`.
- **Fix:** Bounded read:
  ```rust
  const MAX: usize = 4 * 1024 * 1024; // 4 MB is generous for a cow + fortune
  let mut buf = Vec::with_capacity(8 * 1024);
  io::stdin().take(MAX as u64).read_to_end(&mut buf)?;
  let buffer = String::from_utf8(buf)?;
  if buffer.len() >= MAX { return Err(...); }
  ```

### 🟢 BUG-D5 — JSON parse error returns `Ok(())` → exit code 0, caller can't detect failure
- **Files:** `engine/src/main.rs:469-475`. *(Repo audit adjacent.)*
- **Fix:** `std::process::exit(1)` or return `Err(InvalidInput)`. `InvokeEngine.ps1` should check `$proc.ExitCode`.

### 🟢 BUG-D6 — `--file` with no path silently falls through to stdin
- **Files:** `engine/src/main.rs:456-462`.
- **Fix:** `else { eprintln!("--file requires a path"); return Ok(()); }`.

### 🟡 BUG-D7 — `duration * fps` can overflow `u32`
- **Files:** `engine/src/main.rs:161,270`.
- **Fix:** `.saturating_mul()` and compute as `u64`. Cap at e.g. 24 h of frames.

---

## 5. Effects & particle system

### 🔴 BUG-E1 — `Cell::dirty` participates in `PartialEq` → scheduler never idles (60 fps forever)
- **Files:** `engine/src/framebuffer.rs:5-12` (derives `PartialEq` incl. `dirty`), `:26` (`Cell::new` sets `dirty=true`), `:130` (`back != front`), `:152-154` (sets `front.dirty=false`).
- **Symptom:** Even a static cow with no animation burns 60 fps forever → CPU/battery drain, fan spin.
- **Root cause:** Every frame `fb.clear()` resets `back` (`dirty=false`), `effect.render()` calls `Cell::new` (`dirty=true`). `compute_damage` compares `back[i] != front[i]`; since `back.dirty≠front.dirty`, **every** rendered cell is "damaged" even if `ch/fg/bg/alpha` are identical. `damage_count` is always > 0 → scheduler stays in active mode.
- **Fix (pick one):**
  1. Manual `PartialEq` comparing only `ch, fg, bg, alpha`:
     ```rust
     impl PartialEq for Cell {
         fn eq(&self, o: &Self) -> bool {
             self.ch == o.ch && self.fg == o.fg && self.bg == o.bg && self.alpha == o.alpha
         }
     }
     ```
  2. Remove `dirty` entirely (it's never read by `render_region`).
  3. Track damage via a separate `BitVec`/`HashSet<usize>`.
  Recommended: option 2 (simplest, least memory).

### 🟠 BUG-E2 — Effects hardcode `80×40` canvas in `new()` → wrong particle positions
- **Files:** `engine/src/effects.rs:44,86,179,242,285,326,371,427,478,579,619,674,723,775,839,893,972,1020,1064` (all `center_offset(th, tw, 80, 40)`).
- **Symptom:** Shatter/Dissolve spawn particles at construction using an 80×40 canvas; on any other terminal size they start at the wrong place. `on_resize` updates offsets but doesn't re-spawn.
- **Fix:** Either pass real `cols/rows` into `new()`, or defer particle spawning to the first `on_resize()` (add `initialized: bool`). For Shatter/Dissolve, re-spawn on resize.

### 🟠 BUG-E3 — `speed` and `particles` from `style_matcher` are parsed but never applied
- **Files:** `engine/src/style_matcher.rs` (`CowStyle.speed`, `.particles`), `engine/src/protocol.rs:23` (`speed`), `main.rs:111-123` (discards both).
- **Symptom:** Per-cow speeds (dolphin 0.5×, nyan 2.0×…) and themed particles (dragon fire, dolphin bubbles, koala Zzz, nyan stars) never trigger. All cows animate identically.
- **Fix:**
  - `resolve_effect_name` → return `CowStyle { base, speed, particles }`.
  - `create_effect(name, cow_text, speed, particles)` — store `speed`, multiply `dt` by it in `update()`.
  - Add a `ParticleOverlay` wrapper effect that composes themed particles over any base effect. Spawn `Bubbles`/`Stars`/`Zzz`/`Fire` based on `particles`.

### 🟠 BUG-E4 — Shatter/Dissolve go permanently inactive → blank overlay for the rest of `duration`
- **Files:** `engine/src/effects.rs:215,1104` (`if alive_count == 0 { self.active = false; }`).
- **Symptom:** After particles die (~3 s), the overlay is blank for the remaining minutes of `duration`.
- **Fix:** Add `Effect::is_done() -> bool` to the trait; break the render loop when done. Or auto-reset the effect. Or shorten `max_frames` for one-shot effects.

### 🟠 BUG-E5 — `BounceEffect` physics is wrong (missing `0.5`, ~2-row bounce, no squash)
- **Files:** `engine/src/effects.rs:786-808`.
- **Symptom:** "bounce" barely moves; looks like jitter.
- **Fix:** Correct kinematics: `y = base + vy0·t + 0.5·g·t²`. Raise `vy0` (e.g. -20), lower `g` (e.g. 30). Apply squash to the **shape** (render rows twice when squashed, skip alternate rows when stretched), not just color brightness. Don't reset `vy` on small values.

### 🟡 BUG-E6 — `PortalEffect` hue modulo doesn't handle negatives
- **Files:** `engine/src/effects.rs:347-348`.
- **Fix:** `let h = ((expr % 360.0) + 360.0) % 360.0;` and normalize `h` inside `hsv_to_rgb`.

### 🟡 BUG-E7 — Negative-float `as usize` saturates to 0 → particles render at column 0
- **Files:** `engine/src/effects.rs:158,220,951,1109`.
- **Fix:** Bounds-check before casting:
  ```rust
  if x >= 0.0 && y >= 0.0 {
      let (px, py) = (x as usize, y as usize);
      if px < fb.width && py < fb.height { fb.set_cell_in_region(px, py, cell, clip); }
  }
  ```

### 🟡 BUG-E8 — `ParticlePool::spawn` is O(n) per spawn (linear scan)
- **Files:** `engine/src/particles.rs:33`.
- **Fix:** Maintain a free-list (stack of recycled indices). `spawn` pops, death pushes. O(1).

---

## 6. Framebuffer, scheduler & rendering perf

### 🟠 BUG-F1 — `render_region` emits a `MoveTo` before every changed cell (no run coalescing)
- **Files:** `engine/src/framebuffer.rs:131`.
- **Symptom:** Full repaint = ~3200 cells × ~4 escape sequences = ~12 800 sequences/frame. Terminals stutter, especially over SSH.
- **Fix:** Row-run coalescing:
  ```rust
  for y in 0..h {
      let mut x = 0;
      while x < w {
          if !dirty(x,y) { x += 1; continue; }
          // find run end
          let mut end = x;
          while end < w && dirty(end,y) && same_color(end,y,x,y) { end += 1; }
          queue!(out, MoveTo(x as u16, y as u16))?;
          for cx in x..end { write_cell(out, cx, y)?; }
          x = end;
      }
  }
  ```
  Also keep `last_fg/last_bg` across the whole `render_region` call (not per cell).

### 🟡 BUG-F2 — Background mode doesn't clear old overlay pixels after resize
- **Files:** `engine/src/main.rs:294` (`fb.clear()` clears the buffer, not the screen) + BUG-B3.
- **Fix:** On resize, write spaces to the **old** bounds on-screen before reallocating `fb`.

### 🟢 BUG-F3 — `WriteTerminalFrame.ps1` uses `Write-Host` in TTY mode (breaks pipelines) *(repo audit #17)*
- **Files:** `Private/WriteTerminalFrame.ps1:54`.
- **Fix:** Use `[Console]::Out.Write()` so ANSI sequences still render but stdout is capturable. Document the limitation.

---

## 7. Cross-platform correctness

### 🟠 BUG-C1 — `InvokeEngine.ps1` foreground path uses `cmd /c` → broken on Linux/macOS
- **Files:** `Private/Animation/InvokeEngine.ps1:54`.
- **Symptom:** Foreground animation on macOS/Linux PowerShell Core: `cmd: command not found`.
- **Fix:** `& $enginePath --file $tmpFile` directly. If `cmd /c` is needed on Windows for console-handle inheritance, gate it: `if ($IsWindows) { & cmd /c … } else { & $enginePath --file $tmpFile }`. Once BUG-B9 lands (CONOUT$/`/dev/tty`), remove the workaround entirely.

### 🟡 BUG-C2 — `InvokeForgumTUI.ps1` "open config" uses `Start-Process -Verb Open` → Windows-only
- **Files:** `Private/InvokeForgumTUI.ps1:169`.
- **Fix:** `Invoke-Item $path`, or platform branch: `xdg-open` (Linux), `open` (macOS).

### 🟡 BUG-C3 — `GetForgumConfigPath` ignores `$env:Forgum_CONFIG` *(repo audit #27)*
- **Files:** `Private/CrossPlatform.ps1:43-58` vs `Private/GetConfigPath.ps1:19-21`.
- **Fix:** Delete `GetForgumConfigPath`; use `GetConfigPath` everywhere (it honors the env var).

### 🟡 BUG-C4 — 10 animation files split on `` `n `` instead of `` `r?`n `` *(repo audit #11, [PARTIALLY-FIXED])*
- **Fix:** Verify all animation files now use `` `r?`n ``. The fix was applied; re-audit on next change.

### 🟢 BUG-C5 — `winapi` crate is legacy; prefer `windows-sys`
- **Files:** `engine/Cargo.toml:16`, `engine/src/main.rs:42-43`.
- **Fix:** Migrate `open_console_out` to `windows-sys::Win32::Storage::FileSystem` + `Console` APIs. `winapi` is unmaintained.

### 🟢 BUG-C6 — No ARM64 pre-built binaries in releases
- **Files:** `.github/workflows/*`, `package-managers/homebrew/forgum.rb`.
- **Fix:** Add `aarch64-apple-darwin` and `aarch64-unknown-linux-gnu` targets to the release matrix (see `08-TESTING-STRATEGY.md`). Homebrew formula should dispatch on `Hardware::CPU.arch`.

---

## 8. PowerShell module internals

*(These are the repo audit items not already covered. Items marked [FIXED] were applied 2026-06-20.)*

| ID | Repo # | Severity | File:Line | Issue | Status / Remaining work |
|----|--------|----------|-----------|-------|--------------------------|
| P1 | 1 | 🔴 | `Get-HelpMessage.ps1:573` | `'h'` alias → `'version'` not `'root'` | [FIXED] added `'h'='root'` |
| P2 | 2 | 🔴 | `Invoke-ForgumRun.ps1:30-36` | Double output: render + pipeline return | **OPEN** — gate pipeline return on `-Passthru` only |
| P3 | 3 | 🔴 | `Read-CowFile.ps1:24-26` | No path-traversal validation on `$CustomPath` | [FIXED] — re-verify rejects `..`/abs paths |
| P4 | 4 | 🔴 | `forgum.ps1:96-97` | Dead code after `default` | [FIXED] |
| P5 | 5 | 🔴 | `Forgum.psm1:182-227` | Tab completion for non-existent params | [FIXED] |
| P6 | 6 | 🔴 | `Set-Forgum.ps1:28` | Stale `ValidateSet` missing flagship modes | [FIXED] — keep in sync with engine |
| P7 | 7 | 🔴 | `Bounce.ps1:75` | `IndexOf` instead of loop var | [FIXED] |
| P8 | 8 | 🔴 | `Blink.ps1:52-53` | Blink never fires (modulo 2 ≠ 3) | [FIXED] |
| P9 | 9 | 🔴 | `Invoke-Engine.ps1:70-80` | Background process never disposed | [FIXED] try/finally + Dispose + WaitForExit — but see BUG-T3/D2 |
| P10| 10 | 🔴 | `Invoke-ForgumExport.ps1:19,43` | Path traversal in `--output` | [FIXED] |
| P11| 12 | 🟠 | `Write-TerminalFrame.ps1:57` | `Write-Host` in non-TTY | see BUG-F3 |
| P12| 13 | 🟠 | Gallery/History/Live/List `[int]` casts | Unvalidated | [FIXED] `TryParse` |
| P13| 14 | 🟠 | `Forgum.psm1:120-131` | Config rewrite + process spawn on every import | [FIXED] |
| P14| 15 | 🟠 | `Set-CFConfig.ps1:39` | Predictable temp-file race | [FIXED] random name |
| P15| 16 | 🟠 | `Parse-ForgumArguments.ps1:49-53` | Missing value silently dropped | **OPEN** — error on known-option-with-no-value |
| P16| 18 | 🟡 | `Invoke-ForgumTheme.ps1:55-61` | Reset only resets lolcat | **OPEN** — full-config reset |
| P17| 19 | 🟡 | `Set-CFCowAnimate.ps1:16` | No mode validation | **OPEN** — validate against engine list |
| P18| 20 | 🟡 | `Set-CFCowEyes.ps1:30` | Silent no-op on invalid eyes | **OPEN** — error |
| P19| 21 | 🟡 | `Invoke-ForgumEyes.ps1:20` | Silent no-op on invalid preset | **OPEN** — error |
| P20| 23 | 🟡 | `PhysicsCow.ps1:228-230,275-276` | Particle leak in Talk/Fire | [FIXED] |
| P21| 24 | 🟡 | `Talking.ps1:28` | Regex misses most templates | **OPEN** |
| P22| 25 | 🟡 | `Dynamic.ps1:47-49` | Dead no-op guard | **OPEN** |
| P23| 26 | 🟡 | `Invoke-ForgumRun.ps1` | `--fortune` parsed but unused | **OPEN** |
| P24| 28 | 🟢 | `PhysicsCow.ps1:92-100` | O(n²) array append | **OPEN** — preallocate |
| P25| 29 | 🟢 | `Disco.ps1:51-67` | Per-char HSV→RGB | **OPEN** — cache LUT |
| P26| 30 | 🟢 | `Typewriter.ps1:20-24` | 50 ms/char slow | **OPEN** — 25 ms, batch |
| P27| 31 | 🟢 | `Wiggle.ps1:41-43` | 73 % static | **OPEN** |
| P28| 32 | 🟢 | `Procedural.ps1:76` | Snowflake crash when width ≤ 1 | **OPEN** — guard |

### 🟡 BUG-P29 — `InvokeForgumTUI.ps1` lists legacy modes the engine doesn't know
- **Files:** `Private/InvokeForgumTUI.ps1:56`.
- **Symptom:** Selecting `talking/typewriter/wave/wiggle/fade-in/slide-in/disco/blink/dynamic/procedural` silently falls back to aurora.
- **Fix:** Either implement them in Rust or remove from the TUI. At minimum warn on unsupported selection.

### 🟡 BUG-P30 — `GetEngineBinary.ps1` auto-rebuilds on every call when binary missing
- **Files:** `Private/GetEngineBinary.ps1:26-38`.
- **Fix:** Build once at install time. Cache via a sentinel; never rebuild inside the hot path.

---

## 9. Security & input validation

### 🔴 BUG-X1 — Path traversal in cow-file & export paths *(repo audit #3, #10, [FIXED])*
- **Status:** Fixed. **Re-verify** the guard rejects `..`, absolute paths, and symlinks escaping the cow dir. Add a fuzz test.

### 🟡 BUG-X2 — Unbounded stdin (BUG-D4) + no auth on the control socket (planned BUG-B1 fix)
- **Fix:** The control socket must be mode `0600` and live under `$XDG_RUNTIME_DIR` (per-user). Validate commands against an allowlist (`STOP/PAUSE/RESUME/EFFECT <known>/SPEED <n>`).

### 🟢 BUG-X3 — `--file` path not sanitized (symlink to `/dev/null` etc.)
- **Fix:** Reject if the resolved path isn't inside the temp dir or the Forgum data dir.

---

## 10. Installers, package managers & completions

### 🟠 BUG-I1 — `install.sh` doesn't build/symlink the engine
- **Files:** `install.sh`.
- **Symptom:** Unix install leaves the user without `forgum-engine` unless they manually `cargo build`.
- **Fix:** Detect `cargo`; build `--release`; install binary to `$PREFIX/bin`. If no cargo, download a prebuilt release asset for the detected arch.

### 🟡 BUG-I2 — Homebrew formula doesn't ship prebuilt ARM64 bottles
- **Files:** `package-managers/homebrew/forgum.rb`.
- **Fix:** Add `sha256 arm64_monterey: …` bottles; or `depends_on :rust` and build from source.

### 🟢 BUG-I3 — Zsh completion `--mode` list is stale
- **Files:** `scripts/completions/_forgum:27`.
- **Fix:** Sync with `create_effect`. Better: generate completions from `forgum-engine --list-effects`.

### 🟢 BUG-I4 — `InvokeEngine.ps1` dead vars `$escapedPath/$escapedFile`
- **Files:** `Private/Animation/InvokeEngine.ps1:36-37`. Delete.

### 🟢 BUG-I5 — Scoop/winget manifests reference v1.1.2; engine is v0.3.0 — version drift
- **Fix:** Single source `version` in `Cargo.toml` + `Forgum.psd1` + manifests; CI asserts parity.

---

## 11. Consolidated fix priority matrix

| Priority | Bug IDs | Theme | Effort |
|----------|---------|-------|--------|
| **P0 — ship-blockers** | T1, T2, B1, B2, S1, S2, S3, S4, D1, E1 | Terminal safety, background engine works at all, shell hooks work at all, no CPU burn | M–L each |
| **P1 — flagship quality** | B3, B4, B5, B7, B8, B9, D2, D3, E2, E3, E4, E5, C1, S5, S6, T3 | Resize, detach, dt, perf, cow-specific styling, cross-platform invoke | M each |
| **P2 — polish & correctness** | B6, D4, D7, E6, E7, E8, F1, F2, C2, C3, C4, P15–P28, P29, P30, X2 | Edge cases, perf, validation | S–M each |
| **P3 — hygiene** | D5, D6, C5, C6, I1–I5, X3, F3 | Exit codes, deps, packaging, completions | S each |

**Effort key:** S ≤ 1 day · M 1–3 days · L > 3 days.

---

### Open items carried from `docs/AUDIT-2026-06-20.md` not yet fixed

> `#16` (silent option drop), `#18` (theme reset incomplete), `#19` (no mode validation), `#20/#21` (silent no-ops on invalid eyes/preset), `#22` (broad kill — now BUG-D3), `#24` (Talking regex), `#25` (Dynamic dead guard), `#26` (`--fortune` unused), `#28–#32` (perf/crash in legacy PS animations). These are tracked as P15–P28 above.

---

**Next:** see `03-ANIMATION-ENGINE-ABOVE-PROMPT.md` for the design that makes the overlay engine actually work above the prompt, and `04-PROMPT-INTEGRATION.md` for the prompt-safety guarantees.
