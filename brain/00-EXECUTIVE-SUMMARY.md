# Forgum — Executive Summary

> **The 5-minute briefing.** What Forgum is, what's broken, and what to do — in priority order — to make it a robust, cross-platform, prompt-safe, genuinely cool terminal companion.

---

## What Forgum is

A **cross-platform PowerShell module** (cowsay + fortune + lolcat) backed by a **Rust ANSI framebuffer animation engine** (`forgum-engine`). The flagship promise: animated ASCII cows render **as a non-blocking overlay above the shell prompt** while the prompt stays fully interactive. Ships 109 cows, 19 effects, and shell hooks for bash/zsh/fish/tmux. v1.1.2 / engine v0.3.0.

## What's broken (the headlines)

1. **The background overlay engine doesn't really work.**
   - It **steals keystrokes** from the shell — `event::poll`/`event::read` on `/dev/tty` competes with the shell; pressing Enter to run a command instead exits the animation. *(BUG-B1)*
   - `duration=0` runs **150 frames (~5 s)** instead of infinite, contradicting the spec. *(BUG-B2)*
   - **No resize handling** in the background loop → resizing the terminal corrupts the display. *(BUG-B3)*
   - `Cell::dirty` participates in `PartialEq` → damage tracking is broken → the scheduler **never idles, 60 fps forever**, even for a static cow. CPU/battery drain. *(BUG-E1)*

2. **The shell hooks (bash/zsh/fish) are non-functional.**
   - Hand-rolled JSON **doesn't escape backslashes** → invalid JSON for almost every cow (cow art is full of `\`). *(BUG-S1)*
   - Fish hook: `'\n'` is literal, not a newline → invalid JSON. *(BUG-S2)*
   - Hooks read the **wrong config key** (`effect` vs `animation.mode`) → always fall back to `random`. *(BUG-S3)*
   - Hooks call the **system `cowsay`** instead of Forgum's 109-cow library. *(BUG-S4)*

3. **Terminal corruption on abnormal exit.** No signal handlers; `Terminal::Drop` is a no-op; `InvokeEngine.ps1` `Kill()`s the engine → raw mode / alternate screen / hidden cursor left on after panic, SIGHUP, SIGTERM, or the 30 s watchdog. Users must `reset` / `stty sane`. *(BUG-T1/T2/T3)*

4. **Cross-platform gaps.** `InvokeEngine.ps1` uses `cmd /c` (Windows-only) on Linux/macOS; `GetShell` misidentifies WSL/Git Bash as `pwsh`; the Windows `CONOUT$` helper is dead code; no ARM64 prebuilt binaries. *(BUG-C1/S8/B9/C6)*

5. **Daemon lifecycle is broken.** `StartDaemon` launches `--daemon` with **no config** → exits immediately; the child **isn't detached** → dies with the shell; `StopDaemon` kills **all** `forgum-engine` processes; temp files leak. *(BUG-D1/B7/D3/D2)*

6. **Cow-specific styling is ignored.** `style_matcher` defines per-cow `speed` and `particles` (dragon fire, dolphin bubbles, nyan stars) but `create_effect` discards them — every cow animates identically. *(BUG-E3)*

