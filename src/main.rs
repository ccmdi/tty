mod client;
mod exec;
mod prompt;
mod ui;

use std::io::IsTerminal;

use anyhow::{bail, Result};
use client::{LlmClient, groq::GroqClient};

struct Args {
    think: bool,
    show_reasoning: bool,
    query: String,
}

fn parse_args() -> Result<Args> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        bail!("Usage: tty \"your request\"\n       tty --think \"your request\"");
    }

    let mut think = false;
    let mut show_reasoning = false;
    let mut rest = &args[..];

    while !rest.is_empty() {
        match rest[0].as_str() {
            "--think" => {
                think = true;
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
        show_reasoning,
        query: rest.join(" "),
    })
}

fn run() -> Result<()> {
    let args = parse_args()?;

    let client = GroqClient::from_env()?;
    let system = prompt::build_system_prompt();
    let result = client.complete(&system, &args.query, args.think)?;

    let is_tty = std::io::stdout().is_terminal();

    if !is_tty {
        print!("{}", result.command);
        return Ok(());
    }

    let explanation = if args.show_reasoning {
        result.explanation.as_deref()
    } else {
        None
    };

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
