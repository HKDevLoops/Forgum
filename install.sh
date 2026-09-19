#!/usr/bin/env bash
#
# install.sh - One-command celestial installer for Forgum.
#
# Detects OS/arch, runs preflight terminal diagnostics (TrueColor, UTF-8),
# detects existing package manager installations (Homebrew, Pacman, Apt, Nix, Cargo),
# reconciles conflicts interactively, installs forgum into ~/.local/bin or /usr/local/bin,
# ensures the directory is on PATH, records receipt tracking, and launches the rich
# Celestial Terminal UI Wizard for interactive configuration, shell integration,
# channel selection, and transparent motivation telemetry consent.
#
# Usage:
#   ./install.sh [--version X.Y.Z] [--channel stable|nightly|dev] [--headless] [--telemetry allow|decline]
#
# Environment overrides:
#   FORGUM_VERSION      release version to install (e.g. 0.4.0)
#   FORGUM_CHANNEL      release channel stream (default: stable)
#   FORGUM_INSTALL_DIR  explicit install directory (must be on PATH)
#   FORGUM_REPO         owner/name of the GitHub repo (default: HKDevLoops/Forgum)
#
# Examples:
#   ./install.sh                    # interactive celestial installer (stable)
#   ./install.sh --channel nightly  # install nightly channel build
#   ./install.sh --headless         # non-interactive installation

set -euo pipefail

REPO="${FORGUM_REPO:-HKDevLoops/Forgum}"
VERSION_OVERRIDE=""
CHANNEL="${FORGUM_CHANNEL:-stable}"
HEADLESS=0
TELEMETRY=""
FIRST_RUN=0

# --- argument parsing -------------------------------------------------------
while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      [ $# -ge 2 ] || { echo "error: --version requires an argument" >&2; exit 2; }
      VERSION_OVERRIDE="$2"; shift 2 ;;
    --channel)
      [ $# -ge 2 ] || { echo "error: --channel requires an argument (stable|nightly|dev)" >&2; exit 2; }
      CHANNEL="$2"; shift 2 ;;
    --headless) HEADLESS=1; shift ;;
    --telemetry)
      [ $# -ge 2 ] || { echo "error: --telemetry requires an argument (allow|decline)" >&2; exit 2; }
      TELEMETRY="$2"; shift 2 ;;
    --first-run) FIRST_RUN=1; shift ;;
    -h|--help)
      grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "error: unknown argument '$1' (try --help)" >&2; exit 2 ;;
  esac
done

# --- banner -----------------------------------------------------------------
echo -e "\033[35m✦ FORGUM CELESTIAL INSTALLER ✦\033[0m"
echo -e "\033[90mIdempotent, non-destructive installer with conflict reconciliation\033[0m"

# --- terminal preflight diagnostics -----------------------------------------
HAS_TRUECOLOR=0
if [ "${COLORTERM:-}" = "truecolor" ] || [ "${COLORTERM:-}" = "24bit" ] || \
   [ -n "${KITTY_WINDOW_ID:-}" ] || [ -n "${WEZTERM_PANE:-}" ] || [ -n "${ALACRITTY_LOG:-}" ] || \
   [ -n "${GHOSTTY_RESOURCES_DIR:-}" ] || [ "${TERM_PROGRAM:-}" = "iTerm.app" ] || [ "${TERM_PROGRAM:-}" = "ghostty" ]; then
  HAS_TRUECOLOR=1
fi

HAS_UTF8=0
LOCALE_VARS="${LC_ALL:-} ${LC_CTYPE:-} ${LANG:-}"
if echo "$LOCALE_VARS" | grep -qi -E 'utf-8|utf8'; then
  HAS_UTF8=1
elif [ -n "${TERM_PROGRAM:-}" ] || [ -n "${COLORTERM:-}" ]; then
  HAS_UTF8=1
fi

echo -e "\033[36m>> Preflight Terminal Diagnostics:\033[0m"
if [ "$HAS_TRUECOLOR" -eq 1 ]; then
  echo -e "  \033[32m[✓] 24-bit TrueColor display detected\033[0m"
else
  echo -e "  \033[33m[!] TrueColor not detected (256-color fallback will be used)\033[0m"
fi

if [ "$HAS_UTF8" -eq 1 ]; then
  echo -e "  \033[32m[✓] UTF-8 encoding active\033[0m"
else
  echo -e "  \033[33m[!] Locale does not specify UTF-8 (cosmic box-drawing glyphs may degrade)\033[0m"
fi

# --- ffmpeg optional recording check ----------------------------------------
if command -v ffmpeg >/dev/null 2>&1; then
  echo -e "  \033[32m[✓] ffmpeg detected (mascot recording and video exports active)\033[0m"
else
  echo -e "  \033[90m[-] ffmpeg not detected (optional)\033[0m"
  echo -e "      \033[90mTip: Install via your package manager ('brew install ffmpeg' or 'apt install ffmpeg') to record animations.\033[0m"
fi

# --- install location -------------------------------------------------------
if [ -n "${FORGUM_INSTALL_DIR:-}" ]; then
  INSTALL_DIR="$FORGUM_INSTALL_DIR"
