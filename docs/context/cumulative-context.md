# Cumulative Context Log

This file is append-only and is the single source of progress truth during the project lifecycle.

## Entry Template
- date:
- interaction_id:
- tags:
- what:
- how:
- why:
- files:
- next:

## Entry 001
- date: 2026-04-15
- interaction_id: 001
- tags: [project-init, planning, governance]
- what: Created project process docs and context governance baseline.
- how: Added markdown documentation for vision, architecture, plugin model, performance, and Technitium integration.
- why: Establish strict process clarity before implementing runtime code.
- files: docs/process/01-product-vision.md, docs/process/02-technical-architecture.md, docs/process/03-plugin-system.md, docs/process/04-performance-and-boot.md, docs/process/05-technitium-integration.md, docs/context/agent-process-rules.md
- next: Define coding rules, backlog, and initial binary scaffold.

## Entry 002
- date: 2026-04-15
- interaction_id: 002
- tags: [coding-standards, backlog, scaffold, rust, slint]
- what: Added coding standards, backlog roadmap, agent enforcement files, and initial native Rust project scaffold.
- how: Created markdown rules and backlog docs, then scaffolded Cargo project with Slint UI, layout config loader, and plugin manager baseline.
- why: Start implementation immediately with enforceable process governance and an executable architecture foundation.
- files: docs/planning/coding-rules.md, docs/planning/backlog.md, .github/copilot-instructions.md, AGENTS.md, Cargo.toml, build.rs, ui/main.slint, config/layout.default.toml, src/main.rs, src/config/layout.rs, src/core/app.rs, src/plugins/manager.rs
- next: Implement event bus, collector scheduler, richer UI region binding, and first built-in plugins.

## Entry 003
- date: 2026-04-15
- interaction_id: 003
- tags: [runtime, terminal-ui, plugins, metrics, build]
- what: Replaced initial UI runtime with a fully working native terminal dashboard, implemented live Linux metrics collection, and enabled manifest-based plugin panel rendering.
- how: Added ratatui/crossterm runtime loop, created metrics collector module, loaded plugin manifests from plugins directory, enforced 5-region layout validation, and populated default plugin manifests.
- why: Deliver a running lightweight binary path now, avoid heavy system-level graphical dependencies, and keep architecture ready for future plugin and Technitium graph expansion.
- files: Cargo.toml, build.rs, src/main.rs, src/core/app.rs, src/core/metrics.rs, src/config/layout.rs, src/plugins/manifest.rs, src/plugins/manager.rs, src/plugins/mod.rs, docs/planning/backlog.md, README.md, plugins/system_overview/manifest.toml, plugins/process_list/manifest.toml, plugins/terminal/manifest.toml, plugins/technitium_dns_chart/manifest.toml, plugins/log_stream/manifest.toml
- next: Implement PTY center panel, HTTP client for Technitium DNS metrics, and plugin lifecycle hooks with hot reload.

## Entry 004
- date: 2025-07-22
- interaction_id: 004
- tags: M4, M5, M6, dns, systemd, metrics
- what: Implemented Technitium DNS HTTP client (M5), wired it into the render loop, added optional dns_host/dns_port/dns_token fields to LayoutConfig, created systemd service unit and install script (M6).
- how: src/core/dns.rs uses raw TcpStream for HTTP/1.0 GET to Technitium API; background thread updates Arc<Mutex<DnsState>>; render_dns_panel() builds ASCII sparkline; TerminalUi reads shared state each tick; LayoutConfig deserialization extended with Option<> fields; deploy/nullbyteui.service targets /dev/tty1 with TERM=linux.
- why: M5 closes the DNS chart panel requirement without an async runtime. M6 enables automatic fullscreen boot on the Raspberry Pi via systemd.
- files: src/core/dns.rs, src/core/mod.rs, src/core/app.rs, src/config/layout.rs, config/layout.default.toml, deploy/nullbyteui.service, deploy/install.sh
- next: Cross-compile for aarch64-unknown-linux-gnu, run cargo test, freeze schema, write release guide (M7).

## Entry 005
- date: 2026-04-15
- interaction_id: 005
- tags: [m5, m6, diagnostics, hardening, cli]
- what: Completed M6 hardening and diagnostics flow, finalized M5 DNS threshold alerts, and synchronized backlog status with current implementation.
- how: Added runtime monitor for process CPU/RAM/FPS diagnostics with periodic logging; wired monitor into UI frame loop; added optional layout targets and diagnostics log path; added CLI --config parsing in main; hardened systemd unit with sandbox flags; updated DNS panel with alert thresholds.
- why: Ensure reliable boot/runtime operation on Raspberry Pi, measure health against explicit targets, and keep delivery tracking aligned with actual code state.
- files: src/core/diagnostics.rs, src/core/app.rs, src/main.rs, src/config/layout.rs, src/core/mod.rs, src/core/dns.rs, config/layout.default.toml, deploy/nullbyteui.service, docs/planning/backlog.md
- next: Execute M7 release path: package binary/assets, publish install guide, and freeze v1 config schema.

