#!/bin/bash
set -euo pipefail

# Resolve repo root (script lives in packaging/pacman).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

# Build the engine binary from source first (self-contained; no release download needed).
echo "Building forgum-engine..."
cargo build --release --locked -p forgum-engine

# Work in an isolated makepkg directory so we don't pollute the tree.
WORKDIR="$(mktemp -d)"
trap 'rm -rf "${WORKDIR}"' EXIT
PKG_DIR="${WORKDIR}/forgum"
mkdir -p "${PKG_DIR}/src"

# Generate a CI-local PKGBUILD that installs the already-built binary instead of
# downloading a release tarball (which does not exist for arbitrary commits).
VERSION="$(grep -m1 '^version' "${REPO_ROOT}/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/')"
cat > "${PKG_DIR}/PKGBUILD" <<EOF
# Auto-generated for CI. Builds from the local checkout.
# Maintainer: HKDEVS <hkdevs@example.com>
pkgname=forgum
pkgver=${VERSION}
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
  install -Dm755 "${REPO_ROOT}/target/release/forgum-engine" "\${pkgdir}/usr/bin/forgum-engine"
  install -Dm644 "${REPO_ROOT}/LICENSE" "\${pkgdir}/usr/share/licenses/\${pkgname}/LICENSE"
}
EOF

cd "${PKG_DIR}"
makepkg --nodeps --force --noconfirm
echo "Built pacman package(s) in: ${PKG_DIR}"
ls -la "${PKG_DIR}"/*.zst 2>/dev/null || true
