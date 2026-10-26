use crate::key::{self, MenuAction};
use futures::{StreamExt, channel::mpsc::{UnboundedSender, unbounded}};
use gpui_kit::App;
use gpui_kit::component::{Theme, ThemeMode};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};

pub fn run(cx: &mut App) {
    gpui_kit::init(cx);
    Theme::change(ThemeMode::Dark, None, cx);

    let (tx, mut rx) = unbounded();
    spawn_tray(tx.clone());
    key::spawn_input_monitor(tx);
    let window = crate::pie::open(cx).expect("failed to start");

    cx.spawn(async move |cx| {
        while let Some(action) = rx.next().await {
            _ = window.update(cx, |overlay, window, cx| overlay.handle(action, window, cx));
        }
    })
    .detach();
}

fn spawn_tray(tx: UnboundedSender<MenuAction>) {
    let menu = Menu::new();
    _ = menu.append_items(&[
        &MenuItem::with_id("settings", "Settings", true, None),
        &MenuItem::with_id("quit", "Quit", true, None),
    ]);
    let image = image::load_from_memory(include_bytes!("../readme/Logo.png"))
        .expect("Failed to load tray icon image")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let icon = tray_icon::Icon::from_rgba(image.into_raw(), width, height).expect("Failed to create tray icon");

    #[cfg(target_os = "linux")]
    gtk::init().expect("Failed to initialize GTK");

    let tray = tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Buffer Pie")
        .with_icon(icon)
        .build()
        .unwrap();
    std::mem::forget(tray);

    std::thread::spawn(move || {
        while let Ok(event) = MenuEvent::receiver().recv() {
            match event.id.0.as_str() {
                "settings" => _ = tx.unbounded_send(MenuAction::OpenSettings),
                "quit" => std::process::exit(0),
                _ => {}
            }
        }
    });
}
