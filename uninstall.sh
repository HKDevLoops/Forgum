#!/usr/bin/env bash
#
# uninstall.sh - One-command celestial uninstaller for Forgum.
#
# Safely de-orbits Forgum from your environment with two distinct methods:
#   Method 1: Soft Uninstall (Default)
#     - Removes binary from disk and PATH
#     - Strips shell hooks and completions from profiles (Bash, Zsh, Fish, Nushell)
#     - Deletes completions directory (~/.config/forgum/completions)
#     - PRESERVES user configuration (~/.config/forgum) and custom cows
#   Method 2: Purge Uninstall
#     - Complete clean slate removal
#     - Deletes binary, PATH, shell hooks, completions
#     - PERMANENTLY deletes configuration, cache, logs, daemons, and state
#
# Usage:
#   ./uninstall.sh [--method soft|purge] [--yes] [--tui]
#
# Examples:
#   ./uninstall.sh
#   ./uninstall.sh --method purge --yes

set -euo pipefail

METHOD="soft"
YES=0
TUI=0

while [ $# -gt 0 ]; do
  case "$1" in
    --method|-m)
      [ $# -ge 2 ] || { echo "error: --method requires an argument (soft|purge)" >&2; exit 2; }
      METHOD="$(echo "$2" | tr '[:upper:]' '[:lower:]')"; shift 2 ;;
    --yes|-y) YES=1; shift ;;
    --tui) TUI=1; shift ;;
    -h|--help)
      grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "error: unknown argument '$1' (try --help)" >&2; exit 2 ;;
  esac
done

# Locate binary if available
BIN_PATH=""
if command -v forgum >/dev/null 2>&1; then
  BIN_PATH="$(command -v forgum)"
elif command -v forgum-engine >/dev/null 2>&1; then
  BIN_PATH="$(command -v forgum-engine)"
elif [ -f "$HOME/.local/bin/forgum" ]; then
  BIN_PATH="$HOME/.local/bin/forgum"
elif [ -f "$HOME/.local/bin/forgum-engine" ]; then
  BIN_PATH="$HOME/.local/bin/forgum-engine"
elif [ -f "/usr/local/bin/forgum" ]; then
  BIN_PATH="/usr/local/bin/forgum"
elif [ -f "/usr/local/bin/forgum-engine" ]; then
  BIN_PATH="/usr/local/bin/forgum-engine"
fi

# If TUI requested or interactive without --yes, launch TUI wizard if binary exists
if { [ "$TUI" -eq 1 ] || [ "$YES" -eq 0 ]; } && [ -t 0 ] && [ -t 1 ] && [ -n "$BIN_PATH" ] && [ -x "$BIN_PATH" ]; then
  exec "$BIN_PATH" uninstall --tui
fi

echo "━━━ Forgum De-Orbit Uninstaller ━━━"
echo "Method: $METHOD"

if [ "$YES" -eq 0 ]; then
  printf "Are you sure you want to proceed with %s uninstallation? [y/N] " "$METHOD"
  read -r response
  case "$response" in
    [yY][eE][sS]|[yY]) ;;
    *) echo "Uninstallation cancelled."; exit 0 ;;
  esac
fi

# If binary is available, delegate to the engine uninstaller for complete platform-native uninstallation
if [ -n "$BIN_PATH" ] && [ -x "$BIN_PATH" ]; then
  "$BIN_PATH" uninstall --method "$METHOD" --yes
  exit 0
fi

# Fallback: Binary is missing, perform manual script-based cleanup across all 15 shells
# 1. Clean shell rc files
RC_FILES=(
  "$HOME/.bashrc"
  "$HOME/.bash_profile"
  "$HOME/.profile"
  "$HOME/.zshrc"
  "$HOME/.zshenv"
  "$HOME/.config/fish/config.fish"
  "$HOME/.config/nushell/config.nu"
  "$HOME/.config/elvish/rc.elv"
  "$HOME/.xonshrc"
  "$HOME/.cshrc"
  "$HOME/.tcshrc"
  "$HOME/.kshrc"
  "$HOME/.yashrc"
  "$HOME/.config/ion/initrc"
  "$HOME/.config/oil/oshrc"
  "$HOME/.config/powershell/Microsoft.PowerShell_profile.ps1"
)

for rc in "${RC_FILES[@]}"; do
  if [ -f "$rc" ]; then
    if grep -q ">>> forgum" "$rc" 2>/dev/null; then
      sed -i.bak '/# >>> forgum/,/# <<< forgum/d' "$rc" 2>/dev/null || sed -i '' '/# >>> forgum/,/# <<< forgum/d' "$rc" 2>/dev/null || true
      rm -f "${rc}.bak" 2>/dev/null || true
      echo "✓ Removed hooks from $rc"
    fi
  fi
done

# 2. Remove completions directory
COMP_DIR="$HOME/.config/forgum/completions"
if [ -d "$COMP_DIR" ]; then
  rm -rf "$COMP_DIR"
  echo "✓ Removed completions directory $COMP_DIR"
fi

# 3. Remove binaries
BIN_CANDIDATES=(
  "$HOME/.local/bin/forgum-engine"
  "$HOME/.local/bin/forgum"
  "$HOME/bin/forgum-engine"
  "$HOME/bin/forgum"
  "/usr/local/bin/forgum-engine"
  "/usr/local/bin/forgum"
)

for b in "${BIN_CANDIDATES[@]}"; do
  if [ -f "$b" ]; then
    rm -f "$b" 2>/dev/null || sudo rm -f "$b" 2>/dev/null || true
    echo "✓ Deleted binary $b"
  fi
done

# 4. Method handling: Soft vs Purge
if [ "$METHOD" = "purge" ]; then
  echo ">> Executing Purge: Deleting user configurations, logs, and cache..."
  rm -rf "$HOME/.config/forgum"
  rm -rf "$HOME/.cache/forgum"
  rm -rf "$HOME/.local/state/forgum"
  rm -rf "$HOME/.local/share/forgum"
  echo
  echo "✓ Purge complete. Zero trace of Forgum remains on this machine."
else
  echo
  echo "★ Soft uninstall complete."
  echo "✓ Preserved user configuration in ~/.config/forgum/ so your custom settings are saved if you reinstall."
fi
