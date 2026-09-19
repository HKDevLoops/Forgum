# GitHub Copilot Repository Instructions for Forgum

This repository is maintained and owned exclusively by **harish2222** ([HKDevLoops](https://github.com/HKDevLoops)).
All AI assistants, GitHub Copilot agents, and automated workflows MUST strictly follow these rules.

---

## 1. Absolute Maintainer Authority & PR Directives

- **Sole Maintainer**: `harish2222` is the sole owner and architect of Forgum. All contributions, commits, and releases are attributed to `harish2222 <harish2222@users.noreply.github.com>`.
- **Zero Unsolicited PRs or Branches**:
  - NEVER automatically create feature branches, draft pull requests, or comparison branches (such as `copilot/*` or unsolicited `fix-*` branches) without explicit instructions from the owner.
  - NEVER trigger or introduce branch divergence that creates "[Compare & pull request]" prompts on GitHub.
  - Development is consolidated directly under `dev` and synchronized with `main`. Do not create extraneous staging branches.

---

## 2. Unified Keyword Mandate (Strictly `forgum`)

- All workspace crates, CLI subcommands, wrappers, shell hooks, shell completions, packaging manifests, and documentation MUST use strictly and exclusively the `forgum` keyword.
- **NEVER** use, suggest, or output `forgum-engine` in user-facing commands, help texts, documentation, or shell hooks.
- Binary targets:
  - Binary name: `forgum` (`crates/cli/src/main.rs` and `crates/engine/src/main.rs` built as `--bin forgum`).
  - Library crate: `forgum_engine` (`crates/engine/src/lib.rs`).

---

## 3. Strict Architectural Mandates

Every piece of code generated, refactored, or reviewed must strictly conform to the following architectural laws:

### A. Memory Ceiling (< 100MB Resident RAM)
- The entire Forgum runtime across all parallel render loops, TUI sessions, procedural generation, and background daemons MUST strictly consume **under 100MB of resident RAM**.
- Hot loops in the render and simulation pipelines must be strictly zero-allocation. Operate exclusively on stack primitives, static lookups, and pre-allocated double framebuffers.

### B. Zero-CFG Containment in Engine
- `crates/engine/src/` must remain 100% platform-agnostic.
- **Zero** `#[cfg(target_os = ...)]`, `#[cfg(windows)]`, or `#[cfg(unix)]` directives are permitted in `crates/engine/src/`.
- All OS-specific logic (process isolation, terminal handles, VT mode enabling, signal handling, config paths) MUST reside in `crates/platform`.

### C. Single Source of Truth for Mascot DNA
- `data/Cows/animations.json` is the sole authoritative definition of mascot kinematics, particles, speeds, and color palettes.
- NEVER hardcode mascot kinematic overrides in `scenery.rs` or `effects.rs`.
- All mascot effects must be wrapped in `CompoundSignatureEffect` to preserve particle emitters and signature physical kinematics.

### D. True Physical Kinematics & Bounding-Hull Occlusion
- Animations must execute true 2D physical translation across terminal bounds ($\omega = v / \lambda$). Cosmetic-only character cycling is strictly forbidden.
- Occlusion integrity: `draw_char_with_hull` must be applied so that scenery, mountains, foliage, and roads NEVER bleed through a mascot's interior body.

### E. Nature Mathematics
- Procedural scenery must adhere to deterministic mathematical algorithms:
  - Mountains: Harmonic sinusoidal superposition $H(x) = H_{\text{base}} \cdot [1 + \sum A_k |\sin(2\pi k x / \lambda + \phi_k)|^\gamma]$.
  - Vegetation: Fibonacci / Golden Ratio phyllotaxis spacing ($\Phi \approx 1.6180339887$).
  - Roads: Mathematical cubic/sine spline undulation.

### F. Universal Shell Integrity (15 Shells)
- Shell hooks and completions across all 15 supported shells (Bash, Zsh, Fish, PowerShell, Pwsh, Nushell, Elvish, Xonsh, Tcsh, Ksh, Ion, Oil, Yash, Cmd) must maintain strict delimiter isolation:
  ```
  # >>> forgum >>>
  ...
  # <<< forgum <<<
  ```
- Shell startup latency must remain non-blocking (under 5ms hook evaluation).

---

## 4. Repository Cleanliness & Hygiene

- NEVER commit AI instruction files, agent prompts, conversation logs, temporary data, or build binaries into git.
- Adhere strictly to `.gitignore`.
- All documentation tables must maintain perfect column alignment and escaped vertical pipes (`\|`).
