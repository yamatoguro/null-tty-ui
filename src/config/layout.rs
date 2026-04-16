use std::collections::HashMap;
use std::fs;

use anyhow::Context;
use serde::Deserialize;

/// Represents one layout region binding to a plugin id.
#[derive(Debug, Clone, Deserialize)]
pub struct RegionConfig {
    pub plugin: String,
}

/// Represents the complete layout definition loaded from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct LayoutConfig {
    pub schema_version: u32,
    pub profile: String,
    pub regions: HashMap<String, RegionConfig>,
    /// Hostname of the Technitium DNS instance (optional, default "localhost").
    pub dns_host: Option<String>,
    /// TCP port for Technitium API (optional, default 5380).
    pub dns_port: Option<u16>,
    /// API token for authenticating Technitium requests (optional).
    pub dns_token: Option<String>,
    /// Optional diagnostics log output file path.
    pub diagnostics_log_path: Option<String>,
    /// Minimum expected rendered frames-per-second.
    pub target_fps: Option<f32>,
    /// Maximum expected process CPU usage percent.
    pub target_process_cpu_percent: Option<f32>,
    /// Maximum expected process RSS in MB.
    pub target_process_rss_mb: Option<u64>,
    /// Optional startup command executed inside the PTY panel.
    pub terminal_boot_command: Option<String>,
    /// Optional root path for file navigation plugin.
    pub file_nav_root: Option<String>,
}

const REQUIRED_REGIONS: [&str; 5] = ["top", "left", "center", "right", "bottom"];

impl LayoutConfig {
    /// Loads a layout configuration from disk and parses it as TOML.
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path).with_context(|| format!("failed reading {path}"))?;
        let parsed: Self = toml::from_str(&raw).with_context(|| format!("invalid TOML in {path}"))?;
        parsed.validate().with_context(|| format!("invalid layout in {path}"))?;
        Ok(parsed)
    }

    /// Validates schema and required regions.
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.schema_version == 0 {
            anyhow::bail!("schema_version must be greater than zero");
        }

        for region in REQUIRED_REGIONS {
            if !self.regions.contains_key(region) {
                anyhow::bail!("missing required region: {region}");
            }
        }

        Ok(())
    }

    /// Returns the plugin id configured for a known region.
    pub fn plugin_for_region(&self, region: &str) -> Option<&str> {
        self.regions.get(region).map(|item| item.plugin.as_str())
    }
}
