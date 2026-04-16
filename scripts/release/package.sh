#!/usr/bin/env bash
# Build and package first release artifact for NullByteUI.
set -euo pipefail

VERSION="${1:-0.1.0}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ARCH="$(uname -m)"
DIST_DIR="$ROOT_DIR/dist"
PKG_DIR="$DIST_DIR/nullbyteui-v${VERSION}-linux-${ARCH}"
TARBALL="$DIST_DIR/nullbyteui-v${VERSION}-linux-${ARCH}.tar.gz"

mkdir -p "$DIST_DIR"

source "$HOME/.cargo/env"
cd "$ROOT_DIR"
cargo build --release

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/bin" "$PKG_DIR/config" "$PKG_DIR/plugins" "$PKG_DIR/deploy" "$PKG_DIR/docs"

cp target/release/nullbyteui "$PKG_DIR/bin/nullbyteui"
cp config/layout.default.toml "$PKG_DIR/config/layout.default.toml"
cp -r config/schema "$PKG_DIR/config/schema"
cp -r plugins/* "$PKG_DIR/plugins/"
cp deploy/nullbyteui.service "$PKG_DIR/deploy/nullbyteui.service"
cp deploy/install.sh "$PKG_DIR/deploy/install.sh"
cp docs/release/install-and-customization.md "$PKG_DIR/docs/install-and-customization.md"
cp docs/spec/layout-schema-v1.md "$PKG_DIR/docs/layout-schema-v1.md"
cp README.md "$PKG_DIR/README.md"

chmod +x "$PKG_DIR/deploy/install.sh"

tar -C "$DIST_DIR" -czf "$TARBALL" "$(basename "$PKG_DIR")"
sha256sum "$TARBALL" > "$TARBALL.sha256"

echo "Package created: $TARBALL"
echo "SHA256 file: $TARBALL.sha256"
