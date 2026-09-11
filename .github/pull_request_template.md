## 📋 Pull Request Summary

### 🎯 Objective & Description
<!-- Provide a concise description of the changes made and the motivation behind them. -->

### 🏷️ Change Classification
- [ ] 🐛 **Bug Fix** (non-breaking fix for unexpected behavior)
- [ ] ✨ **Feature / Enhancement** (new capability, scenery, creature, or theme)
- [ ] ⚡ **Performance / Memory Optimization** (zero-alloc refinement, dirty-damage optimization)
- [ ] 🐚 **Shell Integration / Completions** (hook updates, cross-shell portability)
- [ ] 📦 **Packaging / Distribution** (Homebrew, Winget, Scoop, APT, DNF, Pacman, Choco, Nix)
- [ ] 📖 **Documentation** (modest guides, nature mathematics formulas, API map)
- [ ] 🧪 **Testing & Quality Assurance** (new test suites, invariant checks, stress tests)

---

## 🛡️ Crucial Standards Verification (Mandatory Gate)

Every PR must comply with the official Forgum Operating Guidelines (`AGENTS.md`) and architectural mandates:

- [ ] **Rule A (Single Source of Truth for Mascot DNA)**: `data/Cows/animations.json` is respected as authoritative; all referenced mascots exist and validate without catalog poisoning.
- [ ] **Rule B (True Physical Animation)**: Kinematic motion preserves Newtonian physics; stride-velocity coupling ($\omega = v/\lambda$) stops leg cycling when velocity is zero; airborne creatures follow dual-harmonic swoops.
- [ ] **Rule C (Convex Hull Occlusion Integrity)**: Bounding-hull occlusion (`draw_char_with_hull`) is preserved; background mountains, roads, and flora NEVER bleed through the animal's silhouette.
- [ ] **Rule D (Resilient Deserialization)**: Entry-by-entry deserialization with graceful fallbacks; no single malformed record crashes or empties the mascot catalog.
- [ ] **Rule E (TUI & Settings Synchronization)**: Clear separation between text editing and navigation keys; numeric fields (duration, FPS) validate and clamp gracefully without crashes or truncation.
- [ ] **Rule F (Multi-Channel Packaging & Zero-Vendor Lock-in)**: Install and uninstall pipelines are idempotent, non-destructive, and reversible.
- [ ] **Rule G (Universal Shell Integrity)**: Standard marker isolation (`# >>> forgum >>>` ... `# <<< forgum <<<`) maintained; both Soft uninstall (preserves user configs) and Purge uninstall verified.
- [ ] **Rule H (Unified Keyword Mandate: Strictly `forgum`)**: All crate interactions, CLI subcommands, wrappers, shell hooks, completions, and documentation use exclusively the `forgum` keyword (zero user-facing `forgum-engine`).
- [ ] **Rule I (Nature Mathematics & Strict <100MB RAM Mandate)**: Viewport-adaptive harmonic mountains, Fibonacci phyllotaxis tree spacing ($\Phi \approx 1.618$), animal-relative canopy scaling, and procedural road roughness. Resident memory stays strictly under **100MB RAM** across all concurrent render loops.
- [ ] **Zero-CFG Mandate**: Zero platform-targeting `#[cfg]` attributes in `crates/engine/src/` (all platform branching lives in `crates/platform`).

---

## 🧪 Pre-Flight Verification & Concrete Proof

Contributors must execute and attach evidence of the following local gates before opening a PR:

### 1. Cargo Test Suite (100% Passing)
```bash
cargo test --workspace
```
*Proof (paste summary):*
```text
test result: ok. 540+ passed; 0 failed; 0 ignored
```

### 2. Zero-Warning Clippy Audit
```bash
cargo clippy --workspace --all-targets -- -D warnings
```
- [ ] Clean Clippy pass with zero warnings

### 3. Rust Code Formatting
```bash
cargo fmt --check
```
- [ ] Clean rustfmt check

### 4. Memory & Nature Mathematics Budget Test
```bash
cargo test -p forgum-engine --test memory_budget
```
- [ ] Resident set size verified < 100MB RAM under parallel execution

### 5. Platform CFG Isolation Check
```bash
cargo test -p forgum-engine --test cfg_containment
```
- [ ] Zero platform `#[cfg]` in engine source confirmed

### 6. Manifest Version Parity
- [ ] All 10 package manager manifests match `Cargo.toml` version

---

## 🌐 GitHub CI/CD Pipeline & Cross-Platform Matrix

- [ ] **Linux (Ubuntu)**: Build, test, Clippy, fmt, shell hooks (Bash, Zsh, Fish) pass 100%
- [ ] **macOS (Darwin)**: Build, test, shell hooks (Bash, Zsh, Fish) pass 100%
- [ ] **Windows (Win11/Server)**: Build, test, Pester, shell hooks (PowerShell 7+, Windows PowerShell 5.1, Cmd) pass 100%
- [ ] **No Skipped or Ignored Merge-Blocking Checks**

---

## 🔍 Thorough Manual Quality Check (Reviewer & Contributor)

- [ ] **Visual Terminal Output**: Rendered mascots and procedural horizons look correct without character clipping or artifacting.
- [ ] **Signal Responsiveness**: Foreground execution terminates immediately and restores the terminal cursor cleanly upon pressing `Ctrl+C`, `'q'`, or `Esc`.
- [ ] **Scrollback Buffer Hygiene**: Overlay rendering does not pollute the shell history or prompt lines.
- [ ] **Systematic Bug Prevention**: Added an automated unit/integration test preventing future regressions for any bug addressed in this PR.
