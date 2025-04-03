use crossterm::ExecutableCommand;
use crossterm::cursor::{MoveTo, MoveToRow};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::terminal::{self};
use std::io::{self, Write};
use std::string::String;

mod audio;
mod config;
mod player;
mod search_utils;
mod term_utils;

use config::ConfigHandler;
use player::PlayerCommand;
use player::PlayerMode;
use player::PlayerState;
use search_utils::*;
use term_utils::*;

fn main() {
    let mut search_results = Vec::new();
    let mut player_state = PlayerState::init();
    player_state.cfg_handler.startup();

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
                    player_state.audio_cmd(PlayerCommand::TogglePause);
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
                    if let Some(song) = get_song(&search_results, player_state.index) {
                        player_state.audio_cmd(PlayerCommand::Stop);
                        player_state.audio_cmd(PlayerCommand::ClearQueue);
                        player_state.queue = Some(song);
                        player_state.play_queue();
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
            } else if key_event.code == KeyCode::Char('a') {
                if player_state.mode == PlayerMode::Select {
                    player_state.audio_cmd(PlayerCommand::Stop);
                    player_state.audio_cmd(PlayerCommand::ClearQueue);
                    player_state.queue = get_album(&search_results, player_state.index);
                    player_state.play_queue();
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
                            search_results =
                                walkdir(&mut query, player_state.cfg_handler.sources.clone());
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
                        search_results =
                            walkdir(&mut query, player_state.cfg_handler.sources.clone());
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
