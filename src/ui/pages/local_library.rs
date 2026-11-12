use nsh_core::prelude::*;

use crate::{
    library::reload_local_library,
    model::{Config, CurrentTrackList},
    player::play_context,
    ui::{
        components::track_list::track_list,
        icons::{IconName, icon_elem},
        theme::{AppRadius, Clickable, Finish, Finished, colors},
    },
};

pub fn local_library_page() -> impl IntoElement {
    dynamic(local_library_page_view)
}

fn local_library_page_view() -> impl IntoElement {
    let track_list_state = global::<CurrentTrackList>().get();

    let content = match track_list_state.tracks {
        Some(tracks) if !tracks.is_empty() => {
            let tracks_for_play = tracks.clone();
            let count = tracks.len();

            view()
                .flex_1()
                .gap(px(16.0))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .child(
                            view()
                                .gap(px(2.0))
                                .child(
                                    text("Local Library")
                                        .text_size(px(28.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors::TEXT),
                                )
                                .child(
                                    text(format!(
                                        "{count} tracks loaded from your music directory."
                                    ))
                                    .text_size(px(13.0))
                                    .text_color(colors::MUTED),
                                ),
                        )
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .h(px(36.0))
                                .px(px(16.0))
                                .gap(px(8.0))
                                .app_radius(1.0)
                                .bg(colors::ACCENT_FILL)
                                .finish(Finish::Accent)
                                .clickable()
                                .label("Play All")
                                .on_press(move || {
                                    play_context(tracks_for_play.clone(), 0);
                                })
                                .child(icon_elem(IconName::Play, 14.0, colors::ON_ACCENT))
                                .child(
                                    text("Play All")
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors::ON_ACCENT),
                                ),
                        ),
                )
                .child(track_list(tracks))
                .into_element()
        }
        Some(_) if let Some(error) = track_list_state.load_error => view()
            .flex_1()
            .items_center()
            .justify_center()
            .gap(px(12.0))
            .child(text(error).text_size(px(14.0)).text_color(colors::DANGER))
            .child(
                view()
                    .h(px(34.0))
                    .px(px(14.0))
                    .items_center()
                    .justify_center()
                    .app_radius(1.0)
                    .bg(colors::ACCENT_FILL)
                    .clickable()
                    .label("Retry library scan")
                    .on_press(reload_local_library)
                    .child(
                        text("Retry")
                            .text_size(px(13.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::ON_ACCENT),
                    ),
            )
            .into_element(),
        Some(_) => view()
            .flex_1()
            .items_center()
            .justify_center()
            .gap(px(10.0))
            .child(dynamic(|| {
                text(format!(
                    "No audio tracks found in {}.",
                    global::<Config>().get().music_dir.display()
                ))
                .text_size(px(15.0))
                .text_color(colors::MUTED)
            }))
            .child(
                view()
                    .h(px(34.0))
                    .px(px(14.0))
                    .items_center()
                    .justify_center()
                    .app_radius(1.0)
                    .bg(colors::RAISED)
                    .border_1()
                    .border_color(colors::BORDER)
                    .clickable()
                    .label("Rescan music directory")
                    .on_press(reload_local_library)
                    .child(
                        text("Rescan")
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT),
                    ),
            )
            .into_element(),
        None => view()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                text("Loading local library...")
                    .text_size(px(15.0))
                    .text_color(colors::MUTED),
            )
            .into_element(),
    };

    view().flex_1().p(px(24.0)).child(content)
}
