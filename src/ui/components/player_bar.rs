use gpui_kit::{
    assets::IconName,
    base::{h_flex, v_flex},
    component::{ActiveTheme, button::Button},
    *,
};

use crate::{
    audio::AudioCommand,
    model::{AudioPlayer, Config},
    save_config,
    ui::components::audio_slider::AudioSlider,
};

#[derive(IntoElement)]
pub struct PlayerBar;

impl PlayerBar {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for PlayerBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let volume = cx.global::<Config>().volume;

        let (current_track, is_playing, progress_ms) =
            cx.read_global::<AudioPlayer, _>(|player, _cx| {
                (
                    player.current_track.clone(),
                    player.is_playing,
                    player.progress_ms.clone(),
                )
            });
        let shorten = cx.global::<Config>().shorten_titles;
        let title: SharedString = current_track
            .as_ref()
            .map(|t| t.display_title(shorten).into())
            .unwrap_or_else(|| "No track playing".into());
        let artist: SharedString = current_track
            .as_ref()
            .map(|t| t.artist.clone().into())
            .unwrap_or_else(|| "".into());
        let play_icon = if is_playing {
            IconName::Pause
        } else {
            IconName::Play
        };

        if is_playing {
            window.use_keyed_state("playback-timer", cx, |window, cx| {
                window
                    .spawn(cx, async move |cx| {
                        loop {
                            cx.background_executor()
                                .timer(std::time::Duration::from_millis(100))
                                .await;
                            let _ = cx.update(|window, _cx| {
                                window.refresh();
                            });
                        }
                    })
                    .detach();
            });
        }

        let current_secs = progress_ms.load(std::sync::atomic::Ordering::Relaxed) as f32 / 1000.0;
        let total_secs = current_track.as_ref().map(|t| t.duration_secs).unwrap_or(0);
        let progress = if total_secs > 0 {
            (current_secs / total_secs as f32).clamp(0.0, 1.0)
        } else {
            0.0
        };

        h_flex()
            .relative()
            .h_20()
            .w_full()
            .items_center()
            .justify_between()
            .px_3()
            .bg(cx.theme().sidebar)
            .border_t_1()
            .border_color(cx.theme().border)
            // Left
            .child(
                h_flex()
                    .w(px(260.0))
                    .flex_shrink_0()
                    .min_w_0()
                    .gap_3()
                    .items_center()
                    .child(
                        img("TODO.svg")
                            .id("inbox-image")
                            .size(px(56.))
                            .flex_shrink_0()
                            .rounded(cx.theme().radius)
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().muted)
                            .overflow_hidden()
                            .object_fit(ObjectFit::Contain)
                            .with_loading(|| {
                                h_flex()
                                    .size_full()
                                    .justify_center()
                                    .items_center()
                                    .child(IconName::Loader)
                                    .into_any_element()
                            })
                            .with_fallback(|| {
                                h_flex()
                                    .size_full()
                                    .justify_center()
                                    .items_center()
                                    .child(IconName::Music)
                                    .into_any_element()
                            }),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .overflow_hidden()
                            .text_left()
                            .gap_1()
                            .child(
                                div()
                                    .h(px(32.0))
                                    .flex()
                                    .items_center()
                                    .text_sm()
                                    .truncate()
                                    .child(title),
                            )
                            .child(
                                div()
                                    .h(px(16.0))
                                    .flex()
                                    .items_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(artist),
                            ),
                    ),
            )
            // Middle
            .child(
                div()
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(550.0))
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .child(
                                        Button::new("Prev")
                                            .icon(IconName::SkipBack)
                                            .child("NOT WORKING"),
                                    )
                                    .child(Button::new("Play").icon(play_icon).on_click(
                                        |_event, _window, cx| {
                                            cx.update_global::<AudioPlayer, _>(
                                                |audio_player, _cx| audio_player.toggle_play(),
                                            );
                                        },
                                    ))
                                    .child(
                                        Button::new("Next")
                                            .icon(IconName::SkipForward)
                                            .child("NOT WORKING"),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .gap_2()
                                    // Current time
                                    .child(
                                        div()
                                            .w_10()
                                            .h(px(16.0))
                                            .flex()
                                            .items_center()
                                            .justify_end()
                                            .text_xs()
                                            .line_height(px(16.0))
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format_time(current_secs as u32)),
                                    )
                                    // Bar
                                    .child(
                                        div().flex_1().h(px(16.0)).flex().items_center().child(
                                            AudioSlider::new("playback-progress-slider", progress)
                                                .on_change(move |new_val, window, cx| {
                                                    let target_secs = new_val * total_secs as f32;
                                                    let _ = cx
                                                        .global::<AudioPlayer>()
                                                        .sender
                                                        .send(AudioCommand::Seek(target_secs));
                                                    window.refresh();
                                                }),
                                        ),
                                    )
                                    // Total
                                    .child(
                                        div()
                                            .w_10()
                                            .h(px(16.0))
                                            .flex()
                                            .items_center()
                                            .justify_start()
                                            .text_xs()
                                            .line_height(px(16.0))
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format_time(total_secs)),
                                    ),
                            ),
                    ),
            )
            // Right
            .child(
                h_flex()
                    .w(px(260.0))
                    .flex_shrink_0()
                    .min_w_0()
                    .justify_end()
                    .items_center()
                    .gap_2()
                    .child(div().w(px(80.0)).child(
                        AudioSlider::new("player-volume-slider", volume).on_change(
                            |new_vol, window, cx| {
                                let _ = cx
                                    .global::<AudioPlayer>()
                                    .sender
                                    .send(AudioCommand::SetVolume(new_vol));
                                cx.update_global::<Config, _>(|config, _cx| {
                                    config.volume = new_vol;
                                    let _ = save_config(config);
                                });
                                window.refresh();
                            },
                        ),
                    ))
                    .child(
                        div()
                            .w_8()
                            .text_xs()
                            .text_right()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("{:.0}%", volume * 100.0)),
                    ),
            )
    }
}

fn format_time(secs: u32) -> String {
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
