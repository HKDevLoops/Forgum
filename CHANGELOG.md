# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.0.4-beta] - 2026-10-10

### Added
- **2D Cartoon Kinematics & Rigging Engine (`crates/engine/src/kinematics.rs`)**:
  - Implemented Disney 2D cartoon animation principles: continuous multi-segment Squash & Stretch, head bobbing, caudal pendulum follow-through, and spine harmonic undulation.
  - Automatic anatomical zone detection: Cranial, Thoracic, Pedal, and Caudal kinetic regions based on mascot landmark geometry.
  - Decoupled dialogue physics: speech and thought bubbles float with elastic spring-damper inertia, lagging behind mascot movement and settling smoothly to equilibrium.
  - Comprehensive integration test suite in `crates/engine/tests/cartoon_kinematics.rs`.
- **`--speech` (`-S`) Dialogue Override**:
  - Added `--speech` CLI flag that explicitly overrides `--think` and subcommands, guaranteeing speech bubbles (`< >`) without requiring config edits.
  - Added unit and CLI builder tests in `crates/engine/tests/cli_args.rs`.
- **Zero CP437 Mojibake Invariant 8**:
  - Added `zero_cp437_mojibake_symbols_invariance` in `crates/engine/tests/adversarial_invariants.rs` asserting zero occurrences of `Γÿ` or characters in the CP437 collision range `U+2600..=U+263F` across all 132 mascot files and `animations.json`.
- **Autonomous Release Standard Operating Procedure**:
  - Added [`RELEASE_SOP.md`](file:///Z:/Projects/Forgum/RELEASE_SOP.md) defining the canonical 7-stage flight checklist for cutting releases.
  - Added [`WORKFLOW_PLAYBOOK.md`](file:///Z:/Projects/Forgum/WORKFLOW_PLAYBOOK.md) documenting step-by-step repeatable standard workflows for repeating engineering tasks.
  - Added [`FIX_LOGS.md`](file:///Z:/Projects/Forgum/FIX_LOGS.md) detailing defect resolutions from BUG-001 through BUG-012.

### Changed
- **Codebase Formatting**:
  - Standardized formatting across all workspace crates and tests using `cargo fmt`.

### Fixed
- **CP437 Encoding Glyph Glitches**:
  - Replaced legacy collision symbols (☀, ☂, ☎, ☺) with safe universal ASCII / high-range Unicode glyphs across installation scripts, cow files, and TUI widgets.
- **Large Mascot Viewport Bounds & Occlusion**:
  - Resolved character clipping on wide mascots (`whale`, `dragon`, `elephant`) with clamped deformation envelopes.
- **Thought Bubble Landmark Drift**:
  - Anchored dialogue bubbles to creature mouth landmarks tracked continuously across animation frames.

---

## [0.0.3-beta] - 2026-10-04

### Added
- **Vulkan SPIR-V & MSL Shader Distribution**:
  - Precompiled SPIR-V words and Metal Shading Language compute kernels for all 5 nature mathematics passes.
  - Runtime dynamic loader with zero-overhead fallback to SIMD CPU compute.
- **Universal NPU Hardware Acceleration**:
  - Microarchitecture taxonomy and hardware probing for Intel AI Boost, AMD XDNA (Ryzen AI), Apple Neural Engine (ANE), and Qualcomm Hexagon.
  - Routed through `--accel <auto|dedicated|integrated|npu|cpu>` CLI routing and TUI selectors.
- **TUI Modeless Direct Navigation & Remapping**:
  - Added canonical modeless keybindings (`gg`, `G`, `gt`, `gT`, `ZZ`, `ZQ`, `Ctrl+d`, `Ctrl+u`, `?`).
  - Added user-definable `[keybindings]` table in `config.toml` via `KeybindingsConfig`.

### Fixed
- **WSL Terminal Native Split Resolution**:
  - Bypassed host `wt.exe` invocation inside WSL; routed directly to hardware DECSTBM margins.
- **Logging Channel Race Conditions**:
  - Implemented synchronous file appending with atomic $O(1)$ fast-path threshold checking.

---

## [0.0.2-beta] - 2026-09-21

### Added
- **Biological Coat Polymorphism & Habitats**:
  - Added 6 cattle phenotypes, 3 tiger morphs, and biological taxonomy to mascot definitions.
  - Added `allowed_effects` gating in `data/Cows/animations.json`.
- **5 Universal Shell Attach Modes**:
  - `startup`, `prompt`, `clear`, `split`, and `pane` execution modes supported across 15 shells.
- **Liquid Glass / Acrylic Terminal Contrast Modes**:
  - Added `AcrylicGlass` contrast profile with luminance elevation for transparent terminals.

### Fixed
- **Road Texture Zero-Allocation**:
  - Replaced dynamic vectors in scenery road generation with pure harmonic mathematical stack evaluation.
- **Arch Linux PKGBUILD Version Normalization**:
  - Replaced hyphens with dots in package versioning for `makepkg` compatibility.

---

## [0.0.1-beta] - 2026-09-19

### Initial Release
- Multi-channel packaging across Scoop, Winget, Homebrew, Arch Linux, Debian, RPM, and Nix.
- True physical kinematics (`WalkEffect`, `FloatEffect`) with bounding-hull occlusion.
- Terminal resize handling and signal trapping.
- Resident memory consumption strictly under 100MB RAM.
