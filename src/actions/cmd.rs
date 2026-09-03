use std::process::{Command, Stdio};

/// runs a command line, optionally inside a visible terminal window
pub fn run_command(command: &str, show_terminal: bool) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;

        let mut cmd = Command::new("powershell");
        if show_terminal {
            // a fresh console that runs the command and shows its output
            cmd.arg("-NoExit").arg("-Command").arg(command);
            cmd.creation_flags(CREATE_NEW_CONSOLE);
        } else {
            cmd.arg("-WindowStyle").arg("Hidden").arg("-Command").arg(command);
            cmd.creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null());
        };

        // TODO: log errors elsewhere
        _ = cmd.spawn();
    }

    #[cfg(not(windows))]
    {
        let mut parts = command.split_whitespace();
        let Some(program) = parts.next() else {
            return;
        };
        _ = Command::new(program).args(parts).spawn();
    }
}
