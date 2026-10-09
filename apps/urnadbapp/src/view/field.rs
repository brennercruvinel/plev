//! `Field`: the app's single-line input, a thin alias over the design
//! system's [`TextField`] that keeps the app's own [`EditKey`] (with
//! Enter and Tab, which stay with the screen) and its focus/edit plumbing
//! the Open, Search and Chunks screens share.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

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
            EditKey::Backspace => engine::widgets::form::EditKey::Backspace,
            EditKey::Delete => engine::widgets::form::EditKey::Delete,
            EditKey::Left => engine::widgets::form::EditKey::Left,
            EditKey::Right => engine::widgets::form::EditKey::Right,
            EditKey::Home => engine::widgets::form::EditKey::Home,
            EditKey::End => engine::widgets::form::EditKey::End,
            EditKey::SelectLeft => engine::widgets::form::EditKey::SelectLeft,
            EditKey::SelectRight => engine::widgets::form::EditKey::SelectRight,
            EditKey::SelectHome => engine::widgets::form::EditKey::SelectHome,
            EditKey::SelectEnd => engine::widgets::form::EditKey::SelectEnd,
            EditKey::Enter | EditKey::Tab => return false,
        };
        self.input.edit(key)
    }

    /// Advance the cursor blink: while focused, a frame at each flip and
    /// the time to the next one; blurred, nothing.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.input.wake(dt)
    }

    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        self.input.render(c, rect, theme);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shift with the arrows, Home and End extends the selection, which
    /// the next deletion takes.
    #[test]
    fn the_select_keys_extend_the_selection() {
        let mut field = Field::new("query");
        field.input.focus();
        assert!(field.insert("hello"));
        assert!(field.edit(EditKey::SelectLeft));
        assert!(field.edit(EditKey::SelectLeft));
        assert!(field.edit(EditKey::Backspace));
        assert_eq!(field.text(), "hel");
        assert!(field.edit(EditKey::SelectHome));
        assert!(field.edit(EditKey::Delete));
        assert_eq!(field.text(), "");
    }
}
