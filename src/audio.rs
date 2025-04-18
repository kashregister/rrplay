use rodio::OutputStream;
use rodio::Sink;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AudioPlayer {
    pub playing: Arc<Mutex<bool>>,
    pub skip: Arc<Mutex<bool>>,
    pub stop: Arc<Mutex<bool>>,
    pub sink: Arc<Mutex<Option<Sink>>>,
    _stream: OutputStream, // to keep it alive
}

impl AudioPlayer {
    pub fn init() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        AudioPlayer {
            playing: Arc::new(Mutex::new(true)),
            skip: Arc::new(Mutex::new(false)),
            stop: Arc::new(Mutex::new(false)),
            sink: Arc::new(Mutex::new(Some(Sink::try_new(&stream_handle).unwrap()))),
            _stream: _stream,
        }
    }

    pub fn play(&mut self) {
        let mut playing_guard = self.playing.lock().unwrap();
        *playing_guard = true;
        // self.sink.stop();
    }

    // pub fn is_playing(&mut self) -> bool {
    //     let playing_guard = self.playing.lock().unwrap();
    //     playing_guard.eq(&true)
    // }

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
