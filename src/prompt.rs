use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::ContextConfig;
use crate::detect;

pub fn build_system_prompt(context: &ContextConfig, history: Option<&[String]>) -> String {
    let mut parts = vec![
        "You are a shell command assistant. \
         When the user describes what they want to do, use the run_command tool to suggest the appropriate shell command. \
         The command should be ready to execute as-is. \
         Do not include explanations in the command. \
         If the request is ambiguous, make a reasonable assumption. \
         Never suggest commands that delete files without confirmation flags. \
         Never use sudo unless the user explicitly asks."
            .to_string(),
    ];

    if let Some(os) = &context.os {
        let shell = context.shell.as_deref().unwrap_or("sh");
        parts.push(format!("Environment: {os}, shell: {shell}."));
    }

    if !context.tools.is_empty() {
        let tool_list: Vec<String> = context
            .tools
            .iter()
            .map(|name| {
                if let Some(desc) = detect::tool_description(name) {
                    format!("{name} ({desc})")
                } else {
                    name.clone()
                }
            })
            .collect();
        parts.push(format!(
            "Available tools beyond coreutils: {}. Prefer these over slower alternatives when appropriate.",
            tool_list.join(", ")
        ));
    }

    if let Some(cmds) = history {
        if !cmds.is_empty() {
            let formatted: Vec<String> = cmds.iter().map(|c| format!("$ {c}")).collect();
            parts.push(format!(
                "Recent shell history (context only, NOT the current request):\n{}",
                formatted.join("\n")
            ));
        }
    }

    parts.join("\n")
}

pub fn read_shell_history(limit: usize) -> Vec<String> {
    let histfile = std::env::var("HISTFILE").ok().unwrap_or_else(|| {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.ends_with("zsh") {
            format!("{home}/.zsh_history")
        } else {
            format!("{home}/.bash_history")
        }
    });

    let bytes = match std::fs::read(&PathBuf::from(&histfile)) {
        Ok(b) => b,
        Err(_) => return vec![],
    };
    let content = String::from_utf8_lossy(&bytes);

    let mut seen = HashSet::new();
    let commands: Vec<String> = content
        .lines()
        .rev()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let cmd = if line.starts_with(": ") {
                line.splitn(2, ';').nth(1)?
            } else {
                line
            };
            if cmd.starts_with("tty ") || !seen.insert(cmd.to_string()) {
                return None;
            }
            Some(cmd.to_string())
        })
        .take(limit)
        .collect();

    commands.into_iter().rev().collect()
}
