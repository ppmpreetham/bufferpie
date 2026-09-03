use crate::key::{self, MenuAction};
use crate::ui::create_pie_menu_window;

use futures::StreamExt;
use futures::channel::mpsc::unbounded;
use gpui::*;
use gpui_component::{Theme, ThemeMode};
use std::sync::Arc;

pub fn run(cx: &mut App) {
    gpui_component::init(cx);
    Theme::change(ThemeMode::Dark, None, cx);

    let state = Arc::new(key::MenuState::default());
    let (tx, mut rx) = unbounded();

    key::spawn_input_monitor(state.clone(), tx);
    let window = create_pie_menu_window(cx, state).expect("failed to start");

    cx.spawn(async move |cx| {
        while let Some(action) = rx.next().await {
            _ = window.update(cx, |view, window, cx| match action {
                MenuAction::Open { x, y } => {
                    view.open_at(x, y, cx);
                    window.activate_window();
                }
                // releasing caps lock fires the hovered item before closing
                MenuAction::Close => view.finish(cx),
                MenuAction::Cancel => view.close(cx),
                MenuAction::ShowSettingsButton => view.show_settings_button(cx),
                MenuAction::HideSettingsButton => view.hide_settings_button(cx),
                // recorded keys changed, repaint the macro form
                MenuAction::KeysChanged => {
                    crate::ui::settings::window::refresh_settings(cx);
                }
            });

            // the overlay size is derived from real view state every message,
            // so a missed event can never leave an invisible fullscreen layer
            _ = window.update(cx, |view, window, cx| {
                let active = view.visible || view.settings_visible;
                let target = if active {
                    Bounds::maximized(None, cx).size
                } else {
                    size(px(0.0), px(0.0))
                };
                window.resize(target);
            });
        }
    })
    .detach();
}
