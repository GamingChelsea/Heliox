use gpui_kit::component::setting::{
    SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_kit::component::{ActiveTheme, Theme, ThemeConfig, ThemeRegistry};
use gpui_kit::*;

use crate::model::Config;
use crate::save_config;

#[derive(IntoElement)]
pub struct SettingsPage;

impl SettingsPage {
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for SettingsPage {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let themes: Vec<(SharedString, SharedString)> = ThemeRegistry::global(cx)
            .sorted_themes()
            .into_iter()
            .map(|theme| (theme.name.clone(), theme.name.clone()))
            .collect();

        let current_theme_name = cx.theme().theme_name().clone();

        Settings::new("app-settings").pages(vec![
            SettingPage::new("General").default_open(true).group(
                SettingGroup::new().title("Appearance").item(
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
                ),
            ),
        ])
    }
}
