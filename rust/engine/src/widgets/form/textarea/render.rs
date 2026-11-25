//! The text area's draw: the label, the Field's box for the state, then
//! under a clip of the text box the selection, the text (or the
//! placeholder) wrapped to its width and scrolled to keep the caret in,
//! the composition's underline and the caret; the message line under the
//! box.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::form::field::Message;
use crate::widgets::form::text_field::render::draw_box;
use crate::widgets::recipe::focus_ring;
use crate::widgets::widget::{Rect, with_alpha};

use super::TextArea;
use super::lines::Lines;

impl TextArea {
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let l = self.layout(bounds, theme);
        let form = &theme.form;
        let r = &form.input;
        let alpha = if self.input.disabled {
            r.disabled_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * alpha);
        if let (Some(label), Some(at)) = (&self.label, l.label) {
            label.draw(c, layer, at.x, at.y, Some(at.w), theme);
        }
        if let Some(id) = &self.sem {
            let b = l.input;
            c.bind_to_layer(layer, id, [b.x, b.y, b.w, b.h]);
        }
        let focused = self.input.is_focused();
        if focused {
            let radius = r.radius.min(l.input.h / 2.0);
            c.push_to_layer(layer, focus_ring(l.input, radius, theme));
        }
        let surface = if focused {
            &r.focus
        } else if self.hovered {
            &r.hover
        } else {
            &r.rest
        };
        draw_box(c, layer, l.input, r, surface, alpha);

        self.remember(l.text, theme);
        let style = self.style(theme);
        let shown = self.shown();
        let lines = Lines::new(&shown, &style, l.text.w);
        let (caret_x, caret_line) = lines.caret(self.caret_pos());
        let scroll = self.follow(caret_line, lines.count(), l.text.h, theme);
        let t = l.text;
        c.push_to_layer(
            layer,
            SceneNode::PushClip {
                x: l.input.x,
                y: t.y,
                w: l.input.w,
                h: t.h,
            },
        );
        let top = t.y - scroll;
        let lh = style.line_height;
        if focused && self.input.input.composed().is_none() {
            if let Some((a, b)) = self.input.input.buffer.selection() {
                for (x, w, i) in lines.boxes(a.min(b), a.max(b)) {
                    c.push_to_layer(
                        layer,
                        SceneNode::Rect {
                            x: t.x + x,
                            y: top + i as f32 * lh,
                            w,
                            h: lh,
                            color: r.selection.0,
                        },
                    );
                }
            }
        }
        let (text, color) = if shown.is_empty() {
            (self.input.input.placeholder.as_str(), r.placeholder)
        } else {
            (shown.as_str(), r.text_color)
        };
        if !text.is_empty() {
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(text, &style, Some(t.w)),
                    x: t.x,
                    y: top,
                    color: fade(color),
                },
            );
        }
        if let Some((_, span)) = self.input.input.composed() {
            let under = theme.control.edge_width;
            for (x, w, i) in lines.boxes(span.start, span.end) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: t.x + x,
                        y: top + (i + 1) as f32 * lh - under,
                        w,
                        h: under,
                        color: r.composing.0,
                    },
                );
            }
        }
        if self.input.input.cursor_visible() {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: t.x + caret_x,
                    y: top + caret_line as f32 * lh,
                    w: theme.control.focus_ring_width,
                    h: lh,
                    color: r.caret.0,
                },
            );
        }
        c.push_to_layer(layer, SceneNode::PopClip);

        if let (Some(m), Some(at)) = (&self.message, l.message) {
            let color = match m {
                Message::Error(_) => form.field.error_color,
                Message::Helper(_) => form.field.helper_color,
            };
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(
                        m.text(),
                        &theme.typography.run(form.field.message),
                        Some(at.w),
                    ),
                    x: at.x,
                    y: at.y,
                    color: fade(color),
                },
            );
        }
    }

    /// Where the input method's window goes: the caret, laid out in
    /// `bounds`, one caret wide and one line tall.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        let style = self.style(theme);
        let shown = self.shown();
        let lines = Lines::new(&shown, &style, l.text.w);
        let (x, i) = lines.caret(self.caret_pos());
        let lh = style.line_height;
        Rect::new(
            l.text.x + x,
            l.text.y - self.scroll.get() + i as f32 * lh,
            theme.control.focus_ring_width,
            lh,
        )
    }
}
