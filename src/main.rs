mod app;
mod key;
mod startup;
mod ui;

use gpui::Application;

fn main() {
    startup::register_startup();
    Application::new().run(app::run);
}
