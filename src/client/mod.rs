pub mod groq;
pub mod ollama;

use anyhow::Result;

pub struct CommandResult {
    pub command: String,
    pub explanation: Option<String>,
}

pub trait LlmClient {
    fn complete(&self, system: &str, user: &str, think: bool) -> Result<CommandResult>;
}

pub fn parse_tool_call(body: &serde_json::Value) -> Result<String> {
    let command = body
        .pointer("/choices/0/message/tool_calls/0/function/arguments")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("no tool call in response"))?;

    let args: serde_json::Value = serde_json::from_str(command)?;
    args["command"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("no 'command' field in tool call arguments"))
}

#[allow(dead_code)]
pub fn strip_think_tags(s: &str) -> &str {
    let s = s.trim();
    if s.find("<think>").is_some() {
        if let Some(end) = s.find("</think>") {
            let after = &s[end + "</think>".len()..];
            return after.trim();
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_think_empty() {
        assert_eq!(strip_think_tags("<think>\n\n</think>\n\nls -la"), "ls -la");
    }

    #[test]
    fn test_strip_think_with_content() {
        assert_eq!(
            strip_think_tags("<think>some reasoning</think>\nfind / -name '*.mp4'"),
            "find / -name '*.mp4'"
        );
    }

    #[test]
    fn test_strip_think_no_tags() {
        assert_eq!(strip_think_tags("docker ps"), "docker ps");
    }

    #[test]
    fn test_parse_tool_call_valid() {
        let body = serde_json::json!({
            "choices": [{
                "message": {
                    "tool_calls": [{
                        "function": {
                            "name": "run_command",
                            "arguments": "{\"command\": \"ls -la\"}"
                        }
                    }]
                }
            }]
        });
        assert_eq!(parse_tool_call(&body).unwrap(), "ls -la");
    }

    #[test]
    fn test_parse_tool_call_missing() {
        let body = serde_json::json!({"choices": [{"message": {}}]});
        assert!(parse_tool_call(&body).is_err());
    }
}
