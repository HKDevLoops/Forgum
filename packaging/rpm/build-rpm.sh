#!/bin/bash
set -euo pipefail

VERSION="${1:?Usage: $0 <version>}"

RPM_DIR=$(pwd)/rpmbuild
mkdir -p "${RPM_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Download linux binary from GitHub releases
curl -L "https://github.com/harish2222/Forgum/releases/download/v${VERSION}/forgum-engine-linux-amd64" \
  -o "${RPM_DIR}/SOURCES/forgum-engine"

rpmbuild -bb --define "_topdir ${RPM_DIR}" --define "version ${VERSION}" forgum.spec

echo "Built RPM in: ${RPM_DIR}/RPMS/x86_64/"
