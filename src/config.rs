use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::detect;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub client: ClientConfig,
    #[serde(default)]
    pub context: ContextConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientConfig {
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
    pub endpoint: String,
    pub model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default)]
    pub os: Option<String>,
    #[serde(default)]
    pub shell: Option<String>,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BehaviorConfig {
    #[serde(default)]
    pub auto_execute: bool,
    #[serde(default)]
    pub show_reasoning: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_execute: false,
            show_reasoning: false,
        }
    }
}

fn default_temperature() -> f64 {
    0.2
}
fn default_max_tokens() -> u32 {
    200
}

fn config_dir() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_fallback().join(".config")
        })
        .join("tty")
}

fn dirs_fallback() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();

        if path.exists() {
            let content =
                std::fs::read_to_string(&path).context("failed to read config file")?;
            let mut config: Config =
                toml::from_str(&content).context("failed to parse config file")?;

            // Env var overrides config file
            if let Ok(key) = std::env::var("GROQ_API_KEY") {
                if config.client.provider == "groq" {
                    config.client.api_key = Some(key);
                }
            }
            return Ok(config);
        }

        // No config file -- try env var fallback
        if let Ok(key) = std::env::var("GROQ_API_KEY") {
            let env = detect::detect();
            return Ok(Config {
                client: ClientConfig {
                    provider: "groq".into(),
                    api_key: Some(key),
                    endpoint: "https://api.groq.com/openai/v1/chat/completions".into(),
                    model: "qwen/qwen3-32b".into(),
                    temperature: default_temperature(),
                    max_tokens: default_max_tokens(),
                },
                context: ContextConfig {
                    os: Some(env.os),
                    shell: Some(env.shell),
                    tools: env.tools,
                },
                behavior: BehaviorConfig::default(),
            });
        }

        anyhow::bail!(
            "no config found. Run `tty init` or set GROQ_API_KEY environment variable"
        )
    }

    pub fn init() -> Result<()> {
        let dir = config_dir();
        std::fs::create_dir_all(&dir).context("failed to create config directory")?;

        let path = config_path();
        if path.exists() {
            eprintln!("config already exists at {}", path.display());
            eprintln!("delete it first if you want to reinitialize");
            return Ok(());
        }

        let env = detect::detect();

        let config = Config {
            client: ClientConfig {
                provider: "groq".into(),
                api_key: Some("YOUR_API_KEY_HERE".into()),
                endpoint: "https://api.groq.com/openai/v1/chat/completions".into(),
                model: "qwen/qwen3-32b".into(),
                temperature: default_temperature(),
                max_tokens: default_max_tokens(),
            },
            context: ContextConfig {
                os: Some(env.os),
                shell: Some(env.shell.clone()),
                tools: env.tools,
            },
            behavior: BehaviorConfig::default(),
        };

        let content = toml::to_string_pretty(&config).context("failed to serialize config")?;
        std::fs::write(&path, &content).context("failed to write config file")?;

        eprintln!("config written to {}", path.display());
        eprintln!("detected: {} / {}", config.context.os.as_deref().unwrap_or("?"), env.shell);
        eprintln!(
            "detected tools: {}",
            config.context.tools.join(", ")
        );
        eprintln!("\nedit the file to set your API key");

        Ok(())
    }
}
