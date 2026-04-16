use std::io;
use std::time::{Duration, Instant};

use anyhow::Context;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::{Frame, Terminal};

use crate::config::layout::LayoutConfig;
use crate::core::bus::{EventBus, Topic};
use crate::core::diagnostics::{PerformanceTargets, RuntimeMonitor};
use crate::core::dns::{self, DnsState};
use crate::core::file_nav::{self, FileNavState};
use crate::core::metrics::{MetricsCollector, SystemSnapshot};
use crate::plugins::lifecycle::PluginInstance;
use crate::plugins::manager::PluginManager;

/// Coordinates startup: config load, plugin init, event bus wiring, and UI launch.
pub struct AppRuntime {
    layout: LayoutConfig,
    plugin_manager: PluginManager,
}

impl AppRuntime {
    /// Creates the runtime with validated layout and initialized plugin registry.
    pub fn new(layout: LayoutConfig) -> anyhow::Result<Self> {
        let plugin_manager = PluginManager::new(&layout).context("failed to build plugin manager")?;
        Ok(Self { layout, plugin_manager })
    }

    /// Runs a realtime fullscreen terminal dashboard and exits on q.
    pub fn run(self) -> anyhow::Result<()> {
        let mut ui = TerminalUi::new(self.layout, self.plugin_manager)?;
        ui.run_loop()
    }
}

/// Holds all live runtime state for the terminal dashboard.
struct TerminalUi {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    layout: LayoutConfig,
    bus: EventBus,
    collector: MetricsCollector,
    instances: Vec<PluginInstance>,
    dns_state: std::sync::Arc<std::sync::Mutex<DnsState>>,
    file_nav_state: std::sync::Arc<std::sync::Mutex<FileNavState>>,
    monitor: RuntimeMonitor,
}

impl TerminalUi {
    /// Sets up raw terminal, instantiates plugin instances, and wires event bus.
    fn new(layout: LayoutConfig, plugin_manager: PluginManager) -> anyhow::Result<Self> {
        enable_raw_mode().context("failed enabling raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).context("failed entering alternate screen")?;

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).context("failed creating terminal backend")?;

        let instances = plugin_manager
            .region_plugin_pairs()
            .into_iter()
            .map(|(region, plugin_id)| PluginInstance::init(plugin_id, region))
            .collect();

        // Start DNS poller with defaults; token left empty if not configured.
        let dns_host = layout.dns_host.clone().unwrap_or_else(|| "localhost".to_string());
        let dns_port = layout.dns_port.unwrap_or(5380);
        let dns_token = layout.dns_token.clone().unwrap_or_default();
        let dns_state = dns::start_poller(&dns_host, dns_port, &dns_token, Duration::from_millis(2000));

        let file_nav_root = std::path::PathBuf::from(
            layout
                .file_nav_root
                .clone()
                .unwrap_or_else(|| "/home/pi".to_string()),
        );
        let file_nav_state = file_nav::start_file_nav_poller(file_nav_root, Duration::from_secs(3));

        let monitor = RuntimeMonitor::new(
            layout.diagnostics_log_path.clone(),
            PerformanceTargets {
                min_fps: layout.target_fps.unwrap_or(2.0),
                max_process_cpu_percent: layout.target_process_cpu_percent.unwrap_or(20.0),
                max_process_rss_mb: layout.target_process_rss_mb.unwrap_or(180),
            },
        );

        Ok(Self {
            terminal,
            layout,
            bus: EventBus::new(128),
            collector: MetricsCollector::new(),
            instances,
            dns_state,
            file_nav_state,
            monitor,
        })
    }

    /// Runs the fixed-cadence input + collect + render loop until the user quits.
    fn run_loop(&mut self) -> anyhow::Result<()> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(500);

