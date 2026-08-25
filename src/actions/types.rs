use rdev::Key;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    /// Executes a cmd
    Command(gpui::SharedString),
    /// Executes a sequence of keystrokes
    Macro {
        /// The keys to press
        keys: Vec<Key>,
        /// Delay in milliseconds between key strikes if needed
        delay: u32,
    },
    /// Executes an app from the filesystem
    App { path: PathBuf },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CellType {
    /// Holdable Cells take in the scroll wheel into account (up and down)
    Holdable,
    /// These are normal clicks, and won't trigger holdable behavior
    Normal,
}
