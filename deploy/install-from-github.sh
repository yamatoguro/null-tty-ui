#!/usr/bin/env bash
# Instala o NullByteUI direto do GitHub sem precisar clonar o repositório.
# Resultado: comando global `null-ui` disponível no sistema.
set -euo pipefail

REPO="${REPO:-yamatoguro/null-tty-ui}"
REF="${REF:-main}"
INSTALL_DIR="${INSTALL_DIR:-/opt/nullbyteui}"
BIN_LINK="${BIN_LINK:-/usr/local/bin/null-ui}"

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Erro: comando '$cmd' não encontrado."
    exit 1
  fi
}

ensure_rust() {
  if command -v cargo >/dev/null 2>&1; then
    return
  fi

  require_cmd curl
  echo "[install] Rust não encontrado. Instalando via rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1090
  source "$HOME/.cargo/env"
}

main() {
  require_cmd curl
  require_cmd tar
  require_cmd sudo

  ensure_rust
  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
  fi

  local workdir
  workdir="$(mktemp -d)"
  trap 'rm -rf "$workdir"' EXIT

  echo "[install] Baixando código-fonte de https://github.com/$REPO ($REF)..."
  curl -fsSL "https://codeload.github.com/$REPO/tar.gz/$REF" -o "$workdir/src.tar.gz"
  tar -xzf "$workdir/src.tar.gz" -C "$workdir"

  local src_dir
  src_dir="$(find "$workdir" -maxdepth 1 -type d -name "null-tty-ui-*" | head -n 1)"
  if [ -z "$src_dir" ]; then
    echo "Erro: não foi possível localizar diretório extraído."
    exit 1
  fi

  echo "[install] Compilando release..."
  (
    cd "$src_dir"
    cargo build --release
  )

  echo "[install] Instalando arquivos em $INSTALL_DIR..."
  sudo install -d "$INSTALL_DIR/config" "$INSTALL_DIR/plugins"
  sudo install -m 0755 "$src_dir/target/release/nullbyteui" "$INSTALL_DIR/nullbyteui"
  sudo cp "$src_dir/config/layout.default.toml" "$INSTALL_DIR/config/layout.default.toml"
  sudo cp -r "$src_dir/plugins/." "$INSTALL_DIR/plugins/"

  echo "[install] Criando comando global $BIN_LINK..."
  sudo tee "$BIN_LINK" >/dev/null <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd /opt/nullbyteui
exec /opt/nullbyteui/nullbyteui --config /opt/nullbyteui/config/layout.default.toml "$@"
EOF
  sudo chmod +x "$BIN_LINK"

  echo
  echo "Instalação concluída."
  echo "Execute com: null-ui"
  echo "Para sair da UI: pressione q"
}

main "$@"
