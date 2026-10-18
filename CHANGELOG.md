# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.0.11-beta] - 2026-10-18

### Added
- **Exact Sub-Column & 2D Coordinate-Grid Test Oracle (`crates/engine/tests/exact_alignment_oracle.rs`)**:
  - Implemented deterministic terminal cell width oracle enforcing Unicode Standard Annex #11 (*EastAsianWidth*) and UAX #29 grapheme cluster rules (`0` for control/ZWJ/combining marks, `2` for East Asian Wide/Fullwidth/CJK/Emoji, `1` for ASCII/Box drawing).
  - Added strict 0-column wall drift assertions across speech and thought bubble borders, preventing single-column right-border misalignment.
  - Implemented Single-Ray Tether Invariant assertions verifying that thought bubble connector rows strictly emit $\le 1$ tether circle per row (`VIS-001`).
  - Added full test coverage for catalog mascots (`default`, `cat`, `dragon`, `sheep`, `elephant`, `bunny`, `turtle`) and timer border formatting.
- **Crash-Safe Emergency VT State Restoration & Panic Hook (`crates/platform/src/guards.rs`, `crates/platform/src/lib.rs`)**:
  - Implemented `EMERGENCY_VT_RESTORE_BYTES` atomic byte sequence resetting DEC 2026 synchronized updates (`\x1b[?2026l`), X11/SGR mouse tracking (`\x1b[?1000l`..`\x1b[?1015l`), bracketed paste (`\x1b[?2004l`), DECSTBM scroll margins (`\x1b[r`), alternate screen buffer (`\x1b[?1049l`), and SGR attributes + cursor (`\x1b[0m\x1b[?25h`).
  - Added `mark_vt_dirty()`, `emergency_restore_terminal_idempotent()`, and `install_crash_safe_vt_hook()`, guarding terminal states against unexpected worker thread (`sim_thread`/`render_thread`) panics and signal teardown.
- **Context-Rich Platform Error Diagnostics & Deterministic Exit Codes (`crates/platform/src/error.rs`)**:
  - Added `PlatformError::IoContext { op, path, source }` providing explicit operation and path context on file and socket failures.
  - Standardized `sysexits.h` POSIX exit codes across `PlatformError` and `EngineError` (`64` EX_USAGE, `65` EX_DATAERR, `71` EX_OSERR, `78` EX_CONFIG).

### Fixed
- **Visual & Kinematics Remediation (`crates/engine/src/runner.rs`, `crates/engine/src/engine_core.rs`, `crates/engine/src/cow.rs`)**:
  - Suppressed duplicate `$thoughts` expansion in `.cow` template processing by using `" "` when `is_thought` is active, completely resolving duplicate diagonal connector trails alongside dynamic thought tethers (`VIS-001`).
  - Enforced `str_display_width` and `char_display_width` across `display_cmd`, `duration_str`, and failure status symbol in `timer.rs`, replacing 2-column wide symbol `叉` (U+53C9) with 1-column `✗` (U+2717) to eliminate border misalignment (`SMALL-001`, `SMALL-002`).
  - Implemented OKLab perceptual lightness elevation (`elevate_oklab_lightness`, $L \ge 0.40$) in `terminal_theme.rs`, resolving dark navy `#1D3580` contrast washout against dark/acrylic terminal themes (`VIS-003`).
- **Memory Handling & Zero-Allocation Hot-Path Caps (`crates/engine/src/logger.rs`)**:
  - Capped active log files at 2MB (`MAX_LOG_SIZE = 2 * 1024 * 1024`) with automatic 2-generation rotation (`forgum.log` $\to$ `forgum.log.1`).
  - Replaced unbounded log entry accumulation in `read_recent_logs()` with a bounded `VecDeque` ring buffer, preventing memory spikes on long-running daemon sessions (`MEM-LEAK-006`).
- **Concurrent Test Synchronization (`crates/platform/src/audio_bell.rs`)**:
  - Added test lock synchronization across audio bell unit tests to eliminate concurrent cooldown atomic race conditions during parallel workspace test runs.

