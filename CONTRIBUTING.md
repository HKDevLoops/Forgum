# Contributing to Forgum

Thanks for helping build Forgum! This guide reflects the **actual** current workspace:
a Cargo workspace of **three crates** plus one feature-gated crate. The older
plan in `brain/19-CONTRIBUTING-GUIDE.md` described a different (fictional) layout
and is **not** authoritative — see [Note on historical docs](#note-on-historical-docs).

## Quick start

```bash
git clone <your-fork-or-this-repo> forgum
cd forgum
cargo build --workspace
cargo test  --workspace
cargo run -p forgum-engine -- say "moo"
```

The last command renders a cow saying "moo" — a quick smoke test that the
renderer, scene config, and engine binary all wire up correctly. The binary is
`forgum-engine` (not `forgum`).

## Repository layout

The workspace root `Cargo.toml` uses `resolver = "2"`, declares
`[workspace] members`, pins `[workspace.package]` to `v0.4.0`, and defines
`[workspace.lints]` (rust + clippy) that both core crates inherit with
`workspace = true`.

| Crate | Kind | Responsibility |
|-------|------|----------------|
| `crates/engine` | binary `forgum-engine` | Scene config (`SceneConfig`), the render loop, all CLI subcommands (`init`, `config`, `say`, `render`, `status`, `tmux`, `herd`, `theme`, `demo`, `remote`, `timer`, `battle`, `fortune`, …). **MUST stay free of platform-targeting `#[cfg]` attributes.** |
| `crates/platform` | library `forgum-platform` | **ALL** platform `#[cfg]` lives here. Exposes `ShellKind`, `config_path()`, and the terminal capability probe. The `sixel` feature lives here. |
| `crates/tui` | feature-gated library `forgum-tui` | A `ratatui` config menu, enabled by the engine's `tui` feature via an optional `forgum-tui` dependency. Called from engine using the `cfg!(feature = "tui")` macro, **not** a `#[cfg]` attribute. |

Engine features: only `synchronized-update` (off by default; toggled at runtime
via the `cfg!` macro in `render.rs`). Platform feature: `sixel`. The `tui`
feature gates the config TUI.

**The ONE RULE:** `crates/engine/src/` contains **no** `unix` / `windows` /
`target_os` / `target_family` branching. Ever.

## The one rule: no platform `#[cfg]` in `crates/engine/src/`

All platform-specific branching (`#[cfg(unix)]`, `#[cfg(windows)]`,
`#[cfg(target_os = "...")]`, `#[cfg(target_family = "...")]`) belongs in
**`crates/platform`**. The engine consumes platform differences through the
platform crate's API (`ShellKind`, `config_path()`, capability probe) rather
than by splitting its own source per-OS.

Feature toggles inside the engine are done with the **`cfg!(feature = "...")`**
*macro*, not a `#[cfg]` *attribute* — e.g. `render.rs` switches the
`synchronized-update` behavior at runtime with `cfg!(feature = "synchronized-update")`.
This matters because CI grep and the containment test forbid platform
**attributes** in engine `src/`, but `cfg!` macros are allowed (they evaluate to
a `bool` and don't restructure the module tree).

The same pattern applies to the TUI: the optional `forgum-tui` dependency is
pulled in under the `tui` feature, and the engine call site guards it with
`cfg!(feature = "tui")`. The TUI *implementation* stays in `crates/tui`.

### Enforcement

CI (`.github/workflows/ci.yml`) has a `cfg-grep` step that **fails the build**
if `crates/engine/src/` contains any platform `#[cfg]`. The test
`crates/engine/tests/cfg_containment.rs` walks `src` and asserts **zero**
platform `#[cfg]` attributes, so you'll catch it locally with
`cargo test --workspace` too.

If you need an OS-specific behavior, put it in `crates/platform` and expose a
plain (cfg-free) function or enum that the engine calls.

## The 10 principles

1. **Dirty-tracking renderer** — only redraw what changed.
2. **Zero-alloc where possible** — hot paths avoid allocations.
3. **Capability probe before fancy features** — check the terminal before
   using sixel/raw-mode/etc. (via `crates/platform`).
4. **Leak-proofed daemon** — the long-running / tmux-backed render loop must
   not accumulate memory.
5. **`deny_unknown_fields` config** — `SceneConfig` rejects unknown keys.
6. **Sentinel-safe merge** — `SceneConfig::merge()` / `merge()` in
   `crates/engine/src/config.rs` overrides only set fields.
7. **Hook per shell** — each shell gets its own generated hook in `init.rs`.
8. **Feature-flagged heavy deps** — heavy optional deps (TUI, sixel) are
   behind features, never default.
9. **Cross-platform paths via `config_path()`** — never hardcode a path; use
   the platform crate's `config_path()`.
10. **Tests gate features** — feature-specific tests run under their feature
    flag (see Test tiers).

## Test tiers

Run these locally before opening a PR:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check

# Feature-flagged tests
cargo test --workspace --features forgum-engine/synchronized-update
cargo test -p forgum-platform --features forgum-platform/sixel

# PowerShell/Pester integration tests
Invoke-Pester -Path Tests/

# Dependency audit
cargo audit
```

CI also checks **version parity**: the `Cargo.toml` version must match the
version stamped into the 8 packaging manifests.

## Adding a feature

Short pointers (full detail lives in `ADVANCED.md`):

- **Add a config key** → add the field in `crates/engine/src/protocol.rs`
  (`SceneConfig`, `pub`, `Default`, `deny_unknown_fields`, 11 fields today) →
  handle it in `merge()` in `crates/engine/src/config.rs` → surface it in
  `cli.rs` (`config set <key> <value>`, `config --tui`) → optionally add a TUI
  widget in `crates/tui` → add a test (incl. round-trip via
  `read_config_file`).
- **Add a render/effect** → implement it in `crates/engine/src/effects.rs`
  (respecting the capability probe + `synchronized-update` toggle).
- **Add a shell hook** → extend the `Shell` enum in `crates/engine/src/init.rs`
  (Bash/Zsh/Fish/Pwsh/Cmd/PowerShell, with `Shell::parse`) and its
  `generate_hook`. Note: this is the engine's `Shell`; the platform crate has a
  *separate* `ShellKind` — don't confuse them.

Never put platform `#[cfg]` in engine `src/`; route it through
`crates/platform`.

## PR checklist

- [ ] `cargo clippy --workspace --all-targets -- -D warnings` is clean
- [ ] `cargo fmt --check` passes
- [ ] `cargo test --workspace` (and the relevant feature tests) pass
- [ ] Commit messages follow **Conventional Commits**
- [ ] One concern per PR (keep it focused and reviewable)
- [ ] New behavior has tests
- [ ] User-facing changes update the docs (README, this file, or `ADVANCED.md`)
- [ ] No platform `#[cfg]` slipped into `crates/engine/src/` (CI + containment test)

## Note on historical docs

The file `brain/19-CONTRIBUTING-GUIDE.md` is the original **plan** for this
guide (historical). It described a fictional crate layout
(`config`/`tui`/`cli`/`cowsay`/`ai`/`shell` crates) that does **not** exist in
the current workspace. This `CONTRIBUTING.md` is the current, accurate guide.
