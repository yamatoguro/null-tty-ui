mod config;
mod core;
mod plugins;

use anyhow::Context;
use config::layout::LayoutConfig;
use core::app::AppRuntime;

/// Parses supported CLI arguments and returns config file path.
fn parse_config_path() -> String {
    let mut args = std::env::args().skip(1);
    let mut config_path = "config/layout.default.toml".to_string();

    while let Some(arg) = args.next() {
        if arg == "--config" {
            if let Some(value) = args.next() {
                config_path = value;
            }
        }
    }

    config_path
}

/// Bootstraps the application by loading config, initializing runtime, and launching UI.
fn main() -> anyhow::Result<()> {
    let config_path = parse_config_path();

    let layout = LayoutConfig::load_from_file(&config_path)
        .context("failed to load layout config")?;

    let runtime = AppRuntime::new(layout).context("failed to initialize runtime")?;
    runtime.run().context("runtime terminated with error")
}
