use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    time::Duration,
};

use nsh_base::{CommandEntry, CommandItem, command};
use nsh_core::prelude::*;
use strum::IntoEnumIterator;

use crate::{
    model::{Config, Track},
    player::play_track,
    providers::youtube::{self, SearchFilter},
    queue::RepeatMode,
    router::{NavigationRoute, navigate_to},
    save_config,
    ui::{
        components::text_input::visible_input_text,
        icons::{IconName, icon_elem},
        theme::{AppRadius, Finish, Finished, THEMES, colors, select_theme},
    },
};

pub mod actions;
pub mod modes;

use actions::all_action_groups;
use modes::PaletteMode;

#[derive(Clone, Copy, Default)]
pub struct CommandPaletteOpen(pub bool);

#[derive(Clone, Default)]
pub struct CommandPaletteQuery(pub String);

#[derive(Clone, Default)]
struct PaletteSearchState {
    query: String,
    filter: SearchFilter,
    tracks: Vec<Track>,
    loading: bool,
    error: Option<String>,
}

pub fn open() {
    global::<CommandPaletteOpen>().set(CommandPaletteOpen(true));
    global::<CommandPaletteQuery>().set(CommandPaletteQuery(String::new()));
}

pub fn close() {
    global::<CommandPaletteOpen>().set(CommandPaletteOpen(false));
}

pub fn close_if_open() {
    if global::<CommandPaletteOpen>().get_untracked().0 {
        close();
    }
}

pub fn toggle() {
    if global::<CommandPaletteOpen>().get_untracked().0 {
        close();
    } else {
        open();
    }
}

pub fn set_query(query: &str) {
    global::<CommandPaletteQuery>().set(CommandPaletteQuery(query.to_string()));
}

pub fn command_palette() -> impl IntoElement {
    dynamic(command_palette_view)
}

fn command_palette_view() -> impl IntoElement {
    if !global::<CommandPaletteOpen>().get().0 {
        return view().into_element();
    }

    overlay()
        .absolute()
        .inset_0()
        .items_center()
        .justify_center()
        .child(
            view()
                .absolute()
                .inset_0()
                .bg(hsla(0.0, 0.0, 0.0, 0.6))
                .on_press(close),
        )
        .child(
            view()
                .w(px(520.0))
                .max_h(px(480.0))
                .p(px(14.0))
                .app_radius(1.5)
                .finish(Finish::Card)
                .bg(colors::SURFACE)
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    command()
                        .items(move || palette_items())
                        .query(move || Some(global::<CommandPaletteQuery>().get().0.into()))
                        .searchable(true)
                        .filterable(false)
                        .loading(move || palette_search_loading())
                        .placeholder("Type a command ('ct ', 'nav ', 'search ')...")
                        .auto_focus(true)
                        .accessibility_label("Heliox commands")
                        .search_accessibility_label("Search commands")
                        .search_prefix(icon_elem(IconName::Search, 18.0, colors::INPUT_MUTED))
                        .search_suffix(when(cfg!(target_os = "windows"), move || {
                            view()
                                .absolute()
                                .inset_0()
                                .pl(px(40.0))
                                .flex_row()
                                .items_center()
                                .pointer_events(PointerEvents::None)
                                .accessibility_elements_hidden(true)
                                .child(visible_input_text(
                                    move || global::<CommandPaletteQuery>().get().0,
                                    "Type a command ('ct ', 'nav ', 'search ')...",
                                    px(14.0),
                                ))
                        }))
                        .search_bar_style(|style| {
                            style
                                .relative()
                                .flex_row()
                                .items_center()
                                .h(px(42.0))
                                .px(px(12.0))
                                .gap(px(10.0))
                                .bg(colors::INPUT)
                                .text_color(colors::INPUT_TEXT)
                                .border_1()
                                .border_color(colors::BORDER)
                                .app_radius(1.0)
                        })
                        .search_style(|style| {
                            style
                                .flex_1()
                                .text_color(colors::INPUT_TEXT)
                                .text_size(px(14.0))
                                .bg(colors::INPUT)
                        })
                        .list_class("max-h-[380px]")
                        .item_class("flex-row items-center w-full h-9 px-2.5 gap-2.5 rounded-lg")
                        .group_class("px-2 pt-2 text-xs")
                        .item_style(|style| style.text_color(colors::TEXT))
                        .group_style(|style| style.text_color(colors::FAINT))
                        .empty(
                            text(move || palette_empty_message())
                                .text_size(px(13.0))
                                .text_color(colors::MUTED),
                        )
                        .render_item(|entry: &CommandEntry| {
                            let icon = palette_icon(&entry.id);
                            let shortcut = palette_shortcut(&entry.id);
                            let label = text(entry.label.clone())
                                .flex_1()
                                .truncate()
                                .text_size(px(13.0))
                                .text_color(colors::TEXT);
                            view()
                                .flex_row()
                                .items_center()
                                .w(px(470.0))
                                .gap(px(10.0))
                                .child(icon_elem(icon, 16.0, colors::ACCENT))
                                .child(label)
                                .children(shortcut.map(|shortcut| {
                                    view()
                                        .px(px(7.0))
                                        .py(px(3.0))
                                        .app_radius(0.5)
                                        .finish(Finish::Pill)
                                        .flex_shrink_0()
                                        .child(
                                            text(shortcut)
                                                .text_size(px(10.0))
                                                .text_color(colors::TEXT),
                                        )
                                }))
                        })
                        .on_query_change(move |query: &SharedString| set_query(query))
                        .on_select(move |id: &SharedString| execute_palette_item(id))
                        .on_cancel(close)
                        .h(px(450.0)),
                ),
        )
        .into_element()
}

