use crate::key::{self, MenuAction};
use crate::ui::create_pie_menu_window;

use futures::StreamExt;
use futures::channel::mpsc::unbounded;
use gpui::*;
use std::sync::Arc;

pub fn run(cx: &mut App) {
    let state = Arc::new(key::MenuState::default());
    let (tx, mut rx) = unbounded();

    key::spawn_input_monitor(state, tx);
    let window = create_pie_menu_window(cx).expect("failed to start");

    cx.spawn(async move |cx| {
        while let Some(action) = rx.next().await {
            let _ = window.update(cx, |view, window, cx| match action {
                MenuAction::Open { x, y } => {
                    let bounds = Bounds::maximized(None, cx);
                    window.resize(bounds.size);
                    view.open_at(x as f32, y as f32, cx);
                    window.activate_window();
                }
                MenuAction::Close => {
                    view.close(cx);
                    window.resize(size(px(0.0), px(0.0)));
                }
            });
        }
    })
    .detach();
}
