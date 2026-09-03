use rdev::{EventType, Key, SimulateError, simulate};
use std::thread::sleep;
use std::time::Duration;

/// readable label for a captured key
use std::borrow::Cow;

pub fn key_label(key: &Key) -> Cow<'static, str> {
    match key {
        Key::ControlLeft | Key::ControlRight => Cow::Borrowed("ctrl"),
        Key::MetaLeft | Key::MetaRight => Cow::Borrowed("meta"),
        Key::ShiftLeft | Key::ShiftRight => Cow::Borrowed("shift"),
        Key::Return | Key::KpReturn => Cow::Borrowed("enter"),
        Key::CapsLock => Cow::Borrowed("caps"),
        Key::Delete | Key::KpDelete => Cow::Borrowed("del"),
        Key::PrintScreen => Cow::Borrowed("printscr"),
        Key::Unknown(code) => Cow::Owned(format!("unknown_{}", code)),

        other => {
            let raw_name = format!("{:?}", other);
            let cleaned = if let Some(stripped) = raw_name.strip_prefix("Key") {
                stripped.to_lowercase()
            } else if let Some(stripped) = raw_name.strip_prefix("Num") {
                stripped.to_lowercase()
            } else {
                raw_name.to_lowercase()
            };
            Cow::Owned(cleaned)
        }
    }
}

/// strikes a sequence of keys with an optional delay in between
pub fn run_keystrokes(keys: &[Key], delay: u32) {
    let delay_duration = Duration::from_millis(delay as u64);

    // TODO: log the keys later for user
    for key in keys {
        _ = strike(*key, true);
        if delay > 0 {
            sleep(delay_duration);
        }
        _ = strike(*key, false);
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
