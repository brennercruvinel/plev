//! Git section: the plev ide workspace (changes, stacks, diff) embedded in
//! the gallery's content rect against the repository the showcase was
//! launched from. The panels are the ide's own views (`ide::views`), so a
//! comps change shows up here and in the ide binary alike. The git worker
//! wakes the event loop through the host-provided callback; its events
//! are drained on the next tick.
//!
//! Needs the `git` CLI and a thread: desktop only. Elsewhere the section
//! explains itself instead of rendering a dead workspace.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
mod desktop;
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use desktop::GitSection;

#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
mod stub;
#[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
pub use stub::GitSection;

/// Wakes the host event loop from the git worker thread.
pub type Wake = Box<dyn Fn() + Send + 'static>;

/// Vertical navigation of the file list (arrow keys).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NavKey {
    Up,
    Down,
}
