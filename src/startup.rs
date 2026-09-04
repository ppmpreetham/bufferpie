use auto_launch::{AutoLaunchBuilder, LinuxLaunchMode, MacOSLaunchMode, WindowsEnableMode};

const APP_ID: &str = "buffer_pie_daemon";
const STARTUP_ARGS: &[&str] = &["--minimized"];

#[allow(dead_code)]
pub fn register_startup() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(path) = exe.to_str() else {
        return;
    };

    let launcher = AutoLaunchBuilder::new()
        .set_app_name(APP_ID)
        .set_app_path(path)
        .set_args(STARTUP_ARGS)
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
        .set_linux_launch_mode(LinuxLaunchMode::Systemd)
        .build()
        .expect("Failed to create auto-launch instance");

    if matches!(launcher.is_enabled(), Ok(false)) {
        _ = launcher.enable();
    }
}
