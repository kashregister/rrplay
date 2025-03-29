use crossterm::ExecutableCommand;
use crossterm::cursor::{MoveTo, MoveToRow};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::terminal::{self};
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::result::Result;
use std::string::String;
// Import audio player struct
mod player;
use player::Player;

mod search_utils;
use search_utils::*;

mod term_utils;
use term_utils::*;

fn run_cmd(cmd: &String) -> Result<&'static str, &'static str> {
    if cmd.eq(":q") {
        Ok("exit")
    } else if cmd.eq("source") {
        Ok(":source")
    } else {
        Err("Wrong syntax")
    }
}

fn display_query(query: &String) {
    t_flush();
    t_cursor_hide();
    t_mv_sol();
    print!("/{query}");
    t_mv_end();
}

fn check_config_dir() -> bool {
    if let Some(cfg_dir) = dirs::config_dir() {
        let config_file = cfg_dir.join("rrplay").join("config");
        if let Ok(path) = std::fs::read_to_string(config_file) {
            let path = path.trim().to_string();
            if !Path::exists(Path::new(&path)) {
                t_clear_all();
                t_mv_sol();
                println!("invalid path in config file");
                t_mv_sol();

                terminal::disable_raw_mode().unwrap();
                exit(1);
            }
        }
    }
    if let Some(cfg_dir) = dirs::config_dir() {
        let config_file = cfg_dir.join("rrplay").join("config");
        if !config_file.is_file() {
            return false;
        } else {
            println!("{}", config_file.to_str().unwrap());
            return true;
        }
    } else {
        println!("failed fetching config dir");
        return false;
    }
}