fn palette_items() -> Vec<CommandItem> {
    let query = global::<CommandPaletteQuery>().get().0;
    match PaletteMode::parse(&query) {
        PaletteMode::Actions => {
            let filter = query.trim().to_lowercase();
            all_action_groups()
                .into_iter()
                .flat_map(|(group, actions)| {
                    actions
                        .into_iter()
                        .filter(|action| {
                            filter.is_empty()
                                || action.label.to_lowercase().contains(&filter)
                                || action
                                    .keywords
                                    .iter()
                                    .any(|keyword| keyword.contains(&filter))
                        })
                        .map(move |action| {
                            CommandItem::new(action.id, action.label)
                                .keywords(action.keywords.iter().copied())
                                .group(group)
                        })
                })
                .collect()
        }
        PaletteMode::Navigation(filter) => NavigationRoute::get_clean_routes()
            .into_iter()
            .filter(|route| filter.is_empty() || route.meta().name.to_lowercase().contains(filter))
            .map(|route| {
                CommandItem::new(format!("route:{}", route.meta().name), route.meta().name)
                    .group("Pages")
            })
            .collect(),
        PaletteMode::Themes(filter) => THEMES
            .into_iter()
            .filter(|theme| filter.is_empty() || theme.to_lowercase().contains(filter))
            .map(|theme| CommandItem::new(format!("theme:{theme}"), theme).group("Themes"))
            .collect(),
        PaletteMode::RepeatMode(filter) => RepeatMode::iter()
            .filter(|repeat| repeat.title().to_lowercase().contains(filter))
            .map(|repeat| {
                CommandItem::new(format!("repeat:{repeat:?}"), repeat.title()).group("Repeat")
            })
            .collect(),
        PaletteMode::Search(search_query, filter) => {
            if search_query.is_empty() {
                return Vec::new();
            }

            let state = global::<PaletteSearchState>().get();
            if state.query != query || state.filter != filter {
                return vec![
                    CommandItem::new(
                        "search:execute",
                        format!("Search YouTube for {search_query}"),
                    )
                    .group("Search"),
                ];
            }
            if state.loading {
                return vec![
                    CommandItem::new("search:loading", "Searching YouTube…").disabled(true),
                ];
            }
            if let Some(error) = state.error {
                return vec![
                    CommandItem::new("search:error", format!("Search failed: {error}"))
                        .disabled(true),
                ];
            }
            if state.tracks.is_empty() {
                return vec![CommandItem::new("search:empty", "No tracks found.").disabled(true)];
            }

            let mut items = vec![
                CommandItem::new(
                    "search:execute",
                    format!("Search YouTube for {search_query}"),
                )
                .group("Search"),
            ];
            items.extend(state.tracks.into_iter().map(|track| {
                CommandItem::new(format!("track:{}", track.id), track.title)
                    .keywords([track.artist])
                    .group("Results")
            }));
            items
        }
    }
}

fn palette_search_loading() -> bool {
    let query = global::<CommandPaletteQuery>().get().0;
    let PaletteMode::Search(search_query, filter) = PaletteMode::parse(&query) else {
        return false;
    };
    let state = global::<PaletteSearchState>().get();
    !search_query.is_empty() && state.query == query && state.filter == filter && state.loading
}

