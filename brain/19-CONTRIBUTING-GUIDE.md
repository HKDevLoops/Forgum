# Forgum — Contributing Guide (plan)

> **The "understand it in one go" blueprint.** This is the plan for the `CONTRIBUTING.md` that ships in the Forgum repo. It is written so that a developer — from a first-time Rust contributor to a 20-year veteran — can read it once and know: how to get the project building, how the code is structured, what the rules are, how to run the tests, how to add a cow / an effect / a shell hook, how to open a PR that passes CI, and how a release happens. No lore, no "ask someone in Discord" — everything is on the page.
>
> This document specifies the **structure** of `CONTRIBUTING.md`, the **dev environment**, the **code map**, the **coding standards** (the v2 10 principles, restated for contributors), the **task guides** (add-a-cow, add-an-effect, add-a-shell-hook, add-a-CLI-flag), the **PR + commit conventions**, the **test-running guide**, the **release process**, and the **Code of Conduct summary**. It is the contributor's manual; the wiki (`18-…`) is the user's manual.
>
> Read after `06-ARCHITECTURE.md`, `10-FINE-TUNED-MASTER-PLAN.md`, and `13-TEST-COVERAGE-MATRIX.md`.

---

## 0. The contributing constitution (the one-screen summary)

1. **Read `CONTRIBUTING.md` once.** It's designed to be readable end-to-end in 15 minutes. If something isn't clear, it's a bug in this doc — open an issue.
2. **The 10 engineering principles are law.** (`10-…` §0.) Every PR is reviewed against them. The CI greps enforce the structural ones (no `#[cfg]` outside `crates/platform/`, no `event::read` in background, no `static mut`).
3. **Tests are the spec.** A PR without tests for new behavior is not merged. The 8-tier pyramid (`13-…` + `14-…` §11) defines what "tested" means.
4. **Small PRs win.** One feature, one fix, one cow. <400 lines ideal. Split if it touches >2 crates.
5. **Conventional Commits.** `feat(render): add kitty-graphics fallback`, `fix(shell): escape backslashes in JSON (BUG-S1)`. The changelog is generated from these.
6. **Be kind.** Code of Conduct (summary in §9). We're making a toy cow say funny things; keep perspective.

---

## 1. `CONTRIBUTING.md` structure (the shipped file's outline)

```
1.  Welcome (1 paragraph: you're here, great, here's the 15-minute tour)
2.  Quick start (clone, build, run the tests, see a cow) — 5 commands
3.  Repository layout (the crate graph, one paragraph + one diagram)
4.  The 10 engineering principles (restated, with links to the deep docs)
5.  Development environment (rustup toolchain, just, cargo, the dev deps)
6.  Running the tests (the 8 tiers, which to run locally vs CI)
7.  Task guides (the "how do I add a X" recipes):
    - Add a cow
    - Add an effect
    - Add a shell hook / completion
    - Add a CLI flag / subcommand
    - Add a config key + TUI toggle (the no-orphan-config dance)
    - Add an AI feature
8.  Pull request checklist (the pre-PR self-review)
9.  Commit & branch conventions (Conventional Commits, branch naming)
10. Release process (how a version ships — for maintainers)
11. Code of Conduct (summary + link to full)
12. Getting help (issues, discussions, the "good first issue" label)
```

---

## 2. Quick start (section 2 of the shipped file)

```bash
# 1. Clone
git clone https://github.com/harish2222/Forgum.git
cd Forgum

# 2. Toolchain (stable + the components the CI uses)
rustup show           # rust-toolchain.toml pins the version
cargo install just    # the task runner (optional but recommended)

# 3. Build
cargo build           # or: just build

# 4. Run the tests (fast tier first)
just test-fast        # unit + integration (~30 s)
just test-e2e         # E2E under tmux (~2 min, needs tmux installed)

# 5. See a cow
cargo run -- --cow dragon --effect rainbow --duration 3

# 6. Open the TUI
cargo run --          # launches the config menu
```

If all six work, you're set. If `just` isn't installed, the `justfile` commands map to `cargo` ones — read it; it's short.

---

## 3. Repository layout (section 3)

