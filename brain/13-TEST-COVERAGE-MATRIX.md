# Forgum — Test Coverage Matrix

> **The "tests are the spec" document.** Every invariant in `10-FINE-TUNED-MASTER-MASTER-PLAN.md` and every design in `11-ENGINE-INTERNALS-V2.md` / `12-PER-ANIMAL-ANIMATION-DESIGN.md` has a named test here. A phase is "done" when its test gate is green on all Tier-1 platforms — not when the code is written.
>
> **Audience:** the engineer writing `crates/engine/tests/` and `.github/workflows/`. This is the checklist you check boxes against.

---

## 1. The 6-tier test pyramid

```
                    ┌─────────────────┐
                    │  6. Perceptual  │  weekly, CI-slow
                    │     (pHash)     │  "does this look right"
                    └─────────────────┘
                  ┌───────────────────────┐
                  │  5. Golden-visual     │  every PR
                  │     (blake3 hash)     │  "did the output change"
                  └───────────────────────┘
                ┌─────────────────────────────┐
                │  4. Fuzz                    │  nightly
                │     (cargo-fuzz)            │  "does it crash on bad input"
                └─────────────────────────────┘
              ┌───────────────────────────────────┐
              │  3. Bench (criterion)             │  every PR
              │     regression detection          │  "did it get slower"
              └───────────────────────────────────┘
            ┌─────────────────────────────────────────┐
            │  2. E2E (xvfb-run tmux + portable-pty)  │  every PR
            │     20 scenarios                        │  "does it work end-to-end"
            └─────────────────────────────────────────┘
          ┌───────────────────────────────────────────────┐
          │  1. Unit + Integration                        │  every PR
          │     (cargo test + Pester)                    │  "does each fn work"
          └───────────────────────────────────────────────┘
```

| Tier | Tool | Frequency | What it proves | Cost |
|------|------|-----------|----------------|------|
| 1. Unit/Integration | `cargo test` + Pester | every PR | Each function correctness | seconds |
| 2. E2E | `xvfb-run tmux` + `portable-pty` | every PR | End-to-end flows; prompt safety; keystroke pass-through | minutes |
| 3. Bench | `criterion` | every PR | Performance regression detection | minutes |
| 4. Fuzz | `cargo-fuzz` (libFuzzer) | nightly | Parser robustness; no crashes on malformed input | hours |
| 5. Golden-visual | `blake3` framebuffer hash | every PR | Visual regression; "the dragon stopped breathing" | seconds |
| 6. Perceptual | `tiny-skia` + pHash | weekly | "Does it look right" (catches hash-blind changes) | minutes |

---

## 2. Tier 1 — Unit + Integration tests (`cargo test` + Pester)

### 2.1 Engine unit tests (`crates/engine/src/**/*.rs` `#[cfg(test)]`)

