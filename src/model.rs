use gpui_kit::{Global, SharedString, private::serde::Deserialize};
use serde::Serialize;
use std::path::PathBuf;

use crate::audio::AudioCommand;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub theme_name: SharedString,
    #[serde(default)]
    pub music_dir: PathBuf,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: SharedString::from("Default Dark"),
            music_dir: dirs::audio_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_default(),
        }
    }
}

impl Global for Config {}

pub struct AudioPlayer(pub std::sync::mpsc::Sender<AudioCommand>);
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

#[derive(Clone)]
pub struct CurrentTrackList {
    pub tracks: Option<Vec<Track>>,
    pub selected_id: Option<u64>,
}
impl CurrentTrackList {
    pub fn new() -> Self {
        Self {
            tracks: None,
            selected_id: None,
        }
    }
}
impl Global for CurrentTrackList {}
