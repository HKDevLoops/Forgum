# Forgum Agent Operating Guidelines (CLAUDE.md)

This document defines the core operational standards, architectural mandates, and execution workflows for all Claude and AI agents working on the Forgum codebase.

---

## 1. Mandatory Skill & Agent Usage Rule

1. **Activate Relevant Skills Before Implementation**:
   - Before implementing changes in specialized domains (customizations, CLI bindings, shell hooks, terminal rendering protocols), agents MUST consult and inspect the relevant skill instructions.
   - Ground all architectural decisions in the official system guidelines (agy-customizations, antigravity-guide).

2. **Specialized Agent Delegation**:
   - Do not perform broad exploratory auditing, data validation, and core refactoring in a single monolithic context.
   - Deploy specialized subagents mastered for targeted scopes:
      - **DNA & Data Auditor**: Dedicated to validating JSON schemas, cow art files, and animal profiles.
      - **Engine & Graphics Specialist**: Dedicated to framebuffers, kinematics, effects, and occlusion shaders.
      - **TUI & Platform Engineer**: Dedicated to crossterm/ratatui event loops, shell hooks, and cross-platform process isolation.
      - **Packaging & Multi-Channel Deployment Specialist**: Dedicated to cross-OS (Linux, macOS, Windows, BSD), cross-architecture (x86_64, aarch64, armv7, riscv64, i686), package managers (Homebrew, Scoop, Chocolatey, Winget, AUR, Debian/APT, RPM/DNF, Nix, Alpine), CI/CD releases, auto-update, and maintenance doctor.
      - **Cross-Shell & Environment Auditor**: Dedicated to syntax verification, rc paths, marker delimiters, prompt integration, and completion generation across Bash, Zsh, Fish, PowerShell, Pwsh, Nushell, Elvish, Xonsh, Tcsh, Ksh, Ion, Oil, Yash, Cmd.

---

## 2. Core Architectural Mandates

### A. Single Source of Truth for Mascot DNA
- data/Cows/animations.json is the authoritative definition of creature kinematics, particle emitters, speed, and glows.
- The CLI resolver and scenery.rs must NEVER override or discard an animal's DNA base animation.
- All creature effects must be wrapped in CompoundSignatureEffect so particle pools (fire, bubbles, stars) and glows are rendered as designed.

### B. True Physical Animation
- Effects must never be "stagnant place-holders" with cosmetic-only character swaps.
- FloatEffect must compute vertical floating bobbing on creature lines while preserving speech bubble anchors.
- WalkEffect must drive physical horizontal translation across terminal boundaries.

### C. Occlusion Integrity (No Scenery Bleed-Through)
- All creature rendering passes must apply bounding-hull occlusion (draw_char_with_hull) to draw opaque spaces inside the animal's silhouette. Scenery and road textures must never bleed through the animal's interior.

### D. Resilient Deserialization
- Never fail the entire mascot catalog due to a single invalid record. Deserialization of data files must be entry-by-entry with graceful fallback to defaults and descriptive warnings.

### E. TUI & Settings Synchronization
- The TUI must cleanly distinguish between text input modes and navigation/toggle shortcuts.
- Numeric settings (duration, FPS) must parse cleanly without crashing, truncation, or mode confusion.

### F. Multi-Channel Packaging & Zero-Vendor Lock-in
- All installation and uninstallation pipelines across every OS, package manager, and shell must follow idempotent, non-destructive, reversible standards with automated verification.

### G. Universal Shell Integrity
- Shell hooks and completions across all 15 supported shells must strictly maintain standard marker isolation (`# >>> forgum >>>` ... `# <<< forgum <<<`), never pollute shell startup latency, and support both Soft uninstallation (preserving user configs) and Purge uninstallation (clean slate).

