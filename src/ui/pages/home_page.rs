use nsh_core::prelude::*;

use crate::{
    router::{NavigationRoute, navigate_to},
    ui::{
        icons::{IconName, icon_elem},
        theme::{AppRadius, Clickable, Finish, Finished, colors},
    },
};

pub fn home_page() -> impl IntoElement {
    let action_card =
        |title: &'static str, desc: &'static str, icon_name: IconName, route: NavigationRoute| {
            view()
                .w(px(200.0))
                .p(px(16.0))
                .app_radius(1.5)
                .finish(Finish::Card)
                .border_1()
                .border_color(colors::BORDER)
                .gap(px(10.0))
                .clickable()
                .label(title)
                .on_press(move || {
                    navigate_to(route.clone());
                })
                .child(
                    view()
                        .size(px(40.0))
                        .app_radius(2.5)
                        .bg(colors::INSET)
                        .items_center()
                        .justify_center()
                        .child(icon_elem(icon_name, 22.0, colors::ACCENT)),
                )
                .child(
                    text(title)
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(text(desc).text_size(px(12.0)).text_color(colors::MUTED))
        };

    view()
        .flex_1()
        .p(px(32.0))
        .gap(px(24.0))
        // Title & Welcome
        .child(
            view()
                .gap(px(6.0))
                .child(
                    text("Welcome to Heliox")
                        .text_size(px(32.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text("Fast, lightweight, native music player built in pure Rust with NSH.")
                        .text_size(px(14.0))
                        .text_color(colors::MUTED),
                ),
        )
        // Quick Action Cards
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .gap(px(16.0))
                .child(action_card(
                    "Local Library",
                    "Browse and play tracks from your local music collection.",
                    IconName::Library,
                    NavigationRoute::LocalLibrary,
                ))
                .child(action_card(
                    "Search Music",
                    "Find and stream tracks and videos directly from YouTube.",
                    IconName::Search,
                    NavigationRoute::Search,
                ))
                .child(action_card(
                    "Settings",
                    "Configure theme, corner radius, and audio directories.",
                    IconName::Settings,
                    NavigationRoute::Settings,
                )),
        )
}
