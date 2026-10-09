//! showcase library target: the design-system gallery as a reusable crate.
//!
//! Pure, gpu-free backend modules (`model`) are unit tested ahead of the
//! GPU-facing `view`/`renderer`/`app`. The `app` module owns the winit
//! `ApplicationHandler` and the per-platform entry points re-exported here,
//! so desktop (`main.rs`), the browser, android (cdylib `android_main`) and
//! iOS (staticlib `uigallery_ios_main`) all drive the same gallery.

// Let the view modules keep referring to this crate by name
// (`uigallery::domain::...`) now that they live in the library.
extern crate self as showcase;

pub mod domain;
pub mod keyboard;
pub mod renderer;
pub mod shell;
pub mod views;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use shell::run;
#[cfg(target_arch = "wasm32")]
pub use shell::run_web;
