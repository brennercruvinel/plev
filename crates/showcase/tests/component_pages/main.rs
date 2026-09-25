//! The component pages as a person, a screen reader and an agent meet them
//! (task-75 phase 2b): for each kind of widget, the same request from the
//! three, each on a gallery of its own, ends in the same state, the page's
//! projection node for node.
//!
//! - the person clicks where the widget is bound in the frame (or types),
//!   through `ShowcaseView::handle_event` and `handle_key`, as the window
//!   delivers it;
//! - the reader acts on the tree the shell's bridge builds from the frame,
//!   read back by `accessibility::translate` and run by `act`, as
//!   `Actor::Assistive`;
//! - the agent connects to a real agent port in a directory of its own,
//!   observes the gallery and acts from the revision it saw, from a thread
//!   of its own, while this thread serves the port as the shell does.
//!
//! Nothing here opens a window, a dialog of the system or a device.
#![cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]

mod harness;

/// Buttons: a press, and the focus a reader leaves.
mod buttons;

/// Toggles and values: a checkbox, a switch, a chip, a slider, a field, a
/// select.
mod values;

/// Choices in a collection: tabs, rows, items, links, crumbs, a tree, a
/// split.
mod choices;

/// The privacy promises: no case on these pages, guarded.
mod promises;
