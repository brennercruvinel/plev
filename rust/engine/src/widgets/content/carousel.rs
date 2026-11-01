//! The carousel of the hoff core kit: its Variations tiles in a row that
//! pages, under a "k of N" counter, over its ViewController bar (an arrow
//! each side, a tick per page, the current one taller). The row holds as
//! many square slides as its width allows at the recipe's least edge, so
//! the carousel answers to the space it gets, never to the window: a
//! narrow one shows one slide a page, a wide one many. The visual values
//! are `theme.disclosure.carousel`'s.
//!
//! A slide is chosen as an item is (a click, or `select`); the arrows
//! page back and forth (`previous`, `next`). With a [`SemId`]
//! ([`Carousel::sem`]) the render binds the slides it shows,
//! `<id>.slide[<i>]`, and the arrows, `<id>.previous` and `<id>.next`;
//! [`Carousel::project`] declares the carousel a `Role::Group` with its
//! counter as its description, the slides on the page as options and the
//! arrows as buttons, an arrow with nowhere to go not enabled.

use std::cell::Cell;

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::aspect_ratio::AspectRatio;
use super::tokens::CarouselRecipe;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::icons;
use crate::widgets::recipe::{
    focus_ring_inside, rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets,
};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// What a part of the carousel is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CarouselPart {
    Slide(usize),
    Previous,
    Next,
}

/// Where a carousel's parts go in its bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct CarouselLayout {
    pub counter: Rect,
    /// The slides on the page, `(index, rect)`.
    pub slides: Vec<(usize, Rect)>,
    pub bar: Rect,
    pub previous: Rect,
    pub next: Rect,
    /// A tick per page, the current one taller.
    pub ticks: Vec<Rect>,
}

