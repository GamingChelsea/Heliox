use gpui_kit::{Global, SharedString, private::serde::Deserialize};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
};

use crate::{audio::AudioCommand, library::clean_title};

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    pub theme_name: SharedString,
    #[serde(default)]
    pub music_dir: PathBuf,
    #[serde(default)]
    pub shorten_titles: bool,
    #[serde(default)]
    pub volume: f32,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: SharedString::from("Default Dark"),
            music_dir: dirs::audio_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_default(),
            shorten_titles: true,
            volume: 0.50,
        }
    }
}

impl Global for Config {}

pub struct AudioPlayer {
    pub sender: std::sync::mpsc::Sender<AudioCommand>,
    pub current_track: Option<Track>,
    pub is_playing: bool,
    pub progress_ms: Arc<AtomicU64>,
}
impl AudioPlayer {
    pub fn new(sender: std::sync::mpsc::Sender<AudioCommand>, progress_ms: Arc<AtomicU64>) -> Self {
        Self {
            sender,
            current_track: None,
            is_playing: false,
            progress_ms,
        }
    }

    pub fn play(&mut self, track: Track) {
        let _ = self.sender.send(AudioCommand::Play(track.path.clone()));
        self.current_track = Some(track);
        self.is_playing = true;
    }

    pub fn toggle_play(&mut self) {
        let _ = self.sender.send(AudioCommand::ResumePause);
        self.is_playing = !self.is_playing;
    }

    pub fn stop(&mut self) {
        let _ = self.sender.send(AudioCommand::Stop);
        self.is_playing = false;
    }
}
impl Global for AudioPlayer {}

#[derive(Clone)]
pub struct Track {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u32,
    pub path: std::path::PathBuf,
}
impl Track {
    pub fn display_title(&self, shorten: bool) -> String {
        if shorten {
            clean_title(&self.title, &self.artist)
        } else {
            self.title.clone()
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum TrackListSource {
    LocalLibrary,
    Playlist(u64),
    Album(u64),
}

#[derive(Clone)]
pub struct CurrentTrackList {
    pub source: Option<TrackListSource>,
    pub tracks: Option<Vec<Track>>,
    pub selected_id: Option<u64>,
}
impl CurrentTrackList {
    pub fn new() -> Self {
        Self {
            source: None,
            tracks: None,
            selected_id: None,
        }
    }
}
impl Global for CurrentTrackList {}
