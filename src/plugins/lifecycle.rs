use crate::core::bus::{Event, EventBus, Topic};
use crate::core::metrics::SystemSnapshot;

/// View model computed by a plugin for rendering in its region.
#[derive(Debug, Clone, Default)]
pub struct PanelViewModel {
    /// Main content lines shown in the panel body.
    pub lines: Vec<String>,
}

impl PanelViewModel {
    /// Creates a view model from a single block of text.
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            lines: text.into().lines().map(str::to_string).collect(),
        }
    }

    /// Renders view model content as a single newline-joined string.
    pub fn render(&self) -> String {
        self.lines.join("\n")
    }
}

/// Lifecycle state tracked for each active plugin instance.
pub struct PluginInstance {
    pub id: String,
    pub region: String,
    last_view: PanelViewModel,
}

impl PluginInstance {
    /// Initializes a plugin instance for a given region binding.
    pub fn init(id: impl Into<String>, region: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            region: region.into(),
            last_view: PanelViewModel::default(),
        }
    }

    /// Called on every scheduler tick; updates view from latest snapshot and emits panel event.
    pub fn update(&mut self, snapshot: &SystemSnapshot, bus: &mut EventBus) {
        let view = self.compute_view(snapshot);
        let changed = view.lines != self.last_view.lines;
        self.last_view = view;

        if changed {
            bus.publish(Event::with_payload(
                Topic::PanelUpdate { region: self.region.clone() },
                self.region.clone(),
            ));
        }
    }

    /// Returns the last computed view model without recomputing.
    pub fn current_view(&self) -> &PanelViewModel {
        &self.last_view
    }

    /// Derives a rendered view from the provided system snapshot.
    /// Each plugin computes its own content from the shared snapshot.
    fn compute_view(&self, snapshot: &SystemSnapshot) -> PanelViewModel {
        let content = match self.id.as_str() {
            "system_overview" => format!(
                "CPU:    {:>6.1}%\nRAM:    {:>4}/{:>4} MB\nTemp:   {}\nUptime: {}",
                snapshot.cpu_percent,
                snapshot.memory_used_mb,
                snapshot.memory_total_mb,
                snapshot
                    .temp_celsius
                    .map(|v| format!("{v:.1} C"))
                    .unwrap_or_else(|| "n/a   ".to_string()),
                format_uptime(snapshot.uptime_secs),
            ),
            "process_list" => format!(
                "Disk:   {:.1}/{:.1} GB\nNet RX: {}\nNet TX: {}\nLoad:   {}",
                snapshot.disk_used_gb,
                snapshot.disk_total_gb,
                format_bytes(snapshot.net_rx_bytes),
                format_bytes(snapshot.net_tx_bytes),
                snapshot
                    .load_avg
                    .map(|(a, b, c)| format!("{a:.2} {b:.2} {c:.2}"))
                    .unwrap_or_else(|| "n/a".to_string()),
            ),
            "technitium_dns_chart" => snapshot
                .dns_summary
                .clone()
                .unwrap_or_else(|| "DNS: connecting...".to_string()),
            "file_navigation" => snapshot
                .file_nav_summary
                .clone()
                .unwrap_or_else(|| "file_navigation: connecting...".to_string()),
            "log_stream" => snapshot.recent_logs.join("\n"),
            _ => format!("[{}] no data", self.id),
        };

        PanelViewModel::from_text(content)
    }

    /// Called on process exit or plugin removal to free any held resources.
    pub fn dispose(&self) {
        // Stateless plugins have nothing to release.
    }
}

/// Formats seconds into d/h/m/s human readable string.
fn format_uptime(secs: u64) -> String {
    let d = secs / 86400;
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if d > 0 {
        format!("{d}d {h:02}:{m:02}:{s:02}")
    } else {
        format!("{h:02}:{m:02}:{s:02}")
    }
}

/// Formats raw bytes into human-readable KB/MB/GB suffix.
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{bytes} B")
    }
}

