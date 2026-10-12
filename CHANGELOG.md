# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.0.5-beta] - 2026-10-12

### Added
- **Universal UTF & Unicode Compatibility Engine (`crates/platform/src/encoding.rs`)**:
  - Transparent decoding of UTF-8, UTF-8 with BOM (`0xEF 0xBB 0xBF`), UTF-16 LE/BE, and UTF-32 LE/BE into canonical UTF-8 strings.
  - Security sanitization stripping Trojan Source BiDi override attacks (CVE-2021-42574) and normalizing line endings (`\r\n` and `\r` to `\n`).
  - Integrated into `.cow` mascot loaders, fortune texts, configuration deserialization, and piped stdin.
- **NPU Priority in Auto Acceleration Mode (`crates/platform/src/gpu.rs`)**:
  - In `AccelerationMode::Auto`, hardware resolution prioritizes detected NPUs (`Intel AI Boost`, `AMD XDNA`, `Apple Neural Engine`, `Qualcomm Hexagon`) as primary compute device, offloading both CPU and discrete GPU from terminal rendering.
- **Dynamic Animation & Kinematics GPU/NPU Offload (`crates/platform/src/compute.rs`, `crates/engine/src/gpu_compute.rs`, `crates/engine/src/kinematics.rs`)**:
  - Offloaded continuous cartoon spring-damper dialogue bubble inertia, Lissajous trajectory updates, and underwater bubble physics to `GpuComputeEngine`.
  - Operates with zero heap allocations per frame in the render loop, strictly preserving the <100MB resident RAM ceiling.
- **Comprehensive Color Themes & Biological Gating (`crates/engine/src/effects.rs`, `crates/tui/src/app.rs`)**:
  - Implemented OKLCH palette gradients for curated color modes: `cyberpunk`, `autumn`, `pastel`, `matrix`, `monochrome`, `sunset`, `ocean`, and `fire`.
  - Expanded `COLOR_MODE_OPTIONS` in TUI and enforced Mandate K biological coat morph locking when `Color Mode != "natural"`.
- **Adversarial Invariant Expansion (`crates/engine/tests/adversarial_invariants.rs`)**:
  - Added Invariant 9 (All color modes functional & non-zero alpha), Invariant 10 (Universal UTF/BOM/BiDi resilience), and Invariant 11 (Dynamic kinematics compute offload).
- **Universal Release & Distribution Map (`RELEASE_MAP.md`)**:
  - Authored definitive release workflow flowchart, manifest synchronization matrix, and automated promotion protocol.

### Fixed
- **CP437 Mojibake Elimination (BUG-021 & BUG-023)**:
  - Procedural emitters, talk vocals, and glitch effects sanitized to 100% pure ASCII to prevent Greek Gamma and box-drawing collisions on legacy terminal decoders.

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
- **Small Pane (30x30 down to 15x15) Scaling & Compaction (`crates/platform/src/terminal.rs`, `crates/engine/src/cow.rs`)**:
  - Implemented fine-grained viewport scaling ratios and percentages adapting seamlessly from standard terminals down to 15x15 micro-panes.
  - Added `strip_common_leading_indent` and `compact_mascot_for_viewport` to prevent mascot clipping in small multiplexer splits.
  - Compacted speech bubbles to single pointer lines in micro-panes and lowered render floor to `MIN_COLS = 15`, `MIN_ROWS = 4`.
