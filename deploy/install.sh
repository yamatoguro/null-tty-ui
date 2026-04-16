#!/usr/bin/env bash
# Installs and enables the NullByte UI dashboard as a systemd service.
# Must be run as root on the target Raspberry Pi.
set -euo pipefail

BINARY_SRC="${1:-target/release/nullbyteui}"
INSTALL_DIR="/opt/nullbyteui"

echo "[install] copying binary and config"
install -d "$INSTALL_DIR/config"
install -m 0755 "$BINARY_SRC" "$INSTALL_DIR/nullbyteui"
cp -n config/layout.default.toml "$INSTALL_DIR/config/layout.default.toml" || true

echo "[install] installing systemd unit"
cp deploy/nullbyteui.service /etc/systemd/system/nullbyteui.service
systemctl daemon-reload
systemctl enable --now nullbyteui.service

echo "[install] done — status:"
systemctl status nullbyteui.service --no-pager || true
