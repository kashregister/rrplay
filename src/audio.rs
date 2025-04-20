use rodio::OutputStream;
use rodio::Sink;
use std::sync::Arc;
use std::sync::Mutex;

pub struct AudioPlayer {
    pub sink: Arc<Mutex<Option<Sink>>>,
    _stream: OutputStream, // to keep it alive
}

impl AudioPlayer {
    pub fn init() -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        AudioPlayer {
            sink: Arc::new(Mutex::new(Some(Sink::try_new(&stream_handle).unwrap()))),
            _stream: _stream,
        }
    }
}