### H. Unified Keyword Mandate (Strictly `forgum`)
- All workspace crates, CLI subcommands, wrappers, shell hooks, completions, and documentation must be accessed strictly and exclusively via the `forgum` keyword.
- No other binary name or execution keyword (such as `forgum-engine`) is accepted for user-facing interaction.
- All package manager manifests, installers, scripts, and runtime commands must route uniformly through `forgum`.

### I. Nature Mathematics & Strict <100MB RAM Mandate
- Procedural environmental scenery (mountains, trees, roads) must employ deterministic mathematical algorithms to realistically mimic natural topographies:
  - **Mountain Elevation & Peak Count**: Harmonic sinusoidal superposition $H(x) = H_{\text{base}} \cdot [1 + \sum A_k |\sin(2\pi k x / \lambda + \phi_k)|^\gamma]$ with mathematical peak count scaling by viewport width, modeling geological alpine crests and erosion valleys.
  - **Vegetation & Foliage**: Natural flora distribution governed by Fibonacci / Golden Ratio phyllotaxis spacing ($\Phi \approx 1.6180339887$) and procedural parabolic/branching crown envelopes.
  - **Roads & Terrain**: Mathematical spline undulation and procedural harmonic surface roughness.
- **Strict Memory Ceiling (< 100MB RAM)**: The entire Forgum execution across all parallel render loops and CLI operations must strictly consume under 100MB of resident RAM. All mathematical evaluations in the render pipeline must be zero-allocation, operating exclusively on stack primitives and pre-allocated framebuffers.

### J. Strict Prohibition on Autonomous Versioning & Tagging
- **Zero Autonomous Version Bumping**: AI agents are strictly forbidden from deciding version numbers, bumping `Cargo.toml` versions (e.g. creating `0.0.3-beta`, `1.0.0`, etc.), cutting releases, or creating Git tags.
- **Human Authority**: All version increments, Git release tags, release branch cutoffs, and package manager pushes are exclusively the prerogative of the human user and must occur ONLY when explicitly commanded by the user.

### K. Biological Coat Polymorphism & Color Palette Gating
- **Authentic Biological Coat Morphs**: Mascot natural colors must reflect real phenotypes observed in nature (e.g. Cattle: 6 morphs [Holstein, Charolais, Black Angus, Jersey, Normande, Grey Ox]; Tiger: 3 morphs [Bengal, White Bengal, Royal Bengal]; Koala: 1 monomorphic wild morph).
- **Strict Color Mode Linkage**: The `Color Palette` selection must remain locked (`🔒 [Locked: Select 'natural' Color Mode]`) unless `Color Mode == "natural"`.

### L. The 5 Universal Shell Integration & Execution Modes
- All shell integrations must rigorously support the 5 canonical execution patterns without crosstalk:
  1. `startup`: Fastfetch-style static ASCII banner on shell launch (`--banner --duration 0`), zero recurring prompt latency.
  2. `prompt`: Interactive prompt hook updating after each command prompt.
  3. `clear`: Single startup banner + re-rendering exclusively after `clear` or `cls` commands (never on ordinary prompts).
  4. `split`: Hardware DECSTBM top margin reservation, zero interference with shell typing or fullscreen apps.
  5. `pane`: Dedicated multiplexer split pane or background daemon.

### M. Habitat & Locomotion Kinematics Taxonomy
- Animations and locomotion must strictly conform to creature biology and habitat taxonomy:
  - Aquatic mascots (whales, dolphins, octopuses, fish, turtles) must never walk; their locomotion is `swim`, `float`, or `drift` with particle bubbles submerged beneath water horizons.
  - Aerial mascots fly/glide/hover; terrestrial mascots walk/breathe/sway.

