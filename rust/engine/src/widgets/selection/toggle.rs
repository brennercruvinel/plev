//! The toggle of the hoff core kit: a button that stays pressed. Its
//! surface is the kit's toolbar action (`components/Toolbar/Action`), the
//! `toolbar` recipe while off and `toolbar_active` while on, painted by
//! the buttons' own `paint_surface` over the buttons' own [`Press`]. No
//! second press model, no recipe of its own.
//!
//! Two forms. Icon only, the kit's action, on the icon-button ladder
//! (`size-10 rounded-xl`, a 20px icon: [`IconButtonSize::Lg`]). With a
//! label, and an optional leading icon, on the labeled ladder
//! ([`ButtonSize::Md`] by default, the same 40 / 12 as the action); the
//! kit has no labeled toggle, so the label borrows the button's.
//!
//! The size is intrinsic (the measured label plus the ladder's padding).
//! Given less than that, the label truncates to the bounds with an
//! ellipsis; [`Toggle::min_width`] is the narrowest that still shows one.
//!
//! A click (release inside) flips `on`, as [`Toggle::toggle`] does for a
//! reader or an agent. With a [`SemId`] ([`Toggle::sem`]) the render binds
//! the bounds it draws and hit-tests with, and [`Toggle::project`]
//! declares a `Role::Toggle` with its `on` state, as the switch does.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{ButtonMetrics, ButtonRecipe, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::{ButtonSize, IconButtonSize, Press, paint_surface};
use crate::widgets::icons;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The truncation mark, measured for [`Toggle::min_width`].
const ELLIPSIS: &str = "…";

#[derive(Clone, Debug)]
pub struct Toggle {
    /// The accessible name; drawn too, unless the toggle is icon only.
    pub label: String,
    /// The leading icon, or the whole face of an icon-only toggle.
    pub icon: Option<&'static str>,
    /// Draw the icon alone (the kit's toolbar action).
    pub icon_only: bool,
    pub on: bool,
    pub disabled: bool,
    /// Step of the labeled ladder.
    pub size: ButtonSize,
    /// Step of the icon-only ladder.
    pub icon_size: IconButtonSize,
    press: Press,
    sem: Option<SemId>,
}

