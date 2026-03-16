use anyhow::{bail, Result};
use serde_json::json;
use std::process::Command;

use crate::client::CommandResult;
use crate::config::ClientConfig;

const MAX_TURNS: usize = 5;

const READONLY_ALLOWLIST: &[&str] = &[
    // filesystem inspection
    "ls", "find", "fd", "es", "cat", "head", "tail", "wc", "file", "stat",
    "readlink", "realpath", "basename", "dirname", "tree", "bat", "eza",
    // search
    "rg", "grep",
    // system info
    "which", "where", "type", "pwd", "date", "whoami", "hostname", "uname",
    "env", "printenv", "df", "du", "free", "uptime", "ps", "lsof", "lsblk",
    // network info (read-only)
    "ip", "ifconfig", "dig", "nslookup", "ss",
    // data processing (stateless, no side effects)
    "sort", "uniq", "tr", "cut", "jq", "yq",
    // vcs read-only (git status/log/etc are safe)
    "git",
];

fn is_readonly_command(cmd: &str) -> bool {
    let first_token = cmd.split_whitespace().next().unwrap_or("");
    let binary = first_token.rsplit('/').next().unwrap_or(first_token);
    READONLY_ALLOWLIST.contains(&binary)
}

fn execute_readonly(cmd: &str) -> Result<String> {
    if !is_readonly_command(cmd) {
        bail!("blocked: '{cmd}' is not in the readonly allowlist");
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let output = Command::new(&shell)
        .arg("-c")
        .arg(cmd)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Truncate to avoid blowing up context
    let mut result = String::new();
    if !stdout.is_empty() {
        let truncated: String = stdout.chars().take(2000).collect();
        result.push_str(&truncated);
        if stdout.len() > 2000 {
            result.push_str("\n... (truncated)");
        }
    }
    if !stderr.is_empty() && !output.status.success() {
        result.push_str(&format!("\nstderr: {}", stderr.chars().take(500).collect::<String>()));
    }

    Ok(result)
}

fn research_tools() -> serde_json::Value {
    json!([
        {
            "type": "function",
            "function": {
                "name": "read_system",
                "description": "Run a read-only command to gather information. Only informational commands are allowed (ls, find, cat, grep, etc). No writes, no deletes, no sudo.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The read-only shell command to execute"
                        }
                    },
                    "required": ["command"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "run_command",
                "description": "Suggest the final shell command for the user to execute. Call this once you have gathered enough information.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }
            }
        }
    ])
}

fn parse_command_arg(arguments: &str) -> Result<String> {
    let args: serde_json::Value = serde_json::from_str(arguments)?;
    args["command"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("no 'command' field in tool call arguments"))
}

pub fn research(config: &ClientConfig, system: &str, query: &str) -> Result<CommandResult> {
    let api_key = config
        .api_key
        .clone()
        .ok_or_else(|| anyhow::anyhow!("API key not set"))?;

    let mut messages = vec![
        json!({ "role": "system", "content": format!("/no_think\n{system}\nIMPORTANT: You MUST use read_system first to gather real information from the user's system before suggesting a command. Do NOT guess paths, project names, or locations. Use read_system to discover them, then use run_command with the concrete result. Be efficient -- run the lookups you need, then suggest the final command with real paths/values.") }),
        json!({ "role": "user", "content": query }),
    ];

    for _turn in 0..MAX_TURNS {
        let turn_start = std::time::Instant::now();
        let body = json!({
            "model": config.model,
            "messages": messages,
            "tools": research_tools(),
            "temperature": config.temperature,
            "max_tokens": 4096,
        });

        let resp = ureq::post(&config.endpoint)
            .header("Authorization", &format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send_json(&body);

        let response: serde_json::Value = match resp {
            Ok(mut r) => r.body_mut().read_json()?,
            Err(ureq::Error::StatusCode(429)) => {
                bail!("rate limited by API -- wait a moment and try again")
            }
            Err(ureq::Error::StatusCode(401)) => {
                bail!("invalid API key")
            }
            Err(e) => return Err(e.into()),
        };
        let api_ms = turn_start.elapsed().as_millis();

        let choice = &response["choices"][0];
        let message = &choice["message"];

        // Add assistant message to conversation
        messages.push(message.clone());

        let tool_calls = match message["tool_calls"].as_array() {
            Some(calls) => calls.clone(),
            None => {
                // Model responded with text instead of a tool call -- nudge it
                messages.push(json!({
                    "role": "user",
                    "content": "You must use the run_command tool to suggest a final command. Do not respond with text."
                }));
                continue;
            }
        };

        for tool_call in &tool_calls {
            let name = tool_call["function"]["name"].as_str().unwrap_or("");
            let arguments = tool_call["function"]["arguments"].as_str().unwrap_or("{}");
            let tool_call_id = tool_call["id"].as_str().unwrap_or("");

            match name {
                "run_command" => {
                    let command = parse_command_arg(arguments)?;
                    return Ok(CommandResult {
                        command,
                        explanation: None,
                    });
                }
                "read_system" => {
                    let cmd = parse_command_arg(arguments)?;
                    let exec_start = std::time::Instant::now();
                    let result = match execute_readonly(&cmd) {
                        Ok(output) => output,
                        Err(e) => format!("error: {e}"),
                    };
                    let exec_ms = exec_start.elapsed().as_millis();
                    eprintln!("\x1b[2m  [research] {cmd}  (api: {api_ms}ms, exec: {exec_ms}ms)\x1b[0m");

                    messages.push(json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": result,
                    }));
                }
                other => {
                    messages.push(json!({
                        "role": "tool",
                        "tool_call_id": tool_call_id,
                        "content": format!("unknown tool: {other}"),
                    }));
                }
            }
        }
    }

    bail!("research exceeded {MAX_TURNS} turns without producing a command")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_readonly_allows_safe_commands() {
        assert!(is_readonly_command("ls -la"));
        assert!(is_readonly_command("find . -name '*.rs'"));
        assert!(is_readonly_command("cat /etc/hostname"));
        assert!(is_readonly_command("rg pattern"));
        assert!(is_readonly_command("git status"));
        assert!(is_readonly_command("es something"));
    }

    #[test]
    fn test_readonly_blocks_unsafe_commands() {
        assert!(!is_readonly_command("rm -rf /"));
        assert!(!is_readonly_command("sudo ls"));
        assert!(!is_readonly_command("mv a b"));
        assert!(!is_readonly_command("cp a b"));
        assert!(!is_readonly_command("chmod 777 file"));
        assert!(!is_readonly_command("chown user file"));
        assert!(!is_readonly_command("dd if=/dev/zero"));
        assert!(!is_readonly_command("python3 script.py"));
        assert!(!is_readonly_command("node -e 'process.exit()'"));
        assert!(!is_readonly_command("docker run ubuntu"));
        assert!(!is_readonly_command("cargo build"));
    }

    #[test]
    fn test_readonly_handles_full_paths() {
        assert!(is_readonly_command("/usr/bin/ls -la"));
        assert!(!is_readonly_command("/usr/bin/rm file"));
    }
}