fn palette_empty_message() -> String {
    let query = global::<CommandPaletteQuery>().get().0;
    match PaletteMode::parse(&query) {
        PaletteMode::Search(search_query, _) if !search_query.is_empty() => String::new(),
        PaletteMode::Search(_, _) => "Type a search query, then press Enter.".to_string(),
        _ => "No commands found.".to_string(),
    }
}

fn palette_icon(id: &str) -> IconName {
    match id {
        "search:execute" => IconName::Search,
        id if id.starts_with("route:") => IconName::House,
        id if id.starts_with("theme:") => IconName::Spark,
        id if id.starts_with("repeat:") => IconName::Repeat,
        id if id.starts_with("track:") => IconName::Music,
        id => all_action_groups()
            .into_iter()
            .flat_map(|(_, actions)| actions)
            .find(|action| action.id == id)
            .map_or(IconName::Settings, |action| action.icon),
    }
}

fn palette_shortcut(id: &str) -> Option<&'static str> {
    all_action_groups()
        .into_iter()
        .flat_map(|(_, actions)| actions)
        .find(|action| action.id == id)
        .and_then(|action| action.shortcut)
}

fn execute_palette_item(id: &str) {
    if let Some(action) = all_action_groups()
        .into_iter()
        .flat_map(|(_, actions)| actions)
        .find(|action| action.id == id)
    {
        (action.execute)();
        return;
    }

    if let Some(theme) = id.strip_prefix("theme:") {
        select_theme(theme);
        close();
    } else if let Some(route) = id.strip_prefix("route:") {
        if let Some(route) = NavigationRoute::get_clean_routes()
            .into_iter()
            .find(|candidate| candidate.meta().name == route)
        {
            navigate_to(route);
            close();
        }
    } else if let Some(mode) = id.strip_prefix("repeat:") {
        if let Some(repeat) = RepeatMode::iter().find(|repeat| format!("{repeat:?}") == mode) {
            global::<Config>().update(|config| config.repeat_mode = repeat);
            save_config(&global::<Config>().get_untracked());
            close();
        }
    } else if id == "search:execute" {
        start_search();
    } else if let Some(track_id) = id
        .strip_prefix("track:")
        .and_then(|id| id.parse::<u64>().ok())
    {
        if let Some(track) = global::<PaletteSearchState>()
            .get_untracked()
            .tracks
            .into_iter()
            .find(|track| track.id == track_id)
        {
            play_track(track);
            close();
        }
    }
}

fn start_search() {
    let query = global::<CommandPaletteQuery>().get_untracked().0;
    let (search_query, filter) = match PaletteMode::parse(&query) {
        PaletteMode::Search(search_query, filter) if !search_query.is_empty() => {
            (search_query.to_string(), filter)
        }
        _ => return,
    };

    global::<PaletteSearchState>().set(PaletteSearchState {
        query: query.clone(),
        filter,
        tracks: Vec::new(),
        loading: true,
        error: None,
    });

    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let result =
            youtube::search_filtered(&search_query, filter).map_err(|error| error.to_string());
        let _ = sender.send(result);
    });
    poll_search(receiver, query, filter);
}

fn poll_search(
    receiver: Receiver<Result<Vec<Track>, String>>,
    query: String,
    filter: SearchFilter,
) {
    set_timeout(Duration::from_millis(50), move || {
        match receiver.try_recv() {
            Ok(result) => {
                if global::<CommandPaletteQuery>().get_untracked().0 != query {
                    return;
                }
                global::<PaletteSearchState>().set(match result {
                    Ok(tracks) => PaletteSearchState {
                        query,
                        filter,
                        tracks,
                        loading: false,
                        error: None,
                    },
                    Err(error) => PaletteSearchState {
                        query,
                        filter,
                        tracks: Vec::new(),
                        loading: false,
                        error: Some(error),
                    },
                });
            }
            Err(TryRecvError::Empty) => poll_search(receiver, query, filter),
            Err(TryRecvError::Disconnected) => {
                if global::<CommandPaletteQuery>().get_untracked().0 == query {
                    global::<PaletteSearchState>().set(PaletteSearchState {
                        query,
                        filter,
                        tracks: Vec::new(),
                        loading: false,
                        error: Some("Search task stopped unexpectedly.".to_string()),
                    });
                }
            }
        }
    });
}