| Module | Test | Asserts |
|--------|------|---------|
| `framebuffer.rs` | `test_cell_new_is_dirty` | `Cell::new` sets `dirty=true` |
| `framebuffer.rs` | `test_set_and_get_cell` | `set_cell` then `get_cell` returns the cell |
| `framebuffer.rs` | `test_bounds_check` | OOB `set_cell` is a no-op |
| `framebuffer.rs` | `test_compute_damage` | damage detected on back≠front, cleared after render |
| `framebuffer.rs` | `test_dirty_excluded_from_partial_eq` | **BUG-E1 fix**: two cells differing only in `dirty` are equal |
| `framebuffer.rs` | `test_resize_clears` | resize resets back+front+damage |
| `framebuffer.rs` | `test_region_clip` | `set_cell_in_region` respects clip bounds |
| `scheduler.rs` | `test_scheduler_creation` | `Scheduler::new(30)` → `current_fps=30` |
| `scheduler.rs` | `test_clamp_fps` | `new(200)` → 120; `new(0)` → 1 |
| `scheduler.rs` | `test_idle_adaptation` | 20 frames of 0 damage → idle_fps |
| `scheduler.rs` | `test_active_adaptation` | after idle, 1 frame of damage → active_fps |
| `scheduler.rs` | `test_fixed_timestep_accumulator` | 120Hz render, 60Hz sim → 1 sim step per 2 render frames |
| `scheduler.rs` | `test_spiral_clamp` | frame_ns > 250ms → accumulator capped, no spiral |
| `scheduler.rs` | `test_integer_ns_no_drift` | 3-hour simulated run: no precision loss |
| `particles.rs` | `test_slotmap_spawn_kill` | O(1) spawn returns key; kill removes; stale key invalid |
| `particles.rs` | `test_pool_full_returns_none` | capacity reached → spawn returns None (no silent drop) |
| `particles.rs` | `test_integrate_euler` | particle at (0,0) v=(1,1) dt=1 → (1,1) |
| `particles.rs` | `test_rayon_parallel_integrate` | 10k particles, par_chunks_mut == serial result |
| `color.rs` | `test_hsv_to_rgb_primary` | hsv(0,1,1)→red, hsv(120,1,1)→green, hsv(240,1,1)→blue |
| `color.rs` | `test_oklch_gradient_uniform` | 10 steps in OKLCH: perceptual deltas ≈ equal |
| `color.rs` | `test_gaussian_glow_falloff` | at d=0 → 1.0; at d=radius → ~0.135; at d=2*radius → ~0 |
| `color.rs` | `test_lolcat_formula` | canonical lolcat at (0,0,0,0) matches reference |
| `color.rs` | `test_rgb_to_xterm256` | white→231, black→16, red→196 |
| `color.rs` | `test_bayer_dither` | dithered quantize distributes error |
| `style_matcher.rs` | `test_known_cow_default` | default.cow → Breathe, speed 1.0 |
| `style_matcher.rs` | `test_known_cow_dragon` | dragon.cow → Fire, Fire particles |
| `style_matcher.rs` | `test_known_cow_nyan` | nyan.cow → Fly, speed 2.0, Stars |
| `style_matcher.rs` | `test_unknown_cow_defaults` | unknown → Breathe, speed 1.0 |
| `style_matcher.rs` | `test_all_109_cows_have_dna` | every .cow in Data/Cows has an animations.json entry |
| `animation/easing.rs` | `test_easing_endpoints` | all 8 fns: f(0)=0, f(1)=1 |
| `animation/easing.rs` | `test_easing_monotonic` | cubic_inout, sine_inout, expo_out monotonic on [0,1] |
| `animation/verlet.rs` | `test_verlet_chain_pinned` | link 0 stays at anchor after step |
| `animation/verlet.rs` | `test_verlet_constraint_distance` | adjacent links maintain rest_len ± 0.01 after 4 iterations |
| `animation/verlet.rs` | `test_verlet_impulse` | impulse moves link, propagates to children |
| `animation/dna.rs` | `test_dna_parses` | all 109 entries parse against schema |
| `animation/dna.rs` | `test_phase_seed_unique` | all 109 phase_seeds are distinct |
| `animation/dna.rs` | `test_instance_phase_desyncs` | 5 instances of default.cow → 5 distinct breath phases |
| `effects.rs` | `test_all_19_effects_render` | each effect: update + render doesn't panic, produces non-empty fb |
| `effects.rs` | `test_effect_is_done` | Shatter/Dissolve: is_done()=true after particles die |
| `effects.rs` | `test_effect_on_resize` | on_resize updates offset; no panic on 1×1 terminal |
| `effects.rs` | `test_create_effect_applies_dna` | dragon.cow → Fire effect with speed=1.0, Fire particles rate=18 |
| `effects.rs` | `test_aurora_hue_cycles` | 360 frames → hue advanced by ~360° |
| `effects.rs` | `test_portal_hue_modulo` | **BUG-E6 fix**: negative hue wraps correctly |
| `effects.rs` | `test_bounce_kinematics` | **BUG-E5 fix**: parabolic trajectory, squash on land |
| `region.rs` | `test_allocator_allocate` | allocate returns id, get returns bounds |
| `region.rs` | `test_allocator_resize` | resize_canvas + resize_region update bounds |
| `protocol.rs` | `test_scene_config_parse` | valid JSON → SceneConfig |
| `protocol.rs` | `test_scene_config_rejects_invalid` | malformed JSON → Err |
| `protocol.rs` | `test_control_msg_roundtrip` | all variants serialize + deserialize |
| `terminal.rs` | `test_term_caps_probe` | probe returns TermCaps with all fields populated |
| `terminal.rs` | `test_tmux_detection` | $TMUX set → tmux=true |
| `terminal.rs` | `test_kitty_detection` | mock DCS reply → kitty_graphics=true |
| `main.rs` | `test_resolve_effect_auto` | auto + cow_file → cow's base |
| `main.rs` | `test_duration_zero_infinite` | duration=0 → max_frames=0 (infinite) |
| `main.rs` | `test_bounded_stdin` | 4MB+1 input → rejected |

