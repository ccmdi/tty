use anyhow::Result;
use serde_json::json;

use super::{CommandResult, LlmClient, parse_tool_call};
use crate::config::ClientConfig;

pub struct GroqClient {
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    pub temperature: f64,
}

impl GroqClient {
    pub fn from_config(config: &ClientConfig) -> Result<Self> {
        let api_key = config
            .api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("API key not set. Run `tty init` or set GROQ_API_KEY"))?;
        Ok(Self {
            api_key,
            endpoint: config.endpoint.clone(),
            model: config.model.clone(),
            temperature: config.temperature,
        })
    }
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

impl LlmClient for GroqClient {
    fn complete(&self, system: &str, user: &str, think: bool) -> Result<CommandResult> {
        let system_content = if think {
            format!("{system}\nThink step by step about what the user needs, then use the run_command tool.")
        } else {
            format!("/no_think\n{system}")
        };

        let max_tokens: u32 = if think { 4096 } else { 200 };

        let mut body = json!({
            "model": self.model,
            "messages": [
                { "role": "system", "content": system_content },
                { "role": "user", "content": user }
            ],
            "tools": tool_schema(),
            "temperature": self.temperature,
            "max_tokens": max_tokens,
        });

        // Force tool call in fast mode for consistency; let model decide in think mode
        if !think {
            body["tool_choice"] = json!({ "type": "function", "function": { "name": "run_command" } });
        }

        let resp = ureq::post(&self.endpoint)
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .send_json(&body);

        let response: serde_json::Value = match resp {
            Ok(mut r) => r.body_mut().read_json()?,
            Err(ureq::Error::StatusCode(429)) => {
                anyhow::bail!("rate limited by API -- wait a moment and try again")
            }
            Err(ureq::Error::StatusCode(401)) => {
                anyhow::bail!("invalid API key")
            }
            Err(e) => return Err(e.into()),
        };

        let command = parse_tool_call(&response)?;

        let explanation = if think {
            response
                .pointer("/choices/0/message/reasoning")
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
