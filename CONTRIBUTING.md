# ╔══════════════════════════════════════════════════════════════════════════╗
# ║                   🤝  C O N T R I B U T I N G  🤝                      ║
# ║                                                                        ║
# ║   Welcome to Forgum! This guide outlines our core architectural        ║
# ║   standards, pre-flight test gates, and pull request merge protocols.  ║
# ╚══════════════════════════════════════════════════════════════════════════╝

Thank you for helping build and refine Forgum! We are committed to crafting a terminal mascot animation engine that pairs visual beauty and mathematical realism with zero-allocation performance and cross-platform reliability.

To ensure our codebase remains robust, performant, and maintainable over the long term, all contributions are held to clear architectural mandates and a rigorous two-stage verification process before merge.

---

## 🚀 Quick start & local workspace

```bash
git clone https://github.com/HKDevLoops/Forgum.git forgum
cd forgum
cargo build --workspace
cargo test  --workspace
cargo run -p forgum-engine --bin forgum -- say "moo"
```

The user-facing binary and execution keyword is accessed strictly and exclusively as `forgum`.

### Workspace architecture

| Crate | Package | Responsibility |
|---|---|---|
| `crates/engine` | `forgum-engine` (binary `forgum`) | Mascot DNA kinematics, procedural nature mathematics, framebuffers, dirty-tracking renderer, and CLI subcommands. **MUST stay 100% free of platform-targeting `#[cfg]` attributes.** |
| `crates/platform` | `forgum-platform` (library) | **ALL platform-specific branching lives here.** Shell detection, config discovery, capability probing, and process signal isolation across Linux, macOS, and Windows. |
| `crates/tui` | `forgum-tui` (feature-gated library) | Interactive Ratatui settings dashboard and Celestial installation/uninstallation wizard. Enabled via the optional `tui` feature in `forgum-engine`. |

---

## 🛡️ Crucial standards & architectural mandates (Rules A–I)

Every pull request must comply with the official Forgum Operating Guidelines (`AGENTS.md`):

### Rule A: Single Source of Truth for Mascot DNA
`data/Cows/animations.json` is the authoritative definition of creature kinematics, particle emitters, stride speed, and glowing auras. Scenery generators, effects wrappers, and CLI resolvers must never override or discard an animal's DNA base animation. All creature effects must be wrapped in `CompoundSignatureEffect` so particle pools and radial glows render as designed.

### Rule B: True Physical Animation
Effects must never be "stagnant place-holders" or superficial character swaps.
- **Stride-Velocity Coupling**: Mascot leg animation frequency is mathematically coupled to translation speed ($\omega = v / \lambda$). When stationary, walking animations freeze naturally in neutral stance.
- **Dynamic Kinematics**: Airborne creatures (dragons, owls, pterodactyls) follow dual-harmonic swooping kinematics; floating creatures maintain smooth vertical bobbing while keeping speech bubble anchors rock-steady.

### Rule C: Convex Hull Occlusion Integrity (No Scenery Bleed-Through)
All creature rendering passes must apply bounding-hull occlusion (`draw_char_with_hull`) to draw opaque blank spaces inside the mascot's silhouette. Background mountains, roads, and atmospheric particles must **never** bleed through the animal's interior torso or head. Outside cells preserve background scenery completely.

### Rule D: Resilient Deserialization (Zero Catalog Poisoning)
Never fail or empty the entire mascot catalog due to a single malformed or invalid record. Deserialization of data files must be entry-by-entry with graceful fallbacks to defaults and descriptive diagnostic warnings.

### Rule E: TUI & Settings Synchronization
The TUI dashboard must cleanly separate text editing mode from navigation and toggle hotkeys. Numeric fields (duration, FPS) must parse robustly, clamp safely to valid ranges, and handle arbitrary/fuzzed user input without crashes, truncation, or mode confusion.

### Rule F: Multi-Channel Packaging & Zero-Vendor Lock-in
All installation, update, and uninstallation pipelines across Linux, macOS, Windows, and BSD package managers (Homebrew, Winget, Scoop, APT, DNF, Pacman, Chocolatey, Nix, FreeBSD Ports) must be idempotent, non-destructive, and fully reversible.

