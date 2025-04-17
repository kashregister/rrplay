use crate::audio::AudioPlayer;

use crate::search_utils::{get_album, get_song, song_entries_print, walkdir};
use crate::term_utils::*;
use crate::{ConfigHandler, SongEntry};

use rodio::Decoder;

use std::path::Path;
use std::{fs::File, io::BufReader, sync::Arc};

use tokio::task;
use tokio::time::Duration;

#[derive(PartialEq)]
pub enum PlayerMode {
    Sitback,
    Select,
    Command,
    Search,
    Bye,
}

#[derive(PartialEq)]
pub enum PlayerCommand {
    Quit,
    Skip,
    ClearQueue,
    TogglePause,
    Stop,
    Help,
}

#[derive(PartialEq)]
pub enum SearchCommand {
    GetAlbum,
    GetSingle,
    PrintEntries,
    SearchResults,
}

pub struct PlayerState {
    pub mode: PlayerMode,
    pub query: Option<String>,
    pub cfg_handler: ConfigHandler,
    pub queue: Option<Vec<String>>,
    pub current_song: Option<SongEntry>,
    pub index: i32,
    pub audio_player: AudioPlayer,
    pub search_results: Option<Vec<String>>,
}

impl PlayerState {
    pub fn init() -> PlayerState {
        return PlayerState {
            mode: PlayerMode::Sitback,
            query: None,
            cfg_handler: ConfigHandler::init(),
            queue: None,
            current_song: None,
            index: 2,
            audio_player: AudioPlayer::init(),
            search_results: None,
        };
    }
    pub async fn play_queue(&mut self) {
        // Stop and clear previous sink
        if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
            sink.stop();
        }

        if let Some(queue) = self.queue.clone() {
            let playing = Arc::clone(&self.audio_player.playing);
            let stop = Arc::clone(&self.audio_player.stop);
            let skip = Arc::clone(&self.audio_player.skip);
            let sink_ref = Arc::clone(&self.audio_player.sink);

            self.audio_player.play();

            task::spawn_blocking(move || {
                for entry in queue {
                    let file = BufReader::new(File::open(entry).unwrap());
                    let source = Decoder::new(file).unwrap();

                    if let Ok(mut sink_guard) = sink_ref.lock() {
                        if let Some(sink) = sink_guard.as_mut() {
                            sink.append(source);
                        }
                    }
                }

                loop {
                    {
                        let playing_guard = playing.lock().unwrap();
                        let stop_guard = stop.lock().unwrap();
                        let mut skip_guard = skip.lock().unwrap();
                        let mut sink_guard = sink_ref.lock().unwrap();

                        if let Some(sink) = sink_guard.as_mut() {
                            if *stop_guard {
                                sink.stop();
                                break;
                            }

                            if !*playing_guard {
                                sink.pause();
                            } else {
                                sink.play();
                            }

                            if *skip_guard {
                                sink.stop(); // crude skip
                                *skip_guard = false;
                                break;
                            }

                            if sink.empty() {
                                break;
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_millis(100));
                }
            });

            self.audio_player.skip_song(false);
            self.audio_player.stop_all(false);
        }
    }

    pub fn audio_cmd(&mut self, pcmd: PlayerCommand) {
        match pcmd {
            PlayerCommand::Quit => self.mode = PlayerMode::Bye,
            PlayerCommand::Skip => self.audio_player.skip_song(true),
            PlayerCommand::ClearQueue => self.queue = None,
            PlayerCommand::TogglePause => self.audio_player.toggle_pause(),
            PlayerCommand::Stop => self.audio_player.stop_all(true),
            PlayerCommand::Help => {}
        }
    }

    pub fn search_cmd(&mut self, search_cmd: SearchCommand) {
        match search_cmd {
            SearchCommand::GetAlbum => {
                if let Some(has_results) = self.search_results.clone() {
                    self.queue = get_album(&has_results, self.index)
                }
            }
            SearchCommand::GetSingle => {
                if let Some(has_results) = self.search_results.clone() {
                    self.queue = get_song(&has_results, self.index)
                } else {
                    self.display_err();
                }
            }
            SearchCommand::PrintEntries => {
                if let Some(has_results) = self.search_results.clone() {
                    song_entries_print(&has_results, self.index);
                }
            }
            SearchCommand::SearchResults => {
                if let Some(mut query_exists) = self.query.clone() {
                    self.search_results =
                        Some(walkdir(&mut query_exists, self.cfg_handler.sources.clone()));
                }
            }
        }
    }
    pub fn search(&mut self) {
        self.mode = PlayerMode::Search;
    }
    pub fn sitback(&mut self) {
        self.mode = PlayerMode::Sitback;
    }

    pub fn command(&mut self) {
        self.mode = PlayerMode::Command;
    }
    pub fn select(&mut self) {
        self.mode = PlayerMode::Select;
    }
    pub fn push_chr(&mut self, char: char) {
        if let Some(mut query) = self.query.clone() {
            query.push(char);
            self.query = Some(query);
        }
    }
    pub fn pop(&mut self) {
        if let Some(mut query) = self.query.clone() {
            query.pop();
            self.query = Some(query);
        }
    }

    pub fn display_err(&self) {
        t_mv_end();
        t_clear_line();
        t_mv_sol();
        t_flush();
        t_bg_rgb([255, 20, 20]);
        print!("Error playing file");
        t_bg_reset();
        println!();
    }
    pub fn display_query(&self) {
        let t_sz = t_size_get();
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
        t_mv_col(t_sz.0 - 13);
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

    pub fn info_print(&self) {
        t_mv_sol();
        t_mv_start();
        if let Some(song) = self.current_song.clone() {
            println!("Current song:");
            t_mv_sol();
            println!("{}", song.file.to_str().unwrap());
        }

        if let Some(queue_ok) = self.queue.clone() {
            println!("Queue:");
            for song in queue_ok {
                t_mv_sol();
                let path = Path::new(&song);
                if let Some(fname) = path.file_name() {
                    println!("{}", fname.to_str().unwrap());
                }
            }
        }
        t_flush();
        t_mv_end();
        for (source, valid) in self.cfg_handler.sources.clone().into_iter() {
            t_mv_one_up();
            t_mv_sol();
            if valid {
                t_bg_rgb([145, 230, 100]);
                t_txt_rgb([255, 255, 255]);
                t_txt_bold();
                print!("{}", source);
                t_bg_reset();
            } else {
                t_bg_rgb([255, 0, 0]);
                t_txt_rgb([230, 137, 137]);
                t_txt_bold();
                print!("{}", source);
                t_bg_reset();
            }

            t_flush();
        }
        print!("Sourcing from:");
        t_flush();
        t_cursor_hide();
    }
    pub fn run_cmd(&self) -> Result<PlayerCommand, ()> {
        if self.mode == PlayerMode::Command {
            if let Some(cmd) = self.query.clone() {
                println!("{cmd}");
                if cmd.eq(":q") {
                    Ok(PlayerCommand::Quit)
                } else if cmd.eq(":h") || cmd.eq(":help") {
                    Ok(PlayerCommand::Help)
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
