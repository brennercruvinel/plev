//! The text field's draw: the box (its surface for the state, the shadow
//! list, the border), then the selection wash, the text or placeholder
//! under the line fit's clip, the composition's underline and the caret.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextMeasurer;
use crate::styling::color::Color;
use crate::styling::theme::{ShadowSpec, Theme};
use crate::widgets::form::tokens::Slot;
use crate::widgets::recipe::{
    focus_ring, rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets,
};
use crate::widgets::widget::{Rect, with_alpha};

use super::TextField;
use crate::widgets::form::tokens::{InputRecipe, InputSurface};

/// An input box of `recipe` in `surface`'s state: its drop shadows, its
/// fill, its border inside the box and its inset shadows, at `alpha`
/// (a disabled box's fade). Its radius is the recipe's, never more than
/// half its height. The text field, a code's cells and a text area draw
/// their boxes with it.
pub(crate) fn draw_box(
    c: &mut Compositor,
    layer: LayerId,
    bounds: Rect,
    recipe: &InputRecipe,
    surface: &InputSurface,
    alpha: f32,
) {
    let fade = |color: Color| with_alpha(color, color.0[3] * alpha);
    let radius = recipe.radius.min(bounds.h / 2.0);
    let shadows: Vec<_> = surface
        .shadow
        .iter()
        .map(|s| ShadowSpec {
            color: Color(fade(s.color)),
            ..*s
        })
        .collect();
    shadow_drops(c, layer, bounds, radius, &shadows);
    let Rect { x, y, w, h } = bounds;
    c.push_to_layer(layer, rounded_rect(x, y, w, h, radius, fade(surface.fill)));
    c.push_to_layer(
        layer,
        rounded_rect_stroke(
            x,
            y,
            w,
            h,
            radius,
            fade(surface.border),
            recipe.border_width,
        ),
    );
    shadow_insets(c, layer, bounds, radius, &shadows);
}

impl TextField {
    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        self.render_in(c, layer, bounds, theme, self.slots);
    }

    /// [`render_to_layer`](Self::render_to_layer) with the ends taken by
    /// `slots`.
    pub fn render_in(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
        slots: [Slot; 2],
    ) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let recipe = self.recipe(theme);
        let alpha = if self.disabled {
            recipe.disabled_alpha
        } else {
            1.0
        };
        let fade = |color: Color| with_alpha(color, color.0[3] * alpha);
        let focused = self.input.focused;
        let style = self.style(theme);
        let radius = recipe.radius.min(bounds.h / 2.0);

        if focused {
            c.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }

        let surface = if focused {
            &recipe.focus
        } else if self.hovered {
            &recipe.hover
        } else {
            &recipe.rest
        };
        draw_box(c, layer, bounds, recipe, surface, alpha);

        let (text_x, text_y, text_w) = self.text_box(bounds, &style, theme, slots);
        let fit = self.fit(&style, text_w, theme);
        // A line wider than the box draws under a clip of the text's box,
        // slid left by the fit's scroll.
        if fit.overflows {
            c.push_to_layer(
                layer,
                SceneNode::PushClip {
                    x: text_x,
                    y: bounds.y,
                    w: text_w,
                    h: bounds.h,
                },
            );
        }
        let origin = text_x - fit.scroll;

        // Selection wash under the text, one box per stretch the logical
        // range covers on screen.
        if focused {
            for (sel_x, sel_w) in self.input.selection_boxes(&style, fit.width) {
                c.push_to_layer(
                    layer,
                    SceneNode::Rect {
                        x: origin + sel_x,
                        y: text_y,
                        w: sel_w,
                        h: style.line_height,
                        color: recipe.selection.0,
                    },
                );
            }
        }

        // The text (the composition in it while composing, bullets when
        // masked), or the placeholder while empty; cut with an ellipsis
        // where it does not fit and is not being edited.
        let shown = self.input.shown();
        let (drawn, color, width) = if shown.is_empty() {
            (
                TextMeasurer::truncate_to_width(&self.input.placeholder, &style, text_w),
                fade(recipe.placeholder),
                Some(text_w),
            )
        } else if fit.overflows && !focused {
            (
                TextMeasurer::truncate_to_width(&shown, &style, text_w),
                fade(recipe.text_color),
                Some(text_w),
            )
        } else {
            (shown, fade(recipe.text_color), fit.width)
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

        // The preedit's underline: one edge stroke at the foot of the line
        // box, under each stretch the preedit covers on screen.
        let underline = theme.control.edge_width;
        for (x, w) in self.input.preedit_boxes(&style, fit.width) {
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: origin + x,
                    y: text_y + style.line_height - underline,
                    w,
                    h: underline,
                    color: recipe.composing.0,
                },
            );
        }

        // Caret: one focus-ring stroke wide, the line box tall.
        if self.input.cursor_visible() {
            let caret_x = self.input.caret_x(&style, fit.width);
            c.push_to_layer(
                layer,
                SceneNode::Rect {
                    x: origin + caret_x,
                    y: text_y,
                    w: theme.control.focus_ring_width,
                    h: style.line_height,
                    color: recipe.caret.0,
                },
            );
        }
        if fit.overflows {
            c.push_to_layer(layer, SceneNode::PopClip);
        }
    }
}
