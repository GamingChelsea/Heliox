use nsh_base::button;
use nsh_core::prelude::*;

use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, Config},
    player::{play_next, play_prev, toggle_play, toggle_repeat},
    queue::{QueueManager, RepeatMode},
    save_config,
    ui::{
        components::{
            audio_slider::{audio_slider, seek_slider},
            cover::cover_art,
            queue_popover::toggle_queue_popover,
        },
        icons::{IconName, icon_elem},
        theme::{Finish, Finished, Pressable, colors},
    },
};

fn format_time(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}

fn duration_secs() -> u32 {
    global::<AudioPlayer>()
        .get()
        .current_track
        .map_or(0, |track| track.duration_secs)
}

fn progress() -> f32 {
    let player = global::<AudioPlayer>().get();
    let duration = player
        .current_track
        .as_ref()
        .map_or(0, |track| track.duration_secs);
    if duration == 0 {
        return 0.0;
    }
    let current_secs = player
        .progress_ms
        .load(std::sync::atomic::Ordering::Relaxed) as f32
        / 1000.0;
    (current_secs / duration as f32).clamp(0.0, 1.0)
}

pub fn player_bar() -> impl IntoElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .h(px(80.0))
        .w_full()
        .px(px(16.0))
        .bg(colors::SURFACE)
        .border_t_1()
        .border_t_color(colors::BORDER)
        .child(dynamic(track_info))
        .child(
            view()
                .flex_1()
                .max_w(px(550.0))
                .items_center()
                .gap(px(6.0))
                .child(dynamic(playback_controls))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .w_full()
                        .gap(px(8.0))
                        .child(dynamic(|| {
                            text(format_time((progress() * duration_secs() as f32) as u32))
                                .w(px(40.0))
                                .text_right()
                                .text_size(px(11.0))
                                .text_color(colors::MUTED)
                        }))
                        .child(seek_slider(progress, |new_value| {
                            let target_secs = new_value * duration_secs() as f32;
                            if let Some(sender) = global::<AudioPlayer>().get_untracked().sender {
                                let _ = sender.send(AudioCommand::Seek(target_secs));
                            }
                        }))
                        .child(dynamic(|| {
                            text(format_time(duration_secs()))
                                .w(px(40.0))
                                .text_left()
                                .text_size(px(11.0))
                                .text_color(colors::MUTED)
                        })),
                ),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_end()
                .w(px(260.0))
                .gap(px(10.0))
                .child(
                    button()
                        .size(px(32.0))
                        .items_center()
                        .justify_center()
                        .rounded(px(6.0))
                        .pressable()
                        .accessibility_label("Queue")
                        .on_press(toggle_queue_popover)
                        .child(icon_elem(IconName::ListMusic, 18.0, colors::MUTED)),
                )
                .child(dynamic(volume_icon))
                .child(view().w(px(90.0)).child(audio_slider(
                    || global::<Config>().get().volume,
                    |new_volume| {
                        let volume = new_volume.clamp(0.0, 1.0);
                        if let Some(sender) = global::<AudioPlayer>().get_untracked().sender {
                            let _ = sender.send(AudioCommand::SetVolume(volume));
                        }
                        global::<Config>().update(|config| config.volume = volume);
                        save_config(&global::<Config>().get_untracked());
                        crate::update_media_controls_volume(volume as f64);
                    },
                ))),
        )
}

fn track_info() -> impl IntoElement {
    let player = global::<AudioPlayer>().get();
    let shorten = global::<Config>().get().shorten_titles;
    let (title, subtitle) = player
        .current_track
        .map(|track| {
            let title = track.display_title(shorten);
            let subtitle = if track.album.is_empty() || track.album == track.title {
                track.artist
            } else {
                format!("{} • {}", track.artist, track.album)
            };
            (title, subtitle)
        })
        .unwrap_or_else(|| ("No track playing".to_string(), String::new()));

    view()
        .flex_row()
        .items_center()
        .w(px(260.0))
        .gap(px(12.0))
        .child(cover_art(52.0))
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(title)
                        .number_of_lines(1)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text(subtitle)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(colors::MUTED),
                ),
        )
}

fn playback_controls() -> impl IntoElement {
    let player = global::<AudioPlayer>().get();
    let queue = global::<QueueManager>().get();
    let repeat_icon = match queue.repeat_mode {
        RepeatMode::Off => IconName::RepeatOff,
        RepeatMode::Context => IconName::Repeat,
        RepeatMode::Track => IconName::Repeat1,
    };

    view()
        .flex_row()
        .items_center()
        .gap(px(12.0))
        .child(
            button()
                .size(px(32.0))
                .items_center()
                .justify_center()
                .rounded(px(16.0))
                .pressable()
                .accessibility_label("Previous track")
                .on_press(play_prev)
                .child(icon_elem(IconName::SkipBack, 18.0, colors::TEXT)),
        )
        .child(
            button()
                .size(px(38.0))
                .items_center()
                .justify_center()
                .rounded(px(19.0))
                .pressable()
                .bg(colors::ACCENT_FILL)
                .finish(Finish::Accent)
                .accessibility_label(if player.is_playing { "Pause" } else { "Play" })
                .on_press(toggle_play)
                .child(icon_elem(
                    if player.is_playing {
                        IconName::Pause
                    } else {
                        IconName::Play
                    },
                    18.0,
                    colors::ON_ACCENT,
                )),
        )
        .child(
            button()
                .size(px(32.0))
                .items_center()
                .justify_center()
                .rounded(px(16.0))
                .pressable()
                .accessibility_label("Next track")
                .on_press(play_next)
                .child(icon_elem(IconName::SkipForward, 18.0, colors::TEXT)),
        )
        .child(
            button()
                .size(px(32.0))
                .items_center()
                .justify_center()
                .rounded(px(16.0))
                .pressable()
                .accessibility_label("Repeat")
                .on_press(toggle_repeat)
                .child(icon_elem(
                    repeat_icon,
                    16.0,
                    if queue.repeat_mode == RepeatMode::Off {
                        colors::MUTED
                    } else {
                        colors::ACCENT
                    },
                )),
        )
}

fn volume_icon() -> impl IntoElement {
    let volume = global::<Config>().get().volume;
    view()
        .child(icon_elem(
            if volume <= 0.01 {
                IconName::VolumeX
            } else {
                IconName::Volume
            },
            18.0,
            colors::MUTED,
        ))
        .child(
            text(format!("{:.0}%", volume * 100.0))
                .w(px(34.0))
                .text_size(px(11.0))
                .text_color(colors::MUTED),
        )
}
