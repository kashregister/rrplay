use crate::event::{AppEvent, Event, EventHandler};
use crate::widgets::{PopupManual, PopupNotif};
use crossterm::event::KeyEventKind;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use lofty::file::AudioFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Color,
};
use rodio::{OutputStream, Sink};
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;
use walkdir::WalkDir;

const VOLUME_CHANGE: f32 = 0.05;
const NOTIF_DURATION: usize = 60;
const SEEK_CHANGE: Duration = Duration::from_secs(5);
/// Application.
#[derive(Clone)]
pub struct Song {
    pub file_path: String,
    pub file_type: String,
    pub file_name: String,
    pub is_valid: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub genre: String,
    pub duration: Duration,
}
pub struct App {
    /// Is the application running?
    pub running: bool,

    pub search_results: Vec<Song>,
    pub search_cache: Vec<Song>,
    pub queue: Vec<Song>,
    pub query: String,
    pub select_index: usize,
    pub sources: Option<Vec<(String, bool)>>,
    /// Event handler.
    pub events: EventHandler,
    pub mode: Mode,
    pub sink: Sink,
    pub stream: OutputStream,
    pub terminal_size: (u16, u16),
    pub search_by: SearchBy,
    pub popup_manual: Option<PopupManual>,
    pub popup_notif: Vec<PopupNotif>,
    // pub stream_handle: OutputStreamHandle,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    Sitback,
    Search,
    Select,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SearchBy {
    FilePath,
    Title,
    Artist,
    Album,
    Genre,
}

impl Default for App {
    fn default() -> Self {
        // Constructs a new instance of [`App`].
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sources = App::check_config_validity();

        let mut init = Self {
            running: true,
            search_results: Vec::new(),
            search_cache: Vec::new(),
            queue: Vec::new(),
            query: String::new(),
            events: EventHandler::default(),
            mode: Mode::Sitback,
            select_index: 0,
            sources,
            sink: Sink::try_new(&stream_handle).unwrap(),
            stream: stream,
            terminal_size: (0, 0),
            search_by: SearchBy::FilePath,
            popup_manual: None,
            popup_notif: Vec::new(),
        };
        init.events.send(AppEvent::InitPopup);
        init.events.send(AppEvent::RefreshCache);
        init.events.send(AppEvent::HelpDesk);
        return init;
    }
}

impl App {
    pub fn config_check_file_exists() -> bool {
        if let Some(cfg_dir) = dirs::config_dir() {
            let exists = cfg_dir.join("rrplay").join("config.txt");
            if !exists.is_file() {
                let mut config_file = cfg_dir.join("rrplay");
                std::fs::create_dir_all(config_file.clone()).unwrap();
                config_file = cfg_dir.join("rrplay/config.txt");
                std::fs::write(config_file, "").unwrap();
                true
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn check_config_validity() -> Option<Vec<(String, bool)>> {
        if App::config_check_file_exists() {
            if let Some(cfg_dir) = dirs::config_dir() {
                let config_file = cfg_dir.join("rrplay").join("config.txt");
                let file_contents: Option<String> = match std::fs::read_to_string(config_file) {
                    Ok(content) => {
                        if content.is_empty() {
                            None
                        } else {
                            Some(content)
                        }
                    }
                    Err(_) => None,
                };
                match file_contents {
                    Some(c) => {
                        let paths = c.split("\n");
                        let mut output: Vec<(String, bool)> = Vec::new();
                        for path in paths {
                            let ap = path.trim().to_string();

                            if Path::new(&ap).exists() {
                                output.push((ap, true));
                            } else {
                                output.push((ap, false));
                            }
                        }
                        return Some(output);
                    }
                    None => {
                        return None;
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    pub fn search_directories(check_sources: Option<Vec<(String, bool)>>) -> Vec<Song> {
        if let Some(sources) = check_sources {
            let mut out = Vec::new();
            let source_filter: Vec<(String, bool)> = sources.into_iter().filter(|x| x.1).collect();
            for source in source_filter {
                let file_types = [
                    "flac", "m4a", "mp3", "wav", "ogg", "opus", "m4p", "aiff", "3gp", "aac",
                ];

                for entry in WalkDir::new(source.0).into_iter().filter_map(|e| e.ok()) {
                    if let Some(ext) = entry.path().extension() {
                        if file_types.contains(&ext.to_str().unwrap_or("No ext")) {
                            if let Some(filename) =
                                Path::new(entry.clone().path().as_os_str()).file_name()
                            {
                                let tagged_file = Probe::open(entry.clone().path())
                                    .expect("ERROR: Bad path provided!")
                                    .read()
                                    .expect("ERROR: Failed to read file!");
                                let duration: Duration = tagged_file.properties().duration();
                                match tagged_file.primary_tag() {
                                    Some(primary_tag) => {
                                        let new_entry: Song = Song {
                                            file_path: entry.path().display().to_string(),
                                            file_name: filename.to_str().unwrap().to_string(),
                                            file_type: ext.to_str().unwrap().to_string(),
                                            is_valid: true,
                                            title: primary_tag
                                                .title()
                                                .as_deref()
                                                .unwrap_or("N/A")
                                                .to_string(),
                                            album: {
                                                primary_tag
                                                    .album()
                                                    .as_deref()
                                                    .unwrap_or("N/A")
                                                    .to_string()
                                            },
                                            artist: primary_tag
                                                .artist()
                                                .as_deref()
                                                .unwrap_or("N/A")
                                                .to_string(),
                                            duration,
                                            genre: primary_tag
                                                .genre()
                                                .as_deref()
                                                .unwrap_or("N/A")
                                                .to_string(),
                                        };
                                        out.push(new_entry);
                                    }

                                    // If the "primary" tag doesn't exist, we just grab the
                                    // first tag we can find. Realistically, a tag reader would likely
                                    // iterate through the tags to find a suitable one.
                                    _ => {
                                        let new_entry: Song = Song {
                                            file_path: entry.path().display().to_string(),
                                            file_name: filename.to_str().unwrap().to_string(),
                                            file_type: ext.to_str().unwrap().to_string(),
                                            is_valid: true,
                                            title: filename.to_str().unwrap().to_string(),
                                            artist: "N/A".to_string(),
                                            album: "N/A".to_string(),
                                            duration,
                                            genre: "N/A".to_string(),
                                        };
                                        out.push(new_entry);
                                    }
                                };
                            }
                        }
                    }
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
            let size = terminal.size().unwrap();
            self.terminal_size = (size.width, size.height);
            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event)?
                    }
                }
                Event::App(app_event) => match app_event {
                    AppEvent::Quit => self.quit(),
                    AppEvent::Search => {
                        self.mode = Mode::Search;
                    }
                    AppEvent::Select => {
                        if !self.search_results.is_empty() && self.mode == Mode::Search {
                            self.mode = Mode::Select;
                            self.select_index = self.search_results.len() - 1;
                        }
                    }
                    AppEvent::Escape => {
                        if self.popup_manual.is_some() {
                            self.popup_manual = None;
                        } else {
                            if self.mode == Mode::Select {
                                self.mode = Mode::Search;
                            } else {
                                self.mode = Mode::Sitback;
                            }
                        }
                    }
                    AppEvent::MoveUp => {
                        if (self.select_index as i32) < (self.search_results.len() as i32 - 1)
                            && !self.search_results.is_empty()
                        {
                            self.select_index += 1;
                        }
                    }
                    AppEvent::MoveDown => {
                        if self.select_index > 0 && !self.search_results.is_empty() {
                            self.select_index -= 1;
                        }
                    }
                    AppEvent::HelpDesk => {
                        self.popup_manual = Some(PopupManual {
                            title: "Help desk".to_string(),
                            border_color: {
                                if self.sources.is_some() {
                                    Color::White
                                } else {
                                    Color::Red
                                }
                            },
                            bottom_hint: PopupManual::default().bottom_hint,
                            message: {
                                if let Some(sources_ok) = self.sources.clone() {
                                    let mut out: Vec<(String, Color)> = Vec::new();
                                    out.push(("Sources:".to_string(), Color::White));
                                    for s in sources_ok {
                                        if s.1 {
                                            out.push((s.0, Color::Green));
                                        } else {
                                            out.push((s.0, Color::Red));
                                        }
                                    }
                                    out
                                } else {
                                    let mut ret = vec![
                                        "No sources found...".to_string(),
                                        "Add some!".to_string(),
                                        "File location:".to_string(),
                                    ];
                                    if let Some(cfg_dir) = dirs::config_dir() {
                                        ret.push(
                                            cfg_dir
                                                .join("rrplay")
                                                .join("config.txt")
                                                .display()
                                                .to_string(),
                                        );
                                    }
                                    ret.clone()
                                        .iter()
                                        .map(|t| (t.to_string(), Color::White))
                                        .collect()
                                }
                            },
                        });
                    }
                    AppEvent::AddSingle => {
                        if !self.search_results.is_empty() {
                            let index = self.search_results.len() - 1 - self.select_index;

                            let song = &self.search_results[index];

                            if Path::new(&song.file_path).is_file() {
                                let file = std::fs::File::open(song.clone().file_path).unwrap();

                                if let Ok(decoder) = rodio::Decoder::new(BufReader::new(file)) {
                                    self.sink.append(decoder);
                                    self.queue.push(song.clone());
                                } else {
                                    self.search_results[index].is_valid = false;
                                }
                            } else {
                                self.search_results[index].is_valid = false;
                            }
                        }
                    }
                    AppEvent::AddAlbum => {
                        if !self.search_results.is_empty() {
                            let index = self.search_results.len() - 1 - self.select_index;
                            let song = self.search_results[index].clone();
                            let album_name = song.album;

                            for song in &mut self.search_cache {
                                if song.album == album_name {
                                    if Path::new(&song.file_path).is_file() {
                                        let file =
                                            std::fs::File::open(song.clone().file_path).unwrap();
                                        if let Ok(decoder) =
                                            rodio::Decoder::new(BufReader::new(file))
                                        {
                                            self.sink.append(decoder);
                                            self.queue.push(song.clone());
                                        } else {
                                            song.is_valid = false;
                                        }
                                    } else {
                                        song.is_valid = false;
                                    }
                                }
                            }
                        }

                        self.popup_notif.push({
                            PopupNotif {
                                message: vec![(
                                    "Added album to the queue".to_string(),
                                    Color::White,
                                )],
                                border_color: Color::Green,
                                duration_ticks: Some(NOTIF_DURATION),
                                title: "".to_string(),
                                index: 1,
                            }
                        });
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
                        if !self.queue.is_empty() && pos + SEEK_CHANGE < self.queue[0].duration {
                            let _ = self.sink.try_seek(pos + SEEK_CHANGE);
                        }
                    }
                    AppEvent::MoveBackward => {
                        let pos = self.sink.get_pos();

                        if !self.queue.is_empty() {
                            if pos.as_secs() as i32 - SEEK_CHANGE.as_secs() as i32 > 0 {
                                let _ = self.sink.try_seek(pos - SEEK_CHANGE);
                            } else {
                                let _ = self.sink.try_seek(Duration::from_millis(0));
                            }
                        }
                    }
                    AppEvent::RefreshResults => {
                        if !self.query.is_empty() {
                            let matcher = SkimMatcherV2::default();
                            let mut entries_with_score: Vec<(Song, i64)> = Vec::new();
                            for entry in self.search_cache.clone() {
                                if let Some(score) = matcher.fuzzy_match(
                                    {
                                        match self.search_by {
                                            SearchBy::FilePath => entry.file_path.as_str(),
                                            SearchBy::Title => entry.title.as_str(),
                                            SearchBy::Artist => entry.artist.as_str(),
                                            SearchBy::Album => entry.album.as_str(),
                                            SearchBy::Genre => entry.genre.as_str(),
                                        }
                                    },
                                    self.query.as_str(),
                                ) {
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
                            self.select_index = (self.search_results.len() as i32 - 1_i32) as usize;
                        }
                    }
                    AppEvent::RefreshCache => {
                        let src = self.sources.clone();
                        self.search_cache = App::search_directories(src);
                    }
                    AppEvent::InitPopup => {
                        self.popup_manual = Some(PopupManual {
                            title: "".to_string(),
                            border_color: Color::Blue,
                            bottom_hint: PopupManual::default().bottom_hint,
                            message: vec![(
                                "Scanning your directories...".to_string(),
                                Color::White,
                            )],
                        });
                    }
                },
            }
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        if self.popup_manual.is_some() && key_event.kind == KeyEventKind::Press {
            match key_event.code {
                KeyCode::Esc => self.events.send(AppEvent::Escape),
                _ => {}
            }
        } else if self.popup_manual.is_none() && key_event.kind == KeyEventKind::Press {
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
                    // Select only
                    KeyCode::Char('j') if self.mode == Mode::Select => {
                        self.events.send(AppEvent::MoveDown)
                    }
                    KeyCode::Char('k') if self.mode == Mode::Select => {
                        self.events.send(AppEvent::MoveUp)
                    }
                    KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                    KeyCode::Char('p') => {
                        if self.mode == Mode::Sitback {
                            if self.sink.is_paused() {
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
                            self.events.send(AppEvent::Resume)
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
                    KeyCode::Char(':') => {
                        self.events.send(AppEvent::HelpDesk);
                    }
                    KeyCode::Char('1') => {
                        self.search_by = SearchBy::FilePath;
                        self.events.send(AppEvent::RefreshResults);
                    }
                    KeyCode::Char('2') => {
                        self.search_by = SearchBy::Title;
                        self.events.send(AppEvent::RefreshResults);
                    }
                    KeyCode::Char('3') => {
                        self.search_by = SearchBy::Artist;
                        self.events.send(AppEvent::RefreshResults);
                    }
                    KeyCode::Char('4') => {
                        self.search_by = SearchBy::Album;
                        self.events.send(AppEvent::RefreshResults);
                    }
                    KeyCode::Char('5') => {
                        self.search_by = SearchBy::Genre;
                        self.events.send(AppEvent::RefreshResults);
                    }

                    KeyCode::Char('c' | 'C') => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            self.events.send(AppEvent::Quit)
                        } else {
                            self.events.send(AppEvent::ClearQueue);

                            self.popup_notif.push({
                                PopupNotif {
                                    message: vec![("Cleared the queue".to_string(), Color::White)],
                                    border_color: Color::Yellow,
                                    duration_ticks: Some(NOTIF_DURATION),
                                    title: "".to_string(),
                                    index: self.popup_notif.len() + 1,
                                }
                            });
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
                        self.query.push(' ');
                        self.events.send(AppEvent::RefreshResults);
                    }
                    KeyCode::Backspace => {
                        if KeyModifiers::ALT == key_event.modifiers {
                            self.query.clear();
                            self.search_results.clear();
                        } else {
                            self.query.pop();
                            if !self.query.is_empty() {
                                self.events.send(AppEvent::RefreshResults);
                            } else {
                                self.search_results.clear();
                            }
                        }
                    }
                    KeyCode::Esc => {}
                    _ => {
                        self.query.push_str(&key_event.code.to_string());
                        self.events.send(AppEvent::RefreshResults);
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
        if !self.popup_notif.is_empty() {}
        let mut n_idx = Vec::new();
        for (i, notif) in self.popup_notif.iter_mut().enumerate() {
            if let Some(t) = notif.duration_ticks {
                if t == 1 {
                    notif.duration_ticks = None;
                } else {
                    notif.duration_ticks = Some(t - 1);
                }
            } else {
                n_idx.push(i);
            }
        }
        for i in n_idx {
            self.popup_notif.remove(i);
        }
        for (i, notif) in self.popup_notif.iter_mut().enumerate() {
            notif.index = i + 1;
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
