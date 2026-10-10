# Forgum Master Fix Logs & Defect Remediation History

> **Target Release:** `v0.0.4-beta`  
> **Repository:** `HKDevLoops/Forgum`  
> **Unified Keyword:** Strictly `forgum`  
> **Memory Ceiling:** Strict `< 100 MB RAM`  
> **Standard Delimiters:** `# >>> forgum >>>` ... `# <<< forgum <<<`  
> **Inventory Invariant:** Exactly 132 Physical Mascot Files  

---

## 1. Overview

This document records the complete, immutable register of all bugs, defects, regression risks, and architectural faults identified and permanently resolved in Forgum. Every entry details the defect identifier, affected components, root cause analysis, permanent resolution, verification methodology, and architectural invariant enforced.

---

## 2. Master Defect Ledger (BUG-001 through BUG-012)

### BUG-001: WSL `wt.exe` Command Resolution Failure & Subcommand Duplication
- **Severity:** Blocker / Critical
- **Component:** `crates/engine/src/runner.rs`, `crates/platform/src/terminal.rs`
- **Symptom:** Running split-shell modes under openSUSE WSL returned Windows error `0x80070002` ("The system cannot find the file specified"), and the child process received duplicate `render render` arguments.
- **Root Cause:** When running in WSL inside Windows Terminal, `WT_SESSION` leaked into the WSL environment via `WSLENV`. `detect_terminal_emulator()` recognized the host terminal as `WindowsTerminal`, causing `plan_native_split()` to spawn `wt.exe`. `wt.exe` cannot resolve native Linux paths without `wsl.exe -d <distro>`. Furthermore, `forward_args` appended arguments without preserving the binary path.
- **Permanent Resolution:**
  1. Added `forgum_platform::is_wsl()` detection inspecting `WSL_DISTRO_NAME`, `WSL_INTEROP`, and `/proc/version`.
  2. Bypassed host `wt.exe` split execution in WSL; Windows Terminal ConPTY cleanly handles hardware DECSTBM margins (`\x1b[top;bottom r`).
  3. Reconstructed `forward_args` to prepend `current_exe()` and prevent duplicate subcommand injection.
- **Verification:** Verified via `crates/engine/tests/cli_args.rs` and manual execution in WSL containers.
- **Invariant Enforced:** Mandate L (Universal Shell Integration & DECSTBM margins).

---

### BUG-002: Mascot Catalog Invariant Mismatch (132 vs 133 Counts)
- **Severity:** High
- **Component:** `crates/engine/src/options_table.rs`, `crates/engine/tests/adversarial_invariants.rs`
- **Symptom:** CI runner failed on Linux, macOS, and Windows with `options_table does not announce 132 available animals`.
- **Root Cause:** Drift between the physical `.cow` inventory (132 mascots in `data/Cows/`) and completion candidates (133 items including the `random` dynamic archetype). `options_table.rs` listed `random` in the physical animal count rather than separating physical animals from selector keywords.
- **Permanent Resolution:** Hardened `options_table.rs` to announce exactly 132 physical animals. Documented `random` as a dynamic selection keyword across CLI help and completions.
- **Verification:** `cargo test -p forgum-engine --test adversarial_invariants` (`mascot_universe_132_parity_across_all_systems`).
- **Invariant Enforced:** Mandate A & Core Invariant Checklist Item 2 (Deterministic 132 Mascot Universe).

---

### BUG-003: Logger Channel Race Condition & Asynchronous Flakiness
- **Severity:** High
- **Component:** `crates/engine/src/logger.rs`, `crates/platform/tests/log_subsystem_strict.rs`
- **Symptom:** Tests `log_level_filtering` and `log_write_and_read_roundtrip` intermittently failed due to read-after-write delays from asynchronous thread channels.
- **Root Cause:** An asynchronous channel writer thread could drop or delay pending log entries before disk flush, creating race conditions during instant CLI queries (`forgum logs`, `forgum doctor`).
- **Permanent Resolution:** Implemented synchronous file appending with an atomic $O(1)$ fast-path threshold filter (`LOG_THRESHOLD.load(Ordering::Relaxed)`). Discarded entries execute in $\approx 2$ CPU cycles without heap allocation; logged entries flush immediately.
- **Verification:** `logger_atomic_filter_fast_path_and_concurrency` invariant test passing under heavy multithreaded contention.
- **Invariant Enforced:** Mandate I (<100MB RAM, zero-allocation fast-paths).

---

### BUG-004: GitHub Actions Cross-Compilation Missing `core` Crate
- **Severity:** Blocker
- **Component:** `.github/workflows/release-nightly.yml`, `rust-toolchain.toml`
- **Symptom:** `Build Nightly (aarch64-pc-windows-msvc)` and `x86_64-apple-darwin` failed with `error[E0463]: can't find crate for core`.
- **Root Cause:** `rust-toolchain.toml` pinned the local channel to `1.98.0`, but GitHub Actions workflow used `dtolnay/rust-toolchain@stable` without pinning, installing targets for `1.98.1` while `cargo` switched to `1.98.0`.
- **Permanent Resolution:** Pinned `toolchain: 1.98.0` in workflow matrices and added an explicit `rustup target add ${{ matrix.target }}` step before cross-compilation.
- **Verification:** GitHub Actions Nightly Release workflow completed with 100% success across all 8 target triples.
- **Invariant Enforced:** Mandate F (Multi-Channel Packaging Parity).

