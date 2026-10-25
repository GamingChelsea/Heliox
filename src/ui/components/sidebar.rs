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

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
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
            .when(!self.collapsed, |this| this.w_40())
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .gap_2()
                        .child(Icon::new(IconName::Music))
                        .when(!self.collapsed, |this| this.child("Heliox")),
                ),
            )
            .child(
                SidebarGroup::new("Navigation").child(
                    SidebarMenu::new()
                        .children(
                            NavigationRoute::get_clean_routes()
                                .into_iter()
                                .map(|route| {
                                    let meta = route.meta();
                                    let is_current = current_route == route;
                                    SidebarMenuItem::new(meta.name)
                                        .icon(meta.icon)
                                        .active(is_current)
                                        .on_click(move |_event, _window, cx| {
                                            let route = route.clone();
                                            cx.update_global::<NavigationHistory, _>(
                                                move |history, _cx| {
                                                    history.push(route.clone());
                                                },
                                            );
                                        })
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
                            this.toggle(cx);
                        }))
                        .child(Icon::new(toggle_icon))
                        .when(!self.collapsed, |this| this.child("Collapse")),
                ),
            )
    }
}