```
Forgum/
├── crates/
│   ├── engine/          # the Rust animation engine (sim/render/control threads)
│   ├── renderer/        # the Renderer trait + 4 backends (ANSI/SyncAnsi/Kitty/Wgpu)
│   ├── platform/        # ALL #[cfg] lives here (the single audit surface)
│   ├── cowsay/          # the .cow parser + 109-cow library + bubble renderer
│   ├── fortune/         # fortune file reader
│   ├── cli/             # clap arg parsing (the dual interface, 16-…)
│   ├── config/          # config.toml schema + migrate ladder + singleton
│   ├── tui/             # the ratatui config menu + eye-tracking cow (17-…)
│   ├── ai/              # the AiEngine trait + backends (14-…)
│   ├── shell/           # shell-hook emission + completions (bash/zsh/fish/pwsh)
│   ├── integrations/    # tmux / rmux / herdr / wezterm
│   └── forgum/          # the binary crate, wires it all together
├── cows/                # the 109 .cow files (data, not code)
├── fortunes/            # fortune data files
├── completions/         # generated completion scripts (CI-managed)
├── man/                 # manpage (CI-managed)
├── packages/            # manifest templates: winget/scoop/wix/deb/rpm/AUR/nix/brew
├── install.{sh,ps1}     # the interactive installers (15-…)
├── docs/                # doc-CI fixtures, sample configs, diagrams
├── eval/                # the AI eval-golden suites (14-… §11)
├── fuzz/                # cargo-fuzz targets
├── justfile             # task runner
├── Cargo.toml           # the workspace root (single version source)
├── rust-toolchain.toml  # pinned toolchain
├── CONTRIBUTING.md      # this file
├── README.md
├── CHANGELOG.md
└── LICENSE
```

**The one rule:** `crates/engine/src/` has **zero** `#[cfg]`. All platform conditionals live in `crates/platform/`. A CI grep (`rg '#\[cfg' crates/engine/src/`) must return 0 hits. This is principle #1 and the most common PR-rejection reason.

---

## 4. The 10 engineering principles (section 4, restated for contributors)

These are copied from `10-…` §0 and re-phrased as contributor-facing rules. Every PR self-review (§8) checks against them:

1. **Platform code is centralized.** `#[cfg]` only in `crates/platform/`. CI-enforced.
2. **RAII for terminal state.** Every `enable_raw_mode`/alt-screen/hide-cursor is a guard with `Drop`. Panics restore the terminal via `catch_unwind`.
3. **No input reads in background.** No `event::poll`/`event::read` in the daemon path. CI-enforced.
4. **Zero alloc in the hot loop.** Per-frame path allocates 0 bytes after warmup. The `dhat` test asserts it.
5. **DCL singletons, no `static mut`.** `OnceLock`/`LazyLock` only. Edition 2024 rule, adopted now.
6. **Software path is the contract.** CPU/ANSI is always correct; GPU backends are accelerators with fallback.
7. **Fixed-timestep sim, variable render.** 60 Hz sim, interpolated render. Deterministic.
8. **Every cow is unique.** The 7-axis DNA; no two cows animate identically.
9. **Tests are the spec.** Phase "done" = test gate green, not "code written."
10. **One version, everywhere.** `Cargo.toml` is the source; CI generates all package manifests from it.

---

## 5. Development environment (section 5)

- **Rust**: stable, pinned via `rust-toolchain.toml`. `rustup show` installs the right toolchain.
- **just**: the task runner. `just` with no args lists tasks. `just watch` rebuilds on save.
- **tmux**: required for the E2E test tier. `apt install tmux` / `brew install tmux`.
- **Optional, for full feature work:**
  - `cargo install cargo-deb cargo-generate-rpm cargo-wix` — for packaging (P1+).
  - `nix` (any version) — for the Nix flake tests.
  - `dhat` (rustup component) — for the zero-alloc gate.
  - a GPU + kitty/wezterm — for the kitty-graphics/wgpu backend tests.
- **AI dev (v3):** local models are *not* required to build/test the engine. The `ai` crate's tests use a `MockBackend`. To run AI evals locally, `just eval` downloads models lazily.

---

## 6. Running the tests (section 6)

The 8-tier pyramid (`13-…` + `14-…` §11). What to run when:

```bash
just test-fast       # tier 1+2: unit + integration (~30 s) — run before every commit
just test-e2e        # tier 3: E2E under tmux (~2 min) — run before pushing
just test-bench      # tier 4: latency budgets — run if touching the hot loop
just test-fuzz       # tier 5: short fuzz run (1 min) — CI runs the long version
just test-golden     # tier 6: visual regression — run if touching rendering
just test-eval       # tier 7: AI eval-golden — run if touching ai/
just test-perceptual # tier 8: VLM-judged — weekly, CI-only (needs a VLM)
just test-all        # everything that's reasonable locally (1–7)
just ci              # the exact CI command — run before opening a PR
```

**The "run `just ci` before you push" rule:** it's the single command that catches 90% of CI failures locally. It runs fast tiers + lint + the cfg-grep + the no-orphan-config check + the completion-drift check.

---

## 7. Task guides (section 7 — the recipes contributors actually need)

### 7.1 Add a cow

