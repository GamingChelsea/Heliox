use nsh_core::prelude::*;

use crate::{
    model::{Config, CurrentTrackList, Track},
    player::play_context,
    ui::theme::{AppRadius, Clickable, colors},
};

fn format_duration(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

pub fn track_list(tracks: Vec<Track>) -> impl IntoElement {
    dynamic(move || track_list_view(tracks.clone()))
}

fn track_list_view(tracks: Vec<Track>) -> impl IntoElement {
    let shorten = global::<Config>().get().shorten_titles;
    let selected_id = global::<CurrentTrackList>().get().selected_id;
    let all_tracks = tracks.clone();

    let header = view()
        .flex_row()
        .items_center()
        .h(px(32.0))
        .px(px(16.0))
        .gap(px(12.0))
        .border_b_1()
        .border_b_color(colors::LINE)
        .child(
            text("#")
                .w(px(36.0))
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        )
        .child(
            text("Title")
                .flex_1()
                .flex_grow(3.0)
                .truncate()
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        )
        .child(
            text("Artist")
                .flex_1()
                .flex_grow(2.0)
                .truncate()
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        )
        .child(
            text("Album")
                .flex_1()
                .flex_grow(2.0)
                .truncate()
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        )
        .child(
            text("Duration")
                .w(px(60.0))
                .text_right()
                .text_size(px(12.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        );

    let rows = tracks.into_iter().enumerate().map(|(idx, track)| {
        let is_selected = selected_id == Some(track.id);
        let tracks_clone = all_tracks.clone();
        let display_title = track.display_title(shorten);
        let duration = format_duration(track.duration_secs);

        view()
            .flex_row()
            .items_center()
            .h(px(40.0))
            .px(px(16.0))
            .gap(px(12.0))
            .app_radius(0.75)
            .map(|v| {
                if is_selected {
                    v.bg(colors::RAISED).border_1().border_color(colors::ACCENT)
                } else {
                    v
                }
            })
            .clickable()
            .label(display_title.clone())
            .on_press(move || {
                play_context(tracks_clone.clone(), idx);
            })
            .child(
                text((idx + 1).to_string())
                    .w(px(36.0))
                    .text_size(px(13.0))
                    .text_color(if is_selected {
                        colors::ACCENT
                    } else {
                        colors::MUTED
                    }),
            )
            .child(
                text(display_title)
                    .truncate()
                    .flex_1()
                    .flex_grow(3.0)
                    .text_size(px(14.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if is_selected {
                        colors::ACCENT
                    } else {
                        colors::TEXT
                    }),
            )
            .child(
                text(track.artist)
                    .truncate()
                    .flex_1()
                    .flex_grow(2.0)
                    .text_size(px(13.0))
                    .text_color(colors::MUTED),
            )
            .child(
                text(track.album)
                    .truncate()
                    .flex_1()
                    .flex_grow(2.0)
                    .text_size(px(13.0))
                    .text_color(colors::MUTED),
            )
            .child(
                text(duration)
                    .w(px(60.0))
                    .text_right()
                    .text_size(px(13.0))
                    .text_color(colors::MUTED),
            )
    });

    view()
        .flex_1()
        .child(header)
        .child(scroll().flex_1().child(view().py(px(4.0)).children(rows)))
}
