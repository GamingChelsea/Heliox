use nsh_core::prelude::*;

use crate::{
    router::{NavigationHistory, NavigationRoute, navigate_to},
    ui::{
        icons::{IconName, icon_elem},
        theme::{AppRadius, Clickable, colors},
    },
};

#[derive(Clone, Copy, Default)]
pub struct SidebarCollapsed(pub bool);

pub fn toggle_sidebar() {
    global::<SidebarCollapsed>().update(|c| c.0 = !c.0);
}

pub fn app_sidebar() -> impl IntoElement {
    dynamic(app_sidebar_view)
}

fn app_sidebar_view() -> impl IntoElement {
    let collapsed = global::<SidebarCollapsed>().get().0;
    let current_route = global::<NavigationHistory>().get().current.clone();

    let nav_items = [
        (NavigationRoute::Home, "Home", IconName::House),
        (NavigationRoute::Search, "Search", IconName::Search),
        (
            NavigationRoute::LocalLibrary,
            "Local Library",
            IconName::Library,
        ),
        (
            NavigationRoute::TestPage {
                message: "Hello from sidebar".into(),
            },
            "Test Page",
            IconName::TestTube,
        ),
        (NavigationRoute::Settings, "Settings", IconName::Settings),
    ];

    let items = nav_items.into_iter().map(|(route, name, icon_name)| {
        let is_active = match (&current_route, &route) {
            (NavigationRoute::Home, NavigationRoute::Home) => true,
            (NavigationRoute::Search, NavigationRoute::Search) => true,
            (NavigationRoute::LocalLibrary, NavigationRoute::LocalLibrary) => true,
            (NavigationRoute::Settings, NavigationRoute::Settings) => true,
            (NavigationRoute::TestPage { .. }, NavigationRoute::TestPage { .. }) => true,
            _ => false,
        };

        let route_to_nav = route.clone();
        view()
            .flex_row()
            .items_center()
            .h(px(40.0))
            .px(px(if collapsed { 14.0 } else { 12.0 }))
            .gap(px(10.0))
            .app_radius(1.0)
            .map(|v| if is_active { v.bg(colors::RAISED) } else { v })
            .clickable()
            .label(name)
            .on_press(move || {
                navigate_to(route_to_nav.clone());
            })
            .child(icon_elem(
                icon_name,
                20.0,
                if is_active {
                    colors::ACCENT
                } else {
                    colors::MUTED
                },
            ))
            .children(if !collapsed {
                Some(
                    text(name)
                        .number_of_lines(1)
                        .text_size(px(14.0))
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::MEDIUM
                        })
                        .text_color(if is_active {
                            colors::TEXT
                        } else {
                            colors::MUTED
                        }),
                )
            } else {
                None
            })
    });

    let toggle_icon = if collapsed {
        IconName::PanelLeftOpen
    } else {
        IconName::PanelLeftClose
    };

    view()
        .w(px(if collapsed { 64.0 } else { 200.0 }))
        .h_full()
        .bg(colors::INSET)
        .border_r_1()
        .border_r_color(colors::LINE)
        .p(px(10.0))
        // Header
        .child(
            view()
                .flex_row()
                .items_center()
                .h(px(44.0))
                .px(px(if collapsed { 8.0 } else { 6.0 }))
                .gap(px(10.0))
                .mb(px(12.0))
                .child(icon_elem(IconName::Music, 24.0, colors::ACCENT))
                .children(if !collapsed {
                    Some(
                        text("Heliox")
                            .text_size(px(18.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors::TEXT),
                    )
                } else {
                    None
                }),
        )
        // Navigation items
        .child(view().flex_1().gap(px(4.0)).children(items))
        // Footer (Collapse Toggle)
        .child(
            view()
                .flex_row()
                .items_center()
                .h(px(40.0))
                .px(px(if collapsed { 14.0 } else { 12.0 }))
                .gap(px(10.0))
                .app_radius(1.0)
                .clickable()
                .label("Collapse sidebar")
                .on_press(toggle_sidebar)
                .child(icon_elem(toggle_icon, 20.0, colors::MUTED))
                .children(if !collapsed {
                    Some(
                        text("Collapse")
                            .text_size(px(13.0))
                            .text_color(colors::MUTED),
                    )
                } else {
                    None
                }),
        )
}
