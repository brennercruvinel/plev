//! The submit button of the kit (`components/SubmitButton`): a square
//! pale surface that turns into the dark one on hover or press, and stays
//! dark while `active` (a request in flight), easing back a step when
//! hovered then. The arrow follows: ink on paper, paper on ink. Two
//! sizes, the default and the kit's `isMedium`. Every value is
//! `theme.button.submit`.
//!
//! With a [`SemId`] ([`SubmitButton::sem`]) the render binds the bounds
//! it draws and hit-tests with, and [`SubmitButton::project`] declares
//! the button in a semantic projection, `active` as its `busy` state.

use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::{ButtonSurface, IconButtonMetrics, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::press::{Press, paint_surface};
use crate::widgets::icons;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The submit button's ladder (`theme.button.submit.sizes`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SubmitButtonSize {
    /// 32, the kit's `isMedium` (a comment's send).
    Md,
    /// 40, the default (the prompt's send).
    #[default]
    Lg,
}

impl SubmitButtonSize {
    pub fn metrics(self, theme: &Theme) -> IconButtonMetrics {
        theme.button.submit.sizes[self as usize]
    }
}

#[derive(Clone, Debug)]
pub struct SubmitButton {
    pub icon: &'static str,
    pub size: SubmitButtonSize,
    pub disabled: bool,
    pub active: bool,
    press: Press,
    sem: Option<SemId>,
}

impl SubmitButton {
    pub fn new() -> Self {
        Self {
            icon: "arrow-up",
            size: SubmitButtonSize::Lg,
            disabled: false,
            active: false,
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
    /// named `label` (the arrow names nothing, so the app gives the
    /// accessible label), enabled unless disabled, its `busy` state (a
    /// request in flight, the widget's `active`) and an `activate` action
    /// that reads `busy`, so a submit decided on an idle button goes stale
    /// once another one is in flight. What `activate` does is the app's;
    /// this only reads the widget. Pass the id given to
    /// [`SubmitButton::sem`] so the node and the binding name the same
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
        node.state("busy", self.active).enabled(!self.disabled);
        node.action("activate").reads(format!("{id}:busy"));
        node
    }

    pub fn size(mut self, size: SubmitButtonSize) -> Self {
        self.size = size;
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

    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused && !self.disabled;
    }

    fn metrics(&self, theme: &Theme) -> IconButtonMetrics {
        self.size.metrics(theme)
    }

    /// Pale at rest; dark hovered, pressed or active; hovered while
    /// active, the dark eased back over the pale.
    fn surface<'a>(&self, theme: &'a Theme) -> &'a ButtonSurface {
        let s = &theme.button.submit;
        match (self.active, self.press.hovered || self.press.pressed) {
            (true, true) => &s.busy_hover,
            (true, false) | (false, true) => &s.lit,
            (false, false) => &s.rest,
        }
    }

    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let m = self.metrics(theme);
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
        let m = self.metrics(theme);
        let alpha = if self.disabled {
            theme.button.submit.disabled_alpha
        } else {
            1.0
        };
        let radius = m.radius.min(bounds.h / 2.0);
        let fg = paint_surface(
            compositor,
            layer,
            bounds,
            radius,
            self.surface(theme),
            alpha,
            self.press.focused,
            theme,
        );
        let icon = m.icon;
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

impl Default for SubmitButton {
    fn default() -> Self {
        Self::new()
    }
}
