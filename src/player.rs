use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, Config, CurrentTrackList, DiscordRpc, MediaControl, Track},
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