elif [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.local/bin"
fi

mkdir -p "$INSTALL_DIR"
BIN_PATH="$INSTALL_DIR/forgum"
LEGACY_PATH="$INSTALL_DIR/forgum-engine"

# --- detect existing package manager installations & shadow binaries --------
EXISTING_PM=""
EXISTING_PATH=""
EXISTING_UPDATE_CMD=""

if command -v brew >/dev/null 2>&1 && brew list forgum >/dev/null 2>&1; then
  EXISTING_PM="Homebrew"
  EXISTING_PATH="$(brew --prefix forgum 2>/dev/null || echo 'Homebrew cellar')"
  EXISTING_UPDATE_CMD="brew upgrade forgum"
elif command -v pacman >/dev/null 2>&1 && pacman -Q forgum >/dev/null 2>&1; then
  EXISTING_PM="Pacman"
  EXISTING_PATH="/usr/bin/forgum"
  EXISTING_UPDATE_CMD="sudo pacman -Syu forgum"
elif command -v dpkg >/dev/null 2>&1 && dpkg -s forgum >/dev/null 2>&1; then
  EXISTING_PM="Apt"
  EXISTING_PATH="/usr/bin/forgum"
  EXISTING_UPDATE_CMD="sudo apt update && sudo apt install --only-upgrade forgum"
elif command -v nix-env >/dev/null 2>&1 && nix-env -q forgum 2>/dev/null | grep -q forgum; then
  EXISTING_PM="Nix"
  EXISTING_PATH="nix profile"
  EXISTING_UPDATE_CMD="nix profile upgrade forgum"
elif [ -f "$HOME/.cargo/bin/forgum" ]; then
  EXISTING_PM="Cargo"
  EXISTING_PATH="$HOME/.cargo/bin/forgum"
  EXISTING_UPDATE_CMD="cargo install --force forgum-cli"
elif command -v forgum >/dev/null 2>&1; then
  ACTIVE_BIN="$(command -v forgum)"
  if [ "$ACTIVE_BIN" != "$BIN_PATH" ]; then
    EXISTING_PM="PATH Shadow"
    EXISTING_PATH="$ACTIVE_BIN"
    EXISTING_UPDATE_CMD="Replace $ACTIVE_BIN"
  fi
fi

# --- interactive conflict reconciliation menu -------------------------------
if [ -n "$EXISTING_PM" ] && [ "$HEADLESS" -eq 0 ] && [ -t 0 ] && [ -t 1 ]; then
  echo ""
  echo -e "\033[33m⚠ Package Manager Conflict Detected!\033[0m"
  echo -e "  • Found existing Forgum via \033[36m${EXISTING_PM}\033[0m: ${EXISTING_PATH}"
  echo ""
  echo -e "How would you like to proceed?"
  echo -e "  \033[32m[1] Delegate to package manager (Run '${EXISTING_UPDATE_CMD}' and exit)\033[0m"
  echo -e "  \033[36m[2] Switch to standalone channel build (Install to ${INSTALL_DIR} and update PATH)\033[0m"
  echo -e "  \033[33m[3] Clean reinstall / overwrite\033[0m"
  echo ""
  read -r -p "Select option [1/2/3] (default: 2): " CHOICE || true
  CHOICE="${CHOICE:-2}"
  if [ "$CHOICE" = "1" ]; then
    echo -e "\033[32m>> Delegating update to ${EXISTING_PM}...\033[0m"
    eval "$EXISTING_UPDATE_CMD"
    exit 0
  fi
fi

# --- resolve version --------------------------------------------------------
if [ -n "$VERSION_OVERRIDE" ]; then
  VERSION="$VERSION_OVERRIDE"
elif [ -n "${FORGUM_VERSION:-}" ]; then
  VERSION="$FORGUM_VERSION"
elif [ -f "Cargo.toml" ]; then
  VERSION="$(grep '^version' Cargo.toml | head -n1 | sed -E 's/version = "([^"]+)"/\1/')"
else
  # Fall back to latest GitHub release tag.
  VERSION="$(curl -fsSL -H "User-Agent: forgum-install" \
    "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name":' | head -n1 | sed -E 's/.*"v([^"]+)".*/\1/')"
fi

if [ -z "$VERSION" ]; then
  echo "error: could not determine version. Pass --version or run inside repo." >&2
  exit 1
fi

TAG="v${VERSION}"
echo -e "\033[36m>> Installing Forgum ${VERSION} (${CHANNEL} channel) from ${REPO} (${TAG})\033[0m"

# --- detect OS and architecture ---------------------------------------------
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    if ldd --version 2>&1 | grep -qi musl || ls -l /lib/ld-musl* 2>/dev/null | grep -q musl; then
      TARGET_OS="unknown-linux-musl"
    else
      TARGET_OS="unknown-linux-gnu"
    fi
    ;;
  darwin) TARGET_OS="apple-darwin" ;;
  freebsd) TARGET_OS="unknown-freebsd" ;;
  *) echo "error: unsupported OS '$OS' (install.sh supports linux, darwin, freebsd; use install.ps1 on Windows)" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) TARGET_ARCH="x86_64" ;;
  aarch64|arm64) TARGET_ARCH="aarch64" ;;
  armv7*|armhf) TARGET_ARCH="armv7" ;;
  riscv64*) TARGET_ARCH="riscv64gc" ;;
  i686|i386) TARGET_ARCH="i686" ;;
  *) echo "error: unsupported architecture '$ARCH'" >&2; exit 1 ;;
