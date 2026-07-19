# PR #2 — build(packaging): winget/scoop/choco/nix/deb/rpm/msi/pacman/gentoo manifests, CI validation, Cmd/PowerShell hooks, terminal capability probe

Adds distribution packaging for **9 package managers** (body of work B), plus shell-hook expansion and a runtime terminal-capability probe.

## Package managers (all version `0.4.0`, `https://github.com/HKDEVS/forgum` URLs)
- **winget** — `packaging/winget/HKDEVS.Forgum.yaml` (multi-doc: version/installer/defaultLocale).
- **scoop** — `packaging/scoop/forgum.json` (x64 + arm64 zips, autoupdate).
- **choco** — `packaging/choco/forgum.nuspec` + `tools/chocolateyinstall.ps1`.
- **nix** — `packaging/nix/{flake.nix,package.nix,module.nix,README.md}`.
- **deb** — `packaging/deb/DEBIAN/{control,postinst}` + `build-deb.sh`.
- **rpm** — `packaging/rpm/forgum.spec` + `build-rpm.sh`.
- **msi/wix** — `packaging/windows/forgum.wxs` (consumed by `candle`/`light`).
- **pacman** — `packaging/pacman/PKGBUILD` (+ `build-pacman.sh`).
- **gentoo** — `packaging/gentoo/forgum-0.4.0.ebuild` + `metadata.xml`.
- (Plus existing **homebrew** `packaging/homebrew/forgum.rb` for parity.)

## CI validation jobs
- **Layer 3 packaging** jobs in `ci.yml`: `deb`, `rpm`, `msi`, `nix`, `pacman`, `choco`.
- **`windows-pkg-validate`** (Windows): validates scoop (JSON + zip URLs), choco (well-formed XML + version), winget (parsed YAML, `PackageVersion` + installer URL `forgum-0.4.0-windows-{x64,arm64}.zip`).
- **`version-parity`**: asserts every manifest's version equals `Cargo.toml` `0.4.0`.

## Shell expansion
- `crates/engine/src/init.rs`: added `Shell::Cmd` and `Shell::PowerShell` (now 6 variants: Bash/Zsh/Fish/Pwsh/Cmd/PowerShell), `generate_cmd_hook` (prompt-macro + doskey `forgum` + `sweep`), `generate_powershell_hook`.
- `crates/engine/src/cli.rs`: `ShellArg` gains `Cmd`/`PowerShell`; `Init` gains `--check` (CI validation mode).
- `ci.yml` **`shell-hooks`** matrix extended to cover bash/zsh/fish/pwsh/cmd.

## Terminal capability probe
- `crates/platform/src/terminal.rs`: `TerminalCapabilities` gains `sync: bool` (via `detect_sync_support()`, conservative allowlist) and `graphics: GraphicsCaps` (via `detect_graphics_cap()`); `terminal_supports_sync()` re-exported from `lib.rs`.
- `crates/platform/src/sixel.rs`: feature-gated Sixel/Kitty backend behind the `sixel` feature (default off).
- `crates/platform/src/lib.rs`: re-exports `terminal_supports_sync`, `GraphicsCaps`, `handle_count`.

## README updates (`README.md`)
- Terminal compatibility table with **Sync (DEC 2026)** + **Sixel/graphics** columns.
- Shell list now documents all six.
- Install matrix covers winget/scoop/choco/homebrew/deb/pacman/rpm/gentoo/nix.

## Release flow (`release.yml`)
i686 lane is `build_only` (never blocks x64/arm64, never packaged).

## Verify
```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
cargo test -p forgum-platform --features forgum-platform/sixel
# packaging + parity (CI): deb / rpm / msi / nix / pacman / choco + windows-pkg-validate + version-parity
```
