# Phase 5 — tmux MVP Implementation Plan

## Tasks

### Task 1: Platform `mux.rs` — Multiplexer Detection
**File:** `crates/platform/src/mux.rs`
**What:** Add `Mux` enum and `detect_mux()` function.
**Tests:** Unit tests with mock env vars (set/remove in unsafe blocks).
**Exports:** `Mux`, `detect_mux` from `lib.rs`.

### Task 2: CLI — Add `Tmux` and `StatusLine` subcommands
**File:** `crates/engine/src/cli.rs`
**What:** Add `Commands::Tmux { sub: TmuxSub }` and `Commands::StatusLine { max_len: u16 }`.
- `TmuxSub::Install` — prints tmux config block
- `StatusLine { max_len }` — one-shot status line render

### Task 3: Tmux Config Generator
**File:** `crates/engine/src/init.rs` (extend)
**What:** Add `generate_tmux_config(engine_path: &str) -> String` that produces the tmux config block.

### Task 4: Status Line Renderer
**File:** `crates/engine/src/status_line.rs` (new)
**What:** `render_status_line(max_len: usize) -> String` — picks random fortune, loads cow, renders single-line ANSI output, truncates to max_len.

### Task 5: Main Dispatch
**File:** `crates/engine/src/main.rs`
**What:** Match on `Commands::Tmux` and `Commands::StatusLine`.

### Task 6: Export mux from platform
**File:** `crates/platform/src/lib.rs`
**What:** `pub mod mux;` + re-export `detect_mux` and `Mux`.

### Task 7: Tests
- Unit tests in `mux.rs` (mock env)
- Unit tests in `status_line.rs` (deterministic with seeded random)
- CLI parse tests in `cli.rs`
- Integration: `forgum-engine tmux install` output matches expected block
- All existing tests pass
- clippy + fmt clean
