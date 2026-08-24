#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod actions;
mod app;
mod key;
mod startup;
mod ui;

use gpui::Application;

fn main() {
    #[cfg(not(debug_assertions))]
    startup::register_startup();

    Application::new().run(app::run);
}