### 2.2 Platform integration tests (`crates/platform/tests/`)

| Test | Asserts |
|------|---------|
| `test_raw_mode_raii` | enable → Drop → terminal restored (even on panic via catch_unwind) |
| `test_alt_screen_raii` | EnterAlternateScreen → Drop → LeaveAlternateScreen |
| `test_cursor_hide_raii` | Hide → Drop → Show |
| `test_signal_install` | install_shutdown_handler → SIGTERM sets flag |
| `test_sigwinch_install` | install_resize_handler → SIGWINCH sets flag |
| `test_sigtstp_sigcont_dance` | SIGTSTP → terminal restored; SIGCONT → reacquired |
| `test_open_render_output_stdout` | is_terminal → Stdout variant |
| `test_open_render_output_tty` | piped stdout + /dev/tty available → Tty variant |
| `test_open_render_output_pipe` | piped stdout + no tty → Pipe variant |
| `test_detach_setsid` | spawn_daemon → child has different pgid (setsid worked) |
| `test_config_path_xdg` | $XDG_CONFIG_HOME → ~/.config/Forgum/config.json |
| `test_config_path_appdata` | %APPDATA% → Forgum\config.json |
| `test_control_socket_mode` | socket created with mode 0600 |
| `test_control_socket_session` | $TMUX_PANE → path includes pane id |

### 2.3 Pester tests (`Tests/*.Tests.ps1`)

The existing 14 Pester test files (216+ tests) are kept and extended:

| File | What |
|------|------|
| `Forgum.Tests.ps1` | module loads, public API |
| `CLI.Tests.ps1` | subcommands parse |
| `Subcommands.Tests.ps1` | each subcommand works |
| `NewSubcommands.Tests.ps1` | herd/theme/tmux |
| `CowAnimation.Tests.ps1` | cow rendering |
| `CrossPlatform.Tests.ps1` | Windows/Linux path detection |
| `Engine.Tests.ps1` | engine binary invocation |
| `Engine-Manual.Tests.ps1` | manual engine scenarios |
| `Ghost.Tests.ps1` | ghost effect |
| `LiveShow.Tests.ps1` | live mode |
| `Permutations.Tests.ps1` | cow×effect×eyes matrix |
| `ShellUsability.Tests.ps1` | shell hooks |
| `Visual.Tests.ps1` | visual output snapshots |
| `Comprehensive.Tests.ps1` | end-to-end |

**New Pester tests (v2):**
| Test | Asserts |
|------|---------|
| `Forgum.Engine.Version` | `forgum-engine --version` matches `Forgum.psd1` version |
| `Forgum.Completions.NoDrift` | `forgum-engine completions bash` == committed `forgum.bash` |
| `Forgum.Config.LiveReload` | change config.json → next `forgum` call uses new effect |
| `Forgum.Herd.MultiPane` | 2 panes: `forgum herd stop --all` kills both |

---

## 3. Tier 2 — E2E tests (`xvfb-run tmux` + `portable-pty`)

These are the **most important tests** — they catch the bugs that unit tests can't (terminal corruption, keystroke theft, prompt breakage). They run under a real pty + tmux so the full terminal stack is exercised.

### 3.1 The harness

```bash
# Linux CI: xvfb-run provides a virtual display; tmux provides the multiplexer
xvfb-run -a bash -c '
  tmux new-session -d -s test "bash" &&
  tmux send-keys -t test "forgum daemon start --effect aurora" Enter &&
  sleep 2 &&
  tmux capture-pane -t test -p > /tmp/pane.txt &&
  tmux send-keys -t test "echo HELLO_MARKER" Enter &&
  sleep 1 &&
  tmux capture-pane -t test -p > /tmp/pane2.txt
'
# Assertions:
# 1. /tmp/pane.txt contains aurora animation chars (overlay rendered)
# 2. /tmp/pane2.txt contains "HELLO_MARKER" (keystroke pass-through worked)
# 3. No MoveTo with y >= ob_y1 in the raw ANSI stream
```

