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
    VolumeUp,
    VolumeDown,
    Resume,
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
            let sink_ref = Arc::clone(&self.audio_player.sink);

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
                        let mut sink_guard = sink_ref.lock().unwrap();

                        if let Some(sink) = sink_guard.as_mut() {
                            if sink.empty() {
                                break;
                            }
                        }
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
            });
        }
    }

    pub fn audio_cmd(&mut self, pcmd: PlayerCommand) {
        match pcmd {
            PlayerCommand::Quit => self.mode = PlayerMode::Bye,
            PlayerCommand::Skip => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    sink.skip_one();
                }
            }
            PlayerCommand::ClearQueue => self.queue = None,
            PlayerCommand::TogglePause => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    if sink.is_paused() {
                        sink.play();
                    } else {
                        sink.pause();
                    }
                }
            }
            PlayerCommand::Resume => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    sink.play();
                }
            }
            PlayerCommand::Stop => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    sink.stop();
                }
            }
            PlayerCommand::Help => {}
            PlayerCommand::VolumeUp => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    let mut volume = sink.volume();
                    volume += 0.05;
                    if volume > 1.0 {
                        volume = 1.0;
                    }
                    sink.set_volume(volume);
                }
                self.display_query();
            }
            PlayerCommand::VolumeDown => {
                if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                    let mut volume = sink.volume();
                    volume -= 0.05;
                    if volume < 0.0 {
                        volume = 0.0;
                    }
                    sink.set_volume(volume);
                }
                self.display_query();
            }
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
        t_cursor_hide();
        t_mv_end();
        t_mv_sol();
        t_clear_line();

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
        if self.mode == PlayerMode::Search || self.mode == PlayerMode::Select {
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

        t_mv_col(t_sz.0 - 25);
        if self.mode != PlayerMode::Command {
            if let Some(ref mut sink) = *self.audio_player.sink.lock().unwrap() {
                let volume = (sink.volume() * 100.0) as i16;
                print!("volume: {}", volume);
            }
        }

        t_txt_bold();
        t_mv_col(t_sz.0 - 13);
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
        t_mv_end();
        self.display_query();
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
