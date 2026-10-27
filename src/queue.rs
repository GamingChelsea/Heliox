use gpui_kit::{Global, assets::IconName};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use strum::EnumIter;

use crate::model::Track;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Debug, EnumIter)]
pub enum RepeatMode {
    #[default]
    Off,
    Context,
    Track,
}
impl RepeatMode {
    pub fn title(&self) -> String {
        match &self {
            Self::Off => String::from("Off"),
            Self::Context => String::from("Original Playlist"),
            Self::Track => String::from("Current Track"),
        }
    }
    pub fn icon(&self) -> IconName {
        match &self {
            Self::Off => IconName::RepeatOff,
            Self::Context => IconName::Repeat,
            Self::Track => IconName::Repeat1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum QueueTab {
    #[default]
    Queue,
    History,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TrackSource {
    UserAdded,
    Context,
}

#[derive(Clone)]
pub struct QueueItem {
    pub id: u64,
    pub track: Track,
    pub source: TrackSource,
}

#[derive(Clone, Default)]
pub struct QueueManager {
    pub history: VecDeque<Track>,
    pub queue: VecDeque<QueueItem>,
    pub original_context: Vec<Track>,
    pub next_id: u64,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub active_tab: QueueTab,
}
impl QueueManager {
    pub fn play_from_context(&mut self, tracks: Vec<Track>, index: usize) -> Option<Track> {
        self.original_context = tracks.clone();
        self.queue.clear();

        for track in tracks.into_iter().skip(index + 1) {
            self.next_id += 1;
            self.queue.push_back(QueueItem {
                id: self.next_id,
                track,
                source: TrackSource::Context,
            });
        }

        self.original_context.get(index).cloned()
    }

    pub fn advance_next(&mut self, current: Option<Track>) -> Option<Track> {
        if let Some(track) = current {
            self.history.push_back(track);
            if self.history.len() > 50 {
                self.history.pop_front();
            }
        }

        if let Some(item) = self.queue.pop_front() {
            return Some(item.track);
        }

        if self.repeat_mode == RepeatMode::Context && !self.original_context.is_empty() {
            for track in self.original_context.iter().skip(1).cloned() {
                self.next_id += 1;
                self.queue.push_back(QueueItem {
                    id: self.next_id,
                    track,
                    source: TrackSource::Context,
                });
            }
            return self.original_context.first().cloned();
        }

        None
    }

    pub fn advance_auto(&mut self, current: Option<Track>) -> Option<Track> {
        if self.repeat_mode == RepeatMode::Track {
            return current;
        }

        self.advance_next(current)
    }

    pub fn advance_previous(&mut self, current: Option<Track>) -> Option<Track> {
        if let Some(prev_track) = self.history.pop_back() {
            if let Some(curr) = current {
                self.next_id += 1;
                self.queue.push_front(QueueItem {
                    id: self.next_id,
                    track: curr,
                    source: TrackSource::UserAdded,
                });
            }
            Some(prev_track)
        } else {
            None
        }
    }

    pub fn enqueue_next(&mut self, track: Track) {
        self.next_id += 1;
        self.queue.push_front(QueueItem {
            id: self.next_id,
            track,
            source: TrackSource::UserAdded,
        });
    }

    pub fn enqueue_last(&mut self, track: Track) {
        self.next_id += 1;
        self.queue.push_back(QueueItem {
            id: self.next_id,
            track,
            source: TrackSource::UserAdded,
        });
    }

    pub fn insert(&mut self, index: usize, track: Track) {
        self.next_id += 1;
        let pos = index.min(self.queue.len());
        self.queue.insert(
            pos,
            QueueItem {
                id: self.next_id,
                track,
                source: TrackSource::UserAdded,
            },
        );
    }

    pub fn remove_at(&mut self, index: usize) -> Option<QueueItem> {
        self.queue.remove(index)
    }

    pub fn move_item(&mut self, from: usize, to: usize) {
        if from < self.queue.len() && to < self.queue.len() && from != to {
            if let Some(item) = self.queue.remove(from) {
                self.queue.insert(to, item);
            }
        }
    }

    pub fn play_queue_at(&mut self, index: usize, current: Option<Track>) -> Option<Track> {
        if let Some(track) = current {
            self.history.push_back(track);
        }
        for _ in 0..index {
            if let Some(skipped) = self.queue.pop_front() {
                self.history.push_back(skipped.track);
            }
        }
        self.queue.pop_front().map(|item| item.track)
    }

    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.repeat_mode = match self.repeat_mode {
            RepeatMode::Off => RepeatMode::Context,
            RepeatMode::Context => RepeatMode::Track,
            RepeatMode::Track => RepeatMode::Off,
        };
        self.repeat_mode
    }
}

impl Global for QueueManager {}
