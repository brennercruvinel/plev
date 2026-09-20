//! `Field`: the app's single-line input, a thin alias over the design
//! system's [`TextField`] that keeps the app's own [`EditKey`] (with
//! Enter and Tab, which stay with the screen) and its focus/edit plumbing
//! the Open, Search and Chunks screens share.

use comps::form::TextField;
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::EditKey;

/// Canonical field height for a theme (one `Md` control).
pub fn field_h(theme: &Theme) -> f32 {
    TextField::height(theme)
}

pub struct Field {
    pub input: TextField,
}

impl Field {
    pub fn new(placeholder: &str) -> Self {
        Self {
            input: TextField::new(placeholder),
        }
    }

    pub fn text(&self) -> &str {
        self.input.text()
    }

    pub fn is_empty(&self) -> bool {
        self.input.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    /// A press inside focuses and places the caret; a press outside blurs.
    pub fn handle_event(&mut self, event: &WidgetEvent, rect: Rect, theme: &Theme) -> EventResult {
        self.input.handle_event(event, rect, theme)
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
    }

    /// Type `s` (characters or a pasted string). `false` when unfocused.
    pub fn insert(&mut self, s: &str) -> bool {
        self.input.insert(s)
    }

    /// Route a non-character editing key. `false` when unfocused or the
    /// key is not an editing key (Enter/Tab stay with the screen).
    pub fn edit(&mut self, key: EditKey) -> bool {
        let key = match key {
            EditKey::Backspace => comps::form::EditKey::Backspace,
            EditKey::Delete => comps::form::EditKey::Delete,
            EditKey::Left => comps::form::EditKey::Left,
            EditKey::Right => comps::form::EditKey::Right,
            EditKey::Home => comps::form::EditKey::Home,
            EditKey::End => comps::form::EditKey::End,
            EditKey::Enter | EditKey::Tab => return false,
        };
        self.input.edit(key)
    }

    /// Advance the cursor blink. `true` while focused (the blink needs
    /// frames under render-on-demand).
    pub fn tick(&mut self, dt: f32) -> bool {
        self.input.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.input.render(c, rect, theme);
    }
}
