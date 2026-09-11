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
