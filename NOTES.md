# NOTES — the user's world (canonical terms)

Raw, evolving notes. Sharpened terms get a `> canonical:` marker.

## Project
- Forgum: a Rust CLI that renders ANSI cows (cowsay-like) in a live terminal.
  Repository (verified by user): `https://github.com/HKDevLoops/Forgum.git`, branch `main`.
  > canonical: `repo` = HKDevLoops/Forgum @ main. (Note: older docs referenced `HKDEVS/forgum` — stale; the real remote is HKDevLoops/Forgum.)
- Workspace = 3 crates: `crates/engine` (binary `forgum-engine`, must stay free of platform `#[cfg]`), `crates/platform` (all platform `#[cfg]` lives here), `crates/tui` (feature-gated ratatui config menu).
- Engine features: `synchronized-update` (off by default, runtime `cfg!` macro), `tui` (optional `forgum-tui` dep). Platform feature: `sixel`.

## Test gate (the "tests must pass" bar)
The local gate mirrors CI (`.github/workflows/ci.yml`):
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --check`
- cfg-grep: engine/src must contain NO platform-targeting `#[cfg]` (`unix`/`windows`/`target_os`/`target_family`/`target_arch`/etc). CI prints `OK: no platform cfg in engine source.`
- `Invoke-Pester Tests/`
- Deep-check (optional/slow): `cargo test -p forgum-engine --features forgum-engine/synchronized-update`, `cargo test -p forgum-platform --features forgum-platform/sixel`
> canonical: `fast-gate` = test+clippy+fmt+cfg-grep+pester. `deep-gate` = fast-gate + feature-flag variants.

## Channels / tools the user processes
- `git` + `gh` CLI (user has authenticated `gh` locally).
- Direct-to-`main` push is the user's stated preference ("stable, tried and tested is main; other branches for contributors").
- Shell execution is BLOCKED inside the assistant's session harness (`{"permission":"bash","action":"deny","pattern":"*"}`), so the assistant cannot run git/gh itself — the user runs prepared scripts locally. The *workflow* below is meant to run in the user's own environment where bash/gh work.
> canonical: `push-target` = `origin/main`, direct (no PR required by owner). `assistant-session` = git/gh-blocked; user executes locally.

## Workflow vocabulary (from the grilling session)
- `loop` — a recurring pattern in the user's life made real by running a workflow on it.
- `workflow` — a spec of one loop, in `workflows/*.md`, source of truth.
- `trigger` — event (commit/push attempt) or schedule (nightly) that fires a run.
- `checkpoint` — human-in-the-loop verify/decide point; may be absent (autonomous) or absent-of-AI.
- `push-right` — defer the checkpoint as far as possible; do maximal work, ask once, late.
- `brief` — decision-ready summary (what / why / link to asset), never raw output.

## commit-push workflow decisions (grilling resolutions)
- Q1 trigger: A (event on commit/push attempt) + B (nightly schedule).
- Q2 event mechanism: A (local wrapper/command) + B (pre-push hook).
- Q3 gate: all — fast-gate + deep-gate + minimal subset documented.
- Q4 on-fail: all — mechanical fix → investigate/research → robust fix removing latent bugs → checkpoint if exhausted.
- Q5 retries: A (bounded escalating) + B rejected in favor of hard cap (5 attempts OR 10 min).
- Q6 brief: A (decision-ready).
- Q7 asset: A (local report file under `workflows/reports/` + terminal print).
- Q8 schedule: B (nightly pushes already-committed unpushed only; never auto-commits WIP).
- Q9 .gitignore: A (pre-flight check, auto-append missing artifact patterns, bundle into commit, surface in brief).
- Q10 specs: A (workflows/*.md + NOTES.md are first-class, always committed; transient generated markdown ignored; brief names driving spec).
