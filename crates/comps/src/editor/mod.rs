//! Multi-line, multi-cursor text editor widget.
//!
//! Built on the headless foundations:
//! - [`rope`]: rope document, transactional edits, multi-cursor
//!   selections and undo history;
//! - [`TextMeasurer`](engine::text::TextMeasurer): GPU-free shaping for
//!   hit-testing and caret geometry;
//! - the compositor scene graph for rendering.
//!
//! The widget itself stays GPU-free: [`EditorView::render`] only emits
//! [`SceneNode`](engine::compositor::SceneNode)s, so every behavior is
//! testable headless. Line rendering is virtualized: only the lines
//! visible in the viewport (plus overscan) are shaped per frame. The
//! clipboard providers live in [`engine::clipboard`].

mod config;
mod input;
mod view;

#[cfg(test)]
mod tests;

pub use config::{EditorConfig, EditorTheme};
pub use input::MouseEvent;
pub use view::{EditorView, Preedit};
