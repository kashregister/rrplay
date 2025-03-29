use crossterm::terminal;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::io::{self, stdout};
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::term_utils::*;

#[derive(Clone)]
pub struct SongEntry {
    pub file: PathBuf,
    pub score: i64,
}

pub fn sort_entries(mut song_entries: Vec<SongEntry>) -> Vec<SongEntry> {
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

pub fn bubble_sort(mut vec: Vec<SongEntry>) -> Vec<SongEntry> {
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

pub fn walkdir(query: &mut String) -> Vec<SongEntry> {
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

pub fn song_entries_print(s_e_vec: &[SongEntry], index: usize) {
    let t_sz = terminal::size().unwrap();
    t_clear_all();
    for (i, entry) in s_e_vec.iter().enumerate() {
        t_mv_sol();
        t_flush();
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

pub fn get_song(s_e_vec: &[SongEntry], index: usize) -> SongEntry {
    let song = s_e_vec.get(s_e_vec.len() - index + 1).unwrap();
    return song.clone();
}
