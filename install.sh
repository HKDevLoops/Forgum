#!/usr/bin/env bash
#
# install.sh - One-command celestial installer for Forgum.
#
# Detects OS/arch, WSL runtime, Linux distribution, and native package manager
# (Zypper, APT, DNF/YUM, Pacman, APK, XBPS, Portage, Homebrew, FreeBSD pkg),
# automatically installs missing dependencies with elevated root privileges,
# reconciles conflicts interactively, downloads prebuilt binaries or compiles from source,
# ensures directories are on PATH, records receipt tracking, and launches the rich
# Celestial Terminal UI Wizard for interactive configuration.
#
# Usage:
#   ./install.sh [--version X.Y.Z] [--channel stable|nightly|dev] [--headless]
#                [--telemetry allow|decline] [-y|--yes] [--no-deps] [--build-from-source]
#
# Environment overrides:
#   FORGUM_VERSION      release version to install (e.g. 0.4.0)
#   FORGUM_CHANNEL      release channel stream (default: stable)
#   FORGUM_INSTALL_DIR  explicit install directory (must be on PATH)
#   FORGUM_REPO         owner/name of the GitHub repo (default: HKDevLoops/Forgum)
#   FORGUM_AUTO_DEPS    set to 1 to auto-install dependencies, 0 to skip (default: 1)
#
# Examples:
#   curl -fsSL https://raw.githubusercontent.com/HKDevLoops/Forgum/dev/install.sh | bash
#   ./install.sh --channel nightly
#   ./install.sh --headless -y
#

set -euo pipefail

REPO="${FORGUM_REPO:-HKDevLoops/Forgum}"
VERSION_OVERRIDE=""
CHANNEL="${FORGUM_CHANNEL:-stable}"
HEADLESS=0
TELEMETRY=""
FIRST_RUN=0
ASSUME_YES=0
AUTO_DEPS="${FORGUM_AUTO_DEPS:-1}"
BUILD_FROM_SOURCE=0

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
    -y|--yes) ASSUME_YES=1; shift ;;
    --no-deps) AUTO_DEPS=0; shift ;;
    --build-from-source|--source) BUILD_FROM_SOURCE=1; shift ;;
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

# ffmpeg optional recording check
if command -v ffmpeg >/dev/null 2>&1; then
  echo -e "  \033[32m[✓] ffmpeg detected (mascot recording and video exports active)\033[0m"
else
  echo -e "  \033[90m[-] ffmpeg not detected (optional)\033[0m"
  echo -e "      \033[90mTip: Install ffmpeg to enable high-fps mascot recording & export.\033[0m"
fi

# --- WSL, container, and platform / distro detection ------------------------
IS_CONTAINER=0
if [ -f /.dockerenv ] || [ -f /run/.containerenv ] || [ -f /run/systemd/container ] || [ -n "${container:-}" ]; then
  IS_CONTAINER=1
fi

IS_WSL=0
WSL_DISTRO=""
if [ "$IS_CONTAINER" -eq 0 ]; then
  if [ -n "${WSL_DISTRO_NAME:-}" ]; then
    IS_WSL=1
    WSL_DISTRO="${WSL_DISTRO_NAME}"
  elif [ -f /proc/sys/fs/binfmt_misc/WSLInterop ] || [ -d /run/WSL ]; then
    IS_WSL=1
    WSL_DISTRO="WSL"
  elif [ -f /proc/version ] && grep -qi -E 'microsoft|wsl' /proc/version 2>/dev/null; then
    IS_WSL=1
    WSL_DISTRO="WSL"
  fi
fi

RAW_OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

DISTRO_ID=""
DISTRO_NAME=""
DISTRO_VERSION=""
DISTRO_ID_LIKE=""

