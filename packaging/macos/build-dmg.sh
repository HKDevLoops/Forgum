#!/usr/bin/env bash
#
# build-dmg.sh - Automated macOS DMG Disk Image Packager for Forgum.
#
# Packages standalone macOS DMG disk images for Apple Silicon (arm64) and Intel (x86_64).
# Can be run natively on macOS runners in GitHub Actions or locally on Darwin.
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# Version: first CLI arg, else derive from workspace Cargo.toml.
if [ $# -ge 1 ] && [ -n "${1:-}" ]; then
  VERSION="$1"
else
  VERSION="$(grep -m1 '^version' "${REPO_ROOT}/Cargo.toml" | sed -E 's/.*"([^"]+)".*/\1/')"
fi

ARCH="$(uname -m)"
case "$ARCH" in
  arm64|aarch64) MAC_ARCH="arm64" ;;
  x86_64|amd64)  MAC_ARCH="x86_64" ;;
  *)             MAC_ARCH="$ARCH" ;;
esac

DMG_NAME="forgum-${VERSION}-macos-${MAC_ARCH}.dmg"
STAGE_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'forgum-dmg')"
trap 'rm -rf "$STAGE_DIR"' EXIT

echo ">> Staging Forgum v${VERSION} (${MAC_ARCH}) for DMG disk image creation..."

# Locate or build the release binary
BIN_PATH="${REPO_ROOT}/target/release/forgum"
if [ ! -f "$BIN_PATH" ]; then
  if [ -f "${REPO_ROOT}/target/${ARCH}-apple-darwin/release/forgum" ]; then
    BIN_PATH="${REPO_ROOT}/target/${ARCH}-apple-darwin/release/forgum"
  else
    echo ">> Building release binary natively via cargo..."
    cargo build --release --locked -p forgum-engine --bin forgum
  fi
fi

cp "$BIN_PATH" "${STAGE_DIR}/forgum"
chmod 755 "${STAGE_DIR}/forgum"

# Helper installation script inside the DMG for drag-and-drop or terminal execution
cat << 'EOF' > "${STAGE_DIR}/install.sh"
#!/usr/bin/env bash
set -e
TARGET_DIR="/usr/local/bin"
if [ ! -w "$TARGET_DIR" ]; then
  TARGET_DIR="$HOME/.local/bin"
fi
mkdir -p "$TARGET_DIR"
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "$DIR/forgum" "$TARGET_DIR/forgum"
chmod 755 "$TARGET_DIR/forgum"
echo "✓ Forgum installed successfully to $TARGET_DIR/forgum"
"$TARGET_DIR/forgum" --version
EOF
chmod 755 "${STAGE_DIR}/install.sh"

[ -f "${REPO_ROOT}/README.md" ] && cp "${REPO_ROOT}/README.md" "${STAGE_DIR}/"
[ -f "${REPO_ROOT}/LICENSE" ] && cp "${REPO_ROOT}/LICENSE" "${STAGE_DIR}/"

OUTPUT_DMG="${REPO_ROOT}/${DMG_NAME}"
rm -f "$OUTPUT_DMG"

if command -v hdiutil >/dev/null 2>&1; then
  echo ">> Generating compressed UDZO disk image: ${DMG_NAME}..."
  hdiutil create -volname "Forgum ${VERSION}" -srcfolder "$STAGE_DIR" -ov -format UDZO "$OUTPUT_DMG"
  (
    cd "${REPO_ROOT}"
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 "${DMG_NAME}" > "${DMG_NAME}.sha256"
    elif command -v sha256sum >/dev/null 2>&1; then
      sha256sum "${DMG_NAME}" > "${DMG_NAME}.sha256"
    fi
  )
  echo ">> Successfully generated: ${OUTPUT_DMG}"
else
  echo ">> Warning: hdiutil not found (only available on macOS). Packaging as tarball fallback..."
  tar -czf "${REPO_ROOT}/forgum-${VERSION}-macos-${MAC_ARCH}.tar.gz" -C "$STAGE_DIR" .
fi
