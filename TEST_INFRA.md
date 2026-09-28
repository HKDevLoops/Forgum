# TEST_INFRA.md — Forgum E2E Test Suite Infrastructure Specification

This document establishes the architecture, test harness, oracle references, and coverage matrix for the opaque-box end-to-end (E2E) test suite of the Forgum Mascot Engine.

---

## 1. Test Architecture & Principles

### 1.1 Opaque-Box Invariant
All test suites interact with Forgum exclusively via:
1. **The Official CLI Binary (`forgum`)**: Spawned through `assert_cmd::Command::cargo_bin("forgum")` or `std::process::Command`. Tests never reference deprecated binaries (such as `forgum-engine`).
2. **Public Workspace APIs**: Exposed through `forgum_engine`, `forgum_platform`, and `forgum_tui`.
3. **No Private Struct/Method Dependencies**: Tests treat internal algorithms, memory layouts, and unexported structures as black boxes.

### 1.2 Deterministic Output Derivation (Oracle Sourcing)
Every assertion in this test suite is grounded in an authoritative source:
- **DNA Catalog & Biomes**: `data/Cows/animations.json`, `crates/platform/src/biome.rs:ALL_MASCOTS`, and `.cow` art files in `data/Cows/`.
- **Architectural Specifications**: `PROJECT.md`, `AGENTS.md`, and `brain/01-BUGS-AND-ISSUES.md`.
- **System Standards**: Cross-platform ANSI/VT100 escape codes, DECSTBM margin commands (`\x1b[top;bottom r`), and shell marker delimiters (`# >>> forgum >>>` ... `# <<< forgum <<<`).
- **Mathematical Invariants**: Closed-form harmonic mountain elevation, Golden Ratio phyllotaxis ($\Phi \approx 1.6180339887$), and real wall-clock delta-time tracking ($\Delta t$).

### 1.3 Memory & Resource Guardrails
All test suites enforce:
- **Strict Memory Ceiling**: Peak resident memory consumption must remain `< 100MB RAM` under all rendering loops and high-framerate simulations.
- **Process Isolation & Teardown**: Temp directories created via `tempfile::TempDir`, subprocesses cleanly reaped, and ANSI terminals restored via RAII guards.

---

## 2. Test Suite Structure & Organization

The E2E test suite is organized into 4 tiers located in `crates/engine/tests/`:

```
crates/engine/tests/
├── e2e_tier1_features.rs      # Tier 1: Feature Coverage (>=5 test cases per feature in isolation, Features 1..=24)
├── e2e_tier2_boundaries.rs    # Tier 2: Boundary & Corner Cases (empty inputs, 40x12, 200x50, 240 FPS, duration=0, large mascots)
├── e2e_tier3_combinations.rs  # Tier 3: Cross-Feature Combinations (split-scroll + biological palettes, high FPS + overlays, etc.)
└── e2e_tier4_scenarios.rs     # Tier 4: Real-World Application Scenarios (>=5 realistic CLI workflows across shells and terminals)
```

---

## 3. Four-Tier Coverage Inventory

### Tier 1: Feature Coverage (Isolation, >=5 Tests Per Feature)
Features 1 through 24 from `PROJECT.md § Feature Inventory` are systematically validated in isolation:

