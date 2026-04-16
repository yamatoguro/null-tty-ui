# nullbyteui

Painel eDEX-inspired para Raspberry Pi com foco em leveza, binario nativo e personalizacao por plugins.

## Estado atual
- Runtime terminal fullscreen nativo (Rust + ratatui).
- Layout em 5 regioes: top, left, center, right, bottom.
- Carregamento de plugins via manifestos em `plugins/*/manifest.toml`.
- Coleta de metricas Linux em tempo real (CPU, memoria, disco, rede, load e logs).
- Painel de navegacao de arquivos no centro com polling leve.
- Integracao com Technitium DNS via HTTP local com alertas de bloqueio.
- Diagnostico periodico de FPS/CPU/RAM do processo.

## Executar localmente
1. Instale Rust toolchain.
2. Rode `cargo run`.
3. Pressione `q` para sair.

## Gerar release v0.1.0
```bash
chmod +x scripts/release/package.sh
./scripts/release/package.sh 0.1.0
```

Artefato gerado em `dist/`:
- `nullbyteui-v0.1.0-linux-<arch>.tar.gz`
- `nullbyteui-v0.1.0-linux-<arch>.tar.gz.sha256`

## Layout
Arquivo de configuracao principal: `config/layout.default.toml`.

Cada regiao referencia um plugin por id:
- top
- left
- center
- right
- bottom

## Plugins
Cada plugin usa manifesto TOML com os campos:
- id
- version
- title
- description
- update_interval_ms
- permissions

## Instalacao
- Instalacao direta sem clone (Raspberry Pi):

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/yamatoguro/null-tty-ui/main/deploy/install-from-github.sh)
```

- Reinstalacao limpa (reparo):

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/yamatoguro/null-tty-ui/main/deploy/install-from-github.sh) --clean
```

- Comando para executar apos instalar: `null-ui`
- Guia completo: `docs/release/install-and-customization.md`
- Schema congelado v1: `docs/spec/layout-schema-v1.md`
