use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use gpui_kit::Global;

use crate::model::Track;

#[derive(Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Debug)]
pub enum RepeatMode {
    #[default]
    Off,
    Track,
    Context,
}

#[derive(Clone, Default)]
pub struct QueueManager {
    pub history: VecDeque<Track>,
    pub user_queue: VecDeque<Track>,
    pub context: Vec<Track>,
    pub context_index: Option<usize>,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
}
impl QueueManager {
    pub fn play_from_context(&mut self, tracks: Vec<Track>, index: usize) -> Option<Track> {
        self.context = tracks;
        self.context_index = Some(index);
        self.context.get(index).cloned()
    }

    pub fn advance_next(&mut self, current: Option<Track>) -> Option<Track> {
        if let Some(track) = current {
            self.history.push_back(track);
            if self.history.len() > 50 {
                self.history.pop_front();
            }
        }

        if let Some(next_user_track) = self.user_queue.pop_front() {
            return Some(next_user_track);
        }

        if let Some(ref mut idx) = self.context_index {
            let next_idx = *idx + 1;
            if next_idx < self.context.len() {
                *idx = next_idx;
                return self.context.get(*idx).cloned();
            } else if self.repeat_mode == RepeatMode::Context && !self.context.is_empty() {
                *idx = 0;
                return self.context.first().cloned();
            }
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
                self.user_queue.push_front(curr);
            }
            Some(prev_track)
        } else {
            None
        }
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
