use std::io::{Write, stdout};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

pub enum Action {
    Execute,
    Cancel,
}

pub fn confirm_command(command: &str, explanation: Option<&str>) -> Result<Action> {
    if let Some(explanation) = explanation {
        eprintln!("\x1b[2m{explanation}\x1b[0m\n");
    }

    eprint!("\x1b[1;32m>\x1b[0m {command}  \x1b[2m[enter] run  [esc] cancel\x1b[0m");
    stdout().flush()?;

    terminal::enable_raw_mode()?;
    let result = loop {
        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
            match code {
                KeyCode::Enter => break Action::Execute,
                KeyCode::Esc => break Action::Cancel,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    break Action::Cancel
                }
                _ => {}
            }
        }
    };
    terminal::disable_raw_mode()?;
    eprintln!();

    Ok(result)
}