if [ -f /etc/os-release ]; then
  DISTRO_ID="$((grep -E '^ID=' /etc/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  DISTRO_NAME="$((grep -E '^PRETTY_NAME=' /etc/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  [ -z "$DISTRO_NAME" ] && DISTRO_NAME="$((grep -E '^NAME=' /etc/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  DISTRO_VERSION="$((grep -E '^VERSION_ID=' /etc/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  DISTRO_ID_LIKE="$((grep -E '^ID_LIKE=' /etc/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
elif [ -f /usr/lib/os-release ]; then
  DISTRO_ID="$((grep -E '^ID=' /usr/lib/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  DISTRO_NAME="$((grep -E '^PRETTY_NAME=' /usr/lib/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
  DISTRO_ID_LIKE="$((grep -E '^ID_LIKE=' /usr/lib/os-release 2>/dev/null || true) | head -n1 | cut -d= -f2 | tr -d '"'\' )"
elif [ -f /etc/SuSE-release ]; then
  DISTRO_ID="opensuse"
  DISTRO_NAME="openSUSE"
elif [ -f /etc/debian_version ]; then
  DISTRO_ID="debian"
  DISTRO_NAME="Debian $(cat /etc/debian_version 2>/dev/null)"
elif [ -f /etc/redhat-release ]; then
  DISTRO_ID="rhel"
  DISTRO_NAME="$(cat /etc/redhat-release 2>/dev/null)"
elif [ -f /etc/arch-release ]; then
  DISTRO_ID="arch"
  DISTRO_NAME="Arch Linux"
elif [ -f /etc/alpine-release ]; then
  DISTRO_ID="alpine"
  DISTRO_NAME="Alpine Linux $(cat /etc/alpine-release 2>/dev/null)"
fi

if [ -z "$DISTRO_NAME" ]; then
  case "$RAW_OS" in
    darwin) DISTRO_ID="darwin"; DISTRO_NAME="macOS $(sw_vers -productVersion 2>/dev/null || true)" ;;
    freebsd) DISTRO_ID="freebsd"; DISTRO_NAME="FreeBSD $(uname -r)" ;;
    linux) DISTRO_ID="linux"; DISTRO_NAME="Generic Linux" ;;
    *) DISTRO_ID="$RAW_OS"; DISTRO_NAME="$RAW_OS" ;;
  esac
fi

# Resolve native package manager
PACKAGE_MANAGER=""
PM_CMD=""

COMBINED_DISTRO="${DISTRO_ID} ${DISTRO_ID_LIKE}"
case "$COMBINED_DISTRO" in
  *opensuse*|*sles*|*suse*)
    if command -v zypper >/dev/null 2>&1; then
      PACKAGE_MANAGER="zypper"
      PM_CMD="$(command -v zypper)"
    fi
    ;;
  *debian*|*ubuntu*|*kali*|*pop*|*linuxmint*|*raspbian*)
    if command -v apt-get >/dev/null 2>&1; then
      PACKAGE_MANAGER="apt-get"
      PM_CMD="$(command -v apt-get)"
    fi
    ;;
  *fedora*|*rhel*|*centos*|*rocky*|*almalinux*)
    if command -v dnf >/dev/null 2>&1; then
      PACKAGE_MANAGER="dnf"
      PM_CMD="$(command -v dnf)"
    elif command -v yum >/dev/null 2>&1; then
      PACKAGE_MANAGER="yum"
      PM_CMD="$(command -v yum)"
    fi
    ;;
  *arch*|*manjaro*|*endeavouros*|*artix*)
    if command -v pacman >/dev/null 2>&1; then
      PACKAGE_MANAGER="pacman"
      PM_CMD="$(command -v pacman)"
    fi
    ;;
  *alpine*)
    if command -v apk >/dev/null 2>&1; then
      PACKAGE_MANAGER="apk"
      PM_CMD="$(command -v apk)"
    fi
    ;;
esac

# Fallback detection if distribution string was empty or generic
if [ -z "$PACKAGE_MANAGER" ]; then
  if command -v zypper >/dev/null 2>&1; then
    PACKAGE_MANAGER="zypper"; PM_CMD="$(command -v zypper)"
  elif command -v apt-get >/dev/null 2>&1; then
    PACKAGE_MANAGER="apt-get"; PM_CMD="$(command -v apt-get)"
  elif command -v dnf >/dev/null 2>&1; then
    PACKAGE_MANAGER="dnf"; PM_CMD="$(command -v dnf)"
  elif command -v yum >/dev/null 2>&1; then
    PACKAGE_MANAGER="yum"; PM_CMD="$(command -v yum)"
  elif command -v pacman >/dev/null 2>&1; then
    PACKAGE_MANAGER="pacman"; PM_CMD="$(command -v pacman)"
  elif command -v apk >/dev/null 2>&1; then
    PACKAGE_MANAGER="apk"; PM_CMD="$(command -v apk)"
  elif command -v xbps-install >/dev/null 2>&1; then
    PACKAGE_MANAGER="xbps"; PM_CMD="$(command -v xbps-install)"
  elif command -v emerge >/dev/null 2>&1; then
    PACKAGE_MANAGER="emerge"; PM_CMD="$(command -v emerge)"
  elif command -v brew >/dev/null 2>&1; then
    PACKAGE_MANAGER="brew"; PM_CMD="$(command -v brew)"
  elif command -v pkg >/dev/null 2>&1 && [ "$RAW_OS" = "freebsd" ]; then
    PACKAGE_MANAGER="pkg"; PM_CMD="$(command -v pkg)"
  fi
