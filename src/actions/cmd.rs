use std::process::{Command, Stdio};

/// runs a command line, optionally inside a visible terminal window
pub fn run_command(command: &str, show_terminal: bool) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let mut cmd = if show_terminal {
            // a fresh console that runs the command and shows its output
            let mut cmd = Command::new("cmd");
            cmd.arg("/C").raw_arg(command);
            cmd
        } else {
            let mut parts = command.split_whitespace();
            let Some(program) = parts.next() else {
                return;
            };
            let mut cmd = Command::new(program);
            cmd.args(parts)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .stdin(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW);
            cmd
        };

        // TODO: log errors elsewhere
        let _ = cmd.spawn();
    }

    #[cfg(not(windows))]
    {
        let mut parts = command.split_whitespace();
        let Some(program) = parts.next() else {
            return;
        };
        let _ = Command::new(program).args(parts).spawn();
    }
}
