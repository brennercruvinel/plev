//! Labeled single-line text field wrapping `engine::text_input::TextInput`.
//!
//! Provides label above, field below, focus ring when focused, and
//! integrates with the shell's keyboard routing (EditKey, handle_text).

use engine::compositor::{Compositor, Rect};
use engine::text_input::TextInput;
use engine::theme::Theme;
use engine::ui::widgets::{EventResult, Rect as WidgetRect, WidgetEvent, focus_ring};

use super::{EditKey, ScreenCtx};
use crate::art::ArtCache;

/// A labeled text field with optional placeholder, focus handling,
/// and integration with the shell's keyboard routing.
pub struct LabeledField {
    label: String,
    input: TextInput,
    focused: bool,
    changed: bool,
}

impl LabeledField {
    pub fn new(
        label: impl Into<String>,
        placeholder: impl Into<String>,
        theme: &Theme,
    ) -> Self {
        let mut input = TextInput::new()
            .with_placeholder(placeholder)
            .with_font_size(16.0)
            .with_text_color(theme.colors.text.0)
            .with_bg_color(theme.glass.field.0);
        input.placeholder_color = theme.glass.text_placeholder.0;
        input.cursor_color = theme.colors.accent.0;
        input.selection_color = [
            theme.colors.accent.0[0],
            theme.colors.accent.0[1],
            theme.colors.accent.0[2],
            0.25,
        ];

        Self {
            label: label.into(),
            input,
            focused: false,
            changed: false,
        }
    }

    pub fn with_font_size(mut self, size: f32) -> Self {
        self.input = self.input.with_font_size(size);
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.input.set_text(&value.into());
        self
    }

    pub fn value(&self) -> &str {
        self.input.buffer.text()
    }

    pub fn set_value(&mut self, value: &str) {
        self.input.set_text(value);
    }

    pub fn is_empty(&self) -> bool {
        self.input.buffer.is_empty()
    }

    pub fn focus(&mut self) {
        self.input.focus();
        self.focused = true;
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
        self.focused = false;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Handle keyboard characters. Returns true if consumed.
    pub fn handle_text(&mut self, s: &str) -> bool {
        if self.focused {
            for c in s.chars() {
                self.input.handle_char(c);
            }
            self.changed = true;
            true
        } else {
            false
        }
    }

    pub fn handle_edit_key(&mut self, key: super::EditKey) -> bool {
        if !self.focused {
            return false;
        }
        match key {
            super::EditKey::Backspace => self.input.handle_backspace(),
            super::EditKey::Delete => self.input.handle_delete(),
            super::EditKey::Left => self.input.handle_left(),
            super::EditKey::Right => self.input.handle_right(),
            super::EditKey::Home => self.input.handle_home(),
            super::EditKey::End => self.input.handle_end(),
            _ => false,
        };
        self.changed = true;
        true
    }

    /// Handle mouse click on the field. `local_x` is relative to field left edge.
    pub fn handle_click(&mut self, local_x: f32) {
        self.focus();
        self.input.handle_click(local_x - 8.0); // TEXT_PAD = 8.0
    }

    /// Advance cursor blink. Returns true while focused (needs frames).
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.focused {
            self.input.tick(dt);
            true
        } else {
            false
        }
    }

    /// Check if field was modified since last check.
    pub fn take_changed(&mut self) -> bool {
        let c = self.changed;
        self.changed = false;
        c
    }

    pub fn set_focused(&mut self, focused: bool) {
        if focused {
            self.focus();
        } else {
            self.unfocus();
        }
    }

    pub fn render(
        &mut self,
        c: &mut Compositor,
        rect: Rect,
        theme: &Theme,
        _art: &mut ArtCache,
    ) {
        // Label
        let label_y = rect.y;
        engine::ui::widgets::text(
            c,
            &self.label,
            12.0,
            600,
            rect.x,
            label_y,
            theme.glass.text_placeholder.0,
        );

        // Field background
        let field_rect = Rect::new(
            rect.x,
            rect.y + 20.0,
            rect.w,
            36.0,
        );
        c.push(engine::ui::widgets::rounded_rect(
            field_rect.x,
            field_rect.y,
            field_rect.w,
            field_rect.h,
            theme.radius.sm,
            self.input.bg_color,
        ));

        // Focus ring
        if self.focused {
            c.push(engine::ui::widgets::focus_ring(
                field_rect,
                theme.radius.sm,
                theme,
            ));
        }

        // Text input scene
        for node in self.input.build_scene(field_rect.x, field_rect.y, field_rect.w) {
            c.push(node);
        }
    }

    /// Handle events (mouse/keyboard). Returns true if consumed.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        rect: Rect,
        _ctx: &mut super::ScreenCtx,
    ) -> bool {
        let field_rect = Rect::new(rect.x, rect.y + 20.0, rect.w, 36.0);

        match event {
            WidgetEvent::MouseDown { x, y } => {
                if field_rect.contains(x, y) {
                    self.handle_click(x - field_rect.x);
                    true
                } else if self.focused {
                    // Click outside blurs
                    self.unfocus();
                    false
                } else {
                    false
                }
            }
            WidgetEvent::MouseMove { .. } => false,
            _ => false,
        }
    }
}

impl LabeledField {
    pub fn unfocus(&mut self) {
        self.input.unfocus();
        self.focused = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_creation() {
        let theme = crate::engine::theme::Theme::hoff();
        let field = LabeledField::new("Nome", "Digite o nome...", &theme);
        assert_eq!(field.value(), "");
        assert!(!field.is_focused());
    }
}