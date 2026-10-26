use futures::channel::mpsc::UnboundedSender;
use rdev::{EventType, Key, listen};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Mutex, MutexGuard};

pub enum MenuAction {
    Open { x: f32, y: f32 },
    Close,
    /// closes without running the selected action
    Cancel,
    ShowSettingsButton,
    HideSettingsButton,
    /// recorded keys changed, the settings window should repaint
    KeysChanged,
    OpenSettings,
}

const MOTION_THRESHOLD: i32 = 64;

static HELD: AtomicBool = AtomicBool::new(false);
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// where the cursor was when caps lock went down, 0/0 until the first move
static ORIGIN: [AtomicI32; 2] = [AtomicI32::new(0), AtomicI32::new(0)];

/// stops tracking the open menu so a later release won't fire its action
pub fn deactivate() {
    ACTIVE.store(false, Relaxed);
    ORIGIN.iter().for_each(|axis| axis.store(0, Relaxed));
}

/// forgets the open menu, returns whether there was one
fn dismiss() -> bool {
    let active = ACTIVE.swap(false, Relaxed);
    if active {
        deactivate();
    }
    active
}

/// live macro recorder shared between the input thread and the settings ui
struct Recorder {
    recording: bool,
    pressed: Vec<Key>,
    keys: Vec<Key>,
}

const IDLE: Recorder = Recorder { recording: false, pressed: Vec::new(), keys: Vec::new() };
static RECORDER: Mutex<Recorder> = Mutex::new(IDLE);

fn recorder() -> MutexGuard<'static, Recorder> {
    RECORDER.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn start_recording() {
    *recorder() = Recorder { recording: true, ..IDLE };
}

/// stops recording and hands over the captured keys
pub fn stop_recording() -> Vec<Key> {
    let mut recorder = recorder();
    recorder.recording = false;
    recorder.pressed.clear();
    std::mem::take(&mut recorder.keys)
}

/// snapshot of the captured keys for display
pub fn recorded() -> Vec<Key> {
    recorder().keys.clone()
}

pub fn spawn_input_monitor(tx: UnboundedSender<MenuAction>) {
    std::thread::spawn(move || {
        let send = move |action: MenuAction| _ = tx.unbounded_send(action);
        _ = listen(move |event| match event.event_type {
            EventType::KeyPress(Key::CapsLock) => {
                HELD.store(true, Relaxed);
                send(MenuAction::ShowSettingsButton);
            }
            EventType::KeyRelease(Key::CapsLock) => {
                HELD.store(false, Relaxed);
                send(MenuAction::HideSettingsButton);
                if dismiss() {
                    send(MenuAction::Close);
                }
            }
            // esc dismisses the menu without firing its hovered action, and a
            // still-held caps must not reopen it on the next mouse move
            EventType::KeyPress(Key::Escape) => {
                HELD.store(false, Relaxed);
                if dismiss() {
                    send(MenuAction::Cancel);
                }
            }
            // stores global presses while recording, ignoring auto-repeat
            EventType::KeyPress(key) => {
                let mut recorder = recorder();
                if recorder.pressed.contains(&key) {
                    return;
                }
                recorder.pressed.push(key);
                if recorder.recording {
                    recorder.keys.push(key);
                    drop(recorder);
                    send(MenuAction::KeysChanged);
                }
            }
            EventType::KeyRelease(key) => recorder().pressed.retain(|pressed| *pressed != key),
            EventType::MouseMove { x, y } if HELD.load(Relaxed) && !ACTIVE.load(Relaxed) => {
                let (x, y) = (x as i32, y as i32);
                if ORIGIN.iter().all(|axis| axis.load(Relaxed) == 0) {
                    ORIGIN[0].store(x, Relaxed);
                    ORIGIN[1].store(y, Relaxed);
                }
                let [ox, oy] = [0, 1].map(|axis| ORIGIN[axis].load(Relaxed));
                if (x - ox).pow(2) + (y - oy).pow(2) > MOTION_THRESHOLD {
                    ACTIVE.store(true, Relaxed);
                    send(MenuAction::Open { x: ox as f32, y: oy as f32 });
                }
            }
            _ => {}
        });
    });
}
