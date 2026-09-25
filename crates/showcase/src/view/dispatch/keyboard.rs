//! The keyboard on the component pages (task-75 phase 2c): Tab and
//! Shift-Tab walk the projection in the order a screen reader walks it.
//!
//! There is one order and it is the projection's: `focus::next` over the
//! view's snapshot (`ShowcaseView::semantics`: the page, the toasts and
//! the overlay that holds the pointer) names the next node, and the page
//! takes it through `assist_focus`, the way it takes the focus a screen
//! reader moves.

use engine::accessibility::focus::{self, Direction};

use super::super::ShowcaseView;

/// A key the keyboard walks the projection with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::view) enum Walk {
    /// Tab, or Shift-Tab while the shift key is held.
    Tab,
}

impl ShowcaseView {
    /// Whether the keyboard walks the projection: on a component page, or
    /// while an overlay holds the pointer (the modal, the menu).
    pub(in crate::view) fn walks(&self) -> bool {
        self.overlay_holds_pointer()
            || self
                .pane()
                .is_some_and(|pane| self.panes.component(pane).is_some())
    }

    /// One key over the projection as it stands: `true` when it moved the
    /// focus.
    pub(in crate::view) fn walk(&mut self, key: Walk) -> bool {
        let Some(seen) = self.semantics() else {
            return false;
        };
        match key {
            Walk::Tab => {
                let direction = if self.shift {
                    Direction::Previous
                } else {
                    Direction::Next
                };
                focus::next(&seen, direction).is_some_and(|id| self.assist_focus(&id))
            }
        }
    }
}
