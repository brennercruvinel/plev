//! TEXT FIELDS group: three single-line [`comps::form::TextField`]s plus
//! the live value preview line. Focus bookkeeping stays in
//! [`FormsSection`](super::FormsSection); this module owns the fields,
//! the cursor blink and the scene generation.

use comps::form::{EditKey, TextField};
use comps::prelude::Rect;
use engine::compositor::Compositor;
use engine::theme::Theme;

use super::super::text;

/// Number of fields in the group.
pub const COUNT: usize = 3;

pub struct TextFields {
    fields: [TextField; COUNT],
}

impl TextFields {
    pub fn new() -> Self {
        let fields = ["Full name", "Email address", "Project codename"].map(TextField::new);
        Self { fields }
    }

    /// Field rects stacked under `(x, y)`, all `w` wide, one field
    /// height each with the `md` spacing between them.
    pub fn rects(x: f32, y: f32, w: f32, theme: &Theme) -> [Rect; COUNT] {
        let h = TextField::height(theme);
        let gap = theme.spacing.md;
        std::array::from_fn(|i| Rect::new(x, y + i as f32 * (h + gap), w, h))
    }

    /// Bottom edge of the group (fields plus the preview line).
    pub fn bottom(rects: &[Rect; COUNT], theme: &Theme) -> f32 {
        let last = rects[COUNT - 1];
        last.y + last.h + theme.spacing.sm + theme.typography.caption_sm().line_height
    }

    pub fn focused(&self) -> Option<usize> {
        self.fields.iter().position(|f| f.is_focused())
    }

    /// Buffer contents, exposed for the section tests.
    #[cfg(test)]
    pub fn value(&self, i: usize) -> &str {
        self.fields[i].text()
    }

    /// Caret position (byte index), exposed for the section tests.
    #[cfg(test)]
    pub fn cursor(&self, i: usize) -> usize {
        self.fields[i].input.buffer.cursor()
    }

    /// Focus exactly `target` (or nothing), unfocusing the rest.
    pub fn set_focused(&mut self, target: Option<usize>) {
        for (i, field) in self.fields.iter_mut().enumerate() {
            field.set_focused(Some(i) == target);
        }
    }

    /// Route a pointer event to the field at `i` (focus + caret on a
    /// press inside).
    pub fn handle_event(
        &mut self,
        i: usize,
        event: &comps::prelude::WidgetEvent,
        rect: Rect,
        theme: &Theme,
    ) -> comps::prelude::EventResult {
        self.fields[i].handle_event(event, rect, theme)
    }

    /// Type `s` into the focused field. `false` when nothing is focused.
    pub fn insert(&mut self, s: &str) -> bool {
        match self.focused() {
            Some(i) => self.fields[i].insert(s),
            None => false,
        }
    }

    /// Route a non-character editing key to the focused field.
    pub fn edit(&mut self, key: EditKey) -> bool {
        match self.focused() {
            Some(i) => self.fields[i].edit(key),
            None => false,
        }
    }

    /// Advance the cursor blink. `true` while a field is focused (the
    /// blink needs frames under render-on-demand).
    pub fn tick(&mut self, dt: f32) -> bool {
        match self.focused() {
            Some(i) => self.fields[i].tick(dt),
            None => false,
        }
    }

    /// The live preview line mirrored under the group.
    pub fn preview(&self) -> String {
        let filled: Vec<&str> = self
            .fields
            .iter()
            .map(|f| f.text())
            .filter(|t| !t.is_empty())
            .collect();
        if filled.is_empty() {
            "live value: (empty)".to_string()
        } else {
            format!("live value: {}", filled.join(" / "))
        }
    }

    /// Fields + preview line.
    pub fn render(&self, c: &mut Compositor, rects: &[Rect; COUNT], theme: &Theme) {
        for (field, rect) in self.fields.iter().zip(rects) {
            field.render(c, *rect, theme);
        }
        let last = rects[COUNT - 1];
        let style = theme.typography.caption_sm();
        text(
            c,
            &self.preview(),
            style.font_size,
            style.font_weight,
            last.x,
            last.y + last.h + theme.spacing.sm,
            theme.colors.text_dim.0,
        );
    }
}
