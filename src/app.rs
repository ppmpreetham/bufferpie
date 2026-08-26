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
    let mut caps_held = false;
    let mut menu_open = false;

    cx.spawn(async move |cx| {
        while let Some(action) = rx.next().await {
            let _ = window.update(cx, |view, window, cx| match action {
                MenuAction::Open { x, y } => {
                    menu_open = true;
                    let bounds = Bounds::maximized(None, cx);
                    window.resize(bounds.size);
                    view.open_at(x, y, cx);
                    window.activate_window();
                }
                MenuAction::Close => {
                    menu_open = false;
                    view.finish(cx);
                    if !caps_held {
                        window.resize(size(px(0.0), px(0.0)));
                    }
                }
                MenuAction::Cancel => {
                    menu_open = false;
                    view.close(cx);
                    if !caps_held {
                        window.resize(size(px(0.0), px(0.0)));
                    }
                }
                MenuAction::ShowSettingsButton => {
                    caps_held = true;
                    let bounds = Bounds::maximized(None, cx);
                    window.resize(bounds.size);
                    view.show_settings_button(cx);
                }
                MenuAction::HideSettingsButton => {
                    caps_held = false;
                    view.hide_settings_button(cx);
                    if !menu_open {
                        window.resize(size(px(0.0), px(0.0)));
                    }
                }
                // recorded keys changed, repaint the macro form
                MenuAction::KeysChanged => {
                    crate::ui::settings::window::refresh_settings(cx);
                }
            });
        }
    })
    .detach();
}
