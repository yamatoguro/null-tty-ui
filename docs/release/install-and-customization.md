# Install and Customization Guide (v0.1.0)

## 1) Build release
```bash
source "$HOME/.cargo/env"
cargo build --release
```

Binario gerado:
- target/release/nullbyteui

## Instalacao direta do GitHub (sem clone)
Cole este comando na Raspberry Pi 4:

```bash
bash <(curl -fsSL https://raw.githubusercontent.com/yamatoguro/null-tty-ui/main/deploy/install-from-github.sh)
```

Depois da instalacao, execute:

```bash
null-ui
```

Para sair da interface: pressione `q`.

## 2) Install on Raspberry Pi with systemd
No Raspberry Pi (com permissao root):

```bash
sudo ./deploy/install.sh target/release/nullbyteui
```

O script realiza:
- copia do binario para /opt/nullbyteui/nullbyteui
- copia do layout default para /opt/nullbyteui/config/layout.default.toml
- instalacao do unit file /etc/systemd/system/nullbyteui.service
- enable + start do servico

Comandos uteis:
```bash
sudo systemctl status nullbyteui.service --no-pager
sudo journalctl -u nullbyteui.service -f
```

## 3) Install from packaged artifact
Se voce recebeu o pacote .tar.gz:

```bash
tar -xzf nullbyteui-v0.1.0-linux-<arch>.tar.gz
cd nullbyteui-v0.1.0-linux-<arch>
sudo ./deploy/install.sh ./bin/nullbyteui
```

## 4) Customization
Edite o arquivo:
- /opt/nullbyteui/config/layout.default.toml

Campos comuns:
- regions.<regiao>.plugin: troca plugin por regiao
- dns_host / dns_port / dns_token: integra Technitium local
- terminal_boot_command: comando que roda no painel PTY
- file_nav_root: raiz do file_navigation
- target_fps / target_process_cpu_percent / target_process_rss_mb: metas de desempenho

Depois de alterar:
```bash
sudo systemctl restart nullbyteui.service
```

## 5) Diagnostics
- Arquivo padrao: /tmp/nullbyteui/startup-diagnostics.log
- O runtime registra status periodico com FPS, CPU e RSS do processo.
