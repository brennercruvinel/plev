//! The collapsible, the primitive that opens and closes: a trigger (the
//! label, an optional icon tile, a chevron that turns half a circle) and
//! the region it reveals below, whose height grows from 0 to the
//! content's measured height. The [`Accordion`] stacks several.
//!
//! Nothing visual lives here: the trigger's box, its states, the label
//! and the chevron come from a recipe of `theme.disclosure` (the kit's
//! Dropdown or its Accordion, [`DisclosureLook`]), the motion from a
//! [`DisclosureMotion`]. The height moves as a fraction of the content's
//! measured height, so a width change mid-motion stays right; with the
//! system's "reduce motion" on (`compose::animation::reduced_motion`) it
//! opens and closes at once, as the kit's `AnimateHeight` does.
//!
//! The widget does not know its content: the caller measures it, gives
//! its height to [`Collapsible::height`] and draws it in the closure of
//! [`Collapsible::render`], clipped to what is revealed.
//!
//! With a [`SemId`] ([`Collapsible::sem`]) the render binds the trigger,
//! and [`Collapsible::project`] declares it a `Role::Button` with its
//! `expanded` state and a `toggle_expanded` that reads it (no adapter
//! says `expanded` yet, adr-0068: the reader hears the control and the
//! region that appears); [`Collapsible::project_panel`] declares the
//! region while it is open.
//!
//! [`Accordion`]: super::accordion::Accordion

use std::hash::{DefaultHasher, Hash, Hasher};

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::{DisclosureMotion, DisclosureRecipe, DisclosureSurface};
use crate::compose::animation::{Tween, reduced_motion};
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_node};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Which recipe of `theme.disclosure` the trigger reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisclosureLook {
    /// The kit's Dropdown: a bordered, rounded trigger.
    #[default]
    Collapsible,
    /// The kit's Accordion: a divider above, a bare header.
    Accordion,
}

/// A trigger and the region it reveals.
#[derive(Clone, Debug)]
pub struct Collapsible {
    title: String,
    icon: Option<&'static str>,
    look: DisclosureLook,
    open: bool,
    disabled: bool,
    current: bool,
    /// The first section of a stack draws no divider above it.
    first: bool,
    press: Press,
    motion: DisclosureMotion,
    /// The revealed fraction, 0 closed to 1 open, eased.
    reveal: Tween<f32>,
    /// The chevron's half turn, 0 to 1.
    turn: Tween<f32>,
    sem: Option<SemId>,
}

/// The trigger's inner geometry: where the tile, the label and the
/// chevron box go.
struct Inner {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label_w: f32,
    label_h: f32,
}

impl Collapsible {
    /// Closed, enabled, the Collapsible look, the kit's motion.
    pub fn new(title: impl Into<String>) -> Self {
        let motion = DisclosureMotion::hoff_core();
        Self {
            title: title.into(),
            icon: None,
            look: DisclosureLook::default(),
            open: false,
            disabled: false,
            current: false,
            first: false,
            press: Press::default(),
            motion,
            reveal: Tween::new(0.0, motion.reveal, motion.reveal_easing),
            turn: Tween::new(0.0, motion.turn, motion.turn_easing),
            sem: None,
        }
    }

    pub fn look(mut self, look: DisclosureLook) -> Self {
        self.look = look;
        self
    }

