//! The shell's reach out of the window: background, capture, links, menu,
//! notifications, printing, sleep, touch and tray.

/// `PLEV_BACKGROUND=1`: the window comes up without taking the focus.
#[cfg(target_os = "macos")]
pub(super) mod background;
/// The camera and the microphone follow the windows (task-76 item 10).
pub(super) mod capture;
/// The app's URL scheme: a link followed as a link, never as the person
/// (task-76 item 6).
pub mod links;
/// The native menu bar (task-76 item 4), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod menu;
/// The system notifications, and the focus that decides them (task-76
/// item 5), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod notify;
/// The page on paper: the platform's printer and the print keys (task-76
/// item 9), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod print;
/// Asleep, covered, awake again (task-76 item 7).
pub(super) mod sleep;
pub(super) mod touch;
/// The tray icon with the shell's menu (task-76 item 5), desktop only.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub(super) mod tray;