## [0.0.10-beta] - 2026-10-17

### Added
- **Hardened Split-Terminal Engine & Viewport Bounds Guards (`crates/platform/src/mux.rs`, `crates/engine/src/split.rs`)**:
  - Implemented dynamic DECSTBM top-margin tracking with graceful fallback when hardware margins are unsupported (`SPLIT-001`).
  - Added robust minimum terminal height threshold validation (`>= 12` lines) preventing panics in constricted viewport environments (`SPLIT-005`).
  - Implemented automatic raw-mode terminal teardown and cursor restoration on unexpected process signal exits (`SPLIT-003`).

### Fixed
- **Kinematic Particle Physics & 240Hz High-Precision Interpolation (`crates/engine/src/scenery.rs`, `crates/engine/src/battle.rs`)**:
  - Clamped particle emitter boundary trajectories to prevent sub-pixel underflows and frame overflows (`ANIM-002`, `ANIM-003`).
  - Normalized sway frequency harmonic decay across high-refresh rates up to 240Hz (`ANIM-005`).
- **TUI Dashboard Navigation & Configuration Sync (`crates/engine/src/tui_app.rs`)**:
  - Resolved focus state desynchronization and keyboard stepping bugs across numeric sliders and enum selectors (`TUI-001` through `TUI-014`).
  - Preserved color mode toggle state persistence across session re-entries without config corruption.
- **Security & Input Sanitization (`crates/engine/src/runner.rs`, `crates/engine/src/dna.rs`)**:
  - Enforced strict canonical path resolution and traversal validation when importing custom cow art assets (`SEC-002`).
  - Sanitized fortune phrase inputs against arbitrary escape sequence injections (`SEC-001`).
- **Packaging, Shell Integration & Cross-Platform Parity**:
  - Synchronized and validated all 25 packaging manifests (Gentoo, RPM, Winget, Homebrew, Scoop, Guix, Debian, WiX MSI, PKGBUILD, Chocolatey, Nix).
  - Maintained zero CRLF invariant across shell scripts (`install.sh`).

## [0.0.9-beta] - 2026-10-16

### Added
- **LSP-Like Context-Aware Shell Autocompletion (`forgum complete-context`, `crates/engine/src/completions.rs`)**:
  - Syntactically aware completion engine evaluating the full token stream, cursor index, and active subcommand/flag context.
  - Dynamically offers only acceptable, valid arguments and choices (suppresses already-used non-repeatable flags, narrows choices to active subcommands or config keys).
  - Full support for `forgum config <TAB>`, `forgum config set <TAB>`, and `forgum config set <key> <TAB>`, exposing all 36 configuration keys with types, descriptions, and enum options.
  - Shell integration scripts updated with dynamic context-aware invocation and static fallbacks for Bash, Zsh, Fish, PowerShell, and Nushell.
- **Diagnostic Error Recovery & Rich CLI Feedback (`crates/engine/src/cli.rs`)**:
  - Intercepts invalid subcommands, unknown flags, and invalid choice values with helpful diagnostics.
  - Provides fuzzy suggestion matching using Levenshtein distance (`💡 Did you mean '<candidate>'?`).
  - Enumerates all available subcommands or flags/values present under the specific failing scope.
- **Unified Config File API & Validation Methods (`crates/engine/src/config.rs`)**:
  - Implemented `get_config_value`, `set_config_value`, `validate_config`, and `supported_config_keys` covering all 36 `SceneConfig` fields.
  - Comprehensive validation with bounds checking (`fps <= 240`, `split_ratio 0.1..0.8`), fuzzy suggestions on unknown keys or invalid values, and full choice listings.
  - Refactored `forgum config get` and `forgum config set` in `runner.rs` to delegate directly to the unified API.
