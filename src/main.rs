use crossterm::ExecutableCommand;
use crossterm::cursor::{MoveTo, MoveToColumn, MoveToRow};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::style::{Colors, SetBackgroundColor};
use crossterm::terminal::{self};
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;
use std::result::Result;
use std::string::String;

mod player;
use player::AudioPlayer;

mod search_utils;
use search_utils::*;

mod term_utils;
use term_utils::*;

#[derive(PartialEq)]
enum PlayerMode {
    Sitback,
    Select,
    Command,
    Search,
}

#[derive(PartialEq)]
enum PlayerCommand {
    Quit,
}

struct PlayerState {
    mode: PlayerMode,
    query: Option<String>,
    sources: Option<Vec<String>>,
    index: i32,
}

impl PlayerState {
    fn search(&mut self) {
        self.mode = PlayerMode::Search;
    }
    fn sitback(&mut self) {
        self.mode = PlayerMode::Sitback;
    }

    fn command(&mut self) {
        self.mode = PlayerMode::Command;
    }
    fn select(&mut self) {
        self.mode = PlayerMode::Select;
    }
    fn push_chr(&mut self, char: char) {
        if let Some(mut query) = self.query.clone() {
            query.push(char);
            self.query = Some(query);
        }
    }
    fn pop(&mut self) {
        if let Some(mut query) = self.query.clone() {
            query.pop();
            self.query = Some(query);
        }
    }

    fn display_err(&self) {
        t_mv_end();
        t_clear_line();
        t_mv_sol();
        t_flush();
        t_bg_rgb([255, 20, 20]);
        print!("Error playing file");
        t_bg_reset();
        println!();
    }
    fn display_query(&self) {
        let t_sz = terminal::size().unwrap();
        t_flush();
        t_cursor_hide();
        t_mv_sol();
        if self.mode == PlayerMode::Command {
            if let Some(qr) = self.query.clone() {
                let query = qr;
                if let Some(qury) = query.chars().nth(0) {
                    if qury == ':' {
                        print!("{query}");
                    } else {
                        print!(":{query}");
                    }
                }
            }
        }
        if self.mode == PlayerMode::Search {
            if let Some(qr) = self.query.clone() {
                let query = qr;
                if let Some(qury) = query.chars().nth(0) {
                    if qury == '/' {
                        print!("{query}");
                    } else {
                        print!("/{query}");
                    }
                }
            }
        }

        t_flush();
        std::io::stdout()
            .execute(MoveToColumn(t_sz.0 - 20))
            .unwrap();

        t_txt_bold();

        if self.mode == PlayerMode::Search {
            print!("Search mode")
        }

        if self.mode == PlayerMode::Command {
            print!("Command mode")
        }

        if self.mode == PlayerMode::Sitback {
            print!("Sitback mode")
        }

        if self.mode == PlayerMode::Select {
            print!("Select mode")
        }
        t_txt_nobold();
        t_bg_reset();

        t_flush();
        t_mv_end();
    }

    fn info_print(&self) {
        print!("/ search\n\r");
        print!(": cmd mode\n\r");
        print!("enter - track mode\n");
        t_mv_sol();
        print!("p - pause\n\r");
        t_mv_end();
        if let Some(sources) = self.sources.clone() {
            for i in sources.into_iter() {
                t_mv_one_up();
                t_mv_sol();
                print!("{}", i);
            }
        }
    }
    fn run_cmd(&self) -> Result<PlayerCommand, ()> {
        if self.mode == PlayerMode::Command {
            if let Some(cmd) = self.query.clone() {
                println!("{cmd}");
                if cmd.eq(":q") {
                    Ok(PlayerCommand::Quit)
                } else {
                    t_mv_end();
                    t_clear_all();

                    t_flush();

                    t_bg_rgb([255, 165, 0]);
                    // t_mv_one_up();
                    print!("Wrong syntax");
                    t_bg_reset();
                    // println!();
                    Err(())
                }
            } else {
                Err(())
            }
        } else {
            Err(())
        }
    }
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
            return true;
        }
    } else {
        println!("failed fetching config dir");
        return false;
    }
}