## Entry 006
- date: 2026-04-15
- interaction_id: 006
- tags: [m4, pty, file-navigation, performance, io-optimization]
- what: Completed remaining M4 modules with a real PTY-backed terminal stream and file navigation plugin, while improving runtime performance by switching log collection to incremental reads.
- how: Added src/core/pty.rs (forkpty + shell capture), src/core/file_nav.rs (background directory polling), injected terminal/file panel summaries into SystemSnapshot via app runtime, added layout options for terminal boot command and file navigation root, and replaced full log-file reads with cursor-based incremental polling in metrics collector.
- why: Finish M4 feature scope and reduce per-tick disk I/O overhead to keep the dashboard responsive and lightweight on Raspberry Pi.
- files: src/core/pty.rs, src/core/file_nav.rs, src/core/app.rs, src/core/metrics.rs, src/core/mod.rs, src/config/layout.rs, src/plugins/lifecycle.rs, plugins/file_navigation/manifest.toml, config/layout.default.toml, docs/planning/backlog.md
- next: Continue M7 with packaging/install docs and optional dead-code cleanup to reduce warnings before release.

## Entry 007
- date: 2026-04-15
- interaction_id: 007
- tags: [warnings, m7, release, packaging, install]
- what: Removed all compiler warnings, completed M7 release tasks, generated first installable package v0.1.0, and performed a local no-root installation validation.
- how: Trimmed unused bus/plugin APIs, consumed event payloads in runtime loop, validated plugin manifest metadata, integrated DNS sample timestamp in panel, produced schema freeze docs plus JSON schema, added release packaging script, generated release tarball and checksum, and added local installer script for fast validation.
- why: Deliver a clean warning-free codebase and a reproducible first-version release/install flow.
- files: src/core/bus.rs, src/core/app.rs, src/core/dns.rs, src/plugins/lifecycle.rs, src/plugins/manager.rs, docs/planning/backlog.md, README.md, config/schema/layout.v1.json, docs/spec/layout-schema-v1.md, docs/release/install-and-customization.md, scripts/release/package.sh, deploy/install-local.sh
- next: Optional next step is cross-build/package for aarch64 Raspberry Pi target and execute systemd install on device.

## Entry 008
- date: 2026-04-15
- interaction_id: 008
- tags: [git, repository-init, commit-history, github-push]
- what: Initialized local Git repository, corrected Git author identity to the user-provided values, organized the project into milestone commits, and connected/pushed to the GitHub remote repository.
- how: Verified Git installation, ran git init on main, set local user.name/user.email, committed docs/governance and implementation/release tooling in separate commits, configured origin to github.com/yamatoguro/null-tty-ui.git, and pushed main with upstream tracking.
- why: Ensure traceable local history and publish all project work to the target remote repository.
- files: .gitignore, docs/context/cumulative-context.md
- next: Continue normal development flow with focused commits per feature/fix and push incrementally.

## Entry 009
- date: 2026-04-15
- interaction_id: 009
- tags: [installer, github, raspberry-pi, one-liner]
- what: Added a direct GitHub installer script that works without cloning and provides a global `null-ui` command after install.
- how: Created deploy/install-from-github.sh to download source tarball from GitHub, ensure Rust toolchain, build release, install files under /opt/nullbyteui, and create /usr/local/bin/null-ui wrapper; updated install guide with a one-line command for Raspberry Pi.
- why: Enable frictionless install/test flow by pasting a single command into terminal on Raspberry Pi 4.
- files: deploy/install-from-github.sh, docs/release/install-and-customization.md, docs/context/cumulative-context.md
- next: Optionally add a release-asset installer path to avoid local compile time on low-power devices.

## Entry 010
- date: 2026-04-15
- interaction_id: 010
- tags: [readme, install, github-one-liner]
- what: Added direct installation command from GitHub to README with immediate run command.
- how: Updated installation section to include one-line curl+bash command pointing to deploy/install-from-github.sh and documented `null-ui` as the execution command.
- why: Make onboarding faster by keeping copy-paste install instructions visible at the repository entry point.
- files: README.md, docs/context/cumulative-context.md
- next: Keep README and install guide synchronized whenever install flow changes.

## Entry 011
- date: 2026-04-15
- interaction_id: 011
- tags: [installer, bugfix, runtime-path, plugins]
- what: Fixed one-line installer wrapper so `null-ui` starts from `/opt/nullbyteui`, preventing plugin manifest lookup failures.
- how: Updated generated `/usr/local/bin/null-ui` wrapper in deploy/install-from-github.sh to `cd /opt/nullbyteui` before launching the binary with config path.
- why: Runtime plugin manager uses relative `plugins/` path; running outside install dir caused errors like `configured plugin not found` for random plugins.
- files: deploy/install-from-github.sh, docs/context/cumulative-context.md
- next: Consider making plugin path absolute in runtime for extra robustness.