| Feature ID | Feature Name | Test Scope & Invariants | Min Tests |
|---|---|---|---|
| 1 | `DNA-132-SYNC` | 132 mascot palette reconciliation between `animations.json` and `biome.rs` | 5 |
| 2 | `DNA-RESILIENT-PARSE` | Entry-by-entry resilient parsing; fallback from `data/animations.json` to `data/Cows/animations.json` | 5 |
| 3 | `DNA-BIO-PALETTES` | 5-slot anatomical palettes, dark/light contrast adaptation, custom `--palette` precedence | 5 |
| 4 | `ENG-TIMING-INVARIANT` | Wall-clock delta-time tracking at 30, 60, 120, 240 FPS; no idle tier 10x drift | 5 |
| 5 | `ENG-PARTICLE-TIME-COUPLING` | Particle rotation, spawn rate, and physics decoupled from raw `frame_count` to wall-clock seconds | 5 |
| 6 | `ENG-DURATION-SAFETY` | Premature exit elimination (`DissolveEffect::is_done()`, non-TTY stdout guards) | 5 |
| 7 | `ENG-DYNAMIC-GEOMETRY` | Dynamic anatomy-based row/col reservation in native split and DECSTBM split-scroll for tall mascots | 5 |
| 8 | `ENG-HULL-OCCLUSION` | Interior bounding-hull occlusion in Nyan, Dissolve, Walk; zero background scenery bleed-through | 5 |
| 9 | `ENG-NATURE-MATH-MEM` | Zero-allocation closed-form nature mathematics maintaining resident RAM < 100MB | 5 |
| 10 | `ENG-BUGS-TERMINAL-SAFETY` | Verification of BUG-T1..T3, BUG-B1..B9, BUG-E1 (signals, RAII, stdin, duration=0, dirty equality) | 5 |
| 11 | `TUI-RESPONSIVE-BREAKPOINTS` | Root window Breakpoint scoping (<80x22, 80..=120x22+, >120x24) with zero clipping | 5 |
| 12 | `TUI-LABEL-LAYOUT` | Left-pane 21-character label layout collision avoidance (`auto_render_on_prompt`) | 5 |
| 13 | `TUI-CONFIG-LOCKSTEP` | Bidirectional lockstep synchronization: `split_scroll`, `split_mode`, `shell_attach_mode` | 5 |
| 14 | `TUI-PALETTE-SYNC` | Biological palette delineation from custom hex overrides; Tab 3 and Tab 5 preview sync | 5 |
| 15 | `SHELL-15-HOOKS` | Shell hook generation across 15 shells using CLI arguments rather than shell-escaped JSON | 5 |
| 16 | `SHELL-MARKER-ISOLATION` | Standard `# >>> forgum >>>` marker isolation and 29 historical marker recognition | 5 |
| 17 | `SHELL-PROMPT-LATENCY` | Zero prompt latency overhead: in-process daemon check, zero subshell forks on idle prompt | 5 |
| 18 | `SHELL-UNINSTALL-LIFECYCLE` | Soft uninstallation (preserving config) vs Purge uninstallation (clean slate) | 5 |
| 19 | `SHELL-DAEMON-BUGS` | Bounded 4MB stdin (BUG-D4), non-zero exit on malformed JSON (BUG-D5), clap file arg (BUG-D6), u64 saturating mul (BUG-D7) | 5 |
| 20 | `CI-MATRIX-DEDUPLICATION` | Downstream packaging consuming prebuilt binaries; no redundant test runs in release workflows | 5 |
| 21 | `CI-SECURITY-SHAS` | Pinning all GitHub Action dependencies to 40-character commit SHAs in workflow files | 5 |
| 22 | `CI-MSRV-ALIGNMENT` | MSRV toolchain alignment at 1.98.0 across Cargo.toml, rust-toolchain.toml, and ci.yml | 5 |
| 23 | `CI-CROSS-TARGETS` | Cross-compilation targets `armv7-unknown-linux-gnueabihf` and `riscv64gc-unknown-linux-gnu` | 5 |
| 24 | `UNIFIED-FORGUM-KEYWORD` | Strict `forgum` binary keyword enforcement; zero leakage of deprecated `forgum-engine` | 5 |
| **Total Tier 1** | | **>= 120 Isolated Feature Verification Tests** | **>=120** |

---

### Tier 2: Boundary & Corner Cases
Stress tests and boundary condition validation:
- **Terminal Dimension Boundaries**:
  - Compact constrained viewports (`40x12`, `60x18`)
  - Standard viewports (`80x24`, `100x30`)
  - Ultrawide / Large viewports (`140x45`, `200x50`)
  - Degenerate dimensions (`0x0`, `1x1`, single-row, single-col)
- **Extreme Simulation & Timing Boundaries**:
  - High refresh rates (`240 FPS`, `120 FPS`)
  - Low refresh rates (`5 FPS`, `1 FPS`)
  - Boundary durations: `duration = 0` (unbounded simulation), `duration = 1` second
  - Saturating frame counts (`u64::MAX` overflow prevention)
