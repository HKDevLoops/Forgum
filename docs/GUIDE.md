# 📖 Master User Guide & Reference

Welcome to the comprehensive user guide for **Forgum**.

---

## 📑 Table of Contents

1. [Installation](#1-installation)
2. [Command-Line Reference](#2-command-line-reference)
   - [Basic Invocation](#basic-invocation)
   - [Subcommands](#subcommands)
3. [Configuration Reference](#3-configuration-reference)
   - [Unified Config Path](#unified-config-path)
   - [Supported Formats (JSON, YAML, TOML)](#supported-formats)
   - [Config Schema Options](#config-schema-options)
   - [Migration Command](#migration-command)
   - [Interactive TUI Menu](#interactive-tui-menu)
4. [Attachment Modes & Shell Integration](#4-attachment-modes--shell-integration)
5. [Terminal Multiplexers (tmux, Zellij, WezTerm, Screen)](#5-terminal-multiplexers)
6. [Diagnostics with `checkhealth`](#6-diagnostics-with-checkhealth)
7. [Structured Logging](#7-structured-logging)
8. [Custom Cows & DNA Signatures](#8-custom-cows--dna-signatures)

---

## 1. Installation

Install Forgum using your preferred package manager:

```bash
# Windows
winget install HKDevLoops.Forgum
scoop bucket add extras && scoop install forgum
choco install forgum

# macOS
brew install forgum

# Linux
sudo apt install forgum      # Debian/Ubuntu (.deb)
sudo dnf install forgum      # Fedora/RHEL (.rpm)
sudo pacman -S forgum        # Arch Linux
sudo emerge forgum           # Gentoo
nix-env -iA nixpkgs.forgum   # Nix

# Rust / Cargo
cargo install forgum
```

---

## 2. Command-Line Reference

### Basic Invocation
```bash
# Display the default cow with a random fortune
forgum

# Render a specific cow with custom eyes and speech bubble
forgum -c tux -e '$$' -t 'U' "Hello from Tux!"

# Run with a specific animation effect and color mode
forgum --effect rainbow --color-mode rainbow --duration 3
```

### Subcommands

| Subcommand | Description | Example |
| :--- | :--- | :--- |
| **`say <command...>`** | Runs a shell command and displays output inside the cow's speech bubble | `forgum say git status` |
| **`fortune`** | Prints a humorous terminal fortune | `forgum fortune` |
| **`timer <command...>`** | Benchmarks command execution and displays elapsed runtime | `forgum timer cargo build` |
| **`checkhealth`** *(alias: `health`)* | Runs diagnostic audit across 7 layers | `forgum checkhealth` |
| **`config`** | Inspects or edits configuration options | `forgum config set color_mode rainbow` |
| **`config --tui`** | Opens the interactive configuration TUI | `forgum config --tui` |
| **`config --migrate <fmt>`** | Migrates active config to `json`, `yaml`, or `toml` | `forgum config --migrate toml` |
| **`logs`** | Displays structured system and execution logs | `forgum logs -f` |
| **`herd`** | Lists or manages cows and background animation daemons | `forgum herd list` |
| **`control`** | Sends IPC commands to active daemon sessions | `forgum control status` |
| **`init <shell>`** | Generates shell hook integration scripts | `forgum init pwsh` |
| **`completions <shell>`** | Generates shell auto-completion scripts | `forgum completions zsh` |

---

## 3. Configuration Reference

### Unified Config Path
Forgum looks in `~/.config/forgum/` on **Windows**, **macOS**, and **Linux**:
- Windows: `%USERPROFILE%\.config\forgum\config.json`
- macOS: `$HOME/.config/forgum/config.json`
- Linux: `$HOME/.config/forgum/config.json`

### Supported Formats

#### JSON (`~/.config/forgum/config.json`)
```json
{
  "cow": "default",
  "text": "Moo!",
  "effect": "static",
  "background": "none",
  "duration": 0,
  "fps": 30,
  "eyes": "oo",
  "tongue": "  ",
  "default_shell": "pwsh",
  "auto_render_on_prompt": true,
  "color_mode": "rainbow"
}
```

#### YAML (`~/.config/forgum/config.yaml`)
```yaml
cow: default
text: Moo!
effect: static
background: none
duration: 0
fps: 30
eyes: "oo"
tongue: "  "
default_shell: pwsh
auto_render_on_prompt: true
color_mode: rainbow
```

#### TOML (`~/.config/forgum/config.toml`)
```toml
cow = "default"
text = "Moo!"
effect = "static"
background = "none"
duration = 0
fps = 30
eyes = "oo"
tongue = "  "
default_shell = "pwsh"
auto_render_on_prompt = true
color_mode = "rainbow"
```

---

## 4. Attachment Modes & Shell Integration

Forgum supports four distinct shell attachment strategies:

1. **Banner Burst:** Renders a fast, non-intrusive splash screen above your prompt.
2. **Split Margin (DECSTBM):** Pins a designated viewport margin for the cow while allowing the shell to scroll freely.
3. **PSReadLine Idle Overlay:** Fades in when the terminal is idle and immediately suspends when input is detected.
4. **Manual CLI:** Runs on-demand without altering shell prompt hooks.

### Shell Hook Commands:

```powershell
# PowerShell 7+ ($PROFILE)
forgum init pwsh | Out-String | Invoke-Expression

# Bash (~/.bashrc)
eval "$(forgum init bash)"

# Zsh (~/.zshrc)
eval "$(forgum init zsh)"

# Fish (~/.config/fish/config.fish)
forgum init fish | source
```

---

## 5. Terminal Multiplexers

Forgum includes built-in configuration helpers for popular multiplexers:

```bash
# tmux
forgum tmux install >> ~/.tmux.conf

# Zellij
forgum tmux zellij >> ~/.config/zellij/config.kdl

# WezTerm
forgum tmux wezterm >> ~/.wezterm.lua

# GNU Screen
forgum tmux screen >> ~/.screenrc
```

---

## 6. Diagnostics with `checkhealth`

Modeled after Neovim's `:checkhealth`, running `forgum checkhealth` inspects:

```
==============================================================================
forgum checkhealth — Engine v0.4.0
==============================================================================

## System & Platform Environment
  [OK] OS & Architecture: windows (x86_64)
  [OK] Engine Binary: exists on disk

## Configuration & Multi-Format Subsystem
  [OK] Active Configuration: ~/.config/forgum/config.json (JSON)
  [OK] Config Directory Permissions: Writable

## Terminal Capabilities & Color Protocols
  [OK] Terminal Grid: 188x53 (TTY: true)
  [OK] Color Capabilities: TrueColor 24-bit RGB supported

## Pasture Assets & 2D Compound Signatures
  [OK] Cow Pasture Assets: 109 cow files available
  [OK] DNA Signature Animation Profiles: 10 registered

## Shell Integration & Prompt Hooks
  [OK] Active Shell Environment: Pwsh

## Daemon & IPC Subsystem
  [OK] Background Daemons: 0 running (Dead daemon auto-sweep active)

## Structured Logging & Traceability
  [OK] Logging Subsystem: Writable (forgum.log & forgum.jsonl)
```

---

## 7. Structured Logging

Forgum logs all operations with timestamp, level, target, and message:

```bash
# View table of recent logs
forgum logs

# Filter by level
forgum logs --level error

# Real-time tailing
forgum logs -f
```

---

## 8. Custom Cows & DNA Signatures

Add custom cows by dropping any ASCII `.cow` file into `~/.config/forgum/cows/`:

```
~/.config/forgum/cows/mycritter.cow
```

Forgum will automatically detect your custom creature and apply matching DNA physics profiles (tail wags, breathing oscillation, float, trot, and particle streams).
