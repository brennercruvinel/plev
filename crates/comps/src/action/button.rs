//! HOFF push button: glass pill in four variants (solid, outline, ghost,
//! danger) and three control heights, label measured and drawn with one
//! `TextStyle`. Click fires on release inside the bounds; keyboard focus
//! (via [`Button::set_focused`]) draws the shared accent focus ring.
//!
//! Every dimension comes from the theme: heights and paddings from
//! `theme.control`, the pill radius from `theme.shape`, the label ramp
//! from `theme.typography`, fills and rims from `theme.glass`.

use crate::icons;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{ControlSize, Intent, Theme};

use crate::core::{EventResult, Rect, WidgetEvent, intent_fill, with_alpha};
use crate::recipe::{focus_ring, glass_pill};

/// Visual variant, HOFF naming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Glass pill: graphite fill + top edge-light (the HOFF default
    /// button, `glass.button` at rest, `glass.button_hover` hovered).
    #[default]
    Solid,
    /// Transparent with the edge border always visible.
    Outline,
    /// Transparent until hovered (chip-social hover recipe).
    Ghost,
    /// Glass pill with the label/icon in the destructive color
    /// (unfollow-style).
    Danger,
}

/// The three button heights of the HOFF kit, mapped onto the control
/// ladder: chip-social (Sm), button (Md), button-medium (Lg).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Social chip: `ControlSize::Sm`, caption-sm label.
    Sm,
    /// Standard pill: `ControlSize::Md`, base-2sm label.
    #[default]
    Md,
    /// Medium pill: `ControlSize::Xl`, base-2sm label.
    Lg,
}

impl ButtonSize {
    /// The control ladder step this button size is.
    pub fn control(self) -> ControlSize {
        match self {
            ButtonSize::Sm => ControlSize::Sm,
            ButtonSize::Md => ControlSize::Md,
            ButtonSize::Lg => ControlSize::Xl,
        }
    }

    pub fn height(self, theme: &Theme) -> f32 {
        theme.control.height(self.control())
    }

    pub fn pad_x(self, theme: &Theme) -> f32 {
        theme.control.pad_x(self.control())
    }

    /// Icon edge that fits this control.
    pub fn icon_size(self, theme: &Theme) -> f32 {
        theme.control.icon_for(self.control())
    }

    /// Label style, HOFF exact: chips are caption-sm, pills are base-2sm.
    /// One source for measuring AND rendering.
    pub fn text_style(self, theme: &Theme) -> TextStyle {
        match self {
            ButtonSize::Sm => theme.typography.caption_sm(),
            ButtonSize::Md | ButtonSize::Lg => theme.typography.base_2sm(),
        }
    }
}

/// Retained push button.
///
/// A click is reported (via [`EventResult::clicked`]) on mouse-up inside
/// the bounds after a mouse-down inside them, the standard "press can be
/// cancelled by dragging away" behavior.
#[derive(Clone, Debug)]
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub intent: Intent,
    pub disabled: bool,
    /// Optional leading icon (a [`crate::icons`] name).
    pub icon: Option<&'static str>,
    hovered: bool,
    pressed: bool,
    focused: bool,
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

    /// Keyboard focus, driven by the owning view (plev has no global focus
    /// chain). Disabled buttons refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Width the leading icon takes, gap included; zero without an icon.
    fn icon_slot(&self, theme: &Theme) -> f32 {
        if self.icon.is_some() {
            self.size.icon_size(theme) + theme.control.inline_gap
        } else {
            0.0
        }
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let style = self.size.text_style(theme);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        (
            (text_w + self.icon_slot(theme) + self.size.pad_x(theme) * 2.0).ceil(),
            self.size.height(theme),
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
    /// the current state, resolved from glass tokens.
    fn colors(&self, theme: &Theme) -> ([f32; 4], [f32; 4], [f32; 4]) {
        let intent = if self.variant == ButtonVariant::Danger {
            Intent::Destructive
        } else {
            self.intent
        };
        let glass = &theme.glass;
        let alpha = if self.disabled {
            glass.disabled_alpha
        } else {
            1.0
        };

        // Label: text-secondary at rest, text-active on hover. Intents
        // recolor the label, never the glass fill.
        let label_rest = match intent {
            Intent::Neutral => theme.colors.text_mid.0,
            other => intent_fill(theme, other),
        };
        let label_hot = match intent {
            Intent::Neutral => glass.text_active.0,
            other => intent_fill(theme, other),
        };
        let fg = if self.hovered { label_hot } else { label_rest };

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
        (
            with_alpha(engine::color::Color(bg), bg[3] * alpha),
            with_alpha(engine::color::Color(edge), edge[3] * alpha),
            with_alpha(engine::color::Color(fg), fg[3] * alpha),
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
        let style = self.size.text_style(theme);
        // Pill: the shape radius clamps to half the control height.
        let radius = theme.shape.pill.min(bounds.h / 2.0);

        if self.focused {
            compositor.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }

        if bg[3] > 0.001 || edge[3] > 0.001 {
            // Edge-light underlay + glass fill (HOFF :before border). Icon
            // paths pushed later stack on top: the compositor preserves
            // push order across primitive types.
            for node in glass_pill(bounds, radius, edge, theme.control.edge_width_strong, bg) {
                compositor.push_to_layer(layer, node);
            }
        }

        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let icon_size = self.size.icon_size(theme);
        let icon_slot = self.icon_slot(theme);
        let content_w = text_w + icon_slot;
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
            cx += icon_slot;
        }

        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.label, &style, None),
                x: cx,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: fg,
            },
        );
    }
}
