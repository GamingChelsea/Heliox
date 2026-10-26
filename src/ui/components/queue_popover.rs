use crate::{
    model::{AudioPlayer, Config, Track},
    player::play_track,
    queue::{QueueManager, QueueTab},
    ui::components::cover::{cover_art, cover_art_sized},
};
use gpui_kit::{
    assets::IconName,
    base::{h_flex, v_flex},
    component::{
        ActiveTheme,
        scroll::{Scrollbar, ScrollbarMode},
        tab::{Tab, TabBar},
    },
    prelude::FluentBuilder,
    *,
};

#[derive(Clone, Copy, Debug)]
pub struct DraggedQueueIndex(pub usize);

pub struct DragPreview {
    title: String,
}

impl Render for DragPreview {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius)
            .shadow_md()
            .text_xs()
            .font_weight(gpui_kit::FontWeight::SEMIBOLD)
            .child(self.title.clone())
    }
}

#[derive(IntoElement)]
pub struct QueuePopover;

impl QueuePopover {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for QueuePopover {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let queue_scroll = window
            .use_keyed_state("queue-scroll-handle", cx, |_, _| ScrollHandle::default())
            .read(cx)
            .clone();
        let history_scroll = window
            .use_keyed_state("history-scroll-handle", cx, |_, _| ScrollHandle::default())
            .read(cx)
            .clone();
        let current_track = cx.global::<AudioPlayer>().current_track.clone();
        let queue_items = cx.global::<QueueManager>().queue.clone();
        let active_tab = cx.global::<QueueManager>().active_tab;
        let shorten = cx.global::<Config>().shorten_titles;
        let index_tab = match active_tab {
            QueueTab::Queue => 0,
            QueueTab::History => 1,
        };