/// Square slides that page, with a counter and a bar.
#[derive(Clone, Debug, Default)]
pub struct Carousel {
    slides: Vec<(String, &'static str)>,
    /// The first slide on the page.
    first: usize,
    selected: Option<usize>,
    hovered: Option<CarouselPart>,
    pressed: Option<CarouselPart>,
    focused: Option<CarouselPart>,
    /// The slides a page held at the last layout: what an arrow pages by
    /// and the projection holds (the layout follows the width it gets).
    held: Cell<usize>,
    sem: Option<SemId>,
}

impl Carousel {
    /// Slides, each `(label, glyph)`: a label for the reader, a glyph
    /// where the image goes.
    pub fn new<S: Into<String>>(slides: impl IntoIterator<Item = (S, &'static str)>) -> Self {
        Self {
            slides: slides.into_iter().map(|(l, g)| (l.into(), g)).collect(),
            held: Cell::new(1),
            ..Self::default()
        }
    }

    pub fn selected(mut self, i: usize) -> Self {
        if i < self.slides.len() {
            self.selected = Some(i);
        }
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn len(&self) -> usize {
        self.slides.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slides.is_empty()
    }

    pub fn first(&self) -> usize {
        self.first
    }

    pub fn selected_slide(&self) -> Option<usize> {
        self.selected
    }

    /// `<id>.slide[<i>]`, `<id>.previous`, `<id>.next`.
    pub fn part_id(&self, part: CarouselPart) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match part {
            CarouselPart::Slide(i) => Some(id.child("slide").ok()?.key(i.to_string())),
            CarouselPart::Previous => id.child("previous").ok(),
            CarouselPart::Next => id.child("next").ok(),
        }
    }

    /// The part whose id is `id`.
    pub fn part_of(&self, id: &SemId) -> Option<CarouselPart> {
        [CarouselPart::Previous, CarouselPart::Next]
            .into_iter()
            .chain((0..self.slides.len()).map(CarouselPart::Slide))
            .find(|p| self.part_id(*p).as_ref() == Some(id))
    }

    /// How many slides a row `w` wide holds at the recipe's least edge.
    pub fn per_page(&self, w: f32, theme: &Theme) -> usize {
        let r = &theme.disclosure.carousel;
        (((w + r.gap) / (r.slide_min + r.gap)).floor() as usize).clamp(1, self.slides.len().max(1))
    }

    pub fn can_go_back(&self) -> bool {
        self.first > 0
    }

    pub fn can_go_on(&self) -> bool {
        self.first + self.held.get() < self.slides.len()
    }

    /// One page back; `false` at the start.
    pub fn back(&mut self) -> bool {
        if !self.can_go_back() {
            return false;
        }
        self.first = self.first.saturating_sub(self.held.get());
        true
    }

    /// One page on; `false` at the end.
    pub fn forward(&mut self) -> bool {
        if !self.can_go_on() {
            return false;
        }
        self.first = (self.first + self.held.get()).min(self.slides.len() - self.held.get());
        true
    }

    /// Chooses slide `i`; `false` when there is none.
    pub fn select(&mut self, i: usize) -> bool {
        if i >= self.slides.len() {
            return false;
        }
        self.selected = Some(i);
        true
    }

    /// The keyboard focus on a part, or on none.
    pub fn set_focused(&mut self, part: Option<CarouselPart>) {
        self.focused = part;
    }

    /// Keeps the page whole at a new width: a page holds `per_page(w)`
    /// slides, and the first is where a whole page still fits.
    pub fn fit(&mut self, w: f32, theme: &Theme) {
        self.held.set(self.per_page(w, theme));
        let last_first = self.slides.len().saturating_sub(self.held.get());
        self.first = self.first.min(last_first);
    }

    fn counter_style(r: &CarouselRecipe, theme: &Theme) -> TextStyle {
        theme.typography.run(r.counter_type)
    }

    /// "k of N": the chosen slide, or 0 with none.
    pub fn counter(&self) -> String {
        let k = self.selected.map_or(0, |i| i + 1);
        format!("{k} of {}", self.slides.len())
    }

    fn page(&self) -> usize {
        self.first.div_ceil(self.held.get().max(1))
    }

    /// Where everything goes in a carousel `w` wide from `(x, y)`.
    pub fn layout(&self, x: f32, y: f32, w: f32, theme: &Theme) -> CarouselLayout {
        let r = &theme.disclosure.carousel;
        let per = self.per_page(w, theme);
        self.held.set(per);
        let line = Self::counter_style(r, theme).line_height;
        let counter = Rect::new(x, y, w, line);
        let edge = ((w - r.gap * (per as f32 - 1.0)) / per as f32).max(0.0);
        let h = AspectRatio::SQUARE.height(edge);
        let top = y + line + r.counter_gap;
        let first = self.first.min(self.slides.len().saturating_sub(per));
        let slides = (first..(first + per).min(self.slides.len()))
            .enumerate()
            .map(|(col, i)| (i, Rect::new(x + col as f32 * (edge + r.gap), top, edge, h)))
            .collect();
        let pages = self.slides.len().div_ceil(per).max(1);
        let ticks_w = pages as f32 * r.tick_w + (pages as f32 - 1.0) * r.tick_gap;
        let bar_w = (r.bar_pad * 2.0 + r.arrow * 2.0 + r.bar_gap * 2.0 + ticks_w).min(w);
        let bar = Rect::new(x + (w - bar_w) / 2.0, top + h + r.bar_space, bar_w, r.bar_h);
        let mid = bar.y + bar.h / 2.0;
        let previous = Rect::new(bar.x + r.bar_pad, mid - r.arrow / 2.0, r.arrow, r.arrow);
        let next = Rect::new(
            bar.right() - r.bar_pad - r.arrow,
            mid - r.arrow / 2.0,
            r.arrow,
            r.arrow,
        );
        let page = first.div_ceil(per);
        let tick_x = bar.x + (bar.w - ticks_w) / 2.0;
        let ticks = (0..pages)
            .map(|p| {
                let th = if p == page { r.tick_h_on } else { r.tick_h };
                Rect::new(
                    tick_x + p as f32 * (r.tick_w + r.tick_gap),
                    mid - th / 2.0,
                    r.tick_w,
                    th,
                )
            })
            .collect();
        CarouselLayout {
            counter,
            slides,
            bar,
            previous,
            next,
            ticks,
        }
    }

    /// The carousel's height at width `w`.
    pub fn height(&self, w: f32, theme: &Theme) -> f32 {
        let l = self.layout(0.0, 0.0, w, theme);
        l.bar.bottom()
    }

    /// The part under `(x, y)`; an arrow's target is the whole half of
    /// the bar it sits in, a comfortable hit for a finger.
    fn part_at(l: &CarouselLayout, x: f32, y: f32) -> Option<CarouselPart> {
        if let Some((i, _)) = l.slides.iter().find(|(_, r)| r.contains(x, y)) {
            return Some(CarouselPart::Slide(*i));
        }
        if !l.bar.contains(x, y) {
            return None;
        }
        Some(if x < l.bar.center().0 {
            CarouselPart::Previous
        } else {
            CarouselPart::Next
        })
    }

    /// Hover and press; a release on what was pressed chooses the slide
    /// or pages (`clicked`).
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.fit(bounds.w, theme);
        let l = self.layout(bounds.x, bounds.y, bounds.w, theme);
        let (x, y) = event.pos();
        let at = Self::part_at(&l, x, y);
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
                if at != pressed {
                    return EventResult::changed();
                }
                let done = match pressed {
                    Some(CarouselPart::Slide(i)) => self.select(i),
                    Some(CarouselPart::Previous) => self.back(),
                    Some(CarouselPart::Next) => self.forward(),
                    None => false,
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
        let r = &theme.disclosure.carousel;
        let l = self.layout(bounds.x, bounds.y, bounds.w, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        let style = Self::counter_style(r, theme);
        let text = self.counter();
        let (tw, _) = TextMeasurer::measure_styled(&text, &style, None);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&text, &style, None),
            x: l.counter.right() - tw,
            y: l.counter.y,
            color: r.counter.0,
        });
        for (i, at) in &l.slides {
            if let Some(id) = self.part_id(CarouselPart::Slide(*i)) {
                c.bind(&id, [at.x, at.y, at.w, at.h]);
            }
            let radius = r.radius.min(at.w / 2.0);
            c.push(rounded_rect(at.x, at.y, at.w, at.h, radius, r.fill.0));
            let size = at.w * r.glyph;
            if let Some(node) = icons::icon_at(
                self.slides[*i].1,
                size,
                r.glyph_color.0,
                at.x + (at.w - size) / 2.0,
                at.y + (at.h - size) / 2.0,
            ) {
                c.push(node);
            }
            let part = Some(CarouselPart::Slide(*i));
            if self.selected == Some(*i) || self.hovered == part {
                for (width, color) in [r.ring_inner, r.ring_outer] {
                    c.push(rounded_rect_stroke(
                        at.x, at.y, at.w, at.h, radius, color.0, width,
                    ));
                }
            }
            if self.focused == part {
                c.push(focus_ring_inside(*at, radius, theme));
            }
        }
        self.paint_bar(c, &l, r, theme);
    }

