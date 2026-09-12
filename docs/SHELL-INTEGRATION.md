# Shell Integration

Forgum hooks into your shell so the cow shows up automatically above your prompt.

## Quick Setup

```bash
forgum init <shell>
```

Where `<shell>` is one of: `bash`, `zsh`, `fish`, `pwsh`, `cmd`, `powershell`.

## Supported Shells

### Bash

```bash
eval "$(forgum init bash)"
```

**What it does:**
- Creates a `forgum()` function that renders cows in the background
- Adds `__forgum_precmd` to `PROMPT_COMMAND` to clean up dead daemons
- Creates runtime directory at `$XDG_RUNTIME_DIR/forgum` or `/tmp/forgum-$UID/forgum`

**Edge cases:**
- Works with bash 4.x and 5.x (including array `PROMPT_COMMAND`)
- `mkdir -p` silenced to avoid errors on read-only filesystems
- Guard: returns early if `__FORGUM_ENGINE` is unset

### Zsh

```bash
eval "$(forgum init zsh)"
```

**What it does:**
- Creates a `forgum()` function
- Adds `__forgum_precmd` to `precmd_functions` array
- Creates runtime directory

**Edge cases:**
- `typeset -a precmd_functions` guard for old zsh versions
- Same dead-daemon cleanup as bash

### Fish

```fish
forgum init fish | source
```

**What it does:**
- Creates `forgum` function
- Creates `__forgum_sweep` event handler on `fish_prompt`
- Creates runtime directory

**Edge cases:**
- Uses `set -q argv[1]` instead of `count $argv` for robustness
- Guard: returns early if `__forgum_engine` is unset

### PowerShell 7+ (pwsh)

```powershell
forgum init pwsh | Out-String | Invoke-Expression
```

**What it does:**
- Creates `forgum` function
- Wraps `global:prompt` to clean up dead daemons
- Backs up original prompt to `$global:__ForgumPromptBackup`

