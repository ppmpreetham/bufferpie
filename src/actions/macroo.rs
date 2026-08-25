use rdev::{EventType, Key, SimulateError, simulate};
use std::thread::sleep;
use std::time::Duration;

#[allow(dead_code)]
pub fn run_macro(keys: &[Key], delay: u32) {
    let delay_duration = Duration::from_millis(delay as u64);

    // TODO: log the keys later for user
    for key in keys {
        let _ = send_key_event(*key, true);
        if delay > 0 {
            sleep(delay_duration);
        }
        let _ = send_key_event(*key, false);
    }
}

fn send_key_event(key: Key, is_press: bool) -> Result<(), SimulateError> {
    let event_type = if is_press {
        EventType::KeyPress(key)
    } else {
        EventType::KeyRelease(key)
    };
    simulate(&event_type)
}

// TODO: add recording macros
