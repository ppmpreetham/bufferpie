use std::process::{Command, Stdio};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[allow(dead_code)]
pub fn run_command(command: &str) {
    let mut parts = command.trim().split_whitespace();
    let Some(program) = parts.next() else {
        return;
    };
    let mut cmd = Command::new(program);

    cmd.args(parts)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    // TODO: log errors elsewhere
    let _ = cmd.spawn();
}