    /// A leading icon, in the tile of a look that has one.
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    /// The section holds the current page (the Dropdown's `active`).
    pub fn current(mut self, on: bool) -> Self {
        self.current = on;
        self
    }

    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        if on {
            self.press = Press::default();
        }
        self
    }

    /// No divider above: the first section of a stack.
    pub fn first(mut self, on: bool) -> Self {
        self.first = on;
        self
    }

    /// Open or closed from the start, at rest (nothing moves).
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        let at = if on { 1.0 } else { 0.0 };
        self.reveal.reset(at);
        self.turn.reset(at);
        self
    }

    /// The motion of a theme (`theme.disclosure.motion`), once at setup:
    /// a motion under way is settled at its end.
    pub fn with_motion(mut self, motion: &DisclosureMotion) -> Self {
        let at = if self.open { 1.0 } else { 0.0 };
        self.motion = *motion;
        self.reveal = Tween::new(at, motion.reveal, motion.reveal_easing);
        self.turn = Tween::new(at, motion.turn, motion.turn_easing);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn set_focused(&mut self, on: bool) {
        self.press.focused = on;
    }

    pub fn focused(&self) -> bool {
        self.press.focused
    }

    /// Opens or closes it; `false` when disabled or already there. The
    /// height and the chevron move from where they are, or land at once
    /// with the motion reduced.
    pub fn set_open(&mut self, on: bool) -> bool {
        if self.disabled || self.open == on {
            return false;
        }
        self.open = on;
        let to = if on { 1.0 } else { 0.0 };
        if reduced_motion() {
            self.reveal.reset(to);
            self.turn.reset(to);
        } else {
            self.reveal.set_target(to);
            self.turn.set_target(to);
        }
        true
    }

    /// Flips it; `false` when disabled.
    pub fn toggle(&mut self) -> bool {
        self.set_open(!self.open)
    }

    /// The revealed fraction now, 0 to 1.
    pub fn revealed(&self) -> f32 {
        self.reveal.get().clamp(0.0, 1.0)
    }

    pub fn is_animating(&self) -> bool {
        self.reveal.is_animating() || self.turn.is_animating()
    }

    /// Advances the motion: a frame now and the next while it moves, a
    /// last frame when it lands, nothing at rest.
    pub fn wake(&mut self, dt: f32) -> Wake {
        let was = self.is_animating();
        self.reveal.tick(dt);
        self.turn.tick(dt);
        if self.is_animating() {
            Wake::continuous()
        } else if was {
            Wake::now()
        } else {
            Wake::idle()
        }
    }

    fn recipe<'t>(&self, theme: &'t Theme) -> &'t DisclosureRecipe {
        match self.look {
            DisclosureLook::Collapsible => &theme.disclosure.collapsible,
            DisclosureLook::Accordion => &theme.disclosure.accordion,
        }
    }

    fn label_style(r: &DisclosureRecipe, theme: &Theme) -> TextStyle {
        theme
            .typography
            .exact(r.font_size, r.line_height, r.font_weight)
    }

    /// The divider's share of the height: none on the first section.
    fn divider_h(&self, r: &DisclosureRecipe) -> f32 {
        if self.first { 0.0 } else { r.divider_width }
    }

    /// The tile's edge and the gap after it, when there is one to draw.
    fn tile_span(&self, r: &DisclosureRecipe) -> Option<(f32, f32)> {
        let tile = r.tile.as_ref()?;
        self.icon.map(|_| (tile.size, tile.gap))
    }

    /// The label wraps in what the tile and the chevron leave.
    fn inner(&self, trigger: Rect, theme: &Theme) -> Inner {
        let r = self.recipe(theme);
        let [top, right, _, left] = r.pad;
        let bw = r.border_width;
        let x = trigger.x + bw + left;
        let w = (trigger.w - bw * 2.0 - left - right).max(0.0);
        let tile = self.tile_span(r);
        let label_w =
            (w - tile.map_or(0.0, |(size, gap)| size + gap) - r.chevron_gap - r.chevron_box)
                .max(0.0);
        let style = Self::label_style(r, theme);
        let (_, text_h) = TextMeasurer::measure_styled(&self.title, &style, Some(label_w));
        let label_h = text_h.max(style.line_height);
        let h = label_h
            .max(r.chevron_box)
            .max(tile.map_or(0.0, |(size, _)| size));
        Inner {
            x,
            y: trigger.y + bw + top,
            w,
            h,
            label_w,
            label_h,
        }
    }

    /// The trigger's height at width `w`, its divider included: the
    /// label wraps, so a narrower trigger can be taller.
    pub fn trigger_height(&self, w: f32, theme: &Theme) -> f32 {
        let r = self.recipe(theme);
        let [top, _, bottom, _] = r.pad;
        let inner = self.inner(Rect::new(0.0, 0.0, w, 0.0), theme);
        self.divider_h(r) + r.border_width * 2.0 + top + inner.h + bottom
    }

    /// The region's full height around content `content_h` tall.
    fn panel_height(&self, content_h: f32, theme: &Theme) -> f32 {
        let [top, _, bottom, _] = self.recipe(theme).content_pad;
        top + content_h.max(0.0) + bottom
    }

    /// The whole height now at width `w` around content `content_h` tall:
    /// the trigger and the revealed part of the region.
    pub fn height(&self, w: f32, content_h: f32, theme: &Theme) -> f32 {
        self.trigger_height(w, theme) + self.revealed() * self.panel_height(content_h, theme)
    }

    /// Where the trigger is in `bounds`: under the divider, full width.
    pub fn trigger_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let r = self.recipe(theme);
        let divider = self.divider_h(r);
        let h = self.trigger_height(bounds.w, theme) - divider;
        Rect::new(bounds.x, bounds.y + divider, bounds.w, h)
    }

    /// Where the content goes at full size, `content_h` tall; the render
    /// clips it to what is revealed.
    pub fn content_rect(&self, bounds: Rect, content_h: f32, theme: &Theme) -> Rect {
        let [top, right, _, left] = self.recipe(theme).content_pad;
        let trigger = self.trigger_rect(bounds, theme);
        Rect::new(
            bounds.x + left,
            trigger.bottom() + top,
            (bounds.w - left - right).max(0.0),
            content_h.max(0.0),
        )
    }

    /// The revealed part of the region: below the trigger, to the bottom
    /// of `bounds`.
    pub fn shown_rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let trigger = self.trigger_rect(bounds, theme);
        Rect::new(
            bounds.x,
            trigger.bottom(),
            bounds.w,
            (bounds.bottom() - trigger.bottom()).max(0.0),
        )
    }

    /// Hover and press on the trigger; a release inside it flips the
    /// disclosure (`clicked`). A disabled one reacts to nothing.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let trigger = self.trigger_rect(bounds, theme);
        let r = self.press.handle(event, trigger, self.disabled);
        if r.clicked {
            self.toggle();
        }
        r
    }

    fn surface<'r>(&self, r: &'r DisclosureRecipe) -> &'r DisclosureSurface {
        if self.current {
            &r.current
        } else if self.open {
            &r.open
        } else if self.press.hovered {
            &r.hover
        } else {
            &r.rest
        }
    }

    /// Draws the divider, the trigger and, clipped to what is revealed,
    /// the content through `content` at its full-size rect (`content_h`
    /// tall, the height given to [`Collapsible::height`]).
    pub fn render(
        &self,
        c: &mut Compositor,
        bounds: Rect,
        content_h: f32,
        theme: &Theme,
        content: impl FnOnce(&mut Compositor, Rect),
    ) {
        let r = self.recipe(theme);
        let alpha = if self.disabled {
            theme.glass.disabled_alpha
        } else {
            1.0
        };
        let divider = self.divider_h(r);
        if divider > 0.0 && r.divider.0[3] > 0.0 {
            c.push(SceneNode::Rect {
                x: bounds.x,
                y: bounds.y,
                w: bounds.w,
                h: divider,
                color: faded(r.divider, alpha),
            });
        }
        let trigger = self.trigger_rect(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [trigger.x, trigger.y, trigger.w, trigger.h]);
        }
        self.paint_trigger(c, trigger, r, alpha, theme);

        let shown = self.shown_rect(bounds, theme);
        if shown.h > 0.0 {
            let full = self.content_rect(bounds, content_h, theme);
            c.push_clip(shown.x, shown.y, shown.w, shown.h);
            if self.open
                && let Some(panel) = self.sem.as_ref().and_then(|id| id.child("panel").ok())
            {
                c.bind(&panel, [full.x, full.y, full.w, full.h]);
            }
            content(c, full);
            c.pop_clip();
        }
    }

    fn paint_trigger(
        &self,
        c: &mut Compositor,
        trigger: Rect,
        r: &DisclosureRecipe,
        alpha: f32,
        theme: &Theme,
    ) {
        let radius = r.radius.min(trigger.h / 2.0);
        let surface = self.surface(r);
        if surface.fill.0[3] > 0.0 {
            c.push(rounded_rect(
                trigger.x,
                trigger.y,
                trigger.w,
                trigger.h,
                radius,
                faded(surface.fill, alpha),
            ));
        }
        if r.border_width > 0.0 && surface.border.0[3] > 0.0 {
            c.push(rounded_rect_stroke(
                trigger.x,
                trigger.y,
                trigger.w,
                trigger.h,
                radius,
                faded(surface.border, alpha),
                r.border_width,
            ));
        }
        if self.press.focused && !self.disabled {
            c.push(focus_ring_inside(trigger, radius, theme));
        }

        let inner = self.inner(trigger, theme);
        let mut x = inner.x;
        if let (Some(tile), Some(name)) = (r.tile.as_ref(), self.icon) {
            let at = Rect::new(
                x,
                inner.y + (inner.h - tile.size) / 2.0,
                tile.size,
                tile.size,
            );
            let color = if self.current {
                if let Some(node) = shadow_node(at, tile.radius, &tile.shadow_current) {
                    c.push(node);
                }
                c.push(rounded_rect(
                    at.x,
                    at.y,
                    at.w,
                    at.h,
                    tile.radius,
                    faded(tile.fill_current, alpha),
                ));
                tile.icon_current
            } else {
                tile.icon_rest
            };
            let inset = (tile.size - tile.icon) / 2.0;
            if let Some(node) = icons::icon_at(
                name,
                tile.icon,
                faded(color, alpha),
                at.x + inset,
                at.y + inset,
            ) {
                c.push(node);
            }
            x += tile.size + tile.gap;
        }

        let style = Self::label_style(r, theme);
        let label_y = inner.y + (inner.h - inner.label_h) / 2.0;
        c.push_clip(x, label_y, inner.label_w, inner.label_h);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&self.title, &style, Some(inner.label_w)),
            x,
            y: label_y,
            color: faded(r.label, alpha),
        });
        c.pop_clip();

        let box_x = inner.x + inner.w - r.chevron_box;
        let box_y = inner.y + (inner.h - r.chevron_box) / 2.0;
        let inset = (r.chevron_box - r.chevron) / 2.0;
        if let Some(node) = turned(
            "chevron-down",
            r.chevron,
            faded(r.chevron_color, alpha),
            box_x + inset,
            box_y + inset,
            self.turn.get() * std::f32::consts::PI,
        ) {
            c.push(node);
        }
    }

    /// Declares the trigger as `id`: a `Role::Button` named by the title,
    /// enabled unless disabled, with `expanded` and a `toggle_expanded`
    /// that reads it. Hover is never projected. It only reads the widget;
    /// the app runs the action ([`Collapsible::toggle`]). Pass the id
    /// given to [`Collapsible::sem`] so the node and the binding name the
    /// same control.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Button, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.enabled(!self.disabled).state("expanded", self.open);
        node.action("toggle_expanded")
            .reads(format!("{id}:expanded"));
        node
    }

    /// While open, the revealed region as `<id>.panel`: a `Role::Region`
    /// named by the title, beside the trigger under `parent`. `None`
    /// while closed (a closing region is not there any more).
    #[track_caller]
    pub fn project_panel<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> Option<NodeBuilder<'b>> {
        if !self.open {
            return None;
        }
        let panel = id.child("panel").ok()?;
        let mut node = b.node(&panel, Role::Region, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        Some(node)
    }
}

fn faded(c: Color, alpha: f32) -> [f32; 4] {
    [c.0[0], c.0[1], c.0[2], c.0[3] * alpha]
}

/// An icon turned by `angle` radians about its center, its hash moved
/// with the angle so the compositor's diff sees the turn.
fn turned(name: &str, size: f32, color: [f32; 4], x: f32, y: f32, angle: f32) -> Option<SceneNode> {
    let mut data = icons::icon_path(name, size, color, x, y)?;
    if angle != 0.0 {
        let (cx, cy) = (x + size / 2.0, y + size / 2.0);
        let (sin, cos) = angle.sin_cos();
        for v in &mut data.vertices {
            let (dx, dy) = (v.position[0] - cx, v.position[1] - cy);
            v.position = [cx + dx * cos - dy * sin, cy + dx * sin + dy * cos];
        }
        let mut h = DefaultHasher::new();
        data.hash.hash(&mut h);
        angle.to_bits().hash(&mut h);
        data.hash = h.finish();
    }
    Some(SceneNode::Path { data })
}
