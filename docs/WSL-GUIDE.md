# WSL Guide

Forgum works in Windows Subsystem for Linux (WSL). This guide covers installation, configuration, and known issues.

### Option 1: Universal One-Command Installer (Recommended)

Run the celestial installer directly inside your WSL terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/HKDevLoops/Forgum/dev/install.sh | bash
```

The installer automatically:
1. Detects your WSL2 environment and active distribution (`openSUSE`, `Ubuntu`, `Debian`, `Kali`, `Arch`, `Fedora`, `Alpine`, etc.).
2. Identifies the distribution package manager (`zypper`, `apt-get`, `pacman`, `dnf`, `apk`).
3. Prompts for root/sudo credentials if build or system utilities (`tar`, `rust`, `cargo`, `gcc`, `git`) are missing, automatically installing them via your distribution's native package manager.
4. Resiliently resolves release versions without 404 aborts, downloading prebuilt binaries or compiling seamlessly from source.
5. Adds Forgum to your `$PATH` and launches the celestial setup wizard.

### Option 2: Native Linux Build (Manual)

Build Forgum manually inside WSL:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install dependencies (Ubuntu/Debian)
sudo apt-get update && sudo apt-get install -y build-essential git

# Or for openSUSE:
sudo zypper in rust cargo gcc git

# Clone and build
git clone https://github.com/HKDevLoops/Forgum.git
cd Forgum
cargo build --release --bin forgum
sudo install -m 0755 target/release/forgum /usr/local/bin/forgum
```

### Option 3: Windows Binary via WSL Interop

Use the Windows binary from WSL:

```bash
# Add Windows binary to PATH
export PATH="$PATH:/mnt/c/Program Files/Forgum"

# Or create a symlink
sudo ln -s /mnt/c/Program\ Files/Forgum/forgum.exe /usr/local/bin/forgum
```

**Caveat:** Windows binaries may have different path handling (backslashes vs forward slashes).

### Option 4: Pre-built Binary

Download the Linux binary from GitHub releases:

```bash
# Download
curl -sL https://github.com/HKDevLoops/Forgum/releases/latest/download/forgum-x86_64-unknown-linux-gnu.tar.gz | tar xz

# Install
sudo install forgum /usr/local/bin/
```

## Shell Hook Setup

### Bash (default in most WSL distros)

```bash
eval "$(forgum init bash)"
```

Add to `~/.bashrc` for persistence.

### Zsh

```bash
eval "$(forgum init zsh)"
```

Add to `~/.zshrc` for persistence.

### Fish

```fish
forgum init fish | source
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
forgum tmux install >> ~/.tmux.conf

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

### "command not found: forgum"

1. Check installation: `which forgum`
2. If using Windows binary: `ls /mnt/c/Program\ Files/Forgum/forgum.exe`
3. Add to PATH: `export PATH="$PATH:/usr/local/bin"`

### "Permission denied"

1. Check binary permissions: `ls -la $(which forgum)`
2. Fix: `chmod +x $(which forgum)`

### "No cows found"

1. Check data directory: `ls ~/.config/forgum/cows/` or `ls data/Cows/`
2. If missing, built-in cows are embedded in the binary (109+ creatures)

### "Terminal does not support sync"

This is expected in some WSL configurations. Forgum will fall back to regular ANSI output.

To force sync in Windows Terminal:
```bash
export WT_SESSION=1
```