### Rule G: Universal Shell Integrity
Shell hooks and completions across all 15 supported shells (Bash, Zsh, Fish, PowerShell, Pwsh, Cmd, Elvish, Nushell, Carapace, Xonsh, Tcsh, Ksh, Ion, Oil, Yash) must strictly maintain standard marker isolation (`# >>> forgum >>>` ... `# <<< forgum <<<`). Uninstallation must support both:
1. **Soft Uninstallation**: Removes binaries and shell hooks while preserving user configuration and custom cows.
2. **Purge Uninstallation**: Performs a clean slate removal of all configs, cache, and hooks.

### Rule H: Unified Keyword Mandate (Strictly `forgum`)
All workspace crates, CLI subcommands, shell hooks, completions, packaging manifests, and documentation must be accessed strictly and exclusively via the `forgum` keyword. No internal binary name or legacy execution keyword (such as `forgum-engine`) is accepted for user-facing interaction.

### Rule I: Nature Mathematics & Strict <100MB RAM Mandate
Procedural environmental scenery uses deterministic algorithms to realistically mimic natural topographies:
- **Mountain Elevation & Peak Count**: Harmonic sinusoidal superposition $H(x) = H_{\text{base}} \cdot [1 + \sum A_k |\sin(2\pi k x / \lambda + \phi_k)|^\gamma]$ with peak counts scaling deterministically by viewport width.
- **Vegetation & Foliage**: Flora distribution governed by Fibonacci / Golden Ratio phyllotaxis spacing ($\Phi \approx 1.6180339887$) and animal-relative canopy scaling.
- **Strict Resident Memory Ceiling (<100MB RAM)**: The entire Forgum execution across parallel render loops and CLI operations must strictly consume under **100MB of resident RAM**. Hot render paths must be zero-allocation, operating on pre-allocated framebuffers and stack primitives.

### Platform CFG Containment Mandate
`crates/engine/src/` must contain **zero** platform-targeting `#[cfg]` attributes (`#[cfg(unix)]`, `#[cfg(windows)]`, `#[cfg(target_os)]`, etc.). All operating-system branching belongs strictly in `crates/platform`. Enforced by `cargo test -p forgum-engine --test cfg_containment` and CI `cfg-grep`.

---

## 🧪 Contributor pre-flight verification & concrete proof

Before opening a pull request, contributors **must** run the local pre-flight test gate and attach verifiable proof in their PR description.

### Mandatory local verification commands

```bash
# 1. Full workspace test suite (must pass 100% with 0 failures)
cargo test --workspace

# 2. Zero-warning Clippy check
cargo clippy --workspace --all-targets -- -D warnings

# 3. Rust formatting check
cargo fmt --check

# 4. Strict <100MB RAM Mandate & Concurrent Peak Memory Test
cargo test -p forgum-engine --test memory_budget

# 5. Platform CFG Isolation Check (asserts 0 platform #[cfg] in engine source)
cargo test -p forgum-engine --test cfg_containment

# 6. Nature Mathematics & Bounding Hull Occlusion Integrity Test
cargo test -p forgum-engine --test render_correctness

# 7. Resilient Deserialization & DNA Catalog Protection Test
cargo test -p forgum-engine --test dna_completeness

# 8. Universal Shell Hooks & Uninstallation Test (all 15 shells)
cargo test -p forgum-engine --test init_shell

# 9. Multi-Shell Completion Generation Test (all 15 shells)
cargo test -p forgum-engine --test completions

# 10. TUI Resilience & Numeric Fuzzing Test
cargo test -p forgum-tui

# 11. Optional feature tests
cargo test --workspace --features forgum-engine/synchronized-update
cargo test -p forgum-platform --features forgum-platform/sixel
```

### Attaching proof to pull requests
Our pull request template at [`.github/pull_request_template.md`](.github/pull_request_template.md) requires pasting the summary output of `cargo test --workspace` and confirming each verification gate. PRs without attached test proof will not be merged.

---

