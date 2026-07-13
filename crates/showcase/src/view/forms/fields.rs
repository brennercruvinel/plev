//! TEXT FIELDS group: three single-line [`engine::text_input::TextInput`]s
//! restyled with HOFF glass tokens (the widget had zero consumers until
//! now), plus the live value preview line. Focus bookkeeping stays in
//! [`FormsSection`](super::FormsSection); this module owns the buffers,
//! the cursor blink and the scene generation.

use engine::compositor::Compositor;
use engine::text_input::TextInput;
use engine::theme::Theme;
use engine::ui::widgets::{Rect, focus_ring};

use super::super::text;
use super::EditKey;

/// Number of fields in the group.
pub const COUNT: usize = 3;
/// One style for layout AND drawing: `TextInput::build_scene` sizes the
/// field at `font_size * 2.0`, so the layout rect must use the same font.
pub const FIELD_FONT: f32 = 16.0;
pub const FIELD_H: f32 = FIELD_FONT * 2.0;
pub const FIELD_GAP: f32 = 12.0;
/// Inner horizontal padding `TextInput::build_scene` applies to its text;
/// clicks map through it so the caret lands on the clicked glyph.
const TEXT_PAD: f32 = 8.0;

pub struct TextFields {
    inputs: [TextInput; COUNT],
}

impl TextFields {
    pub fn new(theme: &Theme) -> Self {
        let accent = theme.colors.accent.0;
        let inputs = ["Full name", "Email address", "Project codename"].map(|placeholder| {
            let mut input = TextInput::new()
                .with_placeholder(placeholder)
                .with_font_size(FIELD_FONT)
                .with_text_color(theme.colors.text.0)
                .with_bg_color(theme.glass.field.0);
            input.placeholder_color = theme.glass.text_placeholder.0;
            input.cursor_color = accent;
            input.selection_color = [accent[0], accent[1], accent[2], 0.25];
            input
        });
        Self { inputs }
    }

    /// Field rects stacked under `(x, y)`, all `w` wide.
    pub fn rects(x: f32, y: f32, w: f32) -> [Rect; COUNT] {
        std::array::from_fn(|i| Rect::new(x, y + i as f32 * (FIELD_H + FIELD_GAP), w, FIELD_H))
    }

    pub fn focused(&self) -> Option<usize> {
        self.inputs.iter().position(|i| i.focused)
    }

    /// Buffer contents, exposed for the section tests.
    #[cfg(test)]
    pub fn value(&self, i: usize) -> &str {
        self.inputs[i].buffer.text()
    }

    /// Caret position (byte index), exposed for the section tests.
    #[cfg(test)]
    pub fn cursor(&self, i: usize) -> usize {
        self.inputs[i].buffer.cursor()
    }

    /// Focus exactly `target` (or nothing), unfocusing the rest.
    pub fn set_focused(&mut self, target: Option<usize>) {
        for (i, input) in self.inputs.iter_mut().enumerate() {
            if Some(i) == target {
                input.focus();
            } else {
                input.unfocus();
            }
        }
    }

    /// Click at `local_x` from the field's left edge: focus + caret there.
    pub fn click(&mut self, i: usize, local_x: f32) {
        self.inputs[i].handle_click(local_x - TEXT_PAD);
    }

    /// Type `s` into the focused field. `false` when nothing is focused.
    pub fn insert(&mut self, s: &str) -> bool {
        let Some(i) = self.focused() else {
            return false;
        };
        for c in s.chars() {
            self.inputs[i].handle_char(c);
        }
        true
    }

    /// Route a non-character editing key to the focused field.
    pub fn edit(&mut self, key: EditKey) -> bool {
        let Some(i) = self.focused() else {
            return false;
        };
        let input = &mut self.inputs[i];
        match key {
            EditKey::Backspace => input.handle_backspace(),
            EditKey::Delete => input.handle_delete(),
            EditKey::Left => input.handle_left(),
            EditKey::Right => input.handle_right(),
            EditKey::Home => input.handle_home(),
            EditKey::End => input.handle_end(),
            // Tab is focus traversal, owned by FormsSection.
            EditKey::Tab => return false,
        }
        true
    }

    /// Advance the cursor blink. `true` while a field is focused (the
    /// blink needs frames under render-on-demand).
    pub fn tick(&mut self, dt: f32) -> bool {
        match self.focused() {
            Some(i) => {
                self.inputs[i].tick(dt);
                true
            }
            None => false,
        }
    }

    /// The live preview line mirrored under the group.
    pub fn preview(&self) -> String {
        let filled: Vec<&str> = self
            .inputs
            .iter()
            .map(|i| i.buffer.text())
            .filter(|t| !t.is_empty())
            .collect();
        if filled.is_empty() {
            "live value: (empty)".to_string()
        } else {
            format!("live value: {}", filled.join(" / "))
        }
    }

    /// Fields + preview line. The focused field also gets the kit's
    /// accent focus ring so focus reads like every other form widget.
    pub fn render(&self, c: &mut Compositor, rects: &[Rect; COUNT], theme: &Theme) {
        for (input, rect) in self.inputs.iter().zip(rects) {
            if input.focused {
                c.push(focus_ring(*rect, theme.radius.sm, theme));
            }
            for node in input.build_scene(rect.x, rect.y, rect.w) {
                c.push(node);
            }
        }
        let last = rects[COUNT - 1];
        text(
            c,
            &self.preview(),
            12.0,
            500,
            last.x,
            last.y + last.h + 8.0,
            theme.colors.text_dim.0,
        );
    }
}
