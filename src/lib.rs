// -- Error
pub mod error;

// -- Rendering
pub mod compositor;
pub mod effects;
pub mod gpu;
pub mod path;

// -- Text
pub mod editor;
pub mod text;
pub mod text_input;

// -- Input & Events
pub mod actions;
pub mod input;

// -- Reactive & Components
pub mod builder;
pub mod component;
pub mod signal;
pub mod view;

// -- UI & Overlay
pub mod color;
pub mod layout;
pub mod overlay;
pub mod theme;
pub mod ui;

// -- Animation
pub mod animation;

// -- Performance instrumentation
pub mod perf;

// -- Platform
pub mod platform;
pub mod window;

// -- Accessibility
#[cfg(feature = "accessibility")]
pub mod accessibility;

// -- Hot Reload
#[cfg(feature = "hot-reload")]
pub mod hot_reload;
#[cfg(feature = "hot-reload")]
pub mod narrate_runtime;

pub use macros::component;

pub use wgpu;
pub use winit;

/// Resolve a narrate block: check override map first, fall back to compiled code.
///
/// Called by proc-macro generated code. When `hot-reload` is off, this inlines
/// to a direct call of `compiled()` with zero overhead.
#[cfg(feature = "hot-reload")]
#[inline(always)]
pub fn narrate_resolve<F: FnOnce() -> builder::Element>(
    file: &str,
    line: u32,
    compiled: F,
) -> builder::Element {
    if let Some(el) = hot_reload::narrate_override(file, line) {
        el
    } else {
        compiled()
    }
}

#[cfg(not(feature = "hot-reload"))]
#[inline(always)]
pub fn narrate_resolve<F: FnOnce() -> builder::Element>(
    _file: &str,
    _line: u32,
    compiled: F,
) -> builder::Element {
    compiled()
}

/// Shared event loop runner used by native desktop platforms.
#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
pub fn run_event_loop(event_loop: winit::event_loop::EventLoop<window::AppEvent>) {
    let mut app = window::App::new();
    event_loop.run_app(&mut app).unwrap();
}

/// Android entry point for the engine's built-in [`window::App`].
///
/// Gated behind the `android-entry` cargo feature (off by default), mirroring
/// `web-entry`: a cdylib exports exactly one `android_main` symbol, so a
/// downstream app (showcase, ide, ...) that ships its own GameActivity entry
/// must be able to opt out and define its own.
#[cfg(all(target_os = "android", feature = "android-entry"))]
#[unsafe(no_mangle)]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("plev"),
    );

    log::info!("plev android_main started");

    let event_loop = winit::event_loop::EventLoop::<window::AppEvent>::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();
    let mut app = window::App::new_with_proxy(proxy);
    match event_loop.run_app(&mut app) {
        Ok(()) => log::info!("Event loop exited cleanly"),
        Err(e) => log::error!("Event loop error: {:?}", e),
    }
}

#[cfg(all(target_arch = "wasm32", feature = "web-entry"))]
use wasm_bindgen::prelude::*;

/// Browser entry point for the engine's built-in [`window::App`].
///
/// Gated behind the `web-entry` cargo feature (off by default): a wasm
/// module may only have one `#[wasm_bindgen(start)]`, so downstream apps
/// that ship their own `main` on the web must be able to opt out.
#[cfg(all(target_arch = "wasm32", feature = "web-entry"))]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    use winit::platform::web::EventLoopExtWebSys;

    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to init logger");

    let event_loop = winit::event_loop::EventLoop::<window::AppEvent>::with_user_event()
        .build()
        .unwrap();
    let proxy = event_loop.create_proxy();
    let app = window::App::new_with_proxy(proxy);
    event_loop.spawn_app(app);
}
