mod client;
mod prompt;

use anyhow::{bail, Result};
use client::{LlmClient, groq::GroqClient};

fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        bail!("Usage: tty \"your request\"\n       tty --think \"your request\"");
    }

    let (think, query) = if args[0] == "--think" {
        if args.len() < 2 {
            bail!("Usage: tty --think \"your request\"");
        }
        (true, args[1..].join(" "))
    } else {
        (false, args.join(" "))
    };

    let client = GroqClient::from_env()?;
    let system = prompt::build_system_prompt();
    let result = client.complete(&system, &query, think)?;

    if let Some(explanation) = &result.explanation {
        eprintln!("\x1b[2m{explanation}\x1b[0m");
    }

    println!("{}", result.command);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("tty: {e}");
        std::process::exit(1);
    }
}
