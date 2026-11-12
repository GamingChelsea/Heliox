use std::{
    path::{Path, PathBuf},
    sync::{Mutex, TryLockError, mpsc::Receiver},
};

use lofty::config::ParseOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use nsh_core::prelude::*;
use rayon::prelude::*;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

use crate::model::{Config, CurrentTrackList, Track, TrackListSource};

static LIBRARY_SCAN_RX: Mutex<Option<Receiver<Result<Vec<Track>, String>>>> = Mutex::new(None);

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a"
            )
        })
        .unwrap_or(false)
}

pub fn parse_track(path: &Path, options: ParseOptions) -> Option<Track> {
    let id = xxh3_64(path.to_string_lossy().as_bytes());
    let mut title = String::from("Not found");
    let mut artist = String::from("Not found");
    let mut album = String::from("Not found");
    let mut duration_secs = 1;

    if let Ok(tagged_file) = Probe::open(path).and_then(|p| p.options(options).read()) {
        let properties = tagged_file.properties();
        duration_secs = properties.duration().as_secs() as u32;

        if let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            if let Some(tag_title) = tag.title() {
                title = tag_title.to_string();
            }
            if let Some(tag_artist) = tag.artist() {
                artist = tag_artist.to_string();
            }
            if let Some(tag_album) = tag.album() {
                album = tag_album.to_string();
            } else {
                album = title.clone();
            }
        }
    }

    Some(Track {
        id,
        title,
        artist,
        album,
        duration_secs,
        source: crate::model::TrackSource::Local(path.to_path_buf()),
    })
}

pub fn scan_music_dir(music_dir: PathBuf) -> Result<Vec<Track>, String> {
    if !music_dir.is_dir() {
        return Err(format!(
            "Music directory does not exist: {}",
            music_dir.display()
        ));
    }

    let options = ParseOptions::new()
        .read_properties(true)
        .read_cover_art(false);

    let paths: Vec<PathBuf> = WalkDir::new(&music_dir)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(Ok(entry)),
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not scan music directory: {error}"))?
        .into_iter()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| is_audio_file(p))
        .collect();

    Ok(paths
        .par_iter()
        .filter_map(|path| parse_track(path, options))
        .collect())
}

pub fn load_local_library() {
    let state = global::<CurrentTrackList>().get_untracked();
    if state.source == Some(TrackListSource::LocalLibrary)
        && (state.tracks.is_some() || state.load_error.is_none())
    {
        return;
    }
    load_local_library_from_config();
}

pub fn reload_local_library() {
    global::<CurrentTrackList>().update(|list| {
        list.source = None;
        list.tracks = None;
        list.load_error = None;
    });
    load_local_library_from_config();
}

fn load_local_library_from_config() {
    let music_dir = global::<Config>().get_untracked().music_dir.clone();
    global::<CurrentTrackList>().update(|list| {
        list.source = Some(TrackListSource::LocalLibrary);
        list.tracks = None;
        list.load_error = None;
    });

    let (tx, rx) = std::sync::mpsc::channel();
    *LIBRARY_SCAN_RX
        .lock()
        .unwrap_or_else(|error| error.into_inner()) = Some(rx);
    std::thread::spawn(move || {
        let tracks = scan_music_dir(music_dir);
        let _ = tx.send(tracks);
    });
}

pub fn poll_local_library() {
    let result = match LIBRARY_SCAN_RX.try_lock() {
        Ok(mut receiver) => {
            let Some(rx) = receiver.as_ref() else {
                return;
            };
            match rx.try_recv() {
                Ok(result) => {
                    receiver.take();
                    Some(result)
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    receiver.take();
                    Some(Err("The library scan stopped unexpectedly.".to_string()))
                }
            }
        }
        Err(TryLockError::WouldBlock) => return,
        Err(TryLockError::Poisoned(error)) => {
            let mut receiver = error.into_inner();
            receiver.take();
            Some(Err("The library scan state could not be read.".to_string()))
        }
    };

    if let Some(result) = result {
        global::<CurrentTrackList>().update(|list| {
            list.source = Some(TrackListSource::LocalLibrary);
            match result {
                Ok(tracks) => {
                    list.tracks = Some(tracks);
                    list.load_error = None;
                }
                Err(error) => {
                    list.tracks = Some(Vec::new());
                    list.load_error = Some(error);
                }
            }
        });
    }
}

pub fn clean_title(title: &str, artist: &str) -> String {
    let without_artist = if let Some((left, right)) = title.split_once(" - ") {
        let left_lower = left.to_lowercase();
        let artist_lower = artist.to_lowercase();

        if !artist_lower.is_empty()
            && (left_lower.contains(&artist_lower) || artist_lower.contains(&left_lower))
        {
            right
        } else {
            title
        }
    } else {
        title
    };

    let mut cleaned = String::new();
    let mut chars = without_artist.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '(' || c == '[' {
            let close_char = if c == '(' { ')' } else { ']' };
            let mut bracket_content = String::new();

            for inner in chars.by_ref() {
                if inner == close_char {
                    break;
                }
                bracket_content.push(inner);
            }

            let lower = bracket_content.to_lowercase();
            let is_junk = [
                "official",
                "video",
                "audio",
                "lyrics",
                "lyric",
                "visualizer",
                "hd",
                "hq",
                "4k",
                "music video",
            ]
            .iter()
            .any(|word| lower.contains(word));

            if !is_junk {
                cleaned.push(c);
                cleaned.push_str(&bracket_content);
                cleaned.push(close_char);
            }
        } else {
            cleaned.push(c);
        }
    }

    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        title.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audio_extensions_are_case_insensitive() {
        assert!(is_audio_file(Path::new("Track.MP3")));
        assert!(is_audio_file(Path::new("track.flac")));
        assert!(!is_audio_file(Path::new("cover.JPG")));
    }

    #[test]
    fn scanning_a_missing_directory_returns_an_error() {
        let missing =
            std::env::temp_dir().join(format!("heliox-missing-library-{}", std::process::id()));
        assert!(scan_music_dir(missing).is_err());
    }
}
