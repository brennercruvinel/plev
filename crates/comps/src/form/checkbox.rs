//! HOFF checkbox: 18px radius-6 box, white fill with a dark check when
//! on, `rgba($n2,.05)` glass with a `.25` border when off, plus an
//! optional base-2r trailing label. The whole bounds are the hit area;
//! keyboard focus ([`Checkbox::set_focused`]) rings the box itself.

use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::text::TextMeasurer;
use crate::theme::{Theme, TypographyScale};
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, contrast_text, focus_ring, with_alpha};

/// Box edge length.
const BOX: f32 = 18.0;
/// Micro-action radius (HOFF radius 6 on small controls).
const BOX_RADIUS: f32 = 6.0;

/// Retained checkbox with optional trailing label. The whole bounds act
/// as the hit area (box + label), like every native toolkit.
#[derive(Clone, Debug)]
pub struct Checkbox {
    pub checked: bool,
    pub label: Option<String>,
    pub disabled: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl Checkbox {
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            disabled: false,
            hovered: false,
            pressed: false,
            focused: false,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled checkboxes refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if self.disabled {
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
                    self.checked = !self.checked;
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            WidgetEvent::Scroll { .. } => EventResult::IGNORED,
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let bx = bounds.x;
        let by = bounds.y + (bounds.h - BOX) / 2.0;
        let glass = &theme.glass;

        if self.focused {
            // Ring the box, not the label: that is where the action is.
            compositor.push(focus_ring(Rect::new(bx, by, BOX, BOX), BOX_RADIUS, theme));
        }

        if self.checked {
            // Filled white ($text-primary) with a dark check on top (push
            // order is preserved across primitive types).
            let fill = theme.colors.text;
            compositor.push(super::rounded_rect(
                bx,
                by,
                BOX,
                BOX,
                BOX_RADIUS,
                with_alpha(fill, fill.0[3] * alpha),
            ));
            let mark = contrast_text(fill.0);
            let mark = [mark[0], mark[1], mark[2], alpha];
            if let Some(node) = icons::icon_at("check", 12.0, mark, bx + 3.0, by + 3.0) {
                compositor.push(node);
            }
        } else {
            // Unchecked: rgba($n2,.05) glass with a .25 border (hover .1 bg).
            let bg = if self.hovered {
                glass.surface_active
            } else {
                glass.field
            };
            let border = glass.field_focus_border;
            compositor.push(SceneNode::RoundedRect {
                x: bx,
                y: by,
                w: BOX,
                h: BOX,
                color: with_alpha(bg, bg.0[3] * alpha),
                corner_radius: BOX_RADIUS,
                border_width: 1.0,
                border_color: with_alpha(border, border.0[3] * alpha),
            });
        }

        if let Some(label) = &self.label {
            // Label: base-2r.
            let style = TypographyScale::hoff().base_2r();
            let text = theme.colors.text_mid;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: bx + BOX + 10.0,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: with_alpha(text, text.0[3] * alpha),
            });
        }
    }
}
