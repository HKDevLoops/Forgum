# Forgum — Development Roadmap

> A phased, dependency-ordered plan to take Forgum from "lots of bugs, broken shell hooks, no real background engine" to "the coolest cross-platform terminal companion." Each phase has a definition-of-done and a test gate. Phases are sized so a single focused engineer can complete one in ~1 week.

**Convention:** `[P0]` = blocks release · `[P1]` = flagship quality · `[P2]` = polish · `[dep: X]` = depends on phase X.

---

## Phase 0 — Stop the bleeding (P0, 3–4 days)

**Goal:** no terminal corruption, no CPU burn, no keystroke theft.

| Task | Bug | Deliverable |
|------|-----|-------------|
| 0.1 RAII guards | T2 | `RawModeGuard`/`AltScreenGuard`/`CursorShowGuard` in `forgum-platform`; foreground loop uses them |
| 0.2 Signal handlers | T1 | `signal-hook` (unix) + `SetConsoleCtrlHandler` (win); `ShutdownFlag` checked each frame |
| 0.3 Remove input reads from background loop | B1 | delete `event::poll`/`read` in `render_loop_background` |
| 0.4 Fix `duration=0` → infinite | B2 | `max_frames = 0` semantics + saturating math (D7) |
| 0.5 Fix `Cell::dirty` PartialEq | E1 | manual `PartialEq` on `ch/fg/bg/alpha`; scheduler can idle |
| 0.6 Graceful kill in `InvokeEngine.ps1` | T3 | SIGTERM-first, raise timeout, force-restore escape on kill |
| 0.7 Bounded stdin | D4 | 4 MB cap |
| 0.8 Exit codes on parse error | D5 | `exit(1)` |

**Definition of done:** the 8 verification tests in `03-…` §13 pass under `xvfb-run tmux`. CPU < 1 % for a static cow over 60 s. `kill -TERM` leaves the terminal pristine.

---

## Phase 1 — Make the overlay actually work (P0, 4–5 days)

**Goal:** background animation renders above the prompt, survives resize, exits cleanly, doesn't die with the shell. [dep: 0]

| Task | Bug | Deliverable |
|------|-----|-------------|
| 1.1 `open_render_output()` | B9, C1 | unifies stdout/`/dev/tty`/CONOUT$; removes `cmd /c` |
| 1.2 Background `Resize` handler | B3, F2 | `SIGWINCH` → clear-old → resize fb/region → recenter |
| 1.3 Stale-safe cleanup | B4 | `OverlayGuard` reads live `ob_y1`/`cols` |
| 1.4 Foreground full canvas | B5 | `Rect::new(0,0,cols,rows)` in foreground |
| 1.5 Real `dt` | B8 | `Instant`-based, clamped |
| 1.6 Daemon detach + PID file | B7, D3 | `setsid`/`DETACHED_PROCESS`; `daemon.json` with `{pid,ob_y1,cols,pane,session}` |
| 1.7 Control socket | (new) | `STOP/PAUSE/RESUME/EFFECT/SPEED/COW/STATUS/PING`; per-session path |
| 1.8 `StopDaemon` per-session | D3 | reads `daemon.json`, SIGTERM-first |
| 1.9 Temp file cleanup | D2 | engine `remove_file` after `--file` read; startup sweep |

**Definition of done:** a background animation survives a pane resize, a `kill -TERM`, and the shell exiting (daemon stays). Two panes animate independently; `forgum daemon stop` kills only the current pane's.

---

## Phase 2 — Shell hooks that work (P0, 5–6 days)

**Goal:** `forgum init bash/zsh/fish/pwsh` produces a hook that renders the right cow, eyes, effect, lolcat — read live from config — without breaking the prompt. [dep: 1]

| Task | Bug | Deliverable |
|------|-----|-------------|
| 2.1 Engine CLI (clap) | (new) | `render/fortune/daemon/herd/theme/tmux/init/completions/status-line` subcommands |
| 2.2 Native cow renderer | S4 | load `.cow`, expand `$eyes/$tongue/$thoughts`, speech bubble; bundled via `include_dir!` |
| 2.3 `--config` reading + merge | S3, S7 | engine is the single config reader; `argv > file > defaults` |
| 2.4 Rewrite `handle_init` | S1, S2, S6 | bash/zsh/fish/pwsh hooks using the §4–§7 templates; absolute engine path |
| 2.5 `parent_comm()` shell detect | S8 | WSL/Git Bash/Cygwin correct |
| 2.6 `precmd` sweep in all hooks | (new) | `kill -9` safety net |
| 2.7 Delete `GetForgumShellHook.ps1` | S6 | route `forgum init` through the engine |
| 2.8 Generated completions | I3 | `forgum-engine completions <shell>`; CI drift check |
| 2.9 Installer PATH/symlink | S5, I1 | Unix: `/usr/local/bin`; Windows: PATH; prebuilt binary download |

