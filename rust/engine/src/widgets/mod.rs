//! comps: the plev design system.
//!
//! Retained widgets over the engine, one category per module, every
//! visual value read from [`crate::styling::theme::Theme`]. A widget is a plain
//! struct the app owns across frames: feed it [`WidgetEvent`]s with the
//! bounds you gave it, read the [`EventResult`], and call
//! `render(compositor, bounds, theme)` to emit scene nodes. Nothing here
//! touches the GPU, so every widget is testable without a window.
//!
//! Visual language: HOFF "dark glass" (monochrome white-on-graphite
//! alphas, pill buttons, top-lit edge borders, translucent surfaces),
//! resolved from `Theme` tokens (`theme.glass`, `theme.shape`,
//! `theme.control`, `theme.shadows`...). Non-HOFF palettes derive an
//! equivalent recipe, so every widget renders under every theme.
//!
//! Categories (docs/ref/component-catalog.md):
//! - [`buttons`]: button, icon button, chip, badge
//! - [`form`]: text field, checkbox, switch, slider, select
//! - [`nav`]: tabs, nav link, sidebar, panel header, breadcrumb, split
//!   pane
//! - [`content`]: card, avatar, panel, stat, code block, skeleton, table,
//!   empty state, virtual list, tree, separator
//! - [`feedback`]: modal, context menu, toast, tooltip, progress, spinner,
//!   scrollbar
//! - [`overlay`]: the overlay stack (menus, modals, toasts) with spring
//!   motion
//! - [`charts`]: pure chart geometry plus scene emission
//! - [`graph`]: node-link canvases, `GraphView` (a layout computed once)
//!   and `LiveGraph` (positions as state, d3's force simulation live)
//! - [`editor`]: multi-cursor text editor over `rope`
//! - [`shell`]: the app chrome (sidebar by breakpoint, header, content
//!   rect, safe area)
//! - [`icons`]: the lucide set, tessellated and cached
//! - [`core`] and [`recipe`]: the widget contract and the glass drawing
//!   recipes every category shares
//! - [`wake`]: what a tick asks of the loop (a frame now, the next
//!   deadline), and the caret clock the text widgets share

pub mod recipe;
pub mod wake;

pub mod icons;

pub mod content;
pub mod feedback;
pub mod form;
pub mod nav;

pub mod charts;
pub mod editor;
pub mod graph;
pub mod overlay;
pub mod shell;

pub mod buttons;
#[cfg(test)]
mod tests;
pub mod widget;

pub use crate::styling::theme::{self, Intent, Theme};
pub use wake::Wake;
pub use widget::{EventResult, Rect, WidgetEvent};

/// Everything an app screen usually needs, in one import.
pub mod prelude {
    pub use crate::styling::theme::{Intent, Theme};
    pub use crate::widgets::buttons::*;
    pub use crate::widgets::content::*;
    pub use crate::widgets::feedback::*;
    pub use crate::widgets::form::*;
    pub use crate::widgets::graph::{EdgeTone, GraphView, LiveGraph};
    pub use crate::widgets::nav::*;
    pub use crate::widgets::recipe::*;
    pub use crate::widgets::shell::{AppShell, SafeArea, ShellLayout};
    pub use crate::widgets::wake::Wake;
    pub use crate::widgets::widget::{EventResult, Rect, WidgetEvent};
}
