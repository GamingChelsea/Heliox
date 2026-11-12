use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, Config, CurrentTrackList, DiscordRpc, Track, TrackSource},
    providers::youtube,
    queue::{QueueManager, RepeatMode},
};
use nsh_core::prelude::*;

pub fn play_track(track: Track) {
    let track_id = track.id;
    global::<CurrentTrackList>().update(|list| {
        list.selected_id = Some(track_id);
    });

    let sender = global::<AudioPlayer>().get_untracked().sender.clone();
    if let Some(ref sender) = sender {
        match &track.source {
            TrackSource::Local(path) => {
                let _ = sender.send(AudioCommand::Play(path.clone()));
            }
            TrackSource::Stream {
                direct_url: Some(url),
                hint_extension,
                ..
            } => {
                let _ = sender.send(AudioCommand::PlayStream {
                    url: url.clone(),
                    hint_extension: hint_extension.clone(),
                    content_length: None,
                });
            }
            TrackSource::Stream {
                direct_url: None,
                id,
                provider,
                hint_extension,
            } => {
                let id_clone = id.clone();
                let provider_clone = provider.clone();
                let ext_clone = hint_extension.clone();
                let sender_clone = sender.clone();

                std::thread::spawn(move || {
                    if provider_clone == "youtube" {
                        match youtube::get_stream_url(&id_clone) {
                            Ok((url, content_length)) => {
                                let _ = sender_clone.send(AudioCommand::PlayStream {
                                    url,
                                    hint_extension: ext_clone,
                                    content_length,
                                });
                            }
                            Err(err) => {
                                tracing::error!(?err, "Failed to resolve YouTube stream URL");
                            }
                        }
                    }
                });
            }
        }
    }

    global::<AudioPlayer>().update(|player| {
        player.current_track = Some(track.clone());
        player.is_playing = true;
    });

    crate::update_media_controls_track(&track);

    if global::<Config>().get_untracked().discord_rpc {
        global::<DiscordRpc>().get_untracked().update_track(&track);
    }
}

pub fn toggle_play() {
    let is_playing = global::<AudioPlayer>().get_untracked().is_playing;
    if is_playing {
        pause();
    } else {
        resume();
    }
}

pub fn resume() {
    let mut current_track = None;
    global::<AudioPlayer>().update(|player| {
        if let Some(ref s) = player.sender {
            let _ = s.send(AudioCommand::Resume);
        }
        player.is_playing = true;
        current_track = player.current_track.clone();
    });

    crate::update_media_controls_playback(true);

    if global::<Config>().get_untracked().discord_rpc {
        if let Some(track) = current_track {
            global::<DiscordRpc>().get_untracked().update_track(&track);
        }
    }
}

pub fn pause() {
    let mut current_track = None;
    global::<AudioPlayer>().update(|player| {
        if let Some(ref s) = player.sender {
            let _ = s.send(AudioCommand::Pause);
        }
        player.is_playing = false;
        current_track = player.current_track.clone();
    });

    crate::update_media_controls_playback(false);

    if global::<Config>().get_untracked().discord_rpc {
        if let Some(track) = current_track {
            global::<DiscordRpc>().get_untracked().set_paused(&track);
        }
    }
}

pub fn stop() {
    global::<AudioPlayer>().update(|player| {
        if let Some(ref s) = player.sender {
            let _ = s.send(AudioCommand::Stop);
        }
        player.is_playing = false;
        player.current_track = None;
    });

    crate::update_media_controls_playback(false);

    if global::<Config>().get_untracked().discord_rpc {
        global::<DiscordRpc>().get_untracked().set_idle();
    }
}

pub fn play_next() {
    let current_track = global::<AudioPlayer>()
        .get_untracked()
        .current_track
        .clone();
    let mut next_track = None;
    global::<QueueManager>().update(|qm| {
        next_track = qm.advance_next(current_track);
    });

    if let Some(track) = next_track {
        play_track(track);
    } else {
        stop();
    }
}

pub fn play_prev() {
    let progress_ms = global::<AudioPlayer>()
        .get_untracked()
        .progress_ms
        .load(std::sync::atomic::Ordering::Relaxed);

    // If more than 3 seconds in, restart the track
    if progress_ms > 3000 {
        if let Some(ref s) = global::<AudioPlayer>().get_untracked().sender {
            let _ = s.send(AudioCommand::Seek(0.0));
        }
        return;
    }

    let mut prev_track = None;
    global::<QueueManager>().update(|qm| {
        prev_track = qm.previous();
    });
    if let Some(track) = prev_track {
        play_track(track);
    }
}

pub fn toggle_repeat() {
    let mut new_mode = RepeatMode::Off;
    global::<QueueManager>().update(|qm| {
        new_mode = qm.cycle_repeat();
    });
    global::<Config>().update(|c| {
        c.repeat_mode = new_mode;
    });
    crate::save_config(&global::<Config>().get_untracked());
}

pub fn toggle_shuffle() {
    global::<QueueManager>().update(|qm| {
        qm.toggle_shuffle();
    });
}

pub fn play_context(tracks: Vec<Track>, index: usize) {
    let mut track = None;
    global::<QueueManager>().update(|qm| {
        track = qm.play_from_context(tracks, index);
    });
    if let Some(track) = track {
        play_track(track);
    }
}

pub fn on_track_end() {
    let repeat_mode = global::<QueueManager>().get_untracked().repeat_mode;
    if repeat_mode == RepeatMode::Track {
        if let Some(track) = global::<AudioPlayer>()
            .get_untracked()
            .current_track
            .clone()
        {
            play_track(track);
            return;
        }
    }
    play_next();
}
