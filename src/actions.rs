use gpui_kit::SharedString;
use rdev::{EventType, Key, simulate};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, process::Command, thread::sleep, time::Duration};

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    /// runs a command line, optionally in a visible terminal window
    Command { cmd: SharedString, show_terminal: bool },
    /// strikes a sequence of keys, `delay` ms apart
    Macro { keys: Vec<Key>, delay: u32 },
    /// opens an app from the filesystem
    App { path: PathBuf },
}

#[derive(Clone, Serialize, Deserialize)]
pub enum CellType {
    /// Holdable Cells take in the scroll wheel into account (up and down)
    Holdable,
    /// These are normal clicks, and won't trigger holdable behavior
    Normal,
}

impl Action {
    pub fn run(&self) {
        match self {
            Self::Command { cmd, show_terminal } => run_command(cmd, *show_terminal),
            Self::Macro { keys, delay } => run_keystrokes(keys, *delay),
            Self::App { path } => _ = open::that(path),
        }
    }
}

#[cfg(windows)]
fn run_command(command: &str, show_terminal: bool) {
    use std::{os::windows::process::CommandExt, process::Stdio};
    const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let mut cmd = Command::new("powershell");
    if show_terminal {
        // a fresh console that runs the command and shows its output
        cmd.args(["-NoExit", "-Command", command]).creation_flags(CREATE_NEW_CONSOLE);
    } else {
        cmd.args(["-WindowStyle", "Hidden", "-Command", command])
            .creation_flags(CREATE_NO_WINDOW)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null());
    }
    // TODO: log errors elsewhere
    _ = cmd.spawn();
}

#[cfg(not(windows))]
fn run_command(command: &str, _show_terminal: bool) {
    let mut parts = command.split_whitespace();
    if let Some(program) = parts.next() {
        _ = Command::new(program).args(parts).spawn();
    }
}

fn is_modifier(key: &Key) -> bool {
    matches!(
        key,
        Key::Alt | Key::AltGr | Key::ControlLeft | Key::ControlRight | Key::MetaLeft | Key::MetaRight | Key::ShiftLeft | Key::ShiftRight
    )
}

/// holds the modifiers down while striking the other keys in order
fn run_keystrokes(keys: &[Key], delay: u32) {
    let strike = |key, press| _ = simulate(&if press { EventType::KeyPress(key) } else { EventType::KeyRelease(key) });
    let pause = || sleep(Duration::from_millis(delay.max(10).into()));
    let (modifiers, rest): (Vec<_>, Vec<_>) = keys.iter().copied().partition(is_modifier);

    // TODO: log the keys later for user
    for &key in &modifiers {
        strike(key, true);
        pause();
    }
    for key in rest {
        strike(key, true);
        pause();
        strike(key, false);
        pause();
    }
    for &key in modifiers.iter().rev() {
        strike(key, false);
    }
}

/// readable label for a captured key
pub fn key_label(key: &Key) -> String {
    match key {
        Key::ControlLeft | Key::ControlRight => "ctrl".into(),
        Key::MetaLeft | Key::MetaRight => "meta".into(),
        Key::ShiftLeft | Key::ShiftRight => "shift".into(),
        Key::Return | Key::KpReturn => "enter".into(),
        Key::CapsLock => "caps".into(),
        Key::Delete | Key::KpDelete => "del".into(),
        Key::PrintScreen => "printscr".into(),
        Key::Unknown(code) => format!("unknown_{code}"),
        other => {
            let name = format!("{other:?}");
            name.strip_prefix("Key").or_else(|| name.strip_prefix("Num")).unwrap_or(&name).to_lowercase()
        }
    }
}
