//! Pagination: page numbers in cells between two arrows, derived from the
//! hoff core kit's language (it has the "k of N" counter of its
//! Variations, not a pager): the Dropdown's cell, a wash on hover, its
//! current look on the current page. The visual values are
//! `theme.disclosure.pager`'s.
//!
//! The pager answers to the width it gets: as many cells as fit, the first
//! and the last page always among them, the pages around the current one
//! in the middle and a gap where pages are left out, down to the arrows
//! and the current page alone.
//!
//! With a [`SemId`] ([`Pagination::sem`]) the render binds each cell,
//! `<id>.page[<k>]`, and the arrows, `<id>.previous` and `<id>.next`;
//! [`Pagination::project`] declares a `Role::Group` described by "Page k
//! of N", the arrows and the pages it shows as buttons with an
//! `activate` (a page also says whether it is `current`).

use std::cell::RefCell;

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::PagerRecipe;
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{focus_ring_inside, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// A cell of the pager.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageSlot {
    Previous,
    /// Page `k`, from 0.
    Page(usize),
    /// Pages left out.
    Gap,
    Next,
}

/// Page numbers between two arrows.
#[derive(Clone, Debug, Default)]
pub struct Pagination {
    pages: usize,
    current: usize,
    hovered: Option<PageSlot>,
    pressed: Option<PageSlot>,
    focused: Option<PageSlot>,
    /// The cells of the last layout: what the projection shows.
    shown: RefCell<Vec<PageSlot>>,
    sem: Option<SemId>,
}

impl Pagination {
    /// `pages` pages (at least one), on the first.
    pub fn new(pages: usize) -> Self {
        Self {
            pages: pages.max(1),
            ..Self::default()
        }
    }

    /// On page `k` (from 0), clamped to the last.
    pub fn current(mut self, k: usize) -> Self {
        self.current = k.min(self.pages - 1);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn pages(&self) -> usize {
        self.pages
    }

    pub fn current_page(&self) -> usize {
        self.current
    }

    /// Goes to page `k`; `false` when there is none or it is already
    /// there.
    pub fn go(&mut self, k: usize) -> bool {
        if k >= self.pages || k == self.current {
            return false;
        }
        self.current = k;
        true
    }

    pub fn back(&mut self) -> bool {
        self.current > 0 && self.go(self.current - 1)
    }

    pub fn forward(&mut self) -> bool {
        self.go(self.current + 1)
    }

    pub fn set_focused(&mut self, slot: Option<PageSlot>) {
        self.focused = slot;
    }

    /// `<id>.page[<k>]`, `<id>.previous`, `<id>.next`; a gap has none.
    pub fn slot_id(&self, slot: PageSlot) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match slot {
            PageSlot::Page(k) => Some(id.child("page").ok()?.key(k.to_string())),
            PageSlot::Previous => id.child("previous").ok(),
            PageSlot::Next => id.child("next").ok(),
            PageSlot::Gap => None,
        }
    }

    /// The slot whose id is `id` (any page, shown or not).
    pub fn slot_of(&self, id: &SemId) -> Option<PageSlot> {
        [PageSlot::Previous, PageSlot::Next]
            .into_iter()
            .chain((0..self.pages).map(PageSlot::Page))
            .find(|s| self.slot_id(*s).as_ref() == Some(id))
    }

    fn style(r: &PagerRecipe, theme: &Theme) -> TextStyle {
        theme
            .typography
            .exact(r.font_size, r.line_height, r.font_weight)
    }

    /// The cells a pager `w` wide shows, the arrows at the ends.
    pub fn slots(&self, w: f32, theme: &Theme) -> Vec<PageSlot> {
        let r = &theme.disclosure.pager;
        let fit = ((w + r.gap) / (r.cell + r.gap)).floor() as usize;
        let room = fit.saturating_sub(2).max(1);
        let mut out = vec![PageSlot::Previous];
        out.extend(window(self.pages, self.current, room));
        out.push(PageSlot::Next);
        out
    }

    /// Each cell's rect in a pager `w` wide from `(x, y)`, centered.
    pub fn layout(&self, x: f32, y: f32, w: f32, theme: &Theme) -> Vec<(PageSlot, Rect)> {
        let r = &theme.disclosure.pager;
        let slots = self.slots(w, theme);
        self.shown.replace(slots.clone());
        let total = slots.len() as f32 * r.cell + (slots.len() as f32 - 1.0) * r.gap;
        let x0 = x + ((w - total) / 2.0).max(0.0);
        slots
            .into_iter()
            .enumerate()
            .map(|(i, s)| {
                (
                    s,
                    Rect::new(x0 + i as f32 * (r.cell + r.gap), y, r.cell, r.cell),
                )
            })
            .collect()
    }

