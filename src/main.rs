use crate::audio::init_audio;
use crate::model::{AudioPlayer, Config, CurrentTrackList};
use crate::router::{NavigationHistory, NavigationRoute};
use crate::ui::components::command_palette;
use crate::ui::components::player_bar::PlayerBar;
use crate::ui::components::sidebar::AppSidebar;
use crate::ui::pages::home_page::HomePage;
use crate::ui::pages::local_library::LocalLibraryPage;
use crate::ui::pages::settings_page::SettingsPage;
use crate::ui::pages::test_page::TestPage;
use gpui_kit::assets::AllAssets;
use gpui_kit::component::TitleBar;
use gpui_kit::component::*;
use gpui_kit::*;
use std::fs::{self};
use std::path::PathBuf;

pub mod audio;
pub mod library;
pub mod model;
pub mod router;
pub mod ui;

actions!(nav, [GoBack, GoForward]);
actions!(cmd, [OpenCommandField]);
actions!(sidebar, [ToggleSidebar]);
actions!(playback, [ResumePause]);

pub struct MusicApp {
    sidebar: Entity<AppSidebar>,
    focus_handle: FocusHandle,
}

impl MusicApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<NavigationHistory>(|_this, cx| {
            let route = cx.global::<NavigationHistory>().current_route().clone();

            match route {
                NavigationRoute::LocalLibrary => {
                    library::load_local_library(cx);
                }
                _ => {}
            }
            cx.notify();
        })
        .detach();

        let sidebar = cx.new(AppSidebar::new);
        let focus_handle = cx.focus_handle();

        Self {
            sidebar,
            focus_handle,
        }
    }
}

impl Render for MusicApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_route = cx.global::<NavigationHistory>().current_route().clone();

        div()
            .track_focus(&self.focus_handle)
            // Nav with Mouse Buttons
            .capture_any_mouse_down(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                window.focus(&this.focus_handle, cx);
                match event.button {
                    MouseButton::Navigate(NavigationDirection::Back) => {
                        cx.update_global::<NavigationHistory, _>(|history, _cx| history.go_back());
                        cx.stop_propagation();
                    }
                    MouseButton::Navigate(NavigationDirection::Forward) => {
                        cx.update_global::<NavigationHistory, _>(|history, _cx| {
                            history.go_forward()
                        });
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }))
            // Key Bindings Handler
            .on_action(cx.listener(|_this, _action: &GoBack, _window, cx| {
                cx.update_global::<NavigationHistory, _>(|history, _cx| history.go_back());
            }))
            .on_action(cx.listener(|_this, _action: &GoForward, _window, cx| {
                cx.update_global::<NavigationHistory, _>(|history, _cx| history.go_forward());
            }))
            .on_action(cx.listener(|this, _action: &ToggleSidebar, _window, cx| {
                this.sidebar.update(cx, |sidebar, cx| {
                    sidebar.toggle(cx);
                });
            }))
            .on_action(
                cx.listener(|_this, _action: &OpenCommandField, window, cx| {
                    command_palette::open(window, cx);
                }),
            )
            .on_action(cx.listener(|_this, _action: &ResumePause, _window, cx| {
                cx.update_global::<AudioPlayer, _>(|audio_player, _cx| {
                    let _ = audio_player.toggle_play();
                })
            }))
            .v_flex()
            .size_full()
            .child(TitleBar::new().child("Music Player"))
            .child(
                div()
                    .flex()
                    .w_full()
                    .h_full()
                    .child(self.sidebar.clone())
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .items_center()
                            .content_center()
                            .justify_center()
                            .child(match current_route {
                                NavigationRoute::Home => HomePage::new().into_any_element(),
                                NavigationRoute::LocalLibrary => {
                                    LocalLibraryPage::new().into_any_element()
                                }
                                NavigationRoute::TestPage { message } => {
                                    TestPage::new(message).into_any_element()
                                }
                                NavigationRoute::Settings => SettingsPage::new().into_any_element(),
                            }),
                    ),
            )
            .child(PlayerBar::new())
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    let cmd_sender = init_audio();

    let audio_player = AudioPlayer::new(cmd_sender);
    // GPUI
    gpui_kit::application()
        .with_assets(AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);

            cx.bind_keys([
                KeyBinding::new("alt-left", GoBack, None),
                KeyBinding::new("alt-right", GoForward, None),
                KeyBinding::new("ctrl-b", ToggleSidebar, None),
                KeyBinding::new("cmd-b", ToggleSidebar, None),
                KeyBinding::new("ctrl-p", OpenCommandField, None),
                KeyBinding::new("ctrl-shift-p", OpenCommandField, None),
                KeyBinding::new("cmd-p", OpenCommandField, None),
                KeyBinding::new("cmd-shift-p", OpenCommandField, None),
                KeyBinding::new("space", ResumePause, None),
            ]);

            cx.set_global(NavigationHistory::new(NavigationRoute::Home));
            cx.set_global(CurrentTrackList::new());
            cx.set_global(audio_player);
            init_config(cx);

            // Save Settings on quit
            cx.on_app_quit(|cx| {
                let mut config = cx.global::<Config>().clone();
                config.theme_name = cx.theme().theme_name().clone();

                async move {
                    save_config(&config);
                }
            })
            .detach();

            gpui_kit::open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    window_bounds: Some(WindowBounds::centered(size(px(1270.0), px(720.0)), cx)),
                    app_owns_titlebar_drag: true,
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    // Theme
                    let config = load_config();
                    if let Err(err) =
                        ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
                            if let Some(theme) = ThemeRegistry::global(cx)
                                .themes()
                                .get(&config.theme_name)
                                .cloned()
                            {
                                Theme::update(cx, |current| current.apply_config(&theme));
                            }
                        })
                    {
                        tracing::error!(?err, "Failed to watch themes directory")
                    }

                    let app = cx.new(MusicApp::new);
                    let focus_handle = app.read(cx).focus_handle.clone();
                    window.focus(&focus_handle, cx);
                    app
                },
            )
            .expect("Failed to open window");
        });
}

fn init_config(cx: &mut App) {
    let config = load_config();
    cx.set_global(config);
}

fn save_config(config: &Config) {
    if let Ok(serialized) = ron::ser::to_string_pretty(config, ron::ser::PrettyConfig::default()) {
        let _ = fs::write(get_config_path(), serialized);
    }
}

fn load_config() -> Config {
    fs::read_to_string(get_config_path())
        .ok()
        .and_then(|content| ron::from_str::<Config>(&content).ok())
        .unwrap_or_else(Config::default)
}

fn get_path() -> PathBuf {
    let path = dirs::config_dir()
        .and_then(|p| Some(p.join("GamingChelsea").join("MusicPlayer")))
        .expect("Error crating path");
    let _ = fs::create_dir_all(path.clone()).expect("Error creating directories");
    path
}

fn get_config_path() -> PathBuf {
    get_path().join("config.ron")
}
