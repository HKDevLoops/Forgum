# Forgum — Planning & Recovery Kit

> A thorough, engineer-to-engineer plan to fix, harden, and **massively coolify** the [Forgum](https://github.com/harish2222/Forgum) project — a cross-platform PowerShell cowsay+fortune+lolcat with a Rust ANSI framebuffer animation engine.

This directory contains the complete plan: every identified bug with a concrete fix, the cross-platform strategy, the design for the flagship "animation above the prompt" engine, the shell-hook redesign, the multiplexer/herder/remote vision, the target architecture, the phased roadmap, the testing gates, and a backlog of cool features.

The cloned source under review lives in `repo/` (Forgum v1.1.2, engine v0.3.0).

---

## What's in here

| # | Document | What it answers |
|---|----------|-----------------|
| **00** | `00-EXECUTIVE-SUMMARY.md` | The 5-minute briefing: what's broken, what to do, in what order. **Read this first.** |
| **01** | `01-BUGS-AND-ISSUES.md` | The full bug catalog — **~70 issues** (the repo's 32-item audit + a fresh 45-finding deep analysis, deduplicated) — each with severity, file:line, root cause, cross-platform impact, and a concrete fix. |
| **02** | `02-CROSS-PLATFORM-STRATEGY.md` | Windows / macOS / Linux / x86_64 / aarch64 / musl; the 5 platform seams; the `forgum-platform` crate; the build & distribution matrix; the CI gate. |
| **03** | `03-ANIMATION-ENGINE-ABOVE-PROMPT.md` | **The flagship.** The exact design for the overlay engine that renders above the prompt: region math, cursor save/restore, no-keystroke-theft, resize handling, signal-safe exit, daemon detach, damage tracking, coalesced rendering. |
| **04** | `04-PROMPT-INTEGRATION.md` | The shell-hook redesign (bash/zsh/fish/pwsh) that renders the *right* cow/eyes/effect from the live config, never breaks the prompt, and sweeps dead daemons. |
| **05** | `05-TMUX-MULTIPLEXER-INTEGRATION.md` | tmux (4 surfaces), zellij, GNU screen, wezterm, **rmux** (remote/follow-me/shared sessions), and **herdr** (the daemon fleet manager). Plus `forgum demo`. |
| **06** | `06-ARCHITECTURE.md` | The target architecture: workspace layout, data flow, the 8 enforced invariants, the platform-crate contract, the shrinking role of PowerShell. |
| **07** | `07-DEVELOPMENT-ROADMAP.md` | 8 phases with tasks, dependencies, and a ~3-week critical path to a working, cross-platform, prompt-safe Forgum. |
| **08** | `08-TESTING-STRATEGY.md` | The 4-tier pyramid (unit → integration → E2E-under-tmux → fuzz), the 20 E2E scenarios, the CI gates (incl. the `cfg`-grep and completion-drift checks). |
| **09** | `09-MAKE-IT-COOLER.md` | The feature backlog: reactive effects (CPU/audio/battery/git), a cow-headed REPL, `forgum battle`, AI fortunes, `forgum showcase`, accessibility. |
| **10** | `10-FINE-TUNED-MASTER-PLAN.md` | **The v2 master plan.** Supersedes `07-…` with 9 test-gated phases, 10 engineering principles, the 3-thread engine, the `Renderer` trait, fixed-timestep scheduler, the "professional program" checklist, and the v2.0 release vision. **Start here for implementation.** |
| **11** | `11-ENGINE-INTERNALS-V2.md` | The deep dive: 3-thread model (sim/render/control), DCL singletons (`OnceLock`/`LazyLock`), `bumpalo` arena (zero-alloc hot loop), `slotmap` particle pool, `rayon` parallel integration, `Renderer` trait with 4 backends (Ansi/SyncAnsi/KittyAnim/WgpuHybrid), fixed-timestep accumulator, signal handling, daemon lifecycle. |
| **12** | `12-PER-ANIMAL-ANIMATION-DESIGN.md` | The 7-axis DNA profile that makes all 109 cows animate uniquely: base/particles/speed/amplitude/palette/easing/phase_seed. Per-base easing defaults, OKLCH color, Verlet tails, golden-ratio desync, the full 109-cow DNA table, the JSON schema. |
| **13** | `13-TEST-COVERAGE-MATRIX.md` | The 6-tier test pyramid (unit → E2E → bench → fuzz → golden-visual → perceptual), 20 E2E scenarios, 15 CI gates, per-phase definition-of-done checklist, coverage targets. |

---

## v2 fine-tuning (the professional upgrade)

Documents `10`–`13` are the **fine-tuned, professional-grade** plan that elevates Forgum from "a cowsay clone with colors" to "a terminal companion that feels alive." They add:

- **Multi-threaded engine** (3 threads: sim/render/control, no shared mutable state, scoped threads).
- **DCL singletons** via `OnceLock`/`LazyLock` — no `static mut`, no `unsafe`, no `lazy_static!`.
- **Zero-allocation hot loop** via `bumpalo` arena + `slotmap` particle pool + `rayon` parallel integration, enforced by a `dhat` CI gate.
- **Hardware + software rendering** via a `Renderer` trait with 4 backends (ANSI / sync-ANSI / kitty-graphics / wgpu-compute), runtime selection, transparent fallback.
- **Fixed-timestep accumulator** scheduler (integer nanoseconds, spiral-of-death clamp, 3-tier adaptive FPS, battery-friendly `park_timeout`).
- **Per-animal uniqueness** via the 7-axis DNA profile (base/particles/speed/amplitude/palette/easing/phase_seed) with the full 109-cow table.
- **OKLCH color** (perceptually uniform, not HSV), Gaussian glow, 256-color dithering fallback.
- **Verlet physics** for tails/capes (unconditionally stable, natural whip/lag/overshoot).
- **Golden-ratio phase randomization** so herds never sync.
- **6-tier test pyramid** with 20 E2E scenarios, 15 CI gates, golden-blake3 visual regression, perceptual-hash weekly check.

---

## The 5-minute briefing

### What Forgum is
A **cross-platform PowerShell module** that fuses cowsay + fortune + lolcat, backed by a **Rust ANSI framebuffer animation engine** (`forgum-engine`) that renders animated ASCII cows. The flagship promise: animations run **above the shell prompt as a non-blocking overlay** while the prompt stays fully interactive. Ships 109 cows, 19 effects, hooks for bash/zsh/fish/tmux.

### What's actually broken (the headlines)
1. **The background overlay engine doesn't really work.** It steals keystrokes from the shell (`event::poll` on `/dev/tty`), runs only ~5 s instead of infinite (`duration=0` bug), doesn't handle terminal resize, and its `Cell::dirty` field breaks damage tracking so the scheduler **never idles — 60 fps forever**, burning CPU/battery even for a static cow. *(BUG-B1/B2/B3/E1)*
2. **The shell hooks (bash/zsh/fish) are non-functional.** They hand-roll JSON without escaping backslashes → invalid JSON for almost every cow; they read the wrong config key (`effect` vs `animation.mode`) → always `random`; and they call the **system `cowsay`** instead of Forgum's 109-cow library. *(BUG-S1/S2/S3/S4)*
3. **Terminal corruption on abnormal exit.** No signal handlers, `Terminal::Drop` is a no-op, and `InvokeEngine.ps1` `Kill()`s the engine → raw mode/alt screen/hidden cursor left on after panic, SIGHUP, SIGTERM, or the 30 s watchdog. Users must `reset` / `stty sane`. *(BUG-T1/T2/T3)*
4. **Cross-platform gaps.** `InvokeEngine.ps1` uses `cmd /c` (Windows-only) for foreground rendering on Linux/macOS; `GetShell` misidentifies WSL/Git Bash as `pwsh`; the Windows `CONOUT$` helper is dead code; no ARM64 prebuilt binaries. *(BUG-C1/S8/B9/C6)*
5. **Daemon lifecycle is broken.** `StartDaemon` launches `--daemon` with no config → exits immediately; the child isn't detached → dies with the shell; `StopDaemon` kills *all* `forgum-engine` processes; temp files leak. *(BUG-D1/B7/D3/D2)*
6. **Cow-specific styling is ignored.** `style_matcher` defines per-cow `speed` and `particles` (dragon fire, dolphin bubbles, nyan stars) but `create_effect` discards them — every cow animates identically. *(BUG-E3)*

The full ~70-issue catalog with fixes is in `01-BUGS-AND-ISSUES.md`.

### The plan, in one breath
Extract a `forgum-platform` crate so all `#[cfg]` lives in one place; add RAII guards + signal handlers so the terminal is **always** restored; delete keystroke-reading from the background loop and exit via duration/signals/control-socket; fix `duration=0` and the `Cell::dirty` PartialEq; give the engine a real CLI (clap) that renders cows natively, reads the config itself, and generates correct shell hooks + completions; make the daemon truly detached with a PID file; add a per-session control socket; then layer tmux/zellij/wezterm integration, a `herd` fleet manager, remote/follow-me sync, reactive effects, and a `forgum showcase` reel.

### Critical path (~3 weeks)
**Phase 0** stop-the-bleeding (RAII, signals, no input reads, `duration=0`, `Cell::dirty`) → **Phase 1** make the overlay work (`open_render_output`, resize, detach, control socket) → **Phase 2** shell hooks that work (engine CLI, native cow renderer, `--config`, rewritten `init`, `precmd` sweep). Phases 3 (effects), 4 (x-platform), 5 (mux) parallelize after that. See `07-DEVELOPMENT-ROADMAP.md`.

### What "cross-platform" means here (concretely)
Tier-1: **Windows 11, macOS 14+ (Intel + Apple Silicon), Ubuntu/Debian/Fedora/Arch (x86_64 + aarch64)**, on **pwsh 7, bash 5, zsh 5.8, fish 3**, under **Windows Terminal, iTerm2, Alacritty, kitty, gnome-terminal, tmux 3.3+**. CI builds 7 prebuilt binaries (incl. universal macOS and static musl), runs `cargo test` + Pester + an `xvfb-run tmux` E2E harness that asserts the **prompt row is never touched** and **keystrokes aren't stolen**. The `forgum-platform` crate is the only place `#[cfg(windows)]`/`#[cfg(unix)]` appears — enforced by a CI grep. See `02-CROSS-PLATFORM-STRATEGY.md`.

### The "cool" part (tmux / rmux / herdr)
- **tmux** gets four integration surfaces: pane overlays, a living status line, a `Ctrl-f` popup cow (`display-popup`), and focus-aware auto start/stop via `set-hook`.
- **zellij / wezterm / screen** each get a tailored `install` + status surface.
- **rmux** (remote): a Forgum daemon that **follows you across SSH** via `RemoteForward` of the control socket, and **syncs** to all peers on a shared tmux session (deterministic effects → ~10 B/s sync traffic).
- **herdr** (`forgum herd`): a fleet manager — `list/stop/effect/speed/pause/resume` across every daemon in every pane/session/host, `follow` (only the focused pane animates), `theme rotate` (mood cycling), `census` (watchdog + auto-restart), and a `--watch` live dashboard in a tmux popup.
- Plus `forgum demo`, `forgum battle`, `forgum pet`, audio-reactive aurora, CPU-load ember, AI fortunes, and a `forgum showcase` reel. See `05-…` and `09-…`.

---

## How to use this kit

1. **Skim `00-EXECUTIVE-SUMMARY.md`** (or this README) for the big picture.
2. **Triage with `01-BUGS-AND-ISSUES.md`** — start with the P0 ship-blockers (BUG-T1/T2, B1/B2, S1–S4, D1, E1).
3. **Implement Phase 0 → 1 → 2** of `07-DEVELOPMENT-ROADMAP.md`, using `03-…` and `04-…` as the detailed designs.
4. **Gate every PR** with the CI checks in `08-TESTING-STRATEGY.md` §6 — especially the prompt-row-untouched assertion and the `cfg`-grep.
5. **Layer cool stuff** from `05-…` and `09-…` once the core is solid.

---

## Provenance

- **Source reviewed:** `repo/` = `git clone https://github.com/harish2222/Forgum.git` (commit at clone time, ~v1.1.2 / engine v0.3.0).
- **Findings sources:** the repo's own `docs/AUDIT-2026-06-20.md` (32 items, partially fixed) + a fresh 45-finding deep audit of the Rust engine and PowerShell shell-integration layer (full text in `/home/z/my-project/worklog.md` under Task ID `ANALYSIS-1`).
- **Note on `plan1.md` / `plan2.md`:** these files (described as exported agent sessions containing the project's main idea) are **not present** in the repository. The "main idea" was reconstructed from the repo's own planning docs: `docs/PLAN-2026-06-20-BACKGROUND-ENGINE.md`, `docs/TUI_Animation_Plan.md`, `docs/Cow_Animation_Manifesto.md`, `docs/PLAN-*.md`, `.agents/orchestrator/plan.md`, and the wiki. If the user supplies `plan1.md`/`plan2.md`, this plan should be cross-checked against them for any additional intent.
