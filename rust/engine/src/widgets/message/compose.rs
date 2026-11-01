//! The line a person composes in: the bare text of the kit's prompt and
//! comment boxes (`PromptInput`, `NewComment`, `Comment/NewMessage`,
//! `QuickComment/Comment`), with no box of its own, the box being the
//! composer's. One line, as the kit's prompt (`!h-6`): a text wider than
//! it slides to keep the caret in while focused, and is cut with an
//! ellipsis at rest.
//!
//! The editing is the text field's ([`TextField`], its [`TextInput`]
//! core): typing, the editing keys, the clipboard, the input method and
//! the caret blink reach it as they reach any field. What is the line's
//! own is the look: the run it draws, measures and hit-tests with is the
//! composer's token (a [`MessageText`]) and the placeholder's color too;
//! the caret, the selection and the composition's underline are the bare
//! field's (`theme.form.bare`).
//!
//! Projected as the field projects ([`TextField::project`]): a text input
//! with its `text` and `set_value`.
//!
//! [`TextInput`]: crate::widgets::form::text_input::TextInput

use accord::{NodeBuilder, SemId, SnapshotBuilder};

use super::tokens::MessageText;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::text_input::view::LineFit;
use crate::widgets::form::tokens::Look;
use crate::widgets::form::{EditKey, TextField};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, with_alpha};

/// What a composer draws its line with: the run and the placeholder's
/// color, from its own tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineLook {
    pub text: MessageText,
    pub placeholder: Color,
    /// The alpha it draws at (a composer that leaves it out fades it).
    pub alpha: f32,
}

#[derive(Clone, Debug)]
pub struct ComposeLine {
    field: TextField,
}

impl ComposeLine {
    pub fn new(placeholder: impl AsRef<str>) -> Self {
        Self {
            field: TextField::new(placeholder).look(Look::Bare),
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.field.set_text(text);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.field = self.field.sem(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.field.sem_id()
    }

    pub fn text(&self) -> &str {
        self.field.text()
    }

    pub fn placeholder(&self) -> &str {
        &self.field.input.placeholder
    }

    /// The placeholder, as a composer's state changes it (voice input
    /// asks something else).
    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.field.input.placeholder = placeholder.to_owned();
    }

    pub fn is_empty(&self) -> bool {
        self.field.is_empty()
    }

    /// The text, as a reader or an agent sets it. `false` when refused.
    pub fn set_value(&mut self, text: &str) -> bool {
        !self.field.disabled && self.field.set_value(text)
    }

    pub fn clear(&mut self) {
        self.field.set_text("");
    }

    pub fn disabled(&self) -> bool {
        self.field.disabled
    }

    /// Left out (voice input): it takes no press, no key and no focus.
    pub fn set_disabled(&mut self, on: bool) {
        self.field.disabled = on;
        if on {
            self.field.unfocus();
        }
    }

    pub fn is_focused(&self) -> bool {
        self.field.is_focused()
    }

    pub fn set_focused(&mut self, on: bool) {
        self.field.set_focused(on);
    }

    /// The field under the line, for the keys a page routes to it (a
    /// clipboard chord, the field's menu).
    pub fn field_mut(&mut self) -> &mut TextField {
        &mut self.field
    }

    pub fn insert(&mut self, s: &str) -> bool {
        self.field.insert(s)
    }

    pub fn edit(&mut self, key: EditKey) -> bool {
        self.field.edit(key)
    }

    pub fn compose(
        &mut self,
        committed: &str,
        preedit: &str,
        cursor: Option<(usize, usize)>,
    ) -> bool {
        self.field.compose(committed, preedit, cursor)
    }

    pub fn end_compose(&mut self) -> bool {
        self.field.end_compose()
    }

    pub fn has_selection(&self) -> bool {
        self.field.has_selection()
    }

    /// The caret's blink while focused.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.field.wake(dt)
    }

    fn style(look: &LineLook, theme: &Theme) -> TextStyle {
        look.text.style(&theme.typography)
    }

    /// The line's height: its run's.
    pub fn height(look: &LineLook, theme: &Theme) -> f32 {
        look.text.line_height(&theme.typography)
    }

