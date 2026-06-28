# Forgum — Testing Strategy

> A four-tier test pyramid plus CI gates that enforce every invariant in `06-ARCHITECTURE.md` §4. The current suite (492 Pester + 45 Rust unit tests) is broad but **shallow** — it doesn't actually verify the terminal isn't corrupted, the prompt isn't stolen, or the overlay renders above the right row. This plan closes that gap.

---

## 1. The test pyramid

```
                    ┌──────────────────────┐
                    │  Tier 4: Adversarial  │  fuzz, mutation, property
                    └──────────────────────┘
                  ┌────────────────────────────┐
                  │  Tier 3: E2E (pty + tmux)   │  real terminal, real shell, real daemon
                  └────────────────────────────┘
              ┌─────────────────────────────────────┐
              │  Tier 2: Integration (engine+config) │  JSON protocol, config merge, daemon spawn
              └─────────────────────────────────────┘
          ┌─────────────────────────────────────────────┐
          │  Tier 1: Unit (Rust + Pester)                │  per-function, fast, deterministic
          └─────────────────────────────────────────────┘
```

| Tier | Count target | Runtime | Runs where |
|------|--------------|---------|-----------|
| 1 Unit | ~600 | < 30 s | every push (matrix) |
| 2 Integration | ~80 | < 2 min | every push (matrix) |
| 3 E2E | ~40 | < 8 min | every PR + nightly |
| 4 Adversarial | ~15 (fuzz) | nightly 10 min | nightly |

---

## 2. Tier 1 — Unit tests

### 2.1 Rust unit (in-crate `#[cfg(test)]`)

Expand the current 45 to ~400. Critical modules to reach high coverage:

| Module | What to test |
|--------|--------------|
| `framebuffer` | `set_cell_in_region` clipping, `compute_damage` correctness (BUG-E1 regression: equal cells → 0 damage), `resize` preserves nothing (expected), `render_region` coalescing produces ≤ N moves for N runs |
| `region` | `Rect::contains/intersect`, `clamp_to_canvas`, `resize_canvas` hides out-of-bounds regions, `resize_region` updates bounds |
| `scheduler` | idle transition after 15 zero-damage frames; active rebound on damage; `wait_if_needed` respects target fps; `should_render` skip |
| `color` | `hsv_to_rgb` at 0/60/120/180/240/300/360 and negatives (BUG-E6), `lerp`, 256-color downgrade |
| `style_matcher` | every shipped `.cow` maps to a known effect; `speed`/`particles` populated for themed cows |
| `cow` renderer | placeholder expansion (`$eyes/$tongue/$thoughts`), speech-bubble wrapping at `max_width`, word-wrap, traversal guard rejects `..`/abs/symlink-escape |
| `config` | merge precedence (argv > file > defaults), missing file → defaults, malformed JSON → error (not silent), `$FORGUM_CONFIG` honored |
| `protocol` | `SceneConfig` round-trip serde; unknown fields ignored; `duration=0` → infinite flag |

### 2.2 Property tests (`proptest`)

- `compute_damage(back, front).len()` ≤ count of cells where `ch/fg/bg/alpha` differ (BUG-E1 invariant).
- For any `Rect` and any cell `(x,y)`, `set_cell_in_region` writes iff `clip.contains(x,y)`.
- `hsv_to_rgb(h, s, v)` for `h ∈ (-720, 720)` returns r/g/b ∈ `[0,255]` (no panic, no negatives).
- `coalesced_render` emits `MoveTo` count ≤ number of dirty runs (≤ dirty cell count).

### 2.3 Pester unit (PowerShell)

