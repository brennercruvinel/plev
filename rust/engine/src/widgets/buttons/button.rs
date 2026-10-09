//! The labeled button of the hoff core kit: a label, an optional leading
//! icon, one of the kit's surface recipes (`theme.button`) and one step
//! of its height ladder. Click fires on release inside the bounds;
//! keyboard focus (via [`Button::set_focused`]) draws the shared focus
//! ring.
//!
//! Nothing visual lives here: fills, rings, key-lights and shadows come
//! from the recipe, heights, paddings, radii and the label style from
//! the size metrics, so the same widget renders the core kit and every
//! derived theme.
//!
//! With a [`SemId`] ([`Button::sem`]) the render binds the bounds it
//! draws and hit-tests with, and [`Button::project`] declares the button
//! in a semantic projection.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{ButtonMetrics, ButtonRecipe, Intent, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::press::{Press, paint_surface};
use crate::widgets::icons;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Which recipe of `theme.button` the surface reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// The pale raised button (`isPrimary`), the default.
    #[default]
    Primary,
    /// The dark call-to-action (`isSecondary`).
    Secondary,
    /// The orange warning button (`isOrange`).
    Warning,
    /// Bordered and flat: the like button.
    Outline,
    /// No border, a wash on hover.
    Ghost,
}

/// The height ladder of the kit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// 28: the like-comment chip.
    Xs,
    /// 36: `isSmall`.
    Sm,
    /// 40: the default.
    #[default]
    Md,
    /// 44: the like button.
    Lg,
}

impl ButtonSize {
    pub fn metrics(self, theme: &Theme) -> ButtonMetrics {
        theme.button.sizes[self as usize]
    }

    pub fn height(self, theme: &Theme) -> f32 {
        self.metrics(theme).height
    }

    /// Label style, one source for measuring and drawing.
    pub fn text_style(self, theme: &Theme) -> TextStyle {
        let m = self.metrics(theme);
        theme
            .typography
            .exact(m.font_size, m.line_height, m.font_weight)
    }
}

#[derive(Clone, Debug)]
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    /// Optional leading icon (a [`crate::icons`] name).
    pub icon: Option<&'static str>,
    /// Icon color when it should not follow the label (the like heart).
    pub icon_color: Option<[f32; 4]>,
    press: Press,
    sem: Option<SemId>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            icon: None,
            icon_color: None,
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
    /// named by its label, enabled unless disabled, with an `activate`
    /// action. What `activate` does is the app's; this only reads the
    /// widget. Pass the id given to [`Button::sem`] so the node and the
    /// binding name the same control. Returns the node for the app to add
    /// to (the state `activate` reads, its policy).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.enabled(!self.disabled);
        node.action("activate");
        node
    }

    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Intent maps onto the kit's recipes: destructive is the warning
    /// button, everything else keeps the current variant.
    pub fn intent(mut self, intent: Intent) -> Self {
        if intent == Intent::Destructive {
            self.variant = ButtonVariant::Warning;
        }
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

    pub fn icon_color(mut self, color: [f32; 4]) -> Self {
        self.icon_color = Some(color);
        self
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.press.pressed
    }

    /// Keyboard focus, driven by the owning view (plev has no global
    /// focus chain). Disabled buttons refuse focus.
    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    fn recipe<'a>(&self, theme: &'a Theme) -> &'a ButtonRecipe {
        let b = &theme.button;
        match self.variant {
            ButtonVariant::Primary => &b.primary,
            ButtonVariant::Secondary => &b.secondary,
            ButtonVariant::Warning => &b.warning,
            ButtonVariant::Outline => &b.outline,
            ButtonVariant::Ghost => &b.ghost,
        }
    }

    /// Width the leading icon takes, gap included; zero without an icon.
    fn icon_slot(&self, m: &ButtonMetrics) -> f32 {
        if self.icon.is_some() {
            m.icon + m.gap
        } else {
            0.0
        }
    }

    /// Intrinsic size from real text measurement.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let m = self.size.metrics(theme);
        let style = self.size.text_style(theme);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        (
            (text_w + self.icon_slot(&m) + m.pad_x * 2.0).ceil(),
            m.height,
        )
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        self.press.handle(event, bounds, self.disabled)
    }

    pub fn render(&self, compositor: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(compositor, LayerId::DEFAULT, bounds, theme);
    }

    /// Render onto a specific layer (modals and overlays embed buttons).
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

        let style = self.size.text_style(theme);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let content_w = text_w + self.icon_slot(&m);
        let mut cx = bounds.x + (bounds.w - content_w) / 2.0;

        if let Some(name) = self.icon {
            let color = match self.icon_color {
                Some(c) => [c[0], c[1], c[2], c[3] * alpha],
                None => fg,
            };
            if let Some(node) = icons::icon_at(
                name,
                m.icon,
                color,
                cx,
                bounds.y + (bounds.h - m.icon) / 2.0,
            ) {
                compositor.push_to_layer(layer, node);
            }
            cx += self.icon_slot(&m);
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
