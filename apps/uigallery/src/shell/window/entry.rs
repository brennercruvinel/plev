//! The per-platform entry points of the gallery: desktop `run`, browser
//! `run_web`, android `android_main`, iOS `showcase_ios_main`. Each builds
//! the winit event loop its platform wants and drives the same [`App`].

use super::{App, UserEvent};
use winit::event_loop::EventLoop;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use super::{background, record};
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
use crate::view::ShowcaseView;

/// Desktop entry: build the winit event loop and run the showcase.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub fn run() {
    // A recording says where its files go: the gallery's own info lines
    // show by default when it records. Off unless `PLEV_RECORD=1` in a
    // debug build; nothing is created then.
    let recording = record::Recording::from_env();
    let filter = if recording.is_some() {
        "warn,showcase=info"
    } else {
        "warn"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter)).init();
    let mut builder = EventLoop::<UserEvent>::with_user_event();
    #[cfg(target_os = "macos")]
    background::event_loop(&mut builder);
    let event_loop = builder.build().unwrap();
    let mut app = App::new();
    app.record(recording);
    let proxy = event_loop.create_proxy();
    app.access.proxy = Some(proxy.clone());
    let agent = proxy.clone();
    app.access.agent = ShowcaseView::agent_port(move || {
        let _ = agent.send_event(UserEvent::Agent);
    });
    app.view.connect_git(Box::new(move || {
        let _ = proxy.send_event(UserEvent::Wake);
    }));
    event_loop.run_app(&mut app).unwrap();
}

/// Browser entry: GPU init is async, so hand the proxy to the app and let
/// `spawn_app` keep the loop alive inside the browser's own event loop
/// (`run_app` would throw to escape `main`).
#[cfg(target_arch = "wasm32")]
pub fn run_web() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("failed to init console_log");

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    let mut app = App::new();
    app.proxy = Some(event_loop.create_proxy());
    event_loop.spawn_app(app);
}

/// Android entry: GameActivity calls `android_main` on the native side. The
/// `android-game-activity` winit backend (enabled via the
/// `plev/android-game-activity` feature) provides the JNI glue; we own the
/// event loop here.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub fn android_main(android_app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plev-showcase"),
    );

    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .with_android_app(android_app)
        .build()
        .expect("android event loop");
    let mut app = App::new();
    if let Err(e) = event_loop.run_app(&mut app) {
        log::error!("event loop error: {e:?}");
    }
}

/// iOS entry: the thin Objective-C `main` in the Xcode app calls this; winit
/// drives `UIApplicationMain` from `run_app` and never returns.
#[cfg(target_os = "ios")]
#[unsafe(no_mangle)]
pub extern "C" fn showcase_ios_main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("ios event loop");
    let mut app = App::new();
    let _ = event_loop.run_app(&mut app);
}