**Definition of done:** on a fresh bash/zsh/fish/pwsh, `forgum "hello"` renders the configured cow+effect. Changing `animation.mode` in config takes effect on the next call (no re-init). `shellcheck`/`fish --no-execute` clean. The pty keystroke test passes (typing `echo hi<Enter>` runs while a background animation plays).

---

## Phase 3 — Effects & particle polish (P1, 4–5 days)

**Goal:** effects look right, perform well, and use per-cow styling. [dep: 1]

| Task | Bug | Deliverable |
|------|-----|-------------|
| 3.1 Apply `speed` + `particles` | E3, E16/17 | `resolve_effect_name` returns `CowStyle`; `create_effect` accepts speed+particles; themed particle overlays (bubbles/stars/zzz/fire) |
| 3.2 Effect constructors take real size | E2 | pass `cols/rows` to `new()`; Shatter/Dissolve spawn after first `on_resize` |
| 3.3 `Effect::is_done()` | E4 | one-shot effects break the loop |
| 3.4 Bounce physics fix | E5 | correct kinematics + shape squash |
| 3.5 Portal hue modulo | E6 | `((x % 360)+360) % 360` |
| 3.6 Particle bounds check | E7 | skip out-of-bounds before `as usize` |
| 3.7 O(1) particle spawn | E8 | free-list |
| 3.8 Coalesced run rendering | F1 | row-run coalescing + cross-cell color caching |

**Definition of done:** dragon breathes fire, dolphin blows bubbles, nyan trails stars. Shatter exits when done (no blank overlay). Full repaint < 2 ms at 80×40. No particle smear at col 0.

---

## Phase 4 — Cross-platform hardening (P1, 3–4 days)

**Goal:** Tier-1 matrix green in CI. [dep: 1, 2]

| Task | Bug | Deliverable |
|------|-----|-------------|
| 4.1 Extract `forgum-platform` | (arch) | all `#[cfg]` contained; `#[cfg]`-grep CI gate on `engine/src/` |
| 4.2 Config/runtime paths via `directories` | C3 | `$FORGUM_CONFIG`/`$FORGUM_DATA` honored |
| 4.3 `GetShell` WSL fix | S8 (dup) | parent-comm detection |
| 4.4 TUI cross-platform | C2, P29 | `Invoke-Item`; sync mode list with engine |
| 4.5 `GetEngineBinary` no auto-rebuild | P30 | build at install; sentinel cache |
| 4.6 `winapi` → `windows-sys` | C5 | migrate `open_console_out` |
| 4.7 Legacy PS animation line-ending audit | C4 | confirm `` `r?`n `` |
| 4.8 Remaining repo-audit items | P15–P28 | see table in `01-…` §8 |

**Definition of done:** `cargo test` green on all 7 targets. Pester green on Windows + Linux pwsh. `xvfb-run tmux` smoke green. `clippy -D warnings` clean. `#[cfg]`-grep finds zero hits in `engine/src/`.

---

## Phase 5 — Multiplexer integration (P1, 5–6 days)

**Goal:** tmux/zellij/screen/wezterm first-class. [dep: 1, 2]

| Task | Deliverable |
|------|-------------|
| 5.1 `detect_mux()` + per-pane daemon state | `daemon.json` keyed on `$TMUX_PANE`/session |
| 5.2 `forgum tmux install` | writes status-line + hooks + popup keybind (idempotent) |
| 5.3 `forgum-engine status-line` | < 5 ms one-shot; `--live --fps 1` ticker variant |
| 5.4 `--popup` render mode + `forgum tmux popup` | tmux `display-popup` one-shot |
| 5.5 tmux `set-hook` focus-aware | pane-focus-in/out start/stop; `daemon resize` on layout change |
| 5.6 zellij/wezterm/screen installers | ported config templates |

