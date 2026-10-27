use crate::model::{Config, apply_theme};
use crate::queue::{QueueManager, RepeatMode};
use crate::router::{NavigationHistory, NavigationRoute};
use crate::save_config;
use crate::ui::components::command_palette::actions::all_action_groups;
use crate::ui::components::command_palette::modes::PaletteMode;
use gpui_kit::assets::IconName;
use gpui_kit::base::{StyledExt, h_flex};
use gpui_kit::component::command::{Command, CommandGroup, CommandItem, CommandState};
use gpui_kit::component::tag::Tag;
use gpui_kit::component::{ActiveTheme, Icon, ThemeRegistry, WindowExt, v_flex};
use gpui_kit::*;
use std::cell::Cell;
use std::rc::Rc;
use strum::IntoEnumIterator;

pub mod actions;
pub mod modes;

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

    window.open_dialog(cx, move |dialog, _, cx| {
        let command = command_state.clone();
        let focus_on_mount = focus_on_mount.clone();
        let themes = themes_for_closure.clone();
        let routes = pages_for_closure.clone();

        dialog
            .close_button(false)
            .w(px(480.0))
            .p_0()
            .bg(cx.theme().popover)
            .overflow_hidden()
            .content(move |content, window, cx| {
                if focus_on_mount.replace(false) {
                    let command = command.clone();
                    window.defer(cx, move |window, cx| {
                        command.read(cx).focus_handle(cx).focus(window, cx);
                    });
                }

                let raw_query = command.read(cx).query(cx).to_string();
                let mode = PaletteMode::parse(&raw_query);

                let mut cmd = Command::new(&command)
                    .bordered(false)
                    .bg(gpui::transparent_black())
                    .rounded(cx.theme().radius_lg)
                    .w(px(480.0))
                    .on_query(|_query, window, _cx| {
                        window.refresh();
                    })
                    .placeholder("Type a command ('ct ' for themes, 'nav ' for pages)...")
                    .header(|state, _, cx| {
                        h_flex()
                            .h_6()
                            .justify_between()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child("Commands")
                            .child(format!("{} matches", state.matched_count()))
                    })
                    .footer(|_, _, cx| {
                        h_flex()
                            .h_7()
                            .px_3()
                            .py_2()
                            .text_xs()
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .justify_between()
                            .child(
                                div()
                                    .h_flex()
                                    .gap_1()
                                    .child(Tag::secondary().px_1().py_0p5().child("↑↓"))
                                    .child("Navigate"),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .gap_3()
                                    .child(
                                        div()
                                            .h_flex()
                                            .gap_1()
                                            .child(Tag::secondary().px_1().py_0p5().child("Enter"))
                                            .child("Select"),
                                    )
                                    .child(
                                        div()
                                            .h_flex()
                                            .gap_1()
                                            .child(Tag::secondary().px_1().py_0p5().child("Escape"))
                                            .child("Close"),
                                    ),
                            )
                    });

                cmd = match mode {
                    PaletteMode::Themes(search) => configure_theme_mode(cmd, search, &themes, cx),
                    PaletteMode::Navigation(search) => configure_nav_mode(cmd, search, &routes, cx),
                    PaletteMode::RepeatMode(search) => configure_repeat_mode(cmd, search, cx),
                    PaletteMode::Actions => configure_action_mode(cmd, command.clone(), cx),
                };

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

fn configure_theme_mode(
    cmd: Command,
    search: &str,
    themes: &[SharedString],
    cx: &mut App,
) -> Command {
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

    cmd.filterable(false)
        .group(theme_group)
        .on_confirm(move |index_path, _window, cx| {
            if let Some(target_theme) = themes_confirm.get(index_path.row) {
                if let Some(theme) = ThemeRegistry::global(cx)
                    .themes()
                    .get(target_theme)
                    .cloned()
                {
                    apply_theme(&theme, cx);
                    cx.update_global::<Config, _>(|config, _| {
                        config.theme_name = target_theme.clone();
                        let _ = save_config(config);
                    });
                }
            }
        })
}

fn configure_nav_mode(
    cmd: Command,
    search: &str,
    routes: &[NavigationRoute],
    cx: &mut App,
) -> Command {
    let filtered_routes: Vec<NavigationRoute> = routes
        .iter()
        .filter(|r| search.is_empty() || r.meta().name.to_lowercase().contains(search))
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
    cmd.filterable(false)
        .group(nav_group)
        .on_confirm(move |index_path, window, cx| {
            if let Some(route) = routes.get(index_path.row) {
                cx.update_global::<NavigationHistory, _>(|history, _cx| {
                    history.push(route.clone());
                })
            }
            window.close_dialog(cx);
        })
}

fn configure_action_mode(mut cmd: Command, command: Entity<CommandState>, cx: &App) -> Command {
    let groups = all_action_groups(cx);

    for (group_label, actions) in &groups {
        let mut cmd_group = CommandGroup::new().label(*group_label);
        for action in actions {
            let label = action.label;
            let icon = action.icon;
            let keywords = action.keywords.join(", ");
            cmd_group = cmd_group.item(
                CommandItem::new()
                    .keywords(action.keywords.iter().copied())
                    .child({
                        let keywords = keywords.clone();
                        move |_window, cx| {
                            h_flex()
                                .w_full()
                                .justify_between()
                                .items_center()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(Icon::new(icon).size_4())
                                        .child(label),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(
                                            Tag::secondary()
                                                .px_1()
                                                .py_0p5()
                                                .child(keywords.clone()),
                                        ),
                                )
                        }
                    }),
            );
        }
        cmd = cmd.group(cmd_group);
    }

    cmd.on_confirm(move |index_path, window, cx| {
        if let Some((_, actions)) = groups.get(index_path.section) {
            if let Some(action) = actions.get(index_path.row) {
                (action.execute)(&command, window, cx);
            }
        }
    })
}

fn configure_repeat_mode(cmd: Command, search: &str, cx: &mut App) -> Command {
    let current_repeat_mode = cx.global::<QueueManager>().repeat_mode;
    let all_repeat_modes = RepeatMode::iter().collect::<Vec<RepeatMode>>();

    let mut repeat_group = CommandGroup::new().label("Repeat Modes");
    for mode in all_repeat_modes.clone() {
        repeat_group = repeat_group.item(
            CommandItem::new()
                .label(mode.title())
                .icon(mode.icon())
                .checked(mode == current_repeat_mode),
        );
    }

    let filtered_modes: Vec<RepeatMode> = all_repeat_modes
        .iter()
        .filter(|t| search.is_empty() || t.title().to_lowercase().contains(search))
        .cloned()
        .collect();

    let modes_confirm = filtered_modes.clone();

    cmd.filterable(false)
        .group(repeat_group)
        .on_confirm(move |index_path, _window, cx| {
            if let Some(target_mode) = modes_confirm.get(index_path.row) {
                cx.update_global::<QueueManager, _>(|qm, _cx| qm.repeat_mode = *target_mode);
            }
        })
}
