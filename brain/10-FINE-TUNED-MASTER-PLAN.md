# Forgum — Fine-Tuned Master Plan (v2)

> **The professional-grade, fool-proof blueprint.** This document supersedes `07-DEVELOPMENT-ROADMAP.md` as the single source of truth for *how* Forgum is engineered, phased, tested, and shipped. It is written for a senior Rust engineer and assumes the reader has absorbed `00-EXECUTIVE-SUMMARY.md`, `03-ANIMATION-ENGINE-ABOVE-PROMPT.md`, and `06-ARCHITECTURE.md`.
>
> **What changed vs. v1.** v1 was a competent triage plan ("fix the bleeding, then build"). v2 adds the engineering rigor a 20-year veteran expects: a **3-thread engine** (sim / render / control), **DCL singletons** via `OnceLock`/`LazyLock`, **arena allocation** for zero-alloc hot loops, a **`Renderer` trait** with hardware (kitty-graphics / wgpu-compute) and software (ANSI / sync-ANSI) backends selected at runtime, a **fixed-timestep accumulator** scheduler with spiral-of-death protection, a **7-axis per-animal DNA profile** that makes all 109 cows animate uniquely, and a **6-tier test pyramid** with explicit per-phase gates. Nothing in v1 is contradicted; v2 makes it production-grade.

---

## 0. The engineering principles (non-negotiable)

These ten principles are the constitution. Every PR is reviewed against them; every phase's definition-of-done asserts them.

1. **Single source of truth for platform code.** All `#[cfg]` lives in `crates/platform/`. `engine/src/` has zero `#[cfg]` — enforced by a CI grep (`rg '#\[cfg' crates/engine/src/` must return 0 hits). *Rationale: portable logic, single audit surface.*
2. **RAII or it didn't happen.** Every terminal-state mutation (`enable_raw_mode`, `EnterAlternateScreen`, `cursor::Hide`, overlay-paint) is wrapped in a guard whose `Drop` restores the prior state. `catch_unwind(AssertUnwindSafe(..))` wraps the render loop so panic → Drop → clean terminal. *Rationale: BUG-T1/T2/T3 never recur.*
3. **No input reads in background mode.** `event::poll`/`event::read` are absent from `render_loop_background` — enforced by CI grep. Exit is via duration, signal, or control socket. *Rationale: BUG-B1 never recurs.*
4. **Zero allocation in the hot loop.** After warmup, the per-frame path allocates **0 bytes**. Per-frame scratch lives in a `bumpalo::Bump` that is `reset()` at frame top. A `dhat`-based integration test asserts this in CI. *Rationale: no GC-style jitter, no leak growth over weeks of uptime.*
5. **DCL singletons, never `static mut`.** Process-wide singletons use `std::sync::OnceLock<T>` (resettable via `Mutex<Option<T>>`) or `std::sync::LazyLock<T>` for statics. Edition 2024 forbids `static mut`; we adopt the rule now. *Rationale: correct double-checked locking, no UB, no `unsafe`.*
6. **Software path is the contract; hardware is the accelerator.** The CPU ANSI renderer is always correct and always available. The kitty-graphics and wgpu-compute backends are pure optimizations, selected at runtime by a capability probe, with transparent fallback. `forgum --gpu` failing silently becomes `forgum` (CPU). *Rationale: headless servers, SSH, CI, RDP-without-GPU all work.*
7. **Fixed-timestep simulation, variable rendering.** The sim thread runs at a strict 60 Hz (integer-nanosecond accumulator, spiral-of-death clamp at 250 ms). The render thread interpolates between previous and current physics state with `alpha = accumulator / dt`. *Rationale: deterministic, framerate-independent, bit-identical across machines (enables deterministic peer sync).*
8. **Every cow is unique.** All 109 cows are driven by a 7-axis DNA profile (`base`, `particles`, `speed`, `amplitude`, `palette`, `easing`, `phase_seed`) stored in `animations.json`. Even cows sharing a base get a distinct `phase_seed` so a herd never syncs. *Rationale: BUG-E3 fixed structurally, not patch-wise.*
9. **Tests are the spec.** Every invariant in this document has a named test in `13-TEST-COVERAGE-MATRIX.md`. A phase is "done" when its test gate is green on all Tier-1 platforms — not when the code is written. *Rationale: "it compiles" is never done.*
10. **One version, everywhere.** `Cargo.toml` is the single version source. CI generates `Forgum.psd1`, Homebrew formula, scoop/winget manifests, and AUR PKGBUILD from it and asserts byte-equality. *Rationale: no version drift bugs.*

---

## 1. The target system, in one diagram

