use crate::router::{NavigationHistory, NavigationRoute};
use gpui_kit::component::*;
use gpui_kit::component::{button::*, label::Label};
use gpui_kit::*;
use std::path::PathBuf;

mod router;

pub struct MusicApp {
    nav_history: Entity<NavigationHistory>,
}
impl MusicApp {
    pub fn new(nav_history: Entity<NavigationHistory>, cx: &mut Context<Self>) -> Self {
        cx.observe(&nav_history, |_this, _entity, cx| {
            cx.notify();
        })
        .detach();

        Self { nav_history }
    }
}

impl Render for MusicApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_route = self.nav_history.read(cx).current_route().clone();

        div()
            .v_flex()
            .gap_2()
            .size_full()
            .items_center()
            .justify_center()
            .child("Hello, World!")
            .child(match current_route {
                NavigationRoute::Home => Label::new("Home"),
                NavigationRoute::TestPage { message } => Label::new(message),
            })
            .child(Button::new("Home").child("Home").on_click({
                let nav = self.nav_history.clone();
                move |_event, _window, app| {
                    nav.update(app, |history, cx| {
                        history.push(NavigationRoute::Home);
                        cx.notify();
                    });
                }
            }))
            .child(Button::new("TestPage").child("Test Page").on_click({
                let nav = self.nav_history.clone();
                move |_event, _window, app| {
                    nav.update(app, |history, cx| {
                        history.push(NavigationRoute::TestPage {
                            message: "hello from this menu".into(),
                        });
                        cx.notify();
                    });
                }
            }))
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);

        gpui_kit::open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
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

                let nav_history = cx.new(|_| NavigationHistory::new(NavigationRoute::Home));

                cx.new(|cx| MusicApp::new(nav_history, cx))
            },
        )
        .expect("Failed to open window");
    });
}
