use crate::setup::Personalization;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

const FIELDS: &[(&str, &str)] = &[
    ("tone", "Tone (e.g., calm, chaotic, warm, dry)"),
    ("humor_level", "Humor level (none / subtle / lanang / unhinged)"),
    ("formality", "Formality (casual / neutral / formal / poetic)"),
    ("custom_instructions", "Custom instructions (anything extra)"),
];

pub fn run_svisurt(eru_name: &str) -> Result<Personalization, String> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    execute!(stdout, terminal::EnterAlternateScreen).map_err(|e| e.to_string())?;

    let mut values: Vec<String> = FIELDS.iter().map(|_| String::new()).collect();
    let mut current_field = 0usize;

    loop {
        execute!(stdout, terminal::Clear(ClearType::All)).map_err(|e| e.to_string())?;
        write!(stdout, "\x1b[H").map_err(|e| e.to_string())?; 

        writeln!(stdout, "╔═══ SVISURT ═══ Personalize {} ═══╗", eru_name).map_err(|e| e.to_string())?;
        writeln!(stdout, "║ Tab/Enter: next field | Esc: save & exit ║").map_err(|e| e.to_string())?;
        writeln!(stdout, "╚═══════════════════════════════════════════╝").map_err(|e| e.to_string())?;
        writeln!(stdout).map_err(|e| e.to_string())?;

        for (i, (_, label)) in FIELDS.iter().enumerate() {
            let marker = if i == current_field { "►" } else { " " };
            writeln!(stdout, " {} {}: {}", marker, label, values[i]).map_err(|e| e.to_string())?;
        }

        writeln!(stdout).map_err(|e| e.to_string())?;
        write!(stdout, " ► Editing: {}_", FIELDS[current_field].1).map_err(|e| e.to_string())?;
        stdout.flush().map_err(|e| e.to_string())?;

        if let Ok(Event::Key(key)) = event::read() {
            match key.code {
                KeyCode::Esc => break,
                KeyCode::Tab | KeyCode::Enter => {
                    current_field = (current_field + 1) % FIELDS.len();
                }
                KeyCode::Backspace => {
                    values[current_field].pop();
                }
                KeyCode::Char(c) => {
                    values[current_field].push(c);
                }
                _ => {}
            }
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen).map_err(|e| e.to_string())?;
    terminal::disable_raw_mode().map_err(|e| e.to_string())?;

    Ok(Personalization {
        tone: values[0].clone(),
        humor_level: values[1].clone(),
        formality: values[2].clone(),
        custom_instructions: values[3].clone(),
    })
}
