//! HOFF icon button: a square glass control carrying only an icon glyph
//! (from [`crate::ui::icons`]), same variants/intents as [`Button`] and
//! the same click contract (fires on release inside). The label-less form
//! is Tooltip-friendly by design — pair it with one from the owning view;
//! this widget deliberately does not own a tooltip.

use crate::compositor::Compositor;
use crate::theme::{Intent, Theme};
use crate::ui::icons;

use super::{
    ButtonSize, ButtonVariant, EventResult, Rect, WidgetEvent, focus_ring, glass_pill, intent_fill,
    with_alpha,
};

/// Square icon-only button. See the module docs for the tooltip note.
#[derive(Clone, Debug)]
pub struct IconButton {
    pub icon: &'static str,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub intent: Intent,
    pub disabled: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
}

impl IconButton {
    pub fn new(icon: &'static str) -> Self {
        Self {
            icon,
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            intent: Intent::Neutral,
            disabled: false,
            hovered: false,
            pressed: false,
            focused: false,
        }
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled buttons refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Square: side = the control height of the size variant.
    pub fn preferred_size(&self) -> (f32, f32) {
        let h = self.size.height();
        (h, h)
    }

    fn icon_size(&self) -> f32 {
        self.size.font_size() + 4.0
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
        let glass = &theme.glass;
        let intent = if self.variant == ButtonVariant::Danger {
            Intent::Destructive
        } else {
            self.intent
        };
        let fg_base = match intent {
            Intent::Neutral => with_alpha(theme.colors.text, theme.colors.text.0[3] * 0.76),
            other => intent_fill(theme, other),
        };
        let (bg, edge) = match self.variant {
            ButtonVariant::Solid | ButtonVariant::Danger => (
                if self.hovered {
                    glass.button_hover.0
                } else {
                    glass.button.0
                },
                glass.edge.0,
            ),
            ButtonVariant::Outline => (
                if self.hovered {
                    glass.surface_hover.0
                } else {
                    [0.0; 4]
                },
                if self.hovered {
                    glass.field_focus_border.0
                } else {
                    glass.edge.0
                },
            ),
            ButtonVariant::Ghost => (
                if self.hovered {
                    glass.surface_hover.0
                } else {
                    [0.0; 4]
                },
                [0.0; 4],
            ),
        };
        let fg = [fg_base[0], fg_base[1], fg_base[2], fg_base[3] * alpha];

        // Squircle, not pill: icon buttons read as rounded squares.
        let radius = theme.radius.md.min(bounds.h / 2.0);
        if self.focused {
            compositor.push(focus_ring(bounds, radius, theme));
        }
        if bg[3] > 0.001 || edge[3] > 0.001 {
            for node in glass_pill(
                bounds,
                radius,
                edge,
                1.5,
                [bg[0], bg[1], bg[2], bg[3] * alpha],
            ) {
                compositor.push(node);
            }
        }

        let icon = self.icon_size();
        if let Some(node) = icons::icon_at(
            self.icon,
            icon,
            fg,
            bounds.x + (bounds.w - icon) / 2.0,
            bounds.y + (bounds.h - icon) / 2.0,
        ) {
            compositor.push(node);
        }
    }
}
