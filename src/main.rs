use ::std::time::Duration;
use ::terminal_size::{Height, Width, terminal_size};
use console::Key;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll,
};
use crossterm::terminal;
use crossterm::{
    cursor::position,
    event::{Event, KeyCode, KeyModifiers, read},
};
use std::any::Any;
use std::io;
use std::thread::sleep;

// Return None if no key event or if it's not a character key

fn main() {
    let size = terminal_size();
    terminal::enable_raw_mode().unwrap();

    // execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
    // execute!(stdout, cursor::MoveTo(0, 0)).unwrap();

    if let Some((Width(w), Height(h))) = size {
        println!("Your terminal is {} cols wide and {} lines tall", w, h);
    } else {
        println!("Unable to get terminal size");
    }

    'input: loop {
        let event = read().unwrap();

        if let Event::Key(key_event) = event {
            if key_event.code == KeyCode::Esc
                || (key_event.code == KeyCode::Char('c')
                    && key_event.modifiers == KeyModifiers::CONTROL)
            {
                break 'input;
            }

            match key_event.code {
                KeyCode::Char(':') => {
                    println!("\r\ncmd mode");
                }

                KeyCode::Char(chr) => {
                    println!("\r{}", chr);
                }
                _ => {
                    println!(
                        "\r\npressing on key: {:?} with modifier {:?}",
                        key_event.code, key_event.modifiers
                    );
                }
            }
        }
    }
    terminal::disable_raw_mode();
    println!("Bye!");
}
