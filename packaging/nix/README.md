# Nix packaging for forgum

This directory contains Nix expressions to build and distribute `forgum`
(cross-platform cowsay+fortune+lolcat) from the repository.

## Files

- `flake.nix` — standard Nix flake (nixpkgs + flake-utils).
- `package.nix` — callable package expression for non-flake use.
- `module.nix` — NixOS/home-manager module (`programs.forgum`).
- `README.md` — this file.

## Building with the flake (Linux CI)

```sh
nix build .#default
# or just:
nix build
```

This builds the `forgum-engine` crate via `cargo build -p forgum-engine`
against the workspace `Cargo.lock` at the repo root.

## Building without flakes

```sh
nix-build -E 'with import <nixpkgs> {}; import ./packaging/nix/package.nix {}'
```

## NixOS / home-manager module

Enable the shell hook for bash/zsh/fish:

```nix
imports = [ (import ./packaging/nix/flake.nix).nixosModules.forgum ];
programs.forgum.enable = true;
# programs.forgum.package = pkgs.forgum; # optional override
```

> Note: the module is best-effort and untested in CI (Nix CI runs on
> Linux only). It assumes `forgum-engine init <shell>` prints shell init code.