fi

echo -e "\033[36m>> Platform & Environment Diagnostics:\033[0m"
if [ "$IS_CONTAINER" -eq 1 ]; then
  echo -e "  \033[32m[✓] Environment: Container (Docker/Podman Sandbox)\033[0m"
elif [ "$IS_WSL" -eq 1 ]; then
  echo -e "  \033[32m[✓] Environment: WSL2 (${WSL_DISTRO})\033[0m"
else
  echo -e "  \033[32m[✓] Environment: Native (${RAW_OS})\033[0m"
fi
echo -e "  \033[32m[✓] Distribution: ${DISTRO_NAME} [${ARCH}]\033[0m"
if [ -n "$PACKAGE_MANAGER" ]; then
  echo -e "  \033[32m[✓] Package Manager: ${PACKAGE_MANAGER} (${PM_CMD})\033[0m"
else
  echo -e "  \033[33m[!] Package Manager: Standalone / Unrecognized\033[0m"
fi

# --- privilege elevation helper ---------------------------------------------
run_elevated() {
  if [ "$(id -u)" -eq 0 ]; then
    "$@"
  elif command -v sudo >/dev/null 2>&1; then
    echo -e "\033[36m>> Requesting elevated privileges (sudo) to install dependencies...\033[0m"
    sudo "$@"
  elif command -v doas >/dev/null 2>&1; then
    echo -e "\033[36m>> Requesting elevated privileges (doas) to install dependencies...\033[0m"
    doas "$@"
  elif command -v su >/dev/null 2>&1; then
    echo -e "\033[36m>> Requesting root password (su) to install dependencies...\033[0m"
    local escaped_cmd
    escaped_cmd="$(printf '%q ' "$@")"
    su -c "$escaped_cmd"
  else
    echo "error: Elevated privileges required to install dependencies, but neither sudo, doas, nor su found." >&2
    return 1
  fi
}

# --- package installation helper --------------------------------------------
install_system_packages() {
  local pkgs=("$@")
  [ ${#pkgs[@]} -eq 0 ] && return 0

  if [ "$AUTO_DEPS" -eq 0 ]; then
    echo -e "\033[33m>> Skipping automated package installation (--no-deps specified).\033[0m"
    return 1
  fi

  echo -e "\033[32m>> Installing dependencies via ${PACKAGE_MANAGER}: ${pkgs[*]}\033[0m"

  case "$PACKAGE_MANAGER" in
    zypper)
      run_elevated zypper --non-interactive in "${pkgs[@]}"
      ;;
    apt-get)
      run_elevated apt-get update -qq
      run_elevated env DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "${pkgs[@]}"
      ;;
    dnf)
      run_elevated dnf install -y "${pkgs[@]}"
      ;;
    yum)
      run_elevated yum install -y "${pkgs[@]}"
      ;;
    pacman)
      run_elevated pacman -Sy --noconfirm "${pkgs[@]}"
      ;;
    apk)
      run_elevated apk add --no-cache "${pkgs[@]}"
      ;;
    xbps)
      run_elevated xbps-install -Sy "${pkgs[@]}"
      ;;
    emerge)
      run_elevated emerge --ask=n "${pkgs[@]}"
      ;;
    brew)
      brew install "${pkgs[@]}"
      ;;
    pkg)
      run_elevated pkg install -y "${pkgs[@]}"
      ;;
    *)
      echo -e "\033[33m>> Unrecognized package manager. Please install manually: ${pkgs[*]}\033[0m"
      return 1
      ;;
  esac
}

