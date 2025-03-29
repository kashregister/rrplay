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

#[derive(Clone)]
struct SongEntry {
    file: PathBuf,
    score: i64,
}

struct Player {
    current_song: SongEntry,
    sink: Sink,
}

impl Player {
    fn init() -> Player {
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let entry = SongEntry {
            file: PathBuf::new(),
            score: 0,
        };
        return Player {
            current_song: entry,
            sink: sink,
        };
    }
    fn play_song(&mut self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let file = BufReader::new(File::open(&self.current_song.file).unwrap());
        let source = Decoder::new(file).unwrap();

        sink.append(source);
        while (!sink.empty()) {
            println!("waiting for audio to end");
            sleep(Duration::new(1, 0));
        }
        sink.sleep_until_end();
    }
}

// clear the entire terminal
fn t_clear_all() {
    io::stdout()
        .execute(terminal::Clear(ClearType::All))
        .unwrap();
}
// clear the whole line
fn t_clear_line() {
    io::stdout()
        .execute(terminal::Clear(ClearType::CurrentLine))
        .unwrap();
}
// move to the very top (top left)
fn t_mv_start() {
    io::stdout().execute(MoveTo(0, 0)).unwrap();
}
// move to the very bottom
fn t_mv_end() {
    let t_sz = terminal::size().unwrap();
    io::stdout().execute(MoveTo(0, t_sz.1)).unwrap();
}
// move to the start of the current line
fn t_mv_sol() {
    io::stdout().execute(MoveToColumn(0)).unwrap();
}
// change the lines style for the song we are hovering over
fn t_bg_gray() {
    io::stdout()
        .execute(SetBackgroundColor(crossterm::style::Color::DarkGrey))
        .unwrap();
    io::stdout()
        .execute(SetAttribute(crossterm::style::Attribute::Bold))
        .unwrap();
}
// reset the terminals styling
fn t_bg_reset() {
    io::stdout().execute(ResetColor).unwrap();
}

fn t_flush() {
    io::stdout().flush().unwrap();
}

fn t_cursor_show() {
    io::stdout().execute(cursor::Show).unwrap();
}
fn t_cursor_hide() {
    io::stdout().execute(cursor::Hide).unwrap();
}

fn run_cmd(cmd: &String) -> Result<&'static str, &'static str> {
    if cmd.eq(":q") {
        Ok("exit")
    } else if cmd.eq("source") {
        Ok(":source")
    } else {
        Err("Wrong syntax")
    }
}

fn sort_entries(mut song_entries: Vec<SongEntry>) -> Vec<SongEntry> {
    for (i, item) in song_entries.clone().iter().enumerate() {
        if let Some(next_item) = song_entries.get(i + 1) {
            if item.score > next_item.score {
                song_entries.swap(i, i + 1);
            }
        }
    }
    song_entries.reverse();
    song_entries
}

fn bubble_sort(mut vec: Vec<SongEntry>) -> Vec<SongEntry> {
    let mut n = vec.len();
    loop {
        let mut swapped = false;

        for i in 0..n - 1 {
            if vec[i].score > vec[i + 1].score {
                vec.swap(i, i + 1);
                swapped = true;
            }
        }
        if !swapped {
            break;
        }
        n -= 1;
    }
    vec
}

fn walkdir(query: &mut String) -> Vec<SongEntry> {
    if query.starts_with('/') {
        query.remove(0);
    }
    t_clear_all();
    let path = "/mnt/disk_new/Music Library/";
    let file_types = [
        "flac", "m4a", "mp3", "wav", "ogg", "opus", "m4p", "aiff", "3gp", "aac",
    ];
    let mut song_entries = Vec::new();
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if let Some(ext) = entry.path().extension() {
            if file_types.contains(&ext.to_str().unwrap()) {
                let tmp: SongEntry = SongEntry {
                    file: (entry.path().to_owned()),
                    score: (0),
                };
                song_entries.push(tmp);
            }
        }
    }
    let matcher = SkimMatcherV2::default();
    for entry in &mut song_entries {
        if let Some(score) = matcher.fuzzy_match(entry.file.to_str().unwrap(), query) {
            entry.score = score;
        }
    }
    song_entries = sort_entries(song_entries.clone());
    let cpy = song_entries.clone();
    song_entries = bubble_sort(cpy);
    song_entries
}

fn song_entries_print(s_e_vec: &[SongEntry], index: usize) {
    let t_sz = terminal::size().unwrap();
    for (i, entry) in s_e_vec.iter().enumerate() {
        t_mv_sol();
        io::stdout().flush().unwrap();
        if entry.score > 0 {
            let name = entry.file.file_name().unwrap().to_string_lossy();
            let prnt = if name.len() > t_sz.0 as usize - 2 {
                name.split_at(t_sz.0 as usize - 4).0
            } else {
                &name
            };
            if i == s_e_vec.len() - index + 1 {
                t_bg_gray();
                t_flush();
                print!("* {}", prnt);
                t_bg_reset();
                println!();
            } else {
                println!("{prnt}");
            }
        }
    }
}

fn get_song(s_e_vec: &[SongEntry], index: usize) -> SongEntry {
    let song = s_e_vec.get(s_e_vec.len() - index + 1).unwrap();
    return song.clone();
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

                    player.current_song = song;
                    player.play_song();

                    track_mode = false;
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

    t_bg_reset();
}
