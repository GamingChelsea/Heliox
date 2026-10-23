use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, CurrentTrackList, MediaControl, Track},
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
    cx.refresh_windows();
}

pub fn toggle_play(cx: &mut App) {
    let is_playing = cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::ResumePause);
        player.is_playing = !player.is_playing;
        player.is_playing
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.set_playback(is_playing);
    });
    cx.refresh_windows();
}

pub fn resume(cx: &mut App) {
    cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Resume);
        player.is_playing = true;
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.set_playback(true);
    });
    cx.refresh_windows();
}

pub fn pause(cx: &mut App) {
    cx.update_global::<AudioPlayer, _>(|player, _cx| {
        let _ = player.sender.send(AudioCommand::Pause);
        player.is_playing = false;
    });
    cx.update_global::<MediaControl, _>(|mc, _cx| {
        mc.set_playback(false);
    });
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
    cx.refresh_windows();
}
