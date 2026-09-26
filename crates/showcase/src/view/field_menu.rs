//! The text field's menu (task-74, the person's request): a right-click
//! on a field of the Input page gives the field the keys and opens, at
//! the pointer, the context menu the Context Menu page opens: the same
//! comps menu, the same overlay and projection (`context_menu`), so its
//! keyboard is that menu's (the arrows, Enter, Escape). Its items are
//! Cut, Copy, Paste and Select all; one with nothing to act on (Copy with
//! no selection, Paste with an empty clipboard) is disabled.
//!
//! A pick is the person's clipboard key on the field that has the keys
//! (`clipboard.rs`), whether it came from the pointer, the keyboard or a
//! screen reader; an agent's pick is refused, since the clipboard is the
//! person's (`dispatch/overlay.rs`).

use comps::form::ClipKey;
use comps::form::text_input::single_line;
use comps::prelude::{ContextMenu, MenuEntry};

use super::clipboard::{Keys, board};
use super::{Pane, ShowcaseView};

/// The items' ids, apart from the Context Menu page's demo items.
const CUT: u64 = 101;
const COPY: u64 = 102;
const PASTE: u64 = 103;
const SELECT_ALL: u64 = 104;

/// The clipboard key an item of the field's menu names; `None` for any
/// other menu's item.
pub(in crate::view) fn clip_of(item: u64) -> Option<ClipKey> {
    Some(match item {
        CUT => ClipKey::Cut,
        COPY => ClipKey::Copy,
        PASTE => ClipKey::Paste,
        SELECT_ALL => ClipKey::SelectAll,
        _ => return None,
    })
}

/// The label of the item for `key`.
pub(in crate::view) fn label(key: ClipKey) -> &'static str {
    match key {
        ClipKey::Cut => "Cut",
        ClipKey::Copy => "Copy",
        ClipKey::Paste => "Paste",
        ClipKey::SelectAll => "Select all",
    }
}

/// The menu, each item enabled when it has something to act on.
fn menu(selected: bool, pasteable: bool, text: bool) -> ContextMenu {
    let item = |id, key, on: bool| MenuEntry::item(id, label(key)).disabled(!on);
    ContextMenu::new(vec![
        item(CUT, ClipKey::Cut, selected),
        item(COPY, ClipKey::Copy, selected),
        item(PASTE, ClipKey::Paste, pasteable),
        MenuEntry::Separator,
        item(SELECT_ALL, ClipKey::SelectAll, text),
    ])
}

impl ShowcaseView {
    /// A right-click at `(x, y)`: on a field of the active page, the
    /// field takes the keys and its menu opens there. `false` off the
    /// fields, or while an overlay holds the pointer (one fading out
    /// gives way).
    pub(in crate::view) fn open_field_menu(&mut self, x: f32, y: f32) -> bool {
        if self.overlay_holds_pointer() {
            return false;
        }
        let content = self.page_rect();
        let pane = self.pane();
        let took = match pane {
            Some(Pane::Input) => self.panes.input.right_click(x, y, content, &self.theme),
            _ => false,
        };
        if !took {
            return false;
        }
        let held = board(&mut self.clipboard).get_text().unwrap_or_default();
        let pasteable = !single_line(&held).is_empty();
        let widget = match self.panes.keys(pane) {
            Keys::Field(field) => menu(field.has_selection(), pasteable, !field.is_empty()),
            _ => return false,
        };
        self.show_menu(widget, x, y);
        true
    }
}