Keep the existing 492 but:
- Remove tests for the deleted `GetForgumShellHook.ps1`.
- Add tests that `forgum init bash` (delegated to the engine) produces a hook containing the `precmd` sweep and the absolute engine path.
- Add tests for `Get-CFConfig`/`Set-CFConfig` honoring `$env:FORGUM_CONFIG`.
- Add `[int]::TryParse` coverage (repo audit #13 regression).

---

## 3. Tier 2 — Integration tests

### 3.1 Engine + config (Rust integration tests in `tests/`)

- `test_render_background_pipes_to_tty`: spawn engine with `--background` under a pseudo-tty (`rustix::pty` or `portable-pty`), feed JSON, assert the overlay region (`rows 0..ob_y1`) gets ANSI writes and the prompt row (`row = rows-1`) gets **zero** writes. (BUG-B1/B3/B5 gate.)
- `test_daemon_detach_survives_parent`: spawn daemon, drop the parent handle, assert the daemon PID is still alive after 2 s and `daemon.json` exists.
- `test_control_socket_stop`: start daemon, send `STOP` on the control socket, assert the daemon exits within 200 ms and the overlay is cleared.
- `test_control_socket_effect_hotswap`: send `EFFECT ember`, assert the next frame's ANSI contains ember-characteristic sequences.
- `test_resize_sigwinch`: start daemon, send `SIGWINCH` with new size, assert `daemon.json`'s `ob_y1` updated and no writes landed below the new `ob_y1`.
- `test_duration_zero_infinite`: `duration:0` runs > 10 s (must be killed by the test).
- `test_duration_n_seconds`: `duration:2` exits in 2 ± 0.5 s.
- `test_malformed_json_exits_nonzero`: pipe garbage, assert exit code 1 (BUG-D5).
- `test_huge_stdin_rejected`: pipe 5 MB, assert error (BUG-D4).
- `test_config_merge_precedence`: argv `--effect ember` overrides config `animation.mode: aurora`.

### 3.2 Shell-hook integration (bats / fish test / Pester)

- `test_bash_hook_renders`: source the generated bash hook in a non-interactive bash under a pty, run `forgum hi`, assert the engine received `--text hi` and rendered.
- `test_bash_hook_no_keystroke_theft`: under a pty, start `forgum` (background), type `echo hi\n`, assert the shell executed `echo hi` and printed `hi`. (The golden test for BUG-B1.)
- `test_fish_hook_parses`: `fish --no-execute` on the generated hook exits 0.
- `test_zsh_hook_loads`: `zsh -c 'source hook; type forgum'` finds the function.
- `test_precmd_sweep_clears_dead_daemon`: write a stale `daemon.json` with a dead PID, trigger `precmd`, assert the overlay rows were cleared and the file removed.

---

## 4. Tier 3 — End-to-end under a real terminal

This is the tier that **doesn't exist today** and is the reason all the BUG-B/T/S bugs shipped. It runs under `xvfb-run` + `tmux` (Linux) and a Windows Terminal headless mode (Windows).

### 4.1 The pty harness

Use `portable-pty` (pure Rust, cross-platform) to:
1. open a pty,
2. spawn a shell (bash/zsh/fish/pwsh),
3. source the forgum hook,
4. drive input + capture output,
5. assert on the captured ANSI stream.

### 4.2 E2E scenarios (each is one test)

| ID | Scenario | Asserts |
|----|----------|---------|
| E2E-1 | `forgum "hi"` background for 3 s | overlay rows written; prompt row untouched; `ESC7`/`ESC8` balanced; ends with overlay-clear + `ESC[0m` |
| E2E-2 | type `echo hi\n` during background animation | shell prints `hi`; animation continues; no keystrokes lost |
| E2E-3 | resize pane mid-animation (tmux `resize-pane`) | overlay reflows; no writes below new `ob_y1`; no stale pixels |
| E2E-4 | `kill -TERM <daemon>` | overlay cleared; cursor restored; exit code 0 |
| E2E-5 | `kill -9 <daemon>` then trigger prompt | `precmd` sweep clears overlay; `daemon.json` removed |
| E2E-6 | `forgum daemon stop` | daemon exits cleanly; overlay cleared |
| E2E-7 | two panes, `forgum daemon stop` in pane 1 | pane 1 stops; pane 2 still animates |
| E2E-8 | static cow for 60 s | avg CPU < 1 % (BUG-E1) |
| E2E-9 | `forgum herd effect aurora --all` across 3 panes | all 3 switch within 200 ms |
| E2E-10 | `forgum tmux popup --effect shatter --duration 3` | popup appears, shatters, closes; pane untouched |
| E2E-11 | `forgum theme rotate 1` for 3 min | effect changes ~3 times; no pane corruption |
| E2E-12 | `forgum daemon effect ember` hot-swap | next frame is ember; no flicker gap > 1 frame |
| E2E-13 | foreground `--no-background` then `q` | alt screen entered and left; cursor visible; raw mode off |
| E2E-14 | foreground `--no-background` then `SIGINT` | clean exit; terminal restored (BUG-T1) |
| E2E-15 | SSH `RemoteForward` + `forgum remote attach` | local daemon animates from remote command |
| E2E-16 | 1×1 terminal | engine prints `cow_text` statically; no panic (BUG-E7 cousin) |
| E2E-17 | `duration:0` for 30 s | still running; killed by test |
| E2E-18 | `forgum init bash` output under `shellcheck` | 0 warnings |
| E2E-19 | `forgum-engine completions bash` vs committed file | byte-identical |
| E2E-20 | 24-bit color detection under `$COLORTERM=truecolor` | truecolor sequences emitted; under unset, 256-color |

### 4.3 The "prompt row untouched" assertion (the most important one)

```rust
fn assert_prompt_row_untouched(captured: &[u8], cols: u16, rows: u16, ob_y1: u16) {
    let stream = AnsiParser::new(captured);
    for cmd in stream {
        if let AnsiCmd::MoveTo(_, y) = cmd {
            assert!(y < ob_y1, "engine wrote to prompt row {y} (ob_y1={ob_y1})");
        }
    }
}
```
This single assertion, run on every E2E capture, is the gate that would have caught BUG-B1/B3/B4/B5. It becomes a merge blocker.

---

## 5. Tier 4 — Adversarial / fuzz / nightly

### 5.1 Fuzz targets (`cargo-fuzz`)

- `fuzz_parse_sceneconfig`: random bytes → `serde_json::from_str` must not panic; large inputs rejected (BUG-D4).
- `fuzz_cow_expansion`: random `.cow` content + random eyes/tongue → no panic, no infinite loop.
- `fuzz_effect_render`: random `cols/rows ∈ [1, 400]`, random effect, random dt ∈ [0, 1] → `effect.update`/`render` never panics, never indexes OOB (BUG-E7).
- `fuzz_control_socket`: random command strings → allowlist rejects unknown; no panic.

### 5.2 Mutation testing (`cargo-mutants`)

Run nightly on `forgum-platform` and the render core. A surviving mutant = a missing test. Target: < 5 % survivors on the §4 invariants.

### 5.3 Forensic integrity (from `.agents/orchestrator/plan.md`)

- **No hardcoded test results:** CI runs against freshly-built binaries, not cached outputs. A `memcmp` of captured ANSI against a golden file is **forbidden** for logic tests (only allowed for pixel-exact visual regression, which is opt-in).
- **Adversarial tier:** a red-team pass before each minor release tries to corrupt the terminal (resize storms, rapid `kill -9`, SIGFPE injection via a debug build) and asserts the shell remains usable.

---

## 6. CI matrix (the gate)

```yaml
# .github/workflows/ci.yml  (sketch)
jobs:
  rust-unit:
    strategy: { matrix: { os: [ubuntu-22.04, macos-14, windows-latest], target: [x86_64, aarch64] } }
    steps: [toolchain, cargo test --target, cargo clippy -D warnings, cargo fmt --check]

  rust-integration:
    runs-on: ubuntu-22.04
    steps: [toolchain, cargo test --test '*' -- --include-ignored, xvfb-run cargo test --test e2e]

  pester:
    strategy: { matrix: { os: [windows-latest, ubuntu-22.04] } }
    steps: [pwsh install, Invoke-Pester -CI]

  e2e-tmux:
    runs-on: ubuntu-22.04
    steps: [toolchain, build, xvfb-run tmux new-session -d -s ci, cargo test --test e2e -- --tmux]

  cross-compile:
    strategy: { matrix: { target: [aarch64-unknown-linux-gnu, x86_64-unknown-linux-musl, aarch64-apple-darwin, aarch64-pc-windows-msvc] } }
    steps: [cargo install cross, cross build --release --target]

  gates:            # merge blockers
    - rust-unit (all green)
    - clippy + fmt clean
    - pester green (win+linux)
    - e2e-tmux green
    - completion-drift check (forgum-engine completions vs committed)
    - cfg-grep: `rg '#\[cfg' engine/src/` returns 0 hits
    - version-parity: Cargo.toml == Forgum.psd1 == manifests

  nightly:
    - cargo fuzz run fuzz_* -- -max_total_time=600
    - cargo mutants --in-place forgum-platform
    - full e2e matrix (all Tier-1 OS × shell × terminal)
```

### 6.1 The `cfg`-grep gate (architectural enforcement)

```bash
# .github/workflows/ci.yml
- name: no cfg in engine/src
  run: |
    if rg -q '#\[cfg' engine/src/; then
      echo "FAIL: engine/src contains #[cfg] — move to forgum-platform"; exit 1
    fi
```
This makes the `06-…` §2 rule machine-checked.

### 6.2 The completion-drift gate

```bash
- name: completions in sync
  run: |
    ./target/release/forgum-engine completions bash > /tmp/forgum.bash
    ./target/release/forgum-engine completions zsh  > /tmp/_forgum
    ./target/release/forgum-engine completions fish > /tmp/forgum.fish
    diff -q /tmp/forgum.bash scripts/completions/forgum.bash
    diff -q /tmp/_forgum     scripts/completions/_forgum
    diff -q /tmp/forgum.fish scripts/completions/forgum.fish
```

---

## 7. Test debt paydown (from the repo audit)

The repo's `AUDIT-2026-06-20.md` lists fixes applied but no regression tests for several. Add:

| Audit fix | Regression test |
|-----------|-----------------|
| #3 path traversal (`Read-CowFile`) | unit: `../etc/passwd`, abs path, symlink-escape all rejected |
| #10 export `--output` traversal | unit: same |
| #6 `Set-Forgum` ValidateSet | unit: every engine effect accepted; unknown rejected |
| #8 Blink fires | unit: `Blink` produces a blink within N frames |
| #15 temp-file race | integration: 100 concurrent `Set-CFConfig` → no corruption |

---

## 8. Local dev quick-start

```bash
# one-time
cargo install cargo-fuzz cargo-mutants cross
brew install tmux shellcheck fish   # or apt equivalent

# tight loop
cargo test --workspace
cargo clippy --workspace -- -D warnings
xvfb-run -a cargo test --test e2e -- --tmux

# before pushing
./scripts/check-completions.sh
shellcheck scripts/completions/forgum.bash
fish --no-execute scripts/completions/forgum.fish
```

---

**Next:** `09-MAKE-IT-COOLER.md` — the feature backlog that takes Forgum from "works" to "wow."
