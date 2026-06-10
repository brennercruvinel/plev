//! Multi-line, multi-cursor text editor widget (plan §WS-C fase 2).

mod clipboard;
mod config;

#[cfg(not(target_arch = "wasm32"))]
pub use clipboard::SystemClipboard;
pub use clipboard::{ClipboardProvider, LocalClipboard};
pub use config::{EditorConfig, EditorTheme};
