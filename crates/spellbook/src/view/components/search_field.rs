//! Search field with a clear ("x") button. Port of
//! `desktop/ui/js/ui/search-field.js` (`attachClear`).
//!
//! Behavior carried over: the clear button only exists while there is
//! something to clear, clicking it empties the buffer and keeps the focus,
//! and Escape clears too. The screen's own handler reacts to the cleared
//! value the same way it reacts to typed edits - through `changed`, so a
//! clear is indistinguishable from deleting the text by hand, just as the
//! JS dispatched a real `input` event.

use engine::compositor::Compositor;
use engine::theme::Theme;
use engine::ui::icons;
use engine::ui::widgets::{EventResult, Rect, WidgetEvent, rounded_rect};

use super::super::EditKey;
use super::field::{FIELD_H, LabeledField};

/// Clear-button square inside the field.
const CLEAR: f32 = 24.0;

/// One search field. Unlabeled on purpose: the three screens that had one
/// labelled it in their own chrome ("Buscar…" placeholder), and the port
/// keeps it that way - use [`super::field::LabeledField`] when a form row
/// needs a label.
pub struct SearchField {
    pub field: LabeledField,
    hovered_clear: bool,
}

impl SearchField {
    pub fn new(placeholder: &str, theme: &Theme) -> Self {
        Self {
            field: LabeledField::new("", placeholder, theme),
            hovered_clear: false,
        }
    }

    pub fn value(&self) -> &str {
        self.field.value()
    }

    pub fn is_focused(&self) -> bool {
        self.field.input.focused
    }

    /// The plain field rect at `(x, y)`, full width.
    pub fn rect(&self, x: f32, y: f32, w: f32) -> Rect {
        Rect::new(x, y, w, FIELD_H)
    }

    fn clear_rect(&self, field: Rect) -> Rect {
        Rect::new(
            field.x + field.w - CLEAR - (field.h - CLEAR) / 2.0,
            field.y + (field.h - CLEAR) / 2.0,
            CLEAR,
            CLEAR,
        )
    }

    /// Focus + caret at the clicked glyph; a hit on the clear button clears.
    /// Returns the result plus whether the buffer changed.
    pub fn handle_event(&mut self, event: &WidgetEvent, field: Rect) -> EventResult {
        let clear = self.clear_rect(field);
        let has_value = !self.field.value().is_empty();
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let hovered = has_value && clear.contains(x, y);
                if hovered != self.hovered_clear {
                    self.hovered_clear = hovered;
                    return EventResult::changed();
                }
                EventResult::IGNORED
            }
            WidgetEvent::MouseDown { x, y } => {
                if has_value && clear.contains(x, y) {
                    self.field.set_value("");
                    // Clearing keeps the focus, as the JS did.
                    self.field.input.focus();
                    return EventResult::clicked();
                }
                if field.contains(x, y) {
                    self.field.click(x - field.x);
                    return EventResult::changed();
                }
                // A click elsewhere blurs.
                if self.field.input.focused {
                    self.field.input.unfocus();
                    return EventResult::changed();
                }
                EventResult::IGNORED
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Escape clears (the habit every search field trained), then the field
    /// keeps the focus for the next query.
    pub fn handle_escape(&mut self) -> bool {
        if self.field.input.focused && !self.field.value().is_empty() {
            self.field.set_value("");
            return true;
        }
        false
    }

    pub fn handle_text(&mut self, s: &str) -> bool {
        self.field.handle_text(s)
    }

    pub fn handle_edit_key(&mut self, key: EditKey) -> (bool, bool) {
        self.field.handle_edit_key(key)
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        self.field.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, field: Rect, theme: &Theme) {
        // Field body: glass pill + the input's own text/caret, focus-ringed.
        if self.field.input.focused {
            c.push(engine::ui::widgets::focus_ring(field, theme.radius.sm, theme));
        }
        c.push(rounded_rect(
            field.x,
            field.y,
            field.w,
            field.h,
            theme.radius.sm,
            theme.glass.field.0,
        ));
        for node in self.field.input.build_scene(field.x, field.y, field.w) {
            c.push(node);
        }

        // Clear button: visible only while there is something to clear.
        if !self.field.value().is_empty() {
            let clear = self.clear_rect(field);
            let fg = if self.hovered_clear {
                theme.colors.text.0
            } else {
                theme.glass.text_faint.0
            };
            if let Some(node) = icons::icon_at(
                "x",
                13.0,
                fg,
                clear.x + (clear.w - 13.0) / 2.0,
                clear.y + (clear.h - 13.0) / 2.0,
            ) {
                c.push(node);
            }
        }
    }
}