For Windows: `portable-pty` (Rust crate) spawns a ConPTY and drives it programmatically.

### 3.2 The 20 E2E scenarios

| # | Scenario | Asserts | Catches |
|---|----------|---------|---------|
| E1 | Background animation renders above prompt | pane.txt has animation in rows 0..ob_y1, prompt in rows ob_y1.. | BUG-B1, B3, B4 |
| E2 | Keystroke pass-through | type `echo hi<Enter>` while animating → shell executes `echo hi` | BUG-B1 (definitive) |
| E3 | Prompt row untouched | raw ANSI stream: no `MoveTo` with `y >= ob_y1` | BUG-B1/B3/B4 |
| E4 | Cursor save/restore balance | every `ESC 7` has matching `ESC 8` before next frame's `ESC 7` | cursor leaks |
| E5 | Static cow idle (CPU < 1%) | 60s run, `ps` avg CPU < 1% | BUG-E1 |
| E6 | Resize mid-animation | SIGWINCH after 2s → overlay reflows, no bytes below ob_y1 | BUG-B3/B4/F2 |
| E7 | Signal exit cleanliness | `kill -TERM` → stream ends with overlay-clear + `ESC[0m` | BUG-T1/B4 |
| E8 | `kill -9` sweep | `kill -9` + trigger precmd → overlay cleared | §9.3 net |
| E9 | Duration=0 infinite | render with duration:0 → still alive after 10s | BUG-B2 |
| E10 | Daemon survives shell exit | close shell → daemon still in `ps` | BUG-B7 |
| E11 | Two panes independent | 2 panes animate; stop one → other continues | BUG-D3 |
| E12 | Control socket: EFFECT hot-swap | `forgum daemon effect ember` → effect changes < 100ms | new |
| E13 | Control socket: SPEED live | `forgum daemon speed 2.0` → animation 2x | new |
| E14 | Control socket: STATUS | `forgum daemon status` → JSON with fps, effect, pid | new |
| E15 | tmux focus-aware | focus pane → animates; unfocus → idles | Phase 6 |
| E16 | herbstluftwm herder | `forgum herd` → focused frame animates, others idle | Phase 6 |
| E17 | Remote follow-me | SSH RemoteForward → local cow follows | Phase 7 |
| E18 | Kitty graphics backend | in kitty terminal → KittyAnimRenderer engages (damage > 60%) | Phase 4 |
| E19 | wgpu fallback | `forgum --gpu` on headless → CPU fallback, no crash | Phase 4 |
| E20 | Full-screen foreground mode | `forgum run --no-background` → q/Esc exits cleanly | BUG-B5 |

### 3.3 The ANSI stream assertion library

A small Rust helper (`tests/ansi_assert.rs`) parses the captured ANSI stream and provides:

```rust
pub fn assert_no_writes_below(stream: &[u8], ob_y1: u16);
pub fn assert_cursor_save_restore_balanced(stream: &[u8]);
pub fn assert_ends_with_overlay_clear(stream: &[u8]);
pub fn assert_no_raw_mode_left(stream: &[u8]);  // no lingering raw-mode escape
pub fn assert_synchronized_update_balanced(stream: &[u8]);  // Begin == End count
```

---

## 4. Tier 3 — Benchmarks (`criterion`)

### 4.1 The benchmark suite (`crates/engine/benches/`)

| Bench | What | Target |
|-------|------|--------|
| `benches/engine.rs::damage_diff` | `compute_damage` on 1920 cells, 50% dirty | < 50µs |
| `benches/engine.rs::render_region` | `render_region` full 80×24 overlay | < 2ms |
| `benches/engine.rs::coalesced_runs` | run-coalescing on 80-char row, all dirty | < 20µs |
| `benches/anim.rs::particle_update_10k` | integrate 10k particles, dt=16ms | < 500µs |
| `benches/anim.rs::particle_update_1k` | integrate 1k particles (single-thread threshold) | < 50µs |
| `benches/anim.rs::particle_spawn_kill` | 1000 spawn + 1000 kill cycles | < 100µs |
| `benches/anim.rs::verlet_step_4link` | 4-link Verlet + 4 constraint iterations | < 5µs |
| `benches/anim.rs::frame_tick_60fps` | full sim tick (update + render into fb, no I/O) | < 5ms |
| `benches/color.rs::oklch_gradient` | 100-step OKLCH gradient interpolation | < 50µs |
| `benches/color.rs::gaussian_glow_1920` | glow on 1920 cells | < 100µs |

