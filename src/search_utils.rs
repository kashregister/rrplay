use crossterm::terminal;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use std::{path::PathBuf, usize};
use walkdir::WalkDir;

use crate::term_utils::*;

#[derive(Clone)]
pub struct SongEntry {
    pub file: PathBuf,
    pub score: i64,
}

fn sort(mut entries: Vec<SongEntry>) -> Vec<SongEntry> {
    entries.sort_by(|a, b| b.score.cmp(&a.score));
    entries
}

pub fn walkdir(query: &mut String, path: String) -> Vec<SongEntry> {
    if query.starts_with('/') {
        query.remove(0);
    }
    t_clear_all();

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
    song_entries.retain(|entry| entry.score > 0);
    song_entries = sort(song_entries.clone());
    song_entries.reverse();
    song_entries
}

pub fn song_entries_print(s_e_vec: &[SongEntry], index: usize) {
    let t_sz = match terminal::size() {
        Ok(s) => s,
        Err(_) => return,
    };

    t_clear_all();
    for (i, entry) in s_e_vec.iter().enumerate() {
        t_mv_sol();
        t_flush();
        if entry.score > 0 {
            match entry.file.file_name() {
                Some(name) => {
                    let name = name.to_string_lossy();
                    let max_len = t_sz.0 as usize - 2;
                    let prnt = if name.len() > max_len {
                        let mut end_index = max_len - 2;
                        while !name.is_char_boundary(end_index) {
                            end_index -= 1;
                        }
                        &name[..end_index]
                    } else {
                        &name
                    };
                    if i as i32 == s_e_vec.len() as i32 - index as i32 + 1 {
                        t_bg_gray();
                        t_flush();
                        print!("* {}", prnt);
                        t_bg_reset();
                        println!();
                    } else {
                        println!("{prnt}");
                    }
                }

                None => continue,
            }
        }
    }
}
pub fn get_song(s_e_vec: &[SongEntry], index: usize) -> Result<SongEntry, bool> {
    let i = s_e_vec.len() as i32 - index as i32 + 1;
    if let Some(song) = s_e_vec.get(i as usize) {
        Ok(song.clone())
    } else {
        Err(false)
    }
}