### N. Mandatory Architecture & Codebase Map Consultation (`ARCHITECTURE_MAP.md`)
- **Inspect `ARCHITECTURE_MAP.md` First**: Before executing broad searches with `ripgrep`, code search, or exploratory find commands, all AI agents **MUST** read and consult `ARCHITECTURE_MAP.md`.
- **Graphical Mind Maps & Exact Locations**: `ARCHITECTURE_MAP.md` contains the definitive topographical map, Mermaid execution flowcharts, exact file paths, primary functions, and subsystem relationships across the entire repository.
- **Direct Navigation Over Guesswork**: Agents must use the index in `ARCHITECTURE_MAP.md` to navigate directly to the appropriate files, functions, and pipelines rather than performing speculative grep scans.

### O. Universal Hardware Acceleration & Zero Leak Mandate
- **Complete GPU Architecture Coverage (2006–2026)**: Forgum must support all unified shader architectures from 2006 through 2026 across NVIDIA (Tesla to Blackwell), AMD (TeraScale to RDNA 4), Intel (GMA X3000 to Arc Celestial), Apple Silicon (M1–M4), Broadcom VideoCore (IV–VII), ARM Mali/Immortalis, and RISC-V compute accelerators across Windows, Linux, macOS, FreeBSD, and WSL.
- **Seamless Fallback Hierarchy**: Probing and resolution (`--accel <auto|dedicated|integrated|cpu>`) must execute in <0.1ms via direct OS kernel/registry inspection with infallible fallback to native SIMD CPU compute.
- **Memory Safety & Zero Leak Invariant**: The render pipeline must strictly enforce zero heap allocations per frame, deterministic RAII cleanup, handle-count stability, and a resident memory ceiling strictly under 100MB RAM.

### P. The Ultimate Architecture: Vulkan + Metal Wrapper Mandate
- **Universal Cross-Platform Compute**: Core rendering and nature mathematics compute must target Vulkan for Windows, Linux, BSD, and Android, and native Metal / MoltenVK for Apple platforms (macOS/iOS).
- **Zero CPU Render Load**: Offload nature mathematics (mountain harmonic superposition, road undulation, foliage density, particle kinematics, convex hull occlusion) to GPU/NPU compute queues so the host CPU experiences minimal workload on both legacy rigs (integrated APUs, legacy unified shaders) and modern AI/GPU silicon.
- **Infallible Fallback**: When no GPU loader or driver is present, seamlessly fall back to multi-threaded SIMD CPU execution without crashing, dynamic memory allocation, or runtime panic.

### Q. TUI Modeless Direct Navigation Standard
- **Modeless Discipline**: The TUI dashboard operates strictly in a clean, modeless fashion (LazyGit / Zellij style). No modal lock-in, no `NORMAL`, `INSERT`, or `SEARCH` states, and no modal pill badges in the footer.
- **Canonical Shortcuts & Direct Actions**:
  - `h` / `j` / `k` / `l` & Arrow keys: Spatial navigation across lists, tables, and settings.
  - `gg`: Jump to top of active list or configuration panel.
  - `G`: Jump to bottom of active list or configuration panel.
  - `Ctrl+d` / `Ctrl+u`: Half-page scroll down / up.
  - `gt` / `gT`: Next tab / previous tab cycling.
  - `ZZ`: Save configuration changes and cleanly exit (`Action::SaveAndQuit`).
  - `ZQ`: Force quit without saving (`Action::Quit`).
  - `?`: Toggle floating keybinding cheatsheet overlay.
  - `Enter` / `e`: Direct inline edit or toggle on configuration fields.
  - `Esc`: Cancel inline edit or dismiss modal.

