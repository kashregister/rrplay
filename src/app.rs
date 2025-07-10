use crate::event::{AppEvent, Event, EventHandler};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use lofty::file::AudioFile;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use rodio::{OutputStream, Sink};
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;
use walkdir::WalkDir;

const VOLUME_CHANGE: f32 = 0.1;
const SEEK_CHANGE: Duration = Duration::from_secs(5);
/// Application.
// #[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub search_results: Vec<String>,
    pub search_cache: Vec<String>,
    pub queue: Vec<(String, Duration)>,
    pub query: String,
    pub select_index: usize,
    pub sources: Option<Vec<(String, bool)>>,
    /// Event handler.
    pub events: EventHandler,
    pub mode: Mode,
    pub sink: Sink,
    pub stream: OutputStream,
    // pub stream_handle: OutputStreamHandle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Sitback,
    Search,
    Select,
}

impl Default for App {
    fn default() -> Self {
        // let tuple: (OutputStream, OutputStreamHandle) = OutputStream::try_default().unwrap();
        Self {
            running: true,
            search_results: Vec::new(),
            search_cache: Vec::new(),
            queue: Vec::new(),
            query: String::new(),
            events: EventHandler::new(),
            mode: Mode::Sitback,
            select_index: 0,
            sources: App::check_config_validity(),
            sink: Sink::try_new(&OutputStream::try_default().unwrap().1).unwrap(),
            stream: OutputStream::try_default().unwrap().0,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        let mut out = Self::default();
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        out.sink = Sink::try_new(&stream_handle).unwrap();
        out.stream = stream;
        if let Some(_) = out.sources.clone() {
            out.search_cache = App::search_directories(out.sources.clone());
        } else {
            exit(1);
        }
        out
    }

    pub fn config_check_file_exists() -> bool {
        if let Some(cfg_dir) = dirs::config_dir() {
            let config_file = cfg_dir.join("rrplay").join("config");
            if !config_file.is_file() {
                return false;
            } else {
                return true;
            }
        } else {
            return false;
        }
    }

    pub fn check_config_validity() -> Option<Vec<(String, bool)>> {
        if App::config_check_file_exists() {
            if let Some(cfg_dir) = dirs::config_dir() {
                let config_file = cfg_dir.join("rrplay").join("config");
                let file_contents = std::fs::read_to_string(config_file).unwrap();
                let paths = file_contents.split("\n");
                let mut output: Vec<(String, bool)> = Vec::new();
                for path in paths {
                    let ap = path.trim().to_string();

                    if Path::new(&ap).exists() {
                        output.push((ap, true));
                    } else {
                        output.push((ap, false));
                    }
                }
                Some(output)
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn search_directories(sources: Option<Vec<(String, bool)>>) -> Vec<String> {
        if let Some(sources) = sources {
            let mut out = Vec::new();
            for source in sources {
                if source.1 == true {
                    let file_types = [
                        "flac", "m4a", "mp3", "wav", "ogg", "opus", "m4p", "aiff", "3gp", "aac",
                    ];
                    let mut song_entries = Vec::new();
                    for entry in WalkDir::new(source.0).into_iter().filter_map(|e| e.ok()) {
                        if let Some(ext) = entry.path().extension() {
                            if file_types.contains(&ext.to_str().unwrap()) {
                                song_entries.push(entry.path().display().to_string());
                            }
                        }
                    }
                    out.append(&mut song_entries);
                }
            }
            out
        } else {
            Vec::new()
        }
    }
    /// Run the application's main loop.
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event)?,
                    _ => {}
                },
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Search => {
                        self.mode = Mode::Search;
                        // self.search_results = App::search_directory(String::from_str(
                        //     "/mnt/disk_new/Music Library/Oliver Francis",
                        // )?);
                    }
                    AppEvent::Select => {
                        if self.search_results.len() > 0 && self.mode == Mode::Search {
                            self.mode = Mode::Select;
                            self.select_index = self.search_results.len() - 1;
                        }
                    }
                    AppEvent::Escape => {
                        if self.mode == Mode::Select {
                            self.mode = Mode::Search;
                        } else {
                            self.mode = Mode::Sitback;
                        }
                    }
                    AppEvent::MoveUp => {
                        if self.select_index < self.search_results.len() - 1 {
                            self.select_index += 1;
                        }
                    }
                    AppEvent::MoveDown => {
                        if self.select_index > 0 {
                            self.select_index -= 1;
                        }
                    }
                    AppEvent::AddSingle => {
                        let index = self.search_results.len() - 1 - self.select_index;

                        if let Some(filename) =
                            Path::new(self.search_results[index].clone().as_str()).file_name()
                        {
                            let tagged_file =
                                lofty::read_from_path(self.search_results[index].clone())?;
                            let duration: Duration = tagged_file.properties().duration();
                            if let Ok(fname) = filename.to_owned().into_string() {
                                self.queue.push((fname, duration));
                            }
                        }

                        let path_str = self.search_results[index].clone();
                        let file = std::fs::File::open(path_str.clone()).unwrap();
                        self.sink
                            .append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                    }
                    AppEvent::AddAlbum => {
                        let index = self.search_results.len() - 1 - self.select_index;
                        let song = self.search_results[index].clone();
                        let file_types = [
                            "flac", "m4a", "mp3", "wav", "ogg", "opus", "m4p", "aiff", "3gp", "aac",
                        ];

                        // song is the single song
                        let mut queue = Vec::new();
                        let mut dir = PathBuf::from(song);
                        dir.pop();
                        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
                            if let Some(ext) = entry.path().extension() {
                                if file_types.contains(&ext.to_str().unwrap()) {
                                    let push = entry.path().to_owned();
                                    queue.push(push.into_os_string().into_string().unwrap());
                                }
                            }
                        }
                        for entry in queue {
                            if let Some(filename) = Path::new(entry.clone().as_str()).file_name() {
                                let tagged_file = lofty::read_from_path(entry.clone())?;
                                let duration: Duration = tagged_file.properties().duration();
                                if let Ok(fname) = filename.to_owned().into_string() {
                                    self.queue.push((fname, duration));
                                }
                            }

                            let path_str = entry.clone();
                            let file = std::fs::File::open(path_str.clone()).unwrap();
                            self.sink
                                .append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                        }
                    }
                    AppEvent::Resume => {
                        self.sink.play();
                    }
                    AppEvent::ClearQueue => {
                        self.sink.clear();
                        self.queue = Vec::new();
                    }
                    AppEvent::Pause => {
                        self.sink.pause();
                    }
                    AppEvent::Skip => {
                        self.sink.skip_one();
                    }
                    AppEvent::VolumeUp => {
                        let get_volume = self.sink.volume();
                        if get_volume + VOLUME_CHANGE > 1.0 {
                            self.sink.set_volume(1.0);
                        } else {
                            self.sink.set_volume(get_volume + VOLUME_CHANGE);
                        };
                    }
                    AppEvent::VolumeDown => {
                        let get_volume = self.sink.volume();
                        if get_volume - VOLUME_CHANGE < 0.0 {
                            self.sink.set_volume(0.0);
                        } else {
                            self.sink.set_volume(get_volume - VOLUME_CHANGE);
                        };
                    }

                    AppEvent::MoveForward => {
                        let pos = self.sink.get_pos();
                        if self.queue.len() > 0 {
                            if pos + SEEK_CHANGE < self.queue[0].1 {
                                let _ = self.sink.try_seek(pos + SEEK_CHANGE);
                            }
                        }
                    }
                    AppEvent::MoveBackward => {
                        let pos = self.sink.get_pos();

                        if self.queue.len() > 0 {
                            if pos.as_secs() as i32 - SEEK_CHANGE.as_secs() as i32 > 0 {
                                let _ = self.sink.try_seek(pos - SEEK_CHANGE);
                            } else {
                                let _ = self.sink.try_seek(Duration::from_millis(0));
                            }
                        }
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc => self.events.send(AppEvent::Escape),
            KeyCode::Enter => {
                if self.mode == Mode::Select {
                    self.events.send(AppEvent::AddSingle);
                    self.events.send(AppEvent::Resume)
                } else if self.mode == Mode::Search {
                    self.events.send(AppEvent::Select)
                }
            }
            _ => {}
        }
        if self.mode != Mode::Search {
            match key_event.code {
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('j') => self.events.send(AppEvent::MoveDown),
                KeyCode::Char('k') => self.events.send(AppEvent::MoveUp),
                KeyCode::Char('p') => {
                    if self.mode == Mode::Sitback {
                        if self.sink.is_paused() == true {
                            self.events.send(AppEvent::Resume);
                        } else {
                            self.events.send(AppEvent::Pause);
                        }
                    }
                }

                KeyCode::Char('s') => {
                    self.events.send(AppEvent::Skip);
                }

                KeyCode::Char('a') => {
                    if self.mode == Mode::Select {
                        self.events.send(AppEvent::AddAlbum);
                    }
                }
                KeyCode::Char('v') => {
                    self.events.send(AppEvent::VolumeDown);
                }
                KeyCode::Char('h') => {
                    self.events.send(AppEvent::MoveBackward);
                }
                KeyCode::Char('l') => {
                    self.events.send(AppEvent::MoveForward);
                }
                KeyCode::Char('V') => {
                    self.events.send(AppEvent::VolumeUp);
                }

                KeyCode::Char('c' | 'C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        self.events.send(AppEvent::Quit)
                    } else {
                        self.events.send(AppEvent::ClearQueue);
                    }
                }

                KeyCode::Char('/') => self.events.send(AppEvent::Search),

                // Other handlers you could add here.
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Enter => {}
                KeyCode::Char(' ') => {
                    self.query.push_str(" ");
                }
                KeyCode::Backspace => {
                    if KeyModifiers::ALT == key_event.modifiers {
                        self.query.clear();
                    } else {
                        self.query.pop();

                        if self.query.len() > 0 {
                            let matcher = SkimMatcherV2::default();
                            let mut entries_with_score: Vec<(String, i64)> = Vec::new();
                            for entry in self.search_cache.clone() {
                                if let Some(score) =
                                    matcher.fuzzy_match(entry.as_str(), self.query.as_str())
                                {
                                    if score > 0 {
                                        entries_with_score.push((entry, score));
                                    };
                                }
                            }

                            entries_with_score.sort_by(|a, b| b.1.cmp(&a.1));
                            self.search_results = Vec::new();
                            for entry in entries_with_score {
                                self.search_results.push(entry.0)
                            }
                        }
                    }
                }
                KeyCode::Esc => {}
                _ => {
                    self.query.push_str(&key_event.code.to_string());
                    if self.query.len() > 0 {
                        let matcher = SkimMatcherV2::default();
                        let mut entries_with_score: Vec<(String, i64)> = Vec::new();
                        for entry in self.search_cache.clone() {
                            if let Some(score) =
                                matcher.fuzzy_match(entry.as_str(), self.query.as_str())
                            {
                                if score > 0 {
                                    entries_with_score.push((entry, score));
                                };
                            }
                        }

                        entries_with_score.sort_by(|a, b| b.1.cmp(&a.1));
                        self.search_results = Vec::new();
                        for entry in entries_with_score {
                            self.search_results.push(entry.0)
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) {
        if self.sink.len() < self.queue.len() {
            self.queue.remove(0);
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