---

### BUG-005: Release Workflow Binary Flag Omission
- **Severity:** Blocker
- **Component:** `.github/workflows/release.yml`
- **Symptom:** Release build step failed to package artifacts uniformly as `forgum` on Windows and Unix targets.
- **Root Cause:** Build invocation lacked the `--bin forgum` flag, causing Cargo to attempt building all workspace targets without generating clean standalone release archives.
- **Permanent Resolution:** Added `--bin forgum` and explicit target installation to all build matrices in `.github/workflows/release.yml`.
- **Verification:** Verified release matrix builds locally and in CI.
- **Invariant Enforced:** Mandate H (Unified Keyword strictly `forgum`).

---

### BUG-006: CP437 Mojibake Glyphs (`Γÿ` & U+2600..=U+263F Collision Range)
- **Severity:** High
- **Component:** `crates/tui/src/app.rs`, `install.sh`, `install.ps1`, `data/Cows/*.cow`, `data/Cows/animations.json`
- **Symptom:** In terminals using legacy 8-bit code pages (CP437, OEM 850) or non-UTF8 shells, symbols like ☀ (U+2600), ☂ (U+2602), ☎ (U+260E), and ☺ (U+263A) decoded into literal `Γÿ` or `Γÿ╗` mojibake sequences.
- **Root Cause:** Unicode characters in the block `U+2600..=U+263F` share byte patterns with CP437 lead byte `0xE2 0x98` when decoded under Windows 1252 / CP437 instead of UTF-8.
- **Permanent Resolution:**
  1. Replaced all U+2600..=U+263F symbols with safe universal ASCII or high-range Unicode symbols across all installation scripts, TUI widgets, cow files, and `animations.json`.
  2. Implemented `zero_cp437_mojibake_symbols_invariance` in `crates/engine/tests/adversarial_invariants.rs` asserting 0 occurrences of literal `Γÿ` or `0x2600..=0x263F` in all 132 mascot files and `animations.json`.
- **Verification:** `cargo test -p forgum-engine --test adversarial_invariants` passed cleanly.
- **Invariant Enforced:** Invariant 8 (Zero CP437 Mojibake Glyphs Invariance).

---

### BUG-007: Thought Bubble Landmark Tracking & Anchor Decoupling
- **Severity:** Medium
- **Component:** `crates/engine/src/runner.rs`, `crates/engine/src/kinematics.rs`
- **Symptom:** When mascots walked or bobbed across the terminal, thought bubbles (`forgum think`) either drifted away from the creature's mouth or clipped through the creature's back.
- **Root Cause:** Stagnant landmark offsets computed at frame 0 failed to track creature displacement vectors across kinematics passes.
- **Permanent Resolution:** Integrated `CartoonRig` dynamic landmark tracking:
  1. Mouth and head landmarks are evaluated with anatomical zone segmentation.
  2. Speech and thought bubbles are anchored with spring-damper inertia (`bubble_offset_x`, `bubble_offset_y`), producing natural buoyant floating without breaking alignment.
- **Verification:** Verified via `crates/engine/tests/cartoon_kinematics.rs` (`test_spring_damper_bubble_physics_settling`).
- **Invariant Enforced:** Mandate B (True Physical Animation) & Mandate A (Single Source of Truth for Mascot DNA).

---

### BUG-008: Viewport Clipping on Large Mascots & Multi-Segment Deformations
- **Severity:** High
- **Component:** `crates/engine/src/kinematics.rs`, `crates/engine/src/effects.rs`
- **Symptom:** Large mascots (such as `whale`, `dragon`, `elephant`, `stegosaurus`) clipped or panicked when traversing narrow viewports, and multi-segment deformations sheared ASCII characters.
- **Root Cause:** Kinematic position clamping was applied after rendering rather than before deformation, and deformation offsets were unbounded.
- **Permanent Resolution:**
  1. Bounded deformations strictly to integer terminal cells (`dx \in [-2, 2]`, `dy \in [-1, 1]`).
  2. Implemented dynamic viewport shrinkage clamping across all bounds modes (`BoundsMode::Wrap`, `BoundsMode::Bounce`, `BoundsMode::Clamp`, `BoundsMode::Lissajous`).
- **Verification:** `crates/engine/tests/cartoon_kinematics.rs` (`test_kinematic_body_bounds_modes`).
- **Invariant Enforced:** Mandate I (<100MB RAM, zero heap allocations).

---

