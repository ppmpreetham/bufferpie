use crate::key::{self, MenuAction};
use crate::ui::{open_pie_menu, pie_menu::PieMenuView};

use futures::StreamExt;
use futures::channel::mpsc::unbounded;
use gpui::*;
use std::sync::Arc;

pub fn run(cx: &mut App) {
    // dummy keepalive node
    let _keepalive = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::from_corners(
                Point::default(),
                Point::default(),
            ))),
            show: false,
            ..Default::default()
        },
        |_, cx| cx.new(|_| gpui::Empty),
    );

    let state = Arc::new(key::MenuState::default());
    let (tx, mut rx) = unbounded();

    key::spawn_input_monitor(state, tx);

    cx.spawn(async move |cx| {
        let mut active_window: Option<WindowHandle<PieMenuView>> = None;

        while let Some(action) = rx.next().await {
            let _ = cx.update(|app| match action {
                MenuAction::Open { x, y } => {
                    if active_window.is_none()
                        && let Ok(window) = open_pie_menu(app, x, y)
                    {
                        active_window = Some(window);
                    }
                }
                MenuAction::Close => {
                    if let Some(window) = active_window.take() {
                        let _ = window.update(app, |_, window, _| {
                            window.remove_window();
                        });
                    }
                }
            });
        }
    })
    .detach();
}
