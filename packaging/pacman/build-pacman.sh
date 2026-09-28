#!/bin/bash
set -euo pipefail

# Resolve repo root (script lives in packaging/pacman).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

# Build the engine binary from source first (self-contained; no release
# download needed). Must run as the current user; do this BEFORE switching
# to a non-root user (since the build needs write access to ${REPO_ROOT}/target/
# and the cargo home).
echo "Building forgum..."
cargo build --release --locked -p forgum-engine --bin forgum

# Work in an isolated makepkg directory so we don't pollute the tree.
WORKDIR="$(mktemp -d)"
trap 'rm -rf "${WORKDIR}"' EXIT
PKG_DIR="${WORKDIR}/forgum"
mkdir -p "${PKG_DIR}/src"

# Generate a CI-local PKGBUILD that installs the already-built binary instead of
# downloading a release tarball (which does not exist for arbitrary commits).
VERSION="$(grep -m1 '^version' "${REPO_ROOT}/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/')"
PKGVER="$(echo "${VERSION}" | tr '-' '.' | sed 's/\.alpha\./\.alpha/')"
cat > "${PKG_DIR}/PKGBUILD" <<EOF
# Auto-generated for CI. Builds from the local checkout.
# Maintainer: harish2222 (HKDevLoops) <harish2222@users.noreply.github.com>
pkgname=forgum
pkgver=${PKGVER}
pkgrel=1
pkgdesc="Cross-platform cowsay+fortune+lolcat with a Rust ANSI animation engine"
arch=('x86_64' 'aarch64')
url="https://github.com/HKDevLoops/Forgum"
license=('MIT')
makedepends=('cargo')
source=()
sha256sums=()
build() {
  # Binary is already built by the CI wrapper; nothing to do here.
  true
}
package() {
  install -Dm755 "${REPO_ROOT}/target/release/forgum" "\${pkgdir}/usr/bin/forgum"
  install -Dm644 "${REPO_ROOT}/LICENSE" "\${pkgdir}/usr/share/licenses/\${pkgname}/LICENSE"
}
EOF

# makepkg refuses to run as root by default (it taints the resulting package
# and gives a deliberate `==> ERROR` on every modern Arch container image,
# including the CI `archlinux:latest` we use). Downgrade EUID to a normal
# user in the temp WORKDIR (which is the only thing makepkg cares about —
# we still pre-built the binary above), chown the build artifacts so the
# new user can read them, then exec makepkg as that user.
if [ "$(id -u)" = "0" ]; then
  BUILDER="forgum-pkg-builder"
  if ! id "$BUILDER" >/dev/null 2>&1; then
    useradd -m -s /bin/bash "$BUILDER"
  fi
  chown -R "$BUILDER":"$BUILDER" "$WORKDIR"
  chmod 755 "${REPO_ROOT}/target/release/forgum"
  chown "$BUILDER":"$BUILDER" "${REPO_ROOT}/target/release/forgum"
  cd "${PKG_DIR}"
  exec sudo -u "$BUILDER" makepkg --nodeps --force --noconfirm
fi

# Non-root: just run it.
chown -R "$(id -u)":"$(id -g)" "$WORKDIR" || true
cd "${PKG_DIR}"
makepkg --nodeps --force --noconfirm
echo "Built pacman package(s) in: ${PKG_DIR}"
ls -la "${PKG_DIR}"/*.zst 2>/dev/null || true
