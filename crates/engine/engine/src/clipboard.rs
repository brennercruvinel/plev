//! First-level clipboard access: get/set text without importing the
//! editor. Apps that only want "copy to clipboard" belong here; the
//! editor's own clipboard integration (selection sync, cut/paste inside
//! `EditorView`) lives in [`crate::editor`].
//!
//! [`SystemClipboard`] wraps the OS clipboard via arboard on desktop and
//! degrades to a no-op when the clipboard is unavailable (headless CI).
//! [`LocalClipboard`] is the in-memory fallback for tests and wasm.

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use crate::editor::SystemClipboard;
pub use crate::editor::{ClipboardProvider, LocalClipboard};
