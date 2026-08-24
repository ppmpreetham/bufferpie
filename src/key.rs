use rdev::{EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};

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

pub fn spawn_input_monitor(
    state: Arc<MenuState>,
    on_menu_trigger: impl Fn(i32, i32) + Send + Sync + 'static,
) {
    std::thread::spawn(move || {
        let trigger = Arc::new(on_menu_trigger);

        let _ = listen(move |event| match event.event_type {
            EventType::KeyPress(Key::CapsLock) => handle_press(&state),
            EventType::KeyRelease(Key::CapsLock) => handle_release(&state),
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

fn handle_release(state: &MenuState) {
    state.is_key_held.store(false, Relaxed);
}

fn handle_move<F>(state: &MenuState, x: i32, y: i32, trigger: Arc<F>)
where
    F: Fn(i32, i32) + Send + Sync + 'static,
{
    if !state.is_key_held.load(Relaxed) || state.is_menu_active.load(Relaxed) {
        return;
    }

    let anchor_x = state.position.x.load(Relaxed);
    let anchor_y = state.position.y.load(Relaxed);

    if anchor_x == 0 && anchor_y == 0 {
        state.position.x.store(x, Relaxed);
        state.position.y.store(y, Relaxed);
        return;
    }

    if has_crossed_threshold(state, x, y) {
        state.is_menu_active.store(true, Relaxed);
        trigger(anchor_x, anchor_y);
    }
}

fn has_crossed_threshold(state: &MenuState, cx: i32, cy: i32) -> bool {
    let dx = cx - state.position.x.load(Relaxed);
    let dy = cy - state.position.y.load(Relaxed);
    (dx * dx) + (dy * dy) > MOTION_THRESHOLD
}
