use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use nsh_core::prelude::*;

use crate::model::Track;

const CLIENT_ID: u64 = 1563230967584526436;
const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Clone)]
pub enum DiscordCommand {
    UpdateTrack(Track),
    SetPaused(Track),
    SetIdle,
    Clear,
    SetEnabled(bool),
}

#[derive(Clone, Default)]
pub struct DiscordRpc(pub Option<Sender<DiscordCommand>>);

impl DiscordRpc {
    pub fn update_track(&self, track: &Track) {
        if let Some(ref s) = self.0 {
            let _ = s.send(DiscordCommand::UpdateTrack(track.clone()));
        }
    }

    pub fn set_paused(&self, track: &Track) {
        if let Some(ref s) = self.0 {
            let _ = s.send(DiscordCommand::SetPaused(track.clone()));
        }
    }

    pub fn set_idle(&self) {
        if let Some(ref s) = self.0 {
            let _ = s.send(DiscordCommand::SetIdle);
        }
    }

    pub fn clear(&self) {
        if let Some(ref s) = self.0 {
            let _ = s.send(DiscordCommand::Clear);
        }
    }

    pub fn set_enabled(&self, enabled: bool) {
        if let Some(ref s) = self.0 {
            let _ = s.send(DiscordCommand::SetEnabled(enabled));
        }
    }
}

pub fn toggle_discord_rpc() {
    let current = global::<crate::model::Config>().get_untracked().discord_rpc;
    let new_val = !current;
    global::<crate::model::Config>().update(|config| {
        config.discord_rpc = new_val;
    });
    crate::save_config(&global::<crate::model::Config>().get_untracked());
    global::<DiscordRpc>().get_untracked().set_enabled(new_val);
    if new_val {
        let player = global::<crate::model::AudioPlayer>().get_untracked();
        if let Some(ref track) = player.current_track {
            if player.is_playing {
                global::<DiscordRpc>().get_untracked().update_track(track);
            } else {
                global::<DiscordRpc>().get_untracked().set_paused(track);
            }
        } else {
            global::<DiscordRpc>().get_untracked().set_idle();
        }
    }
}

pub fn init(enabled: bool) -> DiscordRpc {
    let (tx, rx) = mpsc::channel::<DiscordCommand>();
    let tx_ready = tx.clone();

    thread::Builder::new()
        .name("discord-rpc".into())
        .spawn(move || {
            run_discord_loop(rx, tx_ready, enabled);
        })
        .expect("Failed to spawn discord rpc thread");

    DiscordRpc(Some(tx))
}

fn set_idle_activity(drpc: &mut discord_presence::Client) {
    let _ = drpc.set_activity(|a| {
        a.activity_type(discord_presence::models::ActivityType::Listening)
            .details("No playback")
            .state("In menu")
            .assets(|ass| {
                ass.large_image("app_logo")
                    .large_text("Heliox")
                    .small_text("Idle")
            })
            .append_buttons(|b| {
                b.label("GitHub")
                    .url("https://github.com/GamingChelsea/Heliox/")
            })
    });
}

fn run_discord_loop(
    rx: Receiver<DiscordCommand>,
    tx_ready: Sender<DiscordCommand>,
    initial_enabled: bool,
) {
    let mut drpc = discord_presence::Client::new(CLIENT_ID);
    drpc.on_ready(move |_ctx| {
        let _ = tx_ready.send(DiscordCommand::SetIdle);
    })
    .persist();
    drpc.start();

    let mut enabled = initial_enabled;
    let mut paused_since: Option<Instant> = None;

    loop {
        let timeout = if enabled {
            paused_since.map(|start| IDLE_TIMEOUT.saturating_sub(start.elapsed()))
        } else {
            None
        };

        let res = match timeout {
            Some(remaining) if remaining.is_zero() => {
                paused_since = None;
                if enabled {
                    set_idle_activity(&mut drpc);
                }
                continue;
            }
            Some(remaining) => rx.recv_timeout(remaining),
            None => rx.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };

        match res {
            Ok(DiscordCommand::SetEnabled(val)) => {
                enabled = val;
                paused_since = None;
                if !enabled {
                    let _ = drpc.clear_activity();
                } else {
                    set_idle_activity(&mut drpc);
                }
            }
            Ok(DiscordCommand::UpdateTrack(track)) => {
                if !enabled {
                    continue;
                }
                paused_since = None;
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let end = now + track.duration_secs as u64;

                let _ = drpc.set_activity(|a| {
                    a.activity_type(discord_presence::models::ActivityType::Listening)
                        .details(&track.title)
                        .state(format!("from {}", track.artist))
                        .timestamps(|t| t.start(now).end(end))
                        .assets(|mut ass| {
                            ass = ass.large_image("app_logo");
                            if !track.album.is_empty() && track.album != track.title {
                                ass = ass.large_text(&track.album);
                            }
                            ass.small_image("play_icon").small_text("Currently playing")
                        })
                        .party(|p| p.id("current_queue").size((1, 10)))
                        .append_buttons(|b| {
                            b.label("GitHub")
                                .url("https://github.com/GamingChelsea/Heliox/")
                        })
                });
            }
            Ok(DiscordCommand::SetPaused(track)) => {
                if !enabled {
                    continue;
                }
                paused_since = Some(Instant::now());
                let _ = drpc.set_activity(|a| {
                    a.activity_type(discord_presence::models::ActivityType::Listening)
                        .details(&track.title)
                        .state(format!("from {} [paused]", track.artist))
                        .assets(|mut ass| {
                            ass = ass.large_image("app_logo");
                            if !track.album.is_empty() && track.album != track.title {
                                ass = ass.large_text(&track.album);
                            }
                            ass.small_image("paused_icon").small_text("Paused")
                        })
                        .append_buttons(|b| {
                            b.label("GitHub")
                                .url("https://github.com/GamingChelsea/Heliox/")
                        })
                });
            }
            Ok(DiscordCommand::SetIdle) => {
                if !enabled {
                    continue;
                }
                paused_since = None;
                set_idle_activity(&mut drpc);
            }
            Ok(DiscordCommand::Clear) => {
                paused_since = None;
                let _ = drpc.clear_activity();
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if enabled {
                    paused_since = None;
                    set_idle_activity(&mut drpc);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}
