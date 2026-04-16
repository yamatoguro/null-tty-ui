#!/usr/bin/env bash
# Instala o NullByteUI direto do GitHub sem precisar clonar o repositório.
# Resultado: comando global `null-ui` disponível no sistema.
set -euo pipefail

REPO="${REPO:-yamatoguro/null-tty-ui}"
REF="${REF:-main}"
INSTALL_DIR="${INSTALL_DIR:-/opt/nullbyteui}"
BIN_LINK="${BIN_LINK:-/usr/local/bin/null-ui}"
CLEAN_INSTALL="${CLEAN_INSTALL:-0}"

REQUIRED_PLUGINS=(
  "system_overview"
  "process_list"
  "terminal"
  "technitium_dns_chart"
  "log_stream"
)

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

repair_or_prepare_target() {
  if [ "$CLEAN_INSTALL" = "1" ] && [ -d "$INSTALL_DIR" ]; then
    echo "[install] CLEAN_INSTALL=1 definido. Limpando instalação anterior em $INSTALL_DIR..."
    sudo rm -rf "$INSTALL_DIR"
  fi

  if [ -d "$INSTALL_DIR" ]; then
    echo "[install] Instalação existente detectada em $INSTALL_DIR. Aplicando modo reparo (refresh completo)."
  fi

  sudo install -d "$INSTALL_DIR/config" "$INSTALL_DIR/plugins"
}

validate_installation() {
  local layout="$INSTALL_DIR/config/layout.default.toml"
  local plugin

  if [ ! -f "$layout" ]; then
    echo "Erro: layout não encontrado em $layout"
    exit 1
  fi

  for plugin in "${REQUIRED_PLUGINS[@]}"; do
    if [ ! -f "$INSTALL_DIR/plugins/$plugin/manifest.toml" ]; then
      echo "Erro: plugin obrigatório ausente: $plugin"
      echo "Dica: reexecute com CLEAN_INSTALL=1 para recriar instalação limpa."
      exit 1
    fi
  done

  # Verifica também os plugins referenciados no layout default.
  while IFS= read -r plugin; do
    [ -z "$plugin" ] && continue
    if [ ! -f "$INSTALL_DIR/plugins/$plugin/manifest.toml" ]; then
      echo "Erro: plugin referenciado no layout e não encontrado: $plugin"
      echo "Dica: reexecute com CLEAN_INSTALL=1 para reparar completamente."
      exit 1
    fi
  done < <(awk -F'"' '/plugin\s*=\s*"/{print $2}' "$layout")
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

  repair_or_prepare_target

  echo "[install] Instalando arquivos em $INSTALL_DIR..."
  # Recria plugins para evitar sobras de instalações antigas/corrompidas.
  sudo rm -rf "$INSTALL_DIR/plugins"
  sudo install -d "$INSTALL_DIR/plugins"
  sudo install -m 0755 "$src_dir/target/release/nullbyteui" "$INSTALL_DIR/nullbyteui"
  sudo cp "$src_dir/config/layout.default.toml" "$INSTALL_DIR/config/layout.default.toml"
  sudo cp -r "$src_dir/plugins/." "$INSTALL_DIR/plugins/"

  echo "[install] Criando comando global $BIN_LINK..."
  sudo tee "$BIN_LINK" >/dev/null <<EOF
#!/usr/bin/env bash
set -euo pipefail
cd "$INSTALL_DIR"
exec "$INSTALL_DIR/nullbyteui" --config "$INSTALL_DIR/config/layout.default.toml" "\$@"
EOF
  sudo chmod +x "$BIN_LINK"

  validate_installation

  echo
  echo "Instalação concluída."
  echo "Execute com: null-ui"
  echo "Para sair da UI: pressione q"
}

main "$@"
