use gpui_kit::{
    App, BorrowAppContext, Entity, Window,
    assets::IconName,
    component::{WindowExt, command::CommandState},
};

use crate::{audio::AudioCommand, model::AudioPlayer, router::NavigationHistory};

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
                cx.update_global::<AudioPlayer, _>(|audio_player, cx| {
                    let _ = audio_player.0.send(AudioCommand::ResumePause);
                    window.close_dialog(cx);
                });
            },
        },
        PaletteAction {
            label: "Stop the current playback",
            icon: IconName::CircleStop,
            keywords: &["stop"],
            execute: |_cmd, window, cx| {
                cx.update_global::<AudioPlayer, _>(|audio_player, cx| {
                    let _ = audio_player.0.send(AudioCommand::Stop);
                    window.close_dialog(cx);
                });
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

pub fn app_actions() -> Vec<PaletteAction> {
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
            label: "Quit the App",
            icon: IconName::Power,
            keywords: &["quit", "exit", "close"],
            execute: |_cmd, _window, cx| {
                cx.quit();
            },
        },
    ]
}

pub fn all_action_groups() -> Vec<(&'static str, Vec<PaletteAction>)> {
    vec![
        ("Playback", playback_actions()),
        ("Navigation", navigation_actions()),
        ("App", app_actions()),
    ]
}
