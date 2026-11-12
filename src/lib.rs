use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use nsh_core::prelude::*;

pub mod audio;
pub mod discord;
pub mod library;
pub mod model;
pub mod player;
pub mod providers;
pub mod queue;
pub mod router;
pub mod stream;
pub mod ui;

use crate::audio::init_audio;
use crate::model::{AudioPlayer, Config, Track, UiMediaEvent};
use crate::player::{on_track_end, pause, play_next, play_prev, resume, stop, toggle_play};
use crate::queue::QueueManager;
use crate::router::{NavigationHistory, NavigationRoute, go_back, go_forward};
use crate::ui::{
    components::{
        command_palette::{self, command_palette},
        player_bar::player_bar,
        queue_popover::queue_popover,
        sidebar::{app_sidebar, toggle_sidebar},
    },
    pages::{
        home_page::home_page, local_library::local_library_page, search_page::search_page,
        settings_page::settings_page, test_page::test_page,
    },
    theme::colors,
};

enum MediaControlCmd {
    UpdateTrack(Track),
    SetPlayback(bool),
    SetVolume(f64),
}

static MEDIA_CONTROL_TX: Mutex<Option<std::sync::mpsc::Sender<MediaControlCmd>>> = Mutex::new(None);
/// The UI-event receiver is stored here so `MusicApp` can take it once and
/// poll it via NSH's async `spawn()` — which runs on the reactive UI thread.
static UI_RECEIVER: Mutex<Option<async_channel::Receiver<UiMediaEvent>>> = Mutex::new(None);
/// The audio command sender is stored here so the media-event task (spawned
/// inside the reactive runtime) can reach the audio thread without
/// going through `global::<AudioPlayer>()` from a plain thread.
static AUDIO_CMD_TX: Mutex<Option<std::sync::mpsc::Sender<crate::audio::AudioCommand>>> =
    Mutex::new(None);

fn replace<T>(slot: &Mutex<Option<T>>, value: T) {
    *slot.lock().unwrap_or_else(|error| error.into_inner()) = Some(value);
}

pub fn update_media_controls_track(track: &Track) {
    if let Some(tx) = MEDIA_CONTROL_TX
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
    {
        let _ = tx.send(MediaControlCmd::UpdateTrack(track.clone()));
    }
}

pub fn update_media_controls_playback(is_playing: bool) {
    if let Some(tx) = MEDIA_CONTROL_TX
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
    {
        let _ = tx.send(MediaControlCmd::SetPlayback(is_playing));
    }
}

pub fn update_media_controls_volume(volume: f64) {
    if let Some(tx) = MEDIA_CONTROL_TX
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .as_ref()
    {
        let _ = tx.send(MediaControlCmd::SetVolume(volume));
    }
}

fn init_media_controls(ui_sender: async_channel::Sender<UiMediaEvent>) {
    let (tx, rx) = std::sync::mpsc::channel::<MediaControlCmd>();
    replace(&MEDIA_CONTROL_TX, tx);

    std::thread::Builder::new()
        .name("media-controls".into())
        .spawn(move || {
            let config = souvlaki::PlatformConfig {
                dbus_name: "heliox",
                display_name: "Heliox",
                hwnd: None,
            };

            let mut controls = match souvlaki::MediaControls::new(config) {
                Ok(c) => c,
                Err(err) => {
                    tracing::warn!(?err, "Failed to initialize souvlaki media controls");
                    return;
                }
            };

            let ui_tx = ui_sender.clone();
            let _ = controls.attach(move |event| match event {
                souvlaki::MediaControlEvent::Toggle => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Toggle);
                }
                souvlaki::MediaControlEvent::Play => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Play);
                }
                souvlaki::MediaControlEvent::Pause => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Pause);
                }
                souvlaki::MediaControlEvent::Stop => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Stop);
                }
                souvlaki::MediaControlEvent::Next => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Next);
                }
                souvlaki::MediaControlEvent::Previous => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Previous);
                }
                souvlaki::MediaControlEvent::SetVolume(val) => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::SetVolume(val.clamp(0.0, 1.0)));
                }
                souvlaki::MediaControlEvent::Quit => {
                    let _ = ui_tx.send_blocking(UiMediaEvent::Quit);
                }
                _ => {}
            });

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    MediaControlCmd::UpdateTrack(track) => {
                        let _ = controls.set_metadata(souvlaki::MediaMetadata {
                            title: Some(&track.title),
                            artist: Some(&track.artist),
                            album: Some(&track.album),
                            duration: Some(std::time::Duration::from_secs(
                                track.duration_secs as u64,
                            )),
                            cover_url: None,
                        });
                        let _ = controls
                            .set_playback(souvlaki::MediaPlayback::Playing { progress: None });
                    }
                    MediaControlCmd::SetPlayback(is_playing) => {
                        let playback = if is_playing {
                            souvlaki::MediaPlayback::Playing { progress: None }
                        } else {
                            souvlaki::MediaPlayback::Paused { progress: None }
                        };
                        let _ = controls.set_playback(playback);
                    }
                    MediaControlCmd::SetVolume(vol) => {
                        #[cfg(all(unix, not(target_os = "macos")))]
                        let _ = controls.set_volume(vol);
                        #[cfg(not(all(unix, not(target_os = "macos"))))]
                        let _ = vol;
                    }
                }
            }
        })
        .expect("Failed to spawn media controls thread");
}

