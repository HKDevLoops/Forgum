# Forgum Agent Operating Guidelines (gent.md)

This document defines the core operational standards, architectural mandates, and execution workflows for all AI agents working on the Forgum codebase.

---

## 1. Mandatory Skill & Agent Usage Rule

1. **Activate Relevant Skills Before Implementation**:
   - Before implementing changes in specialized domains (customizations, CLI bindings, shell hooks, terminal rendering protocols), agents MUST consult and inspect the relevant skill instructions.
   - Ground all architectural decisions in the official system guidelines ( gy-customizations,  ntigravity-guide).

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