impl Toggle {
    /// A labeled toggle, off.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            icon_only: false,
            on: false,
            disabled: false,
            size: ButtonSize::Md,
            icon_size: IconButtonSize::Lg,
            press: Press::default(),
            sem: None,
        }
    }

    /// The kit's action: `icon` alone, named `label` for a reader and an
    /// agent (an icon names nothing).
    pub fn icon_only(icon: &'static str, label: impl Into<String>) -> Self {
        Self {
            icon: Some(icon),
            icon_only: true,
            ..Self::new(label)
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

    /// Declares this toggle as `id` in a projection: a `Role::Toggle`
    /// named by its label, its `on` state, enabled unless disabled, and a
    /// `toggle` action that reads its own `on`. It only reads the widget;
    /// the app owns it and runs `toggle` ([`Toggle::toggle`]). Pass the id
    /// given to [`Toggle::sem`] so the node and the binding name the same
    /// control. Returns the node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Toggle, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("on", self.on).enabled(!self.disabled);
        node.action("toggle").reads(format!("{id}:on"));
        node
    }

    /// A leading icon beside the label.
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn icon_size(mut self, size: IconButtonSize) -> Self {
        self.icon_size = size;
        self
    }

    /// Flip `on`, as a click does. A disabled toggle stays as it is;
    /// returns whether it flipped.
    pub fn toggle(&mut self) -> bool {
        if self.disabled {
            return false;
        }
        self.on = !self.on;
        true
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.press.pressed
    }

    /// Keyboard focus, driven by the owning view. Disabled toggles
    /// refuse it.
    pub fn set_focused(&mut self, focused: bool) {
        self.press.focused = focused && !self.disabled;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    /// The kit's toolbar recipe for this state: selected or not.
    fn recipe<'a>(&self, theme: &'a Theme) -> &'a ButtonRecipe {
        if self.on {
            &theme.button.toolbar_active
        } else {
            &theme.button.toolbar
        }
    }

    /// Label style, one source for measuring and drawing.
    pub fn text_style(&self, theme: &Theme) -> TextStyle {
        self.size.text_style(theme)
    }

    /// Width the leading icon takes, gap included; zero without one.
    fn icon_slot(&self, m: &ButtonMetrics) -> f32 {
        if self.icon.is_some() {
            m.icon + m.gap
        } else {
            0.0
        }
    }

    /// Intrinsic size: the icon-only square, or the measured label with
    /// the icon slot and the ladder's padding.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        if self.icon_only {
            let m = self.icon_size.metrics(theme);
            return (m.size, m.size);
        }
        let m = self.size.metrics(theme);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &self.text_style(theme), None);
        (
            (text_w + self.icon_slot(&m) + m.pad_x * 2.0).ceil(),
            m.height,
        )
    }

    /// The narrowest width that still shows the label as an ellipsis
    /// (the square for an icon-only toggle).
    pub fn min_width(&self, theme: &Theme) -> f32 {
        if self.icon_only {
            return self.icon_size.metrics(theme).size;
        }
        let m = self.size.metrics(theme);
        let (mark, _) = TextMeasurer::measure_styled(ELLIPSIS, &self.text_style(theme), None);
        (mark + self.icon_slot(&m) + m.pad_x * 2.0).ceil()
    }

    /// The label as drawn in `bounds`: whole when it fits, truncated with
    /// an ellipsis when it does not.
    pub fn shown_label(&self, bounds: Rect, theme: &Theme) -> String {
        let m = self.size.metrics(theme);
        let style = self.text_style(theme);
        let room = (bounds.w - m.pad_x * 2.0 - self.icon_slot(&m)).max(0.0);
        let (text_w, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        if text_w <= room {
            self.label.clone()
        } else {
            TextMeasurer::truncate_to_width(&self.label, &style, room)
        }
    }

    /// Pointer events against `bounds`: the shared press model, and a
    /// click flips `on`.
    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
        let result = self.press.handle(event, bounds, self.disabled);
        if result.clicked {
            self.toggle();
        }
        result
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
        let recipe = self.recipe(theme);
        let alpha = if self.disabled {
            recipe.disabled_alpha
        } else {
            1.0
        };
        let radius = if self.icon_only {
            self.icon_size.metrics(theme).radius
        } else {
            self.size.metrics(theme).radius
        }
        .min(bounds.h / 2.0);
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
        if self.icon_only {
            let edge = self.icon_size.metrics(theme).icon;
            self.draw_icon(
                compositor,
                layer,
                edge,
                fg,
                bounds.x + (bounds.w - edge) / 2.0,
                bounds,
            );
            return;
        }

        let m = self.size.metrics(theme);
        let style = self.text_style(theme);
        let shown = self.shown_label(bounds, theme);
        let (text_w, _) = TextMeasurer::measure_styled(&shown, &style, None);
        let content_w = text_w + self.icon_slot(&m);
        let mut cx = (bounds.x + (bounds.w - content_w) / 2.0).max(bounds.x + m.pad_x);
        if self.icon.is_some() {
            self.draw_icon(compositor, layer, m.icon, fg, cx, bounds);
            cx += self.icon_slot(&m);
        }
        compositor.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x: cx,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: fg,
            },
        );
    }

    /// The icon, `edge` square, at `x`, centered on the bounds' height.
    fn draw_icon(
        &self,
        compositor: &mut Compositor,
        layer: LayerId,
        edge: f32,
        color: [f32; 4],
        x: f32,
        bounds: Rect,
    ) {
        let Some(name) = self.icon else {
            return;
        };
        if let Some(node) = icons::icon_at(name, edge, color, x, bounds.y + (bounds.h - edge) / 2.0)
        {
            compositor.push_to_layer(layer, node);
        }
    }
}
