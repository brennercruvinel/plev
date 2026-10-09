//! HOFF nav link: the sidebar row. One `Lg` control tall at the nav
//! radius, an icon in its slot (the small icon centered in an `Xs`-wide
//! box), a base-2sm label, an optional trailing hint (a shortcut digit)
//! or count badge. Faint at rest, text-default hovered, text-active plus
//! the surface-active wash and the edge rim when active. In a rail (no
//! room for labels) only the icon slot draws, centered.
//!
//! With a [`SemId`] ([`NavLink::sem`]) the render binds the bounds it
//! draws and hit-tests with (the rail form too), and
//! [`NavLink::project`] declares the link in a semantic projection.
//! Keyboard focus ([`NavLink::set_focused`], driven by the owning view)
//! rings the row.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{ControlSize, IconSize, Theme};
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::Badge;
use crate::widgets::icons;
use crate::widgets::recipe::focus_ring;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

#[derive(Clone, Debug)]
pub struct NavLink {
    pub label: String,
    pub icon: Option<&'static str>,
    /// Trailing hint text (a shortcut) in the placeholder tone.
    pub hint: Option<String>,
    /// Trailing count badge (unread, changes).
    pub badge: Option<Badge>,
    pub active: bool,
    hovered: bool,
    pressed: bool,
    focused: bool,
    sem: Option<SemId>,
}

impl NavLink {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            hint: None,
            badge: None,
            active: false,
            hovered: false,
            pressed: false,
            focused: false,
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

    /// Declares this link as `id` in a projection: a `Role::Link` named by
    /// its label (also in the rail form, where the label is not drawn),
    /// its `active` state (the place on screen) and an `activate` action
    /// that reads it. What `activate` does (navigate) is the app's; this
    /// only reads the widget. Pass the id given to [`NavLink::sem`] so the
    /// node and the binding name the same control. Returns the node for
    /// the app to add to (a description with the hint or the badge).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Link, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("active", self.active);
        node.action("activate").reads(format!("{id}:active"));
        node
    }

    pub fn icon(mut self, icon: &'static str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn badge(mut self, badge: Badge) -> Self {
        self.badge = Some(badge);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Keyboard focus, driven by the owning view (plev has no global
    /// focus chain): the accent ring around the row, at the nav radius.
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Row height: one `Lg` control.
    pub fn height(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Lg)
    }

    /// The icon slot: an `Xs`-wide square.
    pub fn icon_slot(theme: &Theme) -> f32 {
        theme.control.height(ControlSize::Xs)
    }

    pub fn label_style(theme: &Theme) -> TextStyle {
        theme.typography.base_2sm()
    }

    fn hint_style(theme: &Theme) -> TextStyle {
        theme.typography.caption_r()
    }

    /// Horizontal padding inside the row: the `sm` step (the icon slot
    /// carries its own centering).
    fn pad(theme: &Theme) -> f32 {
        theme.spacing.sm
    }

    /// Width the link wants with its label shown.
    pub fn preferred_width(&self, theme: &Theme) -> f32 {
        let pad = Self::pad(theme);
        let mut w = pad * 2.0;
        if self.icon.is_some() {
            w += Self::icon_slot(theme) + theme.spacing.xs;
        }
        w += TextMeasurer::measure_styled(&self.label, &Self::label_style(theme), None).0;
        if let Some(hint) = &self.hint {
            w += theme.spacing.lg
                + TextMeasurer::measure_styled(hint, &Self::hint_style(theme), None).0;
        }
        if let Some(badge) = &self.badge {
            w += theme.spacing.lg + badge.preferred_size(theme).0;
        }
        w.ceil()
    }

    /// Whether `bounds` is wide enough for the label; narrower rows draw
    /// the rail form (icon only).
    fn shows_label(&self, bounds: Rect, theme: &Theme) -> bool {
        bounds.w >= Self::icon_slot(theme) + Self::pad(theme) * 2.0 + theme.spacing.xl
    }

    pub fn handle_event(&mut self, event: &WidgetEvent, bounds: Rect) -> EventResult {
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

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let glass = &theme.glass;
        let radius = theme.shape.nav.min(bounds.h / 2.0);
        if self.focused {
            c.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }
        // The wash and the rim are the active mark; a hovered link only
        // brightens its text, so the two states never read alike.
        if self.active {
            c.push_to_layer(
                layer,
                rounded_rect(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    radius,
                    glass.surface_active.0,
                ),
            );
            c.push_to_layer(
                layer,
                rounded_rect_stroke(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    radius,
                    glass.edge.0,
                    theme.control.edge_width,
                ),
            );
        }
        let fg = if self.active {
            glass.text_active.0
        } else if self.hovered {
            glass.text_default.0
        } else {
            glass.text_faint.0
        };

        let pad = Self::pad(theme);
        let slot = Self::icon_slot(theme);
        let icon_px = theme.control.icon(IconSize::Md);
        let show_label = self.shows_label(bounds, theme);
        let mut x = bounds.x + pad;
        if let Some(icon) = self.icon {
            // Rail form: the slot centers in the row instead of leading it.
            let slot_x = if show_label {
                x
            } else {
                bounds.x + (bounds.w - slot) / 2.0
            };
            if let Some(node) = icons::icon_at(
                icon,
                icon_px,
                fg,
                slot_x + (slot - icon_px) / 2.0,
                bounds.y + (bounds.h - icon_px) / 2.0,
            ) {
                c.push_to_layer(layer, node);
            }
            x += slot + theme.spacing.xs;
        }
        if !show_label {
            return;
        }

        // Trailing hint or badge, right-aligned; the label truncates
        // against whatever is left.
        let mut right = bounds.x + bounds.w - pad;
        if let Some(badge) = &self.badge {
            let (bw, bh) = badge.preferred_size(theme);
            right -= bw;
            badge.render_to_layer(
                c,
                layer,
                Rect::new(right, bounds.y + (bounds.h - bh) / 2.0, bw, bh),
                theme,
            );
            right -= theme.spacing.sm;
        } else if let Some(hint) = &self.hint {
            let style = Self::hint_style(theme);
            let (hw, _) = TextMeasurer::measure_styled(hint, &style, None);
            right -= hw;
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(hint, &style, None),
                    x: right,
                    y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                    color: glass.text_placeholder.0,
                },
            );
            right -= theme.spacing.sm;
        }
        let style = Self::label_style(theme);
        let avail = (right - x).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&self.label, &style, avail);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: fg,
            },
        );
    }
}