```
                         ┌─────────────────────────────────────────┐
                         │            USER SHELL                    │
                         │  (bash / zsh / fish / pwsh)              │
                         │  forgum fn · precmd sweep · completions  │
                         └──────────────┬──────────────────────────┘
                                        │ argv / control socket / signal
                                        ▼
┌───────────────────────────────────────────────────────────────────────────┐
│                      forgum-engine  (Rust binary, multi-threaded)          │
│                                                                            │
│  ┌──────────┐   ┌──────────┐   ┌───────────┐   ┌──────────────────────┐  │
│  │ CLI(clap)│──►│ Config   │──►│ Cow       │──►│ Effect registry +    │  │
│  │ render/  │   │ merge:   │   │ renderer  │   │ 7-axis DNA profiles  │  │
│  │ fortune/ │   │ argv>    │   │ (.cow +   │   │ (animations.json)    │  │
│  │ daemon/  │   │ file>    │   │ $eyes +   │   │                      │  │
│  │ herd/    │   │ defaults │   │ bubble)   │   │ 10 base anims +      │  │
│  │ theme/   │   └──────────┘   └───────────┘   │ 6 particle types +   │  │
│  │ tmux/    │                                  │ Verlet chains        │  │
│  │ init/    │                                  └──────────┬───────────┘  │
│  │ complet. │                                             │              │
│  └────┬─────┘                                             │              │
│       │           ┌───────────────────────────────────────▼──────────┐   │
│       │           │          ENGINE CORE (3 threads)                 │   │
│       │           │                                                    │   │
│       │           │  ┌─────────────┐  Arc<Frame>  ┌────────────────┐  │   │
│       │           │  │  SIM THREAD │ │(bounded 2)│ │ RENDER THREAD  │  │   │
│       │           │  │ (owns back  │ ├──────────►│ (owns front +   │  │   │
│       │           │  │  + particles│ │ crossbeam │  Renderer trait)│  │   │
│       │           │  │  + DNA)     │ │           │  ├ AnsiRenderer │  │   │
│       │           │  │ rayon::par_ │ │           │  ├ SyncAnsiRen  │  │   │
│       │           │  │  chunks_mut │ │           │  ├ KittyAnimRen │  │   │
│       │           │  │ 60Hz fixed  │ │           │  └ WgpuHybrid   │  │   │
│       │           │  │  timestep   │ │           │    (opt-in)     │  │   │
│       │           │  └──────┬──────┘ │           └────────┬───────┘   │   │
│       │           │         │        │                    │           │   │
│       │           │  ┌──────▼──────┐  │           ┌────────▼───────┐  │   │
│       │           │  │CONTROL THRD │──┘           │ Renderer picks │  │   │
│       │           │  │ SIGWINCH +  │  ControlMsg  │ backend via    │  │   │
│       │           │  │ control sock│ ├───────────►│ OnceLock<Caps> │  │   │
│       │           │  │ NO tty reads│              │ probe atstartup│  │   │
│       │           │  └─────────────┘              └────────────────┘  │   │
│       │           └────────────────────────────────────────────────────   │
│       │                                                                   │
│       │     ┌──────────────────────────────────────────────────────────┐  │
│       │     │  Scheduler: fixed-timestep accumulator (i64 ns)          │  │
│       │     │  tiers: heartbeat=1fps / idle=5fps / active=60fps        │  │
│       │     │  spiral clamp: 250ms · park_timeout on idle (battery)    │  │
│       │     └──────────────────────────────────────────────────────────┘  │
│       │                                                                   │
│       │     ┌──────────────────────────────────────────────────────────┐  │
│       │     │  Memory: bumpalo::Bump per-frame arena (reset, no Drop)  │  │
│       │     │  Particles: slotmap::SlotMap (O(1) spawn/kill, ABA-safe) │  │
│       │     │  Singletons: OnceLock/LazyLock, never static mut         │  │
│       │     └──────────────────────────────────────────────────────────┘  │
│       │                                                                   │
│       │     ┌──────────────────────────────────────────────────────────┐  │
│       │     │  Daemon: setsid/DETACHED + daemon.json + per-session     │  │
│       │     │  control socket (STOP/PAUSE/RESUME/EFFECT/SPEED/COW/...) │  │
│       │     │  Herder: scans daemon.json, parallel fan-out             │  │
│       │     │  Remote: SSH RemoteForward + deterministic-effect sync   │  │
│       │     └──────────────────────────────────────────────────────────┘  │
└───────┼───────────────────────────────────────────────────────────────────┘
        │ bundled via include_dir!
        ▼
   Data/Cows/*.cow · Data/Fortunes/*.txt · Data/Templates/ · animations.json
```

---

## 2. The 9 phases (fine-tuned, test-gated, dependency-ordered)

Each phase has: **Goal** · **Invariants hardened** · **Tasks** · **Test gate (definition of done)** · **Duration**.

Convention: `[P0]` ship-blocker · `[P1]` flagship quality · `[P2]` polish · `[dep: X]` depends on phase X · `[par: Y]` parallelizable with Y.

---

### Phase 0 — Stop the bleeding (P0, 3–4 days)

**Goal:** no terminal corruption, no CPU burn, no keystroke theft, no memory growth.

**Invariants hardened:** #2 (RAII), #3 (no input reads), #4 (zero-alloc — partial), #5 (no `static mut`).

| Task | Bug | Deliverable |
|------|-----|-------------|
| 0.1 RAII guards in `forgum-platform` | T2 | `RawModeGuard`/`AltScreenGuard`/`CursorShowGuard`/`OverlayGuard`; `Drop` restores |
| 0.2 Signal handlers | T1 | `signal-hook` (unix: SIGTERM/INT/HUP/WINCH/TSTP) + `SetConsoleCtrlHandler` (win); `ShutdownFlag(Arc<AtomicBool>)`; SIGTSTP restore-suspend-reacquire dance |
| 0.3 `catch_unwind` around render loop | T2 | `AssertUnwindSafe` wrapper; panic → Drop → clean terminal |
| 0.4 Delete input reads from background loop | B1 | remove `event::poll`/`event::read` in `render_loop_background`; CI grep gate |
| 0.5 Fix `duration=0` → infinite | B2, D7 | `max_frames: u64`; `0` = infinite; saturating math |
| 0.6 Fix `Cell::dirty` PartialEq | E1 | manual `PartialEq` on `ch/fg/bg/alpha`; `dirty` excluded |
| 0.7 Real `dt` | B8 | `Instant`-based, clamped to 0.1s; `i64` nanoseconds (no f64 drift) |
| 0.8 Bounded stdin (4 MB) + exit codes | D4, D5 | cap stdin; `exit(1)` on parse error |
| 0.9 Graceful kill in `InvokeEngine.ps1` | T3 | SIGTERM-first, raise timeout, force-restore escape on kill |

