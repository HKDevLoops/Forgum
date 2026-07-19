#!/bin/bash
set -euo pipefail

# Resolve repo root (script lives in packaging/deb).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Version: first CLI arg, else derive from workspace Cargo.toml.
if [ $# -ge 1 ] && [ -n "${1:-}" ]; then
  VERSION="$1"
else
  VERSION="$(grep -m1 '^version' "${REPO_ROOT}/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/')"
fi
if [ -z "${VERSION}" ]; then
  echo "ERROR: could not determine version (pass as \$1 or set it in Cargo.toml)."
  exit 1
fi

ARCH="amd64"
PKG_NAME="forgum"
DEB_DIR="forgum_${VERSION}_${ARCH}"

cd "${REPO_ROOT}"
mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"

# Build the engine binary from source (self-contained; no release download needed).
echo "Building forgum-engine for version ${VERSION}..."
cargo build --release --locked -p forgum-engine
cp "target/release/forgum-engine" "${DEB_DIR}/usr/bin/forgum-engine"
chmod 755 "${DEB_DIR}/usr/bin/forgum-engine"

# Copy control and postinst
cp "${SCRIPT_DIR}/DEBIAN/control" "${DEB_DIR}/DEBIAN/"
sed -i "s/^Version:.*/Version: ${VERSION}/" "${DEB_DIR}/DEBIAN/control"
cp "${SCRIPT_DIR}/DEBIAN/postinst" "${DEB_DIR}/DEBIAN/"
chmod 755 "${DEB_DIR}/DEBIAN/postinst"

dpkg-deb --build "${DEB_DIR}"
rm -rf "${DEB_DIR}"

echo "Built: ${DEB_DIR}.deb"