- **Liquid Motion Fighting Kinematics & Multi-Mascot Battles (`crates/engine/src/battle.rs`, `crates/engine/src/rps.rs`)**:
  - Implemented physical attack trajectories, dynamic hit recoil, particle impact sparks, and multi-mascot arena dynamics in `forgum battle` and `forgum rps`.

## [0.0.8-beta] - 2026-10-13

### Added
- **Phase 3 De-Monolithing & Declarative Config Reflection (`crates/platform/src/protocol.rs`, `crates/platform/src/lib.rs`)**:
  - Implemented `define_forgum_config!` macro and canonical compile-time `FORGUM_CONFIG_SCHEMA` (41 fields with categories, labels, types, and aliases) eliminating brittle field drift and runtime reflection overhead (ARCH-002, ARCH-003).
  - Added zero-allocation lookup `lookup_config_schema(query: &str) -> Option<&'static ConfigSchemaField>` for instant schema reflection across CLI, TUI, and shell completion generators.
- **Sub-3ms Cold-Start & Embedded Mascot Catalog Memoization (`crates/engine/src/dna.rs`, `crates/engine/src/runner.rs`, `crates/engine/src/engine_core.rs`)**:
  - Implemented `EMBEDDED_CATALOG` (`OnceLock<HashMap<String, CowDna>>`) with `get_embedded_catalog()`, `get_embedded_dna()`, `get_dna_fast()`, and `resolve_cow_dna(&Path, &str) -> CowDna` (PERF-003).
  - Bypasses disk I/O and JSON parsing on CLI cold starts (e.g. `forgum say "..." --banner`), dropping mascot initialization latency from ~25ms to <0.05ms while retaining runtime dynamic override when `animations.json` exists on disk.
- **Global Compute Singleton Consolidation (`crates/platform/src/compute.rs`, `crates/engine/src/gpu_compute.rs`)**:
  - Consolidated redundant compute singletons into `forgum_platform::ComputeEngine` (MEM-002).
  - Replaced duplicate `COMPUTE_ENGINE` `OnceLock` in `forgum-engine` with transparent `Deref<Target = ComputeEngine>` wrapper, reducing heap footprint to 0 bytes and unifying Vulkan and SIMD CPU compute pipelines.
- **Unified Preview & Video Capture Rasterizer (`crates/engine/src/capture_rasterizer.rs`)**:
  - Created canonical zero-allocation RGBA frame rasterizer `render_cell_to_rgba` and `render_framebuffer_to_rgba` with box-drawing glyph rendering (`─`, `│`, `┌`, `┐`, `└`, `┘`, `├`, `┤`, `┼`, `▶`, `█`, `▀`, `▄`) and sub-cell alpha blending (PERF-006, PERF-007).
  - Deduplicated font rasterization logic across `capture_mascot_previews.rs`, `tui_capture.rs`, `ffmpeg_capture.rs`, and `visual_render.rs`.
- **Universal CLI Say Subcommand & Mascot Expression Parity (`crates/engine/src/say.rs`, `crates/engine/src/runner.rs`)**:
  - Enhanced `forgum say` to gracefully distinguish between shell commands and literal speech phrases without cryptic OS error bubbles.
  - Added full support for mascot selection (`--cow / -c`), eye expressions (`--eyes / -e`), tongue shapes (`--tongue / -T`), and banner/effect pipelines in `forgum say`.
- **Dynamic Procedural Tree Species Diversity & Anti-Repetition (`crates/engine/src/scenery.rs`)**:
  - Implemented Golden Ratio Weyl sequence PRNG (`splitmix64_hash`) with maximal period $2^{64}$ and incommensurate phase offsets across 22 botanical species.
  - Strict adjacent neighbor exclusion ensuring `calculate_tree_species_with_width(n) != calculate_tree_species_with_width(n - 1)`.
  - Active viewport sliding-window quota strictly enforcing $\le 3$ instances of any species across columns $[0, \text{width})$.
  - Dynamic clearing expansion (`get_effective_tree_distances`) widening glades on ultra-wide viewports so total trees never exceed $3 \times \text{pool.len()}$.