**Test gate (Phase 0 DoD):**
- ✅ The 8 verification tests in `03-…` §13 pass under `xvfb-run tmux`.
- ✅ Static cow over 60 s: avg CPU < 1% (the `Cell::dirty` fix lets the scheduler idle).
- ✅ `kill -TERM` leaves the terminal pristine (capture stream tail, assert ends with overlay-clear + `ESC[0m`).
- ✅ `dhat` test: per-frame allocation == 0 after warmup (Phase 0 partial — full zero-alloc is Phase 1.4).
- ✅ `rg 'event::(poll|read)' crates/engine/src/render_background.rs` returns 0 hits.

---

### Phase 1 — The 3-thread engine + overlay correctness (P0, 6–7 days)

**Goal:** the engine runs as 3 threads (sim/render/control), the overlay renders above the prompt, survives resize, exits cleanly, doesn't die with the shell, and the hot loop allocates zero bytes. `[dep: 0]`

**Invariants hardened:** #1 (platform crate), #4 (zero-alloc — full), #6 (software contract), #7 (fixed-timestep), #10 (version parity).

| Task | Bug | Deliverable |
|------|-----|-------------|
| 1.1 Extract `crates/platform/` | arch | all `#[cfg]` contained; `engine/src/` cfg-grep gate |
| 1.2 `open_render_output()` | B9, C1 | unifies stdout / `/dev/tty` / `CONOUT$`; removes `cmd /c` |
| 1.3 SIM thread (owns `back` + `particles` + DNA) | (new) | fixed-timestep accumulator; `rayon::par_chunks_mut` for particle integration; ships `Arc<Frame>` to render thread |
| 1.4 RENDER thread (owns `front` + `Renderer`) | (new) | `crossbeam::channel::bounded(2)`; `BeginSynchronizedUpdate`/`EndSynchronizedUpdate` wrapping; `BufWriter<Mutex<Stdout>>` |
| 1.5 CONTROL thread (SIGWINCH + socket) | (new) | NO tty reads; `crossbeam::channel::unbounded<ControlMsg>` to sim thread; installs `SIGTSTP`/`SIGCONT` dance |
| 1.6 `bumpalo::Bump` per-frame arena | (new) | all per-frame scratch (ANSI buf, damage list, run-coalesce strings) allocated in arena; `bump.reset()` at frame top; `dhat` CI gate asserts 0 bytes |
| 1.7 `slotmap::SlotMap` for particles | E8 | O(1) spawn/kill; ABA-safe generational keys; replaces `Vec<bool>` + O(n) scan |
| 1.8 DCL singletons (`OnceLock`/`LazyLock`) | (new) | `CONFIG: LazyLock<Config>`, `TERM_CAPS: OnceLock<TermCaps>`, `POOL: LazyLock<Mutex<Option<ParticlePool>>>`; remove all `static mut`; `lazy_static!`/`once_cell` migrated |
| 1.9 Background `Resize` handler | B3, F2 | SIGWINCH → clear-old → resize fb/region → recenter; stale-safe `ob_y1` via `Cell` |
| 1.10 Stale-safe cleanup | B4 | `OverlayGuard` reads live `ob_y1`/`cols` from `Cell` |
| 1.11 Foreground full canvas | B5 | `Rect::new(0,0,cols,rows)` in foreground; raw mode + alt screen + cursor hide guarded |
| 1.12 Daemon detach + PID file | B7, D3 | `setsid`/`DETACHED_PROCESS`; `daemon.json` = `{pid, ob_y1, cols, pane, session, socket_path}` |
| 1.13 Control socket (per-session) | (new) | `STOP/PAUSE/RESUME/EFFECT/SPEED/COW/STATUS/PING`; mode 0600; keyed on tmux pane / shell PID |
| 1.14 `StopDaemon` per-session | D3 | reads `daemon.json`; SIGTERM-first; sweep on `precmd` |
| 1.15 Temp file cleanup | D2 | engine `remove_file` after `--file` read; startup sweep |
| 1.16 Tiny-terminal guard | E7 | `cols < 20 || rows < 5` → static print fallback |

