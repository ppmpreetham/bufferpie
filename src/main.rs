#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod actions;
mod app;
mod key;
mod platform;
mod startup;
mod ui;

fn main() {
    #[cfg(not(debug_assertions))]
    startup::register_startup();

    gpui_platform::application()
        .with_assets(ui::assets::Assets)
        .run(app::run);
}
