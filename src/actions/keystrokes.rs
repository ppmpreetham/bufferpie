use rdev::{EventType, Key, SimulateError, simulate};
use std::thread::sleep;
use std::time::Duration;

/// readable label for a captured key
pub fn key_label(key: Key) -> &'static str {
    match key {
        Key::Alt => "alt",
        Key::AltGr => "altgr",
        Key::Backspace => "backspace",
        Key::CapsLock => "caps",
        Key::ControlLeft | Key::ControlRight => "ctrl",
        Key::Delete => "del",
        Key::DownArrow => "down",
        Key::End => "end",
        Key::Escape => "esc",
        Key::F1 => "f1",
        Key::F2 => "f2",
        Key::F3 => "f3",
        Key::F4 => "f4",
        Key::F5 => "f5",
        Key::F6 => "f6",
        Key::F7 => "f7",
        Key::F8 => "f8",
        Key::F9 => "f9",
        Key::F10 => "f10",
        Key::F11 => "f11",
        Key::F12 => "f12",
        Key::Home => "home",
        Key::LeftArrow => "left",
        Key::MetaLeft | Key::MetaRight => "meta",
        Key::PageDown => "pagedown",
        Key::PageUp => "pageup",
        Key::Return => "enter",
        Key::RightArrow => "right",
        Key::ShiftLeft | Key::ShiftRight => "shift",
        Key::Space => "space",
        Key::Tab => "tab",
        Key::UpArrow => "up",
        Key::PrintScreen => "printscr",
        Key::ScrollLock => "scrolllock",
        Key::Pause => "pause",
        Key::NumLock => "numlock",
        Key::BackQuote => "`",
        Key::Minus => "-",
        Key::Equal => "=",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Num0 => "0",
        Key::KeyA => "a",
        Key::KeyB => "b",
        Key::KeyC => "c",
        Key::KeyD => "d",
        Key::KeyE => "e",
        Key::KeyF => "f",
        Key::KeyG => "g",
        Key::KeyH => "h",
        Key::KeyI => "i",
        Key::KeyJ => "j",
        Key::KeyK => "k",
        Key::KeyL => "l",
        Key::KeyM => "m",
        Key::KeyN => "n",
        Key::KeyO => "o",
        Key::KeyP => "p",
        Key::KeyQ => "q",
        Key::KeyR => "r",
        Key::KeyS => "s",
        Key::KeyT => "t",
        Key::KeyU => "u",
        Key::KeyV => "v",
        Key::KeyW => "w",
        Key::KeyX => "x",
        Key::KeyY => "y",
        Key::KeyZ => "z",
        _ => "?",
    }
}

/// strikes a sequence of keys with an optional delay in between
pub fn run_keystrokes(keys: &[Key], delay: u32) {
    let delay_duration = Duration::from_millis(delay as u64);

    // TODO: log the keys later for user
    for key in keys {
        let _ = strike(*key, true);
        if delay > 0 {
            sleep(delay_duration);
        }
        let _ = strike(*key, false);
    }
}

fn strike(key: Key, is_press: bool) -> Result<(), SimulateError> {
    let event_type = if is_press {
        EventType::KeyPress(key)
    } else {
        EventType::KeyRelease(key)
    };
    simulate(&event_type)
}

// TODO: add recording macros