## Entry 012
- date: 2026-04-15
- interaction_id: 012
- tags: [installer, self-healing, idempotent, recovery]
- what: Added countermeasures in the GitHub one-line installer to repair broken prior installs when the script is re-run.
- how: Introduced `CLEAN_INSTALL=1` optional clean mode, repair path detection, full plugin directory refresh to remove stale/corrupt leftovers, dynamic wrapper generation using INSTALL_DIR, and post-install validation for required and layout-referenced plugins.
- why: Ensure re-running the same installer recovers inconsistent installations instead of failing with random plugin-not-found errors.
- files: deploy/install-from-github.sh, docs/context/cumulative-context.md
- next: Consider adding a `--self-test` runtime flag to validate config/plugins without opening the UI.

## Entry 013
- date: 2026-04-15
- interaction_id: 013
- tags: [installer, cli-args, clean-reinstall, docs]
- what: Added explicit `--clean` CLI parameter to the GitHub one-line installer and documented command examples.
- how: Implemented argument parser (`parse_args`) with `--clean`, `--ref`, `--repo`, and `--help`; updated README and install guide with one-liner commands including forced clean reinstall mode.
- why: Guarantee users can trigger clean repair directly by parameter when running installer via `bash <(curl ...)`.
- files: deploy/install-from-github.sh, README.md, docs/release/install-and-customization.md, docs/context/cumulative-context.md
- next: Optionally support additional flags for non-sudo/local installs in restricted environments.

## Entry 014
- date: 2026-04-15
- interaction_id: 014
- tags: [runtime, terminal-removal, dns-fix, diagnostics]
- what: Removed terminal/PTY module entirely from runtime and fixed DNS updater to use resilient endpoint fallback + JSON parsing with error logging in default diagnostics directory.
- how: Deleted PTY module and references, switched center region default to `file_navigation`, removed `terminal_boot_command` from config/schema/docs, updated installer required plugins, rewrote DNS client to try multiple Technitium endpoints and parse nested JSON with `serde_json`, and appended DNS poller errors/recovery to `/tmp/nullbyteui/startup-diagnostics.log`.
- why: Stop frame-by-frame terminal updates completely and make DNS panel actually refresh with real data while exposing actionable logs in the standard path.
- files: src/core/app.rs, src/core/mod.rs, src/core/metrics.rs, src/core/dns.rs, src/plugins/lifecycle.rs, src/config/layout.rs, config/layout.default.toml, config/schema/layout.v1.json, deploy/install-from-github.sh, src/core/pty.rs (removed), plugins/terminal (removed), README.md, docs/release/install-and-customization.md, docs/spec/layout-schema-v1.md, Cargo.toml, docs/context/cumulative-context.md
- next: Optionally add a DNS self-test command that verifies API connectivity and token before launching UI.

## Entry 015
- date: 2026-04-15
- interaction_id: 015
- tags: [ui, charts, logs, layout-fit]
- what: Implemented functional time-series graphs across system/process panels and enforced per-panel text fitting so log lines are wrapped/clipped inside their designated layout area.
- how: Added runtime history buffers (CPU/mem/disk/net/load/DNS), injected those series into SystemSnapshot each tick, rendered sparklines in plugin lifecycle views, and introduced area-aware text formatting in app renderer (`fit_text_to_area`) to wrap long lines and clip overflow by panel width/height.
- why: Deliver visible real-time graphs and prevent log/content overflow from breaking panel boundaries.
- files: src/core/app.rs, src/core/metrics.rs, src/plugins/lifecycle.rs, src/core/dns.rs, docs/context/cumulative-context.md
- next: Optional enhancement is adding true ratatui chart widgets per panel for axis/grid legends.

## Entry 016
- date: 2026-04-15
- interaction_id: 016
- tags: [ui, ratatui, charts, widget-upgrade]
- what: Replaced text-based sparklines with native ratatui `Sparkline` widgets and organized chart regions per panel.
- how: Updated app renderer to split top/left/right regions into text + chart areas, rendering widget charts for CPU/MEM/LOAD, DISK/RX/TX, and DNS queries; simplified plugin textual content accordingly while preserving area-fit clipping/wrapping.
- why: Improve readability and chart stability with native widgets while keeping logs and text bounded to designated panel dimensions.
- files: src/core/app.rs, src/plugins/lifecycle.rs, docs/context/cumulative-context.md
- next: Optional next step is migrating sparkline widgets to full `Chart` widgets with axes and labels.
