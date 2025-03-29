use crate::SongEntry;
use rodio::Sink;
use rodio::*;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub struct Player {
    pub current_song: SongEntry,
    playing: Arc<Mutex<bool>>,
    skip: Arc<Mutex<bool>>,
}

impl Player {
    pub fn init() -> Player {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let entry = SongEntry {
            file: PathBuf::new(),
            score: 0,
        };
        return Player {
            current_song: entry,
            playing: Arc::new(Mutex::new(true)),
            skip: Arc::new(Mutex::new(false)),
        };
    }
    pub fn play_song(&mut self) {
        let file = BufReader::new(File::open(&self.current_song.file).unwrap());
        self.resume_song();
        let playing = self.playing.clone();

        let skip = self.skip.clone();
        let handle = thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            let source = Decoder::new(file).unwrap();

            sink.append(source);
            while !sink.empty() {
                let playing_guard = playing.lock().unwrap();

                let skip_guard = skip.lock().unwrap();
                if playing_guard.eq(&false) {
                    sink.pause();
                } else {
                    sink.play();
                }
                if skip_guard.eq(&true) {
                    sink.stop();
                    break;
                }
                // thread::sleep(Duration::new(1, 0));
            }

            // sink.sleep_until_end();
        });

        self.skip_song(false);
    }

    pub fn is_playing(&mut self) -> bool {
        let playing_guard = self.playing.lock().unwrap();
        if playing_guard.eq(&true) {
            return true;
        } else {
            return false;
        }
    }
    pub fn pause_song(&mut self) {
        let mut playing_guard = self.playing.lock().unwrap();
        *playing_guard = false;
    }
    pub fn resume_song(&mut self) {
        let mut playing_guard = self.playing.lock().unwrap();
        *playing_guard = true;
    }
    pub fn skip_song(&mut self, state: bool) {
        let mut skip_guard = self.skip.lock().unwrap();
        *skip_guard = state;
    }
}
