# Layout Schema v1 (Frozen)

Este documento congela o schema de configuracao da primeira versao instalavel do NullByteUI.

## Arquivo
- Referencia canonica: config/schema/layout.v1.json
- Arquivo de exemplo: config/layout.default.toml

## Regras obrigatorias
- schema_version deve ser 1.
- profile deve ser string nao vazia.
- regions deve conter exatamente os bindings operacionais esperados pelo runtime:
  - top
  - left
  - center
  - right
  - bottom
- Cada regiao deve incluir o campo plugin (string nao vazia).

## Campos opcionais
- dns_host: host da API Technitium.
- dns_port: porta da API Technitium.
- dns_token: token da API Technitium.
- diagnostics_log_path: caminho do log de diagnostico.
- target_fps: meta minima de FPS do processo.
- target_process_cpu_percent: teto de uso de CPU do processo.
- target_process_rss_mb: teto de memoria RSS do processo.
- terminal_boot_command: comando inicial do painel PTY.
- file_nav_root: raiz para o plugin de navegacao de arquivos.

## Politica de compatibilidade
- Qualquer mudanca breaking deve criar schema_version = 2.
- Mudancas retrocompativeis sao permitidas em v1 apenas como campos opcionais adicionais.
