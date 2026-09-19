#!/bin/bash
set -euo pipefail

# Resolve repo root (script lives in packaging/rpm).
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

cd "${REPO_ROOT}"
RPM_DIR="$(pwd)/rpmbuild"
mkdir -p "${RPM_DIR}"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Build the forgum binary from source
echo "Building forgum for version ${VERSION}..."
cargo build --release --locked -p forgum-engine --bin forgum
cp "target/release/forgum" "${RPM_DIR}/SOURCES/forgum"
chmod 755 "${RPM_DIR}/SOURCES/forgum"
cp "target/release/forgum" "${RPM_DIR}/BUILD/forgum"
chmod 755 "${RPM_DIR}/BUILD/forgum"

rpmbuild -bb --define "_topdir ${RPM_DIR}" --define "version ${VERSION}" "${SCRIPT_DIR}/forgum.spec"

echo "Built RPM in: ${RPM_DIR}/RPMS/x86_64/"
