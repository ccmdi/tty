mod client;
mod config;
mod detect;
mod exec;
mod prompt;
mod research;
mod ui;

use std::io::IsTerminal;

use anyhow::{bail, Result};
use client::LlmClient;

struct Args {
    think: bool,
    research: bool,
    show_reasoning: bool,
    command: Command,
}

enum Command {
    Query(String),
    Init,
}

fn parse_args() -> Result<Args> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        bail!("Usage: tty \"your request\"\n       tty --think \"your request\"\n       tty init");
    }

    if args[0] == "init" {
        return Ok(Args {
            think: false,
            research: false,
            show_reasoning: false,
            command: Command::Init,
        });
    }

    let mut think = false;
    let mut research = false;
    let mut show_reasoning = false;
    let mut rest = &args[..];

    while !rest.is_empty() {
        match rest[0].as_str() {
            "--think" => {
                think = true;
                rest = &rest[1..];
            }
            "--research" | "-r" => {
                research = true;
                rest = &rest[1..];
            }
            "--show-reasoning" => {
                show_reasoning = true;
                rest = &rest[1..];
            }
            _ => break,
        }
    }

    if rest.is_empty() {
        bail!("Usage: tty \"your request\"");
    }

    Ok(Args {
        think,
        research,
        show_reasoning,
        command: Command::Query(rest.join(" ")),
    })
}

fn build_client(cfg: &config::Config) -> Result<Box<dyn LlmClient>> {
    match cfg.client.provider.as_str() {
        "groq" => Ok(Box::new(client::groq::GroqClient::from_config(&cfg.client)?)),
        "ollama" => Ok(Box::new(client::ollama::OllamaClient {
            endpoint: cfg.client.endpoint.clone(),
            model: cfg.client.model.clone(),
            temperature: cfg.client.temperature,
            max_tokens: cfg.client.max_tokens,
        })),
        other => bail!("unknown provider: {other}. supported: groq, ollama"),
    }
}

fn run() -> Result<()> {
    let args = parse_args()?;

    if let Command::Init = args.command {
        return config::Config::init();
    }

    let query = match &args.command {
        Command::Query(q) => q,
        _ => unreachable!(),
    };

    let cfg = config::Config::load()?;
    let show_reasoning = args.show_reasoning || cfg.behavior.show_reasoning;
    let system = prompt::build_system_prompt(&cfg.context);

    let result = if args.research {
        research::research(&cfg.client, &system, query)?
    } else {
        let client = build_client(&cfg)?;
        client.complete(&system, query, args.think)?
    };

    let is_tty = std::io::stdout().is_terminal();

    if !is_tty {
        print!("{}", result.command);
        return Ok(());
    }

    let explanation = if show_reasoning {
        result.explanation.as_deref()
    } else {
        None
    };

    if cfg.behavior.auto_execute {
        if let Some(exp) = explanation {
            eprintln!("\x1b[2m{exp}\x1b[0m\n");
        }
        eprintln!("\x1b[1;32m>\x1b[0m {}", result.command);
        let code = exec::run_command(&result.command)?;
        std::process::exit(code);
    }

    match ui::confirm_command(&result.command, explanation)? {
        ui::Action::Execute => {
            let code = exec::run_command(&result.command)?;
            std::process::exit(code);
        }
        ui::Action::Cancel => {}
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("tty: {e}");
        std::process::exit(1);
    }
}
