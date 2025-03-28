use ::std::time::Duration;
use ::terminal_size::{Height, Width, terminal_size};
use console::Key;
use crossterm::ExecutableCommand;
use crossterm::cursor::{MoveTo, MoveToColumn};
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll,
};
use crossterm::terminal::{self, ClearType};
use crossterm::{
    cursor::position,
    event::{Event, KeyCode, KeyModifiers, read},
};
use std::any::Any;
use std::io::{self, Cursor};
use std::string::String;

fn t_clear() {
    io::stdout().execute(terminal::Clear(ClearType::All));
}

fn t_mv_start() {
    io::stdout().execute(MoveTo(0, 0));
}

fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut cmd_mode: bool = false;
    let mut cmd_str: String = String::from("hello");
    let t_sz = terminal::size().unwrap();

    t_mv_start();
    t_clear();

    'input: loop {
        let event = read().unwrap();

        if let Event::Key(key_event) = event {
            if (key_event.code == KeyCode::Char('c')
                && key_event.modifiers == KeyModifiers::CONTROL)
            {
                break 'input;
            } else if key_event.code == KeyCode::Esc {
                if cmd_mode {
                    t_mv_start();
                    cmd_mode = false;
                    cmd_str.clear();
                }
            } else if key_event.code == KeyCode::Enter {
                if cmd_mode {
                    cmd_mode = false;
                    t_mv_start();
                    print!("{}", cmd_str);
                    cmd_str.clear();
                }
            }
            match key_event.code {
                KeyCode::Char(':') => {
                    if cmd_mode == false {
                        t_clear();
                        io::stdout().execute(MoveTo(0, t_sz.1));
                        print!(":");

                        cmd_mode = true;
                        cmd_str.clear();
                    }
                }

                KeyCode::Char(chr) => {
                    if cmd_mode == true {
                        print!("{}", key_event.code);
                        cmd_str.push(chr);
                    }
                }
                _ => {
                    if cmd_mode == true {
                        print!("{}", key_event.code);
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode();
    t_clear();
    println!("Bye!");
}
