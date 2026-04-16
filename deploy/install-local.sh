#!/usr/bin/env bash
# Instala a release no espaco do usuario (sem root) para validacao rapida.
set -euo pipefail

BINARY_SRC="${1:-target/release/nullbyteui}"
PREFIX="${2:-$HOME/.local/nullbyteui}"

mkdir -p "$PREFIX/bin" "$PREFIX/config" "$PREFIX/plugins" "$PREFIX/logs"
install -m 0755 "$BINARY_SRC" "$PREFIX/bin/nullbyteui"
cp config/layout.default.toml "$PREFIX/config/layout.default.toml"
cp -r plugins/* "$PREFIX/plugins/"

echo "Instalado em: $PREFIX"
echo "Executar: $PREFIX/bin/nullbyteui --config $PREFIX/config/layout.default.toml"
