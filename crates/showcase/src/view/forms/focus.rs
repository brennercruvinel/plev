//! Section-local focus traversal for the Forms gallery. Tab walks the
//! focusable widgets in reading order (text fields first, then the
//! library controls; disabled ones are skipped), Escape blurs. This is
//! deliberately NOT a global navigation system: the section owns its
//! order and simply calls each widget's `set_focused`.

use super::{FormsSection, fields};

/// Non-character editing keys the platform shell forwards (winit-free so
/// headless tests can drive them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
}

/// Tab order length: the text fields, then tabs, two enabled checkboxes,
/// two enabled switches, two enabled sliders and the select. Disabled
/// controls (locked checkbox/switch, disabled slider) are not focusable,
/// and progress bars are not interactive.
const TAB_ORDER: usize = fields::COUNT + 8;

impl FormsSection {
    /// Focus exactly one tab-order slot (or none), blurring the rest.
    pub(super) fn set_focus(&mut self, target: Option<usize>) {
        self.focus = target;
        let f = fields::COUNT;
        self.fields.set_focused(target.filter(|i| *i < f));
        self.tabs.set_focused(target == Some(f));
        self.autosave.set_focused(target == Some(f + 1));
        self.telemetry.set_focused(target == Some(f + 2));
        self.focus_mode.set_focused(target == Some(f + 3));
        self.wrap_lines.set_focused(target == Some(f + 4));
        self.volume.set_focused(target == Some(f + 5));
        self.steps.set_focused(target == Some(f + 6));
        self.select.set_focused(target == Some(f + 7));
    }

    /// Current tab-order slot (text fields are slots `0..fields::COUNT`),
    /// exposed for the section tests.
    #[cfg(test)]
    pub fn focus_index(&self) -> Option<usize> {
        self.focus
    }

    /// Type characters into the focused text field.
    pub fn handle_text(&mut self, s: &str) -> bool {
        self.fields.insert(s)
    }

    /// Tab cycles the section's tab order; the other editing keys go to
    /// the focused text field.
    pub fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            let next = self.focus.map_or(0, |i| (i + 1) % TAB_ORDER);
            self.set_focus(Some(next));
            return true;
        }
        self.fields.edit(key)
    }

    /// Escape blurs whatever is focused. `false` when nothing was.
    pub fn handle_escape(&mut self) -> bool {
        if self.focus.is_some() {
            self.set_focus(None);
            true
        } else {
            false
        }
    }
}