    fn paint_bar(&self, c: &mut Compositor, l: &CarouselLayout, r: &CarouselRecipe, theme: &Theme) {
        let radius = l.bar.h / 2.0;
        shadow_drops(c, LayerId::DEFAULT, l.bar, radius, &r.bar_shadow);
        c.push(rounded_rect(
            l.bar.x,
            l.bar.y,
            l.bar.w,
            l.bar.h,
            radius,
            r.bar_fill.0,
        ));
        shadow_insets(c, LayerId::DEFAULT, l.bar, radius, &r.bar_shadow);
        c.push(rounded_rect_stroke(
            l.bar.x,
            l.bar.y,
            l.bar.w,
            l.bar.h,
            radius,
            r.bar_border.0,
            1.0_f32.min(radius),
        ));
        let half = |left: bool| {
            let w = l.bar.w / 2.0;
            Rect::new(
                if left { l.bar.x } else { l.bar.x + w },
                l.bar.y,
                w,
                l.bar.h,
            )
        };
        for (part, at, glyph, on) in [
            (
                CarouselPart::Previous,
                l.previous,
                "chevron-left",
                self.can_go_back(),
            ),
            (
                CarouselPart::Next,
                l.next,
                "chevron-right",
                self.can_go_on(),
            ),
        ] {
            if let Some(id) = self.part_id(part) {
                let hit = half(part == CarouselPart::Previous);
                c.bind(&id, [hit.x, hit.y, hit.w, hit.h]);
            }
            let color: Color = if on { r.arrow_on } else { r.arrow_off };
            if let Some(node) = icons::icon_at(glyph, r.arrow, color.0, at.x, at.y) {
                c.push(node);
            }
            if self.focused == Some(part) {
                c.push(focus_ring_inside(
                    half(part == CarouselPart::Previous),
                    radius,
                    theme,
                ));
            }
        }
        let page = self.page();
        for (p, at) in l.ticks.iter().enumerate() {
            let color = if p == page { r.tick_on } else { r.tick_off };
            c.push(rounded_rect(at.x, at.y, at.w, at.h, at.w / 2.0, color.0));
        }
    }