**Definition of done:** `forgum tmux install` in a fresh tmux gives a living status line, focus-aware pane overlays, and `Ctrl-f` popup cow. Resizing a pane reflows the animation. zellij/wezterm/screen each have a working `install` + status surface.

---

## Phase 6 — The herder + remote (P1/P2, 5–6 days)

**Goal:** fleet control + follow-me-across-SSH. [dep: 5]

| Task | Deliverable |
|------|-------------|
| 6.1 `forgum herd list/stop/effect/speed/pause/resume` | scans `daemon.json` files; parallel control-socket fan-out |
| 6.2 `forgum herd follow` | only focused pane animates; others idle |
| 6.3 `--watch` popup dashboard | live herd table in a tmux popup; `e`/`q` keys |
| 6.4 `forgum theme apply/list/rotate` | theme bundles (effect+cow+palette+lolcat); `--rotate N` |
| 6.5 `forgum herd census` | watchdog: sweep dead daemons, optional auto-restart |
| 6.6 `forgum remote attach/sync/who` | SSH reverse-forward of the control socket; deterministic-effect peer sync |

**Definition of done:** 6 panes animate; `forgum herd effect aurora --all` changes all six in < 100 ms. `forgum herd list --watch` shows a live dashboard. SSH to a server with `RemoteForward` and the local cow follows. `forgum demo` runs the §10 showcase.

---

## Phase 7 — Packaging & distribution (P2, 3–4 days)

**Goal:** one-command install on every Tier-1 platform. [dep: 4]

| Task | Deliverable |
|------|-------------|
| 7.1 Release CI matrix | 7 prebuilt binaries + universal macOS; sha256 |
| 7.2 Homebrew formula | universal binary; `brew install forgum` |
| 7.3 Scoop + winget | per-arch manifests; `scoop install forgum` / `winget install HKDEVS.Forgum` |
| 7.4 `.deb` + `.rpm` | `cargo-deb`/`cargo-generate-rpm`; APT/RPM repos |
| 7.5 PowerShell Gallery | `Publish-Module`; engine binary bundled in `bin/` |
| 7.6 `cargo install forgum-engine` | engine-only path for Rust users |
| 7.7 Version parity CI | single source in `Cargo.toml`; asserts parity across manifests |

**Definition of done:** a user on any Tier-1 OS installs with one command and gets a working `forgum` + `forgum-engine` with no Rust toolchain required.

---

## Phase 8 — Make it cooler (P2, ongoing)

See `09-MAKE-IT-COOLER.md` for the full idea backlog. Highlights: audio-reactive effects (via `pw-cat`/WASAPI), system-metric overlays (CPU/mem as ember intensity), seasonal themes, a `forgum lang` REPL where the cow "speaks" command output, AI-fortune generation via a local LLM, and a `forgum gallery` full-screen interactive browser.

---

## Dependency graph

```
Phase 0 (stop bleeding)
   │
   ├─► Phase 1 (overlay works) ──► Phase 3 (effects polish)
   │        │
   │        └─► Phase 2 (shell hooks) ──► Phase 4 (x-platform)
   │                  │                          │
   │                  └─► Phase 5 (mux) ─────────┴─► Phase 7 (packaging)
   │                           │
   │                           └─► Phase 6 (herder/remote)
   │
   └─► (Phase 8 cooler features can start any time after 1+2)
```

Phases 0 → 1 → 2 are the critical path (~3 weeks). 3/4/5 can overlap. 6/7 follow. 8 is perpetual.

---

## Effort & sequencing summary

| Phase | Duration | Parallelizable? |
|-------|----------|-----------------|
| 0 | 3–4 d | solo |
| 1 | 4–5 d | solo (after 0) |
| 2 | 5–6 d | yes (after 1; can overlap 3) |
| 3 | 4–5 d | yes (after 1) |
| 4 | 3–4 d | yes (after 1+2) |
| 5 | 5–6 d | yes (after 2) |
| 6 | 5–6 d | yes (after 5) |
| 7 | 3–4 d | yes (after 4) |
| **Critical path** | **~3 weeks** | to a working, cross-platform, prompt-safe Forgum |

---

**Next:** `08-TESTING-STRATEGY.md` — the test tiers and CI gates that enforce every invariant above.
