use rdev::{EventType, Key, listen};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
    mpsc::Sender,
};

pub enum MenuAction {
    Open { x: i32, y: i32 },
    Close,
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

pub fn spawn_input_monitor(state: Arc<MenuState>, trigger: Sender<MenuAction>) {
    std::thread::spawn(move || {
        let trigger = Arc::new(trigger);

        let _ = listen(move |event| match event.event_type {
            EventType::KeyPress(Key::CapsLock) => handle_press(&state),
            EventType::KeyRelease(Key::CapsLock) => handle_release(&state, trigger.clone()),
            EventType::MouseMove { x, y } => {
                handle_move(&state, x as i32, y as i32, trigger.clone())
            }
            _ => {}
        });
    });
}

fn handle_press(state: &MenuState) {
    state.is_key_held.store(true, Relaxed);
}

fn handle_release(state: &MenuState, trigger: Arc<Sender<MenuAction>>) {
    state.is_key_held.store(false, Relaxed);

    if state.is_menu_active.swap(false, Relaxed) {
        state.position.x.store(0, Relaxed);
        state.position.y.store(0, Relaxed);

        let _ = trigger.send(MenuAction::Close);
    }
}

fn handle_move(state: &MenuState, x: i32, y: i32, trigger: Arc<Sender<MenuAction>>) {
    if !state.is_key_held.load(Relaxed) || state.is_menu_active.load(Relaxed) {
        return;
    }

    let cx = state.position.x.load(Relaxed);
    let cy = state.position.y.load(Relaxed);

    if cx == 0 && cy == 0 {
        state.position.x.store(x, Relaxed);
        state.position.y.store(y, Relaxed);
        return;
    }

    if has_crossed_threshold(state, x, y) {
        state.is_menu_active.store(true, Relaxed);
        let _ = trigger.send(MenuAction::Open { x: cx, y: cy });
    }
}

fn has_crossed_threshold(state: &MenuState, x: i32, y: i32) -> bool {
    let dx = x - state.position.x.load(Relaxed);
    let dy = y - state.position.y.load(Relaxed);
    (dx * dx) + (dy * dy) > MOTION_THRESHOLD
}
