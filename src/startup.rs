use auto_launch::{AutoLaunchBuilder, LinuxLaunchMode, MacOSLaunchMode, WindowsEnableMode};

/// launches the daemon minimized at login
pub fn register_startup() {
    let Some(path) = std::env::current_exe().ok().and_then(|exe| exe.to_str().map(str::to_owned)) else {
        return;
    };
    let launcher = AutoLaunchBuilder::new()
        .set_app_name("buffer_pie_daemon")
        .set_app_path(&path)
        .set_args(&["--minimized"])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
        .set_linux_launch_mode(LinuxLaunchMode::Systemd)
        .build()
        .expect("Failed to create auto-launch instance");
    if matches!(launcher.is_enabled(), Ok(false)) {
        _ = launcher.enable();
    }
}