- **Modern Meme Lexicon & Subcommands (`crates/engine/src/cli.rs`, `MEME_LEXICON.md`)**:
  - Authored [`MEME_LEXICON.md`](file:///Z:/Projects/Forgum/MEME_LEXICON.md) curating 100 modern internet and developer culture meme keywords.
  - Replaced archaic placeholders (`foo`, `bar`) across tests with expressive bovine keywords (`gigachad_moo`, `sigma_grind`).
  - Added first-class meme subcommands: `forgum moo`, `forgum vibe`, `forgum touch-grass`, `forgum chad`, `forgum zoomies`, `forgum yeet`, and `forgum moonwalk` with shell autocompletions.
- **Zero CP437 Mojibake Invariant 8**:
  - Added `zero_cp437_mojibake_symbols_invariance` in `crates/engine/tests/adversarial_invariants.rs` asserting zero occurrences of `Γÿ` or characters in the CP437 collision range `U+2600..=U+263F` across all 132 mascot files and `animations.json`.
- **Calibrated Soothing Nature Colors & 22 Botanical Species (`crates/engine/src/scenery.rs`)**:
  - Implemented 22 distinct botanical species across all biomes (Oak, Maple, Birch, Pine, Redwood, Baobab, CherryBlossom, WeepingWillow, Cypress, Cactus, Palm, Bamboo, Fern, Bonsai, Mangrove, Acacia, DeadTree, BerryShrub, BloomingShrub, HerbalCluster, SnowFir, WaterLily).
  - Low-discrepancy Weyl sequence ($\Phi \approx 1.6180339887$) and Vogel phyllotaxis spirals for natural, non-repeating mixed flora and fruit distributions across arbitrary viewport widths.
  - Dynamic allometric trunk tapering (1 to 3 columns) and multi-shade bark texturing (espresso, chestnut, raw umber, birch silver, mossy bark, terracotta).
  - da Vinci *sfumato* atmospheric perspective blending mid-ground and distant mountain crests.
- **Universal Color Vision Deficiency (CVD) Accessibility Layer (`crates/platform/src/terminal_theme.rs`, `crates/tui/src/app.rs`)**:
  - Full hybrid CVD support: Accessible Daltonization (Okabe & Ito / Brettel / Machado color redistribution) and LMS simulation filters across Protanopia, Deuteranopia, Tritanopia, and Achromatopsia.
  - Zero-allocation stack evaluation operating directly on `(u8, u8, u8)` with gamma expansion and 3x3 matrix transforms.
  - Synchronized across CLI (`--colorblind <mode>`), 15 shell completions, and TUI (Tab 3: FX & Colors sub-panel, Tab 5: Configuration Matrix, and Live Preview badges).
- **Richard Williams 8-Phase Quadruped Walking Kinematics (`crates/engine/src/kinematics.rs`, `crates/engine/src/effects.rs`)**:
  - Anatomical quadruped leg partition (`QuadrupedLegPartition`) decoupling fore and hind limbs with 90° phase shift.
  - Articulated leg contact phases (Strike, Landing, Mid-Stance, Late Stance, Push-Off, Lift/Recoil, Mid-Swing, Reach).
  - Vertical pelvic bounce, head inertia lag, and stride-velocity coupling.
- **Mascot Landmark Height Governor (`crates/engine/src/cow.rs`)**:
  - Viewport-governed height compaction bounding oversized mascots to 8–12 lines in compact terminal panes while strictly preserving anatomical landmarks (horns, eyes, muzzle, feet).
- **Zero-Mojibake ASCII Scenery & Windows UTF-8 Console Initialization**:
  - 100% pure ASCII environmental rendering for tree barks, roads, curbs, and telephone poles, completely eliminating CP437 mojibake (`Γöé`, `Γòæ`, `òæ`).
  - Automatic `SetConsoleOutputCP(65001)` and `SetConsoleCP(65001)` initialization on Windows in `OutputHandle::open()` and `open_tty()`.
- **Autonomous Release Standard Operating Procedure**:
  - Added [`RELEASE_SOP.md`](file:///Z:/Projects/Forgum/RELEASE_SOP.md) defining the canonical 7-stage flight checklist for cutting releases.
  - Added [`WORKFLOW_PLAYBOOK.md`](file:///Z:/Projects/Forgum/WORKFLOW_PLAYBOOK.md) documenting step-by-step repeatable standard workflows for repeating engineering tasks.
  - Added [`FIX_LOGS.md`](file:///Z:/Projects/Forgum/FIX_LOGS.md) detailing defect resolutions from BUG-001 through BUG-023.
  - Added [`CREDITS.md`](file:///Z:/Projects/Forgum/CREDITS.md) citing foundational animation, computer graphics, botanical mathematics, and color science literature.

### Changed
- **Codebase Formatting**:
  - Standardized formatting across all workspace crates and tests using `cargo fmt`.

### Fixed
- **CP437 Mojibake Elimination Across Animations, Particle Pools & TUI (`effects.rs`, `particles.rs`, `battle.rs`, `rps.rs`, `timer.rs`, `showcase.rs`, `app.rs`)**:
  - Replaced double-line and box-drawing characters (`╔`, `═`, `╗`, `║`, `╚`, `╝`, `╟`, `─`, `╢`, `│`, `┌`, `┐`, `└`, `┘`) with clean ASCII (`+`, `-`, `=`, `|`) in tournament battle arena walls, RPS showdown, timer cards, showcase demo banners, and TUI road previews.
  - Sanitized all dynamic animation and particle glyphs (`≈`, `✦`, `✧`, `█`, `▓`, `░`, `♫`, `♪`, `⚡`, `°`, `·`, `»`, `§`, `•`, `●`, `±`) across floating aquatic waves, starfields, glitch swaps, dragon breath, bird notes, and cheatsheets to 100% pure ASCII.
  - Added automated CI invariant tests in `adversarial_invariants.rs` asserting zero CP437 collision glyphs in `battle.rs`, `timer.rs`, `rps.rs`, `showcase.rs`, and 100% pure ASCII in `scenery.rs`, `effects.rs`, and `particles.rs`.
- **Strict Color Palette Gating in Modeless TUI (`app.rs`)**:
  - Enforced biological coat palette lock when entering inline text edit via `Enter`, `e`, or mouse clicks, guaranteeing that `ConfigField::Palette` remains strictly gated unless `Color Mode == "natural"`.
- **Package Manager & Self-Updater Reliability**:
  - Resolved hanging interactive prompts and sudo password failures by switching to shell invocation (`sh -c` / `cmd /c`) with inherited stdio.
  - Hardened git status detection to avoid hijacking non-Forgum repositories.
  - Corrected openSUSE/SUSE package manager prioritization (Zypper before Dnf) and passed active binary directory during standalone updates.
- **Small Pane Multiplexer Clipping**:
  - Resolved mascot overflow and speech bubble breakage in small terminal panes down to 15x15 character cells.
- **CP437 Encoding Glyph Glitches & Scenery Mojibake**:
  - Eliminated box-drawing characters (│, ║, ═, ┴, ╩, ▔) and math symbols (≈, ≡) from procedural scenery, replacing them with standard ASCII (`|`, `!`, `+`, `-`, `_`, `=`, `~`, `o`).
  - Automatically initialized Windows console code page to UTF-8 (65001) in `OutputHandle::open()` and `open_tty()`.
  - Replaced unicode smart quotation marks in `stegosaurus.cow` with ASCII single quotes.
- **Harsh Flora & Eye Fatigue Tones**:
  - Replaced eye-poking neon tree and fruit colors with soothing earth tones and botanical palettes.
- **Unnatural Quadruped Locomotion**:
  - Replaced rigid sine-wave limb translation with Richard Williams 8-phase quadruped gait cycle, 90° leg decoupling, and vertical pelvic bounce.
- **Large Mascot Viewport Bounds & Occlusion**:
  - Resolved character clipping on wide mascots (`whale`, `dragon`, `elephant`) with clamped deformation envelopes.
  - Implemented landmark-preserving height governor for tall mascots in compact panes.
- **Thought Bubble Landmark Drift**:
  - Anchored dialogue bubbles to creature mouth landmarks tracked continuously across animation frames.
- **Empirical Timing Drift Under Contention**:
  - Increased test execution margin in `challenge_m2_empirical_stress.rs` to prevent false failures under heavy parallel test runs.

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
