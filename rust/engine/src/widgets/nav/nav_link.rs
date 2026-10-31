//! Nav link: a row that goes somewhere. An icon in its box (a bare slot,
//! or the kit's tile that rises while the link is current), a label, an
//! optional trailing shortcut (in the kit's key cap, task-104d), hint
//! (plain text) or count badge; the row's box and tones change at rest,
//! hovered, pressed and active. In a rail (no room for labels) only the
//! icon's box draws, centered. Every visual value is a
//! [`NavLinkRecipe`]: the theme's `navigation.link` on its own,
//! `navigation.sidebar_link` inside the sidebar
//! ([`NavLink::render_with`]); the key cap is the Kbd's own.
//!
//! With a [`SemId`] ([`NavLink::sem`]) the render binds the bounds it
//! draws and hit-tests with (the rail form too), and
//! [`NavLink::project`] declares the link in a semantic projection.
//! Keyboard focus ([`NavLink::set_focused`], driven by the owning view)
//! rings the row.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::widgets::buttons::Badge;
use crate::widgets::content::Kbd;
use crate::widgets::icons;
use crate::widgets::nav::tokens::{LinkSurface, NavLinkRecipe};
use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke, shadow_node};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

#[derive(Clone, Debug)]
pub struct NavLink {
    pub label: String,
    pub icon: Option<&'static str>,
    /// Trailing hint text in the placeholder tone (a status, a count).
    pub hint: Option<String>,
    /// Trailing keyboard shortcut, drawn in the kit's key cap.
    pub shortcut: Option<String>,
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
            shortcut: None,
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