## 🔀 Pull request and merge protocols (The two-stage gate)

To systematically eliminate recurring bugs and prevent regressions across platforms and environments, Forgum employs a **two-stage merge gate**:

```
 ┌────────────────────────────────────────────────────────┐
 │ Stage 1: Automated Cross-Platform CI Matrix (100% Pass) │
 └──────────────────────────┬─────────────────────────────┘
                            │
                            ▼
 ┌────────────────────────────────────────────────────────┐
 │ Stage 2: Thorough Manual Quality Assurance Review      │
 └──────────────────────────┬─────────────────────────────┘
                            │
                            ▼
 ┌────────────────────────────────────────────────────────┐
 │ Approved & Merged into main                            │
 └────────────────────────────────────────────────────────┘
```

### Stage 1: Automated cross-platform CI matrix (Merge-blocking)
The GitHub Actions CI pipeline (`.github/workflows/ci.yml`) runs on every pull request and push. All checks are strictly merge-blocking:
1. **Multi-OS Rust Matrix**: Builds, tests, Clippy, and formatting must pass 100% on **Ubuntu Linux**, **macOS**, and **Windows**.
2. **Contributor Standards Gate**: Automatically runs `memory_budget` (<100MB RAM), `cfg_containment`, `render_correctness` (Rule C occlusion & nature math), `dna_completeness` (Rule D), `init_shell`, and `completions`.
3. **Shell Hooks Integration Matrix**: Evaluates hook execution in actual shell environments (`bash`, `zsh`, `fish` on Linux/macOS; `pwsh`, `cmd` on Windows) with `continue-on-error: false`.
4. **Package Manifest Version Parity**: Enforces that version numbers in `Cargo.toml` and all 10 distribution manifests (Homebrew, Winget, Scoop, RPM, DEB, Chocolatey, Pacman, Gentoo, Nix, MSI) match identically.

### Stage 2: Thorough manual quality review
Once CI passes 100%, a maintainer conducts a manual pre-merge verification:
- [ ] **Visual Terminal Inspection**: Verify that mascots render cleanly without character clipping, ANSI sequence leakage, or torn borders.
- [ ] **Signal Responsiveness**: Confirm that foreground animations terminate within sub-millisecond timeframes (<1ms) and restore the terminal cursor cleanly on `Ctrl+C`, `'q'`, or `Esc`.
- [ ] **Scrollback Hygiene**: Verify that non-blocking overlays leave no trailing artifacts or corrupted lines in the shell prompt history.
- [ ] **Systematic Bug Prevention**: Any PR addressing a bug **must** include an automated regression test that reproduces the defect and confirms the fix.

---

## 🔄 Periodic quality assurance & systematic bug reduction

Rather than patching reactively after user issues arise, the Forgum project practices proactive, periodic stabilization:

1. **Scheduled CI Audits**: Automated workflow runs periodically test dependencies (`cargo audit`) and verify package manager repositories.
2. **Fuzz & Bounds Hardening**: CLI parsing and TUI input buffers are continuously tested against malformed input, Unicode edge cases, and unexpected terminal dimensions.
3. **Memory Leak Profiling**: Extended duration tests verify that daemon background rendering and split-scroll overlays operate without RSS creep over thousands of frames.
4. **Cross-Shell Compatibility Tracking**: Upstream changes in modern shells (Nushell, Fish, Elvish, PowerShell) are continuously tracked against our hook generation and completion logic.

---

## 📜 Commit messages & style

We follow the **Conventional Commits** specification:
- `feat(effects)`: Add dynamic bioluminescence pulse to deep-sea creatures
- `fix(render)`: Correct bounding hull occlusion on multi-line cowhorns
- `perf(scenery)`: Optimize harmonic mountain elevation with zero allocations
- `test(dna)`: Add resilient deserialization catalog test
- `docs(readme)`: Update nature mathematics equations and benchmarks

---

## 📄 License

By contributing to Forgum, you agree that your contributions will be licensed under the project's **[MIT License](LICENSE)**.

<div align="center">

``` text
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
```

*Crafted with pride, precision, and respect for terminal artistry.*

</div>