fn main() {
    let mut search_results = Vec::new();
    let mut player = AudioPlayer::init();

    let mut player_state: PlayerState = PlayerState {
        mode: PlayerMode::Sitback,
        query: None,
        sources: None,
        index: 0,
    };

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
                player_state.sources = Some(Vec::new());

                let path = input.to_string().trim().to_string();
                if let Some(ref mut sources) = player_state.sources.clone() {
                    sources.push(path);
                }
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
            let mut path = std::fs::read_to_string(config_file).unwrap();

            path = path.trim().to_string();
            player_state.sources = Some(Vec::new());
            if let Some(ref mut sources) = player_state.sources {
                sources.push(path);
            }
        }
    }
    terminal::enable_raw_mode().unwrap();
    t_mv_start();
    t_clear_all();
    player_state.info_print();

    'input: loop {
        let event = read().unwrap();
        let t_sz = terminal::size().unwrap();
        if let Event::Resize(_x, _y) = event {
            // t_clear_all();
        }
        if let Event::Key(key_event) = event {
            if key_event.code == KeyCode::Char('p') {
                if player_state.mode == PlayerMode::Select
                    || player_state.mode == PlayerMode::Sitback
                {
                    let current_playing = player.is_playing();
                    if current_playing {
                        player.pause_song();
                    } else {
                        player.resume_song();
                    }
                }
            }
            if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL
            {
                break 'input;
            } else if key_event.code == KeyCode::Esc {
                t_cursor_show();
                player_state.sitback();
                player_state.query = None;
                t_mv_start();
                t_clear_all();
                player_state.info_print();
                player_state.index = 2;
                t_flush();
            } else if key_event.code == KeyCode::Enter {
                if player_state.mode == PlayerMode::Search {
                    player_state.select();

                    player_state.index = 2;

                    song_entries_print(&search_results, player_state.index);
                    player_state.display_query();

                    io::stdout()
                        .execute(MoveToRow(t_sz.1 - player_state.index as u16))
                        .unwrap();
                    t_mv_sol();
                } else if player_state.mode == PlayerMode::Command {
                    if let Ok(ret_cmd) = player_state.run_cmd() {
                        if ret_cmd.eq(&PlayerCommand::Quit) {
                            break 'input;
                        }
                    }

                    player_state.sitback();

                    t_flush();
                    t_mv_start();

                    if let Some(query) = player_state.query.clone() {
                        print!("{}", query);
                    } else {
                        println!("err");
                    }
                    player_state.query = None;
                } else if player_state.mode == PlayerMode::Select {
                    if let Ok(song) = get_song(&search_results, player_state.index) {
                        player.skip_song(true);
                        player.current_song = Some(song);
                        player.play_song();
                    } else {
                        player_state.display_err();
                    }
                }
            } else if key_event.code == KeyCode::Char(':') {
                if player_state.mode != PlayerMode::Command {
                    t_clear_all();
                    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();
                    t_flush();

                    player_state.command();
                    player_state.query = Some("".to_string());
                    player_state.display_query();
                }
            } else if key_event.code == KeyCode::Char('/') {
                if player_state.mode == PlayerMode::Select {
                    player_state.search();
                }
                if player_state.mode != PlayerMode::Search {
                    t_clear_all();
                    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();
                    t_flush();
                    player_state.search();
                    player_state.query = Some(String::new());
                }
            } else if key_event.code == KeyCode::Char('j') {
                if player_state.mode == PlayerMode::Select && player_state.index > 2 {
                    player_state.index -= 1;
                    song_entries_print(&search_results, player_state.index);
                    player_state.display_query();
                    io::stdout()
                        .execute(MoveToRow(t_sz.1 - player_state.index as u16))
                        .unwrap();
                    t_mv_sol();
                    t_flush();
                }
            } else if key_event.code == KeyCode::Char('k') {
                if player_state.mode == PlayerMode::Select && player_state.index < t_sz.1 as i32 {
                    if player_state.index - 1 < search_results.len() as i32 {
                        player_state.index += 1;
                        song_entries_print(&search_results, player_state.index);
                        player_state.display_query();
                        io::stdout()
                            .execute(MoveToRow(t_sz.1 - player_state.index as u16))
                            .unwrap();
                        t_mv_sol();
                        t_flush();
                    }
                }
            } else if key_event.code == KeyCode::Backspace {
                if player_state.mode == PlayerMode::Command {
                    if player_state.query == None || player_state.query == Some("".to_string()) {
                        player_state.sitback();
                        t_clear_all();
                        t_mv_start();
                        player_state.info_print();
                    } else {
                        player_state.pop();
                        t_clear_line();
                        t_mv_sol();
                        io::stdout().flush().unwrap();
                        player_state.display_query();
                    }
                } else if player_state.mode == PlayerMode::Search {
                    if player_state.query == None || player_state.query == Some("".to_string()) {
                        player_state.sitback();
                        t_clear_all();
                        t_mv_start();
                        player_state.info_print();
                        player_state.query = None;
                    } else {
                        player_state.search();
                        player_state.pop();
                        if let Some(mut query) = player_state.query.clone() {
                            search_results = walkdir(&mut query, &player_state.sources);
                            song_entries_print(&search_results, player_state.index);
                            player_state.display_query();
                        }
                        t_mv_sol();
                        io::stdout().flush().unwrap();
                    }
                }
            }
            if let KeyCode::Char(chr) = key_event.code {
                if player_state.mode == PlayerMode::Command {
                    io::stdout().flush().unwrap();
                    player_state.push_chr(chr);
                    player_state.display_query();
                } else if player_state.mode == PlayerMode::Search {
                    io::stdout().flush().unwrap();
                    player_state.push_chr(chr);
                    player_state.display_query();
                    if let Some(mut query) = player_state.query.clone() {
                        search_results = walkdir(&mut query, &player_state.sources);
                        song_entries_print(&search_results, player_state.index);
                        player_state.display_query();
                    }
                } else if player_state.mode == PlayerMode::Sitback {
                    t_clear_all();
                    t_mv_start();
                    player_state.info_print();
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
