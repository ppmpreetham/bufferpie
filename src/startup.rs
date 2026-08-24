use auto_launch::{AutoLaunch, WindowsEnableMode};

const APP_ID: &str = "cyber_pie_daemon";
const STARTUP_ARGS: &[&str] = &["--minimized"];

pub fn register_startup() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(path) = exe.to_str() else {
        return;
    };

    let launcher = AutoLaunch::new(APP_ID, path, WindowsEnableMode::CurrentUser, STARTUP_ARGS);
    if matches!(launcher.is_enabled(), Ok(false)) {
        let _ = launcher.enable();
    }
}
