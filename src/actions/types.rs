use rdev::Key;
use std::path::PathBuf;

#[derive(Clone)]
pub enum Action {
    /// Executes a cmd
    Command(String),
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

pub enum Cell {
    /// Holdable Cells take in the scroll wheel into account (up and down)
    Holdable,
    /// These are normal clicks, and won't trigger holdable behavior
    Normal,
}
