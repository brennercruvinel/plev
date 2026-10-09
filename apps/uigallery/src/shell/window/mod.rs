//! The shell's window: entry, events, pacing, the headless and detached
//! hosts, mirroring, accessibility, recording and view-source.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod access;
/// The page in a window of its own (task-76 item 2), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod detached;
/// The per-platform entry points: desktop `run`, browser `run_web`,
/// android `android_main`, iOS `showcase_ios_main`.
pub(super) mod entry;
pub(super) mod events;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod headless;
/// The windows' scenes over the agent port, and the viewer's side of a
/// watch (task-79), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub mod mirror;
pub(super) mod pace;
/// `PLEV_RECORD=1` in a debug build: the session recorded for the
/// inspector, as the ide records its own.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub mod record;
/// cmd+click to the code: desktop debug builds, the terms on which the
/// engine compiles `hot_reload` (its build.rs), so nothing of it exists in
/// release, wasm or mobile.
#[cfg(all(
    debug_assertions,
    any(target_os = "macos", target_os = "linux", target_os = "windows")
))]
pub(super) mod source;
