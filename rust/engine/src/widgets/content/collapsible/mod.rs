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

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::{DisclosureMotion, DisclosureRecipe};
use crate::compose::animation::{Tween, reduced_motion};

mod links;
mod paint;
#[cfg(test)]
mod tests;

use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};
pub use links::DisclosureList;

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
    /// A count beside the label, for a look that draws one.
    counter: Option<String>,
    look: DisclosureLook,
    open: bool,
    disabled: bool,
    current: bool,
    /// The first section of a stack draws no divider above it.
    first: bool,
    /// The label stays on one line and ends in an ellipsis when it does
    /// not fit (a sidebar's row), instead of wrapping; the projected name
    /// is the whole title either way.
    truncate: bool,
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
            counter: None,
            look: DisclosureLook::default(),
            open: false,
            disabled: false,
            current: false,
            first: false,
            truncate: false,
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

    /// A count beside the label (the Dropdown's `counter`), in a look
    /// that draws one.
    pub fn counter(mut self, text: impl Into<String>) -> Self {
        self.counter = Some(text.into());
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

    /// One line, cut with an ellipsis where it does not fit (CSS
    /// `truncate`), rather than wrapped: a row of a narrow sidebar, whose
    /// label must never break inside a word. The whole title is still
    /// what [`Collapsible::project`] names.
    pub fn truncate(mut self, on: bool) -> Self {
        self.truncate = on;
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
        theme.typography.run(r.text_type)
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

    fn counter_style(r: &DisclosureRecipe, theme: &Theme) -> Option<TextStyle> {
        let counter = r.counter.as_ref()?;
        Some(theme.typography.run(counter.text_type))
    }

    /// The counter's pill, `(w, h)`, when there is one to draw.
    fn counter_span(&self, r: &DisclosureRecipe, theme: &Theme) -> Option<(f32, f32)> {
        let text = self.counter.as_deref()?;
        let counter = r.counter.as_ref()?;
        let style = Self::counter_style(r, theme)?;
        let (w, _) = TextMeasurer::measure_styled(text, &style, None);
        let [pad_x, pad_y] = counter.pad;
        Some((w + pad_x * 2.0, style.line_height + pad_y * 2.0))
    }

    /// The label wraps (or, truncated, is cut) in what the tile, the
    /// counter and the chevron leave; a counter stands as far from the label as the chevron from
    /// the counter (the kit's `ml-3`, Dropdown :59), so a long label
    /// never runs into it.
    fn inner(&self, trigger: Rect, theme: &Theme) -> Inner {
        let r = self.recipe(theme);
        let [top, right, _, left] = r.pad;
        let bw = r.border_width;
        let x = trigger.x + bw + left;
        let w = (trigger.w - bw * 2.0 - left - right).max(0.0);
        let tile = self.tile_span(r);
        let counter = self.counter_span(r, theme);
        let label_w = (w
            - tile.map_or(0.0, |(size, gap)| size + gap)
            - counter.map_or(0.0, |(cw, _)| cw + r.chevron_gap)
            - r.chevron_gap
            - r.chevron_box)
            .max(0.0);
        let style = Self::label_style(r, theme);
        let label_h = if self.truncate {
            style.line_height
        } else {
            let (_, text_h) = TextMeasurer::measure_styled(&self.title, &style, Some(label_w));
            text_h.max(style.line_height)
        };
        let h = label_h
            .max(r.chevron_box)
            .max(tile.map_or(0.0, |(size, _)| size))
            .max(counter.map_or(0.0, |(_, ch)| ch));
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