- **Seasonal Foliage Phenology & Dynamic Celestial Arcs (`crates/engine/src/scenery.rs`, `crates/platform/src/protocol.rs`, `crates/engine/src/cli.rs`)**:
  - Botanical seasonal variations via `--season <auto|spring|summer|autumn|winter>`: cherry blossom petals in Spring, chlorophyll emerald in Summer, scarlet maples and golden birches in Autumn, bare frosted boughs and snow in Winter.
  - Solar illumination and celestial kinematics via `--time-of-day <auto|dawn|day|dusk|night>` with parabolic orbital paths for Sun and Moon transit across the sky horizon.
  - Added CLI options discovery table (`forgum options season`, `forgum options tod`) and tab completions across Bash, Zsh, Fish, PowerShell, Nushell, and Carapace.
- **True 240Hz Liquid Motion & Decoupled 120Hz Simulation (`crates/engine/src/engine_core.rs`, `crates/platform/src/signal.rs`)**:
  - Implemented `LiquidFrameGovernor` decoupling fixed 120Hz physics stepping (`FIXED_SIM_DT = 1.0 / 120.0`) from arbitrary display refresh rates (60Hz, 120Hz, 144Hz, 240Hz), completely eliminating frame-rate-dependent acceleration (ANIM-001) and simulation jitter (ANIM-002).
  - Microsecond-precision hybrid frame pacing combines `ShutdownFlag::sleep_interruptible` for large sleep intervals with a sub-millisecond spin-wait window (`SPIN_WINDOW = 1500µs`), achieving sub-0.05ms jitter at 240 FPS liquid motion.
- **Zero-Tear Synchronized Atomic Terminal Output (`crates/engine/src/renderer.rs`, `crates/platform/src/output.rs`, `crates/platform/src/terminal.rs`)**:
  - Expanded `OutputHandle` and `AnsiRenderer` scratch buffers to 64KB (`64 * 1024`).
  - Wrapped damage rendering in DEC Mode 2026 synchronized updates (`\x1b[?2026h` / `\x1b[?2026l`) to guarantee atomic frame presentation with zero visual tearing (ANIM-004).
  - Perceptual color run coalescing (`color_close`) with RGB distance threshold $\le 3$, preventing micro-glows from defeating horizontal run batching.
