use super::{app_open::open_app, cmd::run_command, keystrokes::run_keystrokes};
use rdev::Key;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    /// executes a cmd
    Command {
        /// the command line to run
        cmd: gpui::SharedString,
        /// keeps the spawned terminal window visible
        show_terminal: bool,
    },
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

/// runs the behavior bound to an action
pub fn execute(action: &Action) {
    match action {
        Action::Command { cmd, show_terminal } => run_command(cmd, *show_terminal),
        Action::Macro { keys, delay } => run_keystrokes(keys, *delay),
        Action::App { path } => open_app(path.to_string_lossy().as_ref()),
    }
}
