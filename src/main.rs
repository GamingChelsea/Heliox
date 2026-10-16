use crate::model::Config;
use crate::router::{NavigationHistory, NavigationRoute};
use crate::ui::components::sidebar::AppSidebar;
use crate::ui::pages::settings_page::SettingsPage;
use gpui_kit::assets::AllAssets;
use gpui_kit::component::TitleBar;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::*;
use std::fs::{self};
use std::path::PathBuf;

pub mod model;
pub mod router;
pub mod ui;

pub struct MusicApp {
    sidebar: Entity<AppSidebar>,
}

impl MusicApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<NavigationHistory>(|_this, cx| {
            cx.notify();
        })
        .detach();

        let sidebar = cx.new(AppSidebar::new);

        Self { sidebar }
    }
}

impl Render for MusicApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_route = cx.global::<NavigationHistory>().current_route().clone();

        div()
            .flex()
            .v_flex()
            .w_full()
            .h_full()
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
                            .items_center()
                            .justify_center()
                            .child(match current_route {
                                NavigationRoute::Home => Label::new("Home").into_any_element(),
                                NavigationRoute::TestPage { message } => {
                                    Label::new(message).into_any_element()
                                }
                                NavigationRoute::Settings => SettingsPage::new().into_any_element(),
                            }),
                    ),
            )
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    gpui_kit::application()
        .with_assets(AllAssets)
        .run(move |cx| {
            gpui_kit::init(cx);

            cx.set_global(NavigationHistory::new(NavigationRoute::Home));
            init_config(cx);

            // Save Settings on quit
            cx.on_app_quit(|cx| {
                let config = Config {
                    theme_name: cx.theme().theme_name().clone(),
                };

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
                |_, cx| {
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

                    cx.new(MusicApp::new)
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
