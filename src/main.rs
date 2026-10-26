#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod actions;
mod app;
mod assets;
mod config;
mod key;
mod pie;
mod platform;
mod settings;
#[cfg(not(debug_assertions))]
mod startup;

fn main() {
    #[cfg(not(debug_assertions))]
    startup::register_startup();

    gpui_kit::application()
        .with_assets(assets::Assets)
        .run(app::run);
}