#[component]
pub fn MusicApp() -> impl IntoElement {
    // Global Key Bindings
    on_key("ctrl+b", |_| toggle_sidebar());
    on_key("cmd+b", |_| toggle_sidebar());
    on_key("ctrl+p", |_| command_palette::toggle());
    on_key("cmd+p", |_| command_palette::toggle());
    on_key("ctrl+shift+p", |_| command_palette::toggle());
    on_key("cmd+shift+p", |_| command_palette::toggle());
    on_key("escape", |_| command_palette::close_if_open());
    on_key("space", |_| toggle_play());
    on_key("alt+left", |_| go_back());
    on_key("alt+right", |_| go_forward());

    // Playback progress ticker: when playing, update player bar every 250ms
    set_interval(std::time::Duration::from_millis(250), move || {
        library::poll_local_library();
        let is_playing = global::<AudioPlayer>().get_untracked().is_playing;
        if is_playing {
            global::<AudioPlayer>().update(|_| {});
        }
    });

    // Start the media-event processing loop as an NSH async task.
    // This runs on the reactive UI thread, so global::<T>() is safe here.
    let receiver = UI_RECEIVER
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    if let Some(rx) = receiver {
        spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    UiMediaEvent::Toggle => toggle_play(),
                    UiMediaEvent::Play => resume(),
                    UiMediaEvent::Pause => pause(),
                    UiMediaEvent::Stop => stop(),
                    UiMediaEvent::Next => play_next(),
                    UiMediaEvent::Previous => play_prev(),
                    UiMediaEvent::TrackEnded => on_track_end(),
                    UiMediaEvent::SetVolume(vol) => {
                        let v = vol as f32;
                        if let Some(tx) = AUDIO_CMD_TX
                            .lock()
                            .unwrap_or_else(|error| error.into_inner())
                            .as_ref()
                        {
                            let _ = tx.send(crate::audio::AudioCommand::SetVolume(v));
                        }
                        global::<Config>().update(|c| c.volume = v);
                        save_config(&global::<Config>().get_untracked());
                    }
                    UiMediaEvent::Quit => {}
                }
            }
        });
    }

    let page_content = shader_transition()
        .value(move || {
            use nsh_core::shader::TransitionKey;

            match global::<NavigationHistory>().get().current {
                NavigationRoute::Home => TransitionKey::from("home"),
                NavigationRoute::Search => TransitionKey::from("search"),
                NavigationRoute::LocalLibrary => TransitionKey::from("library"),
                NavigationRoute::TestPage { .. } => TransitionKey::from("test"),
                NavigationRoute::Settings => TransitionKey::from("settings"),
            }
        })
        .duration(std::time::Duration::from_millis(if reduce_motion() {
            0
        } else {
            240
        }))
        .child(dynamic(|| {
            match global::<NavigationHistory>().get().current {
                NavigationRoute::Home => home_page().into_element(),
                NavigationRoute::Search => search_page().into_element(),
                NavigationRoute::LocalLibrary => local_library_page().into_element(),
                NavigationRoute::TestPage { message } => test_page(message).into_element(),
                NavigationRoute::Settings => settings_page().into_element(),
            }
        }));

    view()
        .flex_1()
        .bg(colors::BACKGROUND)
        // Main view: Sidebar + Page content
        .child(
            view()
                .flex_1()
                .flex_row()
                .child(dynamic(app_sidebar))
                .child(view().flex_1().overflow_hidden().child(page_content)),
        )
        // Bottom Player Bar
        .child(dynamic(player_bar))
        // Overlays: Queue Popover & Command Palette
        .child(dynamic(queue_popover))
        .child(dynamic(command_palette))
}

fn setup_app() {
    let config = load_config();
    let initial_volume = config.volume;
    let initial_repeat = config.repeat_mode;

    let (ui_sender, ui_receiver) = async_channel::unbounded::<UiMediaEvent>();
    let (cmd_sender, progress_ms) = init_audio(ui_sender.clone());
    let audio_player = AudioPlayer::new(cmd_sender.clone(), progress_ms);
    let discord_rpc = discord::init(config.discord_rpc);

    let _ = cmd_sender.send(audio::AudioCommand::SetVolume(initial_volume));
    init_media_controls(ui_sender);
    replace(&UI_RECEIVER, ui_receiver);
    replace(&AUDIO_CMD_TX, cmd_sender.clone());

    global::<Config>().set(config);
    crate::ui::theme::apply_theme(&global::<Config>().get_untracked().theme_name);
    global::<QueueManager>().update(|qm| {
        qm.repeat_mode = initial_repeat;
    });
    global::<AudioPlayer>().set(audio_player);
    global::<crate::discord::DiscordRpc>().set(discord_rpc);
}

fn register_app(registry: &mut nsh_core::element::Registry) {
    registry.register::<MusicApp>();
    registry.setup(setup_app);
}

#[native_module("heliox/native")]
fn native(registry: &mut nsh_core::element::Registry) {
    register_app(registry);
}

pub fn ui_sets() -> Vec<nsh_core::element::Registry> {
    let mut app = nsh_core::element::Registry::new();
    register_app(&mut app);
    vec![nsh_base::registry(), app]
}

pub fn save_config(config: &Config) {
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
        .and_then(|p| Some(p.join("GamingChelsea").join("Heliox")))
        .expect("Error creating path");
    let _ = fs::create_dir_all(path.clone()).expect("Error creating directories");
    path
}

fn get_config_path() -> PathBuf {
    get_path().join("config.ron")
}
