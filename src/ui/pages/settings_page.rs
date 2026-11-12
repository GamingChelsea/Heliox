use nsh_core::prelude::*;
use rfd::FileDialog;

use crate::{
    discord::toggle_discord_rpc,
    model::Config,
    save_config,
    ui::{
        components::audio_slider::audio_slider_with_step,
        icons::{IconName, icon_elem},
        theme::{AppRadius, Clickable, Finish, Finished, THEMES, colors, select_theme},
    },
};

fn toggle_switch(
    enabled: bool,
    label: &'static str,
    on_toggle: impl Fn() + 'static,
) -> impl IntoElement {
    view()
        .w(px(44.0))
        .h(px(24.0))
        .app_radius(1.5)
        .p(px(2.0))
        .bg(if enabled {
            colors::ACCENT_FILL
        } else {
            colors::TRACK
        })
        .clickable()
        .label(label)
        .on_press(on_toggle)
        .child(
            view()
                .size(px(20.0))
                .app_radius(1.25)
                .bg(if enabled {
                    colors::ON_ACCENT
                } else {
                    colors::KNOB
                })
                .finish(Finish::Knob)
                .map(|v| {
                    if enabled {
                        v.self_end()
                    } else {
                        v.self_start()
                    }
                }),
        )
}

fn setting_row(title: &'static str, desc: &'static str, control: AnyElement) -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .gap(px(18.0))
        .py(px(14.0))
        .child(
            view()
                .flex_1()
                .gap(px(3.0))
                .child(
                    text(title)
                        .text_size(px(14.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(text(desc).text_size(px(12.0)).text_color(colors::MUTED)),
        )
        .child(control)
        .into_element()
}

fn section(title: &'static str, rows: Vec<AnyElement>) -> AnyElement {
    view()
        .gap(px(8.0))
        .child(
            text(title.to_uppercase())
                .text_size(px(11.0))
                .font_weight(FontWeight::BOLD)
                .text_color(colors::FAINT),
        )
        .child(
            view()
                .px(px(16.0))
                .app_radius(1.25)
                .finish(Finish::Card)
                .border_1()
                .border_color(colors::BORDER)
                .children(rows),
        )
        .into_element()
}

fn theme_selector() -> AnyElement {
    dynamic(|| {
        let current = global::<Config>().get().theme_name;
        view()
            .flex_row()
            .gap(px(6.0))
            .children(THEMES.into_iter().map(|theme| {
                let selected = match current.as_str() {
                    "System" => theme == "System",
                    "Light" => theme == "Light",
                    "Default Light" => theme == "Light",
                    _ => theme == "Dark",
                };
                view()
                    .h(px(30.0))
                    .px(px(11.0))
                    .items_center()
                    .justify_center()
                    .app_radius(0.75)
                    .map(|button| {
                        if selected {
                            button.bg(colors::ACCENT_FILL)
                        } else {
                            button
                                .bg(colors::INSET)
                                .border_1()
                                .border_color(colors::BORDER)
                        }
                    })
                    .clickable()
                    .label(theme)
                    .on_press(move || select_theme(theme))
                    .child(
                        text(theme)
                            .text_size(px(12.0))
                            .font_weight(if selected {
                                FontWeight::BOLD
                            } else {
                                FontWeight::MEDIUM
                            })
                            .text_color(if selected {
                                colors::ON_ACCENT
                            } else {
                                colors::MUTED
                            }),
                    )
            }))
            .into_element()
    })
}

fn appearance_rows() -> Vec<AnyElement> {
    vec![
        setting_row(
            "Color scheme",
            "Follow the system, or choose light or dark mode.",
            theme_selector(),
        ),
        view().h(px(1.0)).bg(colors::LINE).into_element(),
        setting_row(
            "Corner radius",
            "Adjust the rounding used by cards and controls.",
            view()
                .flex_row()
                .items_center()
                .w(px(190.0))
                .gap(px(10.0))
                .child(audio_slider_with_step(
                    || global::<Config>().get().corner_radius / 16.0,
                    1.0 / 16.0,
                    |value| {
                        let radius = (value * 16.0).round().clamp(0.0, 16.0);
                        global::<Config>().update(|config| config.corner_radius = radius);
                        save_config(&global::<Config>().get_untracked());
                    },
                ))
                .child(dynamic(|| {
                    text(format!("{:.0} pt", global::<Config>().get().corner_radius))
                        .w(px(42.0))
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                }))
                .into_element(),
        ),
    ]
}

fn playback_rows() -> Vec<AnyElement> {
    vec![
        dynamic(|| {
            let enabled = global::<Config>().get().discord_rpc;
            setting_row(
                "Discord Rich Presence",
                "Show the current track and album in Discord.",
                toggle_switch(enabled, "Discord Rich Presence", toggle_discord_rpc).into_element(),
            )
        }),
        view().h(px(1.0)).bg(colors::LINE).into_element(),
        dynamic(|| {
            let enabled = global::<Config>().get().shorten_titles;
            setting_row(
                "Shorten track titles",
                "Remove common video tags such as “Official Audio”.",
                toggle_switch(enabled, "Shorten track titles", || {
                    global::<Config>().update(|config| {
                        config.shorten_titles = !config.shorten_titles;
                    });
                    save_config(&global::<Config>().get_untracked());
                })
                .into_element(),
            )
        }),
    ]
}

fn library_rows() -> Vec<AnyElement> {
    vec![dynamic(|| {
        let directory = global::<Config>()
            .get()
            .music_dir
            .to_str()
            .unwrap_or("Not set")
            .to_string();
        setting_row(
            "Music directory",
            "Folder scanned for local library tracks.",
            view()
                .flex_row()
                .items_center()
                .gap(px(8.0))
                .child(
                    text(directory)
                        .number_of_lines(1)
                        .max_w(px(240.0))
                        .text_size(px(12.0))
                        .text_color(colors::MUTED),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .h(px(32.0))
                        .px(px(12.0))
                        .gap(px(6.0))
                        .app_radius(0.75)
                        .bg(colors::RAISED)
                        .border_1()
                        .border_color(colors::BORDER)
                        .clickable()
                        .label("Browse music directory")
                        .on_press(|| {
                            if let Some(folder) = FileDialog::new().pick_folder() {
                                global::<Config>().update(|config| config.music_dir = folder);
                                save_config(&global::<Config>().get_untracked());
                                crate::library::reload_local_library();
                            }
                        })
                        .child(icon_elem(IconName::Folder, 14.0, colors::MUTED))
                        .child(
                            text("Browse")
                                .text_size(px(12.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors::TEXT),
                        ),
                )
                .into_element(),
        )
    })]
}

pub fn settings_page() -> impl IntoElement {
    view()
        .flex_1()
        .p(px(24.0))
        .gap(px(20.0))
        .child(
            view()
                .gap(px(4.0))
                .child(
                    text("Settings")
                        .text_size(px(28.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::TEXT),
                )
                .child(
                    text("Configure your Heliox experience and audio preferences.")
                        .text_size(px(13.0))
                        .text_color(colors::MUTED),
                ),
        )
        .child(
            scroll().flex_1().child(
                view()
                    .gap(px(18.0))
                    .child(section("Appearance", appearance_rows()))
                    .child(section("Playback", playback_rows()))
                    .child(section("Library", library_rows())),
            ),
        )
}
