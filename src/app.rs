use crate::{
    key::{self, MenuAction},
    ui,
};
use gpui::*;
use std::{
    sync::{
        Arc,
        mpsc::{Receiver, channel},
    },
    time::Duration,
};

const POLL_INTERVAL: Duration = Duration::from_millis(5);

pub fn run(cx: &mut App) {
    let state = Arc::new(key::MenuState::default());
    let (tx, rx) = channel();

    key::spawn_input_monitor(state, tx);

    cx.spawn(async move |cx| {
        let mut active_window: Option<WindowHandle<ui::PieMenuView>> = None;

        loop {
            process_actions(cx, &rx, &mut active_window);
            cx.background_executor().timer(POLL_INTERVAL).await;
        }
    })
    .detach();
}

fn process_actions(
    cx: &mut AsyncApp,
    rx: &Receiver<MenuAction>,
    active_window: &mut Option<WindowHandle<ui::PieMenuView>>,
) {
    while let Ok(action) = rx.try_recv() {
        let _ = cx.update(|app| match action {
            MenuAction::Open { x, y } => {
                if active_window.is_none()
                    && let Ok(window) = ui::open_pie_menu(app, x, y)
                {
                    *active_window = Some(window);
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
}
