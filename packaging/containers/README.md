# 🐳 Forgum Podman & Container Environments

This directory provides production-ready container definitions for running and testing Forgum across **Docker**, **Podman**, **Windows Containers**, and **Hybrid Linux/macOS Sandboxes** without modifying any host configuration.

---

## 🚀 Quick Start (Zero Host Modification)

You can launch an isolated interactive Forgum sandbox immediately without installing dependencies or touching your local shell configurations:

### Using Docker Compose (Repo Root)
```bash
# Launch interactive Zsh sandbox with Forgum hooks pre-loaded
docker compose run --rm forgum

# Run the 60-second mascot animation showcase
docker compose run --rm showcase

# Launch an interactive mascot battle
docker compose run --rm battle

# Run the environment and diagnostic doctor
docker compose run --rm doctor

# Run full cross-shell verification suite
docker compose run --rm test
```

### Using Podman / Docker Directly
```bash
# Build multi-stage standalone image
docker build -t forgum-sandbox .
# or:
podman build -t forgum-sandbox .

# Run interactive shell session
docker run -it --rm forgum-sandbox
# or with Podman:
podman run -it --rm forgum-sandbox
```

---

## 🪟 1. Windows Container Environment (`Containerfile.windows`)

Designed for Windows Container hosts running Podman or Docker in Windows container mode.

### Features
- **Base Image:** `mcr.microsoft.com/powershell:lts-nanoserver-ltsc2022`
- **Config:** Preconfigured at `C:\Users\ContainerUser\.config\forgum\config.json`
- **Prompt Hook:** Automatically injects `forgum init pwsh` into `$PROFILE`
- **Primary Binary:** `forgum.exe` in system PATH
- **Aliases:** `cowsay` -> `forgum say`, `lolcat` -> `forgum render`

### Build & Run
```powershell
# Build Windows container image
podman build -f packaging/containers/Containerfile.windows -t forgum-windows .

# Run interactive PowerShell session with Forgum
podman run -it --rm forgum-windows
```

---

## 🍎🐧 2. Hybrid Linux + macOS (Darwin-style) Environment (`Containerfile.linux-macos`)

Designed to emulate a modern POSIX terminal matching both Linux distributions and macOS (Darwin / Homebrew / iTerm2 / Zsh).

### Features
- **Shells:** Zsh (`/bin/zsh` default login shell) & Bash (`/bin/bash`) & Fish (`fish`)
- **Homebrew Compatibility Layout:** Binaries linked under `/opt/homebrew/bin` and `/usr/local/bin`
- **Config:** Unified XDG TOML configuration at `~/.config/forgum/config.toml`
- **Terminal Capabilities:** Truecolor 24-bit ANSI (`COLORTERM=truecolor`, `TERM_PROGRAM=iTerm.app`)
- **Prompt Hooks:** Automatically configured in `~/.zshrc`, `~/.bashrc`, and `~/.config/fish/config.fish`
- **In-Container Updates:** Run `forgum-update` or `forgum update --check`

### Build & Run
```bash
# Build Linux/macOS-hybrid container image
podman build -f packaging/containers/Containerfile.linux-macos -t forgum-unix-darwin .

# Run interactive Zsh terminal session with Forgum
podman run -it --rm forgum-unix-darwin
```

---

## 🔄 Updating Within the Sandbox

Inside any running container sandbox, you can check for upstream repository updates:

```bash
# Check for latest releases from the repo
forgum update --check

# Switch release channels (stable, nightly, dev)
forgum channel switch nightly
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
