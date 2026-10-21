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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let volume = cx.global::<Config>().volume;

        let (current_track, is_playing) = cx.read_global::<AudioPlayer, _>(|player, _cx| {
            (player.current_track.clone(), player.is_playing)
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

        h_flex()
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
                            .gap_y_0p5()
                            .child(div().text_sm().truncate().child(title))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .truncate()
                                    .child(artist),
                            ),
                    ),
            )
            // Middle
            .child(
                v_flex()
                    .flex_1()
                    .h_full()
                    .min_w_0()
                    .items_center()
                    .justify_center()
                    .child(
                        v_flex()
                            .w_full()
                            .max_w(px(550.0))
                            .items_center()
                            .justify_center()
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
                            ),
                    ),
            )
            // Right
            .child(
                h_flex()
                    .w(px(260.0))
                    .min_w_0()
                    .justify_end()
                    .items_center()
                    .gap_2()
                    .child(
                        div().w(px(80.0)).child(
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
                        ),
                    )
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
