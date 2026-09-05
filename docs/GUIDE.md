# 📖 Master User Guide & Reference

<p align="center">
  <img src="assets/forgum_jungle_logo.jpg" alt="Forgum Jungle Engine Logo" width="100%" />
</p>

```text
+--------------------------------------------------------------------------------------------------+
|                              *  ~  *  FORGUM JUNGLE ENGINE  *  ~  *                              |
+==================================================================================================+
|        .~~.                                                                       .~~.           |
|       (____)                                                                     (____)          |
|     .-'    `-.                                                                 .-'    `-.        |
|   .'  / \  //\`.       ███████╗ ██████╗ ██████╗  ██████╗ ██╗   ██╗███╗   ███╗  .'  ^__^     `.   |
|  /|\___/| /   \`\      ██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██║   ██║████╗ ████║ /   /(oo)\____  \  |
| / /O   O \/ // \ \     █████╗  ██║   ██║██████╔╝██║  ███╗██║   ██║██╔████╔██║ |  /  (__)    )\ | |
| |( @_^_@ ) //   | |    ██╔══╝  ██║   ██║██╔══██╗██║   ██║██║   ██║██║╚██╔╝██║ |     ||---w | | | |
|  \ \__^_/ //    | /    ██║     ╚██████╔╝██║  ██║╚██████╔╝╚██████╔╝██║ ╚═╝ ██║ \     ||    || / / |
|   `-(_//)//____.-'     ╚═╝      ╚═════╝ ╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚═╝     ╚═╝  `-.________..-'   |
|       //  ||              * * *  A N I M A T E D   P A S T U R E  * * *             ||    ||     |
|      //   ||              ___                                     ,___.             ||    ||     |
|     //    ||            {~o.o~} ( ( (   K I N E M A T I C   ) ) ) {o,o}             ||    ||     |
|    //     ||             ( Y )  ) ) )      J U N G L E      ( ( ( /)__)             ||    ||     |
|   //      ||            ()~*~() ( ( (      M O T I O N      ) ) )  ""               ||    ||     |
+==================================================================================================+
|     [Dragon: Ember]           [Koala: Zen]           [Toucan: Sky]          [Cow: Pasture]       |
+--------------------------------------------------------------------------------------------------+
```

Welcome to the comprehensive user guide for **Forgum** — the high-performance Rust terminal animation engine featuring 2D kinematics, preloaded themes, and cross-platform terminal isolation.

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
9. [Preloaded Themes Subsystem](#9-preloaded-themes-subsystem)
10. [Kinematics & Physical Terminal Motion](#10-kinematics--physical-terminal-motion)

---

## 1. Installation

Install Forgum using your preferred package manager:

```bash
# Linux (Ubuntu / Kali Linux / Debian)
sudo apt install ./forgum_*.deb

# Linux (Fedora)
sudo dnf install ./forgum-*.rpm

# Linux (openSUSE)
sudo zypper install ./forgum-*.rpm

# Linux (Nix / NixOS)
nix profile install github:HKDevLoops/Forgum

# Windows
winget install HKDevLoops.Forgum
scoop bucket add extras && scoop install forgum
choco install forgum

# macOS
brew install forgum

# Cargo (Any)
cargo install forgum-engine
```

---

## 2. Command-Line Reference

### Basic Invocation
```bash
# Display the default cow with a random fortune
forgum-engine

# Render a specific cow with custom eyes and speech bubble
forgum-engine -c tux -e '$$' -t 'U' "Hello from Tux!"

# Run with a specific animation effect and color mode
forgum-engine --effect rainbow --color-mode rainbow --duration 3
```

### Subcommands

| Subcommand | Aliases | Parameters / Syntax | Description | Example |
| :--- | :--- | :--- | :--- | :--- |
| **`render`** | *(default)* | `[OPTIONS] [TEXT]...` | Renders 2D animated scenes with physical kinematics & scenery | `forgum render -E walk -c dragon --mountain alpine` |
| **`think`** | `ponder` | `[OPTIONS] [TEXT]...` | Thought bubble `( ... )` with circular `o` connectors | `forgum think "Deep thoughts"` |
| **`say`** | `speak` | `[OPTIONS] [TEXT]...` | Classic cowsay speech bubble `\| ... \|` with `\` stems | `forgum say git status` |
| **`fortune`** | `quote` | *(none)* | Prints a humorous terminal fortune quote | `forgum fortune` |
| **`list`** | `options`, `ls`, `show` | `[category]` | Displays formatted table of options for any parameter | `forgum list effects` |
| **`theme list`** | *(none)* | *(none)* | Lists all 15 preloaded and user themes | `forgum theme list` |
| **`theme apply <name>`** | *(none)* | `<name>` | Applies a preloaded theme (`matrix`, `cyberpunk`, etc.) | `forgum theme apply matrix` |
| **`theme rotate`** | *(none)* | `[--interval <s\>]` | Automatically rotates themes on an interval | `forgum theme rotate --interval 5` |
| **`theme seasonal`** | *(none)* | *(none)* | Applies dynamic real-world seasonal themes | `forgum theme seasonal` |
| **`timer <cmd...>`** | `stopwatch` | `<dur\> [cmd...]` | Benchmarks command execution and displays elapsed runtime | `forgum timer 10s cargo build` |
| **`checkhealth`** | `doctor`, `health` | `[--json]` | Runs diagnostic audit across 12 probes | `forgum checkhealth` |
| **`config`** | `cfg` | `[key] [val] [--list]` | Inspects, sets, or lists configuration options | `forgum config list` |
| **`config --tui`** | *(none)* | *(none)* | Opens the interactive configuration TUI | `forgum config --tui` |
| **`config --migrate <fmt>`** | *(none)* | `<json\|yaml\|toml>` | Migrates active config to `json`, `yaml`, or `toml` | `forgum config --migrate toml` |
| **`logs`** | `log` | `[-f] [-l <level\>]` | Displays structured system and execution logs | `forgum logs -f` |
| **`herd`** | `cluster` | `[list\|spawn\|kill]` | Lists or manages cows and background animation daemons | `forgum herd list` |
| **`tmux`** | `mux` | `[list\|install\|status]`| Configures tmux, zellij, or wezterm status lines | `forgum tmux list` |
| **`remote`** | `peers` | `[list\|who\|ping]` | Discovers pasture peers over network clusters | `forgum remote list` |
| **`battle`** | `arena` | `[f1] [f2]` | Turn-based ASCII battle between two critters | `forgum battle "Tux" "Dragon"` |
| **`status-line`** | *(none)* | `[--max-len <len\>]` | Compact ANSI status bar reporter for `$RPROMPT` | `forgum status-line --max-len 80` |
| **`control`** | `ctl` | `<status\|stop>` | Sends IPC commands to active daemon sessions | `forgum control status` |
| **`daemon`** | *(none)* | `<start\|stop>` | Manages background animation daemon | `forgum daemon start` |
| **`init <shell>`** | `hook` | `[shell]` | Generates shell hook integration scripts | `forgum init pwsh` |
| **`completions <shell>`** | `complete` | `[shell]` | Generates shell auto-completion scripts | `forgum completions zsh` |
| **`sweep`** | `clean` | *(none)* | Terminal emergency state recovery and cleanup | `forgum sweep` |

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
  "color_mode": "rainbow",
  "shell_attach_mode": "banner"
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
shell_attach_mode: banner
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
shell_attach_mode = "banner"
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
forgum-engine init pwsh | Out-String | Invoke-Expression

# Bash (~/.bashrc)
eval "$(forgum-engine init bash)"

# Zsh (~/.zshrc)
eval "$(forgum-engine init zsh)"

# Fish (~/.config/fish/config.fish)
forgum-engine init fish | source
```

---

## 5. Terminal Multiplexers

Forgum includes built-in configuration helpers for popular multiplexers:

```bash
# tmux
forgum-engine tmux install >> ~/.tmux.conf

# Zellij
forgum-engine tmux zellij >> ~/.config/zellij/config.kdl

# WezTerm
forgum-engine tmux wezterm >> ~/.wezterm.lua

# GNU Screen
forgum-engine tmux screen >> ~/.screenrc
```

---

## 6. Diagnostics with `checkhealth`

Modeled after Neovim's `:checkhealth`, running `forgum-engine checkhealth` inspects:

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
forgum-engine logs

# Filter by level
forgum-engine logs --level error

# Real-time tailing
forgum-engine logs -f
```

---

## 8. Custom Cows & DNA Signatures

Add custom cows by dropping any ASCII `.cow` file into `~/.config/forgum/cows/`:

```
~/.config/forgum/cows/mycritter.cow
```

Forgum will automatically detect your custom creature and apply matching DNA physics profiles (tail wags, breathing oscillation, float, trot, and particle streams).

---

## 9. Preloaded Themes Subsystem

Forgum includes **15 built-in preloaded themes** ready out-of-the-box. Themes combine effects, mascots, eyes, and expressions into curated presets:

```bash
# List all preloaded and custom themes
forgum-engine theme list

# Apply a theme immediately to active sessions
forgum-engine theme apply matrix
forgum-engine theme apply cyberpunk
forgum-engine theme apply inferno

# Periodically rotate themes every N minutes
forgum-engine theme rotate --interval 10

# Trigger real-world seasonal themes (Halloween, New Year, Valentine, etc.)
forgum-engine theme seasonal
```

| Theme       | Animation Effect | Cow Mascot     | Eyes | Expression | Aesthetic                           |
| :---------- | :--------------- | :------------- | :--- | :--------- | :---------------------------------- |
| `arcade`    | `walk`           | `default`      | `oo` | `U`        | Retro 8-bit pasture walk            |
| `aurora`    | `aurora`         | `default`      | `oo` | `U`        | Polar lights shifting colors        |
| `cyberpunk` | `glitch`         | `mech-and-cow` | `$$` | `U`        | High-tech dystopian scanlines       |
| `forest`    | `breathe`        | `koala`        | `..` | `U`        | Gentle bamboo forest breathing      |
| `ghost`     | `portal`         | `ghost`        | `xx` | `U`        | Ethereal floating apparition        |
| `inferno`   | `ember`          | `dragon`       | `@@` | `U`        | Blazing fire particles and heat     |
| `matrix`    | `glitch`         | `telebears`    | `00` | `U`        | Cascading terminal green stream     |
| `nyan`      | `float`          | `nyan`         | `^^` | `U`        | Rainbow space orbital drift         |
| `ocean`     | `float`          | `dolphin`      | `oo` | `U`        | Underwater buoyancy drift           |
| `retro`     | `walk`           | `default`      | `oo` | `U`        | Classical terminal stride           |
| `stealth`   | `static`         | `tux`          | `--` | `  `       | Silent ninja terminal penguin       |
| `supernova` | `ember`          | `stegosaurus`  | `**` | `U`        | Cosmic starburst particle emissions |
| `valentine` | `glitch`         | `default`      | `@@` | `U`        | Pink & red cyber-heart flutter      |
| `winter`    | `aurora`         | `snowman`      | `**` | `U`        | Crisp arctic snowfall breeze        |
| `zen`       | `breathe`        | `tux`          | `==` | `U`        | Deep meditative penguin stillness   |

---

## 10. Kinematics & Physical Terminal Motion

Entities in Forgum are driven by continuous 2D coordinate kinematics:

### 10.1 Numerical State Integration
Entities track continuous floating-point position $\vec{P}(t)$, velocity $\vec{V}(t)$, and acceleration $\vec{A}(t)$, quantizing to discrete terminal cell bounds each frame:

$$\vec{P}(t + \Delta t) = \vec{P}(t) + \vec{V}(t)\Delta t + \frac{1}{2}\vec{A}(t)\Delta t^2$$

$$\vec{V}(t + \Delta t) = \vec{V}(t) + \vec{A}(t)\Delta t$$

Discrete terminal column and row cell indices:

$$x_{\text{col}} = \lfloor P_x(t) \rceil, \quad y_{\text{row}} = \lfloor P_y(t) \rceil$$

### 10.2 Stride-Velocity Geometric Coupling
Leg gait alternation is mathematically bound to ground velocity, eliminating the "treadmill illusion":

$$\phi_{\text{stride}}(t) = \left( \frac{|P_x(t)|}{\lambda_{\text{step}}} \right) \pmod{1.0}$$

$$\text{LegState}(t) = \begin{cases} (\text{'╱'}, \text{'╲'}), & \text{if } \text{Easing}(\phi_{\text{stride}}) > 0.5 \\ (\text{'╲'}, \text{'╱'}), & \text{otherwise} \end{cases}$$

### 10.3 2D Harmonic Lissajous Buoyancy
Aquatic and space mascots follow continuous orthogonal harmonic drift across screen margins:

$$X(t) = X_{\text{anchor}} + A_x \cdot \sin(\omega_x t + \delta_x), \quad Y(t) = Y_{\text{anchor}} + A_y \cdot \cos(\omega_y t + \delta_y)$$

### 10.4 Differential Damage Matrix
Rather than clearing the whole terminal (`\x1b[2J`), the rasterizer computes the exact difference set between consecutive double buffers:

$$\text{Damage}(t) = \left\{ (x, y) \in \mathcal{W} \times \mathcal{H} \;\middle|\; \text{Back}(x, y) \neq \text{Front}(x, y) \right\}$$

Only coordinates in $\text{Damage}(t)$ emit direct ANSI jump commands (`\x1b[{row};{col}H`), achieving silky-smooth 60 FPS animation at near-zero CPU usage.

### 10.5 Sub-Millisecond Fail-Safe Signal Response
Pressing `Ctrl+C`, `q`, or `Esc` immediately exits foreground interactive mode and cleanly restores the terminal buffer and cursor. If a second interrupt is received before graceful exit completes, the signal subsystem immediately terminates the process (`exit(130)`).

---

## 📜 License & Community

Forgum is free, open-source software licensed under the **[MIT License](../LICENSE)**.

*Maintained with ❤️ by the Forgum Contributors.*
