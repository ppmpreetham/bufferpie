use futures::channel::mpsc::UnboundedSender;
use rdev::{EventType, Key, listen};
use std::sync::{
    Arc,
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

pub fn spawn_input_monitor(state: Arc<MenuState>, trigger: UnboundedSender<MenuAction>) {
    std::thread::spawn(move || {
        let _ = listen(move |event| match event.event_type {
            EventType::KeyPress(Key::CapsLock) => handle_press(&state, &trigger),
            EventType::KeyRelease(Key::CapsLock) => handle_release(&state, &trigger),
            EventType::KeyPress(Key::Escape) => handle_escape(&state, &trigger),

            EventType::MouseMove { x, y } => handle_move(&state, x, y, &trigger),
            _ => {}
        });
    });
}

fn handle_press(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    state.is_key_held.store(true, Relaxed);
    let _ = trigger.unbounded_send(MenuAction::ShowSettingsButton);
}

fn handle_release(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    state.is_key_held.store(false, Relaxed);
    let _ = trigger.unbounded_send(MenuAction::HideSettingsButton);

    if state.is_menu_active.swap(false, Relaxed) {
        state.deactivate();

        let _ = trigger.unbounded_send(MenuAction::Close);
    }
}

/// esc dismisses the open menu without firing its hovered action
fn handle_escape(state: &MenuState, trigger: &UnboundedSender<MenuAction>) {
    // a still-held caps must not reopen the menu on the next mouse move
    state.is_key_held.store(false, Relaxed);

    if state.is_menu_active.swap(false, Relaxed) {
        state.deactivate();

        let _ = trigger.unbounded_send(MenuAction::Cancel);
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
        let _ = trigger.unbounded_send(MenuAction::Open {
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
