//! HOFF chip: a small status/tag pill — caption-sm label in a radius-full
//! glass capsule, tinted by [`Intent`]. Filled (`selected`) reads as the
//! active filter/tag; the outline variant is the quiet default. Static by
//! default; `interactive(true)` opts into the Button press contract
//! (click fires on release inside).

use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme, TypographyScale};

use super::{
    EventResult, Rect, WidgetEvent, intent_fill, rounded_rect, rounded_rect_stroke, with_alpha,
};

/// Chip height (24px: caption-sm + 2×5px vertical pad).
pub const CHIP_H: f32 = 24.0;
const PAD_X: f32 = 12.0;

/// Status/tag pill. See the module docs for the interactivity contract.
#[derive(Clone, Debug)]
pub struct Chip {
    pub label: String,
    pub intent: Intent,
    /// Filled (active) vs outline (quiet) styling.
    pub selected: bool,
    /// Opt into pointer interaction (hover/press/click).
    pub interactive: bool,
    hovered: bool,
    pressed: bool,
}

impl Chip {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            intent: Intent::Neutral,
            selected: false,
            interactive: false,
            hovered: false,
            pressed: false,
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Label style: HOFF caption-sm (12/600), one style for measure+draw.
    fn style() -> TextStyle {
        TypographyScale::hoff().caption_sm()
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self) -> (f32, f32) {
        let (tw, _) = TextMeasurer::measure_styled(&self.label, &Self::style(), None);
        ((tw + PAD_X * 2.0).ceil(), CHIP_H)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if !self.interactive {
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

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let style = Self::style();
        let accent = intent_fill(theme, self.intent);
        let text = theme.colors.text;

        // Filled (selected): intent wash + intent label. Outline: glass
        // edge stroke + dim label. Hover brightens both (interactive only).
        let (bg, edge, fg) = if self.selected {
            let hot = self.hovered && self.interactive;
            (
                with_alpha(theme.colors.accent, if hot { 0.22 } else { 0.14 }),
                [0.0; 4],
                if self.intent == Intent::Neutral {
                    text.0
                } else {
                    accent
                },
            )
        } else {
            let bg = if self.hovered && self.interactive {
                theme.glass.surface_hover.0
            } else {
                [0.0; 4]
            };
            let fg = if self.intent == Intent::Neutral {
                theme.colors.text_mid.0
            } else {
                accent
            };
            (bg, theme.glass.edge_soft.0, fg)
        };

        if bg[3] > 0.001 {
            compositor.push(rounded_rect(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                bounds.h / 2.0,
                bg,
            ));
        }
        if edge[3] > 0.001 {
            compositor.push(rounded_rect_stroke(
                bounds.x,
                bounds.y,
                bounds.w,
                bounds.h,
                bounds.h / 2.0,
                edge,
                1.0,
            ));
        }
        compositor.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.label, &style, None),
            x: bounds.x + PAD_X,
            y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
            color: fg,
        });
    }
}
