# 🐳 Forgum Podman & Container Environments

This directory provides production-ready container definitions for running and testing Forgum across both **Windows Container Runtimes** and a **Hybrid Linux + macOS (Darwin-style) Terminal Environment**.

---

## 🪟 1. Windows Container Environment (`Containerfile.windows`)

Designed for Windows Container hosts running Podman or Docker in Windows container mode.

### Features
- **Base Image:** `mcr.microsoft.com/powershell:lts-nanoserver-ltsc2022`
- **Config:** Preconfigured at `C:\Users\ContainerUser\.config\forgum\config.json`
- **Prompt Hook:** Automatically injects `forgum-engine init pwsh` into `$PROFILE`
- **Engine Features:** 30 FPS, TrueColor rendering, `tux` mascot with speech wrapping

### Build & Run
```powershell
# Build Windows container image
podman build -f packaging/containers/Containerfile.windows -t forgum-windows .

# Run interactive PowerShell session with Forgum
podman run -it --rm forgum-windows
```

---

## 🍎🐧 2. Hybrid Linux + macOS (Darwin-style) Environment (`Containerfile.linux-macos`)

Designed to emulate a modern POSIX terminal matching both Linux distros (Ubuntu/Debian/Arch) and macOS (Darwin / Homebrew / iTerm2 / Zsh).

### Features
- **Shells:** Zsh (`/bin/zsh` default macOS login shell) & Bash (`/bin/bash` Linux standard)
- **Homebrew Compatibility Layout:** Binaries linked under `/opt/homebrew/bin` and `/usr/local/bin`
- **Config:** Unified XDG TOML configuration at `~/.config/forgum/config.toml`
- **Terminal Capabilities:** Truecolor 24-bit ANSI (`COLORTERM=truecolor`, `TERM_PROGRAM=iTerm.app`)
- **Prompt Hooks:** Automatically configured in both `~/.zshrc` and `~/.bashrc`
- **Engine Features:** 60 FPS animation loop, `daemon` cow, dynamic status line

### Build & Run
```bash
# Build Linux/macOS-hybrid container image
podman build -f packaging/containers/Containerfile.linux-macos -t forgum-unix-darwin .

# Run interactive Zsh terminal session with Forgum
podman run -it --rm forgum-unix-darwin
```

---

## ⚙️ Custom Configuration Overrides

You can mount your own custom configurations when launching either container:

### Windows:
```powershell
podman run -it --rm -v "C:\my-config\config.json:C:\Users\ContainerUser\.config\forgum\config.json:ro" forgum-windows
```

### Linux / macOS:
```bash
podman run -it --rm -v "$PWD/custom-config.toml:/home/forgum/.config/forgum/config.toml:ro" forgum-unix-darwin
```

---

## 📜 License
MIT License - Copyright (c) Forgum Contributors.
