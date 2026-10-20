use crate::audio::{AudioCommand, init_audio};
use crate::model::{AudioPlayer, Config, CurrentTrackList, Track, TrackListSource};
use crate::router::{NavigationHistory, NavigationRoute};
use crate::ui::components::command_palette;
use crate::ui::components::sidebar::AppSidebar;
use crate::ui::pages::home_page::HomePage;
use crate::ui::pages::local_library::LocalLibraryPage;
use crate::ui::pages::settings_page::SettingsPage;
use crate::ui::pages::test_page::TestPage;
use gpui_kit::assets::AllAssets;
use gpui_kit::component::TitleBar;
use gpui_kit::component::*;
use gpui_kit::*;
use lofty::config::ParseOptions;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::Accessor;
use rayon::prelude::*;
use std::fs::{self};
use std::path::PathBuf;
use walkdir::WalkDir;
use xxhash_rust::xxh3::xxh3_64;

pub mod audio;
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
                    let music_dir = cx.global::<Config>().music_dir.clone();

                    if cx.global::<CurrentTrackList>().source == Some(TrackListSource::LocalLibrary)
                    {
                        return;
                    }

                    cx.spawn(async move |_this, cx| {
                        let options = ParseOptions::new()
                            .read_properties(true)
                            .read_cover_art(false);

                        let tracks = cx
                            .background_executor()
                            .spawn(async move {
                                let paths: Vec<PathBuf> = WalkDir::new(music_dir)
                                    .into_iter()
                                    .flatten()
                                    .filter(|e| e.file_type().is_file())
                                    .map(|e| e.into_path())
                                    .filter(|p| {
                                        p.extension()
                                            .and_then(|e| e.to_str())
                                            .map(|ext| {
                                                matches!(
                                                    ext,
                                                    "mp3" | "flac" | "wav" | "ogg" | "opus" | "m4a"
                                                )
                                            })
                                            .unwrap_or(false)
                                    })
                                    .collect();

                                let tracks: Vec<Track> = paths
                                    .par_iter()
                                    .filter_map(|path| {
                                        let id = xxh3_64(path.clone().to_string_lossy().as_bytes());
                                        let mut title = String::from("Not found");
                                        let mut artist = String::from("Not found");
                                        let mut album = String::from("Not found");
                                        let mut duration_secs = 1;
                                        if let Ok(tagged_file) = Probe::open(&path)
                                            .and_then(|p| p.options(options).read())
                                        {
                                            let properties = tagged_file.properties();
                                            duration_secs = properties.duration().as_secs() as u32;

                                            if let Some(tag) = tagged_file
                                                .primary_tag()
                                                .or_else(|| tagged_file.first_tag())
                                            {
                                                if let Some(tag_title) = tag.title() {
                                                    title = tag_title.to_string();
                                                }
                                                if let Some(tag_artist) = tag.artist() {
                                                    artist = tag_artist.to_string();
                                                }
                                                if let Some(tag_album) = tag.album() {
                                                    album = tag_album.to_string();
                                                } else {
                                                    album = title.clone();
                                                }
                                            }
                                        }
                                        Some(Track {
                                            id,
                                            title,
                                            artist,
                                            album,
                                            duration_secs,
                                            path: path.clone(),
                                        })
                                    })
                                    .collect();

                                tracks
                            })
                            .await;

                        let _ = cx.update(|cx| {
                            cx.update_global::<CurrentTrackList, _>(|list, _cx| {
                                list.source = Some(TrackListSource::LocalLibrary);
                                list.tracks = Some(tracks);
                            });
                        });
                    })
                    .detach();
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
                    let _ = audio_player.0.send(AudioCommand::ResumePause);
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
                    .child(div().flex().flex_1().h_full().overflow_hidden().child(
                        match current_route {
                            NavigationRoute::Home => HomePage::new().into_any_element(),
                            NavigationRoute::LocalLibrary => {
                                LocalLibraryPage::new().into_any_element()
                            }
                            NavigationRoute::TestPage { message } => {
                                TestPage::new(message).into_any_element()
                            }
                            NavigationRoute::Settings => SettingsPage::new().into_any_element(),
                        },
                    )),
            )
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let cmd_sender = init_audio();

    let audio_player = AudioPlayer(cmd_sender);
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
