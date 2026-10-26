use crate::actions::{Action, CellType};
use gpui_kit::{App, Entity, SharedString};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConfigMode {
    Auto,
    Manual,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Colors {
    pub arc: u32,
    pub surface: u32,
    pub surface_hover: u32,
    pub text: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Item {
    pub label: SharedString,
    pub action: Option<Action>,
    pub celltype: CellType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PieMenu {
    pub name: SharedString,
    pub items: Vec<Item>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: ConfigMode,
    pub menus: Vec<PieMenu>,
    pub colors: Colors,
}

impl Default for AppConfig {
    fn default() -> Self {
        let menu = |name: &'static str, labels: &[&'static str]| PieMenu {
            name: name.into(),
            items: labels.iter().map(|&label| Item { label: label.into(), action: None, celltype: CellType::Normal }).collect(),
        };
        Self {
            mode: ConfigMode::Auto,
            menus: vec![menu("EDIT", &["Copy", "Paste", "Cut", "Undo"]), menu("WINDOW", &["Minimize", "Maximize", "Close"])],
            colors: Colors { arc: 0x4772b3, surface: 0x181818, surface_hover: 0x545454, text: 0xcdd6f4 },
        }
    }
}

fn path() -> PathBuf {
    dirs::config_dir().unwrap_or_else(std::env::temp_dir).join("buffer_pie_daemon").join("config.json")
}

pub fn load() -> AppConfig {
    fs::read_to_string(path()).ok().and_then(|json| serde_json::from_str(&json).ok()).unwrap_or_default()
}

fn save(config: &AppConfig) -> io::Result<()> {
    let path = path();
    fs::create_dir_all(path.parent().expect("config path has a parent"))?;
    fs::write(path, serde_json::to_string_pretty(config)?)
}

/// changes the config, saves it and repaints whatever shows it
pub fn edit(config: &Entity<AppConfig>, cx: &mut App, change: impl FnOnce(&mut AppConfig)) {
    config.update(cx, |config, cx| {
        change(config);
        _ = save(config);
        cx.notify();
    });
}