### 4.2 Regression detection

`criterion` stores a baseline; on the next run it reports `% change` and statistical significance. CI fails on regressions > 5%:

```bash
cargo bench --bench engine -- --save-baseline main
# On PR:
cargo bench --bench engine -- --baseline main
# Criterion exits non-zero if any measurement regressed > 5% with p < 0.05
```

### 4.3 The `dhat` zero-alloc test (the memory invariant)

```rust
// tests/alloc_budget.rs
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn hot_loop_zero_alloc_after_warmup() {
    let mut sim = SimState::new(test_config());
    for _ in 0..10 { sim.tick(16_666_667); }  // warmup
    let before = dhat::GlobalAlloc::get_stats().total_bytes;
    for _ in 0..60 { sim.tick(16_666_667); }  // measured
    let after = dhat::GlobalAlloc::get_stats().total_bytes;
    assert_eq!(after - before, 0, "hot loop allocated {} bytes after warmup", after - before);
}

#[test]
fn render_thread_zero_alloc_after_warmup() {
    // Same for the render path (compute_damage + coalesced runs + emit)
}
```

CI gate: `cargo test --test alloc_budget` must pass.

---

## 5. Tier 4 — Fuzz tests (`cargo-fuzz`)

### 5.1 Fuzz targets (`crates/engine/fuzz/`)

| Target | What it fuzzes | Invariant |
|--------|----------------|----------|
| `fuzz_targets/parse_cow_file.rs` | `parse_cow_file(&[u8])` | no panic, no infinite loop, returns Ok or Err |
| `fuzz_targets/parse_animations_json.rs` | `parse_animations_json(&str)` | no panic; valid JSON → parses; invalid → Err |
| `fuzz_targets/parse_scene_config.rs` | `serde_json::from_str::<SceneConfig>` | no panic |
| `fuzz_targets/control_msg.rs` | control socket command parser | no panic; unknown → Err |
| `fuzz_targets/render_arbitrary_cow.rs` | `create_effect` + `render` on arbitrary cow text | no panic, no OOB |

### 5.2 CI schedule

```yaml
# .github/workflows/fuzz.yml
on:
  schedule:
    - cron: '0 2 * * *'  # nightly 02:00 UTC
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: cargo fuzz run parse_cow_file -- -max_len=4096 -max_total_time=600
      - run: cargo fuzz run parse_animations_json -- -max_len=8192 -max_total_time=600
      # ... etc
```

Each target runs for 10 minutes nightly. Crashes file issues automatically.

---

## 6. Tier 5 — Golden-visual tests (blake3 framebuffer hash)

### 6.1 The mechanism

For each of 109 cows, render 60 frames at 60fps into a `Vec<Cell>` framebuffer, hash with `blake3`, store the hash. CI fails if the hash changes.

```rust
// tests/golden_visual.rs
#[test]
fn golden_framebuffer_per_cow() {
    for cow in all_cows() {
        let dna = load_dna(cow);
        let mut sim = SimState::new(dna, instance_id=0);
        let mut fb = FrameBuffer::new(80, 24);
        for _ in 0..60 { sim.tick(16_666_667); sim.render(&mut fb); }
        let hash = blake3::hash(fb.as_bytes());
        let golden_path = format!("Tests/golden/{}.blake3", cow);
        if std::env::var("REGOLDEN").is_ok() {
            std::fs::write(&golden_path, hash.to_hex().as_bytes()).unwrap();
            continue;
        }
        let golden = std::fs::read_to_string(&golden_path)
            .unwrap_or_else(|_| panic!("missing golden for {}; run with REGOLDEN=1", cow));
        assert_eq!(hash.to_hex().as_str(), golden.trim(),
            "{}: visual regression — golden hash mismatch. \
             If intentional, run: REGOLDEN=1 cargo test golden_framebuffer_per_cow", cow);
    }
}
```

### 6.2 Why blake3

- **Speed:** ~6 GB/s. 60 frames × 30 KB = 1.8 MB hashed in < 1ms.
- **Determinism:** same input → same hash, across platforms.
- **Collision resistance:** 256-bit; no false positives.

