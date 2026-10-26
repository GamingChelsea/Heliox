use gpui_kit::{Global, SharedString, private::serde::Deserialize};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
};
use crate::{audio::AudioCommand, library::clean_title, queue::RepeatMode};

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
    #[serde(default)]
    pub discord_rpc: bool,
    #[serde(default)]
    pub repeat_mode: RepeatMode,
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
            discord_rpc: false,
            repeat_mode: RepeatMode::default(),
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
}

impl Global for AudioPlayer {}

#[derive(Clone, Debug)]
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

pub struct MediaControl(pub souvlaki::MediaControls);
impl Global for MediaControl {}

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

impl MediaControl {
    pub fn update_track(&mut self, track: &Track) {
        let _ = self.0.set_metadata(souvlaki::MediaMetadata {
            title: Some(&track.title),
            artist: Some(&track.artist),
            album: Some(&track.album),
            duration: Some(std::time::Duration::from_secs(track.duration_secs as u64)),
            cover_url: None,
        });
        let _ = self
            .0
            .set_playback(souvlaki::MediaPlayback::Playing { progress: None });
    }

    pub fn set_playback(&mut self, is_playing: bool) {
        let playback = if is_playing {
            souvlaki::MediaPlayback::Playing { progress: None }
        } else {
            souvlaki::MediaPlayback::Paused { progress: None }
        };
        let _ = self.0.set_playback(playback);
    }

    pub fn set_volume(&mut self, volume: f64) {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            let _ = self.0.set_volume(volume);
        }
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            let _ = volume;
        }
    }
}

pub use crate::discord::{DiscordCommand, DiscordRpc};

