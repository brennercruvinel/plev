//! Multi-line, multi-cursor text editor widget (plan §WS-C fase 2).
//!
//! Built on the headless foundations:
//! - [`rope`]: rope document, transactional edits, multi-cursor
//!   selections and undo history;
//! - [`TextMeasurer`](crate::text::TextMeasurer): GPU-free shaping for
//!   hit-testing and caret geometry;
//! - the compositor scene graph for rendering.
//!
//! The widget itself stays GPU-free: [`EditorView::render`] only emits
//! [`SceneNode`](crate::compositor::SceneNode)s, so every behavior is
//! testable headless. Line rendering is virtualized — only the lines
//! visible in the viewport (plus overscan) are shaped per frame.

mod clipboard;
mod config;
mod input;
mod view;

#[cfg(test)]
mod tests;

#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
pub use clipboard::SystemClipboard;
pub use clipboard::{ClipboardProvider, LocalClipboard};
pub use config::{EditorConfig, EditorTheme};
pub use input::MouseEvent;
pub use view::{EditorView, Preedit};
