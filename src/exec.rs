use std::process::{Command, Stdio};

use anyhow::{Context, Result};

pub struct CommandOutcome {
    pub code: i32,
    pub stderr: String,
}

pub fn run_command(command: &str) -> Result<i32> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    let status = Command::new(&shell)
        .arg("-c")
        .arg(command)
        .status()
        .context("failed to execute command")?;

    Ok(status.code().unwrap_or(1))
}

pub fn run_command_captured(command: &str) -> Result<CommandOutcome> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    let output = Command::new(&shell)
        .arg("-c")
        .arg(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to execute command")?
        .wait_with_output()
        .context("failed to wait for command")?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    Ok(CommandOutcome {
        code: output.status.code().unwrap_or(1),
        stderr,
    })
}
