use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme};
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, contrast_text, intent_fill, shade, with_alpha};

/// Visual variant, shadcn/ui naming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Filled with the intent color (primary action).
    #[default]
    Solid,
    /// Transparent with a 1px border.
    Outline,
    /// Transparent until hovered.
    Ghost,
    /// Solid shorthand for destructive actions.
    Danger,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl ButtonSize {
    pub fn height(self) -> f32 {
        match self {
            ButtonSize::Sm => 24.0,
            ButtonSize::Md => 30.0,
            ButtonSize::Lg => 36.0,
        }
    }

    pub fn font_size(self) -> f32 {
        match self {
            ButtonSize::Sm => 12.0,
            ButtonSize::Md => 13.0,
            ButtonSize::Lg => 14.0,
        }
    }

    pub fn pad_x(self) -> f32 {
        match self {
            ButtonSize::Sm => 10.0,
            ButtonSize::Md => 14.0,
            ButtonSize::Lg => 18.0,
        }
    }
}

/// Retained push button.
///
/// A click is reported (via [`EventResult::clicked`]) on mouse-up inside
/// the bounds after a mouse-down inside them — the standard "press can be
/// cancelled by dragging away" behavior.
#[derive(Clone, Debug)]
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub intent: Intent,
    pub disabled: bool,
    /// Optional leading icon (a [`crate::ui::icons`] name).
    pub icon: Option<&'static str>,
    hovered: bool,
    pressed: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            intent: Intent::Neutral,
            disabled: false,
            icon: None,
            hovered: false,
            pressed: false,
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

    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    fn icon_size(&self) -> f32 {
        self.size.font_size() + 2.0
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self) -> (f32, f32) {
        let font = self.size.font_size();
        let style = TextStyle::new(font).with_weight(500);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let icon_w = if self.icon.is_some() {
            self.icon_size() + 6.0
        } else {
            0.0
        };
        (
            (text_w + icon_w + self.size.pad_x() * 2.0).ceil(),
            self.size.height(),
        )
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        if self.disabled {
            // A disabled button swallows nothing and never reacts.
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

    /// Background, border (color, width), and text colors for the current
    /// state — resolved from theme tokens.
    fn colors(&self, theme: &Theme) -> ([f32; 4], [f32; 4], f32, [f32; 4]) {
        let intent = if self.variant == ButtonVariant::Danger {
            Intent::Destructive
        } else {
            self.intent
        };
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let active = self.pressed && self.hovered;

        match self.variant {
            ButtonVariant::Solid | ButtonVariant::Danger => {
                let mut bg = intent_fill(theme, intent);
                if active {
                    bg = shade(bg, 0.18);
                } else if self.hovered {
                    bg = shade(bg, 0.10);
                }
                let fg = contrast_text(bg);
                bg[3] *= alpha;
                (bg, [0.0; 4], 0.0, [fg[0], fg[1], fg[2], alpha])
            }
            ButtonVariant::Outline => {
                let bg = if active {
                    shade(theme.colors.bg_hover.0, 0.06)
                } else if self.hovered {
                    theme.colors.bg_hover.0
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                };
                let border = if self.hovered && !self.disabled {
                    theme.colors.border_active
                } else {
                    theme.colors.divider
                };
                (
                    [bg[0], bg[1], bg[2], bg[3] * alpha],
                    with_alpha(border, alpha),
                    1.0,
                    with_alpha(theme.colors.text, alpha),
                )
            }
            ButtonVariant::Ghost => {
                let bg = if active {
                    shade(theme.colors.bg_hover.0, 0.06)
                } else if self.hovered {
                    theme.colors.bg_hover.0
                } else {
                    [0.0, 0.0, 0.0, 0.0]
                };
                let fg = match intent {
                    Intent::Neutral => with_alpha(theme.colors.text, alpha),
                    other => {
                        let c = intent_fill(theme, other);
                        [c[0], c[1], c[2], alpha]
                    }
                };
                ([bg[0], bg[1], bg[2], bg[3] * alpha], [0.0; 4], 0.0, fg)
            }
        }
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(compositor, LayerId::DEFAULT, bounds, theme);
    }

    /// Render onto a specific layer (modals/overlays embed buttons).
    pub fn render_to_layer(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
    ) {
        let (bg, border_color, border_width, fg) = self.colors(theme);
        let font = self.size.font_size();
        let line_height = font * 1.3;

        if bg[3] > 0.001 || border_width > 0.0 {
            compositor.push_to_layer(
                layer,
                SceneNode::RoundedRect {
                    x: bounds.x,
                    y: bounds.y,
                    w: bounds.w,
                    h: bounds.h,
                    color: bg,
                    corner_radius: theme.radius.md + 2.0,
                    border_width,
                    border_color,
                },
            );
        }

        let style = TextStyle::new(font).with_weight(500);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let icon_size = self.icon_size();
        let icon_w = if self.icon.is_some() {
            icon_size + 6.0
        } else {
            0.0
        };
        let content_w = text_w + icon_w;
        let mut cx = bounds.x + (bounds.w - content_w) / 2.0;

        if let Some(name) = self.icon
            && let Some(node) = icons::icon_at(
                name,
                icon_size,
                fg,
                cx,
                bounds.y + (bounds.h - icon_size) / 2.0,
            )
        {
            compositor.push_to_layer(layer, node);
            cx += icon_w;
        }

        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::new(&self.label, font, line_height, None).with_weight(500),
                x: cx,
                y: bounds.y + (bounds.h - line_height) / 2.0,
                color: fg,
            },
        );
    }
}
