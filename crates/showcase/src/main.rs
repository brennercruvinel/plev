//! Desktop/web binary entry: delegates to the showcase library, which owns
//! the winit application shell and every per-platform entry point. Mobile
//! does not use this binary — android loads the cdylib's `android_main` and
//! iOS links the staticlib's `showcase_ios_main` instead.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn main() {
    showcase::run();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    showcase::run_web();
}

// Keep the binary target compilable when the whole crate is built for a
// mobile target; the real entry points live in the library.
#[cfg(any(target_os = "android", target_os = "ios"))]
fn main() {}