    /// Declares the carousel as `id`, a `Role::Group` named `label`
    /// described by its counter; in it the arrows (`previous`, `next`,
    /// each an `activate`, not enabled with nowhere to go) and the slides
    /// on the page (options with `selected` and a `select` that reads it).
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for (part, name, on) in [
            (CarouselPart::Previous, "previous", self.can_go_back()),
            (CarouselPart::Next, "next", self.can_go_on()),
        ] {
            if let Some(at) = self.part_id(part) {
                let mut node = b.node(&at, Role::Button, name);
                node.parent(id).enabled(on);
                node.action("activate");
            }
        }
        let end = (self.first + self.held.get()).min(self.slides.len());
        for i in self.first..end {
            if let Some(at) = self.part_id(CarouselPart::Slide(i)) {
                let mut node = b.node(&at, Role::Option, self.slides[i].0.as_str());
                node.parent(id).state("selected", self.selected == Some(i));
                node.action("select").reads(format!("{at}:selected"));
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.counter());
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slides(n: usize) -> Carousel {
        Carousel::new((0..n).map(|i| (format!("Material {i}"), "circle")))
            .sem(SemId::parse("variations").unwrap())
    }

    /// A page holds as many slides as the width allows at the least edge,
    /// and the slides are square.
    #[test]
    fn the_page_answers_to_the_width() {
        let theme = Theme::hoff_core();
        let r = &theme.disclosure.carousel;
        let c = slides(8);
        assert_eq!(c.per_page(r.slide_min, &theme), 1);
        assert_eq!(c.per_page(r.slide_min * 2.0 + r.gap, &theme), 2);
        assert_eq!(
            c.per_page(10_000.0, &theme),
            8,
            "never more than the slides"
        );
        let l = c.layout(0.0, 0.0, 400.0, &theme);
        let (_, s) = l.slides[0];
        assert_eq!(s.w, s.h);
        assert!(l.slides.last().unwrap().1.right() <= 400.0 + 0.01);
        assert_eq!(l.ticks.len(), 8_usize.div_ceil(c.per_page(400.0, &theme)));
    }

    /// The arrows page by a page and stop at the ends; a slide's click
    /// chooses it and the counter says so.
    #[test]
    fn the_arrows_page_and_a_click_chooses() {
        let theme = Theme::hoff_core();
        let mut c = slides(5);
        c.fit(200.0, &theme);
        let per = c.per_page(200.0, &theme);
        assert!(!c.back(), "at the start");
        assert!(c.forward());
        assert_eq!(c.first(), per.min(5 - per));
        while c.forward() {}
        assert!(!c.can_go_on() && c.can_go_back());
        let bounds = Rect::new(0.0, 0.0, 200.0, c.height(200.0, &theme));
        let l = c.layout(0.0, 0.0, 200.0, &theme);
        let (i, at) = l.slides[0];
        let (x, y) = at.center();
        c.handle_event(&WidgetEvent::MouseDown { x, y }, bounds, &theme);
        assert!(
            c.handle_event(&WidgetEvent::MouseUp { x, y }, bounds, &theme)
                .clicked
        );
        assert_eq!(c.selected_slide(), Some(i));
        assert_eq!(c.counter(), format!("{} of 5", i + 1));
    }

    /// The projection holds the page's slides and the arrows, the one at
    /// its end not enabled.
    #[test]
    fn it_projects_the_page_and_the_arrows() {
        let theme = Theme::hoff_core();
        let mut c = slides(6);
        c.fit(200.0, &theme);
        let id = SemId::parse("variations").unwrap();
        let mut b = SnapshotBuilder::new();
        c.project(&mut b, &id, "Variations", None);
        let seen = b.finish().unwrap();
        let back = c.part_id(CarouselPart::Previous).unwrap();
        assert!(!seen.get(&back).unwrap().enabled);
        let on = c.part_id(CarouselPart::Next).unwrap();
        assert!(seen.get(&on).unwrap().enabled);
        let per = c.per_page(200.0, &theme);
        let shown = c.part_id(CarouselPart::Slide(per - 1)).unwrap();
        assert!(seen.get(&shown).is_some());
        let hidden = c.part_id(CarouselPart::Slide(per)).unwrap();
        assert!(seen.get(&hidden).is_none());
        assert_eq!(c.part_of(&on), Some(CarouselPart::Next));
    }
}