**Edge cases:**
- Detects PS version: uses `` `e `` for pwsh 7+, `[char]27` for PS 5.1
- Wraps prompt cleanup in `try/catch` for robustness
- Handles missing `$Host.UI.RawUI` gracefully

### PowerShell 5.1 (Windows PowerShell)

```powershell
forgum init powershell | Out-String | Invoke-Expression
```

Same as pwsh but for Windows PowerShell 5.1.

### CMD

```cmd
forgum init cmd
```

**What it does:**
- Registers `forgum` alias via `doskey`
- Wraps `prompt` macro to run dead-daemon sweeper
- Preserves previous prompt in `_FORGUM_OLD_PROMPT`

**Persistence:**
CMD has no precmd hook model. To persist across sessions, set the AutoRun registry key:

```cmd
reg add "HKCU\Software\Microsoft\Command Processor" /v AutoRun /t REG_SZ /d "<path-to-forgum-hook>"
```

## Multiplexer Integration

### tmux

```bash
forgum tmux install >> ~/.tmux.conf
```

Adds:
- `status-right` with cow status line
- `pane-focus-in` hook to start daemon
- `pane-focus-out` hook to stop daemon

### Zellij

```bash
forgum tmux zellij
```

Generates a `zellij run` command for the daemon.

### WezTerm

```bash
forgum tmux wezterm
```

Generates Lua config with `update-status` event handler.

### GNU Screen

```bash
forgum tmux screen
```

Generates `.screenrc` configuration block.

## Split Shell Overhaul & Terminal Adapter Architecture

Forgum features an advanced multi-channel split shell execution engine designed to dock animated mascots, dynamic scenery, and progress indicators at the top or bottom of your terminal session without interfering with your shell prompt, active command execution, or scrollback buffer history.

### The 4 Execution Modes

Forgum dynamically probes terminal capabilities and multiplexer environments to select the optimal split architecture:

| Mode | Identifier | Architecture & Behavior | Typical Targets |
| :--- | :--- | :--- | :--- |
| **Mode 1** | `decstbm` | **DECSTBM Scrolling Margin Split**: The terminal's hardware scrolling margin is locked to rows `[reserve_rows + 1, total_rows]`. Standard shell output scrolls naturally below the margin, while Forgum renders the mascot and scenery in the frozen rows `[1, reserve_rows]`. Zero scrollback corruption, instant cleanup on exit (`\x1b[r`). | Alacritty, Ghostty, Foot, VTE, Konsole, XTerm, Mintty, Apple Terminal, Windows Terminal |
| **Mode 2** | `native-api` | **Native Multiplexer & Terminal API Split**: Forgum delegates pane splitting directly to the active multiplexer or terminal emulator's IPC/CLI interface (`tmux`, `zellij`, `wezterm cli`, `kitty @`, `wt.exe`). Spawns an isolated visual pane above or beside the shell. | Tmux sessions, Zellij layouts, WezTerm CLI, Kitty Remote Control, Windows Terminal tabs |
| **Mode 3** | `precmd-fallback` | **Dynamic Shell Precmd Redraw Fallback**: For legacy or constrained terminals lacking DECSTBM margin isolation (e.g. Windows ConHost, Linux Virtual Console `/dev/tty1-6`, headless VMs), margin scrolling is automatically disabled. Forgum renders as a clean banner above the prompt via shell precmd hooks, avoiding screen scrambling and cursor lockups. | Legacy ConHost (`conhost.exe`), Linux Virtual Console (`/dev/tty1-6`), Serial consoles |
| **Mode 4** | `disabled` | **Diagnostic & Safe Fallback**: Active in non-interactive environments (pipes, headless CI/CD, dumb terminals). All margin manipulation and cursor repositioning are bypassed to guarantee zero stdout pollution. | `TERM=dumb`, file redirects, automated test runners |

---

### Terminal Compatibility Matrix

Forgum detects 18 terminal emulators and 4 terminal multiplexers via deep environment heuristics (prioritizing emulator-specific signatures before generic `TERM` fallbacks):

| Terminal Emulator / Environment | Detection Key | DECSTBM | DECSLRM | Native Split API | Default Split Mode | Notes & Constraints |
| :--- | :--- | :---: | :---: | :--- | :--- | :--- |
| **Windows Terminal** | `WT_SESSION` | ✅ | ❌ | `wt.exe -w 0 sp -H -s <ratio>` | `decstbm` | ConPTY VT engine; full DECSTBM margin support. Native split available via `wt.exe`. |
| **WezTerm** | `WEZTERM_PANE` / `TERM_PROGRAM=WezTerm` | ✅ | ✅ | `wezterm cli split-pane --top --percent <percent>` | `decstbm` | Full VT margins + rich CLI/Lua IPC pane management. |
| **Kitty** | `KITTY_WINDOW_ID` / `KITTY_PID` | ✅ | ❌ | `kitty @ launch --location=hsplit --bias=<percent>` | `decstbm` | Requires `allow_remote_control yes` in `kitty.conf` for native API split. |
| **Alacritty** | `ALACRITTY_WINDOW_ID` / `ALACRITTY_LOG` | ✅ | ✅ | None (Pure VT) | `decstbm` | Ultra-fast GPU renderer with DECSTBM and DECSLRM margin compliance. |
| **Ghostty** | `GHOSTTY_RESOURCES_DIR` / `TERM=ghostty` | ✅ | ✅ | None (Pure VT) | `decstbm` | Supports Synchronized Updates (`\x1b[?2026h`), DECSTBM, and DECSLRM. |
| **iTerm2** | `ITERM_SESSION_ID` / `TERM_PROGRAM=iTerm.app` | ✅ | ✅ | `osascript` AppleScript pane split | `decstbm` | macOS native terminal with DECSTBM, DECSLRM, and AppleScript/Python automation. |
| **Foot** | `FOOT_SERVER_SOCKET` / `TERM=foot` | ✅ | ❌ | None (Wayland Pure VT) | `decstbm` | Lightweight Wayland-native terminal with robust DECSTBM support. |
| **VTE / GNOME Terminal** | `VTE_VERSION` | ✅ | ❌ | None | `decstbm` | Powers GNOME Terminal, Tilix, Terminator, XFCE Terminal. Robust DECSTBM. |
| **Konsole / Yakuake** | `KONSOLE_VERSION` / `KONSOLE_DBUS_SERVICE` | ✅ | ❌ | None | `decstbm` | KDE Qt-based terminal with standard DECSTBM scrolling margins. |
| **XTerm / urxvt / Mintty** | `XTERM_VERSION` / `rxvt` / `mintty` | ✅ | ✅ (XTerm/Mintty) | None | `decstbm` | Canonical VT implementations honoring standard scrolling margins. |
| **macOS Terminal.app** | `TERM_PROGRAM=Apple_Terminal` | ✅ | ❌ | None | `decstbm` | Standard DECSTBM margin support; lacks DECSLRM. |
| **tmux** | `TMUX` | ✅ | ✅ | `tmux split-window -b -v -l <rows>` | `decstbm` (or `native-api`) | Multiplexer passthrough. Native pane split available with `-b -v`. |
| **Zellij** | `ZELLIJ` / `ZELLIJ_SESSION_NAME` | ✅ | ❌ | `zellij action new-pane -d up --` | `decstbm` (or `native-api`) | Rust terminal workspace multiplexer. |
| **Legacy Windows ConHost** | `COMSPEC` / Windows without `WT_SESSION` | ❌ | ❌ | None | `precmd-fallback` | Lacks scrolling margin isolation. Downgrades automatically to prevent visual corruption. |
| **Linux Virtual Console** | `TERM=linux` (`/dev/tty1-6`) | ❌ | ❌ | None | `precmd-fallback` | Kernel VT driver does not support DECSTBM. Downgrades to banner redraws. |
| **Serial / Hypervisor** | `TERM=vt100` / `/dev/ttyS*` | ❌ | ❌ | None | `precmd-fallback` | Constrained bandwidth; margin escape sequences skipped. |
| **Dumb / Non-TTY** | `TERM=dumb` / non-interactive pipe | ❌ | ❌ | None | `disabled` | All ANSI escapes and margin sequences disabled. |

---

### Configuration & CLI Controls

You can control split shell behavior via CLI arguments or through the central configuration file (`~/.config/forgum/config.json` or `%APPDATA%\Forgum\config.json`).

#### CLI Arguments

- `--split-scroll`: Enable scrolling margin overlay. Restricts terminal scrolling to the region beneath the cow animation.
- `--split-mode <MODE>`: Explicitly select the split mode:
  - `decstbm`: Force DECSTBM hardware margins (fails gracefully if unsupported).
  - `native-api`: Spawn a native multiplexer or terminal emulator pane via CLI/IPC.
  - `precmd-fallback`: Force shell precmd banner redraw without hardware margins.
  - `disabled`: Disable all split shell rendering.
- `--reserve-rows <N>`: Explicit number of top rows reserved for the cow animation (default: auto-computed from mascot height).
- `--reserve-cols <N>`: Explicit number of columns reserved for vertical split layouts.
- `--split-ratio <FLOAT>`: Fraction of terminal height or width allocated to Forgum (range: `0.05` to `0.80`, default: `0.30`).

#### Example Commands

```bash
# Render cow in a frozen top margin reserving 10 rows
forgum render --split-scroll --reserve-rows 10 --text "Margined Cow!"

# Launch using native multiplexer split pane (tmux, zellij, wezterm, kitty, wt)
forgum render --split-mode native-api --text "Native Pane Cow!"

# Run daemon with custom split ratio
forgum daemon start --split-scroll --split-ratio 0.25
```

#### Configuration File Schema (`config.json`)

```json
{
  "split_scroll": true,
  "split_mode": "decstbm",
  "reserve_rows": 10,
  "split_ratio": 0.30
}
```

---

### Diagnostics & Troubleshooting

Use Forgum's built-in health inspection commands to audit your terminal environment:

#### 1. Comprehensive System Inspection: `forgum doctor`
Run `forgum doctor` to verify:
- Detected Terminal Emulator identity and ID
- DECSTBM and DECSLRM capability flags
- Recommended and active Split Mode
- Native split command availability (e.g. `wt.exe`, `tmux`, `wezterm cli`)
- Environment limitation warnings (e.g. ConHost, Linux TTY)

```bash
forgum doctor
```

Example Output:
```text
=== Forgum System Diagnostics ===
...
Terminal:
  Emulator: Windows Terminal (windows_terminal)
  DECSTBM (Margins): Yes
  DECSLRM (Cols): No
  Split Mode: decstbm
  Native Split: wt.exe -w 0 sp -H -s 0.30
```

#### 2. Terminal Health Audit: `forgum checkhealth`
Run `forgum checkhealth` for a rapid VT and protocol verification:
```bash
forgum checkhealth
```

#### Common Issues & Recipes

1. **Scrambled text on Windows PowerShell / CMD without Windows Terminal**:
   - Cause: You are running inside legacy `conhost.exe`.
   - Fix: Upgrade to [Windows Terminal](https://github.com/microsoft/terminal), or run with `--split-mode precmd-fallback`.
2. **Kitty native split fails (`kitty @ launch`)**:
   - Cause: Kitty remote control is disabled by default.
   - Fix: Add `allow_remote_control yes` and `listen_on unix:/tmp/mykitty` to your `kitty.conf`.
3. **tmux split does not clear margin**:
   - Cause: In rare cases with older tmux versions, margins require resetting.
   - Fix: Run `reset` or `printf '\033[r'` to restore full-window scrolling.

## Troubleshooting

### Cow doesn't appear

1. Verify hook is loaded: `type forgum` (bash/zsh) or `Get-Command forgum` (pwsh)
2. Check engine path: `which forgum` or `Get-Command forgum`
3. Test manually: `forgum render --text "hello" --duration 3`

### Dead daemon overlay not cleaning up

1. Check runtime directory exists: `ls $XDG_RUNTIME_DIR/forgum/` or `$env:TEMP\Forgum\`
2. Verify daemon.json has stale PID: `cat $XDG_RUNTIME_DIR/forgum/daemon.json`
3. Manual cleanup: `rm -f $XDG_RUNTIME_DIR/forgum/daemon.json`

### Slow prompt

The `__forgum_precmd` function runs on every prompt. It's designed to be fast:
- Checks if `daemon.json` exists (single `test -f`)
- If exists, reads PID with `awk` (fast, no parsing)
- If PID is dead, cleans up (rare case)

If still slow, check if `$COLUMNS` is set (some terminals don't set it).

### WSL-specific issues

See [WSL Guide](WSL-GUIDE.md) for WSL-specific troubleshooting.
