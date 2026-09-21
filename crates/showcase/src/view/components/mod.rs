//! One module per component page (`pages::Pane`, the Components group).
//! Each page owns its widgets and a content-driven layout, and exposes
//! `content_height`, `handle_event`, `render` and, where it animates,
//! `tick`. Nothing here restates a token: gaps come from `theme.spacing`,
//! sizes from the widgets' `preferred_size`.

pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod chart;
pub mod checkbox;
pub mod chip;
pub mod code_block;
pub mod context_menu;
pub mod dialog;
pub mod empty;
pub mod graph;
pub mod icon_button;
pub mod input;
pub mod navigation_menu;
pub mod panel;
pub mod panel_header;
pub mod progress;
pub mod resizable;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod sidebar;
pub mod skeleton;
pub mod slider;
pub mod spinner;
pub mod stat;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod toast;
pub mod tooltip;
pub mod tree;

pub(crate) use super::{EditKey, group_label, panel, text};

/// Tab-order bookkeeping for a page whose widgets take keyboard focus:
/// Tab walks `len` slots and wraps, Escape clears. The page maps the
/// slot onto its widgets' `set_focused`.
#[derive(Debug, Default)]
pub(crate) struct Focus {
    pub slot: Option<usize>,
}

impl Focus {
    /// The slot Tab lands on next.
    pub fn next(&self, len: usize) -> usize {
        self.slot.map_or(0, |i| (i + 1) % len)
    }

    /// Clear the focus. `false` when nothing was focused.
    pub fn clear(&mut self) -> bool {
        self.slot.take().is_some()
    }
}

/// Flow layout for a row of measured widgets: left to right from
/// `(content.x, y)`, wrapping at the content's right edge. Returns the
/// rects and the total height used.
pub(crate) fn flow(
    content: comps::prelude::Rect,
    y: f32,
    gap: f32,
    sizes: impl Iterator<Item = (f32, f32)>,
) -> (Vec<comps::prelude::Rect>, f32) {
    let mut rects = Vec::new();
    let mut x = content.x;
    let mut line_h: f32 = 0.0;
    let mut row_y = y;
    for (w, h) in sizes {
        if x > content.x && x + w > content.right() {
            x = content.x;
            row_y += line_h + gap;
            line_h = 0.0;
        }
        rects.push(comps::prelude::Rect::new(x, row_y, w, h));
        x += w + gap;
        line_h = line_h.max(h);
    }
    (rects, row_y + line_h - y)
}