    pub fn height(&self, theme: &Theme) -> f32 {
        theme.disclosure.pager.cell
    }

    fn enabled(&self, slot: PageSlot) -> bool {
        match slot {
            PageSlot::Previous => self.current > 0,
            PageSlot::Next => self.current + 1 < self.pages,
            PageSlot::Page(_) => true,
            PageSlot::Gap => false,
        }
    }

    /// Hover and press; a release on the cell pressed goes there
    /// (`clicked`).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let cells = self.layout(bounds.x, bounds.y, bounds.w, theme);
        let (x, y) = event.pos();
        let at = cells
            .iter()
            .find(|(s, r)| r.contains(x, y) && self.enabled(*s))
            .map(|(s, _)| *s);
        match *event {
            WidgetEvent::MouseMove { .. } if at != self.hovered => {
                self.hovered = at;
                EventResult::changed()
            }
            WidgetEvent::MouseDown { .. } if at.is_some() => {
                self.pressed = at;
                EventResult::changed()
            }
            WidgetEvent::MouseUp { .. } if self.pressed.is_some() => {
                let pressed = self.pressed.take();
                let done = at == pressed
                    && match pressed {
                        Some(PageSlot::Page(k)) => self.go(k),
                        Some(PageSlot::Previous) => self.back(),
                        Some(PageSlot::Next) => self.forward(),
                        _ => false,
                    };
                if done {
                    EventResult::clicked()
                } else {
                    EventResult::changed()
                }
            }
            _ => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let r = &theme.disclosure.pager;
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let style = Self::style(r, theme);
        for (slot, at) in self.layout(bounds.x, bounds.y, bounds.w, theme) {
            if let Some(id) = self.slot_id(slot) {
                c.bind(&id, [at.x, at.y, at.w, at.h]);
            }
            let surface = if slot == PageSlot::Page(self.current) {
                &r.current
            } else if self.hovered == Some(slot) {
                &r.hover
            } else {
                &r.rest
            };
            let radius = r.radius.min(at.h / 2.0);
            if surface.fill.0[3] > 0.0 {
                c.push(rounded_rect(at.x, at.y, at.w, at.h, radius, surface.fill.0));
            }
            if surface.border.0[3] > 0.0 {
                c.push(rounded_rect_stroke(
                    at.x,
                    at.y,
                    at.w,
                    at.h,
                    radius,
                    surface.border.0,
                    r.border,
                ));
            }
            let lit = if self.enabled(slot) {
                r.label
            } else {
                r.label_dim
            };
            match slot {
                PageSlot::Previous | PageSlot::Next => {
                    let glyph = if slot == PageSlot::Previous {
                        "chevron-left"
                    } else {
                        "chevron-right"
                    };
                    let inset = (r.cell - r.arrow) / 2.0;
                    if let Some(node) =
                        icons::icon_at(glyph, r.arrow, lit.0, at.x + inset, at.y + inset)
                    {
                        c.push(node);
                    }
                }
                PageSlot::Page(_) | PageSlot::Gap => {
                    let text = match slot {
                        PageSlot::Page(k) => (k + 1).to_string(),
                        _ => "…".to_string(),
                    };
                    let color = match slot {
                        PageSlot::Page(k) if k == self.current => r.label,
                        _ => r.label_dim,
                    };
                    let (tw, _) = TextMeasurer::measure_styled(&text, &style, None);
                    c.push(SceneNode::Text {
                        key: TextNodeKey::from_style(&text, &style, None),
                        x: at.x + (at.w - tw) / 2.0,
                        y: at.y + (at.h - style.line_height) / 2.0,
                        color: color.0,
                    });
                }
            }
            if self.focused == Some(slot) {
                c.push(focus_ring_inside(at, radius, theme));
            }
        }
    }

    /// "Page k of N".
    pub fn counter(&self) -> String {
        format!("Page {} of {}", self.current + 1, self.pages)
    }

    /// Declares the pager as `id`, a `Role::Group` named `label` described
    /// by its counter; the arrows and the pages it showed at its last
    /// layout,
    /// buttons with an `activate` (a page with `current`; an arrow with
    /// nowhere to go not enabled).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let shown = self.shown.borrow().clone();
        let shown = if shown.is_empty() {
            vec![
                PageSlot::Previous,
                PageSlot::Page(self.current),
                PageSlot::Next,
            ]
        } else {
            shown
        };
        for slot in shown {
            let Some(at) = self.slot_id(slot) else {
                continue;
            };
            let name = match slot {
                PageSlot::Previous => "previous page".to_string(),
                PageSlot::Next => "next page".to_string(),
                PageSlot::Page(k) => format!("page {}", k + 1),
                PageSlot::Gap => continue,
            };
            let mut node = b.node(&at, Role::Button, name);
            node.parent(id).enabled(self.enabled(slot));
            if let PageSlot::Page(k) = slot {
                node.state("current", k == self.current);
            }
            node.action("activate");
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.counter());
        node
    }
}