esac

# Check for local build first (fast developer install)
if [ -f "target/release/forgum" ]; then
  install -m 0755 "target/release/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$LEGACY_PATH"
  echo -e "\033[32m>> Installed from local release build: $BIN_PATH\033[0m"
elif [ -f "target/debug/forgum" ]; then
  install -m 0755 "target/debug/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$LEGACY_PATH"
  echo -e "\033[32m>> Installed from local debug build: $BIN_PATH\033[0m"
else
  ASSET="forgum-${VERSION}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
  URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
  echo -e "\033[36m>> Downloading ${ASSET}\033[0m"

  TMP="$(mktemp -d 2>/dev/null || mktemp -d -t 'forgum')"
  trap 'rm -rf "$TMP"' EXIT

  DOWNLOAD_OK=0
  if command -v curl >/dev/null 2>&1; then
    if curl -fsSL -H "User-Agent: forgum-install" "$URL" -o "$TMP/$ASSET" 2>/dev/null; then
      DOWNLOAD_OK=1
    fi
  elif command -v wget >/dev/null 2>&1; then
    if wget -q "$URL" -O "$TMP/$ASSET" 2>/dev/null; then
      DOWNLOAD_OK=1
    fi
  fi

  if [ "$DOWNLOAD_OK" -eq 1 ] && [ -s "$TMP/$ASSET" ]; then
    tar xzf "$TMP/$ASSET" -C "$TMP"
    EXTRACTED="$(find "$TMP" -maxdepth 2 -type f -name 'forgum' -o -name 'forgum-engine' | head -n1)"
    if [ -n "$EXTRACTED" ]; then
      install -m 0755 "$EXTRACTED" "$BIN_PATH"
      ln -sf "$BIN_PATH" "$LEGACY_PATH"
      echo -e "\033[32m>> Installed: $BIN_PATH\033[0m"
    else
      DOWNLOAD_OK=0
    fi
  fi

  if [ "$DOWNLOAD_OK" -eq 0 ]; then
    if command -v cargo >/dev/null 2>&1; then
      echo -e "\033[33m>> Release asset unavailable for ${TARGET_ARCH}-${TARGET_OS}. Compiling via Cargo...\033[0m"
      if [ -f "Cargo.toml" ]; then
        cargo build --release --bin forgum --bin forgum-engine
        install -m 0755 "target/release/forgum" "$BIN_PATH"
        ln -sf "$BIN_PATH" "$LEGACY_PATH"
      else
        cargo install forgum-cli --root "$HOME/.local"
        BIN_PATH="$HOME/.local/bin/forgum"
        LEGACY_PATH="$HOME/.local/bin/forgum-engine"
      fi
      echo -e "\033[32m>> Cargo build complete: $BIN_PATH\033[0m"
    else
      echo "error: prebuilt archive not found at $URL and cargo is not installed." >&2
      exit 1
    fi
  fi
fi

# --- check & update PATH (idempotent prepend) --------------------------------
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo -e "\033[32m>> Adding $INSTALL_DIR to PATH...\033[0m"
    PATH_BLOCK="# >>> forgum path >>>\nexport PATH=\"${INSTALL_DIR}:\$PATH\"\n# <<< forgum path <<<"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile"; do
      if [ -f "$rc" ]; then
        if ! grep -q "forgum path" "$rc"; then
          printf "\n%b\n" "$PATH_BLOCK" >> "$rc"
          echo -e "\033[32m>> Added PATH export to $rc\033[0m"
        fi
      fi
    done
    ;;
esac

# --- record installation receipt --------------------------------------------
RECEIPT_DIR="$HOME/.config/forgum"
mkdir -p "$RECEIPT_DIR"
NOW_TS="$(date +%s 2>/dev/null || echo 0)"
cat > "$RECEIPT_DIR/install_receipt.json" <<EOF
{
  "installer_source": "standalone",
  "channel": "${CHANNEL}",
  "version": "${VERSION}",
  "bin_path": "${BIN_PATH}",
  "installed_at": ${NOW_TS},
  "auto_update": true
}
EOF

# --- launch wizard or headless setup ---------------------------------------
if [ "$HEADLESS" -eq 1 ] || [ ! -t 0 ] || [ ! -t 1 ]; then
  TELEM_ARG=""
  if [ -n "$TELEMETRY" ]; then
    TELEM_ARG="--telemetry $TELEMETRY"
  fi
  "$BIN_PATH" install --headless $TELEM_ARG
else
  # Launch interactive Celestial Terminal UI Wizard
  "$BIN_PATH" install
fi