**Test gate (Phase 1 DoD):**
- ✅ A background animation survives a pane resize, a `kill -TERM`, and the shell exiting (daemon stays).
- ✅ Two panes animate independently; `forgum daemon stop` kills only the current pane's.
- ✅ **Keystroke pass-through**: under a pty, type `echo hi<Enter>` into the shell while a background animation runs; assert the shell receives and executes `echo hi`.
- ✅ **Prompt-row assertion**: capture 5 s of ANSI; assert no `MoveTo` with `y >= ob_y1` ever appears.
- ✅ **Cursor-save/restore balance**: every `ESC 7` has a matching `ESC 8` before the next frame.
- ✅ **Static-cow idle**: 60 s, avg CPU < 1%.
- ✅ **Zero-alloc hot loop**: `dhat` integration test asserts 0 bytes allocated after frame 10.
- ✅ **Duration=0 infinite**: still alive after 10 s.
- ✅ **`kill -9` sweep**: `kill -9` the daemon, trigger `precmd`; assert overlay cleared.
- ✅ `rg '#\[cfg' crates/engine/src/` returns 0 hits.
- ✅ `cargo +nightly miri test` clean on `protocol.rs` + `terminal.rs` unsafe blocks.
- ✅ `cargo bench --bench engine` shows no regression vs. Phase 0 baseline.

---

### Phase 2 — Shell hooks that work + native cow renderer (P0, 5–6 days)

**Goal:** `forgum init bash/zsh/fish/pwsh` produces a hook that renders the right cow, eyes, effect, lolcat — read live from config — without breaking the prompt. `[dep: 1]` `[par: 3]`

**Invariants hardened:** #2 (RAII on shell side), #9 (tests are spec).

| Task | Bug | Deliverable |
|------|-----|-------------|
| 2.1 Engine CLI (clap) | (new) | `render/fortune/daemon/herd/theme/tmux/init/completions/status-line` subcommands |
| 2.2 Native cow renderer | S4 | load `.cow`, expand `$eyes/$tongue/$thoughts`, speech bubble; bundled via `include_dir!` |
| 2.3 `--config` reading + merge | S3, S7 | engine is single config reader; `argv > file > defaults` |
| 2.4 Rewrite `handle_init` | S1, S2, S6 | bash/zsh/fish/pwsh hooks using engine-generated JSON (no hand-rolled JSON); absolute engine path |
| 2.5 `parent_comm()` shell detect | S8 | WSL/Git Bash/Cygwin correct |
| 2.6 `precmd` sweep in all hooks | (new) | `kill -9` safety net; reads `daemon.json` |
| 2.7 Delete `GetForgumShellHook.ps1` | S6 | route `forgum init` through engine |
| 2.8 Generated completions | I3 | `forgum-engine completions <shell>`; CI drift check |
| 2.9 Installer PATH/symlink | S5, I1 | Unix: `/usr/local/bin`; Windows: PATH; prebuilt binary download |
| 2.10 Version-parity CI | (new) | single source in `Cargo.toml`; asserts parity across `Forgum.psd1` + manifests |

**Test gate (Phase 2 DoD):**
- ✅ On a fresh bash/zsh/fish/pwsh, `forgum "hello"` renders the configured cow+effect.
- ✅ Changing `animation.mode` in config takes effect on the next call (no re-init).
- ✅ `shellcheck` / `fish --no-execute` clean.
- ✅ The pty keystroke test passes (typing `echo hi<Enter>` runs while a background animation plays).
- ✅ Completion drift check: `diff <(forgum-engine completions bash) scripts/completions/forgum.bash` empty.
- ✅ Version parity: `cargo run -- gen-version` output matches all manifests byte-for-byte.

---

### Phase 3 — Per-animal animation DNA + effects polish (P1, 6–7 days)

**Goal:** effects look right, perform well, and use per-cow styling. Every cow animates uniquely. `[dep: 1]` `[par: 2]`

**Invariants hardened:** #8 (every cow unique), #4 (zero-alloc in particle path).

| Task | Bug | Deliverable |
|------|-----|-------------|
| 3.1 7-axis DNA profile schema | E3 | `animations.json` extended: `{base, particles{type,rate,life,speed}, speed, amplitude{breath,sway,float}, palette[], easing{base,particle_alpha}, phase_seed, glow{color,radius,falloff}}` |
| 3.2 `create_effect` accepts full DNA | E3, E16/17 | `create_effect(base, cow_text, speed, particles, amplitude, palette, easing, phase_seed)`; themed particle overlays (fire/bubbles/stars/zzz/pulse/glitch) |
| 3.3 `ezing` easing crate + per-base defaults | (new) | `sine_inout` (breathe/float/sway), `cubic_inout` (walk), `back_out` (talk), `expo_out` (pulse), `linear` (glitch), `cubic_in` (dissolve); overridable per cow |
| 3.4 OKLCH color via `palette` crate | (new) | gradient interpolation in `Lch` space (not HSV); per-cow `palette` field; lolcat ported to engine |
| 3.5 Verlet chains for tails/capes | (new) | `VerletChain<N>` generic; 4-link tail for cats, 6-link for dragons; distance-constraint relaxation (4 iterations) |
| 3.6 Per-instance phase randomization | (new) | `phase = (phase_seed ^ instance_id) as f32 * 0.618...` (golden ratio); herd never syncs |
| 3.7 Effect constructors take real size | E2 | pass `cols/rows` to `new()`; Shatter/Dissolve spawn after first `on_resize` |
| 3.8 `Effect::is_done()` | E4 | one-shot effects break the loop |
| 3.9 Bounce physics fix | E5 | correct kinematics + shape squash (Disney squash/stretch) |
| 3.10 Portal hue modulo | E6 | `((x % 360)+360) % 360` |
| 3.11 Particle bounds check | E7 | skip out-of-bounds before `as usize` |
| 3.12 Coalesced run rendering | F1 | row-run coalescing + cross-cell color caching; bucket particles by color |
| 3.13 Gaussian radial glow | (new) | replace linear falloff with `exp(-d²/2σ²)`, `σ = radius/2.5`; inverse-square for Pulse cores |
| 3.14 256-color dithering fallback | (new) | detect `COLORTERM != truecolor`; quantize to xterm-256 cube; 4×4 Bayer dither on alpha |

