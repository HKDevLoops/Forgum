# WSL Guide

Forgum works in Windows Subsystem for Linux (WSL). This guide covers installation, configuration, and known issues.

## Installation Options

### Option 1: Native Linux Build (Recommended)

Build Forgum directly inside WSL:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install dependencies
sudo apt-get update
sudo apt-get install -y pkg-config libfontconfig1-dev

# Clone and build
git clone https://github.com/HKDevLoops/Forgum.git
cd Forgum
cargo build --workspace

# Install (optional)
cargo install --path crates/engine
```

### Option 2: Windows Binary via WSL Interop

Use the Windows binary from WSL:

```bash
# Add Windows binary to PATH
export PATH="$PATH:/mnt/c/Program Files/Forgum"

# Or create a symlink
sudo ln -s /mnt/c/Program\ Files/Forgum/forgum-engine.exe /usr/local/bin/forgum-engine
```

**Caveat:** Windows binaries may have different path handling (backslashes vs forward slashes).

### Option 3: Pre-built Binary

Download the Linux binary from GitHub releases:

```bash
# Download
curl -sL https://github.com/HKDevLoops/Forgum/releases/latest/download/forgum-engine-x86_64-unknown-linux-gnu.tar.gz | tar xz

# Install
sudo install forgum-engine /usr/local/bin/
```

## Shell Hook Setup

### Bash (default in most WSL distros)

```bash
eval "$(forgum-engine init bash)"
```

Add to `~/.bashrc` for persistence.

### Zsh

```bash
eval "$(forgum-engine init zsh)"
```

Add to `~/.zshrc` for persistence.

### Fish

```fish
forgum-engine init fish | source
```

Add to `~/.config/fish/config.fish` for persistence.

## WSL1 vs WSL2

| Feature | WSL1 | WSL2 |
|---------|------|------|
| File system performance | Faster `/mnt/c` access | Slower `/mnt/c` access |
| System calls | Translated | Native Linux |
| systemd | Not supported | Supported (Ubuntu 24.04+) |
| GUI apps | Not supported | Supported (with WSLg) |

**Recommendation:** Use WSL2 for better compatibility. The performance difference for `/mnt/c` is negligible for Forgum's use case.

## Known Issues

### `/mnt/c` Path Performance

WSL2 has slower access to Windows files via `/mnt/c`. If Forgum is installed on Windows:
- Config file access may be slightly slower
- Cow file loading may have latency

**Solution:** Install Forgum natively in WSL for best performance.

### systemd

If using systemd (WSL2 with Ubuntu 24.04+), ensure the `forgum` service is properly managed:

```bash
# Check if systemd is active
systemctl is-active default.target

# If yes, you can create a user service for forgum
mkdir -p ~/.config/systemd/user
```

### DISPLAY Variable

WSLg sets `$DISPLAY` automatically. If you're using X11 forwarding:
- Forgum doesn't use GUI, so `$DISPLAY` doesn't matter
- Terminal emulator must support ANSI escape sequences

### Terminal Emulator

Best WSL terminal emulators for Forgum:
- **Windows Terminal** — Full support (sync, truecolor)
- **WezTerm** — Full support (sync, truecolor, sixel)
- **VS Code** — Good support (sync, truecolor)
- **Alacritty** — Good support (sync, truecolor)

Avoid:
- **cmd.exe** — Limited ANSI support
- **PowerShell 5.1** — No truecolor

### tmux in WSL

If using tmux in WSL:

```bash
# Install tmux
sudo apt-get install tmux

# Generate config
forgum-engine tmux install >> ~/.tmux.conf

# Reload
tmux source-file ~/.tmux.conf
```

**Note:** tmux DCS passthrough works in WSL with Windows Terminal.

## Testing WSL Integration

Run tests directly:

```bash
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

This will:
1. Checks Rust installation
2. Builds Forgum
3. Runs all tests
4. Generates and validates shell hooks
5. Reports results

## Docker in WSL

If using Docker Desktop with WSL2 integration:

```bash
# Enable Docker integration in Docker Desktop settings
# Then test Forgum in Docker
docker build -t forgum-test -f packaging/containers/Containerfile.linux .
docker run --rm forgum-test
```

## Troubleshooting

### "command not found: forgum-engine"

1. Check installation: `which forgum-engine`
2. If using Windows binary: `ls /mnt/c/Program\ Files/Forgum/forgum-engine.exe`
3. Add to PATH: `export PATH="$PATH:/usr/local/bin"`

### "Permission denied"

1. Check binary permissions: `ls -la $(which forgum-engine)`
2. Fix: `chmod +x $(which forgum-engine)`

### "No cows found"

1. Check data directory: `ls ~/.config/forgum/cows/` or `ls data/Cows/`
2. If missing, built-in cows are embedded in the binary (109+ creatures)

### "Terminal does not support sync"

This is expected in some WSL configurations. Forgum will fall back to regular ANSI output.

To force sync in Windows Terminal:
```bash
export WT_SESSION=1
```