# --- ensure core utilities --------------------------------------------------
ensure_core_utils() {
  local missing_utils=()
  if ! command -v curl >/dev/null 2>&1 && ! command -v wget >/dev/null 2>&1; then
    missing_utils+=("curl")
  fi
  command -v tar >/dev/null 2>&1 || missing_utils+=("tar")
  command -v gzip >/dev/null 2>&1 || missing_utils+=("gzip")

  if [ ${#missing_utils[@]} -gt 0 ]; then
    echo -e "\033[33m>> Missing essential utilities: ${missing_utils[*]}\033[0m"
    install_system_packages "${missing_utils[@]}" || true
  fi
}
ensure_core_utils

# --- ensure build toolchain dependencies ------------------------------------
ensure_build_dependencies() {
  local need_toolchain=0
  local missing_deps=()

  command -v cargo >/dev/null 2>&1 || { need_toolchain=1; missing_deps+=("cargo"); }
  command -v rustc >/dev/null 2>&1 || { need_toolchain=1; missing_deps+=("rustc"); }
  if ! command -v gcc >/dev/null 2>&1 && ! command -v clang >/dev/null 2>&1 && ! command -v cc >/dev/null 2>&1; then
    need_toolchain=1
    missing_deps+=("gcc/build-essentials")
  fi
  command -v git >/dev/null 2>&1 || { need_toolchain=1; missing_deps+=("git"); }

  if [ "$need_toolchain" -eq 1 ]; then
    echo ""
    echo -e "\033[33m>> Build dependencies required to compile Forgum: ${missing_deps[*]}\033[0m"
    echo -e "\033[36m>> Automatically resolving toolchain for ${DISTRO_NAME}...\033[0m"

    local target_pkgs=()
    case "$PACKAGE_MANAGER" in
      zypper)
        target_pkgs=("rust" "cargo" "gcc" "git")
        ;;
      apt-get)
        target_pkgs=("cargo" "rustc" "build-essential" "git")
        ;;
      dnf|yum)
        target_pkgs=("cargo" "rust" "gcc" "git")
        ;;
      pacman)
        target_pkgs=("rust" "base-devel" "git")
        ;;
      apk)
        target_pkgs=("cargo" "rust" "build-base" "git")
        ;;
      xbps)
        target_pkgs=("rust" "cargo" "base-devel" "git")
        ;;
      emerge)
        target_pkgs=("dev-lang/rust" "sys-devel/gcc" "dev-vcs/git")
        ;;
      brew)
        target_pkgs=("rust" "gcc" "git")
        ;;
      pkg)
        target_pkgs=("rust" "gcc" "git")
        ;;
    esac

    if [ ${#target_pkgs[@]} -gt 0 ]; then
      install_system_packages "${target_pkgs[@]}" || true
    fi
  fi

  # Verify MSRV requirement (Rust 1.98.0+)
  local msrv="1.98.0"
  local need_rust_upgrade=0
  if command -v rustc >/dev/null 2>&1; then
    local current_rustc
    current_rustc="$(rustc --version 2>/dev/null | awk '{print $2}' || true)"
    if [ -n "$current_rustc" ]; then
      local older
      older="$(printf '%s\n%s' "$msrv" "$current_rustc" | sort -V | head -n1)"
      if [ "$older" != "$msrv" ]; then
        echo -e "\033[33m>> Installed rustc (${current_rustc}) is older than required MSRV (${msrv}).\033[0m"
        need_rust_upgrade=1
      fi
    fi
  else
    need_rust_upgrade=1
  fi

  if [ "$need_rust_upgrade" -eq 1 ]; then
    echo -e "\033[36m>> Ensuring modern Rust toolchain (${msrv}+) via rustup...\033[0m"
    if command -v rustup >/dev/null 2>&1; then
      rustup update stable
    else
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
      export PATH="$HOME/.cargo/bin:$PATH"
      if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck disable=SC1091
        . "$HOME/.cargo/env"
      fi
    fi
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: Cargo/Rust toolchain is required to build Forgum from source." >&2
    exit 1
  fi
}

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
  EXISTING_UPDATE_CMD="sudo pacman -S forgum"
elif command -v dpkg >/dev/null 2>&1 && dpkg -s forgum >/dev/null 2>&1; then
  EXISTING_PM="Apt"
  EXISTING_PATH="/usr/bin/forgum"
  EXISTING_UPDATE_CMD="sudo apt update && sudo apt install --only-upgrade forgum"
elif command -v rpm >/dev/null 2>&1 && rpm -q forgum >/dev/null 2>&1; then
  EXISTING_PM="RPM"
  EXISTING_PATH="/usr/bin/forgum"
  EXISTING_UPDATE_CMD="sudo rpm -U forgum"
elif command -v zypper >/dev/null 2>&1 && zypper se -i forgum >/dev/null 2>&1; then
  EXISTING_PM="Zypper"
  EXISTING_PATH="/usr/bin/forgum"
  EXISTING_UPDATE_CMD="sudo zypper update forgum"
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
if [ -n "$EXISTING_PM" ] && [ "$HEADLESS" -eq 0 ] && [ "$ASSUME_YES" -eq 0 ]; then
  if [ -t 0 ]; then
    echo ""
    echo -e "\033[33m⚠ Package Manager Conflict Detected!\033[0m"
    echo -e "  • Found existing Forgum via \033[36m${EXISTING_PM}\033[0m: ${EXISTING_PATH}"
    echo ""
    echo -e "How would you like to proceed?"
    echo -e "  \033[32m[1] Delegate to package manager (Run '${EXISTING_UPDATE_CMD}' and exit)\033[0m"
    echo -e "  \033[36m[2] Switch to standalone channel build (Install to ${INSTALL_DIR} and update PATH)\033[0m"
    echo -e "  \033[33m[3] Clean reinstall / overwrite\033[0m"
    echo ""
    CHOICE="2"
    read -t 10 -r -p "Select option [1/2/3] (default: 2 in 10s): " USER_CHOICE || true
    CHOICE="${USER_CHOICE:-2}"
    if [ "$CHOICE" = "1" ]; then
      echo -e "\033[32m>> Delegating update to ${EXISTING_PM}...\033[0m"
      eval "$EXISTING_UPDATE_CMD"
      exit 0
    fi
  else
    # Non-interactive / piped execution (curl ... | bash): automatically proceed with standalone upgrade
    echo -e "\033[36m>> Found existing Forgum via ${EXISTING_PM} (${EXISTING_PATH}). Proceeding with standalone update...\033[0m"
    CHOICE="2"
  fi
fi

# --- resolve version (100% resilient multi-tier fallback) -------------------
resolve_version() {
  if [ -n "$VERSION_OVERRIDE" ]; then
    echo "$VERSION_OVERRIDE"
    return 0
  fi
  if [ -n "${FORGUM_VERSION:-}" ]; then
    echo "$FORGUM_VERSION"
    return 0
  fi
  if [ -f "Cargo.toml" ]; then
    local v
    v="$(grep -E '^version\s*=' Cargo.toml | head -n1 | sed -E 's/.*"([^"]+)".*/\1/' 2>/dev/null || true)"
    if [ -n "$v" ]; then
      echo "$v"
      return 0
    fi
  fi

  # Attempt raw GitHub Cargo.toml (matches repository codebase version)
  for branch in dev main master; do
    local gh_cargo
    gh_cargo="$(curl -sSL -H "User-Agent: forgum-install" "https://raw.githubusercontent.com/${REPO}/${branch}/Cargo.toml" 2>/dev/null | grep -E '^version\s*=' | head -n1 | sed -E 's/.*"([^"]+)".*/\1/' || true)"
    if [ -n "$gh_cargo" ]; then
      echo "$gh_cargo"
      return 0
    fi
  done

  # Attempt GitHub releases API without failing on 404
  local gh_latest
  gh_latest="$(curl -sSL -H "User-Agent: forgum-install" "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | tr ',' '\n' | grep '"tag_name"' | head -n1 | sed -E 's/.*"tag_name"[[:space:]]*:[[:space:]]*"v?([^"]+)".*/\1/' || true)"
  if [ -n "$gh_latest" ]; then
    echo "$gh_latest"
    return 0
  fi

  # Default fallback constant
  echo "0.4.0"
}

VERSION="$(resolve_version)"
TAG="v${VERSION}"
echo -e "\033[36m>> Installing Forgum ${VERSION} (${CHANNEL} channel) from ${REPO} (${TAG})\033[0m"

# --- target architecture & OS ----------------------------------------------
case "$RAW_OS" in
  linux)
    if ldd --version 2>&1 | grep -qi musl || ls -l /lib/ld-musl* 2>/dev/null | grep -q musl; then
      TARGET_OS="unknown-linux-musl"
    else
      TARGET_OS="unknown-linux-gnu"
    fi
    ;;
  darwin) TARGET_OS="apple-darwin" ;;
  freebsd) TARGET_OS="unknown-freebsd" ;;
  *) echo "error: unsupported OS '$RAW_OS' (use install.ps1 on Windows)" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) TARGET_ARCH="x86_64" ;;
  aarch64|arm64) TARGET_ARCH="aarch64" ;;
  armv7*|armhf) TARGET_ARCH="armv7" ;;
  riscv64*) TARGET_ARCH="riscv64gc" ;;
  i686|i386) TARGET_ARCH="i686" ;;
  *) echo "error: unsupported architecture '$ARCH'" >&2; exit 1 ;;