    /// A keyboard shortcut, drawn in the kit's key cap.
    pub fn shortcut(mut self, keys: impl Into<String>) -> Self {
        self.shortcut = Some(keys.into());
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

    /// Row height of the link on its own.
    pub fn height(theme: &Theme) -> f32 {
        theme.navigation.link.height
    }

    /// The icon's box of the link on its own: its tile or its slot.
    pub fn icon_slot(theme: &Theme) -> f32 {
        theme.navigation.link.icon_box()
    }

    pub fn label_style(theme: &Theme) -> TextStyle {
        theme.navigation.link.label.style(&theme.typography)
    }

    /// Width the link wants with its label shown, on its own.
    pub fn preferred_width(&self, theme: &Theme) -> f32 {
        self.preferred_width_with(theme, &theme.navigation.link)
    }

    /// Width the link wants with its label shown, drawn with `recipe`.
    pub fn preferred_width_with(&self, theme: &Theme, recipe: &NavLinkRecipe) -> f32 {
        let typo = &theme.typography;
        let mut w = recipe.pad_start + recipe.pad_end;
        if self.icon.is_some() {
            w += recipe.icon_box() + recipe.icon_gap;
        }
        w += TextMeasurer::measure_styled(&self.label, &recipe.label.style(typo), None).0;
        if let Some(keys) = &self.shortcut {
            w += recipe.trailing_room + Kbd::new(keys.as_str()).preferred_size(theme).0;
        } else if let Some(hint) = &self.hint {
            w += recipe.trailing_room
                + TextMeasurer::measure_styled(hint, &recipe.hint.style(typo), None).0;
        }
        if let Some(badge) = &self.badge {
            w += recipe.trailing_room + badge.preferred_size(theme).0;
        }
        w.ceil()
    }

    /// Whether `bounds` is wide enough for the label; narrower rows draw
    /// the rail form (icon only).
    fn shows_label(bounds: Rect, recipe: &NavLinkRecipe) -> bool {
        bounds.w >= recipe.icon_box() + recipe.pad_start + recipe.pad_end + recipe.label_min
    }

    /// The row's face now: active over pressed over hovered over rest.
    fn surface<'r>(&self, recipe: &'r NavLinkRecipe) -> &'r LinkSurface {
        if self.active {
            &recipe.active
        } else if self.pressed && self.hovered {
            &recipe.pressed
        } else if self.hovered {
            &recipe.hover
        } else {
            &recipe.rest
        }
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
        self.render_with(c, layer, bounds, theme, &theme.navigation.link);
    }

    /// Draw the link with `recipe` (the sidebar passes its own).
    pub fn render_with(
        &self,
        c: &mut Compositor,
        layer: LayerId,
        bounds: Rect,
        theme: &Theme,
        recipe: &NavLinkRecipe,
    ) {
        if let Some(id) = &self.sem {
            c.bind_to_layer(layer, id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let radius = recipe.radius.min(bounds.h / 2.0);
        if self.focused {
            c.push_to_layer(layer, focus_ring(bounds, radius, theme));
        }
        let face = self.surface(recipe);
        let Rect { x, y, w, h } = bounds;
        if face.fill.0[3] > 0.0 {
            c.push_to_layer(layer, rounded_rect(x, y, w, h, radius, face.fill.0));
        }
        if face.border.0[3] > 0.0 && recipe.border_width > 0.0 {
            let rim = rounded_rect_stroke(x, y, w, h, radius, face.border.0, recipe.border_width);
            c.push_to_layer(layer, rim);
        }

        let slot = recipe.icon_box();
        let show_label = Self::shows_label(bounds, recipe);
        let mut x = bounds.x + recipe.pad_start;
        if let Some(icon) = self.icon {
            // Rail form: the box centers in the row instead of leading it.
            let slot_x = if show_label {
                x
            } else {
                bounds.x + (bounds.w - slot) / 2.0
            };
            let at = Rect::new(slot_x, bounds.y + (bounds.h - slot) / 2.0, slot, slot);
            if let Some(tile) = recipe.tile.filter(|_| self.active) {
                if let Some(node) = shadow_node(at, tile.radius, &tile.shadow_current) {
                    c.push_to_layer(layer, node);
                }
                let raised = rounded_rect(at.x, at.y, at.w, at.h, tile.radius, tile.fill_current.0);
                c.push_to_layer(layer, raised);
            }
            let px = recipe.tile.map_or(recipe.icon, |t| t.icon);
            let (ix, iy) = (at.x + (slot - px) / 2.0, at.y + (slot - px) / 2.0);
            if let Some(node) = icons::icon_at(icon, px, face.icon.0, ix, iy) {
                c.push_to_layer(layer, node);
            }
            x += slot + recipe.icon_gap;
        }
        if !show_label {
            return;
        }

        // Trailing hint or badge, right-aligned; the label truncates
        // against whatever is left.
        let mut right = bounds.x + bounds.w - recipe.pad_end;
        if let Some(badge) = &self.badge {
            let (bw, bh) = badge.preferred_size(theme);
            right -= bw;
            badge.render_to_layer(
                c,
                layer,
                Rect::new(right, bounds.y + (bounds.h - bh) / 2.0, bw, bh),
                theme,
            );
            right -= recipe.trailing_gap;
        } else if let Some(keys) = &self.shortcut {
            let cap = Kbd::new(keys.as_str());
            let (kw, kh) = cap.preferred_size(theme);
            right -= kw;
            let at = Rect::new(right, bounds.y + (bounds.h - kh) / 2.0, kw, kh);
            cap.render_to_layer(c, layer, at, theme);
            right -= recipe.trailing_gap;
        } else if let Some(hint) = &self.hint {
            let style = recipe.hint.style(&theme.typography);
            let (hw, _) = TextMeasurer::measure_styled(hint, &style, None);
            right -= hw;
            c.push_to_layer(
                layer,
                SceneNode::Text {
                    key: TextNodeKey::from_style(hint, &style, None),
                    x: right,
                    y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                    color: recipe.hint_color.0,
                },
            );
            right -= recipe.trailing_gap;
        }
        let style = recipe.label.style(&theme.typography);
        let avail = (right - x).max(0.0);
        let shown = TextMeasurer::truncate_to_width(&self.label, &style, avail);
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&shown, &style, None),
                x,
                y: bounds.y + TextMeasurer::vertical_center(&style, bounds.h),
                color: face.label.0,
            },
        );
    }
}