### R. Release, Tagging & Multi-Channel Upgrader Invariants (`docs/RELEASE_AND_UPGRADE_AUDIT_MAP.md`)
- **Inspect `docs/RELEASE_AND_UPGRADE_AUDIT_MAP.md` First**: Before cutting any release, bumping versions, updating package manifests, modifying shell scripts, or touching upgrader routines, AI agents **MUST** read and follow [`docs/RELEASE_AND_UPGRADE_AUDIT_MAP.md`](file:///Z:/Projects/Forgum/docs/RELEASE_AND_UPGRADE_AUDIT_MAP.md).
- **Zero CRLF Invariant**: All shell scripts (`install.sh`, etc.) must maintain strictly LF line endings (0 carriage returns `\r`). Standalone upgrader curl commands must pipe through `tr -d '\r'` before `bash` to protect against network CRLF pollution.
- **Dual-Branch Synchronization Mandate**: Whenever a release is authorized and committed, `origin/main` MUST be synchronized with `dev` immediately (`git push origin dev:main`) so `raw.githubusercontent.com/.../main/Cargo.toml` and `main/install.sh` never lag behind or report obsolete versions.
- **Triplet Tagging Standard**: Release tags must be pushed concurrently across all 3 variants (`vX.Y.Z-beta`, `X.Y.Z-beta`, `X.Y.Zbeta`) to ensure GitHub Actions triggers the release build (`v*`) and un-prefixed semver resolvers resolve cleanly.
- **Prebuilt Asset Verification**: Never declare a release task complete until the GitHub Actions release workflow completes and precompiled release tarballs/zips are verified on GitHub Releases, ensuring users never endure unexpected 5-minute source builds.
- **Comprehensive Manifest Parity**: Every version bump must synchronize all 25 configuration points across Cargo, Scoop, Winget, Chocolatey, Homebrew, Pacman, Debian, RPM, Nix, Gentoo, Guix, and MSI without skipping any package manager.

### S. Absolute Release Quality Gate & CI/CD Invariant
- **Strict All-Pass Approval Mandate**: A release tag is ONLY approved to release if it passes each and every test, CI, CD, and GitHub Actions workflow gate without exception.
- **Unconditional Release Halt on Any Failure**: If any single test, build target, package compilation, linter check, formatting check, or automated verification gate fails in GitHub Actions or local preflight, the release **MUST BE IMMEDIATELY STOPPED**.
- **Prohibition on Partial Releases**: AI agents and automated workflows must NEVER publish, distribute, or promote release assets or tags while any pipeline check is red or pending. All failures must be investigated, diagnosed, resolved, and re-verified until 100% of pipeline checks are green.

---

## 3. Architecture Roadmap & Completed Capabilities

The following strategic capabilities have been fully implemented, verified, and audited:

1. **Vulkan SPIR-V / MSL Shader Distribution (Completed)**:
   - Shaders distributed as pre-validated Metal Shading Language (MSL 3.0), raw Vulkan GLSL compute kernels, and precompiled binary SPIR-V bytecode headers (`spirv_words`) across all 5 nature mathematics passes.
   - Dynamic runtime loader/driver detection with zero-overhead, infallible fallback to SIMD CPU compute.

2. **Universal NPU Hardware Acceleration (Completed)**:
   - Full microarchitecture taxonomy and hardware probing for Intel AI Boost, AMD XDNA (Ryzen AI), Apple Neural Engine (ANE), and Qualcomm Hexagon (Snapdragon X Elite).
   - Integrated into `--accel <auto|dedicated|integrated|npu|cpu>` CLI routing, TUI selectors, and 15-shell autocompletions.

3. **TUI Custom Keybinding Remapping (Completed)**:
   - User-definable `[keybindings]` table in `config.toml` / `config.json` via `KeybindingsConfig` with multi-format roundtrip serialization.
   - Built-in canonical shortcuts (`gg`, `G`, `gt`, `gT`, `ZZ`, `ZQ`, `Ctrl+d`, `Ctrl+u`, `?`) with instant response and custom overrides.

4. **Future Strategic Horizons (For Next Milestone Alignment)**:
   - **Local GGUF / ONNX Neural Dialogue Generation**: Direct offline AI mascot thought generation routed through DirectML/CoreML hardware NPU queues.
   - **Dynamic Audio / Bell Chime Synchronization**: Mascot-specific auditory cues (e.g. subtle bell chimes on completion) with strict terminal silence guards.