esac

# --- install or compile pipeline --------------------------------------------
if [ "$BUILD_FROM_SOURCE" -eq 0 ] && [ -f "target/release/forgum" ]; then
  install -m 0755 "target/release/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$LEGACY_PATH"
  echo -e "\033[32m>> Installed from local release build: $BIN_PATH\033[0m"
elif [ "$BUILD_FROM_SOURCE" -eq 0 ] && [ -f "target/debug/forgum" ]; then
  install -m 0755 "target/debug/forgum" "$BIN_PATH"
  ln -sf "$BIN_PATH" "$LEGACY_PATH"
  echo -e "\033[32m>> Installed from local debug build: $BIN_PATH\033[0m"
else
  TMP="$(mktemp -d 2>/dev/null || mktemp -d -t 'forgum')"
  trap 'rm -rf "$TMP"' EXIT

  DOWNLOAD_OK=0
  if [ "$BUILD_FROM_SOURCE" -eq 0 ]; then
    CANDIDATE_URLS=(
      "https://github.com/${REPO}/releases/download/${TAG}/forgum-${VERSION}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
      "https://github.com/${REPO}/releases/download/${TAG}/forgum-${TAG}-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
      "https://github.com/${REPO}/releases/download/nightly/forgum-nightly-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
      "https://github.com/${REPO}/releases/latest/download/forgum-${TARGET_ARCH}-${TARGET_OS}.tar.gz"
    )

    ARCHIVE=""
    for CANDIDATE in "${CANDIDATE_URLS[@]}"; do
      ASSET_NAME="$(basename "$CANDIDATE")"
      echo -e "\033[36m>> Checking prebuilt release archive: ${ASSET_NAME}...\033[0m"
      if command -v curl >/dev/null 2>&1; then
        if curl -fsSL -4 --connect-timeout 10 -H "User-Agent: forgum-install" "$CANDIDATE" -o "$TMP/$ASSET_NAME" 2>/dev/null; then
          if [ -s "$TMP/$ASSET_NAME" ]; then
            ARCHIVE="$TMP/$ASSET_NAME"
            DOWNLOAD_OK=1
            break
          fi
        fi
      elif command -v wget >/dev/null 2>&1; then
        if wget -q -4 --timeout=10 "$CANDIDATE" -O "$TMP/$ASSET_NAME" 2>/dev/null; then
          if [ -s "$TMP/$ASSET_NAME" ]; then
            ARCHIVE="$TMP/$ASSET_NAME"
            DOWNLOAD_OK=1
            break
          fi
        fi
      fi
    done

    if [ "$DOWNLOAD_OK" -eq 1 ] && [ -n "$ARCHIVE" ]; then
      tar xzf "$ARCHIVE" -C "$TMP" 2>/dev/null || true
      EXTRACTED="$(find "$TMP" -maxdepth 2 -type f \( -name 'forgum' -o -name 'forgum-engine' \) | head -n1 || true)"
      if [ -n "$EXTRACTED" ]; then
        chmod +x "$EXTRACTED" 2>/dev/null || true
        if "$EXTRACTED" --version >/dev/null 2>&1; then
          if [ -w "$INSTALL_DIR" ]; then
            install -m 0755 "$EXTRACTED" "$BIN_PATH"
          else
            run_elevated install -m 0755 "$EXTRACTED" "$BIN_PATH"
          fi
          ln -sf "$BIN_PATH" "$LEGACY_PATH" 2>/dev/null || true
          echo -e "\033[32m>> Successfully installed prebuilt binary: $BIN_PATH\033[0m"
        else
          echo -e "\033[33m>> Prebuilt binary incompatible with host C library (GLIBC/musl mismatch). Falling back to source build...\033[0m"
          DOWNLOAD_OK=0
        fi
      else
        DOWNLOAD_OK=0
      fi
    fi
  fi

  if [ "$DOWNLOAD_OK" -eq 0 ]; then
    echo -e "\033[33m>> Prebuilt release asset unavailable for ${TARGET_ARCH}-${TARGET_OS}. Compiling Forgum from source...\033[0m"
    ensure_build_dependencies

    if [ -f "Cargo.toml" ]; then
      echo -e "\033[36m>> Building local workspace (--bin forgum)...\033[0m"
      cargo build --release --bin forgum
      install -m 0755 "target/release/forgum" "$BIN_PATH"
      ln -sf "$BIN_PATH" "$LEGACY_PATH"
    else
      echo -e "\033[36m>> Fetching latest source from https://github.com/${REPO}.git...\033[0m"
      BUILD_DIR="$TMP/src"
      if git clone --depth 1 "https://github.com/${REPO}.git" "$BUILD_DIR" 2>/dev/null; then
        cargo build --release --manifest-path "$BUILD_DIR/Cargo.toml" --bin forgum
        install -m 0755 "$BUILD_DIR/target/release/forgum" "$BIN_PATH"
        ln -sf "$BIN_PATH" "$LEGACY_PATH"
      else
        echo -e "\033[36m>> Compiling via cargo install --git...\033[0m"
        cargo install --git "https://github.com/${REPO}.git" --bin forgum --root "$TMP/cargo-bin" --force
        install -m 0755 "$TMP/cargo-bin/bin/forgum" "$BIN_PATH"
        ln -sf "$BIN_PATH" "$LEGACY_PATH"
      fi
    fi
    echo -e "\033[32m>> Compilation and installation complete: $BIN_PATH\033[0m"
  fi
