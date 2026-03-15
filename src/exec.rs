use std::process::Command;

use anyhow::{Context, Result};

pub fn run_command(command: &str) -> Result<i32> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());

    let status = Command::new(&shell)
        .arg("-c")
        .arg(command)
        .status()
        .context("failed to execute command")?;

    Ok(status.code().unwrap_or(1))
}