**Test gate (Phase 3 DoD):**
- ✅ Dragon breathes fire, dolphin blows bubbles, nyan trails stars — visually verified.
- ✅ Shatter exits when done (no blank overlay).
- ✅ Full repaint < 2 ms at 80×40 (`criterion` bench).
- ✅ No particle smear at col 0.
- ✅ **Golden framebuffer hash**: for each of 109 cows, render 60 frames, `blake3::hash` the framebuffer; store in `Tests/golden/<cow>.blake3`; CI fails on hash change. Re-golden via `--features regolden`.
- ✅ **Perceptual hash (weekly)**: rasterize framebuffer to PNG, pHash 8×8 DCT, Hamming distance ≤ 5 to reference.
- ✅ 5 `default.cow` instances in a tmux herd visibly desync (different breath phases).
- ✅ OKLCH gradient: `dragon.cow` palette transitions orange→red→black with no muddy midpoint (visual + pHash).
- ✅ `cargo bench --bench anim`: `damage_diff` < 50µs, `particle_update` (10k) < 500µs, `render_region` < 2ms.

---

### Phase 4 — Hardware rendering backends (P1, 4–5 days)

**Goal:** the `Renderer` trait has 3 backends; kitty-graphics and wgpu-compute are opt-in accelerators with transparent fallback. `[dep: 1]` `[par: 3]`

**Invariants hardened:** #6 (software contract; hardware accelerator).

| Task | Deliverable |
|------|-------------|
| 4.1 `Renderer` trait | `fn render_frame(&mut self, frame: &Frame, damage: &[(usize,usize)]) -> io::Result<()>` |
| 4.2 `AnsiRenderer` (default) | existing per-cell diff; coalesced runs |
| 4.3 `SyncAnsiRenderer` | wraps each frame in `BeginSynchronizedUpdate`/`EndSynchronizedUpdate` (DEC mode 2026) |
| 4.4 `KittyAnimRenderer` | kitty graphics protocol; `a=f`/`a=a,c=N` animation frames; terminal-driven loop (daemon sleeps between frames); tmux passthrough wrapping |
| 4.5 `WgpuHybridRenderer` (opt-in `--gpu`) | compute-shader particle sim → 80×24 RGB grid → `map_async` readback → existing `render_region`; `PowerPreference::LowPower`; transparent CPU fallback on any error |
| 4.6 Capability probe (`OnceLock<TermCaps>`) | 3-stage: env scan (`COLORTERM`/`TERM_PROGRAM`/`TMUX`) → DA1 probe (sixel bit 4) → kitty graphics probe (`DCS Gi=31,a=q` + DA1 race, 50ms timeout) |
| 4.7 Backend selection rule | per frame: `if kitty_graphics && damage_coverage > 60% → KittyAnimRenderer; else if sync_supported → SyncAnsiRenderer; else AnsiRenderer`. `--gpu` adds wgpu to the chain. |
| 4.8 tmux passthrough | `wrap_for_tmux(seq)`: `<ESC>Ptmux;<ESC><seq><ESC>\` when `$TMUX` set + `allow-passthrough on` |

**Test gate (Phase 4 DoD):**
- ✅ `forgum --gpu` on a headless server: silently falls back to CPU, renders correctly, logs one `WARN`.
- ✅ `forgum` in kitty terminal: `KittyAnimRenderer` engages when damage > 60%; daemon CPU drops (terminal drives animation).
- ✅ `forgum` inside tmux without `allow-passthrough`: degrades to `SyncAnsiRenderer`, no corruption.
- ✅ `forgum` in Windows Terminal (no kitty, no sixel): `SyncAnsiRenderer` or `AnsiRenderer`, correct output.
- ✅ Capability probe: 3-stage, cached in `OnceLock`, < 60ms total.
- ✅ `cargo test --features gpu`: wgpu backend unit tests (mock adapter) pass.
- ✅ Visual regression: golden-hash identical across all 4 backends for the same cow (the contract holds).

---

### Phase 5 — Cross-platform hardening (P1, 3–4 days)

**Goal:** Tier-1 matrix green in CI. `[dep: 1, 2]` `[par: 3, 4]`

| Task | Bug | Deliverable |
|------|-----|-------------|
| 5.1 `forgum-platform` fully extracted | arch | cfg-grep gate green; `unix.rs`/`windows.rs`/`macos.rs` |
| 5.2 Paths via `directories` crate | C3 | `$FORGUM_CONFIG`/`$FORGUM_DATA` honored; XDG / AppData / Library |
| 5.3 `GetShell` WSL fix | S8 | parent-comm detection |
| 5.4 TUI cross-platform | C2, P29 | `Invoke-Item`; sync mode list with engine |
| 5.5 `GetEngineBinary` no auto-rebuild | P30 | build at install; sentinel cache |
| 5.6 `winapi` → `windows-sys` | C5 | migrate `open_console_out` |
| 5.7 Legacy PS animation line-ending audit | C4 | confirm `` `r?`n `` |
| 5.8 Remaining repo-audit items | P15–P28 | see `01-…` §8 |
| 5.9 ARM64 prebuilt binaries | C6 | `aarch64-unknown-linux-musl` via `cargo-zigbuild`; Raspberry Pi / Graviton / Apple Silicon Linux VMs |
| 5.10 Windows pre-1709 guard | (new) | refuse background daemon; print plain cow + warning |

**Test gate (Phase 5 DoD):**
- ✅ `cargo test` green on all 7 targets: `x86_64-{linux-gnu,linux-musl,apple-darwin,windows-msvc}`, `aarch64-{linux-musl,apple-darwin}`, universal2 macOS.
- ✅ Pester green on Windows + Linux pwsh.
- ✅ `xvfb-run tmux` smoke green on Linux x86_64 + aarch64.
- ✅ `clippy -D warnings` clean.
- ✅ `#[cfg]`-grep finds zero hits in `engine/src/`.
- ✅ Miri clean on unsafe blocks.
- ✅ SBOM generated (`cargo cyclonedx`) per release artifact.

