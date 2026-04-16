use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

use crate::config::layout::LayoutConfig;
use crate::plugins::manifest::PluginManifest;

/// Stores resolved plugin bindings from layout regions.
pub struct PluginManager {
    plugin_by_region: HashMap<String, String>,
}

impl PluginManager {
    /// Resolves plugin ids from layout regions and validates basic invariants.
    pub fn new(layout: &LayoutConfig) -> anyhow::Result<Self> {
        if layout.regions.is_empty() {
            bail!("layout must define at least one region");
        }

        let manifests = load_manifests(Path::new("plugins"))?;
        validate_manifests(&manifests)?;

        let mut plugin_by_region = HashMap::new();
        for region in ["top", "left", "center", "right", "bottom"] {
            let plugin_id = layout
                .plugin_for_region(region)
                .ok_or_else(|| anyhow::anyhow!("missing plugin binding for region {region}"))?;
            plugin_by_region.insert(region.to_string(), plugin_id.to_string());
        }

        for plugin_id in plugin_by_region.values() {
            if !manifests.contains_key(plugin_id) {
                bail!("configured plugin not found: {plugin_id}");
            }
        }

        Ok(Self { plugin_by_region })
    }

    /// Returns all (region, plugin_id) pairs ordered by layout declaration.
    pub fn region_plugin_pairs(&self) -> Vec<(String, String)> {
        ["top", "left", "center", "right", "bottom"]
            .iter()
            .filter_map(|r| {
                self.plugin_by_region
                    .get(*r)
                    .map(|p| (r.to_string(), p.clone()))
            })
            .collect()
    }

}

/// Validates manifest metadata and consumes required fields used across runtime and docs.
fn validate_manifests(manifests: &HashMap<String, PluginManifest>) -> anyhow::Result<()> {
    for (id, manifest) in manifests {
        if manifest.id.trim().is_empty() || manifest.id != *id {
            bail!("plugin manifest id mismatch or empty: {id}");
        }
        if manifest.version.trim().is_empty() {
            bail!("plugin manifest version missing: {id}");
        }
        if manifest.title.trim().is_empty() {
            bail!("plugin manifest title missing: {id}");
        }
        if manifest.description.trim().is_empty() {
            bail!("plugin manifest description missing: {id}");
        }
        if manifest.update_interval_ms == 0 {
            bail!("plugin manifest update_interval_ms must be > 0: {id}");
        }
        if manifest.permissions.iter().any(|p| p.trim().is_empty()) {
            bail!("plugin manifest has empty permission value: {id}");
        }
    }
    Ok(())
}

/// Loads all plugin manifests found in the plugins directory.
fn load_manifests(plugins_dir: &Path) -> anyhow::Result<HashMap<String, PluginManifest>> {
    let mut manifests = HashMap::new();

    if !plugins_dir.exists() {
        return Ok(manifests);
    }

    let entries = std::fs::read_dir(plugins_dir)
        .with_context(|| format!("failed reading plugins dir {}", plugins_dir.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "failed reading plugin entry")?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path: PathBuf = path.join("manifest.toml");
        if !manifest_path.exists() {
            continue;
        }

        let manifest = PluginManifest::load(&manifest_path)?;
        manifests.insert(manifest.id.clone(), manifest);
    }

    Ok(manifests)
}
