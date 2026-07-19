# PR #1 — perf(engine/platform): dirty-tracking framebuffer, zero-alloc renderer, off-by-default sync, cross-platform CI matrix, daemon leak soak

Implements the engine + platform perf/leak/modern-render refactor (body of work A).

## Tasks
- **T0 — Dirty-tracking framebuffer** (`crates/engine/src/framebuffer.rs`): `set()` compares against `front`; `clear()` marks nothing; `swap()` = `mem::swap`; `compute_damage()` is `O(changed)` and zero-alloc when the frame is static.
- **T1/D1/D3 — Zero-alloc renderer** (`crates/engine/src/renderer.rs`): `AnsiRenderer` reuses a `scratch: Vec<u8>` and a manual `write_decimal`, no `format!()` per cell.
- **T4/G3/D4 — DEC 2026 synchronized update**: `SyncGuard` (RAII) in `render.rs` wraps per-frame damage and always emits `end_sync` on drop; gated by `cfg!(feature = "synchronized-update")` so it is **OFF by default** and byte-identical to before when off.
- **T6/G5 — Cross-platform CI matrix**: `rust-build-test` on ubuntu/windows/macos-14 + `arm64-linux-test` cross lane in `ci.yml`.
- **D2/G2 — Daemon leak soak**: `crates/engine/tests/daemon_soak.rs` asserts fd/handle count stability (delta ≤ 8 over 200 connections) via `forgum_platform::handle_count()`.
- **G8 — cfg isolation**: all `#[cfg(unix/windows/target_os/…)]` confined to `forgum-platform`; `engine/src` stays cfg-free (CI `cfg-grep` gate + `tests/cfg_containment.rs`).
- **GPU stub**: `Renderer` trait is backend-agnostic; deferred wgpu backend documented but **no wgpu dependency added**.

## Bug fixes
- **BUG-A (stale frame):** renderer now reads `get_back()` (the just-built back buffer) instead of `front`. Regression covered by `tests/render_correctness.rs` and `renderer.rs` `ansi_renderer_reads_back_buffer`.
- **BUG-E1 (idle 60fps):** `Cell::PartialEq` no longer includes a `dirty` field, so identical frames produce **0 damage**; scheduler idles correctly on a static cow. Covered by `bug_e1_regression_*` tests.

## Key files
`crates/engine/src/{framebuffer,renderer,render,init,cli}.rs`, `crates/platform/src/{terminal,sixel,lib}.rs`, `crates/engine/tests/{daemon_soak,render_correctness,sync_correctness,cfg_containment}.rs`, `Cargo.toml` (workspace lint `unsafe_code=deny`), `.github/workflows/ci.yml`.

## How to verify
```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
# feature-flag variants:
cargo test --workspace --features forgum-engine/synchronized-update
cargo test -p forgum-platform --features forgum-platform/sixel
cargo test -p forgum-engine --no-default-features
```
- `cfg-grep` job must report "OK: no platform cfg in engine source."
- `tests/cfg_containment.rs::engine_source_has_zero_platform_cfg_attributes` must pass.
