use serde::{Deserialize, Serialize};

use super::colors::Colors;
use super::pie_menu::PieMenu;
use crate::ui::settings::ConfigMode;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: ConfigMode,
    pub menus: Vec<PieMenu>,
    pub manual_json: String,
    pub colors: Colors,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: ConfigMode::Auto,
            menus: Vec::new(),
            manual_json: String::new(),
            colors: Colors::DEFAULT,
        }
    }
}

pub struct ConfigGlobal(pub AppConfig);
impl gpui::Global for ConfigGlobal {}