    /// How the text sits in a box `w` wide (the field's rule).
    fn fit(&self, style: &TextStyle, w: f32, theme: &Theme) -> LineFit {
        let shown = self.field.input.shown();
        let (line_w, _) = TextMeasurer::measure_styled(&shown, style, None);
        let caret = self.field.is_focused().then(|| {
            (
                self.field.input.caret_x(style, None),
                theme.control.focus_ring_width,
            )
        });
        LineFit::new(line_w, w, caret)
    }

    /// A press inside focuses it and puts the caret on the glyph pressed;
    /// a press outside lets the focus go.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        look: &LineLook,
        theme: &Theme,
    ) -> EventResult {
        if self.field.disabled {
            return EventResult::IGNORED;
        }
        match *event {
            WidgetEvent::MouseDown { x, y } if bounds.contains(x, y) => {
                let style = Self::style(look, theme);
                let fit = self.fit(&style, bounds.w, theme);
                self.field.focus();
                self.field
                    .input
                    .handle_click_in(x - bounds.x + fit.scroll, &style, fit.width);
                EventResult::clicked()
            }
            WidgetEvent::MouseDown { .. } if self.field.is_focused() => {
                self.field.unfocus();
                EventResult::changed()
            }
            _ => EventResult::IGNORED,
        }
    }

    /// Where an input method puts its candidate window: the caret.
    pub fn ime_area(&self, bounds: Rect, look: &LineLook, theme: &Theme) -> Rect {
        let style = Self::style(look, theme);
        let fit = self.fit(&style, bounds.w, theme);
        Rect::new(
            bounds.x - fit.scroll + self.field.input.caret_x(&style, fit.width),
            bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
            theme.control.focus_ring_width,
            style.line_height,
        )
    }

    pub fn render(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        look: &LineLook,
        theme: &Theme,
    ) {
        if let Some(id) = self.field.sem_id() {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let marks = theme.form.recipe(Look::Bare);
        let fade = |color: Color| with_alpha(color, color.0[3] * look.alpha);
        let style = Self::style(look, theme);
        let text_y = bounds.y + TextMeasurer::vertical_center(&style, bounds.h);
        let fit = self.fit(&style, bounds.w, theme);
        let focused = self.field.is_focused();
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: bounds.x,
                y: bounds.y,
                w: bounds.w,
                h: bounds.h,
            },
        );
        let origin = bounds.x - fit.scroll;
        if focused {
            for (x, w) in self.field.input.selection_boxes(&style, fit.width) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: origin + x,
                        y: text_y,
                        w,
                        h: style.line_height,
                        color: marks.selection.0,
                    },
                );
            }
        }
        let shown = self.field.input.shown();
        let (drawn, color, width) = if shown.is_empty() {
            let hint = &self.field.input.placeholder;
            (
                TextMeasurer::truncate_to_width(hint, &style, bounds.w),
                fade(look.placeholder),
                Some(bounds.w),
            )
        } else if fit.overflows && !focused {
            (
                TextMeasurer::truncate_to_width(&shown, &style, bounds.w),
                fade(look.text.color),
                Some(bounds.w),
            )
        } else {
            (shown, fade(look.text.color), fit.width)
        };
        if !drawn.is_empty() {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(&drawn, &style, width),
                    x: origin,
                    y: text_y,
                    color,
                },
            );
        }
        let underline = theme.control.edge_width;
        for (x, w) in self.field.input.preedit_boxes(&style, fit.width) {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: origin + x,
                    y: text_y + style.line_height - underline,
                    w,
                    h: underline,
                    color: marks.composing.0,
                },
            );
        }
        if focused && self.field.input.cursor_visible() {
            let caret_x = self.field.input.caret_x(&style, fit.width);
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: origin + caret_x,
                    y: text_y,
                    w: theme.control.focus_ring_width,
                    h: style.line_height,
                    color: marks.caret.0,
                },
            );
        }
        c.push_to_layer(layer, SceneNode::PopClip);
    }

    /// Declares the line as `id`, as the text field does, named `label`.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        self.field.project(b, id, label, parent)
    }
}
