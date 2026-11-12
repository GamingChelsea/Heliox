use nsh_core::prelude::*;

use crate::{
    model::{AudioPlayer, Config},
    player::play_track,
    queue::{QueueManager, QueueTab},
    ui::{
        components::cover::cover_art,
        icons::{IconName, icon_elem},
        theme::{AppRadius, Clickable, Finish, Finished, colors, motion},
    },
};

#[derive(Clone, Copy, Default)]
pub struct QueuePopoverOpen(pub bool);

#[derive(Clone, Copy, Default)]
struct QueuePopoverPresence {
    mounted: bool,
    generation: u64,
}

#[derive(Clone, Copy, Default)]
struct QueuePopoverAnimations {
    panel: Option<Motion>,
    content: Option<Motion>,
    indicator: Option<AnimatedValue>,
}

pub fn toggle_queue_popover() {
    if global::<QueuePopoverOpen>().get_untracked().0 {
        global::<QueuePopoverOpen>().set(QueuePopoverOpen(false));
        global::<QueuePopoverPresence>().update(|presence| {
            presence.generation = presence.generation.wrapping_add(1);
        });
        let generation = global::<QueuePopoverPresence>().get_untracked().generation;

        if let Some(panel) = global::<QueuePopoverAnimations>().get_untracked().panel {
            panel.play(MotionPreset::FadeOutDown);
        }

        if reduce_motion() {
            global::<QueuePopoverPresence>().update(|presence| presence.mounted = false);
        } else {
            set_timeout(std::time::Duration::from_millis(300), move || {
                global::<QueuePopoverPresence>().update(|presence| {
                    if presence.generation == generation
                        && !global::<QueuePopoverOpen>().get_untracked().0
                    {
                        presence.mounted = false;
                    }
                });
            });
        }
    } else {
        global::<QueuePopoverPresence>().update(|presence| {
            presence.mounted = true;
            presence.generation = presence.generation.wrapping_add(1);
        });
        if let Some(panel) = global::<QueuePopoverAnimations>().get_untracked().panel {
            panel.value(MotionChannel::Opacity).set(0.0);
            panel.value(MotionChannel::TranslateY).set(24.0);
            panel.play(MotionPreset::FadeInUp);
        }
        global::<QueuePopoverOpen>().set(QueuePopoverOpen(true));
    }
}

pub fn queue_popover() -> impl IntoElement {
    let panel = motion();
    let content = motion();
    let indicator = animated_value(0.0);
    if global::<QueueManager>().get_untracked().active_tab == QueueTab::History {
        indicator.set(163.0);
    }
    global::<QueuePopoverAnimations>().set(QueuePopoverAnimations {
        panel: Some(panel),
        content: Some(content),
        indicator: Some(indicator),
    });
    dynamic(move || queue_popover_view(panel, content, indicator))
}