### 6.3 The re-golden workflow

When an intentional change alters the visual output (e.g. tuning the dragon's fire rate):

```bash
REGOLDEN=1 cargo test golden_framebuffer_per_cow
git add Tests/golden/*.blake3
git commit -m "regolden: dragon fire rate tuned to 18"
```

The PR review checks: did the cow that was supposed to change, change? Did any *other* cow change unexpectedly?

### 6.4 Cross-backend golden parity

The same cow rendered through `AnsiRenderer`, `SyncAnsiRenderer`, `KittyAnimRenderer`, and `WgpuHybridRenderer` must produce the **same framebuffer** (the contract). The golden test runs against all 4 backends:

```rust
#[test]
fn golden_parity_across_backends() {
    for backend in ["ansi", "sync_ansi", "kitty", "wgpu"] {
        let fb = render_cow_with_backend("dragon.cow", backend, 60);
        let hash = blake3::hash(fb.as_bytes());
        assert_eq!(hash.to_hex().as_str(), golden_dragon_hash,
            "dragon.cow via {} produced different framebuffer than golden", backend);
    }
}
```

This enforces the "software path is the contract; hardware is the accelerator" invariant.

---

## 7. Tier 6 — Perceptual hash tests (weekly, "does it look right")

### 7.1 Why perceptual hash

Golden-blake3 is byte-exact — it catches any change, even invisible ones (a 1-pixel shift changes every byte). Perceptual hash (pHash) catches *visual* changes and ignores invisible ones. Run weekly (slow), it's the "does this look right to a human" check.

### 7.2 The mechanism

1. Render 60 frames into framebuffer.
2. Rasterize the final framebuffer to a PNG via `tiny-skia` (each Cell → a 10×10 px tile with the glyph + color).
3. Convert PNG to 8×8 grayscale DCT.
4. Compute the 64-bit pHash.
5. Compare to reference pHash with Hamming distance ≤ 5.

```rust
// tests/perceptual.rs (weekly CI)
#[test]
fn perceptual_hash_per_cow() {
    for cow in all_cows() {
        let fb = render_cow_60_frames(cow);
        let png = rasterize_to_png(&fb, 10);  // 10px per cell
        let phash = compute_phash(&png, 8);   // 8x8 DCT → 64-bit
        let ref_phash = load_ref_phash(cow);
        let distance = (phash ^ ref_phash).count_ones();
        assert!(distance <= 5,
            "{}: perceptual hash distance {} > 5 — visual regression", cow, distance);
    }
}
```

### 7.3 When pHash catches what blake3 misses

- A refactor renames a field; blake3 changes (panic), pHash unchanged (no visual change) → pHash says "OK."
- A bug shifts every glyph by 1px; blake3 changes (every byte different), pHash changes slightly (distance ~8) → pHash flags it.

Together, blake3 (strict) + pHash (perceptual) give both correctness and human-judgment coverage.

---

## 8. CI enforcement rules (the gates)

### 8.1 Per-PR gates (must pass before merge)

| Gate | Tool | Command |
|------|------|---------|
| Format | rustfmt | `cargo fmt --check` |
| Lint | clippy | `cargo clippy -D warnings` |
| Unit + integration | cargo test | `cargo test --workspace` |
| Pester | Pester | `Invoke-Pester -Path Tests/ -Output Detailed` |
| E2E | xvfb-run tmux | `xvfb-run -a ./tests/e2e/run_all.sh` |
| Bench regression | criterion | `cargo bench --baseline main` (no > 5% regressions) |
| Zero-alloc | dhat | `cargo test --test alloc_budget` |
| Golden-visual | blake3 | `cargo test --test golden_visual` |
| cfg containment | ripgrep | `rg '#\[cfg' crates/engine/src/` → 0 hits |
| No static mut | ripgrep | `rg 'static mut' crates/engine/src/` → 0 hits |
| No Box::leak | ripgrep | `rg 'Box::leak|mem::forget' crates/engine/src/` → 0 hits (reviewed exceptions) |
| No event::poll/read in background | ripgrep | `rg 'event::(poll|read)' crates/engine/src/render_background.rs` → 0 hits |
| Completion drift | diff | `diff <(forgum-engine completions bash) scripts/completions/forgum.bash` empty |
| Version parity | cargo run | `cargo run -- gen-version` matches all manifests |
| animations.json schema | jsonschema | `jsonschema data/animations.json -s schemas/dna.schema.json` |

### 8.2 Nightly gates

| Gate | Tool | Command |
|------|------|---------|
| Fuzz | cargo-fuzz | 10 min × 5 targets |
| Miri | cargo +nightly miri | `cargo +nightly miri test --workspace` (unsafe blocks) |
| Long-running soak | custom | 24h daemon run; RSS growth < 1 MB; no log spam |
| Tier-1 matrix | GitHub Actions | cargo test on 7 targets |

### 8.3 Weekly gates

| Gate | Tool | Command |
|------|------|---------|
| Perceptual hash | tiny-skia + pHash | `cargo test --test perceptual` |
| valgrind | valgrind | `valgrind --leak-check=full --suppressions=forgum-leaks.supp target/debug/forgum-engine daemon --duration 60` |
| heaptrack flamegraph | heaptrack | manual review of alloc hotspots |

### 8.4 Pre-release gates

| Gate | Tool | Command |
|------|------|---------|
| All Tier-1 platforms green | CI matrix | all 7 targets pass unit + E2E |
| SBOM generated | cargo-cyclonedx | `cargo cyclonedx` → `bom.json` per artifact |
| sha256 published | shasum | every release artifact has `.sha256` |
| Version parity | manual | `Cargo.toml` == `Forgum.psd1` == Homebrew == scoop == winget == AUR |

---

## 9. Per-phase test gates (the "definition of done")

This is the master checklist. A phase is complete only when every box in its row is ✅ on all Tier-1 platforms.

| Phase | Unit | E2E | Bench | Zero-alloc | Golden | cfg-grep | Special |
|-------|------|-----|-------|------------|--------|----------|---------|
| 0 | ✅ existing + dirty-PartialEq + duration=0 | ✅ E1-E5, E7, E9 | — | partial | — | — | catch_unwind, signal install |
| 1 | ✅ all framebuffer/scheduler/particles/platform | ✅ E1-E14 (full overlay suite) | ✅ baseline saved | ✅ full | — | ✅ 0 cfg hits | Miri clean on unsafe |
| 2 | ✅ cli/cow/config/completions | ✅ E2 (keystroke), E20 (foreground) | — | — | — | — | completion-drift, version-parity |
| 3 | ✅ all animation/easing/verlet/dna/effects | — | ✅ anim benches pass | — | ✅ 109 cow hashes | — | herd-desync, palette-uniqueness |
| 4 | ✅ renderer trait + 4 backends + caps probe | ✅ E18 (kitty), E19 (wgpu fallback) | — | — | ✅ cross-backend parity | — | — |
| 5 | ✅ platform tests on 7 targets | ✅ xvfb-run on x86_64 + aarch64 | — | — | — | ✅ | clippy clean, SBOM |
| 6 | ✅ herd/tmux/herbstluftwm | ✅ E15 (focus), E16 (herder) | — | — | — | — | — |
| 7 | ✅ remote sync | ✅ E17 (remote) | — | — | ✅ peer golden parity | — | deterministic sync |
| 8 | per-feature | per-feature | per-feature | per-feature | per-feature | — | feature-flagged |

---

## 10. The test file inventory

```
crates/engine/
├── src/**/*.rs                    # #[cfg(test)] unit tests inline
├── tests/
│   ├── alloc_budget.rs            # dhat zero-alloc (Tier 3)
│   ├── golden_visual.rs           # blake3 per-cow (Tier 5)
│   ├── perceptual.rs              # pHash weekly (Tier 6)
│   ├── pty_keystroke.rs           # E2: portable-pty keystroke pass-through (Tier 2)
│   ├── resize_cleanliness.rs      # E6: SIGWINCH mid-animation (Tier 2)
│   ├── ansi_assert.rs             # helper: ANSI stream assertions
│   └── e2e/                       # 20 scenarios (Tier 2)
│       ├── run_all.sh
│       ├── e01_overlay_renders.sh
│       ├── e02_keystroke_passthrough.sh
│       ├── ...
│       └── e20_foreground_mode.sh
├── benches/
│   ├── engine.rs                  # damage_diff, render_region, coalesced_runs
│   ├── anim.rs                    # particle_update, verlet_step, frame_tick
│   └── color.rs                   # oklch_gradient, gaussian_glow
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        ├── parse_cow_file.rs
        ├── parse_animations_json.rs
        ├── parse_scene_config.rs
        ├── control_msg.rs
        └── render_arbitrary_cow.rs

crates/platform/
├── src/**/*.rs                    # #[cfg(test)] inline
└── tests/
    ├── raw_mode_raii.rs
    ├── alt_screen_raii.rs
    ├── signal_install.rs
    ├── open_render_output.rs
    ├── detach_setsid.rs
    └── control_socket.rs

Tests/                             # Pester (PowerShell)
├── Forgum.Tests.ps1
├── CLI.Tests.ps1
├── ... (14 existing files)
├── Engine.Version.Tests.ps1       # new
├── Completions.NoDrift.Tests.ps1  # new
├── Config.LiveReload.Tests.ps1    # new
└── Herd.MultiPane.Tests.ps1       # new

Tests/golden/                      # blake3 hashes (Tier 5)
├── default.cow.blake3
├── dragon.cow.blake3
├── ... (109 files)

Tests/perceptual/                  # pHash references (Tier 6)
├── default.cow.phash
├── dragon.cow.phash
├── ... (109 files)

schemas/
└── dna.schema.json                # animations.json JSON Schema

forgum-leaks.supp                  # valgrind suppressions (intentional LazyLock statics)
```

---

## 11. Coverage measurement

### 11.1 Rust coverage (`cargo-tarpaulin` or `cargo-llvm-cov`)

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --html --output-dir coverage/
# Target: ≥ 90% line coverage on crates/engine/src/ and crates/platform/src/
# Target: 100% coverage on crates/engine/src/animation/ (the DNA system is critical)
```

CI publishes the HTML report as an artifact; the README badge shows the percentage.

### 11.2 Pester coverage (PESTER_SHOW_FAILURES)

Pester doesn't have line coverage, but we track **scenario coverage**: every subcommand × every flag combination is tested at least once. The `Comprehensive.Tests.ps1` enumerates the matrix.

### 11.3 E2E coverage

The 20 E2E scenarios map 1:1 to the invariants in `10-FINE-TUNED-MASTER-PLAN.md` §0. Every invariant has at least one E2E test. The mapping is documented in `tests/e2e/README.md`.

---

## 12. The "professional test suite" checklist

- [ ] 6-tier pyramid implemented (unit → E2E → bench → fuzz → golden → perceptual).
- [ ] ≥ 90% line coverage on `crates/engine/src/` and `crates/platform/src/`.
- [ ] 100% coverage on `crates/engine/src/animation/`.
- [ ] 20 E2E scenarios pass under `xvfb-run tmux` on Linux x86_64 + aarch64.
- [ ] E2E scenarios pass under `portable-pty` on Windows.
- [ ] `dhat` zero-alloc test green (hot loop allocates 0 bytes after warmup).
- [ ] 109 golden-blake3 hashes committed; CI fails on change.
- [ ] Cross-backend golden parity (AnsiRenderer == SyncAnsiRenderer == KittyAnimRenderer == WgpuHybridRenderer).
- [ ] `cargo-fuzz` nightly, 5 targets × 10 min, no crashes.
- [ ] `cargo +nightly miri test` clean on unsafe blocks.
- [ ] `criterion` benches with baseline; > 5% regression fails CI.
- [ ] pHash weekly check; Hamming distance ≤ 5.
- [ ] 24-hour soak test: RSS growth < 1 MB.
- [ ] valgrind `--leak-check=full` clean (with suppressions for intentional statics).
- [ ] All 15 CI gates (§8.1) green on every PR.
- [ ] Tier-1 matrix (7 targets) green.
- [ ] SBOM generated per release artifact.
- [ ] Version parity asserted across all manifests.

When every box is checked, Forgum has a test suite that a senior engineer trusts. That trust is what lets you refactor aggressively, add features confidently, and ship without fear.

---

**Companion documents:**
- `10-FINE-TUNED-MASTER-PLAN.md` — the phased plan (each phase's DoD references this matrix).
- `11-ENGINE-INTERNALS-V2.md` — the engine under test.
- `12-PER-ANIMAL-ANIMATION-DESIGN.md` — the animation system verified by golden + pHash tests.
