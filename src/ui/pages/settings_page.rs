use gpui_kit::base::h_flex;
use gpui_kit::base::input::InputState;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::Button;
use gpui_kit::component::input::Input;
use gpui_kit::component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{ActiveTheme, Theme, ThemeRegistry};
use gpui_kit::*;
use rfd::FileDialog;

use crate::model::{AudioPlayer, Config, DiscordRpc};
use crate::save_config;

#[derive(IntoElement)]
pub struct SettingsPage;

impl SettingsPage {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for SettingsPage {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let themes: Vec<(SharedString, SharedString)> = ThemeRegistry::global(cx)
            .sorted_themes()
            .into_iter()
            .map(|theme| (theme.name.clone(), theme.name.clone()))
            .collect();

        let current_theme_name = cx.theme().theme_name().clone();
        let config = cx.global::<Config>();
        let initial_music_dir = config
            .music_dir
            .clone()
            .into_string()
            .unwrap_or(String::from("Not found"));
        let input = window.use_keyed_state("music-directory-input", cx, move |window, cx| {
            InputState::new(window, cx).default_value(initial_music_dir)
        });

        Settings::new("app-settings").pages(vec![
            SettingPage::new("General")
                .default_open(true)
                .group(
                    SettingGroup::new()
                        .title("Appearance")
                        .item(
                            SettingItem::new(
                                "Theme",
                                SettingField::scrollable_dropdown(
                                    themes,
                                    move |_| current_theme_name.clone(),
                                    |val: SharedString, cx: &mut App| {
                                        if let Some(theme) =
                                            ThemeRegistry::global(cx).themes().get(&val).cloned()
                                        {
                                            Theme::update(cx, |current| current.apply_config(&theme));
                                            cx.update_global::<Config, _>(|config, _cx| {
                                                config.theme_name = SharedString::from(&*theme.name);
                                                let _ = save_config(config);
                                            });
                                        }
                                    },
                                ),
                            )
                            .description("Select a nice theme."),
                        )
                        .item(
                            SettingItem::new(
                                "Shorten Title",
                                SettingField::switch(
                                    |cx: &App| cx.global::<Config>().shorten_titles,
                                    |val: bool, cx: &mut App| {
                                        cx.update_global::<Config, _>(|config, _cx| {
                                            config.shorten_titles = val;
                                            let _ = save_config(config);
                                        });
                                    },
                                ),
                            )
                            .description("If this option is selected the title are shortened, which may result in unexpected behavior."),
                        ),
                ).group(
                    SettingGroup::new()
                        .title("Connections")
                        .item(
                            SettingItem::new(
                                "Discord RPC",
                                SettingField::switch(
                                    |cx: &App| cx.global::<Config>().discord_rpc,
                                    |val: bool, cx: &mut App| {
                                        cx.update_global::<Config, _>(|config, _cx| {
                                            config.discord_rpc = val;
                                            let _ = save_config(config);
                                        });
                                        cx.global::<DiscordRpc>().set_enabled(val);
                                        if val {
                                            let (current_track, is_playing) =
                                                cx.read_global::<AudioPlayer, _>(|player, _| {
                                                    (player.current_track.clone(), player.is_playing)
                                                });
                                            if let Some(track) = current_track {
                                                if is_playing {
                                                    cx.global::<DiscordRpc>().update_track(&track);
                                                } else {
                                                    cx.global::<DiscordRpc>().set_paused(&track);
                                                }
                                            } else {
                                                cx.global::<DiscordRpc>().set_idle();
                                            }
                                        }
                                    },
                                ),
                            )
                            .description("Toggle to display Heliox as an Discord activity."),
                        ),
                )
                .group(
                    SettingGroup::new().title("Files").item(
                        SettingItem::new(
                            "Music Directory",
                            SettingField::render(move |options, _, _| {
                                let input = input.clone();
                                h_flex()
                                    .w_full()
                                    .justify_between()
                                    .gap_1()
                                    .child(
                                        div()
                                            .id("music-directory-tooltip")
                                            .tooltip({
                                                let input = input.clone();
                                                move |window, cx| {
                                                    let val = input.read(cx).value();
                                                    Tooltip::new(val).build(window, cx)
                                                }
                                            })
                                            .child(
                                                Input::new(&input).readonly(true).w_40().truncate(),
                                            ),
                                    )
                                    .child(
                                        Button::new("path")
                                            .label("Switch")
                                            .with_size(options.size())
                                            .on_click({
                                                let input = input.clone();
                                                move |_event, window, cx| {
                                                    let input = input.clone();
                                                    let window_handle = window.window_handle();
                                                    cx.spawn(async move |cx: &mut AsyncApp| {
                                                        let path = cx
                                                            .background_executor()
                                                            .spawn(async move {
                                                                FileDialog::new().pick_folder()
                                                            })
                                                            .await;

                                                        if let Some(path) = path {
                                                            let path_str =
                                                                path.display().to_string();
                                                            let _ = window_handle.update(
                                                                cx,
                                                                |_, window, cx| {
                                                                    input.update(
                                                                        cx,
                                                                        |input, cx| {
                                                                            input.set_value(
                                                                                path_str.clone(),
                                                                                window,
                                                                                cx,
                                                                            );
                                                                        },
                                                                    );
                                                                    cx.update_global::<Config, _>(
                                                                        |config, _cx| {
                                                                            config.music_dir =
                                                                                path_str.into();
                                                                            let _ =
                                                                                save_config(config);
                                                                        },
                                                                    );
                                                                },
                                                            );
                                                        }
                                                    })
                                                    .detach();
                                                }
                                            }),
                                    )
                                    .into_any_element()
                            }),
                        )
                        .description("Path to your local Music location."),
                    ),
                ),
        ])
    }
}