fn main() {
    let mut search_mode: bool = false;
    let mut search_str: String = String::new();
    let mut cmd_mode: bool = false;
    let mut cmd_str: String = String::new();
    let mut index = 0;
    let mut search_results = Vec::new();
    let mut track_mode: bool = false;
    let mut path = String::new();
    let mut player = Player::init();
    if check_config_dir() == false {
        println!("No config found, creating under .config/rrplay/config");
        t_mv_sol();
        println!("Enter the full path of your library for example: /home/kr24/Music");
        t_mv_sol();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Err");
        t_flush();
        let input = input.trim();
        // println!("{input}");
        if Path::new(&input).exists() {
            println!("Path valid, creating config file...");
            t_mv_sol();
            if let Some(cfg_dir) = dirs::config_dir() {
                let mut config_file = cfg_dir.join("rrplay");
                std::fs::create_dir_all(config_file.clone()).unwrap();

                println!("writing to {}", config_file.to_str().unwrap());

                config_file = cfg_dir.join("rrplay/config");
                std::fs::write(config_file, input).unwrap();
                println!("File created!");

                t_mv_sol();
                path = input.to_string();
            } else {
                println!("Path invalid, aborting...");
                exit(1);
            }
        } else {
            println!("Err....");
            exit(1);
        }
    } else {
        if let Some(cfg_dir) = dirs::config_dir() {
            let config_file = cfg_dir.join("rrplay").join("config");
            path = std::fs::read_to_string(config_file).unwrap();
            println!("{path}");
        }
    }

    let path = path.trim().to_string();
    terminal::enable_raw_mode().unwrap();
    t_mv_start();
    t_clear_all();

    'input: loop {
        let event = read().unwrap();
        let t_sz = terminal::size().unwrap();
        if let Event::Key(key_event) = event {
            if key_event.code == KeyCode::Char('p') && (!cmd_mode || !search_mode) {
                // Get the current state
                let current_playing = player.is_playing();

                // Toggle the state based on current value
                if current_playing {
                    player.pause_song();
                } else {
                    player.resume_song();
                }
            }
            if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL
            {
                break 'input;
            } else if key_event.code == KeyCode::Esc {
                if cmd_mode || search_mode || track_mode {
                    t_cursor_show();
                    search_mode = false;
                    cmd_mode = false;
                    track_mode = false;
                    t_mv_start();
                    cmd_str.clear();
                    io::stdout().flush().unwrap();
                }
            } else if key_event.code == KeyCode::Enter {
                if search_mode {
                    t_cursor_show();
                    track_mode = true;
                    index = 2;
                    search_mode = false;
                    song_entries_print(&search_results, index);
                    display_query(&search_str);
                    io::stdout()
                        .execute(MoveToRow(t_sz.1 - index as u16))
                        .unwrap();
                    t_mv_sol();
                } else if cmd_mode {
                    cmd_mode = false;
                    let res = run_cmd(&cmd_str);

                    t_clear_all();
                    t_mv_end();
                    if let Ok(good) = res {
                        print!("Running command: {}", good);
                        if good == "exit" {
                            break 'input;
                        }
                    } else if let Err(bad) = res {
                        print!("{}", bad);
                    }
                    io::stdout().flush().unwrap();
                    t_mv_start();
                    print!("{}", cmd_str);
                    cmd_str.clear();
                } else if track_mode {
                    let song = get_song(&search_results, index);
                    player.skip_song(true);
                    player.current_song = song;
                    player.play_song();

                    // track_mode = false;
                }
            } else if key_event.code == KeyCode::Char(':') {
                if !cmd_mode {
                    t_clear_all();
                    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();
                    io::stdout().flush().unwrap();
                    cmd_mode = true;
                    cmd_str.clear();
                }
            } else if key_event.code == KeyCode::Char('/') {
                if !search_mode {
                    t_clear_all();
                    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();

                    io::stdout().flush().unwrap();
                    search_mode = true;
                    search_str.clear();
                }
            } else if key_event.code == KeyCode::Char('j') {
                if track_mode && index > 2 {
                    index -= 1;
                    song_entries_print(&search_results, index);
                    display_query(&search_str);

                    io::stdout()
                        .execute(MoveToRow(t_sz.1 - index as u16))
                        .unwrap();
                    t_mv_sol();
                    io::stdout().flush().unwrap();
                }
            } else if key_event.code == KeyCode::Char('k') {
                if track_mode && index < t_sz.1 as usize {
                    index += 1;
                    song_entries_print(&search_results, index);
                    display_query(&search_str);

                    io::stdout()
                        .execute(MoveToRow(t_sz.1 - index as u16))
                        .unwrap();
                    t_mv_sol();
                    io::stdout().flush().unwrap();
                }
            } else if key_event.code == KeyCode::Backspace {
                if cmd_mode {
                    if cmd_str.is_empty() {
                        cmd_mode = false;
                        t_clear_all();
                        t_mv_start();
                        cmd_str.clear();
                    } else {
                        cmd_str.pop();
                        t_clear_line();
                        t_mv_sol();
                        io::stdout().flush().unwrap();
                    }
                } else if search_mode {
                    if search_str.is_empty() {
                        search_mode = false;
                        t_clear_all();
                        t_mv_start();
                        cmd_str.clear();
                    } else {
                        search_str.pop();
                        search_results = walkdir(&mut search_str, path.clone());
                        song_entries_print(&search_results, index);
                        display_query(&search_str);
                        t_mv_sol();
                        io::stdout().flush().unwrap();
                    }
                }
            }
            if let KeyCode::Char(chr) = key_event.code {
                if cmd_mode {
                    print!("{}", chr);
                    io::stdout().flush().unwrap();
                    cmd_str.push(chr);
                } else if search_mode {
                    print!("{}", chr);
                    io::stdout().flush().unwrap();
                    search_str.push(chr);
                    if !search_str.is_empty() {
                        search_results = walkdir(&mut search_str, path.clone());
                        song_entries_print(&search_results, index);
                        display_query(&search_str);
                    }
                }
            }
        }
    }

    terminal::disable_raw_mode().unwrap();
    t_clear_all();
    t_mv_start();
    t_cursor_show();
    t_bg_reset();
}
