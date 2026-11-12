use crate::{audio::AudioCommand, library::clean_title, queue::RepeatMode};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicU64},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme_name: String,
    #[serde(default)]
    pub music_dir: PathBuf,
    #[serde(default = "default_true")]
    pub shorten_titles: bool,
    #[serde(default = "default_volume")]
    pub volume: f32,
    #[serde(default)]
    pub discord_rpc: bool,
    #[serde(default)]
    pub repeat_mode: RepeatMode,
    #[serde(default = "default_corner_radius")]
    pub corner_radius: f32,
}

fn default_theme() -> String {
    "Default Dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_volume() -> f32 {
    0.50
}

fn default_corner_radius() -> f32 {
    8.0
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: default_theme(),
            music_dir: dirs::audio_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_default(),
            shorten_titles: true,
            volume: 0.50,
            discord_rpc: false,
            repeat_mode: RepeatMode::default(),
            corner_radius: default_corner_radius(),
        }
    }
}

#[derive(Clone)]
pub struct AudioPlayer {
    pub sender: Option<std::sync::mpsc::Sender<AudioCommand>>,
    pub current_track: Option<Track>,
    pub is_playing: bool,
    pub progress_ms: Arc<AtomicU64>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self {
            sender: None,
            current_track: None,
            is_playing: false,
            progress_ms: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl AudioPlayer {
    pub fn new(sender: std::sync::mpsc::Sender<AudioCommand>, progress_ms: Arc<AtomicU64>) -> Self {
        Self {
            sender: Some(sender),
            current_track: None,
            is_playing: false,
            progress_ms,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TrackSource {
    Local(PathBuf),
    Stream {
        provider: String,
        id: String,
        direct_url: Option<String>,
        hint_extension: String,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub id: u64,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: u32,
    pub source: TrackSource,
}

impl Track {
    pub fn local_path(&self) -> Option<&Path> {
        match &self.source {
            TrackSource::Local(path) => Some(path.as_path()),
            TrackSource::Stream { .. } => None,
        }
    }

    pub fn display_title(&self, shorten: bool) -> String {
        if shorten {
            clean_title(&self.title, &self.artist)
        } else {
            self.title.clone()
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TrackListSource {
    LocalLibrary,
    Playlist(u64),
    Album(u64),
}

#[derive(Clone, Default, Debug)]
pub struct CurrentTrackList {
    pub source: Option<TrackListSource>,
    pub tracks: Option<Vec<Track>>,
    pub selected_id: Option<u64>,
    pub load_error: Option<String>,
}

impl CurrentTrackList {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone)]
pub enum UiMediaEvent {
    Toggle,
    Play,
    Pause,
    Stop,
    Quit,
    SetVolume(f64),
    Next,
    Previous,
    TrackEnded,
}

pub use crate::discord::{DiscordCommand, DiscordRpc};