        loop {
            // Read latest DNS state from background thread.
            let dns_rendered = {
                let s = self.dns_state.lock().unwrap();
                Some(dns::render_dns_panel(&s))
            };

            let mut snapshot = self.collector.collect();
            snapshot.dns_summary = dns_rendered.clone();

            let file_nav_rendered = {
                let s = self.file_nav_state.lock().unwrap();
                Some(file_nav::render_file_nav_panel(&s))
            };
            snapshot.file_nav_summary = file_nav_rendered;

            // Drive each plugin lifecycle with the new snapshot.
            for instance in &mut self.instances {
                instance.update(&snapshot, &mut self.bus);
            }

            // Consume event metadata for lightweight runtime telemetry.
            let mut panel_updates = 0usize;
            for ev in self.bus.drain() {
                match ev.topic {
                    Topic::PanelUpdate { region } => {
                        let _payload_region = ev.payload.unwrap_or(region);
                        panel_updates = panel_updates.saturating_add(1);
                    }
                }
            }

            // Build per-region content strings from live plugin view models.
            let panel = |region: &str| -> String {
                self.instances
                    .iter()
                    .find(|i| i.region == region)
                    .map(|i| i.current_view().render())
                    .unwrap_or_default()
            };

            let top = panel("top");
            let left = panel("left");
            let center = panel("center");
            let right = panel("right");
            let bottom = panel("bottom");
            let status = build_status_line(&self.layout.profile, &snapshot, panel_updates);

            self.terminal
                .draw(|frame| Self::render_frame(frame, &top, &left, &center, &right, &bottom, &status))
                .context("failed drawing terminal frame")?;
            self.monitor.on_frame_rendered();
            self.monitor.report_if_due(Duration::from_secs(5));

            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout).context("failed polling events")? {
                if let Event::Key(key) = event::read().context("failed reading event")? {
                    if key.kind == KeyEventKind::Press && matches!(key.code, KeyCode::Char('q')) {
                        break;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        for instance in &self.instances {
            instance.dispose();
        }

        self.restore_terminal()
    }

    /// Renders all five UI regions into the terminal frame.
    fn render_frame(
        frame: &mut Frame<'_>,
        top: &str,
        left: &str,
        center: &str,
        right: &str,
        bottom: &str,
        status: &str,
    ) {
        let root = frame.area();
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(7),
                Constraint::Min(10),
                Constraint::Length(7),
            ])
            .split(root);

        let middle = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ])
            .split(vertical[2]);

        frame.render_widget(
            Paragraph::new("NULLBYTE UI  |  q = exit  |  layout: config/layout.default.toml")
                .block(Block::default().borders(Borders::ALL).title(" NULLBYTEUI "))
                .style(Style::default().fg(Color::Cyan)),
            vertical[0],
        );

        frame.render_widget(
            Paragraph::new(top)
                .block(Block::default().borders(Borders::ALL).title(" system_overview "))
                .style(Style::default().fg(Color::LightCyan)),
            vertical[1],
        );

        frame.render_widget(
            Paragraph::new(left)
                .block(Block::default().borders(Borders::ALL).title(" process_list "))
                .style(Style::default().fg(Color::Green)),
            middle[0],
        );

        frame.render_widget(
            Paragraph::new(center)
                .block(Block::default().borders(Borders::ALL).title(" file_navigation "))
                .style(Style::default().fg(Color::White)),
            middle[1],
        );

        frame.render_widget(
            Paragraph::new(right)
                .block(Block::default().borders(Borders::ALL).title(" technitium_dns "))
                .style(Style::default().fg(Color::LightYellow)),
            middle[2],
        );

        frame.render_widget(
            Paragraph::new(format!("{bottom}\n{status}"))
                .block(Block::default().borders(Borders::ALL).title(" log_stream "))
                .style(Style::default().fg(Color::DarkGray)),
            vertical[3],
        );
    }

    /// Restores terminal to normal mode on clean exit.
    fn restore_terminal(&mut self) -> anyhow::Result<()> {
        disable_raw_mode().context("failed disabling raw mode")?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)
            .context("failed leaving alternate screen")?;
        self.terminal.show_cursor().context("failed showing cursor")?;
        Ok(())
    }
}

impl Drop for TerminalUi {
    /// Best-effort cleanup in case of early return or panic path.
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Builds the bottom status line from live snapshot data.
fn build_status_line(profile: &str, snap: &SystemSnapshot, panel_updates: usize) -> String {
    format!(
        "profile={} cpu={:.1}% mem={}/{}MB temp={} up={}s rx={:.1}KB tx={:.1}KB load={} upd={}",
        profile,
        snap.cpu_percent,
        snap.memory_used_mb,
        snap.memory_total_mb,
        snap.temp_celsius.map(|v| format!("{v:.1}C")).unwrap_or_else(|| "n/a".to_string()),
        snap.uptime_secs,
        snap.net_rx_bytes as f64 / 1024.0,
        snap.net_tx_bytes as f64 / 1024.0,
        snap.load_avg
            .map(|(a, _b, _c)| format!("{a:.2}"))
            .unwrap_or_else(|| "n/a".to_string()),
        panel_updates,
    )
}
