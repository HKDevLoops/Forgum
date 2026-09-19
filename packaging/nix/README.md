# ╔══════════════════════════════════════════════════════════════════════════╗
║                   🐧  N I X   P A C K A G I N G  🐧                    ║
║                                                                        ║
║   Build, distribute, and sandbox Forgum with Nix and NixOS.            ║
╚══════════════════════════════════════════════════════════════════════════╝

This directory contains production Nix expressions and NixOS modules to build, run, and distribute `forgum` without modifying host configurations.

---

## ⚡ Zero-Host-Modification Instant Sandbox Run

You can run and test Forgum instantly in an isolated ephemeral environment without installing anything to your machine:

```sh
# Run interactive mascot say in sandbox
nix run github:HKDevLoops/Forgum -- say "Zero-config Nix sandbox test!" --cow tux

# Run mascot animation showcase
nix run github:HKDevLoops/Forgum -- showcase

# Run battle arena
nix run github:HKDevLoops/Forgum -- battle --player Knight --cpu Dragon

# Run doctor diagnostics
nix run github:HKDevLoops/Forgum -- doctor
```

---

## 📁 Files

| File | Purpose |
|------|---------|
| `flake.nix` | Standard Nix flake with package, apps, devShell, and NixOS module |
| `package.nix` | Callable standalone package expression (`pkgs.callPackage`) |
| `module.nix` | NixOS / home-manager module (`programs.forgum`) |
| `README.md` | This documentation |

---

## 🏗️ Building with Flakes

```sh
# Build the default forgum package
nix build

# Run the newly built binary
./result/bin/forgum --version
```

---

## 🔧 Building without Flakes

```sh
nix-build -E 'with import <nixpkgs> {}; import ./packaging/nix/package.nix {}'
./result/bin/forgum doctor
```

---

## 🐚 NixOS / home-manager Module

Enable the automatic shell hooks for Bash, Zsh, Fish, and Nushell:

```nix
{
  imports = [ (import ./packaging/nix/flake.nix).nixosModules.forgum ];

  programs.forgum = {
    enable = true;
    # package = pkgs.forgum; # Optional package override
  };
}
```

---

<div align="center">

```
    \   ^__^
     \  (oo)\_______
        (__)\       )\/\
            ||----w |
            ||     ||
```

*Nix is love. Nix is life. 🐮*

</div>
