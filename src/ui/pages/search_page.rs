use nsh_base::button;
use nsh_core::prelude::*;

use crate::{
    model::Track,
    providers::youtube::{self, SearchFilter},
    ui::{
        components::{text_input::text_input, track_list::track_list},
        icons::{IconName, icon_elem},
        theme::{AppRadius, Finish, Finished, colors},
    },
};

#[derive(Clone, Default)]
pub struct SearchState {
    pub query: String,
    pub filter: SearchFilter,
    pub is_loading: bool,
    pub results: Vec<Track>,
    pub error: Option<String>,
    request_id: u64,
}

pub fn trigger_search() {
    let state = global::<SearchState>().get_untracked();
    let q = state.query.trim().to_string();
    let filter = state.filter;

    if q.is_empty() {
        global::<SearchState>().update(|s| {
            s.results.clear();
            s.is_loading = false;
            s.error = None;
            s.request_id = s.request_id.wrapping_add(1);
        });
        return;
    }

    let request_id = state.request_id.wrapping_add(1);
    global::<SearchState>().update(|s| {
        s.is_loading = true;
        s.error = None;
        s.request_id = request_id;
    });

    let (tx, rx) = async_channel::bounded(1);
    std::thread::spawn(move || {
        let result = youtube::search_filtered(&q, filter).map_err(|error| error.to_string());
        let _ = tx.send_blocking(result);
    });
    spawn(async move {
        if let Ok(result) = rx.recv().await {
            global::<SearchState>().update(|s| {
                if s.request_id == request_id {
                    s.is_loading = false;
                    match result {
                        Ok(results) => {
                            s.results = results;
                            s.error = None;
                        }
                        Err(error) => {
                            s.results.clear();
                            s.error = Some(error);
                        }
                    }
                }
            });
        } else {
            global::<SearchState>().update(|s| {
                if s.request_id == request_id {
                    s.is_loading = false;
                    s.error = Some("The search request stopped unexpectedly.".to_string());
                }
            });
        }
    });
}

pub fn search_page() -> impl IntoElement {
    search_page_view()
}

fn search_page_view() -> impl IntoElement {
    view()
        .flex_1()
        .p(px(24.0))
        .gap(px(16.0))
        // Header & Search Box
        .child(
            view()
                .gap(px(12.0))
                .child(
                    text("Search Music")
                        .text_size(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .h(px(44.0))
                        .px(px(14.0))
                        .gap(px(10.0))
                        .bg(colors::INPUT)
                        .text_color(colors::INPUT_TEXT)
                        .border_1()
                        .border_color(colors::BORDER)
                        .app_radius(1.25)
                        .child(icon_elem(IconName::Search, 20.0, colors::INPUT_MUTED))
                        .child(text_input(
                            move || global::<SearchState>().get().query,
                            "Search artists, songs or videos on YouTube...",
                            move |v: &str| {
                                global::<SearchState>().update(|s| {
                                    s.query = v.to_string();
                                    s.results.clear();
                                    s.error = None;
                                    s.is_loading = false;
                                    s.request_id = s.request_id.wrapping_add(1);
                                });
                            },
                            move |_: &str| trigger_search(),
                        ))
                        .child(
                            button()
                                .h(px(30.0))
                                .px(px(12.0))
                                .items_center()
                                .justify_center()
                                .app_radius(0.75)
                                .bg(colors::ACCENT_FILL)
                                .finish(Finish::Accent)
                                .on_press(trigger_search)
                                .child(
                                    text("Search")
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(colors::ON_ACCENT),
                                ),
                        ),
                )
                // Filter Pills
                .child(dynamic(filter_buttons)),
        )
        // Results
        .child(view().flex_1().child(dynamic(search_results_view)))
}

fn filter_buttons() -> impl IntoElement {
    let selected = global::<SearchState>().get().filter;
    let filter_pill = |label: &'static str, filter: SearchFilter| {
        let is_selected = selected == filter;
        button()
            .h(px(32.0))
            .px(px(14.0))
            .items_center()
            .justify_center()
            .app_radius(2.0)
            .map(|v| {
                if is_selected {
                    v.bg(colors::ACCENT_FILL)
                } else {
                    v.bg(colors::RAISED).border_1().border_color(colors::BORDER)
                }
            })
            .on_press(move || {
                global::<SearchState>().update(|s| s.filter = filter);
                trigger_search();
            })
            .child(
                text(label)
                    .text_size(px(13.0))
                    .font_weight(if is_selected {
                        FontWeight::BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(if is_selected {
                        colors::ON_ACCENT
                    } else {
                        colors::MUTED
                    }),
            )
    };

    view()
        .flex_row()
        .gap(px(8.0))
        .child(filter_pill("All", SearchFilter::All))
        .child(filter_pill("Songs", SearchFilter::Songs))
        .child(filter_pill("Videos", SearchFilter::Videos))
}

fn search_results_view() -> impl IntoElement {
    let state = global::<SearchState>().get();
    if state.is_loading {
        return view()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                text("Searching YouTube...")
                    .text_size(px(15.0))
                    .text_color(colors::MUTED),
            )
            .into_element();
    }

    if let Some(error) = state.error {
        return view()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                text(format!("YouTube search failed: {error}"))
                    .text_size(px(14.0))
                    .text_color(colors::DANGER),
            )
            .into_element();
    }

    if state.results.is_empty() {
        return view()
            .flex_1()
            .items_center()
            .justify_center()
            .child(
                text(if state.query.is_empty() {
                    "Type something above to search for tracks or videos."
                } else {
                    "No tracks found for your search."
                })
                .text_size(px(15.0))
                .text_color(colors::MUTED),
            )
            .into_element();
    }
    track_list(state.results).into_element()
}