The full **~70-issue catalog** (the repo's 32-item audit + a fresh 45-finding deep analysis, deduplicated) with concrete fixes is in `01-BUGS-AND-ISSUES.md`.

## The plan, in one breath

Extract a **`forgum-platform`** crate so all `#[cfg]` lives in one place → add **RAII guards + signal handlers** so the terminal is always restored → **delete keystroke-reading** from the background loop and exit via duration/signals/control-socket → fix `duration=0` and the `Cell::dirty` PartialEq → give the engine a **real CLI (clap)** that renders cows natively, reads the config itself, and generates correct shell hooks + completions → make the daemon **truly detached** with a PID file → add a **per-session control socket** → then layer **tmux/zellij/wezterm** integration, a **`herd` fleet manager**, **remote/follow-me sync**, **reactive effects**, and a **`forgum showcase`** reel.

## Critical path (~3 weeks)

```
Phase 0  stop-the-bleeding     (RAII, signals, no input reads, duration=0, Cell::dirty)   ── 3–4 d
   │
Phase 1  make the overlay work  (open_render_output, resize, detach, control socket)      ── 4–5 d
   │
Phase 2  shell hooks that work  (engine CLI, native cow renderer, --config, init, sweep)  ── 5–6 d
   │
   ├──► Phase 3  effects & particle polish                                                ── 4–5 d
   ├──► Phase 4  cross-platform hardening (forgum-platform, CI matrix)                    ── 3–4 d
   └──► Phase 5  multiplexer integration (tmux/zellij/wezterm)                            ── 5–6 d
            │
            └──► Phase 6  herder + remote (forgum herd, rmux)                              ── 5–6 d
                        │
                        └──► Phase 7  packaging & distribution                             ── 3–4 d
                                  │
                                  └──► Phase 8  make it cooler (ongoing)
```

Full task breakdown in `07-DEVELOPMENT-ROADMAP.md`.

## Cross-platform, concretely

**Tier-1:** Windows 11 · macOS 14+ (Intel + Apple Silicon) · Ubuntu/Debian/Fedora/Arch (x86_64 + aarch64) · pwsh 7 / bash 5 / zsh 5.8 / fish 3 · Windows Terminal / iTerm2 / Alacritty / kitty / gnome-terminal / tmux 3.3+.

- A **`forgum-platform`** crate is the *only* place `#[cfg]` appears — enforced by a CI grep (`rg '#\[cfg' engine/src/` must return 0 hits).
- CI builds **7 prebuilt binaries** (incl. universal macOS via `lipo`, and static musl for Alpine).
- An **`xvfb-run tmux` E2E harness** asserts the prompt row is never touched and keystrokes aren't stolen — the single test that would have caught BUG-B1/B3/B5.

Details in `02-CROSS-PLATFORM-STRATEGY.md` and `08-TESTING-STRATEGY.md`.

## The "cool" part (tmux / rmux / herdr)

- **tmux — 4 surfaces:** pane overlays, a living status line, a `Ctrl-f` popup cow (`display-popup`), and focus-aware auto start/stop via `set-hook`.
- **zellij / wezterm / screen** — each gets a tailored `install` + status surface.
- **rmux (remote)** — a Forgum daemon that **follows you across SSH** (via `RemoteForward` of the control socket) and **syncs** to all peers on a shared tmux session (deterministic effects → ~10 B/s sync traffic).
- **herdr (`forgum herd`)** — a fleet manager: `list / stop / effect / speed / pause / resume` across every daemon in every pane/session/host; `follow` (only the focused pane animates); `theme rotate` (mood cycling); `census` (watchdog + auto-restart); and a `--watch` live dashboard in a tmux popup.
- **Plus:** `forgum demo`, `forgum battle`, `forgum pet`, audio-reactive aurora, CPU-load ember, AI fortunes, and a `forgum showcase` reel.

Details in `05-TMUX-MULTIPLEXER-INTEGRATION.md` and `09-MAKE-IT-COOLER.md`.

## What to do right now

1. Open `01-BUGS-AND-ISSUES.md` §11 (the priority matrix). Start with the **P0** row.
2. The first three concrete commits should be: (a) RAII guards + signal handler (`BUG-T1/T2`), (b) delete `event::poll`/`event::read` from `render_loop_background` (`BUG-B1`), (c) fix `duration=0` + `Cell::dirty` PartialEq (`BUG-B2/E1`). Each is < 1 day and unblocks the rest.
3. Stand up the `xvfb-run tmux` E2E harness (`08-…` §4) early — it's the safety net that lets you refactor aggressively without regressing the terminal.
