#!/usr/bin/env bash
#
# install.sh - One-command celestial installer for Forgum.
#
# Detects OS/arch, installs forgum-engine and forgum into ~/.local/bin or
# /usr/local/bin, ensures the directory is on PATH, and launches the rich
# Celestial Terminal UI Wizard for interactive configuration, shell integration,
# and transparent motivation telemetry consent.
#
# Usage:
#   ./install.sh [--version X.Y.Z] [--headless] [--telemetry allow|decline]
#
# Environment overrides:
#   FORGUM_VERSION      release version to install (e.g. 0.4.0)
#   FORGUM_INSTALL_DIR  explicit install directory (must be on PATH)
#   FORGUM_REPO         owner/name of the GitHub repo (default: HKDevLoops/Forgum)
#
# Examples:
#   ./install.sh                 # interactive celestial installer
#   ./install.sh --headless      # non-interactive installation

set -euo pipefail

REPO="${FORGUM_REPO:-HKDevLoops/Forgum}"
VERSION_OVERRIDE=""
HEADLESS=0
TELEMETRY=""
FIRST_RUN=0

# --- argument parsing -------------------------------------------------------
while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      [ $# -ge 2 ] || { echo "error: --version requires an argument" >&2; exit 2; }
      VERSION_OVERRIDE="$2"; shift 2 ;;
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
echo ">> Installing Forgum ${VERSION} from ${REPO} (${TAG})"

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

# --- install location -------------------------------------------------------
if [ -n "${FORGUM_INSTALL_DIR:-}" ]; then
  INSTALL_DIR="$FORGUM_INSTALL_DIR"
elif [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
else
  INSTALL_DIR="$HOME/.local/bin"
fi

mkdir -p "$INSTALL_DIR"
BIN_PATH="$INSTALL_DIR/forgum-engine"
ALIAS_PATH="$INSTALL_DIR/forgum"

# Check for local build first (fast developer install)
if [ -f "target/release/forgum" ]; then
  install -m 0755 "target/release/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$ALIAS_PATH"
  echo ">> Installed from local release build: $BIN_PATH"
elif [ -f "target/debug/forgum" ]; then
  install -m 0755 "target/debug/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$ALIAS_PATH"
  echo ">> Installed from local debug build: $BIN_PATH"
else
  ASSET="forgum-${VERSION}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
  URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
  echo ">> Downloading ${ASSET}"

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
    EXTRACTED="$(find "$TMP" -maxdepth 2 -type f -name 'forgum-engine' -o -name 'forgum' | head -n1)"
    if [ -n "$EXTRACTED" ]; then
      install -m 0755 "$EXTRACTED" "$BIN_PATH"
      ln -sf "$BIN_PATH" "$ALIAS_PATH"
      echo ">> Installed: $BIN_PATH"
    else
      DOWNLOAD_OK=0
    fi
  fi

  if [ "$DOWNLOAD_OK" -eq 0 ]; then
    if command -v cargo >/dev/null 2>&1; then
      echo ">> Release asset unavailable for ${TARGET_ARCH}-${TARGET_OS}. Compiling via Cargo..."
      if [ -f "Cargo.toml" ]; then
        cargo build --release --bin forgum-engine --bin forgum
        install -m 0755 "target/release/forgum" "$BIN_PATH"
        ln -sf "$BIN_PATH" "$ALIAS_PATH"
      else
        cargo install forgum-cli --root "$HOME/.local"
        BIN_PATH="$HOME/.local/bin/forgum-engine"
        ALIAS_PATH="$HOME/.local/bin/forgum"
      fi
      echo ">> Cargo build complete: $BIN_PATH"
    else
      echo "error: prebuilt archive not found at $URL and cargo is not installed." >&2
      exit 1
    fi
  fi
fi

# --- check & update PATH ----------------------------------------------------
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ">> Adding $INSTALL_DIR to PATH..."
    PATH_BLOCK="# >>> forgum path >>>\nexport PATH=\"${INSTALL_DIR}:\$PATH\"\n# <<< forgum path <<<"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile"; do
      if [ -f "$rc" ]; then
        if ! grep -q "forgum path" "$rc"; then
          printf "\n%b\n" "$PATH_BLOCK" >> "$rc"
          echo ">> Added PATH export to $rc"
        fi
      fi
    done
    ;;
esac

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
