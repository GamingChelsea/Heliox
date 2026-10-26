use crate::model::{Config, CurrentTrackList, Track};
use crate::player::play_context;
use gpui_kit::base::{h_flex, v_flex};
use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

#[derive(IntoElement)]
pub struct TrackList {
    tracks: Vec<Track>,
}

impl TrackList {
    pub fn new(tracks: Vec<Track>) -> Self {
        Self { tracks }
    }

    fn format_duration(secs: u32) -> String {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

impl RenderOnce for TrackList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let count = self.tracks.len();
        let selected_id = cx.global::<CurrentTrackList>().selected_id;
        let tracks = self.tracks;
        let shorten = cx.global::<Config>().shorten_titles;

        v_flex()
            .size_full()
            .child(
                h_flex()
                    .w_full()
                    .h_8()
                    .px_4()
                    .gap_3()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().w_10().child("#"))
                    .child(
                        div()
                            .flex_grow(3.0)
                            .flex_basis(px(0.0))
                            .overflow_hidden()
                            .child("Title"),
                    )
                    .child(
                        div()
                            .flex_grow(2.0)
                            .flex_basis(px(0.0))
                            .overflow_hidden()
                            .child("Artist"),
                    )
                    .child(
                        div()
                            .flex_grow(2.0)
                            .flex_basis(px(0.0))
                            .overflow_hidden()
                            .child("Album"),
                    )
                    .child(div().w_16().text_right().child("Duration")),
            )
            .child(
                uniform_list("track-table", count, move |range, _window, cx| {
                    range
                        .map(|idx| {
                            let track = &tracks[idx];
                            let track_id = track.id;
                            let is_selected = selected_id == Some(track_id);
                            let all_tracks = tracks.clone();

                            h_flex()
                                .id(ElementId::NamedInteger("track-row".into(), track_id))
                                .w_full()
                                .h_10()
                                .px_4()
                                .gap_3()
                                .items_center()
                                .rounded(cx.theme().radius)
                                .when(is_selected, |this| this.bg(cx.theme().accent))
                                .hover(|this| this.bg(cx.theme().accent.opacity(0.5)))
                                .cursor_pointer()
                                .on_click(move |_event, _window, cx| {
                                    play_context(all_tracks.clone(), idx, cx);
                                })
                                .child(
                                    div()
                                        .w_10()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("{}", idx + 1)),
                                )
                                .child(
                                    div()
                                        .flex_grow(3.0)
                                        .flex_basis(px(0.0))
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_sm()
                                                .truncate()
                                                .child(track.display_title(shorten)),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_grow(2.0)
                                        .flex_basis(px(0.0))
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .truncate()
                                                .child(track.artist.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_grow(2.0)
                                        .flex_basis(px(0.0))
                                        .overflow_hidden()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .truncate()
                                                .child(track.album.clone()),
                                        ),
                                )
                                .child(
                                    div()
                                        .w_16()
                                        .text_sm()
                                        .text_right()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(Self::format_duration(track.duration_secs)),
                                )
                        })
                        .collect::<Vec<_>>()
                })
                .flex_1(),
            )
    }
}
