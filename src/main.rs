#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;
#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;
#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn main() -> impl std::process::Termination {
    tracing_subscriber::fmt::init();
    platform::set_ui_sets(heliox::ui_sets());

    platform::run_app(
        platform::AppSource::Rust(
            platform::Root::of::<heliox::MusicApp>().app_id("com.heliox.music"),
        ),
        platform::AppOptions {
            title: "Heliox".to_owned(),
            app_id: "com.heliox.music".to_owned(),
            size: (1270, 720),
            ..Default::default()
        },
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!("Run this binary on macOS, Windows or Linux.");
