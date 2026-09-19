<!-- ========================================================================= -->
<!-- 🛫 FORGUM FLIGHT DECK: RIGOROUS PULL REQUEST AUDIT & VERIFICATION       -->
<!-- All PRs MUST target the 'dev' branch. PRs targeting 'main' or 'nightly'  -->
<!-- will be automatically rejected by CI/CD triage guards.                  -->
<!-- ========================================================================= -->

## 🛫 Flight Clearance & Target Branch Verification

> [!IMPORTANT]
> **MANDATORY TARGET BRANCH**: This PR must target `dev`.
> 1. `dev`: Active development, integration, and contributor PRs.
> 2. `nightly`: Automated early-access branch compiled daily from validated `dev`.
> 3. `main`: Release-ready production branch recommended for public users.

- [ ] **I confirm this PR's base branch is set to `dev`** (NOT `main` or `nightly`).

---

## 📋 Pull Request Summary & Architectural Scope

### 🎯 Objective & Motivation
<!-- Provide a clear, technical summary of the problem solved or capability introduced. -->

### 🏷️ Kanban Classification Tags (Check all that apply)

#### Area / Subsystem:
- [ ] `area:engine` (kinematics, effects, renderer, occlusion, math)
- [ ] `area:tui` (ratatui/crossterm interface, interactive configurator, studio)
- [ ] `area:cli` (clap argument parsing, runner dispatch, subcommands)
- [ ] `area:platform` (process isolation, terminal detection, paths, signals)
- [ ] `area:packaging` (installers, Scoop, Homebrew, Winget, APT, DNF, Pacman, Nix)
- [ ] `area:scenery` (procedural mountains, Fibonacci trees, roads, biomes)
- [ ] `area:mascots` (ASCII art, DNA profiles, animations.json)
- [ ] `area:docs` (documentation, man pages, architectural guides)

#### Type of Change:
- [ ] `type:bug` (fix for unexpected behavior or crash)
- [ ] `type:feature` (new user-facing feature or enhancement)
- [ ] `type:performance` (zero-allocation optimization, dirty-frame pruning)
- [ ] `type:security` (dependency audit, permission isolation)
- [ ] `type:refactor` (code reorganization without behavioral modification)

#### Target Channel:
- [ ] `channel:dev` (immediate landing for contributor testing)
- [ ] `channel:nightly` (staged for early access release)
- [ ] `channel:stable` (slated for upcoming public stable tag)

---

## 🛡️ Core Operating Mandates (AGENTS.md Compliance)

Every pull request must strictly abide by the 9 architectural mandates:

- [ ] **Mandate A: Single Source of Truth for Mascot DNA**:
  `data/Cows/animations.json` remains authoritative. Creature effects wrap in `CompoundSignatureEffect`.
- [ ] **Mandate B: True Physical Animation**:
  Kinematic velocity coupling ($\omega = v/\lambda$) is maintained. No stagnant character swaps masquerading as animation.
- [ ] **Mandate C: Occlusion Integrity (No Scenery Bleed-Through)**:
  Bounding-hull occlusion (`draw_char_with_hull`) protects creature silhouettes. Horizon and foliage do not bleed into bodies.
- [ ] **Mandate D: Resilient Deserialization**:
  Malformed records fallback gracefully with warnings without invalidating the mascot catalog.
- [ ] **Mandate E: TUI & Settings Synchronization**:
  Clean separation between text input and navigation modes. Numeric settings clamp cleanly.
- [ ] **Mandate F: Multi-Channel Packaging & Zero-Vendor Lock-in**:
  Installation and uninstallation pipelines are idempotent, non-destructive, and reversible.
- [ ] **Mandate G: Universal Shell Integrity**:
  Standard marker delimiters (`# >>> forgum >>>` ... `# <<< forgum <<<`) are respected. Soft and Purge uninstallation verified.
- [ ] **Mandate H: Unified Keyword Mandate (Strictly `forgum`)**:
  Zero user-facing binary fragmentation. All commands, completions, and docs route solely through `forgum`.
- [ ] **Mandate I: Nature Mathematics & Strict <100MB RAM Ceiling**:
  Deterministic sinusoidal mountains, Fibonacci phyllotaxis tree spacing ($\Phi \approx 1.618$). Execution strictly stays under **100MB resident RAM**.
- [ ] **Zero-CFG Mandate**:
  Zero platform-targeting `#[cfg]` attributes in `crates/engine/src/`. All OS branches live in `forgum-platform`.

---

## 🧪 Concrete Local Preflight Verification Proof

Attach terminal output demonstrating that all local quality gates passed:

### 1. Workspace Test Suite (Zero Failures)
```bash
cargo test --workspace --verbose
```
```text
<!-- Paste your passing test output summary here -->
```

### 2. Zero-Warning Clippy Audit
```bash
cargo clippy --workspace --all-targets -- -D warnings
```
- [ ] Output is 100% clean with zero warnings.

### 3. Code Formatting Verification
```bash
cargo fmt --check
```
- [ ] Code passes standard rustfmt formatting.

### 4. Memory Budget (<100MB RAM)
```bash
cargo test -p forgum-engine --test memory_budget
```
- [ ] Confirmed execution consumes <100MB resident RAM.

### 5. Platform Isolation Verification
```bash
cargo test -p forgum-engine --test cfg_containment
```
- [ ] Zero platform CFG in `crates/engine/src` confirmed.

---

## 🤖 GitHub Copilot & Maintainer Review Checklist
*(To be completed by Maintainer / Reviewer during code review)*

- [ ] Base branch verified as `dev`.
- [ ] Labels and Kanban tags accurately reflect changed files.
- [ ] CI/CD pipeline succeeded across Linux, macOS, and Windows.
- [ ] No extraneous tokens or redundant workflows consumed.
- [ ] Regression test included for any bug addressed.
- [ ] Approved for merge into `dev`.
