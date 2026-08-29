//! nestui library target: the .nest explorer as a reusable crate.
//!
//! Pure, gpu-free backend modules (`model`) are unit tested ahead of the
//! GPU-facing `view`/`renderer`/`app`. The `app` module owns the winit
//! `ApplicationHandler` and the per-platform entry points re-exported here,
//! so desktop (`main.rs`), the browser, android (cdylib `android_main`) and
//! iOS (staticlib `nestui_ios_main`) all drive the same app.
//!
//! The model compiles on every target: pure modules (`types`, `nestread`,
//! `graph`, `bench`) everywhere; the mmap backend and worker thread are
//! native-only, the web build swaps in `nestread` + an inline worker.

// Let the view modules keep referring to this crate by name
// (`nestui::model::...`) now that they live in the library.
extern crate self as nestui;

pub mod model;

pub mod app;
pub mod keys;
pub mod renderer;
pub mod view;
#[cfg(target_arch = "wasm32")]
mod web_picker;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use app::run;
#[cfg(target_arch = "wasm32")]
pub use app::run_web;
