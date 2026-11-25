//! Panel: the raised surface a screen groups content on. The same shell
//! the deck cards use (`glass_surface`: the kit's plain bordered surface
//! in the core theme, the glass wash, top-lit rim and key-light
//! elsewhere), with an optional group label and the panel padding as the
//! content inset; radius, padding and label are the theme's
//! (`theme.disclosure.panel`). Static: a panel frames, the widgets inside
//! it react.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::TextStyle;
use crate::styling::theme::{Elevation, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::recipe::{glass_surface, shadow_stack};
use crate::widgets::widget::Rect;

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

    /// Padding between the panel edge and its content
    /// (`theme.disclosure.panel`).
    pub fn pad(theme: &Theme) -> f32 {
        theme.disclosure.panel.pad
    }

    /// Group label style (`theme.disclosure.panel`).
    pub fn label_style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.panel;
        theme.typography.run(r.label_type)
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

    /// Declares this panel as `id` in a projection: a `Role::Group` named
    /// by its label, or by `name` when it has none. A panel frames and
    /// takes no action; what it holds the app adds under the returned
    /// node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        name: &str,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let label = self.label.as_deref().unwrap_or(name);
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let radius = theme.disclosure.panel.radius;
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
                    color: theme.disclosure.panel.label.0,
                },
            );
        }
    }
}
