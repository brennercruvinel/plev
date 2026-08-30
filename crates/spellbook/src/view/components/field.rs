//! Labeled single-line text field: the `<label>` + `<input>` pair the old
//! forms wrote inline everywhere, wrapped around
//! `engine::text_input::TextInput`.
//!
//! The field owns only its buffer and its focus. Char/edit-key/pointer
//! routing is the owner's job (the focus chain is the screen's, per plev's
//! official app pattern); the answer a caller wants - "the text changed" -
//! is returned by [`LabeledField::handle_text`] / [`LabeledField::edit`].

use engine::compositor::Compositor;
use engine::text_input::TextInput;
use engine::theme::Theme;
use engine::ui::widgets::{Rect, focus_ring, rounded_rect};

use super::super::{EditKey, text};

/// Same sizing the showcase fields use: `TextInput::build_scene` draws the
/// field at `font_size * 2.0`, so layout and drawing share one constant.
pub const FIELD_FONT: f32 = 14.0;
pub const FIELD_H: f32 = FIELD_FONT * 2.0 + 8.0;
/// Label above, 4px gap (CSS: `margin-top:5px` on the input).
pub const LABEL_H: f32 = 20.0;
/// Inner horizontal padding `TextInput::build_scene` applies; clicks map
/// through it so the caret lands on the clicked glyph.
const TEXT_PAD: f32 = 8.0;

pub struct LabeledField {
    pub label: String,
    pub input: TextInput,
}

impl LabeledField {
    pub fn new(label: impl Into<String>, placeholder: &str, theme: &Theme) -> Self {
        let accent = theme.colors.accent.0;
        let mut input = TextInput::new()
            .with_placeholder(placeholder)
            .with_font_size(FIELD_FONT)
            .with_text_color(theme.colors.text.0)
            .with_bg_color(theme.glass.field.0);
        input.placeholder_color = theme.glass.text_placeholder.0;
        input.cursor_color = accent;
        input.selection_color = [accent[0], accent[1], accent[2], 0.25];
        Self {
            label: label.into(),
            input,
        }
    }

    /// Total height the pair occupies.
    pub fn height() -> f32 {
        LABEL_H + FIELD_H
    }

    /// Field rect (below the label) inside the rect the pair was given.
    pub fn field_rect(&self, rect: Rect) -> Rect {
        Rect::new(rect.x, rect.y + LABEL_H, rect.w, rect.h - LABEL_H)
    }

    pub fn value(&self) -> &str {
        self.input.buffer.text()
    }

    /// Trimmed contents, empty string becoming `None` - the shape every
    /// optional form field on the old UI sent (`value.trim() || null`).
    pub fn value_opt(&self) -> Option<String> {
        let v = self.value().trim();
        (!v.is_empty()).then(|| v.to_string())
    }

    pub fn set_value(&mut self, s: &str) {
        self.input.buffer.set_text(s);
    }

    /// Focus + caret at the clicked glyph. `x` is the field's left edge.
    pub fn click(&mut self, local_x: f32) {
        self.input.handle_click(local_x - TEXT_PAD);
    }

    /// Insert text when focused. Returns `true` when the buffer changed -
    /// under render-on-demand this is the redraw signal.
    pub fn handle_text(&mut self, s: &str) -> bool {
        if !self.input.focused {
            return false;
        }
        let before = self.input.buffer.text().to_string();
        for c in s.chars() {
            self.input.handle_char(c);
        }
        self.input.buffer.text() != before
    }

    /// Route a non-character editing key. Returns `(consumed, changed)`.
    pub fn handle_edit_key(&mut self, key: EditKey) -> (bool, bool) {
        if !self.input.focused {
            return (false, false);
        }
        let before = self.input.buffer.text().to_string();
        let cursor = self.input.buffer.cursor();
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            // Enter and Tab are the owner's concern (submit / traversal).
            EditKey::Enter | EditKey::Tab => return (false, false),
        }
        (
            true,
            self.input.buffer.text() != before || self.input.buffer.cursor() != cursor,
        )
    }

    /// Cursor blink: `true` while focused (frames keep being needed).
    pub fn tick(&mut self, dt: f32) -> bool {
        self.input.tick(dt);
        self.input.focused
    }

    /// Label above, field below; accent ring when focused.
    pub fn render(&self, c: &mut Compositor, rect: Rect, theme: &Theme) {
        text(
            c,
            &self.label,
            12.0,
            400,
            rect.x,
            rect.y,
            theme.colors.text_dim.0,
        );
        let field = self.field_rect(rect);
        if self.input.focused {
            c.push(focus_ring(field, theme.radius.sm, theme));
        }
        // TextInput::build_scene draws a square-cornered background; round
        // it out first so the glass field reads like every other pill.
        c.push(rounded_rect(
            field.x,
            field.y,
            field.w,
            field.h,
            theme.radius.sm,
            theme.glass.field.0,
        ));
        for node in self.input.build_scene(field.x, field.y, field.w) {
            c.push(node);
        }
    }
}
