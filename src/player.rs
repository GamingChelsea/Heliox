use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, Config, CurrentTrackList, DiscordRpc, MediaControl, Track},
    queue::QueueManager,
};
use gpui_kit::*;

pub fn play_track(track: Track, cx: &mut App) {
    let track_id = track.id;
    cx.update_global::<CurrentTrackList, _>(|list, _cx| {
        list.selected_id = Some(track_id);
    });
    cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Play(track.path.clone()));
        player.current_track = Some(track.clone());
        player.is_playing = true;
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.update_track(&track);
    });
    if cx.global::<Config>().discord_rpc {
        cx.update_global::<DiscordRpc, _>(|rpc, _cx| rpc.update_track(&track));
    }
    cx.refresh_windows();
}

pub fn toggle_play(cx: &mut App) {
    let is_playing = cx.global::<AudioPlayer>().is_playing;
    if is_playing {
        pause(cx);
    } else {
        resume(cx);
    }
}

pub fn resume(cx: &mut App) {
    let current_track = cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Resume);
        player.is_playing = true;
        player.current_track.clone()
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.set_playback(true);
    });
    if cx.global::<Config>().discord_rpc {
        if let Some(track) = current_track {
            cx.update_global::<DiscordRpc, _>(|rpc, _cx| rpc.update_track(&track));
        }
    }
    cx.refresh_windows();
}

pub fn pause(cx: &mut App) {
    let current_track = cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Pause);
        player.is_playing = false;
        player.current_track.clone()
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.set_playback(false);
    });
    if cx.global::<Config>().discord_rpc {
        if let Some(track) = current_track {
            cx.update_global::<DiscordRpc, _>(|rpc, _cx| rpc.set_paused(&track));
        } else {
            cx.update_global::<DiscordRpc, _>(|rpc, _cx| rpc.set_idle());
        }
    }
    cx.refresh_windows();
}

pub fn stop(cx: &mut App) {
    cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Stop);
        player.is_playing = false;
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        let _ = mc.0.set_playback(souvlaki::MediaPlayback::Stopped);
    });
    if cx.global::<Config>().discord_rpc {
        cx.update_global::<DiscordRpc, _>(|rpc, _cx| rpc.set_idle());
    }
    cx.refresh_windows();
}

pub fn play_context(tracks: Vec<Track>, index: usize, cx: &mut App) {
    let track_to_play =
        cx.update_global::<QueueManager, _>(|qm, _cx| qm.play_from_context(tracks, index));

    if let Some(track) = track_to_play {
        play_track(track, cx);
    }
}

pub fn play_next(cx: &mut App) {
    let current_track = cx.global::<AudioPlayer>().current_track.clone();

    let next_track = cx.update_global::<QueueManager, _>(|qm, _cx| qm.advance_next(current_track));

    if let Some(track) = next_track {
        play_track(track, cx);
    } else {
        stop(cx);
    }
}

pub fn on_track_end(cx: &mut App) {
    let current_track = cx.global::<AudioPlayer>().current_track.clone();

    let next_track = cx.update_global::<QueueManager, _>(|qm, _cx| qm.advance_auto(current_track));

    if let Some(track) = next_track {
        play_track(track, cx);
    } else {
        stop(cx);
    }
}

pub fn play_prev(cx: &mut App) {
    let (current_track, progress_ms) = cx.read_global::<AudioPlayer, _>(|p, _cx| {
        (
            p.current_track.clone(),
            p.progress_ms.load(std::sync::atomic::Ordering::Relaxed),
        )
    });

    if progress_ms > 3000 {
        cx.global::<AudioPlayer>()
            .sender
            .send(AudioCommand::Seek(0.0))
            .ok();
        return;
    }

    let prev_track =
        cx.update_global::<QueueManager, _>(|qm, _cx| qm.advance_previous(current_track));

    if let Some(track) = prev_track {
        play_track(track, cx);
    }
}

pub fn toggle_repeat(cx: &mut App) {
    let new_mode = cx.update_global::<QueueManager, _>(|qm, _cx| qm.cycle_repeat());
    cx.update_global::<Config, _>(|config, _cx| {
        config.repeat_mode = new_mode;
        let _ = crate::save_config(config);
    });
    cx.refresh_windows();
}
