use crate::SongEntry;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use std::thread;
use std::{path::PathBuf, sync::Mutex};

pub struct AudioPlayer {
    pub playing: Arc<Mutex<bool>>,
    pub skip: Arc<Mutex<bool>>,
    pub stop: Arc<Mutex<bool>>,
    pub next_queue: Arc<Mutex<bool>>,
}

impl AudioPlayer {
    pub fn init() -> Self {
        let entry = SongEntry {
            file: PathBuf::new(),
            score: 0,
        };

        // Initialize the output stream and sink

        AudioPlayer {
            playing: Arc::new(Mutex::new(true)),
            skip: Arc::new(Mutex::new(false)),
            stop: Arc::new(Mutex::new(false)),
            next_queue: Arc::new(Mutex::new(false)),
        }
    }

    pub fn play(&mut self) {
        let mut playing_guard = self.playing.lock().unwrap();
        *playing_guard = true;
    }

    pub fn is_playing(&mut self) -> bool {
        let playing_guard = self.playing.lock().unwrap();
        playing_guard.eq(&true)
    }

    pub fn skip_song(&mut self, state: bool) {
        let mut skip_guard = self.skip.lock().unwrap();
        *skip_guard = state;
    }

    pub fn stop_all(&mut self, state: bool) {
        let mut stop_guard = self.stop.lock().unwrap();
        *stop_guard = state;
    }
    pub fn toggle_pause(&mut self) {
        let mut playing_guard = self.playing.lock().unwrap();
        if playing_guard.eq(&false) {
            *playing_guard = true;
        } else {
            *playing_guard = false;
        }
    }
}
