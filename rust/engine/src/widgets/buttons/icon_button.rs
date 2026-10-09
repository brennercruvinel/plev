//! Square icon-only button of the hoff core kit: the bordered one
//! (`outline`), the plain one (`ghost`) and the toolbar item, which has
//! an `active` state for the selected tool. Three sizes (24 / 36 / 40).
//!
//! With a [`SemId`] ([`IconButton::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`IconButton::project`] declares the
//! button in a semantic projection. An icon names nothing, so the
//! projection takes its accessible label from the app.

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::{ButtonRecipe, IconButtonMetrics, Intent, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::press::{Press, paint_surface};
use crate::widgets::icons;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonVariant {
    /// Border and paper fill (the ghost icon button of the kit).
    Outline,
    /// Nothing at rest, a wash on hover.
    #[default]
    Ghost,
    /// Toolbar item; `active` selects it.
    Toolbar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconButtonSize {
    /// 24, radius 8.
    Sm,
    /// 36, radius 10.
    Md,
    /// 40, radius 12.
    #[default]
    Lg,
}

impl IconButtonSize {
    pub fn metrics(self, theme: &Theme) -> IconButtonMetrics {
        theme.button.icon_sizes[self as usize]
    }
}

#[derive(Clone, Debug)]
pub struct IconButton {
    pub icon: &'static str,
    pub variant: IconButtonVariant,
    pub size: IconButtonSize,
    pub disabled: bool,
    /// Toolbar: the tool is selected.
    pub active: bool,
    /// Icon edge override; the size's own by default.
    pub icon_size: Option<f32>,
    press: Press,
    sem: Option<SemId>,
}

impl IconButton {
    pub fn new(icon: &'static str) -> Self {
        Self {
            icon,
            variant: IconButtonVariant::default(),
            size: IconButtonSize::default(),
            disabled: false,
            active: false,
            icon_size: None,
            press: Press::default(),
            sem: None,
        }
    }

    /// The logical identity the render binds its bounds to
    /// (`Compositor::bind_to_layer`).
    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// Declares this button as `id` in a projection: a `Role::Button`
    /// named `label` (the icon names nothing, so the app gives the
    /// accessible label), enabled unless disabled, with an `activate`
    /// action. A toolbar item also carries its `active` state (the
    /// selected tool), which `activate` reads. What `activate` does is the
    /// app's; this only reads the widget. Pass the id given to
    /// [`IconButton::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.enabled(!self.disabled);
        if self.variant == IconButtonVariant::Toolbar {
            node.state("active", self.active);
            node.action("activate").reads(format!("{id}:active"));
        } else {
            node.action("activate");
        }
        node
    }

    pub fn variant(mut self, variant: IconButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: IconButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon_size(mut self, px: f32) -> Self {
        self.icon_size = Some(px);
        self
    }

    /// Intent has no surface in the kit; kept so call sites read alike
    /// with [`super::Button`].
    pub fn intent(self, _intent: Intent) -> Self {
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.press.pressed
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    fn recipe<'a>(&self, theme: &'a Theme) -> &'a ButtonRecipe {
        let b = &theme.button;
        match self.variant {
            IconButtonVariant::Outline => &b.outline,
            IconButtonVariant::Ghost => &b.ghost,
            IconButtonVariant::Toolbar if self.active => &b.toolbar_active,
            IconButtonVariant::Toolbar => &b.toolbar,
        }
    }

    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let m = self.size.metrics(theme);
        (m.size, m.size)
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, self.disabled)
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(compositor, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
    ) {
        if let Some(id) = &self.sem {
            compositor.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let m = self.size.metrics(theme);
        let recipe = self.recipe(theme);
        let alpha = if self.disabled {
            recipe.disabled_alpha
        } else {
            1.0
        };
        let radius = m.radius.min(bounds.h / 2.0);
        let fg = paint_surface(
            compositor,
            layer,
            bounds,
            radius,
            self.press.surface(recipe),
            alpha,
            self.press.focused,
            theme,
        );
        let icon = self.icon_size.unwrap_or(m.icon);
        if let Some(node) = icons::icon_at(
            self.icon,
            icon,
            fg,
            bounds.x + (bounds.w - icon) / 2.0,
            bounds.y + (bounds.h - icon) / 2.0,
        ) {
            compositor.push_to_layer(layer, node);
        }
    }
}