---

### Phase 6 — Multiplexer integration (P1, 5–6 days)

**Goal:** tmux/zellij/screen/wezterm first-class; herbstluftwm herder. `[dep: 2]` `[par: 4]`

| Task | Deliverable |
|------|-------------|
| 6.1 `detect_mux()` + per-pane daemon state | `daemon.json` keyed on `$TMUX_PANE`/session |
| 6.2 `forgum tmux install` | writes status-line + hooks + popup keybind (idempotent) |
| 6.3 `forgum-engine status-line` | < 5ms one-shot; `--live --fps 1` ticker variant |
| 6.4 `--popup` render mode + `forgum tmux popup` | tmux `display-popup` one-shot |
| 6.5 tmux `set-hook` focus-aware | pane-focus-in/out start/stop; `daemon resize` on layout change |
| 6.6 tmux truecolor + passthrough recipe | `terminal-overrides Tc` / `terminal-features RGB`; `allow-passthrough` detection |
| 6.7 zellij/wezterm/screen installers | ported config templates |
| 6.8 `forgum herd list/stop/effect/speed/pause/resume` | scans `daemon.json`; parallel control-socket fan-out |
| 6.9 `forgum herd follow` | only focused pane animates; others idle |
| 6.10 `--watch` popup dashboard | live herd table in tmux popup; `e`/`q` keys |
| 6.11 `forgum theme apply/list/rotate` | theme bundles (effect+cow+palette+lolcat); `--rotate N` |
| 6.12 `forgum herd census` | watchdog: sweep dead daemons, optional auto-restart |
| 6.13 `forgum herd` herbstluftwm mode | `herbstclient --idle` parser; `FrameMap`; `forgum init herbstluftwm` writes `autostart.d/forgum.sh`; Wayland detection + graceful degrade |

**Test gate (Phase 6 DoD):**
- ✅ `forgum tmux install` in a fresh tmux gives a living status line, focus-aware pane overlays, and `Ctrl-f` popup cow.
- ✅ Resizing a pane reflows the animation.
- ✅ 6 panes animate; `forgum herd effect aurora --all` changes all six in < 100ms.
- ✅ `forgum herd list --watch` shows a live dashboard.
- ✅ herbstluftwm: `forgum herd` launches from `autostart`; focused frame animates at 60fps, others idle at 5fps.
- ✅ Wayland: `forgum herd` detects `$WAYLAND_DISPLAY`, logs warning, falls back to per-window daemons.

---

### Phase 7 — Remote + packaging (P1/P2, 5–6 days)

**Goal:** follow-me-across-SSH; one-command install everywhere. `[dep: 5, 6]`

| Task | Deliverable |
|------|-------------|
| 7.1 `forgum remote attach/sync/who` | SSH reverse-forward of control socket; deterministic-effect peer sync (`StdRng::seed_from_u64`, `fixed::I32F32` for cross-peer determinism) |
| 7.2 Release CI matrix | 7 prebuilt binaries + universal macOS; sha256; SBOM per artifact |
| 7.3 Homebrew formula | universal binary; `brew install forgum` |
| 7.4 Scoop + winget | per-arch manifests; refresh on every release (closes BUG-B5) |
| 7.5 `.deb` + `.rpm` + AUR | `cargo-deb`/`cargo-generate-rpm`/`cargo-aur` |
| 7.6 PowerShell Gallery | `Publish-Module`; engine binary bundled in `bin/` |
| 7.7 `cargo install forgum-engine` | engine-only path for Rust users |

**Test gate (Phase 7 DoD):**
- ✅ SSH to a server with `RemoteForward` and the local cow follows.
- ✅ A user on any Tier-1 OS installs with one command and gets a working `forgum` + `forgum-engine` with no Rust toolchain.
- ✅ Deterministic sync: two peers seeing the same cow produce bit-identical frames (golden-hash equality) given the same seed + inputs.
- ✅ SBOM present in every release artifact.

---

### Phase 8 — Make it cooler (P2, ongoing)

**Goal:** reactive effects, AI fortunes, showcase reel, community cow packs. `[dep: 3]`

