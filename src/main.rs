use crate::router::{NavigationHistory, NavigationRoute};
use crate::ui::components::sidebar::AppSidebar;
use crate::ui::pages::settings_page::SettingsPage;
use gpui_kit::assets::AllAssets;
use gpui_kit::component::label::Label;
use gpui_kit::component::*;
use gpui_kit::*;
use std::path::PathBuf;

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

            gpui_kit::open_window(
                WindowOptions {
                    titlebar: Some(TitlebarOptions {
                        appears_transparent: false,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                cx,
                |_, cx| {
                    // Theme
                    let theme_name = SharedString::from("Default Dark");
                    if let Err(err) =
                        ThemeRegistry::watch_dir(PathBuf::from("./themes"), cx, move |cx| {
                            if let Some(theme) =
                                ThemeRegistry::global(cx).themes().get(&theme_name).cloned()
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
