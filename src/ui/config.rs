use serde::{Deserialize, Serialize};

use super::colors::Colors;
use super::pie_menu::{Item, PieMenu};
use crate::actions::types::CellType;
use crate::ui::settings::ConfigMode;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: ConfigMode,
    pub menus: Vec<PieMenu>,
    pub colors: Colors,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: ConfigMode::Auto,
            menus: default_menus(),
            colors: Colors::DEFAULT,
        }
    }
}

fn node(label: &str) -> Item {
    Item {
        label: label.into(),
        action: None,
        celltype: CellType::Normal,
    }
}

/// all menus i guess
pub fn default_menus() -> Vec<PieMenu> {
    [
        ("EDIT", &["Copy", "Paste", "Cut", "Undo"][..]),
        ("WINDOW", &["Minimize", "Maximize", "Close"][..]),
    ]
    .map(|(name, items)| PieMenu {
        name: name.into(),
        items: items.iter().map(|l| node(l)).collect(),
    })
    .into()
}
