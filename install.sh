#!/usr/bin/env bash
#
# install.sh - One-command installer for forgum-engine (Phase 7).
#
# Detects OS/arch, downloads the correct prebuilt forgum-engine binary from the
# GitHub releases page (tag derived from Cargo.toml, or override with --version),
# and installs it to a sensible bin directory.
#
# Usage:
#   ./install.sh [--version X.Y.Z] [--first-run]
#
# Environment overrides:
#   FORGUM_VERSION      release version to install (e.g. 0.4.0)
#   FORGUM_INSTALL_DIR  explicit install directory (must be on PATH)
#   FORGUM_REPO         owner/name of the GitHub repo (default: HKDevLoops/Forgum)
#
# Examples:
#   ./install.sh                 # install latest in Cargo.toml
#   ./install.sh --version 0.4.0 # pin a version
#   FORGUM_INSTALL_DIR=~/bin ./install.sh

set -euo pipefail

REPO="${FORGUM_REPO:-HKDevLoops/Forgum}"
ASSET_PREFIX="forgum-engine"
FIRST_RUN=0

# --- argument parsing -------------------------------------------------------
VERSION_OVERRIDE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --version)
      [ $# -ge 2 ] || { echo "error: --version requires an argument" >&2; exit 2; }
      VERSION_OVERRIDE="$2"; shift 2 ;;
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
elif [ -f Cargo.toml ]; then
  VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')"
  if [ -z "$VERSION" ]; then
    echo "error: could not read version from Cargo.toml" >&2; exit 1
  fi
else
  echo "error: no --version given and no Cargo.toml found in $(pwd)" >&2; exit 1
fi

TAG="v${VERSION}"
echo ">> Installing forgum-engine ${VERSION} from ${REPO} (${TAG})"

# --- detect platform --------------------------------------------------------
OS="$(uname -s)"
ARCH="$(uname -m)"

TARGET=""
case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "error: unsupported Linux arch '$ARCH'" >&2; exit 1 ;;
    esac ;;
  Darwin)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-apple-darwin" ;;
      arm64)   TARGET="aarch64-apple-darwin" ;;
      *) echo "error: unsupported macOS arch '$ARCH'" >&2; exit 1 ;;
    esac ;;
  FreeBSD)
    case "$ARCH" in
      x86_64)  TARGET="x86_64-unknown-freebsd" ;;
      *) echo "error: unsupported FreeBSD arch '$ARCH'" >&2; exit 1 ;;
    esac ;;
  *)
    echo "error: unsupported OS '$OS' (windows users: run install.ps1)" >&2; exit 1 ;;
esac

# --- asset + directory ------------------------------------------------------
ASSET="${ASSET_PREFIX}-${TARGET}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

if [ -n "${FORGUM_INSTALL_DIR:-}" ]; then
  INSTALL_DIR="$FORGUM_INSTALL_DIR"
elif [ -w /usr/local/bin ]; then
  INSTALL_DIR="/usr/local/bin"
elif [ -w "$HOME" ]; then
  INSTALL_DIR="$HOME/bin"
else
  echo "error: no writable install dir (/usr/local/bin or ~/bin); set FORGUM_INSTALL_DIR" >&2; exit 1
fi

BIN_PATH="${INSTALL_DIR}/forgum-engine"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

# --- fetch ------------------------------------------------------------------
if command -v curl >/dev/null 2>&1; then
  echo ">> Downloading ${ASSET}"
  if ! curl -fL --retry 3 -o "$TMP/$ASSET" "$URL"; then
    echo "error: download failed: $URL" >&2; exit 1
  fi
elif command -v wget >/dev/null 2>&1; then
  echo ">> Downloading ${ASSET}"
  if ! wget -O "$TMP/$ASSET" "$URL"; then
    echo "error: download failed: $URL" >&2; exit 1
  fi
else
  echo "error: need curl or wget installed" >&2; exit 1
fi

# --- extract + install ------------------------------------------------------
echo ">> Extracting to ${INSTALL_DIR}"
mkdir -p "$INSTALL_DIR"
tar xzf "$TMP/$ASSET" -C "$TMP"
EXTRACTED="$(find "$TMP" -maxdepth 2 -type f -name 'forgum-engine' | head -n1)"
if [ -z "$EXTRACTED" ]; then
  echo "error: forgum-engine not found inside archive" >&2; exit 1
fi

install -m 0755 "$EXTRACTED" "$BIN_PATH"
echo ">> Installed: $BIN_PATH"

# --- next steps -------------------------------------------------------------
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo
    echo "!! ${INSTALL_DIR} is not on your PATH. Add it:"
    echo "     export PATH=\"${INSTALL_DIR}:\$PATH\""
    if [ "$INSTALL_DIR" = "$HOME/bin" ]; then
      echo "   (many shells add ~/bin automatically; reopen your terminal)"
    fi ;;
esac

# --- fun: fortune + shell hook ----------------------------------------------
echo
echo "=============================================="
echo "  Moooo! Wrapping up your forgum install..."
echo "=============================================="

echo
echo "🐮 Here's a fortune to chew on while we finish up:"
"$INSTALL_DIR/forgum-engine" fortune 2>/dev/null || true

echo
echo ">> Injecting the shell hook so forgum runs on every prompt..."
CURRENT_SHELL="bash"
if [ -n "${SHELL:-}" ]; then
  CURRENT_SHELL="$(basename "${SHELL}")"
elif command -v ps >/dev/null 2>&1; then
  DETECTED="$(ps -p $$ -o comm= 2>/dev/null | sed 's/^-//' | xargs -r basename 2>/dev/null || true)"
  [ -n "$DETECTED" ] && CURRENT_SHELL="$DETECTED"
fi
case "$CURRENT_SHELL" in
  zsh)  INIT_SHELL="zsh" ;;
  fish) INIT_SHELL="fish" ;;
  bash) INIT_SHELL="bash" ;;
  *)    INIT_SHELL="bash" ;;
esac
"$INSTALL_DIR/forgum-engine" init "$INIT_SHELL" 2>/dev/null || true

echo
echo "Done. Try: forgum-engine --help"
if [ "$FIRST_RUN" -eq 1 ]; then
  echo "First run: forgum-engine --daemon start"
fi

echo
echo "✨ Customize your cow anytime:  forgum-engine config --tui"
