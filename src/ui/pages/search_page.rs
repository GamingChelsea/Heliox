use crate::model::Track;
use crate::providers::youtube::{self, SearchFilter};
use crate::ui::components::track_list::TrackList;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::spinner::Spinner;
use gpui_kit::component::{ActiveTheme, Icon, Selectable as _};
use gpui_kit::{
    assets::IconName,
    base::{h_flex, v_flex},
    component::label::Label,
    *,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[derive(Default, Clone)]
pub struct SearchPageState {
    pub results: Vec<Track>,
    pub is_loading: bool,
    pub last_query: String,
}

#[derive(IntoElement)]
pub struct SearchPage;
impl SearchPage {
    pub fn new() -> Self {
        Self
    }
}

pub fn trigger_search(
    query: String,
    filter: SearchFilter,
    debounce_ms: u64,
    state: Rc<RefCell<SearchPageState>>,
    version: Arc<AtomicU64>,
    cx: &mut App,
) {
    let trimmed = query.trim().to_string();
    if trimmed.is_empty() {
        let mut s = state.borrow_mut();
        s.results.clear();
        s.is_loading = false;
        s.last_query.clear();
        cx.refresh_windows();
        return;
    }

    let current_v = version.fetch_add(1, Ordering::SeqCst) + 1;
    state.borrow_mut().is_loading = true;
    state.borrow_mut().last_query = trimmed.clone();
    cx.refresh_windows();

    let state_clone = state.clone();
    let version_clone = version.clone();
    let query_to_fetch = trimmed;

    cx.spawn(async move |cx: &mut AsyncApp| {
        if debounce_ms > 0 {
            cx.background_executor()
                .timer(Duration::from_millis(debounce_ms))
                .await;
        }

        if version_clone.load(Ordering::SeqCst) != current_v {
            return;
        }

        let results =
            cx.background_executor()
                .spawn(async move {
                    youtube::search_filtered(&query_to_fetch, filter).unwrap_or_default()
                })
                .await;

        if version_clone.load(Ordering::SeqCst) == current_v {
            let _ = cx.update(|cx| {
                let mut s = state_clone.borrow_mut();
                s.results = results;
                s.is_loading = false;
                cx.refresh_windows();
            });
        }
    })
    .detach();
}

impl RenderOnce for SearchPage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let search_state = window.use_keyed_state("search-page-state", cx, |_, _| {
            Rc::new(RefCell::new(SearchPageState::default()))
        });
        let search_version = window.use_keyed_state("search-page-version", cx, |_, _| {
            Arc::new(AtomicU64::new(0))
        });

        let selected_filter =
            window.use_keyed_state("search-page-filter", cx, |_, _| std::cell::Cell::new(0u8));
        let current_filter = selected_filter.read(cx).get();
        let current_search_filter = match current_filter {
            1 => SearchFilter::Songs,
            2 => SearchFilter::Videos,
            _ => SearchFilter::All,
        };

        let filter_setter = selected_filter.clone();
        let state_for_filter = search_state.read(cx).clone();
        let version_for_filter = search_version.read(cx).clone();

        let input = window.use_keyed_state("search-page-input", cx, |window, cx| {
            InputState::new(window, cx)
        });

        let focus_on_mount = window.use_keyed_state("search-page-focus-on-mount", cx, |_, _| {
            std::cell::Cell::new(true)
        });
        let focus_handle = input.read(cx).focus_handle(cx);
        let click_handle = focus_handle.clone();

        if focus_on_mount.read(cx).replace(false) {
            let initial_focus = focus_handle.clone();
            window.defer(cx, move |window, cx| {
                initial_focus.focus(window, cx);
            });
        }

        let query_text = input.read(cx).value().to_string();
        {
            let mut state = search_state.read(cx).borrow_mut();
            if state.last_query != query_text {
                state.last_query = query_text.clone();
                drop(state);
                trigger_search(
                    query_text.clone(),
                    current_search_filter,
                    300,
                    search_state.read(cx).clone(),
                    search_version.read(cx).clone(),
                    cx,
                );
            }
        }

        let (tracks, is_loading) = {
            let s = search_state.read(cx).borrow();
            (s.results.clone(), s.is_loading)
        };

        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .child(
                h_flex()
                    .w_full()
                    .items_center()
                    .justify_between()
                    .child(Label::new("Search").text_3xl())
                    .child(
                        div()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded(cx.theme().radius)
                            .w(px(380.0))
                            .on_mouse_down(MouseButton::Left, move |_event, window, cx| {
                                click_handle.focus(window, cx);
                            })
                            .child(
                                h_flex()
                                    .w_full()
                                    .items_center()
                                    .child(
                                        ButtonGroup::new("toggle-group")
                                            .child(
                                                Button::new("btn_all")
                                                    .ghost()
                                                    .label("All")
                                                    .tooltip("Searches for Youtube content (songs and videos)")
                                                    .selected(current_filter == 0),
                                            )
                                            .child(
                                                Button::new("btn_sng")
                                                    .ghost()
                                                    .label("Songs")
                                                    .tooltip("Searches for Youtube Music songs (likely more official releases)")
                                                    .selected(current_filter == 1),
                                            )
                                            .child(
                                                Button::new("btn_vdo")
                                                    .ghost()
                                                    .label("Video")
                                                    .tooltip("Searches for Youtube videos (likely more unreleased / smaller songs)")
                                                    .rounded(px(0.0))
                                                    .selected(current_filter == 2),
                                            )
                                            .on_click({
                                                let input_entity = input.clone();
                                                move |selected_indices, window, cx| {
                                                    if let Some(first) = selected_indices.first() {
                                                        let f = *first as u8;
                                                        filter_setter.read(cx).set(f);
                                                        let sf = match f {
                                                            1 => SearchFilter::Songs,
                                                            2 => SearchFilter::Videos,
                                                            _ => SearchFilter::All,
                                                        };
                                                        let q = input_entity
                                                            .read(cx)
                                                            .value()
                                                            .to_string();
                                                        trigger_search(
                                                            q,
                                                            sf,
                                                            0,
                                                            state_for_filter.clone(),
                                                            version_for_filter.clone(),
                                                            cx,
                                                        );
                                                        window.refresh();
                                                    }
                                                }
                                            }),
                                    )
                                    .child(
                                        div().w(px(1.0)).h(px(18.0)).ml_2().bg(cx.theme().border),
                                    )
                                    .child(
                                        Input::new(&input)
                                            .flex_1()
                                            .prefix(Icon::new(IconName::Search))
                                            .cleanable(true)
                                            .appearance(false),
                                    ),
                            ),
                    ),
            )
            .child(div().flex_1().w_full().overflow_hidden().child(
                if is_loading && tracks.is_empty() {
                    v_flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(Spinner::new())
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Searching YouTube..."),
                        )
                        .into_any_element()
                } else if tracks.is_empty() && !query_text.is_empty() {
                    v_flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(Icon::new(IconName::Search).size_8())
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("No tracks found."),
                        )
                        .into_any_element()
                } else if !tracks.is_empty() {
                    TrackList::new(tracks).into_any_element()
                } else {
                    v_flex()
                        .size_full()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .child(Icon::new(IconName::Search).size_8())
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("Search for songs, albums, or unreleased tracks"),
                        )
                        .into_any_element()
                },
            ))
            .into_any_element()
    }
}
