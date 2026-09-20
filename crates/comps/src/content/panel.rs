//! Panel: the raised glass surface a screen groups content on. The same
//! shell the deck cards use (surface wash, soft top-lit rim, inset
//! key-light) at the card radius, with an optional group label in the
//! caption ramp and the panel padding as the content inset. Static: a
//! panel frames, the widgets inside it react.

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::TextStyle;
use engine::theme::{Elevation, Theme};

use crate::core::Rect;
use crate::recipe::{glass_surface, shadow_stack};

#[derive(Clone, Debug, Default)]
pub struct Panel {
    /// Optional caption-sm label drawn inside the top padding.
    pub label: Option<String>,
    /// Raised panels (dialog sheets, detail drawers) cast the overlay
    /// stack; flat ones sit on the page like a card.
    pub elevation: Elevation,
}

impl Panel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn elevation(mut self, elevation: Elevation) -> Self {
        self.elevation = elevation;
        self
    }

    /// Padding between the panel edge and its content: the `lg` step.
    pub fn pad(theme: &Theme) -> f32 {
        theme.spacing.lg
    }

    /// Group label style: caption-sm, uppercase by convention.
    pub fn label_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_sm()
    }

    /// The rect content lays out in: the bounds inset by the padding,
    /// minus the label band when there is one.
    pub fn content_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let pad = Self::pad(theme);
        let mut r = bounds.inset(pad);
        if self.label.is_some() {
            let band = Self::label_style(theme).line_height + theme.spacing.sm;
            r.y += band;
            r.h = (r.h - band).max(0.0);
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let radius = theme.shape.card;
        shadow_stack(c, layer, bounds, radius, self.elevation, theme);
        glass_surface(c, layer, bounds, radius, theme.glass.surface.0, theme);
        if let Some(label) = &self.label {
            let style = Self::label_style(theme);
            let pad = Self::pad(theme);
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(label, &style, Some(bounds.w - pad * 2.0)),
                    x: bounds.x + pad,
                    y: bounds.y + pad,
                    color: theme.glass.text_faint.0,
                },
            );
        }
    }
}
