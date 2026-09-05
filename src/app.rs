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

    let tray_menu = tray_icon::menu::Menu::new();
    let settings_item = tray_icon::menu::MenuItem::with_id("settings", "Settings", true, None);
    let quit_item = tray_icon::menu::MenuItem::with_id("quit", "Quit", true, None);
    _ = tray_menu.append(&settings_item);
    _ = tray_menu.append(&quit_item);

    let icon_bytes = include_bytes!("../readme/Logo.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load tray icon image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = tray_icon::Icon::from_rgba(rgba, width, height).expect("Failed to create tray icon");

    #[cfg(target_os = "linux")]
    gtk::init().expect("Failed to initialize GTK");

    let tray_icon = tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Buffer Pie")
        .with_icon(icon)
        .build()
        .unwrap();

    std::mem::forget(tray_icon);

    let tx_tray = tx.clone();
    std::thread::spawn(move || {
        let receiver = tray_icon::menu::MenuEvent::receiver();
        while let Ok(event) = receiver.recv() {
            if event.id.0 == "settings" {
                _ = tx_tray.unbounded_send(MenuAction::OpenSettings);
            }
            if event.id.0 == "quit" {
                std::process::exit(0);
            }
        }
    });

    key::spawn_input_monitor(state.clone(), tx);
    let window = create_pie_menu_window(cx, state).expect("failed to start");
    _ = window.update(cx, |_, window, _| crate::platform::make_no_activate(window));

    cx.spawn(async move |cx| {
        while let Some(action) = rx.next().await {
            _ = window.update(cx, |view, window, cx| match action {
                MenuAction::Open { x, y } => {
                    view.open_at(x, y, cx);
                    crate::platform::raise(window);
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
                MenuAction::OpenSettings => {
                    let config = view.config.clone();
                    crate::ui::settings::window::open_settings_window(config, cx);
                }
            });

            _ = window.update(cx, |view, window, cx| {
                let active = view.visible || view.settings_visible;
                let target = if active {
                    Bounds::maximized(None, cx).size
                } else {
                    size(px(1.0), px(1.0))
                };
                window.resize(target);
            });
        }
    })
    .detach();

}