- **Mascot Scale & Anatomy Boundaries**:
  - Oversized mascots (`dragon`, `charizard`, `elephant`, `stegosaurus`)
  - Tiny mascots (`small`, `duck`, `bunny`)
  - Mascots with special ASCII symbols and heredoc wrappers
- **Input & Payload Boundaries**:
  - Empty text string `""` and whitespace-only text
  - Multi-line text with deep word-wrapping
  - Exactly 4MB stdin input payload boundary (BUG-D4 threshold)
  - Inputs with unpaired quotes, backslashes, escape sequences, null bytes

---

### Tier 3: Cross-Feature Combinations
Validates emergent behavior across interdependent engine subsystems:
- **Combination 1: Split-Scroll + Biological Palettes + Contrast Adaptation**
  Dynamic DECSTBM margin reservation rendering biologically verified 5-color mascots with automatic dark/light background adaptation.
- **Combination 2: High FPS (240 FPS) + Background Overlay + Particle Time Decoupling**
  Non-blocking background simulation running at 240 FPS verifying particles maintain real-world rotation speed without drifting or burning CPU.
- **Combination 3: Multiplexer Split (Tmux/Zellij) + Custom Hex Palette Overrides**
  Native multiplexer split pane allocation while overriding mascot DNA biological colors with user-supplied hex gradients.
- **Combination 4: Resilient DNA Parsing + Fallback Catalog + Corrupted Record Isolation**
  Catalog loader encountering damaged JSON records falling back gracefully to biological defaults without crashing sibling mascots.
- **Combination 5: Daemon Control IPC + Graceful Signal Handling + Terminal Restoration**
  Starting a background daemon, issuing socket control commands (`forgum control send stop`), and verifying RAII terminal guards restore terminal state cleanly.

---

### Tier 4: Real-World Application Scenarios
Full end-to-end workflows representing day-to-day user and shell environments:
- **Scenario 1: Interactive Shell Prompt Initialization & Prompt Hook Cycle**
  Executing `forgum init bash` / `forgum init pwsh`, inspecting generated hook scripts for marker isolation (`# >>> forgum >>>`), verifying in-process sweep fast-paths, and ensuring prompt latency is zero.
- **Scenario 2: Classic Pipeline Integration (`fortune | forgum say`)**
  Piping fortune text into `forgum say`, verifying proper speech-bubble generation, word-wrapping, mascot landmark positioning, and exit code 0.
- **Scenario 3: System Health Diagnostics (`forgum doctor` / `checkhealth`)**
  Executing system health audit, verifying terminal capability detection (truecolor, DECSTBM, multiplexer), and reporting actionable recommendations.
- **Scenario 4: Complete Daemon Lifecycle & Remote Control**
  Spawning background daemon (`forgum daemon start`), querying status (`forgum daemon status`), and cleanly terminating via socket IPC (`forgum control send stop`).
- **Scenario 5: Classic Cowsay Drop-in Replacement with Custom Eyes & Tongue**
  Invoking `forgum -e '@@' -T 'U' "Mascot engine operational"`, verifying eyes and tongue are accurately substituted into the mascot art.

---

## 4. Test Execution & Verification Commands

### Run Full E2E Test Suite:
```bash
cargo test --test e2e_tier1_features --test e2e_tier2_boundaries --test e2e_tier3_combinations --test e2e_tier4_scenarios
```

### Run Individual Tiers:
```bash
# Tier 1: Feature Coverage (120 tests)
cargo test --test e2e_tier1_features -- --nocapture

# Tier 2: Boundary & Corner Cases
cargo test --test e2e_tier2_boundaries -- --nocapture

# Tier 3: Cross-Feature Combinations
cargo test --test e2e_tier3_combinations -- --nocapture

# Tier 4: Real-World Scenarios
cargo test --test e2e_tier4_scenarios -- --nocapture
```

### Run Memory Ceiling Verification:
```bash
cargo test --test e2e_tier2_boundaries memory_ceiling -- --nocapture
```

### Verify Code Quality & Clippy Cleanliness:
```bash
cargo clippy --all-targets -- -D warnings
```
