# Phase 6 — Herder Implementation Plan

## Tasks

### Task 1: `herd.rs` — Discovery + List
**File:** `crates/engine/src/herd.rs` (new)
**What:**
- `HerdEntry` struct (session_id, pid, alive, effect, fps, speed, paused, age, socket_path)
- `discover_daemons() -> Vec<HerdEntry>` — scan runtime_dir for daemon-*.json, read each, check alive, query STATUS
- `send_command(socket_path, cmd) -> Result<ControlResponse>` — connect to socket, send cmd, read response
- `format_table(entries) -> String` — formatted table output
- Unit tests with temp daemon state files

### Task 2: Herd Commands
**File:** `crates/engine/src/herd.rs` (extend)
**What:**
- `herd_stop(filter)` — send STOP to filtered daemons
- `herd_effect(name, filter)` — send EFFECT to filtered daemons
- `herd_speed(speed, filter)` — send SPEED to filtered daemons
- `herd_pause(filter)` / `herd_resume(filter)`
- `herd_quiet()` — send SPEED 0.1 to all

### Task 3: Theme System
**File:** `crates/engine/src/theme.rs` (new)
**What:**
- `Theme` struct (effect, cow, eyes, tongue)
- `list_themes(config_dir) -> Vec<String>`
- `load_theme(config_dir, name) -> Result<Theme>`
- `apply_theme(theme, filter)` — send EFFECT+COW to all daemons
- Unit tests

### Task 4: CLI Subcommands
**File:** `crates/engine/src/cli.rs`
**What:** Add `Commands::Herd { sub: HerdSub }` with:
- `HerdSub::List`
- `HerdSub::Stop { session, all }`
- `HerdSub::Effect { name, session, all }`
- `HerdSub::Speed { value, session, all }`
- `HerdSub::Pause { session, all }`
- `HerdSub::Resume { session, all }`
- `HerdSub::Quiet`
- `Commands::Theme { sub: ThemeSub }` with:
  - `ThemeSub::List`
  - `ThemeSub::Apply { name }`

### Task 5: Main Dispatch
**File:** `crates/engine/src/main.rs`
**What:** Match on `Commands::Herd` and `Commands::Theme`.

### Task 6: Export + Tests
**File:** `crates/engine/src/lib.rs`
**What:** `pub mod herd; pub mod theme;`
**Tests:** Unit tests for discovery, command encoding, theme loading.
