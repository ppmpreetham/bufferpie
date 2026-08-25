// bruh simple helper here
#[allow(dead_code)]
pub fn open_app(app_name: &str) {
    let _ = open::that(app_name);
}
