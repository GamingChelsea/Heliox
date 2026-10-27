use crate::{
    model::Config,
    player::{play_next, play_prev, stop, toggle_play},
    router::NavigationHistory,
};
use gpui_kit::{
    App, BorrowAppContext, Entity, Window,
    assets::IconName,
    component::{WindowExt, command::CommandState},
};

pub struct PaletteAction {
    pub label: &'static str,
    pub icon: IconName,
    pub keywords: &'static [&'static str],
    pub execute: fn(&Entity<CommandState>, &mut Window, &mut App),
}

pub fn playback_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            label: "Resume/Pause the current playback",
            icon: IconName::Play,
            keywords: &["play", "resume", "pause"],
            execute: |_cmd, window, cx| {
                toggle_play(cx);
                window.close_dialog(cx);
            },
        },
        PaletteAction {
            label: "Play the previous track",
            icon: IconName::SkipBack,
            keywords: &["next"],
            execute: |_cmd, window, cx| {
                play_prev(cx);
                window.close_dialog(cx);
            },
        },
        PaletteAction {
            label: "Play the next track",
            icon: IconName::SkipForward,
            keywords: &["next"],
            execute: |_cmd, window, cx| {
                play_next(cx);
                window.close_dialog(cx);
            },
        },
        PaletteAction {
            label: "Stop the current playback",
            icon: IconName::CircleStop,
            keywords: &["stop"],
            execute: |_cmd, window, cx| {
                stop(cx);
                window.close_dialog(cx);
            },
        },
        PaletteAction {
            label: "Change Repeat Mode",
            icon: IconName::Repeat,
            keywords: &["crm", "repeat"],
            execute: |command, window, cx| {
                command.update(cx, |state, cx| state.set_query("crm ", window, cx));
                window.refresh();
            },
        },
    ]
}

pub fn navigation_actions() -> Vec<PaletteAction> {
    vec![
        PaletteAction {
            label: "Navigate to specific page",
            icon: IconName::Navigation,
            keywords: &["nav", "navigate"],
            execute: |command, window, cx| {
                command.update(cx, |state, cx| state.set_query("nav ", window, cx));
                window.refresh();
            },
        },
        PaletteAction {
            label: "Navigate forward",
            icon: IconName::ArrowRight,
            keywords: &["fw", "forward"],
            execute: |_cmd, _window, cx| {
                cx.update_global::<NavigationHistory, _>(|history, _cx| {
                    history.go_forward();
                });
            },
        },
        PaletteAction {
            label: "Navigate backward",
            icon: IconName::ArrowLeft,
            keywords: &["bw", "backward"],
            execute: |_cmd, _window, cx| {
                cx.update_global::<NavigationHistory, _>(|history, _cx| {
                    history.go_back();
                });
            },
        },
    ]
}

pub fn app_actions(cx: &App) -> Vec<PaletteAction> {
    let discord_enabled = cx.global::<Config>().discord_rpc;
    let discord_icon = if discord_enabled {
        IconName::ToggleRight
    } else {
        IconName::ToggleLeft
    };

    vec![
        PaletteAction {
            label: "Change Theme",
            icon: IconName::Palette,
            keywords: &["ct", "theme"],
            execute: |command, window, cx| {
                command.update(cx, |state, cx| state.set_query("ct ", window, cx));
                window.refresh();
            },
        },
        PaletteAction {
            label: "Toggle Discord RPC",
            icon: discord_icon,
            keywords: &["dc", "discord"],
            execute: |_cmd, window, cx| {
                crate::discord::toggle_discord_rpc(cx);
                window.close_dialog(cx);
            },
        },
        PaletteAction {
            label: "Quit the App",
            icon: IconName::Power,
            keywords: &["quit", "exit", "close"],
            execute: |_cmd, _window, cx| {
                cx.quit();
            },
        },
    ]
}

pub fn all_action_groups(cx: &App) -> Vec<(&'static str, Vec<PaletteAction>)> {
    vec![
        ("Playback", playback_actions()),
        ("Navigation", navigation_actions()),
        ("App", app_actions(cx)),
    ]
}