fi

# --- install mascot data if available in source checkout ---------------------
SHARE_DIR="/usr/local/share/forgum"
if [ ! -w /usr/local/share ] && [ ! -w /usr/local ]; then
  SHARE_DIR="$HOME/.local/share/forgum"
fi
if [ -d "data" ]; then
  mkdir -p "$SHARE_DIR" 2>/dev/null || run_elevated mkdir -p "$SHARE_DIR" 2>/dev/null || true
  cp -r data/* "$SHARE_DIR/" 2>/dev/null || run_elevated cp -r data/* "$SHARE_DIR/" 2>/dev/null || true
elif [ -n "${BUILD_DIR:-}" ] && [ -d "$BUILD_DIR/data" ]; then
  mkdir -p "$SHARE_DIR" 2>/dev/null || run_elevated mkdir -p "$SHARE_DIR" 2>/dev/null || true
  cp -r "$BUILD_DIR/data/"* "$SHARE_DIR/" 2>/dev/null || run_elevated cp -r "$BUILD_DIR/data/"* "$SHARE_DIR/" 2>/dev/null || true
fi

# --- check & update PATH (idempotent prepend) --------------------------------
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo -e "\033[32m>> Adding $INSTALL_DIR to PATH...\033[0m"
    PATH_BLOCK="# >>> forgum path >>>\nexport PATH=\"${INSTALL_DIR}:\$PATH\"\n# <<< forgum path <<<"
    for rc in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.bash_profile" "$HOME/.profile"; do
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
  "platform": "${RAW_OS}",
  "arch": "${ARCH}",
  "is_wsl": ${IS_WSL},
  "distro": "${DISTRO_ID}",
  "package_manager": "${PACKAGE_MANAGER}",
  "auto_update": true
}
EOF

# --- launch wizard or headless setup ---------------------------------------
TELEM_ARG=""
if [ -n "$TELEMETRY" ]; then
  TELEM_ARG="--telemetry $TELEMETRY"
fi

if [ "$HEADLESS" -eq 1 ] || [ ! -t 0 ]; then
  # Headless execution or piped curl invocation (e.g. curl ... | bash)
  "$BIN_PATH" install --headless $TELEM_ARG
  echo ""
  echo -e "\033[32m✦ FORGUM CELESTIAL INSTALLATION COMPLETE ✦\033[0m"
  echo -e "  • Installed binary: \033[36m${BIN_PATH}\033[0m"
  echo -e "  • Channel: \033[35m${CHANNEL}\033[0m | Release: \033[33m${TAG}\033[0m"
  echo -e "  • Run '\033[32mforgum\033[0m' to launch, or '\033[36mforgum tui\033[0m' for the configuration studio."
  echo ""
elif [ -t 0 ] && [ -t 1 ]; then
  # Standard interactive terminal execution
  "$BIN_PATH" install
else
  # Fallback headless execution
  "$BIN_PATH" install --headless $TELEM_ARG
fi
