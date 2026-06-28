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

This is **Phase 0** of the stabilization plan. It implements the contract
described in [`docs/superpowers/specs/2026-06-28-phase0-stop-the-bleeding-design.md`](docs/superpowers/specs/2026-06-28-phase0-stop-the-bleeding-design.md).

Fixed bug IDs from `brain/01-BUGS-AND-ISSUES.md`:

- **BUG-T1** — signal handlers (`signal-hook` on Unix, `SetConsoleCtrlHandler` on Windows)
- **BUG-T2** — RAII guards (`RawModeGuard`, `AltScreenGuard`, `CursorShowGuard`)
- **BUG-B1** — background loop never reads input
- **BUG-B2** — `duration=0` is infinite
- **BUG-E1** — `Cell::dirty` no longer participates in `PartialEq`
- **BUG-D4** — stdin bounded to 4 MB
- **BUG-D5** — malformed JSON exits non-zero
- **BUG-D7** — `saturating_mul` math, no `u32` overflow
- **BUG-B9** / **BUG-C1** — `open_output()` falls back to `/dev/tty` (Unix) / `CONOUT$` (Windows)

Plus:

- **BUG-T3** — `Invoke-ForgumEngine` does SIGTERM-first kill + force-restore escape
- **BUG-P30** — `Get-ForgumEngineBinary` resolves without auto-rebuild
- **BUG-C3** — `$env:FORGUM_CONFIG` honored
- Architecture invariant — zero `#[cfg(unix/windows)]` in `engine/src/`
  (CI-grep enforced)

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