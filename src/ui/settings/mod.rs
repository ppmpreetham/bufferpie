use serde::{Deserialize, Serialize};
pub mod appearance;
pub mod config_tab;
pub mod manual;
pub mod menus;
pub mod viewer;
pub mod window;
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigMode {
    Auto,
    Manual,
}

use super::config::AppConfig;
use std::fs;
use std::path::PathBuf;

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("cyber_pie_daemon")
        .join("config.json")
}

pub fn load() -> AppConfig {
    let path = config_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(config: &AppConfig) -> anyhow::Result<()> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)?;
    fs::write(path, json)?;
    Ok(())
}
