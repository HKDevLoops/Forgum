# Forgum

Cross-platform cowsay + fortune + lolcat with a Rust ANSI animation engine.

The flagship promise: animated ASCII cows render **above the shell prompt** as
a non-blocking overlay while the prompt stays fully interactive.

## Workspace

This repository contains a Cargo workspace with two crates:

- `forgum-platform` — cross-platform abstraction layer (terminal handles,
  signal guards, raw mode/alt screen/cursor RAII, output redirection,
  config paths, detached spawning). All `#[cfg]` in the workspace lives here.
- `forgum-engine` — the animation engine binary. Zero `#[cfg]`; it
  programs against the `forgum-platform` trait surface.

Plus a PowerShell module (`Forgum.psm1` + `Public/` + `Private/`) that
shells out to the engine.

## Status

**v0.4.0** — all stabilization phases (Phase 0–9) are complete. Cross-platform cowsay/fortune/lolcat with a Rust ANSI animation engine, daemonized overlay rendering above the prompt, tmux/zellij/wezterm/screen integration, a herd fleet manager, remote (rmux) sync across SSH, and packaged builds (deb/rpm/homebrew/winget/scoop). See `docs/superpowers/` for the full planning kit.

## Build

```bash
cargo build --release
./target/release/forgum-engine --help
```

## Test

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```

## Cross-platform paths

Honored environment variables:

| Var                | Purpose                          |
|--------------------|----------------------------------|
| `FORGUM_CONFIG`    | Override config JSON path        |
| `FORGUM_DATA`      | Override data dir (cow files)    |
| `FORGUM_RUNTIME`   | Override runtime dir (PID, sock) |
| `FORGUM_LOG`       | Override log dir                 |
| `FORGUM_ENGINE`    | Override path to engine binary   |

## License

MIT. See `LICENSE`.

## Provenance

Built against the planning kit in `brain/` (v1.1.2 audit + v2 roadmap).
Phase 0 fixes target the specific file:line references in
`brain/01-BUGS-AND-ISSUES.md`.