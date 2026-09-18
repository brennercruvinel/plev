//! `Field`: a single-line [`TextInput`] restyled with HOFF glass tokens,
//! with the focus/click/edit plumbing the Open and Search screens share.
//! Mirrors the showcase's `forms/fields.rs` (one style for layout AND
//! drawing: `build_scene` sizes the field at `font_size * 2.0`).

use engine::compositor::Compositor;
use engine::text_input::TextInput;
use engine::theme::Theme;
use engine::ui::widgets::{Rect, focus_ring};

use super::EditKey;

/// One style for layout AND drawing: `TextInput::build_scene` sizes the
/// field at `font_size * 2.0`, so the layout rect must use the same font.
pub const FIELD_FONT: f32 = 16.0;
pub const FIELD_H: f32 = FIELD_FONT * 2.0;
/// Inner horizontal padding `build_scene` applies to its text; clicks map
/// through it so the caret lands on the clicked glyph.
const TEXT_PAD: f32 = 8.0;

pub struct Field {
    pub input: TextInput,
}

impl Field {
    pub fn new(placeholder: &str, theme: &Theme) -> Self {
        let accent = theme.colors.accent.0;
        let mut input = TextInput::new()
            .with_placeholder(placeholder)
            .with_font_size(FIELD_FONT)
            .with_text_color(theme.colors.text.0)
            .with_bg_color(theme.glass.field.0);
        input.placeholder_color = theme.glass.text_placeholder.0;
        input.cursor_color = accent;
        input.selection_color = [accent[0], accent[1], accent[2], 0.25];
        Self { input }
    }

    pub fn text(&self) -> &str {
        self.input.buffer.text()
    }

    pub fn is_empty(&self) -> bool {
        self.input.buffer.is_empty()
    }

    /// Click at `local_x` from the field's left edge: focus + caret there.
    pub fn click(&mut self, local_x: f32) {
        self.input.handle_click(local_x - TEXT_PAD);
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
    }

    /// Type `s` (characters or a pasted string). `false` when unfocused.
    pub fn insert(&mut self, s: &str) -> bool {
        if !self.input.focused {
            return false;
        }
        for c in s.chars() {
            self.input.handle_char(c);
        }
        true
    }

    /// Route a non-character editing key. `false` when unfocused or the
    /// key is not an editing key (Enter/Tab stay with the screen).
    pub fn edit(&mut self, key: EditKey) -> bool {
        if !self.input.focused {
            return false;
        }
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            EditKey::Enter | EditKey::Tab => return false,
        }
        true
    }

    /// Advance the cursor blink. `true` while focused (the blink needs
    /// frames under render-on-demand).
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.input.focused {
            self.input.tick(dt);
            true
        } else {
            false
        }
    }

    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        if self.input.focused {
            c.push(focus_ring(rect, theme.radius.sm, theme));
        }
        for node in self.input.build_scene(rect.x, rect.y, rect.w) {
            c.push(node);
        }
    }
}