- **2D Boundary Flood-Fill Rig Map & Morphing Kinematics (`crates/engine/src/kinematics.rs`, `crates/engine/src/effects.rs`)**:
  - Implemented `AsciiRigMap` with 2D flood-fill boundary exploration to preserve anatomical concavities between limbs and under wings while maintaining interior opaque hull occlusion (Rule C).
  - Added `morph_ascii_glyph` for dynamic limb pivoting (`/` $\leftrightarrow$ `|` $\leftrightarrow$ `\`) and aerodynamic wing flexing (`^` $\leftrightarrow$ `~` $\leftrightarrow$ `_` $\leftrightarrow$ `v`).
  - Implemented `apply_ambient_occlusion_moat` suppressing scenery punctuation to spaces and dimming background luminance by 68% in the 1-cell halo surrounding the mascot (ANIM-005).
  - Added `compress_scenery_contrast` (`crates/engine/src/scenery.rs`) scaling background scenery luminance to 52% to guarantee $>3.5:1$ figure-ground contrast against foreground mascots.
- **Security & IPC Hardening (`crates/engine/src/init.rs`, `crates/platform/src/daemon_socket.rs`, `crates/engine/src/image_ascii.rs`, `crates/engine/src/herd.rs`)**:
  - Implemented lexical argument escaping (`shell_escape_arg`) across all 15 supported shells in `init.rs`.
  - Enforced `0o700` user-only permissions on Unix daemon sockets and runtime directories.
  - Added 25MB file size (`MAX_IMAGE_FILE_SIZE`) and 4096px dimension (`MAX_IMAGE_DIMENSION`) safety caps in `image_ascii.rs` to prevent OOM decompression bombs (SEC-005, Mandate 2.I).
  - Added `sweep_dead_daemons` in `herd.rs` to prune orphaned daemon state files and socket paths.
- **Cryptographic Release Verification & Package Manager Uninstaller Alignment (`crates/platform/src/package_manager.rs`, `crates/platform/src/uninstaller.rs`, `Public/Stop-ForgumDaemon.ps1`)**:
  - Implemented Ed25519 / Minisign cryptographic signature verification (`verify_binary_signature`, `atomic_replace_binary_verified`) with embedded release public key `FORGUM_RELEASE_PUBLIC_KEY` (SEC-004).
  - Package manager detection in `uninstaller.rs` advising native package manager uninstall commands (`brew uninstall`, `scoop uninstall`, `winget uninstall`, etc.) when managed by system repositories (SEC-007).
  - Synchronized PowerShell daemon control in `Stop-ForgumDaemon.ps1` delegating to canonical `forgum stop`.
- **Procedural Scenery Memoization Cache (`crates/engine/src/scenery.rs`, `crates/engine/src/engine_core.rs`)**:
  - Implemented `SceneryCache` and `SceneryCacheKey` (PERF-001) memoizing background framebuffer generation with 100ms time bucketing, saving ~96% of scenery recalculation load at 240Hz while keeping drifting clouds and flowing water smooth.
  - Wired into `SimState` render loop with zero-allocation cache hits and automatic cache eviction on viewport resize and mascot hot swaps.
- **Adversarial Invariant Verification Suite (`crates/engine/tests/adversarial_invariants.rs`)**:
  - Implemented `invariant_all_effects_fps_velocity_and_phase_independence` asserting velocity and phase match within 0.1% across 30 FPS vs 240 FPS for 5.0 simulated seconds.
  - Implemented `invariant_liquid_governor_240hz_p99_jitter_bound` verifying fixed 4,166,666 ns frame period and interpolation bounds across 500 frames.
  - Implemented `invariant_flood_fill_hull_and_moat_preserve_concavities` verifying hull opacities and transparent limb/wing concavities across all 132 mascots and 1-cell moat halos.
  - Implemented `invariant_uv_pinned_palette_zero_color_swimming` asserting progressive limb/wing slope morphing (`/` $\leftrightarrow$ `|` $\leftrightarrow$ `\`) and strict UV coordinate pinning across animation cycles.
- **Subcommand Unification & Health Audit (`crates/engine/src/runner.rs`)**:
  - Unified `doctor` with `checkhealth` and `diagnose`, running both the detailed system diagnostics and the Neovim-style pass/fail test suite in a single invocation.
- **Phase 5 Feature Expansion & Multi-Platform Hardware Acceleration (`crates/engine/src/fetch.rs`, `crates/engine/src/telemetry_bus.rs`, `crates/engine/src/kitty.rs`, `crates/tui/src/app.rs`, `crates/platform/src/gpu.rs`, `crates/platform/src/vulkan_metal.rs`)**:
  - Implemented `forgum fetch`: Instant and 240Hz live terminal hardware telemetry (`--instant` / `--live`) pairing mascots with 12+ hardware metrics, Braille sparklines, and terminal color dots.
  - Implemented `forgum telemetry report`: Ambient shell event bus receiving exit codes (`-x`), execution duration (`-m`), and Git dirty states to dynamically update active pasture mascots.
  - Added Kitty Graphics Protocol layered z-index escape sequences (`z=-1` background, `z=1` foreground) and native status widgets via `forgum status-line --mux <tmux|zellij>`.
  - Added Tab 6 (`Tab::Studio`) in `forgum-tui`: Interactive Mascot & Shader Studio for real-time convex hull/moat inspection, biological coat morph cycling, and 240Hz kinematics telemetry.
  - Added WSL hardware acceleration across all Linux distributions (Ubuntu, Debian, Fedora, Arch, Alpine, openSUSE, Gentoo, Void, NixOS) via `/dev/dxg`, DirectML, and Mesa Dozen D3D12 Vulkan adapters.
  - Added Apple Metal 3.1 & MoltenVK integration with MSL 3.0 compute shaders, MetalPerformanceShaders detection, and Apple Silicon M-series (M1–M4) vs Intel Mac classification.

---

## [0.0.7-beta] - 2026-10-12

### Added
- **Interactive Package Manager Selector & Configurable Defaults (`crates/platform/src/package_manager.rs`, `crates/tui/src/app.rs`)**:
  - Interactive selection prompt during `forgum update` allowing the user to choose their preferred package manager or standalone method on every update.
  - Configurable `update_method` (`auto`, `standalone`, `zypper`, `apt`, `dnf`, `pacman`, `brew`, `scoop`, `winget`, `choco`, `cargo`) and `check_updates` preferences in configuration files and TUI Tab 5.
  - Cached background update check with ANSI alert box notification on shell startup without recurring network latency.
- **Zero-Allocation Render Pipeline & Damage Reusability (`crates/engine/src/framebuffer.rs`, `crates/engine/src/engine_core.rs`)**:
  - Implemented `compute_full_damage_into(&self, out: &mut Vec<(usize, usize)>)` with scratch vector buffer reuse across simulation frames, eliminating per-frame heap allocations during damage tracking.
  - Elevated single-thread threshold to `n <= 256` in `ComputeEngine` mountain harmonic superposition and road undulation chunking, eliminating up to 1,920 unnecessary OS thread spawns per second in standard terminal widths.
  - Emitted ANSI text color reset (`\x1b[0m`) at frame conclusion to prevent foreground color leakage into host shells and prompts.

### Security
- **Hardened Standalone Upgrade Execution (`crates/platform/src/package_manager.rs`)**:
  - Replaced shell string interpolation with direct environment variable passing (`FORGUM_INSTALL_DIR`) for both PowerShell and Bash invocation, completely eliminating command injection risks.
  - Automated update cache invalidation upon successful upgrade completion.

### Packaging
- **Universal Multi-Distro & Container Validation (`.gitattributes`, packaging manifests)**:
  - Enforced strict LF line endings across all shell scripts (`packaging/**`, `*.sh`) to ensure 100% container and Alpine/Ubuntu/Arch/openSUSE compatibility.
  - Synchronized version parity across all 9 packaging channels (Homebrew, WinGet, RPM, DEB, Chocolatey, Scoop, Pacman, Gentoo, Nix).

---

## [0.0.6-beta] - 2026-10-12

### Added
- **Certified Pantone Nature Matching System (FHI / PMS) Color Palettes (`crates/platform/src/biome.rs`, `crates/engine/src/color.rs`, FEATURE-032)**:
  - Calibrated all 132 mascot definitions in `ALL_MASCOTS` and 60 subspecies coat morph variations across 21 species variation tables to authentic, certified Pantone Nature Matching System colors matching real-world biological phenotypes.
  - Fully synchronized authoritative mascot DNA profiles across `data/Cows/animations.json` and `data/animations.json`.
- **Sub-Millisecond High-Refresh Rate (120Hz / 144Hz / 240Hz) Fluid Motion Engine (`crates/engine/src/engine_core.rs`, `crates/engine/src/scenery.rs`, `crates/engine/src/effects.rs`)**:
  - Implemented Windows multimedia timer precision via `TimerResolutionGuard` (`timeBeginPeriod(1)`), dropping OS scheduling quantum from 15.6ms down to 1.0ms.
  - Calibrated hybrid spin-sleep frame pacing (sleep bulk >2.5ms, spin-loop sub-millisecond remainder) guaranteeing jitter-free frame delivery on 60Hz, 120Hz, 144Hz, and 240Hz displays.
  - Continuous floating-point parallax scrolling (`scroll_speed = 7.5_f32`, continuous `time * scroll_speed`) and wave dynamics in road textures, curb gutters, and subsurfaces, completely eliminating 350ms frame-by-frame integer stutter and freeze stalls.
  - Continuous sinusoidal wave dynamics across `BreatheEffect`, `FloatEffect`, `WalkEffect`, `PulseEffect`, and `FlyEffect`.
- **Procedural Dynamic Flora Ecosystem & Tectonic Mountain Macro-Range Envelopes (`crates/engine/src/scenery.rs`, FEATURE-031)**:
  - Geological plate tectonics modeling macro-wavelength belts ($\Lambda_{\text{macro}} \in [110.0, 140.0]$ cols) with smoothstep foothills and open valley horizon gaps.
  - Multi-tier flora maturity distribution across 22 botanical species with Fibonacci spacing and Da Vinci vertical sfumato depth gradients.
- **Mascot Art Detailing, Sanitization & Spacing Hygiene (`crates/engine/src/cow.rs`, `data/Cows/`)**:
  - Sanitized heredoc watermarks and trailing artist signature lines outside heredoc blocks across `atat.cow`, `smiling-octopus.cow`, `jellyfish.cow`, and `fence.cow`.
  - Calibrated thought bubble connector circles to provide exactly 3 cleanly spaced circles aligned to mascot eye landmarks with automatic deduplication.

### Fixed
- **OpenSUSE WSL Integer Expression Bug in Shell Hooks (`crates/platform/src/shell.rs`)**:
  - Resolved bash integer expression evaluation edge case in WSL terminal startup.
- **Mascot Disgrace Radar & Up-to-date Quote Engine**:
  - Added up-to-date quote engine and mascot telemetry validation.

---

## [0.0.5-beta] - 2026-10-12

### Added
- **High-Entropy SplitMix64 PRNG & Dynamic Incommensurate Particle Showering (`crates/engine/src/particles.rs`, `crates/platform/src/compute.rs`)**:
  - Replaced degenerate 32-bit `xorshift32` with 64-bit SplitMix64 pseudo-random generator coupled to Golden Ratio Weyl constant ($\Phi = 0x9E3779B97F4A7C15$), eliminating 0 fixed-point collapses and expanding PRNG period to $2^{64}$.
  - Implemented Box-Muller Gaussian normal distribution sampling (`rand_normal`) for realistic atmospheric diffusion and thermal plume dispersion.
  - Upgraded particle spawners (`spawn_fire`, `spawn_bubbles`, `spawn_stars`, `spawn_zzz`) to evaluate dual incommensurate frequencies governed by the Golden Ratio ($\omega_2 = \omega_1 \cdot 1.6180339887$), preventing periodic repetition.
  - Eliminated synchronized rigid glyph cycles (`(time * 10.0) as usize % len`) in favor of independent per-particle dynamic glyph selection.
  - Added continuous lateral aerodynamic micro-turbulence to `eval_particle_chunk` in `ComputeEngine`.
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
- **Elimination of Alphabetic Leg Glyphs in Gait Cycles (`crates/engine/src/kinematics.rs`, BUG-028)**:
  - Replaced alphabetic character substitutions (`'J'`, `'L'`, `'d'`, `'b'`) in Richard Williams articulated gait cycle with authentic ASCII limb strokes (`'/'`, `'\\'`, `'|'`, `')'`, `'('`).
  - Added test assertions enforcing `!ch.is_alphabetic()` across all 16 walk phases, directions, and leg partitions.
- **CP437 Mojibake Elimination (BUG-021 & BUG-023)**:
  - Procedural emitters, talk vocals, and glitch effects sanitized to 100% pure ASCII to prevent Greek Gamma and box-drawing collisions on legacy terminal decoders.
- **Small-Pane Calibration & Bubble Margin Integrity**:
  - Calibrated small-pane height threshold to 16 rows to preserve standard 80x24 terminal margins and indentation.
- **Multi-Pane & Split-Mode Alignment Calibration (BUG-027)**:
  - Logged into defect ledger for deferred comprehensive multiplexer geometry calibration.

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
