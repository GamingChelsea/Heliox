use std::path::{Path, PathBuf};

use gpui_kit::*;
use lofty::config::ParseOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rayon::prelude::*;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

use crate::model::{Config, CurrentTrackList, Track, TrackListSource};

pub fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| matches!(ext, "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a"))
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
        path: path.to_path_buf(),
    })
}

pub fn scan_music_dir(music_dir: PathBuf) -> Vec<Track> {
    let options = ParseOptions::new()
        .read_properties(true)
        .read_cover_art(false);

    let paths: Vec<PathBuf> = WalkDir::new(music_dir)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| is_audio_file(p))
        .collect();

    paths
        .par_iter()
        .filter_map(|path| parse_track(path, options))
        .collect()
}

pub fn load_local_library<T: 'static>(cx: &mut Context<T>) {
    let music_dir = cx.global::<Config>().music_dir.clone();

    if cx.global::<CurrentTrackList>().source == Some(TrackListSource::LocalLibrary) {
        return;
    }

    cx.spawn(async move |_this, cx| {
        let tracks = cx
            .background_executor()
            .spawn(async move { scan_music_dir(music_dir) })
            .await;

        let _ = cx.update(|cx| {
            cx.update_global::<CurrentTrackList, _>(|list, _cx| {
                list.source = Some(TrackListSource::LocalLibrary);
                list.tracks = Some(tracks);
            });
        });
    })
    .detach();
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
