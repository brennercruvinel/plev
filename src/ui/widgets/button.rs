use crate::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::text::{TextMeasurer, TextStyle};
use crate::theme::{Intent, Theme};
use crate::ui::icons;

use super::{EventResult, Rect, WidgetEvent, glass_pill, intent_fill, with_alpha};

/// Visual variant, HOFF naming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Glass pill: graphite fill + top edge-light (the HOFF default
    /// button — `rgba(40,40,40,.70)`, hover `rgba(248,248,248,.10)`).
    #[default]
    Solid,
    /// Transparent with the edge border always visible.
    Outline,
    /// Transparent until hovered (chip-social hover recipe).
    Ghost,
    /// Glass pill with the label/icon in `#BD3027` (unfollow-style).
    Danger,
}

/// HOFF control heights: chip-social 40 · button 44 · button-medium 52.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Social chip: 40px, pad 12, caption-sm (12/600).
    Sm,
    /// Standard pill: 44px, pad 24, base-2sm (14/600).
    #[default]
    Md,
    /// Medium pill: 52px, pad 32, base-2sm (14/600).
    Lg,
}

impl ButtonSize {
    pub fn height(self) -> f32 {
        match self {
            ButtonSize::Sm => 40.0,
            ButtonSize::Md => 44.0,
            ButtonSize::Lg => 52.0,
        }
    }

    pub fn font_size(self) -> f32 {
        match self {
            ButtonSize::Sm => 12.0,
            ButtonSize::Md | ButtonSize::Lg => 14.0,
        }
    }

    pub fn pad_x(self) -> f32 {
        match self {
            ButtonSize::Sm => 12.0,
            ButtonSize::Md => 24.0,
            ButtonSize::Lg => 32.0,
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
        self.size.font_size() + 4.0
    }

    fn font_weight(&self) -> u16 {
        600
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self) -> (f32, f32) {
        let font = self.size.font_size();
        let style = TextStyle::new(font).with_weight(self.font_weight());
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let icon_w = if self.icon.is_some() {
            self.icon_size() + 8.0
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

    /// Background, edge-light color (alpha 0 = none), and label color for
    /// the current state — resolved from glass tokens.
    fn colors(&self, theme: &Theme) -> ([f32; 4], [f32; 4], [f32; 4]) {
        let intent = if self.variant == ButtonVariant::Danger {
            Intent::Destructive
        } else {
            self.intent
        };
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let glass = &theme.glass;
        let text = theme.colors.text;

        // Label: $text-secondary (.70 of text) at rest, .76 on hover —
        // intents recolor the label, never the glass fill.
        let label_rest = match intent {
            Intent::Neutral => with_alpha(text, text.0[3] * 0.737),
            other => intent_fill(theme, other),
        };
        let label_hot = match intent {
            Intent::Neutral => with_alpha(text, text.0[3] * 0.8),
            other => intent_fill(theme, other),
        };

        let (bg, edge, fg) = match self.variant {
            ButtonVariant::Solid | ButtonVariant::Danger => {
                let bg = if self.hovered {
                    glass.button_hover.0
                } else {
                    glass.button.0
                };
                let fg = if self.hovered { label_hot } else { label_rest };
                (bg, glass.edge.0, fg)
            }
            ButtonVariant::Outline => {
                let bg = if self.hovered {
                    glass.surface_hover.0
                } else {
                    [0.0; 4]
                };
                let edge = if self.hovered {
                    glass.field_focus_border.0
                } else {
                    glass.edge.0
                };
                let fg = if self.hovered { label_hot } else { label_rest };
                (bg, edge, fg)
            }
            ButtonVariant::Ghost => {
                let bg = if self.hovered {
                    glass.surface_hover.0
                } else {
                    [0.0; 4]
                };
                let fg = if self.hovered { label_hot } else { label_rest };
                (bg, [0.0; 4], fg)
            }
        };
        (
            [bg[0], bg[1], bg[2], bg[3] * alpha],
            [edge[0], edge[1], edge[2], edge[3] * alpha],
            [fg[0], fg[1], fg[2], fg[3] * alpha],
        )
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
        let (bg, edge, fg) = self.colors(theme);
        let font = self.size.font_size();
        let line_height = font * 1.4;
        // Pill: HOFF radius 32 clamps to half the 44px height.
        let radius = theme.radius.xl.min(bounds.h / 2.0);

        if self.icon.is_some() {
            // Icons are quad-pass paths; an SDF background would paint over
            // them, so the background becomes path geometry too (flat fill,
            // uniform border instead of the gradient edge-light).
            if bg[3] > 0.001 {
                compositor.push_to_layer(
                    layer,
                    super::path_rounded_rect(bounds.x, bounds.y, bounds.w, bounds.h, radius, bg),
                );
            }
            if edge[3] > 0.001 {
                compositor.push_to_layer(
                    layer,
                    super::path_rounded_rect_stroke(
                        bounds.x, bounds.y, bounds.w, bounds.h, radius, edge, 1.5,
                    ),
                );
            }
        } else if bg[3] > 0.001 || edge[3] > 0.001 {
            // Edge-light underlay + glass fill (HOFF :before border).
            for node in glass_pill(bounds, radius, edge, 1.5, bg) {
                compositor.push_to_layer(layer, node);
            }
        }

        let style = TextStyle::new(font).with_weight(self.font_weight());
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let icon_size = self.icon_size();
        let icon_w = if self.icon.is_some() {
            icon_size + 8.0
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
                key: TextNodeKey::new(&self.label, font, line_height, None)
                    .with_weight(self.font_weight()),
                x: cx,
                y: bounds.y + (bounds.h - line_height) / 2.0,
                color: fg,
            },
        );
    }
}