1. Drop `mycow.cow` into `cows/`. It must parse with `cowsay::Cow::from_str` (the parser validates width, eye positions, bubble compatibility).
2. Run `just classify-cows` — the VLM tags it with the 9-axis mood DNA, appends to `cow_dna.json`. (If no VLM installed, CI does it; locally you can `just classify-cows --heuristic` for a placeholder.)
3. Run `just test-fast` — the "all cows render" test asserts your cow renders without panic across all 4 renderer backends.
4. Run `just test-golden` — a golden frame is generated for your cow; commit it.
5. Open a PR. The changelog gets `feat(cows): add mycow`.

**The CI gate:** the "cow count matches manifest" test asserts `cows/` count == `cow_dna.json` count == the manifest count. No orphan cows.

### 7.2 Add an effect

1. Add the variant to `renderer::Effect` in `crates/renderer/src/effect.rs`.
2. Implement `Effect::apply(&self, frame: &mut Frame, ctx: &EffectCtx)` — pure function of frame + time + cow DNA.
3. Register it in the `EFFECTS` registry (`crates/renderer/src/registry.rs`).
4. Add a TUI toggle under Settings → Effects (the no-orphan-config dance, §7.5).
5. Add a golden frame for the effect on a default cow.
6. Add a bench: the effect must not exceed the per-frame budget (the `dhat` + `criterion` gate).
7. PR with `feat(renderer): add <effect> effect`.

**The rule:** effects are pure, allocation-free (use the `bumpalo` arena), and deterministic (same frame → same output). The bench gate prevents a slow effect from shipping.

### 7.3 Add a shell hook / completion

1. Add the shell to `crates/shell/src/kind.rs` (if new).
2. Implement `emit_hook(&self) -> String` and `emit_completions(&self) -> String`.
3. Add the shell to `forgum init`'s detection (`crates/shell/src/detect.rs`).
4. Add an E2E test: launch the shell in a ptm, source the hook, run a precmd, assert no errors + the sweep ran.
5. Add a completion-drift test: `forgum completions <shell>` is diffed against `completions/forgum.<ext>`; CI fails on drift.
6. PR with `feat(shell): add <shell> integration`.

### 7.4 Add a CLI flag / subcommand