### BUG-009: `--speech` CLI Flag Conflict with `--think`
- **Severity:** Medium
- **Component:** `crates/engine/src/cli.rs`, `crates/engine/src/runner.rs`
- **Symptom:** When `--speech` (or `-S`) was passed alongside `--think` or under the `think` subcommand, the engine defaulted to thought bubbles (`( )`) instead of speech bubbles (`< >`).
- **Root Cause:** CLI argument parser prioritized the `think` boolean flag over explicit speech overrides.
- **Permanent Resolution:** Updated `build_scene_config` so `--speech` and `-S` explicitly override `think = false`, guaranteeing speech bubble rendering (`< hello >`).
- **Verification:** Added 4 comprehensive CLI tests in `crates/engine/tests/cli_args.rs`.
- **Invariant Enforced:** CLI Argument Precedence & Invariance.

---

### BUG-010: Biological Habitat & Locomotion Kinematics Taxonomy
- **Severity:** High
- **Component:** `data/Cows/animations.json`, `crates/engine/src/effects.rs`, `crates/engine/src/scenery.rs`
- **Symptom:** Aquatic mascots (`whale`, `dolphin`, `octopus`, `seahorse`, `jellyfish`, `fish`) hovered in the sky above the ocean surface or performed walking leg strides across roads.
- **Root Cause:** Mascot profiles lacked biological habitat taxonomy and allowed all effects unconditionally.
- **Permanent Resolution:**
  1. Added `habitat` (`aquatic`, `terrestrial`, `aerial`, `amphibious`) and `allowed_effects` taxonomy across all 132 mascot profiles in `data/Cows/animations.json`.
  2. Removed `walk` from aquatic mascots; remapped aquatic locomotion to `float`, `swim`, or `drift` with bubble particle pools.
  3. Aquatic mascots render submerged below ocean water horizons rather than floating in the sky.
- **Verification:** Verified via `cargo test -p forgum-engine` and visual preview.
- **Invariant Enforced:** Mandate M (Habitat & Locomotion Kinematics Taxonomy).

---

### BUG-011: Terminal Contrast Washout on Transparent / Acrylic Terminals
- **Severity:** Medium
- **Component:** `crates/platform/src/terminal_theme.rs`, `crates/tui/src/app.rs`
- **Symptom:** On terminals with acrylic glass, blur, or transparency (Windows Terminal Acrylic, Alacritty opacity, Kitty background_opacity, macOS Terminal blur), mascots and text washed out against dark or bright desktop wallpapers.
- **Root Cause:** Standard ANSI and pastel palettes lacked minimum luminance flooring and bold styling compensation for translucent surfaces.
- **Permanent Resolution:**
  1. Added `ContrastMode` engine (`Standard`, `AcrylicGlass`, `HighContrast`, `Vibrant`).
  2. Implemented `elevate_luminance()` boosting minimum luminance to $\ge 0.55$, increasing chromatic saturation by 25%, and applying bold text styling in `AcrylicGlass` mode.
  3. Integrated contrast mode selection into the TUI settings dashboard.
- **Verification:** Unit tests in `crates/platform/src/terminal_theme.rs` (`test_acrylic_glass_luminance_elevation_and_bold`).
- **Invariant Enforced:** Mandate O & Cross-Terminal Usability.

---

### BUG-012: TUI Modal Lock-in & Keybinding Rigidity
- **Severity:** Medium
- **Component:** `crates/tui/src/app.rs`, `crates/tui/src/keybindings.rs`
- **Symptom:** TUI required switching between modal states (`NORMAL`, `INSERT`, `SEARCH`), confusing users accustomed to modeless tools like LazyGit and Zellij. Keybindings were hardcoded and could not be remapped in `config.toml`.
- **Root Cause:** Legacy state machine used modal flags rather than modeless spatial navigation and direct action triggers.
- **Permanent Resolution:**
  1. Converted TUI to 100% modeless operation (spatial `h`/`j`/`k`/`l`, `gg`, `G`, `Ctrl+d`, `Ctrl+u`, `gt`, `gT`, `ZZ`, `ZQ`, `?`).
  2. Added `[keybindings]` table in `config.toml` via `KeybindingsConfig`, allowing user-definable custom shortcut remapping.
- **Verification:** 10 comprehensive unit tests in `crates/tui/src/app.rs`.
- **Invariant Enforced:** Mandate Q (TUI Modeless Direct Navigation Standard).

---

## 3. Verification & Quality Gate Record

Every fix in this ledger has been validated against the following automated gates:

```powershell
# 1. Formatting
cargo fmt --check                                              # 🟢 PASSED

# 2. Workspace Linting
cargo clippy --workspace --all-targets -- -D warnings          # 🟢 PASSED (0 warnings)

# 3. Adversarial Invariants & Memory Ceiling
cargo test -p forgum-engine --test adversarial_invariants      # 🟢 PASSED (8/8)
cargo test -p forgum-engine --test cartoon_kinematics          # 🟢 PASSED (5/5)
cargo test -p forgum-engine --test cli_args                    # 🟢 PASSED (59/59)

# 4. Complete Workspace Tests
cargo test --workspace                                         # 🟢 PASSED (100%)
```
