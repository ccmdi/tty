use anyhow::Result;
use serde_json::json;

use super::{CommandResult, LlmClient, parse_tool_call, strip_think_tags};

pub struct OllamaClient {
    pub endpoint: String,
    pub model: String,
    pub temperature: f64,
    pub max_tokens: u32,
}

fn tool_schema() -> serde_json::Value {
    json!([{
        "type": "function",
        "function": {
            "name": "run_command",
            "description": "Execute a shell command",
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
    }])
}

impl LlmClient for OllamaClient {
    fn complete(&self, system: &str, user: &str, think: bool) -> Result<CommandResult> {
        let user_content = if think {
            user.to_string()
        } else {
            format!("/no_think\n{user}")
        };

        let body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user_content }
            ],
            "tools": tool_schema(),
            "stream": false,
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
            }
        });

        let response: serde_json::Value = ureq::post(&self.endpoint)
            .header("Content-Type", "application/json")
            .send_json(&body)?
            .body_mut()
            .read_json()?;

        // Ollama uses the same OpenAI-compatible format for tool calls
        let command = match parse_tool_call(&response) {
            Ok(cmd) => cmd,
            Err(_) => {
                // Fallback: parse from message content if tool calling isn't supported
                let content = response
                    .pointer("/choices/0/message/content")
                    .or_else(|| response.pointer("/message/content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                strip_think_tags(content).to_string()
            }
        };

        let explanation = if think {
            response
                .pointer("/choices/0/message/reasoning")
                .or_else(|| response.pointer("/message/reasoning"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        } else {
            None
        };

        Ok(CommandResult {
            command,
            explanation,
        })
    }
}
