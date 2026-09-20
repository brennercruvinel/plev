//! HOFF text field: a single-line glass field (`glass.field` at rest,
//! `surface_active` hovered, the field-focus border while editing) at
//! the nav radius, one `Md` control tall, body text with the placeholder
//! in the placeholder tone, a caret in the accent and a selection wash.
//! The editing itself lives in [`TextInput`]; this widget adds the look,
//! the pointer contract and the focus ring every form control shares.
//!
//! One `TextStyle` (`theme.typography.body()`) measures the caret, hit
//! tests clicks and draws the text, so the caret always lands on the
//! glyph that was clicked.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, with_alpha};
use crate::form::text_input::TextInput;
use crate::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};

/// Non-character editing keys a field understands. Apps map their key
/// events onto these; Enter and Tab stay with the screen (submit and
/// focus traversal are not the field's business).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    SelectAll,
}

/// Retained single-line text field over a [`TextInput`].
#[derive(Clone, Debug)]
pub struct TextField {
    pub input: TextInput,
    pub disabled: bool,
    hovered: bool,
}

impl TextField {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            input: TextInput::new().with_placeholder(placeholder.as_ref()),
            disabled: false,
            hovered: false,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.input.buffer.set_text(text);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn text(&self) -> &str {
        self.input.buffer.text()
    }

    pub fn set_text(&mut self, text: &str) {
        self.input.buffer.set_text(text);
    }

    pub fn is_empty(&self) -> bool {
        self.input.buffer.is_empty()
    }

    pub fn is_focused(&self) -> bool {
        self.input.focused
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn focus(&mut self) {
        if !self.disabled {
            self.input.focus();
        }
    }

    pub fn unfocus(&mut self) {
        self.input.unfocus();
    }

    /// Focus on `true`, blur on `false` (the owner's focus chain calls
    /// this; a field never steals focus on its own).
    pub fn set_focused(&mut self, focused: bool) {
        if focused {
            self.focus();
        } else {
            self.unfocus();
        }
    }

    /// The text style the field measures, hit-tests and draws with.
    pub fn text_style(theme: &Theme) -> TextStyle {
        theme.typography.body()
    }

    /// Canonical field height: one `Md` control.
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Md)
    }

    /// Horizontal inset from the field edge to the text origin.
    fn pad_x(theme: &Theme) -> f32 {
        theme.spacing.md
    }

    /// Type `s` (characters or a pasted string). `false` when unfocused.
    pub fn insert(&mut self, s: &str) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        for c in s.chars() {
            self.input.handle_char(c);
        }
        true
    }

    /// Route a non-character editing key. `false` when unfocused.
    pub fn edit(&mut self, key: EditKey) -> bool {
        if !self.input.focused || self.disabled {
            return false;
        }
        match key {
            EditKey::Backspace => self.input.handle_backspace(),
            EditKey::Delete => self.input.handle_delete(),
            EditKey::Left => self.input.handle_left(),
            EditKey::Right => self.input.handle_right(),
            EditKey::Home => self.input.handle_home(),
            EditKey::End => self.input.handle_end(),
            EditKey::SelectAll => self.input.handle_select_all(),
        }
        true
    }

    /// IME commit / preedit, forwarded to the core.
    pub fn ime(&mut self, committed: &str, preedit: &str) {
        self.input.handle_ime(committed, preedit);
    }

    /// Advance the caret blink. `true` while focused (the blink needs
    /// frames under render-on-demand).
    pub fn tick(&mut self, dt: f32) -> bool {
        if self.input.focused {
            self.input.tick(dt);
            true
        } else {
            false
        }
    }

    /// Hover tracks the bounds; a press inside focuses and places the
    /// caret on the clicked glyph. A press outside blurs, so a screen
    /// with several fields needs no extra bookkeeping.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if self.disabled {
            if self.hovered || self.input.focused {
                self.hovered = false;
                self.input.unfocus();
                return EventResult::changed();
            }
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseMove { x, y } => {
                let inside = bounds.contains(x, y);
                if inside != self.hovered {
                    self.hovered = inside;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseDown { x, y } => {
                if bounds.contains(x, y) {
                    let style = Self::text_style(theme);
                    self.input
                        .handle_click(x - bounds.x - Self::pad_x(theme), &style);
                    EventResult::clicked()
                } else if self.input.focused {
                    self.input.unfocus();
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };
        let focused = self.input.focused;
        let style = Self::text_style(theme);
        let pad_x = Self::pad_x(theme);
        let radius = theme.shape.nav.min(bounds.h / 2.0);

        if focused {
            c.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }

        let bg = if focused || self.hovered {
            glass.surface_active
        } else {
            glass.field
        };
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                with_alpha(bg, bg.0[3] * alpha),
            ),
        );
        let edge = if focused {
            glass.field_focus_border
        } else {
            glass.edge_soft
        };
        c.push_to_layer(
            layer,
            rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                with_alpha(edge, edge.0[3] * alpha),
                theme.control.edge_width,
            ),
        );

        let text_x = bounds.x + pad_x;
        let text_y = bounds.y + TextMeasurer::vertical_center(&style, bounds.h);
        let text_w = (bounds.w - pad_x * 2.0).max(0.0);
        let text = self.input.buffer.text();

        // Selection wash under the text: the accent at the wash alpha.
        if focused && let Some((start, end)) = self.input.buffer.selection() {
            let (lo, hi) = if start <= end {
                (start, end)
            } else {
                (end, start)
            };
            let sel_x = TextMeasurer::cursor_x_styled(text, &style, None, lo);
            let sel_w = TextMeasurer::cursor_x_styled(text, &style, None, hi) - sel_x;
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: text_x + sel_x,
                    y: text_y,
                    w: sel_w,
                    h: style.line_height,
                    color: with_alpha(theme.colors.accent, glass.wash_alpha),
                },
            );
        }

        // Text, or the placeholder while empty.
        let (shown, color) = if text.is_empty() {
            (
                self.input.placeholder.as_str(),
                with_alpha(glass.text_placeholder, glass.text_placeholder.0[3] * alpha),
            )
        } else {
            (
                text,
                with_alpha(theme.colors.text, theme.colors.text.0[3] * alpha),
            )
        };
        if !shown.is_empty() {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(shown, &style, Some(text_w)),
                    x: text_x,
                    y: text_y,
                    color,
                },
            );
        }

        // Caret: the accent, one focus-ring stroke wide, the line box tall.
        if self.input.cursor_visible() {
            let caret_x =
                TextMeasurer::cursor_x_styled(text, &style, None, self.input.buffer.cursor());
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: text_x + caret_x,
                    y: text_y,
                    w: theme.control.focus_ring_width,
                    h: style.line_height,
                    color: theme.colors.accent.0,
                },
            );
        }
    }
}
