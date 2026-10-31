//! The accordion of the hoff core kit: sections stacked one under the
//! other, each a [`Collapsible`] in the Accordion look (a divider above
//! all but the first, a bare header, the chevron in a box) revealing a
//! text body. In [`AccordionMode::Single`] opening a section closes the
//! open one; in [`AccordionMode::Multiple`] each opens on its own, as the
//! kit's sections do.
//!
//! The height is intrinsic: each body is measured wrapped at the width
//! the accordion gets, and each section is as tall as its trigger plus
//! the revealed part of its body.
//!
//! With a [`SemId`] ([`Accordion::sem`]) each trigger is `<id>.item[<i>]`
//! and binds where it draws; [`Accordion::project`] declares the
//! accordion a `Role::Group` with the triggers and, for the open
//! sections, their regions (`<id>.item[<i>].panel`, described by the
//! body).

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::collapsible::{Collapsible, DisclosureLook};
use super::tokens::DisclosureMotion;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Whether opening a section closes the others.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionMode {
    /// One section open at most.
    Single,
    /// Each section on its own (the kit).
    #[default]
    Multiple,
}

/// Sections that open and close, one under the other.
#[derive(Clone, Debug, Default)]
pub struct Accordion {
    mode: AccordionMode,
    items: Vec<Collapsible>,
    bodies: Vec<String>,
    motion: DisclosureMotion,
    sem: Option<SemId>,
}