/// The pages and gaps of `room` cells over `n` pages around `current`:
/// all when they fit; the pages around the current one when too few cells
/// are left for the ends; otherwise the first, the last, the pages around
/// the current one and a gap on a side that leaves pages out.
fn window(n: usize, current: usize, room: usize) -> Vec<PageSlot> {
    if n <= room {
        return (0..n).map(PageSlot::Page).collect();
    }
    if room < 5 {
        let start = current.saturating_sub(room / 2).min(n - room);
        return (start..start + room).map(PageSlot::Page).collect();
    }
    let middle = room - 4;
    let start = current
        .saturating_sub(middle / 2)
        .clamp(2, n.saturating_sub(2 + middle).max(2));
    let end = (start + middle).min(n - 2);
    let mut out = vec![PageSlot::Page(0)];
    out.push(if start == 2 {
        PageSlot::Page(1)
    } else {
        PageSlot::Gap
    });
    out.extend((start..end).map(PageSlot::Page));
    out.push(if end == n - 2 {
        PageSlot::Page(n - 2)
    } else {
        PageSlot::Gap
    });
    out.push(PageSlot::Page(n - 1));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pages(out: &[PageSlot]) -> Vec<Option<usize>> {
        out.iter()
            .map(|s| match s {
                PageSlot::Page(k) => Some(*k),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_window_keeps_the_ends_and_the_current_page() {
        assert_eq!(
            pages(&window(4, 0, 9)),
            [Some(0), Some(1), Some(2), Some(3)]
        );
        let mid = window(20, 10, 7);
        assert_eq!(mid.len(), 7);
        assert_eq!(
            pages(&mid),
            [Some(0), None, Some(9), Some(10), Some(11), None, Some(19)]
        );
        let start = window(20, 0, 7);
        assert_eq!(start.first(), Some(&PageSlot::Page(0)));
        assert!(start.contains(&PageSlot::Page(1)) && start.len() <= 7);
        let end = window(20, 19, 7);
        assert!(end.contains(&PageSlot::Page(19)) && end.contains(&PageSlot::Page(18)));
        assert!(end.len() <= 7);
        assert_eq!(
            pages(&window(20, 10, 1)),
            [Some(10)],
            "only the current one"
        );
        for n in 1..30 {
            for c in 0..n {
                for room in 1..12 {
                    let w = window(n, c, room);
                    assert!(w.len() <= room.max(1).max(n.min(room)), "{n} {c} {room}");
                    assert!(
                        w.contains(&PageSlot::Page(c)),
                        "{n} {c} {room}: the current page"
                    );
                }
            }
        }
    }

    #[test]
    fn the_pager_answers_to_its_width_and_its_arrows_stop_at_the_ends() {
        let theme = Theme::hoff_core();
        let p = Pagination::new(20).current(10);
        let narrow = p.slots(120.0, &theme);
        let wide = p.slots(900.0, &theme);
        assert!(narrow.len() < wide.len());
        assert_eq!(narrow.first(), Some(&PageSlot::Previous));
        assert_eq!(narrow.last(), Some(&PageSlot::Next));
        let mut p = Pagination::new(3);
        assert!(!p.back());
        assert!(p.forward() && p.forward() && !p.forward());
        assert_eq!(p.counter(), "Page 3 of 3");
        assert!(!p.go(7));
    }

    #[test]
    fn a_click_on_a_page_goes_there_and_it_projects_as_buttons() {
        let theme = Theme::hoff_core();
        let mut p = Pagination::new(5).sem(SemId::parse("pages").unwrap());
        let bounds = Rect::new(0.0, 0.0, 400.0, p.height(&theme));
        let cells = p.layout(0.0, 0.0, 400.0, &theme);
        let (_, at) = cells.iter().find(|(s, _)| *s == PageSlot::Page(3)).unwrap();
        let (x, y) = at.center();
        p.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
        assert!(
            p.handle_event(&WidgetEvent::MouseUp { x, y }, bounds, &theme)
                .clicked
        );
        assert_eq!(p.current_page(), 3);
        let id = SemId::parse("pages").unwrap();
        let mut b = SnapshotBuilder::new();
        p.project(&mut b, &id, "Pages", None);
        let seen = b.finish().unwrap();
        let page = p.slot_id(PageSlot::Page(3)).unwrap();
        let node = seen.get(&page).unwrap();
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.state.get("current"), Some(&accord::Value::Bool(true)));
        assert_eq!(p.slot_of(&page), Some(PageSlot::Page(3)));
        assert_eq!(
            seen.get(&id).unwrap().description.as_deref(),
            Some("Page 4 of 5")
        );
    }
}