| Task | Deliverable |
|------|-------------|
| 8.1 Keep-alive idle micro-animations | breath + blink + tail-flick on every cow, even "static" |
| 8.2 CPU-load-reactive ember | `sysinfo`; dragon fire rate scales with `cpu_usage` |
| 8.3 git-status-reactive | skeleton glows green on clean, red on dirty |
| 8.4 time-of-day reactive | ghost spawns at night; cow wears santa hat in December |
| 8.5 battery-reactive | throttle to 5fps when `< 20%` |
| 8.6 audio-reactive (opt-in `--audio`) | `cpal` + `rustfft`; Pulse cows throb to bass; Fly wing-rate to mids |
| 8.7 `forgum demo` / `forgum showcase` | scripted reel of all effects + cows |
| 8.8 `forgum battle` | two cows fight (Verlet physics + particle collisions) |
| 8.9 `forgum pet` | interactive: click to pet, cow reacts |
| 8.10 `forgum lang` REPL | cow "speaks" command output |
| 8.11 AI fortunes (opt-in, local LLM) | `llm` skill; offline via `mistral.rs` |
| 8.12 Community cow packs | `~/.config/forgum/cows/` + `custom.anim.json`; hot-reload |
| 8.13 nms-style dissolve/reveal transitions | for gallery cow-switching |

**Test gate (Phase 8 DoD):** per-feature; each ships behind a feature flag, golden-hash tested, opt-in.

---

## 3. Dependency graph & critical path

```
Phase 0 (stop bleeding) ── 3-4d
   │
   ├─► Phase 1 (3-thread engine + overlay) ── 6-7d
   │        │
   │        ├─► Phase 3 (per-animal DNA + effects) ── 6-7d ─┐
   │        │                                                │
   │        └─► Phase 4 (hardware renderers) ── 4-5d ────────┤
   │                                                          │
   └─► Phase 2 (shell hooks + native cow) ── 5-6d             │
            │                                                  │
            ├─► Phase 5 (cross-platform) ── 3-4d ─────────────┤
            │                                                  │
            └─► Phase 6 (mux + herder) ── 5-6d ───────────────┤
                        │                                      │
                        └─► Phase 7 (remote + packaging) ── 5-6d
                                                                  │
                                                                  └─► Phase 8 (cooler, ongoing)
```

**Critical path:** 0 → 1 → 2 → 6 → 7 = **~5 weeks** to a fully featured, cross-platform, prompt-safe, multi-threaded, hardware-accelerated Forgum with herder + remote + packaging.

**Parallelizable:** 3 || 4 || 5 can run concurrently after Phase 1+2 land. A 2-engineer team can compress the critical path to ~3.5 weeks.

---

## 4. Effort & sequencing summary

| Phase | Duration | Parallelizable? | Invariants hardened |
|-------|----------|-----------------|---------------------|
| 0 | 3–4 d | solo | #2, #3, #4(partial), #5 |
| 1 | 6–7 d | solo (after 0) | #1, #4(full), #6, #7, #10 |
| 2 | 5–6 d | yes (after 1; overlaps 3) | #2(shell), #9 |
| 3 | 6–7 d | yes (after 1) | #8, #4(particle path) |
| 4 | 4–5 d | yes (after 1) | #6 |
| 5 | 3–4 d | yes (after 1+2) | #1(full) |
| 6 | 5–6 d | yes (after 2) | — |
| 7 | 5–6 d | yes (after 5+6) | — |
| 8 | ongoing | yes (after 3) | — |
| **Critical path** | **~5 weeks** | | to a professional, fool-proof Forgum |

---

## 5. The "professional program" checklist

This is the bar. Forgum is "professional, not a cheap knock-off" when every box is checked.

### 5.1 Architecture
- [ ] 3-thread engine (sim / render / control), no shared mutable state without a channel.
- [ ] `Renderer` trait with 4 backends, runtime selection, transparent fallback.
- [ ] `forgum-platform` crate; zero `#[cfg]` in `engine/src/`.
- [ ] Fixed-timestep accumulator; integer nanoseconds; spiral-of-death clamp.
- [ ] DCL singletons via `OnceLock`/`LazyLock`; zero `static mut`; zero `lazy_static!`.

### 5.2 Correctness
- [ ] RAII on every terminal mutation; `catch_unwind` around render loop.
- [ ] Signal handlers: SIGTERM/INT/HUP/WINCH/TSTP/CONT + Windows console-control.
- [ ] No input reads in background mode (CI grep).
- [ ] `Cell::dirty` excluded from `PartialEq`; scheduler idles to 5fps for static content.
- [ ] `duration=0` = infinite; saturating math everywhere.

### 5.3 Performance & memory
- [ ] Zero allocation in hot loop (dhat CI gate).
- [ ] `bumpalo::Bump` per-frame arena; `reset()` at frame top.
- [ ] `slotmap::SlotMap` for particles; O(1) spawn/kill.
- [ ] `rayon::par_chunks_mut` for particle integration.
- [ ] Coalesced run rendering; particle color bucketing.
- [ ] CPU < 1% for static cow over 60s; < 5% for active cow at 60fps.
- [ ] SSH bandwidth < 5 KB/s per cow (coalesced runs).
- [ ] Miri clean on unsafe; valgrind `--leak-check=full` clean (with suppressions for intentional statics).

### 5.4 Per-animal quality
- [ ] 7-axis DNA profile for all 109 cows in `animations.json`.
- [ ] Per-base easing defaults; overridable per cow.
- [ ] OKLCH color gradients (not HSV) for palettes.
- [ ] Verlet chains for tails/capes.
- [ ] Per-instance phase randomization (golden ratio); herds desync.
- [ ] Golden framebuffer hash per cow (blake3); CI visual regression.
- [ ] Perceptual hash (pHash) weekly check.