        v_flex()
            .w(px(390.0))
            .min_h(px(260.0))
            .gap_3()
            // Tab Bar
            .child(
                TabBar::new("queue-tab")
                    .underline()
                    .selected_index(index_tab)
                    .on_click(|ix, window, cx| {
                        let new_tab = match *ix {
                            0 => QueueTab::Queue,
                            _ => QueueTab::History,
                        };
                        cx.update_global::<QueueManager, _>(|qm, _cx| {
                            qm.active_tab = new_tab;
                        });
                        window.refresh();
                    })
                    .child(Tab::new().label("Queue"))
                    .child(Tab::new().label("History")),
            )
            // Main Content
            .child(match active_tab {
                QueueTab::Queue => v_flex()
                    .gap_2()
                    .when_some(current_track, |this, track| {
                        this.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("Currently playing"),
                        )
                        .child(
                            h_flex()
                                .w_full()
                                .p_2()
                                .gap_2()
                                .items_center()
                                .bg(cx.theme().secondary)
                                .rounded(cx.theme().radius)
                                .child(cover_art(cx))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_w_0()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                                .truncate()
                                                .child(track.display_title(shorten)),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .truncate()
                                                .child(
                                                    if track.album.is_empty()
                                                        || track.album == track.title
                                                    {
                                                        track.artist.clone()
                                                    } else {
                                                        format!(
                                                            "{} • {}",
                                                            track.artist, track.album
                                                        )
                                                    },
                                                ),
                                        ),
                                ),
                        )
                    })
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Next in queue"),
                    )
                    .child(if queue_items.is_empty() {
                        div()
                            .p_3()
                            .text_xs()
                            .text_center()
                            .text_color(cx.theme().muted_foreground)
                            .child("No tracks in queue")
                            .into_any_element()
                    } else {
                        scroll_list(
                            &queue_scroll,
                            "queue-items-scroll",
                            px(260.0),
                            queue_items.iter().enumerate().map(|(idx, item)| {
                                let track = item.track.clone();
                                let track_title = track.display_title(shorten);
                                let track_sub =
                                    if track.album.is_empty() || track.album == track.title {
                                        track.artist.clone()
                                    } else {
                                        format!("{} • {}", track.artist, track.album)
                                    };
                                let preview_title = track_title.clone();

                                h_flex()
                                    .id(ElementId::NamedInteger("queue-item".into(), item.id))
                                    .w_full()
                                    .px_2()
                                    .py(px(4.0))
                                    .gap_2()
                                    .items_center()
                                    .rounded(cx.theme().radius)
                                    .hover(|s| s.bg(cx.theme().accent.opacity(0.3)))
                                    .drag_over::<DraggedQueueIndex>(|style, _data, _window, cx| {
                                        style
                                            .bg(cx.theme().accent.opacity(0.5))
                                            .border_t_2()
                                            .border_color(cx.theme().primary)
                                    })
                                    .on_drop::<DraggedQueueIndex>(move |dragged, window, cx| {
                                        let from = dragged.0;
                                        let to = idx;
                                        cx.update_global::<QueueManager, _>(|qm, _cx| {
                                            qm.move_item(from, to);
                                        });
                                        window.refresh();
                                    })
                                    .child(
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "queue-drag-grip".into(),
                                                item.id,
                                            ))
                                            .cursor_grab()
                                            .text_color(cx.theme().muted_foreground)
                                            .hover(|s| s.text_color(cx.theme().foreground))
                                            .on_drag(DraggedQueueIndex(idx), {
                                                let title = preview_title.clone();
                                                move |_dragged, _pos, _window, cx| {
                                                    cx.new(|_cx| DragPreview {
                                                        title: title.clone(),
                                                    })
                                                }
                                            })
                                            .child(IconName::GripVertical),
                                    )
                                    .child(cover_art_sized(
                                        ElementId::NamedInteger("queue-thumb".into(), item.id),
                                        px(36.0),
                                        cx,
                                    ))
                                    .child(
                                        v_flex()
                                            .id(ElementId::NamedInteger(
                                                "queue-track-click".into(),
                                                item.id,
                                            ))
                                            .flex_1()
                                            .min_w_0()
                                            .cursor_pointer()
                                            .on_click({
                                                let track_clone = track.clone();
                                                move |_event, _window, cx| {
                                                    let curr = cx
                                                        .global::<AudioPlayer>()
                                                        .current_track
                                                        .clone();
                                                    let to_play = cx
                                                        .update_global::<QueueManager, _>(
                                                            |qm, _cx| qm.play_queue_at(idx, curr),
                                                        );
                                                    if let Some(t) = to_play {
                                                        play_track(t, cx);
                                                    } else {
                                                        play_track(track_clone.clone(), cx);
                                                    }
                                                }
                                            })
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                                    .truncate()
                                                    .child(track_title),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .truncate()
                                                    .child(track_sub),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .id(ElementId::NamedInteger(
                                                "queue-remove-btn".into(),
                                                item.id,
                                            ))
                                            .cursor_pointer()
                                            .text_color(cx.theme().muted_foreground)
                                            .hover(|s| s.text_color(cx.theme().foreground))
                                            .on_click(move |_event, window, cx| {
                                                cx.update_global::<QueueManager, _>(|qm, _cx| {
                                                    qm.remove_at(idx);
                                                });
                                                window.refresh();
                                            })
                                            .child(IconName::X),
                                    )
                            }),
                        )
                    })
                    .into_any_element(),
                QueueTab::History => {
                    let history_tracks: Vec<Track> = cx
                        .global::<QueueManager>()
                        .history
                        .iter()
                        .rev()
                        .cloned()
                        .collect();

                    if history_tracks.is_empty() {
                        div()
                            .p_4()
                            .text_xs()
                            .text_center()
                            .text_color(cx.theme().muted_foreground)
                            .child("No listening history yet")
                            .into_any_element()
                    } else {
                        scroll_list(
                            &history_scroll,
                            "history-items-scroll",
                            px(300.0),
                            history_tracks.into_iter().enumerate().map(|(idx, track)| {
                                let track_title = track.display_title(shorten);
                                let track_sub =
                                    if track.album.is_empty() || track.album == track.title {
                                        track.artist.clone()
                                    } else {
                                        format!("{} • {}", track.artist, track.album)
                                    };
                                let track_clone = track.clone();

                                h_flex()
                                    .id(ElementId::NamedInteger(
                                        "history-item".into(),
                                        track.id ^ (idx as u64),
                                    ))
                                    .w_full()
                                    .px_2()
                                    .py(px(4.0))
                                    .gap_2()
                                    .items_center()
                                    .rounded(cx.theme().radius)
                                    .cursor_pointer()
                                    .hover(|s| s.bg(cx.theme().accent.opacity(0.3)))
                                    .on_click(move |_event, _window, cx| {
                                        play_track(track_clone.clone(), cx);
                                    })
                                    .child(cover_art_sized(
                                        ElementId::NamedInteger(
                                            "history-thumb".into(),
                                            track.id ^ (idx as u64),
                                        ),
                                        px(36.0),
                                        cx,
                                    ))
                                    .child(
                                        v_flex()
                                            .flex_1()
                                            .min_w_0()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                                                    .truncate()
                                                    .child(track_title),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .truncate()
                                                    .child(track_sub),
                                            ),
                                    )
                            }),
                        )
                    }
                }
            })
    }
}

fn scroll_list(
    handle: &ScrollHandle,
    id: &'static str,
    max_h: Pixels,
    children: impl IntoIterator<Item = impl IntoElement>,
) -> AnyElement {
    div()
        .relative()
        .w_full()
        .child(
            v_flex()
                .id(id)
                .w_full()
                .max_h(max_h)
                .gap_1()
                .pr_3()
                .overflow_y_scroll()
                .track_scroll(handle)
                .children(children),
        )
        .child(
            div().absolute().inset_0().child(
                Scrollbar::vertical(handle)
                    .id(SharedString::from(format!("{}-bar", id)))
                    .mode(ScrollbarMode::Hover)
                    .viewport_from_layout(),
            ),
        )
        .into_any_element()
}
