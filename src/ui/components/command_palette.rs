use crate::model::Config;
use crate::router::{NavigationHistory, NavigationRoute};
use crate::save_config;
use gpui_kit::assets::IconName;
use gpui_kit::component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::{ActiveTheme, Icon, Theme, ThemeRegistry, WindowExt, v_flex};
use gpui_kit::*;
use std::cell::Cell;
use std::rc::Rc;

pub fn open(window: &mut Window, cx: &mut App) {
    let command_state = cx.new(|cx| CommandState::new(window, cx));
    let focus_on_mount = Rc::new(Cell::new(true));

    let available_themes: Vec<SharedString> = ThemeRegistry::global(cx)
        .sorted_themes()
        .into_iter()
        .map(|t| t.name.clone())
        .collect();

    let themes_for_closure = available_themes.clone();

    let available_pages = NavigationRoute::get_clean_routes();
    let pages_for_closure = available_pages.clone();

    window.open_dialog(cx, move |dialog, _, _| {
        let command = command_state.clone();
        let focus_on_mount = focus_on_mount.clone();
        let themes = themes_for_closure.clone();
        let routes = pages_for_closure.clone();

        dialog
            .close_button(false)
            .w(px(480.0))
            .p_0()
            .content(move |content, window, cx| {
                if focus_on_mount.replace(false) {
                    let command = command.clone();
                    window.defer(cx, move |window, cx| {
                        command.read(cx).focus_handle(cx).focus(window, cx);
                    });
                }

                let raw_query = command.read(cx).query(cx).to_string();
                let trimmed = raw_query.trim_start().to_lowercase();

                let is_theme_mode = trimmed.starts_with("ct");
                let is_nav_mode = trimmed.starts_with("nav ");

                let mut cmd = Command::new(&command)
                    .bordered(false)
                    .w(px(480.0))
                    .on_query(|_query, window, _cx| {
                        window.refresh();
                    })
                    .placeholder("Type a command ('ct ' for themes, 'nav ' for pages)...");

                if is_theme_mode {
                    let search = trimmed.strip_prefix("ct").unwrap_or("").trim();
                    let current_theme_name = cx.theme().theme_name();

                    let filtered_themes: Vec<SharedString> = themes
                        .iter()
                        .filter(|t| search.is_empty() || t.to_lowercase().contains(search))
                        .cloned()
                        .collect();

                    let mut theme_group = CommandGroup::new().label("Themes");
                    for theme_name in &filtered_themes {
                        theme_group = theme_group.item(
                            CommandItem::new()
                                .label(theme_name.clone())
                                .icon(IconName::Palette)
                                .checked(theme_name == current_theme_name),
                        );
                    }

                    let themes_confirm = filtered_themes.clone();
                    cmd = cmd.filterable(false).group(theme_group).on_confirm(
                        move |index_path, window, cx| {
                            if let Some(target_theme) = themes_confirm.get(index_path.row) {
                                if let Some(theme) = ThemeRegistry::global(cx)
                                    .themes()
                                    .get(target_theme)
                                    .cloned()
                                {
                                    Theme::update(cx, |t| t.apply_config(&theme));
                                    cx.update_global::<Config, _>(|config, _| {
                                        config.theme_name = target_theme.clone();
                                        let _ = save_config(config);
                                    });
                                }
                            }
                            window.close_dialog(cx);
                        },
                    );
                } else if is_nav_mode {
                    let search = trimmed.strip_prefix("nav").unwrap_or("").trim();

                    let filtered_routes: Vec<NavigationRoute> = routes
                        .iter()
                        .filter(|r| {
                            search.is_empty() || r.meta().name.to_lowercase().contains(search)
                        })
                        .cloned()
                        .collect();

                    let mut nav_group = CommandGroup::new().label("Navigation");
                    for route in &filtered_routes {
                        let route_meta = route.meta();
                        nav_group = nav_group.item(
                            CommandItem::new()
                                .label(route_meta.name)
                                .icon(route_meta.icon)
                                .checked(cx.read_global::<NavigationHistory, _>(|history, _cx| {
                                    history.current == route.clone()
                                })),
                        );
                    }

                    let routes = filtered_routes.clone();
                    cmd = cmd.filterable(false).group(nav_group).on_confirm(
                        move |index_path, window, cx| {
                            if let Some(route) = routes.get(index_path.row) {
                                cx.update_global::<NavigationHistory, _>(|history, _cx| {
                                    history.push(route.clone());
                                })
                            }
                            window.close_dialog(cx);
                        },
                    );
                } else {
                    let nav_group = CommandGroup::new().label("Navigation").item(
                        CommandItem::new()
                            .label("Navigate to another page")
                            .icon(IconName::Navigation)
                            .keywords(["nav", "navigate"]),
                    );

                    let theme_group = CommandGroup::new().label("Themes").item(
                        CommandItem::new()
                            .label("Change Theme")
                            .icon(IconName::Palette)
                            .keywords(["ct", "theme"]),
                    );

                    let command_for_action = command.clone();

                    cmd = cmd.group(nav_group).group(theme_group).on_confirm(
                        move |index_path, window, cx| match index_path.section {
                            0 => {
                                command_for_action.update(cx, |state, cx| {
                                    state.set_query("nav ", window, cx);
                                });
                                window.refresh();
                            }
                            1 => {
                                command_for_action.update(cx, |state, cx| {
                                    state.set_query("ct ", window, cx);
                                });
                                window.refresh();
                            }
                            _ => {
                                window.close_dialog(cx);
                            }
                        },
                    );
                }

                content.child(cmd.empty(|_, _, _| {
                    v_flex()
                        .items_center()
                        .gap_2()
                        .py_4()
                        .child(Icon::new(IconName::Search).size_8())
                        .child("No results found.")
                }))
            })
    });
}