1. Add the field to `Args` / `Command` in `crates/cli/src/args.rs` (clap derive).
2. Wire it to the engine in `crates/forgum/src/main.rs`.
3. Add a `forgum config set <key>` path if it's a persisted setting (the mirror invariant).
4. Update `wiki/CLI-Reference` (or CI's help-sync fails).
5. Add a unit test for the parse + an E2E for the behavior.
6. PR with `feat(cli): add --<flag>` or `feat(cli): add <subcommand> subcommand`.

### 7.5 Add a config key + TUI toggle (the no-orphan-config dance)

This is the most common multi-crate change. Steps:

1. **Schema:** add the key to `crates/config/src/schema.rs` (with type, default, validation).
2. **Migration:** if it's a new key in an existing section, bump `schema_version` and add a migration step in `crates/config/src/migrate.rs` (additive: set the default for existing configs).
3. **CLI mirror:** add a `--<key>` flag (global if broadly useful) and a `forgum config set <key> <value>` path. *(the mirror invariant, `16-…` §0 rule 7).*
4. **TUI toggle:** add a widget under the right section in `crates/tui/src/sections/<section>.rs`.
5. **No-orphan-config test:** the CI test that enumerates the schema and walks the TUI widget tree will now expect your key — if you forgot the widget, it fails here.
6. **Doc:** add the key to `wiki/Configuration` with the TUI path callout. Doc-CI validates the sample.
7. PR with `feat(config): add <key>`.

**The dance exists so that "everything configurable from the TUI" is mechanically enforced, not aspirational.** Don't skip step 5.

### 7.6 Add an AI feature (v3)

1. Define the feature in `crates/ai/src/features.rs` (the F01–F25 matrix, `14-…` §1).
2. Add the `AiEngine` method + a `MockBackend` impl (for tests) + the real backend impl.
3. Add the redaction path if it touches user data (the redactor is the only path to cloud).
4. Add an eval-golden suite entry (`eval/golden/<feature>.json`).
5. Add the TUI toggle under Settings → AI → Features.
6. Add the privacy impact to `wiki/AI-Privacy`.
7. PR with `feat(ai): add <feature> (F##)`.

**AI features ship behind a default-off toggle.** No new AI feature is on by default; the user opts in via the TUI.

---

## 8. Pull request checklist (section 8)

Before opening a PR, confirm:

- [ ] `just ci` is green locally.
- [ ] Conventional Commit messages (`feat(scope): …`, `fix(scope): …`).
- [ ] One concern per PR (split if it touches >2 crates).
- [ ] Tests added/updated for new behavior (tier 1 minimum; tier 3 if user-facing).
- [ ] If a new config key: the no-orphan-config dance (§7.5) is complete.
- [ ] If touching `crates/engine/src/`: no `#[cfg]` added (CI will reject).
- [ ] If touching rendering: a golden frame committed (`just test-golden --update`).
- [ ] If touching CLI: `wiki/CLI-Reference` updated (or help-sync CI fails).
- [ ] If touching AI: redaction considered; eval-golden updated.
- [ ] `CHANGELOG.md` entry under `[Unreleased]`.
- [ ] PR description links the issue (or explains the motivation).
- [ ] No secrets, no large binaries, no `console.log`/`dbg!` left in.

**The PR template** (in `.github/pull_request_template.md`) contains this checklist verbatim.

---

## 9. Commit & branch conventions (section 9)

- **Commits:** Conventional Commits. `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`, `ci`, `build`. Scope = crate name (`render`, `cli`, `tui`, `ai`, `shell`, `config`, `cows`, `platform`, `integrations`). Example: `fix(shell): escape backslashes in JSON hook (BUG-S1)`.
- **Branches:** `<type>/<short-desc>`, e.g. `fix/shell-json-escape`, `feat/ai-tone-modes`.
- **Squash-merge** on merge; the PR title becomes the commit message. Keep it Conventional.
- **`CHANGELOG.md`** is generated from the merged commits at release time (`just changelog`), grouped by type. Hand-edit only for the release preamble.

---

## 10. Release process (section 10 — for maintainers)

1. **Bump version** in `Cargo.toml` (workspace + the binary crate). `just bump <major|minor|patch>`.
2. **Update `CHANGELOG.md`**: `just changelog` generates from commits; write the release preamble.
3. **Tag:** `git tag v2.0.1 && git push --tags`.
4. **CI does the rest** (the release workflow):
   - builds binaries for the 6 targets (linux-gnu x86_64/aarch64, linux-musl x86_64, darwin x86_64/aarch64, windows x86_64/aarch64).
   - builds .deb, .rpm, .msi, the scoop json, the winget manifests, the brew formula, the AUR PKGBUILD.
   - publishes the crate to crates.io.
   - creates the GitHub Release with artifacts + SHA-256 table.
   - opens the package-manager PRs (winget-pkgs, homebrew-core if eligible, nixpkgs if eligible).
5. **Merge the package-manager PRs** after their CI passes (winget/brew/nixpkgs have human-review steps).
6. **Update the wiki** `_Sidebar` version tag + the Changelog page.

**The release is reproducible:** `just release-dry-run` does everything except publish, so you can inspect artifacts before tagging.

---

## 11. Code of Conduct (section 11 — summary)

Forgum follows the [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). Summary: be kind, be patient with newcomers, assume good faith, give constructive feedback, accept constructive feedback, focus on what's best for the community. Harassment of any kind is not tolerated; report to the maintainers. Enforcement is public and consistent.

We're building a program where a cow says funny things above your terminal prompt. Keep that perspective: the stakes are low, the fun is high, and the people matter more than the code.

---

## 12. Getting help (section 12)

- **Issues:** bug reports + feature requests. Use the templates. Include `forgum doctor` output for bugs.
- **Discussions:** questions, "how do I…", show-and-tell.
- **`good first issue` label:** tasks scoped for first-time contributors — usually a cow, an effect, or a doc page. Mentored.
- **`help wanted` label:** tasks that need a contributor but are bigger.
- **The wiki:** `wiki/Contributing` mirrors this file; `wiki/Architecture` + `wiki/Testing` go deeper.

If you're stuck for >30 minutes, open a draft PR with what you have + your question — early feedback is better than a polished PR that went the wrong direction.

---

## 13. The one-paragraph summary

Forgum's `CONTRIBUTING.md` is a 15-minute read that takes a contributor from clone to merged PR. It opens with a 5-command quick start, maps the 13-crate workspace (with the one rule: no `#[cfg]` outside `crates/platform/`), restates the 10 engineering principles as reviewer-facing law, lists the 8 test tiers and exactly which `just` commands to run when, and gives step-by-step **task guides** for the six common contributions — add a cow, add an effect, add a shell hook, add a CLI flag, add a config key + TUI toggle (the no-orphan-config dance), add an AI feature. A pre-PR checklist (run `just ci`, Conventional Commits, one concern per PR, tests, golden frames, doc sync) makes self-review mechanical. Conventional Commits + squash-merge generate the changelog automatically; the release process is one tag, then CI builds 6 targets + 8 package manifests + opens the winget/brew/nixpkgs PRs. A Code of Conduct summary and a "getting help" section close it. The whole document is designed so that no contributor ever needs to ask "how do I…" in chat — the answer is on the page, or it's a bug in the doc.
