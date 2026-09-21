//! What every button shares: the press model (hover, press, release
//! inside = click, drag away cancels) and the surface painter that turns
//! a [`ButtonRecipe`] state into scene nodes. The three button widgets
//! are thin over this.

use engine::compositor::{Compositor, LayerId};
use engine::theme::{ButtonRecipe, ButtonSurface, Theme};

use crate::core::{EventResult, Rect, WidgetEvent};
use crate::recipe::{focus_ring, gradient_fill, rounded_rect, shadow_drops, shadow_insets};

/// Pointer state of a pressable control.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Press {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

impl Press {
    /// Route a pointer event against `bounds`. A disabled control drops
    /// its transient state and reacts to nothing.
    pub fn handle(&mut self, event: &WidgetEvent, bounds: Rect, disabled: bool) -> EventResult {
        if disabled {
            if self.hovered || self.pressed {
                self.hovered = false;
                self.pressed = false;
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
                    self.pressed = true;
                    EventResult::changed()
                } else {
                    EventResult::IGNORED
                }
            }
            WidgetEvent::MouseUp { x, y } => {
                if !self.pressed {
                    return EventResult::IGNORED;
                }
                self.pressed = false;
                if bounds.contains(x, y) {
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    /// The recipe state for this pointer state.
    pub fn surface<'a>(&self, recipe: &'a ButtonRecipe) -> &'a ButtonSurface {
        if self.pressed {
            &recipe.active
        } else if self.hovered {
            &recipe.hover
        } else {
            &recipe.rest
        }
    }
}

fn faded(c: [f32; 4], alpha: f32) -> [f32; 4] {
    [c[0], c[1], c[2], c[3] * alpha]
}

/// Paint one surface state at `bounds`: drop shadows, gradient fill,
/// overlay wash, inset layers, focus ring. Returns the label color to
/// draw the content with, faded like the rest when disabled.
pub fn paint_surface(
    c: &mut Compositor,
    layer: LayerId,
    bounds: Rect,
    radius: f32,
    surface: &ButtonSurface,
    alpha: f32,
    focused: bool,
    theme: &Theme,
) -> [f32; 4] {
    let shadows: Vec<_> = surface
        .shadow
        .iter()
        .map(|s| {
            let mut s = *s;
            s.color = engine::color::Color(faded(s.color.0, alpha));
            s
        })
        .collect();
    shadow_drops(c, layer, bounds, radius, &shadows);
    let [top, bottom] = surface.fill;
    if top.0[3] > 0.0 || bottom.0[3] > 0.0 {
        c.push_to_layer(
            layer,
            gradient_fill(bounds, radius, faded(top.0, alpha), faded(bottom.0, alpha)),
        );
    }
    if surface.overlay.0[3] > 0.0 {
        c.push_to_layer(
            layer,
            rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                radius,
                faded(surface.overlay.0, alpha),
            ),
        );
    }
    shadow_insets(c, layer, bounds, radius, &shadows);
    if focused {
        c.push_to_layer(layer, focus_ring(bounds, radius, theme));
    }
    faded(surface.text.0, alpha)
}
