use crossterm::cursor::{MoveTo, MoveToColumn, MoveToRow};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::style::{ResetColor, SetAttribute, SetBackgroundColor};
use crossterm::terminal::{self, ClearType};
use crossterm::{ExecutableCommand, cursor};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use rodio::cpal::default_host;
use rodio::source::SamplesConverter;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::sync::Mutex;

use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use std::result::Result;
use std::string::String;
use std::sync::Arc;
use std::thread::{JoinHandle, Thread, sleep, spawn};
use std::time::Duration;
use std::{result, thread};
use walkdir::{self, WalkDir};
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

fn main() {
    terminal::enable_raw_mode().unwrap();
    let mut search_mode: bool = false;
    let mut search_str: String = String::new();
    let mut cmd_mode: bool = false;
    let mut cmd_str: String = String::new();
    let mut index = 0;
    let mut search_results = Vec::new();
    let mut track_mode: bool = false;

    let mut player = Player::init();

    t_mv_start();
    t_clear_all();

    'input: loop {
        let event = read().unwrap();
        let t_sz = terminal::size().unwrap();
        if let Event::Key(key_event) = event {
            if key_event.code == KeyCode::Char('p') {
                if !cmd_mode || !search_mode {
                    // Get the current state
                    let current_playing = player.is_playing();

                    // Toggle the state based on current value
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
                        search_results = walkdir(&mut search_str);
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
                        search_results = walkdir(&mut search_str);
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
