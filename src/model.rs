use gpui_kit::{Global, SharedString, private::serde::Deserialize};
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub theme_name: SharedString,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_name: SharedString::from("Default Dark"),
        }
    }
}

impl Global for Config {}
