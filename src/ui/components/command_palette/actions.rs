use crate::{
    discord::toggle_discord_rpc,
    player::{play_next, play_prev, stop, toggle_play},
    router::{go_back, go_forward},
    ui::icons::IconName,
};

pub struct PaletteAction {
    pub id: &'static str,
    pub label: &'static str,
    pub icon: IconName,
    pub shortcut: Option<&'static str>,
    pub keywords: &'static [&'static str],
    pub execute: fn(),
}

pub fn search_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            id: "search:all",
            label: "Search on YouTube",
            icon: IconName::Search,
            shortcut: Some("search"),
            keywords: &["yt", "search"],
            execute: || {
                super::set_query("search ");
            },
        },
        PaletteAction {
            id: "search:songs",
            label: "Search only songs on YouTube",
            icon: IconName::Search,
            shortcut: Some("search:songs"),
            keywords: &["yt", "search songs"],
            execute: || {
                super::set_query("search:songs ");
            },
        },
        PaletteAction {
            id: "search:videos",
            label: "Search only videos on YouTube",
            icon: IconName::Search,
            shortcut: Some("search:videos"),
            keywords: &["yt", "search videos"],
            execute: || {
                super::set_query("search:videos ");
            },
        },
    ]
}

pub fn playback_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            id: "play:toggle",
            label: "Resume/Pause current playback",
            icon: IconName::Play,
            shortcut: None,
            keywords: &["play", "resume", "pause"],
            execute: || {
                toggle_play();
                super::close();
            },
        },
        PaletteAction {
            id: "play:previous",
            label: "Play previous track",
            icon: IconName::SkipBack,
            shortcut: None,
            keywords: &["prev", "previous"],
            execute: || {
                play_prev();
                super::close();
            },
        },
        PaletteAction {
            id: "play:next",
            label: "Play next track",
            icon: IconName::SkipForward,
            shortcut: None,
            keywords: &["next", "advance"],
            execute: || {
                play_next();
                super::close();
            },
        },
        PaletteAction {
            id: "play:stop",
            label: "Stop playback",
            icon: IconName::Cross,
            shortcut: None,
            keywords: &["stop"],
            execute: || {
                stop();
                super::close();
            },
        },
        PaletteAction {
            id: "repeat:choose",
            label: "Change Repeat Mode",
            icon: IconName::Repeat,
            shortcut: Some("crm"),
            keywords: &["crm", "repeat"],
            execute: || {
                super::set_query("crm ");
            },
        },
    ]
}

pub fn navigation_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            id: "navigation:choose",
            label: "Navigate to page",
            icon: IconName::House,
            shortcut: Some("nav"),
            keywords: &["nav", "navigate"],
            execute: || {
                super::set_query("nav ");
            },
        },
        PaletteAction {
            id: "navigation:forward",
            label: "Navigate forward",
            icon: IconName::ChevronRight,
            shortcut: None,
            keywords: &["fw", "forward"],
            execute: || {
                go_forward();
                super::close();
            },
        },
        PaletteAction {
            id: "navigation:backward",
            label: "Navigate backward",
            icon: IconName::ChevronLeft,
            shortcut: None,
            keywords: &["bw", "backward"],
            execute: || {
                go_back();
                super::close();
            },
        },
    ]
}

pub fn app_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            id: "theme:choose",
            label: "Change Theme",
            icon: IconName::Spark,
            shortcut: Some("ct"),
            keywords: &["ct", "theme"],
            execute: || {
                super::set_query("ct ");
            },
        },
        PaletteAction {
            id: "discord:toggle",
            label: "Toggle Discord RPC",
            icon: IconName::Settings,
            shortcut: None,
            keywords: &["dc", "discord"],
            execute: || {
                toggle_discord_rpc();
                super::close();
            },
        },
    ]
}

pub fn all_action_groups() -> Vec<(&'static str, Vec<PaletteAction>)> {
    vec![
        ("Search", search_actions()),
        ("Playback", playback_actions()),
        ("Navigation", navigation_actions()),
        ("App", app_actions()),
    ]
}