### 5.5 Cross-platform
- [ ] 7 prebuilt binaries + universal macOS + ARM64 musl.
- [ ] Tier-1 matrix green in CI (Windows 11, macOS 14+, Ubuntu/Debian/Fedora/Arch; x86_64 + aarch64).
- [ ] Tier-1 shells (pwsh 7, bash 5, zsh 5.8, fish 3) + terminals (Windows Terminal, iTerm2, Alacritty, kitty, gnome-terminal, tmux 3.3+).
- [ ] `clippy -D warnings` clean; `cargo fmt --check` clean.
- [ ] SBOM per release artifact.

### 5.6 Integration
- [ ] tmux: 4 surfaces (pane overlay, status line, popup, focus-aware hooks).
- [ ] zellij/wezterm/screen: tailored install + status.
- [ ] herbstluftwm: herder via `herbstclient --idle`; Wayland graceful degrade.
- [ ] rmux/remote: SSH `RemoteForward`; deterministic-effect peer sync.
- [ ] `forgum herd`: list/stop/effect/speed/pause/resume/follow/census/theme.

### 5.7 Testing
- [ ] 6-tier test pyramid (unit → integration → E2E-under-tmux → fuzz → bench → golden-visual).
- [ ] 20 E2E scenarios under `xvfb-run tmux` + portable-pty.
- [ ] Per-phase test gate green on all Tier-1 platforms.
- [ ] `cargo-fuzz` nightly on cow parser + config JSON.
- [ ] `criterion` benches on every PR; regression detection at 5%.

### 5.8 Distribution & DX
- [ ] One-command install on every Tier-1 OS.
- [ ] Version parity CI (single source in `Cargo.toml`).
- [ ] Completion drift CI.
- [ ] `forgum init <shell>` generates correct hooks + completions.
- [ ] Community cow packs (`~/.config/forgum/cows/` + `custom.anim.json`).

---

## 6. Risk register & mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| wgpu adds 3-5 MB to binary; GPU drivers missing on SSH/CI | Med | Med | `--gpu` is opt-in feature flag; CPU path always works; never a dependency |
| kitty graphics protocol inside tmux strips sequences | High | Med | `allow-passthrough` detection + `wrap_for_tmux()`; degrade to `SyncAnsiRenderer` |
| Deterministic peer sync breaks on `f32` across CPU archs | High | High | `fixed::I32F32` for visible state; `StdRng::seed_from_u64`; golden-hash equality test across peers |
| `OnceLock` poisoning on panic in init closure | Low | Med | `OnceLock` doesn't poison (unlike `LazyLock`); use `OnceLock` for resettable, `LazyLock` only for truly static |
| `bumpalo::Bump::reset()` skips `Drop` for per-frame values | Med | Low | Only put `Copy`/trivially-droppable data in arena; wrap `Drop`-needing values in `bumpalo::boxed::Box<T>` |
| Windows `timeBeginPeriod(1)` affects global timer resolution | Med | Low | Call only on active tier; restore on idle; document the side-effect |
| herbstluftwm-wl IPC differs from X11 | Med | Med | Detect `$WAYLAND_DISPLAY`; degrade to per-window daemons; log warning |
| `cargo-dist` ubuntu-20.04 runner removed (May 2025) | High | Low | Keep hand-rolled GitHub Actions matrix; evaluate `cargo-dist` after fix stabilizes |

---

## 7. What "done" looks like (the v2.0 release)

A user on any Tier-1 OS runs one install command. They open a terminal. They type `forgum "hello"`. A cow — breathing, blinking, with a tail that flicks every few seconds — fades in above their prompt with a smooth `ease_out_cubic`. Their prompt is untouched. They type `ls -la` and it works. They resize the window; the cow reflows. They `Ctrl-C` the daemon; the overlay clears, the cursor is exactly where it was.

They open tmux. Six panes light up with six different cows — a dragon breathing fire, a dolphin blowing bubbles, a nyan trailing stars, a tux wobbling, a ghost dissolving, a kitty with a Verlet-physics tail. They type `forgum herd effect aurora --all`; all six swap effects in under 100ms. They `forgum herd follow`; only the focused pane animates, the others idle to 5fps.

They SSH to a server with `RemoteForward`. Their local cow follows them. They run `forgum demo`; a 30-second showcase reel plays.

They close the terminal. The daemon survives (detached). They reopen; `precmd` sweep finds it alive. They `kill -9` it; next prompt, the overlay is swept clean.

Zero memory growth over a week of uptime. Zero keystrokes stolen. Zero terminal corruption. Every cow unique. This is Forgum v2.0.

---

**Companion documents:**
- `11-ENGINE-INTERNALS-V2.md` — the deep dive on the 3-thread engine, DCL singletons, arena allocation, `Renderer` trait, fixed-timestep scheduler.
- `12-PER-ANIMAL-ANIMATION-DESIGN.md` — the 7-axis DNA profile, per-base easing, OKLCH color, Verlet chains, the full 109-cow DNA table.
- `13-TEST-COVERAGE-MATRIX.md` — the 6-tier test pyramid, per-phase gates, CI enforcement rules.

**Supersedes:** `07-DEVELOPMENT-ROADMAP.md` (kept for historical context; v2 is authoritative).