fn queue_popover_view(
    panel_motion: Motion,
    content_motion: Motion,
    indicator_motion: AnimatedValue,
) -> impl IntoElement {
    if !global::<QueuePopoverPresence>().get().mounted {
        return view().into_element();
    }

    let is_open = global::<QueuePopoverOpen>().get().0;
    let qm = global::<QueueManager>().get();
    let current_track = global::<AudioPlayer>().get().current_track;
    let shorten = global::<Config>().get().shorten_titles;
    let active_tab = qm.active_tab;

    let tab_button = |title: &'static str, tab: QueueTab| {
        view()
            .flex_1()
            .h(px(32.0))
            .items_center()
            .justify_center()
            .app_radius(0.75)
            .clickable()
            .label(title)
            .on_press(move || {
                select_queue_tab(tab);
            })
            .child(
                text(title)
                    .text_size(px(13.0))
                    .font_weight(if active_tab == tab {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .text_color(move || {
                        if global::<QueueManager>().get().active_tab == tab {
                            colors::TEXT
                        } else {
                            colors::MUTED
                        }
                    }),
            )
    };

    let content = match active_tab {
        QueueTab::Queue => {
            let current_view = current_track.map(|track| {
                let title = track.display_title(shorten);
                view()
                    .p(px(10.0))
                    .app_radius(1.0)
                    .bg(colors::INSET)
                    .mb(px(8.0))
                    .child(
                        text("Now Playing")
                            .text_size(px(11.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::ACCENT)
                            .mb(px(4.0)),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .child(cover_art(40.0))
                            .child(
                                view()
                                    .flex_1()
                                    .gap(px(2.0))
                                    .child(
                                        text(title)
                                            .number_of_lines(1)
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT),
                                    )
                                    .child(
                                        text(track.artist)
                                            .number_of_lines(1)
                                            .text_size(px(12.0))
                                            .text_color(colors::MUTED),
                                    ),
                            ),
                    )
            });

            let items: Vec<_> = qm
                .queue
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let track = item.track.clone();
                    let title = track.display_title(shorten);
                    let track_clone = track.clone();

                    view()
                        .flex_row()
                        .items_center()
                        .h(px(40.0))
                        .px(px(8.0))
                        .gap(px(10.0))
                        .app_radius(0.75)
                        .clickable()
                        .label(title.clone())
                        .on_press(move || {
                            play_track(track_clone.clone());
                        })
                        .child(
                            text((idx + 1).to_string())
                                .w(px(24.0))
                                .text_size(px(12.0))
                                .text_color(colors::FAINT),
                        )
                        .child(
                            view()
                                .flex_1()
                                .gap(px(1.0))
                                .child(
                                    text(title)
                                        .number_of_lines(1)
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::TEXT),
                                )
                                .child(
                                    text(track.artist)
                                        .number_of_lines(1)
                                        .text_size(px(11.0))
                                        .text_color(colors::MUTED),
                                ),
                        )
                })
                .collect();

            view()
                .children(current_view)
                .child(
                    text(format!("Next Up ({})", items.len()))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::MUTED)
                        .mb(px(4.0)),
                )
                .child(scroll().h(px(260.0)).child(view().children(items)))
        }
        QueueTab::History => {
            let items: Vec<_> = qm
                .history
                .iter()
                .rev()
                .enumerate()
                .map(|(idx, track)| {
                    let track_clone = track.clone();
                    let title = track.display_title(shorten);

                    view()
                        .flex_row()
                        .items_center()
                        .h(px(40.0))
                        .px(px(8.0))
                        .gap(px(10.0))
                        .app_radius(0.75)
                        .clickable()
                        .label(title.clone())
                        .on_press(move || {
                            play_track(track_clone.clone());
                        })
                        .child(
                            text((idx + 1).to_string())
                                .w(px(24.0))
                                .text_size(px(12.0))
                                .text_color(colors::FAINT),
                        )
                        .child(
                            view()
                                .flex_1()
                                .gap(px(1.0))
                                .child(
                                    text(title)
                                        .number_of_lines(1)
                                        .text_size(px(13.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::TEXT),
                                )
                                .child(
                                    text(track.artist.clone())
                                        .number_of_lines(1)
                                        .text_size(px(11.0))
                                        .text_color(colors::MUTED),
                                ),
                        )
                })
                .collect();

            view()
                .child(
                    text(format!("History ({})", items.len()))
                        .text_size(px(12.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::MUTED)
                        .mb(px(4.0)),
                )
                .child(scroll().h(px(320.0)).child(view().children(items)))
        }
    };

    // Popover overlay card positioned in bottom right above player bar
    view()
        .absolute()
        .right(px(16.0))
        .bottom(px(88.0))
        .w(px(360.0))
        .p(px(14.0))
        .app_radius(1.5)
        .finish(Finish::Card)
        .border_1()
        .border_color(colors::BORDER)
        .pointer_events(if is_open {
            PointerEvents::Auto
        } else {
            PointerEvents::None
        })
        .bind_motion(&panel_motion)
        // Header
        .child(
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .mb(px(12.0))
                .child(
                    text("Play Queue")
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    view()
                        .size(px(24.0))
                        .items_center()
                        .justify_center()
                        .app_radius(1.5)
                        .clickable()
                        .label("Close queue")
                        .on_press(toggle_queue_popover)
                        .child(icon_elem(IconName::Cross, 14.0, colors::MUTED)),
                ),
        )
        // Tabs
        .child(
            view()
                .relative()
                .flex_row()
                .h(px(36.0))
                .p(px(2.0))
                .bg(colors::INSET)
                .app_radius(1.0)
                .mb(px(12.0))
                .child(
                    view()
                        .absolute()
                        .left(px(2.0))
                        .top(px(2.0))
                        .w(px(163.0))
                        .h(px(32.0))
                        .app_radius(0.75)
                        .finish(Finish::Raised)
                        .bind_transform(nsh_core::TransformKind::TranslateX, &indicator_motion),
                )
                .child(tab_button("Queue", QueueTab::Queue))
                .child(tab_button("History", QueueTab::History)),
        )
        // Content
        .child(view().bind_motion(&content_motion).child(content))
        .into_element()
}

fn select_queue_tab(tab: QueueTab) {
    if global::<QueueManager>().get_untracked().active_tab == tab {
        return;
    }

    let animations = global::<QueuePopoverAnimations>().get_untracked();
    if let Some(indicator) = animations.indicator {
        let target = if tab == QueueTab::Queue { 0.0 } else { 163.0 };
        if reduce_motion() {
            indicator.set(target);
        } else {
            indicator.spring(target, motion::SETTLE);
        }
    }

    if let Some(content) = animations.content {
        content.value(MotionChannel::Opacity).set(0.0);
        content.value(MotionChannel::TranslateY).set(24.0);
    }
    global::<QueueManager>().update(|manager| manager.active_tab = tab);
    if let Some(content) = animations.content {
        content.play(MotionPreset::FadeInUp);
    }
}
