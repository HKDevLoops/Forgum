#!/bin/bash
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"
ARCH="amd64"
PKG_NAME="forgum"
DEB_DIR="forgum_${VERSION}_${ARCH}"

mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"

# Download linux binary from GitHub releases
curl -L "https://github.com/HKDevLoops/Forgum/releases/download/v${VERSION}/forgum-engine-x86_64-unknown-linux-gnu.tar.gz" -o /tmp/forgum-engine.tar.gz \
  && tar -xzf /tmp/forgum-engine.tar.gz -C "${DEB_DIR}/usr/bin" forgum-engine && rm -f /tmp/forgum-engine.tar.gz \
  || curl -L "https://github.com/HKDevLoops/Forgum/releases/download/v${VERSION}/forgum-engine-x86_64-unknown-linux-gnu.tar.gz" \
   -o "${DEB_DIR}/usr/bin/forgum-engine"
  -o "${DEB_DIR}/usr/bin/forgum-engine"

chmod 755 "${DEB_DIR}/usr/bin/forgum-engine"

# Copy control and postinst
cp DEBIAN/control "${DEB_DIR}/DEBIAN/"
sed -i "s/^Version:.*/Version: ${VERSION}/" "${DEB_DIR}/DEBIAN/control"
cp DEBIAN/postinst "${DEB_DIR}/DEBIAN/"
chmod 755 "${DEB_DIR}/DEBIAN/postinst"

dpkg-deb --build "${DEB_DIR}"
rm -rf "${DEB_DIR}"

echo "Built: ${DEB_DIR}.deb"