impl Accordion {
    pub fn new(mode: AccordionMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    /// A closed section titled `title`, revealing `body`.
    pub fn item(mut self, title: impl Into<String>, body: impl Into<String>) -> Self {
        let first = self.items.is_empty();
        let mut item = Collapsible::new(title)
            .look(DisclosureLook::Accordion)
            .first(first)
            .with_motion(&self.motion);
        if let Some(id) = self.item_sem(self.items.len()) {
            item = item.sem(id);
        }
        self.items.push(item);
        self.bodies.push(body.into());
        self
    }

    /// Section `i` open from the start, at rest; in single mode the
    /// others close.
    pub fn opened(mut self, i: usize) -> Self {
        if i >= self.items.len() {
            return self;
        }
        let single = self.mode == AccordionMode::Single;
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .enumerate()
            .map(|(k, item)| match (k == i, single) {
                (true, _) => item.open(true),
                (false, true) => item.open(false),
                (false, false) => item,
            })
            .collect();
        self
    }

    /// Section `i` disabled: it stays as it is and reacts to nothing.
    pub fn disabled_item(mut self, i: usize) -> Self {
        if let Some(item) = self.items.get_mut(i) {
            *item = std::mem::replace(item, Collapsible::new("")).disabled(true);
        }
        self
    }

    /// The motion of a theme (`theme.disclosure.motion`), for the
    /// sections there are and the ones to come.
    pub fn with_motion(mut self, motion: &DisclosureMotion) -> Self {
        self.motion = *motion;
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .map(|item| item.with_motion(motion))
            .collect();
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        let ids: Vec<_> = (0..self.items.len()).map(|i| self.item_sem(i)).collect();
        self.items = std::mem::take(&mut self.items)
            .into_iter()
            .zip(ids)
            .map(|(item, id)| match id {
                Some(id) => item.sem(id),
                None => item,
            })
            .collect();
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// `<id>.item[<i>]`.
    fn item_sem(&self, i: usize) -> Option<SemId> {
        let family = self.sem.as_ref()?.child("item").ok()?;
        Some(family.key(i.to_string()))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn items(&self) -> &[Collapsible] {
        &self.items
    }

    /// The section whose trigger is `id`.
    pub fn position(&self, id: &SemId) -> Option<usize> {
        self.items.iter().position(|item| item.sem_id() == Some(id))
    }

    pub fn is_open(&self, i: usize) -> bool {
        self.items.get(i).is_some_and(Collapsible::is_open)
    }

    pub fn is_disabled(&self, i: usize) -> bool {
        self.items.get(i).is_some_and(Collapsible::is_disabled)
    }

    /// Flips section `i`; in single mode, opening it closes the others.
    /// `false` when it is disabled or not there.
    pub fn toggle(&mut self, i: usize) -> bool {
        let Some(item) = self.items.get_mut(i) else {
            return false;
        };
        if !item.toggle() {
            return false;
        }
        self.close_others(i);
        true
    }

    /// In single mode, the sections other than an open `i` close.
    fn close_others(&mut self, i: usize) {
        if self.mode != AccordionMode::Single || !self.is_open(i) {
            return;
        }
        for (k, other) in self.items.iter_mut().enumerate() {
            if k != i {
                other.set_open(false);
            }
        }
    }

    /// The keyboard focus on the trigger of section `i`, or on none.
    pub fn set_focused(&mut self, i: Option<usize>) {
        for (k, item) in self.items.iter_mut().enumerate() {
            item.set_focused(i == Some(k));
        }
    }

    /// Every section's motion, merged.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.items
            .iter_mut()
            .fold(Wake::idle(), |w, item| w.merge(item.wake(dt)))
    }

    fn body_style(theme: &Theme) -> TextStyle {
        let r = &theme.disclosure.accordion;
        theme
            .typography
            .exact(r.body_size, r.body_line_height, r.body_weight)
    }

    /// The width a body wraps at, in a section `w` wide.
    fn body_width(w: f32, theme: &Theme) -> f32 {
        let [_, right, _, left] = theme.disclosure.accordion.content_pad;
        (w - left - right).max(0.0)
    }

    /// Body `i` measured, wrapped in a section `w` wide.
    fn body_height(&self, i: usize, w: f32, theme: &Theme) -> f32 {
        match self.bodies.get(i) {
            Some(body) if !body.is_empty() => {
                let style = Self::body_style(theme);
                TextMeasurer::measure_styled(body, &style, Some(Self::body_width(w, theme))).1
            }
            _ => 0.0,
        }
    }

    /// The height now at width `w`.
    pub fn height(&self, w: f32, theme: &Theme) -> f32 {
        (0..self.items.len())
            .map(|i| self.items[i].height(w, self.body_height(i, w, theme), theme))
            .sum()
    }

    /// Each section's rect in `bounds`, top to bottom.
    pub fn item_rects(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let mut y = bounds.y;
        (0..self.items.len())
            .map(|i| {
                let h = self.items[i].height(bounds.w, self.body_height(i, bounds.w, theme), theme);
                let r = Rect::new(bounds.x, y, bounds.w, h);
                y += h;
                r
            })
            .collect()
    }

    /// Pointer events to every section; a section a click opened closes
    /// the others in single mode.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects = self.item_rects(bounds, theme);
        let mut result = EventResult::IGNORED;
        let mut flipped = None;
        for (i, (item, rect)) in self.items.iter_mut().zip(rects).enumerate() {
            let r = item.handle_event(event, rect, theme);
            if r.clicked {
                flipped = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = flipped {
            self.close_others(i);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let style = Self::body_style(theme);
        let color = theme.disclosure.accordion.body;
        for (i, (item, rect)) in self
            .items
            .iter()
            .zip(self.item_rects(bounds, theme))
            .enumerate()
        {
            let body = self.bodies[i].as_str();
            let alpha = if item.is_disabled() {
                theme.glass.disabled_alpha
            } else {
                1.0
            };
            let body_h = self.body_height(i, bounds.w, theme);
            item.render(c, rect, body_h, theme, |c, at| {
                if body.is_empty() {
                    return;
                }
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(body, &style, Some(at.w)),
                    x: at.x,
                    y: at.y,
                    color: [color.0[0], color.0[1], color.0[2], color.0[3] * alpha],
                });
            });
        }
    }

    /// Declares the accordion as `id`, a `Role::Group` named `label`, and
    /// in it each section's trigger as [`Collapsible::project`] does and,
    /// for an open one, its region described by its body. It only reads
    /// the widget; the app runs `toggle_expanded` ([`Accordion::toggle`]).
    /// Returns the group's node for the app to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for (item, body) in self.items.iter().zip(&self.bodies) {
            let Some(at) = item.sem_id() else {
                continue;
            };
            item.project(b, at, Some(id));
            if let Some(mut panel) = item.project_panel(b, at, Some(id))
                && !body.is_empty()
            {
                panel.description(body.as_str());
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }
}
