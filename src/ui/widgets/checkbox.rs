use crate::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::theme::Theme;
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, contrast_text, with_alpha};

/// Box edge length.
const BOX: f32 = 16.0;

/// Retained checkbox with optional trailing label. The whole bounds act
/// as the hit area (box + label), like every native toolkit.
#[derive(Clone, Debug)]
pub struct Checkbox {
    pub checked: bool,
    pub label: Option<String>,
    pub disabled: bool,
    hovered: bool,
    pressed: bool,
}

impl Checkbox {
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            disabled: false,
            hovered: false,
            pressed: false,
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

        let accent = theme.colors.accent.0;
        let (bg, border_color, border_width) = if self.checked {
            ([accent[0], accent[1], accent[2], alpha], [0.0; 4], 0.0)
        } else {
            let border = if self.hovered {
                theme.colors.border_active
            } else {
                theme.colors.divider
            };
            let bg = if self.hovered {
                with_alpha(theme.colors.bg_hover, alpha)
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            (bg, with_alpha(border, alpha), 1.0)
        };

        compositor.push(SceneNode::RoundedRect {
            x: bx,
            y: by,
            w: BOX,
            h: BOX,
            color: bg,
            corner_radius: theme.radius.md,
            border_width,
            border_color,
        });

        if self.checked {
            let mark = contrast_text(accent);
            let mark = [mark[0], mark[1], mark[2], alpha];
            if let Some(node) = icons::icon_at("check", 12.0, mark, bx + 2.0, by + 2.0) {
                compositor.push(node);
            }
        }

        if let Some(label) = &self.label {
            let font = 13.0;
            let line_height = font * 1.3;
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(label, font, line_height, None),
                x: bx + BOX + 8.0,
                y: bounds.y + (bounds.h - line_height) / 2.0,
                color: with_alpha(theme.colors.text, alpha),
            });
        }
    }
}
