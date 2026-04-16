use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::Deserialize;

/// Describes plugin metadata loaded from plugins/*/manifest.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub update_interval_ms: u64,
    pub permissions: Vec<String>,
}

impl PluginManifest {
    /// Loads and parses one plugin manifest file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed reading manifest {}", path.display()))?;
        let parsed: Self = toml::from_str(&raw)
            .with_context(|| format!("invalid manifest TOML {}", path.display()))?;
        Ok(parsed)
    }
}
