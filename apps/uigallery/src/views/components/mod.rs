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
/// The showcase mold the kit's component pages share (groups, cards,
/// chips), from the ButtonsPage.
pub(crate) mod showcase;
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

// task-104: each front adds its page modules under its anchor, a group of
// its own (the blank lines keep rustfmt from sorting one front into another)
// task-104 front A, fields (#154)
pub mod field;
pub mod input_group;
pub mod label;

// task-104 front B, overlays (#155)
pub mod alert_dialog;
pub mod drawer;
pub mod dropdown_menu;
pub mod hover_card;
pub mod menubar;
pub mod popover;
pub mod sheet;

// task-104 front C, conversation and feedback (#156)
pub mod alert;
pub mod attachment;
pub mod bubble;
pub mod marker;
pub mod message;
pub mod message_scroller;

// task-104 front D, disclosure and display (#157)
pub mod accordion;
pub mod aspect_ratio;
pub mod carousel;
pub mod collapsible;
pub mod item;
pub mod kbd;
pub mod pagination;
pub mod table_static;

// task-104 front E, selection and dates (#158)
pub mod button_group;
pub mod calendar;
pub mod radio_group;
pub mod toggle;
pub mod toggle_group;

pub(in crate::views) use super::dispatcher::component::{After, Component, under};
pub(crate) use super::{EditKey, group_label, panel, text};

use accord::SemId;

/// A component page's root id, `<name>` (`checkbox_page`); `None`, logged,
/// when the name does not parse, which each page's test proves it does.
pub(crate) fn root_id(name: &str) -> Option<SemId> {
    SemId::name(name)
        .inspect_err(|e| log::warn!("{name}: {e}"))
        .ok()
}

/// The key of `id` under `parent` (`3` for `<parent>[3]`), which the
/// comps gives the children it projects; `None` for any other id.
pub(crate) fn key_under<'a>(parent: &SemId, id: &'a SemId) -> Option<&'a str> {
    id.as_str()
        .strip_prefix(parent.as_str())?
        .strip_prefix('[')?
        .strip_suffix(']')
}

/// `<root>.<name>`, a widget's id or the stem of a keyed family of them;
/// `None` without a root, or, logged, when the name does not parse.
pub(crate) fn child_id(root: Option<&SemId>, name: &str) -> Option<SemId> {
    root?
        .child(name)
        .inspect_err(|e| log::warn!("{name}: {e}"))
        .ok()
}

/// The keyboard focus of a page whose widgets take it: the slot of the
/// widget that has it, which the page maps onto its widgets'
/// `set_focused`. It holds no order: Tab walks the page's projection
/// (`dispatch/keyboard.rs`) and lands through `Component::focus`; Escape
/// clears.
#[derive(Debug, Default)]
pub(crate) struct Focus {
    pub slot: Option<usize>,
}

impl Focus {
    /// Clear the focus. `false` when nothing was focused.
    pub fn clear(&mut self) -> bool {
        self.slot.take().is_some()
    }
}

/// Flow layout for a row of measured widgets: left to right from
/// `(content.x, y)`, wrapping at the content's right edge. Returns the
/// rects and the total height used.
pub(crate) fn flow(
    content: engine::widgets::prelude::Rect,
    y: f32,
    gap: f32,
    sizes: impl Iterator<Item = (f32, f32)>,
) -> (Vec<engine::widgets::prelude::Rect>, f32) {
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
        rects.push(engine::widgets::prelude::Rect::new(x, row_y, w, h));
        x += w + gap;
        line_h = line_h.max(h);
    }
    (rects, row_y + line_h - y)
}
