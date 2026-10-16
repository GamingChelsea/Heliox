use crate::router::{NavigationHistory, NavigationRoute};
use gpui_kit::assets::IconName;
use gpui_kit::base::h_flex;
use gpui_kit::component::Icon;
use gpui_kit::component::sidebar::{
    Sidebar, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem,
};
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub struct AppSidebar {
    collapsed: bool,
}
impl AppSidebar {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { collapsed: false }
    }
}

impl Render for AppSidebar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_route = cx.global::<NavigationHistory>().current_route().clone();

        let toggle_icon = if self.collapsed {
            IconName::PanelLeftOpen
        } else {
            IconName::PanelLeftClose
        };

        Sidebar::new("app-sidebar")
            .collapsed(self.collapsed)
            .collapsible(true)
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::Music))
                        .when(!self.collapsed, |this| this.child("Music Player")),
                ),
            )
            .child(
                SidebarGroup::new("Navigation").child(
                    SidebarMenu::new()
                        .child(
                            SidebarMenuItem::new("Home")
                                .icon(IconName::House)
                                .active(matches!(current_route, NavigationRoute::Home))
                                .on_click(|_event, _window, cx| {
                                    cx.update_global::<NavigationHistory, _>(|history, _cx| {
                                        history.push(NavigationRoute::Home);
                                    });
                                }),
                        )
                        .child(
                            SidebarMenuItem::new("TestPage")
                                .icon(IconName::TestTube)
                                .active(matches!(current_route, NavigationRoute::TestPage { .. }))
                                .on_click(|_event, _window, cx| {
                                    cx.update_global::<NavigationHistory, _>(|history, _cx| {
                                        history.push(NavigationRoute::TestPage {
                                            message: "hello from sidebar".into(),
                                        });
                                    });
                                }),
                        )
                        .child(
                            SidebarMenuItem::new("Settings")
                                .icon(IconName::Settings)
                                .on_click(|_event, _window, cx| {
                                    cx.update_global::<NavigationHistory, _>(|history, _cx| {
                                        history.push(NavigationRoute::Settings);
                                    });
                                }),
                        ),
                ),
            )
            .footer(
                SidebarFooter::new().child(
                    h_flex()
                        .id("collapse-footer")
                        .w_full()
                        .gap_2()
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.collapsed = !this.collapsed;
                            cx.notify();
                        }))
                        .child(Icon::new(toggle_icon))
                        .when(!self.collapsed, |this| this.child("Collapse")),
                ),
            )
    }
}
