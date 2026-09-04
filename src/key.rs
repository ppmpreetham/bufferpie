use futures::channel::mpsc::UnboundedSender;
use parking_lot::Mutex;
use rdev::{EventType, Key, listen};
use std::collections::HashSet;
use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

pub enum MenuAction {
    Open {
        x: f32,
        y: f32,
    },
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

#[derive(Default)]
struct Position {
    x: AtomicI32,
    y: AtomicI32,
}

#[derive(Default)]
pub struct MenuState {
    is_key_held: AtomicBool,
    is_menu_active: AtomicBool,
    position: Position,
}

impl MenuState {
    /// stops tracking the open menu so a later release won't fire its action
    pub fn deactivate(&self) {
        self.is_menu_active.store(false, Relaxed);
        self.position.x.store(0, Relaxed);
        self.position.y.store(0, Relaxed);
    }
}

/// live macro recorder shared between the input thread and the settings ui
struct RecorderState {
    recording: bool,
    pressed: HashSet<Key>,
    keys: Vec<Key>,
}

fn recorder() -> &'static Mutex<RecorderState> {
    static RECORDER: OnceLock<Mutex<RecorderState>> = OnceLock::new();
    RECORDER.get_or_init(|| {
        Mutex::new(RecorderState {
            recording: false,
            pressed: HashSet::new(),
            keys: Vec::new(),
        })
    })
}

pub fn start_recording() {
    let mut state = recorder().lock();
    *state = RecorderState {
        recording: true,
        pressed: HashSet::new(),
        keys: Vec::new(),
    };
}

/// stops recording and hands over the captured keys
pub fn stop_recording() -> Vec<Key> {
    let mut state = recorder().lock();
    state.recording = false;
    state.pressed.clear();
    std::mem::take(&mut state.keys)
}

/// snapshot of the captured keys for display
pub fn recorded() -> Vec<Key> {
    recorder().lock().keys.clone()
}

/// accepts a global key press, storing it while recording (ignores auto-repeat)
fn record_press(key: Key, trigger: &UnboundedSender<MenuAction>) {
    let mut state = recorder().lock();
    if !state.pressed.insert(key) || !state.recording {
        return;
    }
    state.keys.push(key);
    drop(state);
    _ = trigger.unbounded_send(MenuAction::KeysChanged);
}

fn record_release(key: Key) {
    recorder().lock().pressed.remove(&key);
}

pub fn spawn_input_monitor(state: Arc<MenuState>, trigger: UnboundedSender<MenuAction>) {
    std::thread::spawn(move || {
        _ = listen(move |event| match event.event_type {
            EventType::KeyPress(Key::CapsLock) => handle_press(&state, &trigger),
            EventType::KeyRelease(Key::CapsLock) => handle_release(&state, &trigger),
            EventType::KeyPress(Key::Escape) => handle_escape(&state, &trigger),

            EventType::KeyPress(key) => record_press(key, &trigger),
            EventType::KeyRelease(key) => record_release(key),

            EventType::MouseMove { x, y } => handle_move(&state, x, y, &trigger),
            _ => {}
        });
    });
}

fn handle_press(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    state.is_key_held.store(true, Relaxed);
    _ = trigger.unbounded_send(MenuAction::ShowSettingsButton);
}

fn handle_release(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    state.is_key_held.store(false, Relaxed);
    _ = trigger.unbounded_send(MenuAction::HideSettingsButton);

    if state.is_menu_active.swap(false, Relaxed) {
        state.deactivate();

        _ = trigger.unbounded_send(MenuAction::Close);
    }
}

/// esc dismisses the open menu without firing its hovered action
fn handle_escape(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    // a still-held caps must not reopen the menu on the next mouse move
    state.is_key_held.store(false, Relaxed);

    if state.is_menu_active.swap(false, Relaxed) {
        state.deactivate();

        _ = trigger.unbounded_send(MenuAction::Cancel);
    }
}

fn handle_move(state: &MenuState, x: f64, y: f64, trigger: &UnboundedSender<MenuAction>) {
    if !state.is_key_held.load(Relaxed) || state.is_menu_active.load(Relaxed) {
        return;
    }

    let mut cx = state.position.x.load(Relaxed);
    let mut cy = state.position.y.load(Relaxed);

    if cx == 0 && cy == 0 {
        cx = x as i32;
        cy = y as i32;
        state.position.x.store(cx, Relaxed);
        state.position.y.store(cy, Relaxed);
    }

    if has_crossed_threshold(state, x, y) {
        state.is_menu_active.store(true, Relaxed);
        _ = trigger.unbounded_send(MenuAction::Open {
            x: cx as f32,
            y: cy as f32,
        });
    }
}

fn has_crossed_threshold(state: &MenuState, x: f64, y: f64) -> bool {
    let dx = (x as i32) - state.position.x.load(Relaxed);
    let dy = (y as i32) - state.position.y.load(Relaxed);
    (dx * dx) + (dy * dy) > MOTION_THRESHOLD
}
