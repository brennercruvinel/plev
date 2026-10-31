//! Carousel page: the kit's Variations tiles that page under a "k of N"
//! counter, over its ViewController bar, in the showcase mold. Each card
//! holds a carousel as wide as the card: a narrow card shows one slide a
//! page, a wide one many. The widget is `engine::widgets::content::
//! Carousel`; every visual value comes from the theme
//! (`theme.disclosure.carousel`).
//!
//! Projected: a group per title, `carousel_page.group[<g>]`; in it each
//! card's carousel, `...card[<c>]`, a group named by its chip and
//! described by its counter; in that the arrows (`...previous`,
//! `...next`), whose `activate` pages as a click does, and the slides on
//! the page (`...slide[<i>]`), options whose `select` chooses one.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Carousel, CarouselPart};
use engine::widgets::{EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase, under};

const GLYPHS: [&str; 6] = ["circle", "square", "layers", "sun", "moon", "heart"];

struct Card {
    /// The chip: which states this card shows.
    title: &'static str,
    carousel: Carousel,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct CarouselPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The part, `(group, card, part)`, a reader or the keyboard focused.
    focus: Option<(usize, usize, CarouselPart)>,
}

/// `carousel_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `carousel_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

/// `n` materials, each a label and a glyph.
fn slides(n: usize) -> Carousel {
    Carousel::new((0..n).map(|i| (format!("Material {}", i + 1), GLYPHS[i % GLYPHS.len()])))
}

impl CarouselPage {
    pub fn new() -> Self {
        let card = |title, carousel| Card { title, carousel };
        let mut groups = vec![
            Group {
                title: "Carousel",
                columns: 2,
                cards: vec![
                    card("Default, Hover, Focus", slides(8)),
                    card("Chosen", slides(8).selected(1)),
                ],
            },
            Group {
                title: "Carousel, ranges",
                columns: 3,
                cards: vec![
                    card("Two slides", slides(2)),
                    card("Many slides", slides(24).selected(0)),
                    card("One slide", slides(1)),
                ],
            },
        ];
        let root = root_id("carousel_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.carousel = std::mem::take(&mut card.carousel).sem(id);
                    }
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// The part behind `id`: `(group, card, part)`.
    fn at(&self, id: &SemId) -> Option<(usize, usize, CarouselPart)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| card.carousel.part_of(id).map(|p| (g, c, p)))
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, CarouselPart)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let part = at
                    .filter(|(fg, fc, _)| (*fg, *fc) == (g, c))
                    .map(|(_, _, p)| p);
                card.carousel.set_focused(part);
            }
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, w| {
            self.groups[g].cards[c].carousel.height(w, theme)
        })
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (group, placed) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, body)) in group.cards.iter_mut().zip(&placed.cards) {
                result = result.merge(card.carousel.handle_event(event, *body, theme));
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, placed)) in self
            .groups
            .iter()
            .zip(self.layout(content, theme))
            .enumerate()
        {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                card.carousel.render(c, *body, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for CarouselPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.carousel.sem_id() {
                    card.carousel.project(b, id, card.title, Some(&at));
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, p)| self.groups.get(g)?.cards.get(c)?.carousel.part_id(p));
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// An arrow's `activate` pages, a slide's `select` chooses it, as
    /// their clicks do; an arrow with nowhere to go refuses.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let done = match (self.at(&req.target), req.action.as_str()) {
            (Some((g, c, CarouselPart::Previous)), "activate") => {
                self.groups[g].cards[c].carousel.back()
            }
            (Some((g, c, CarouselPart::Next)), "activate") => {
                self.groups[g].cards[c].carousel.forward()
            }
            (Some((g, c, CarouselPart::Slide(i))), "select") => {
                self.groups[g].cards[c].carousel.select(i)
            }
            _ => {
                return Err(format!(
                    "the carousel page has no {} on {}",
                    req.action, req.target
                ));
            }
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(format!("{} has nowhere to go", req.target))
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(self.at(id));
        true
    }

    fn blur(&mut self) -> bool {
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// carousel in its card, its slides and its bar in it, cards apart.
    #[test]
    fn nothing_is_cut_or_overlaps_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = CarouselPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (k, (card, (rect, body))) in
                        group.cards.iter().zip(&placed.cards).enumerate()
                    {
                        let what = format!("{w}px, {}", card.title);
                        let inside = |r: Rect| {
                            r.x >= rect.x - 0.01
                                && r.right() <= rect.right() + 0.01
                                && r.bottom() <= rect.bottom() + 0.01
                        };
                        let l = card.carousel.layout(body.x, body.y, body.w, &theme);
                        assert!(inside(l.bar), "{what}: bar");
                        for (_, s) in &l.slides {
                            assert!(inside(*s), "{what}: slide");
                        }
                        for (other, _) in &placed.cards[k + 1..] {
                            let apart = rect.right() <= other.x || rect.bottom() <= other.y;
                            assert!(apart, "{what}: cards overlap");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn requests_page_and_choose_and_refuse_at_the_ends() {
        let theme = Theme::hoff_core();
        let mut page = CarouselPage::new();
        let mut c = Compositor::new();
        c.begin_frame();
        page.render(&mut c, Rect::new(0.0, 0.0, 1200.0, 800.0), &theme);
        let card = "carousel_page.group[0].card[0]";
        let id = |s: &str| SemId::parse(s).unwrap();
        let ask = |page: &mut CarouselPage, at: &str, action: &str| {
            let req = Request {
                actor: accord::Actor::Test {
                    name: "carousel".into(),
                },
                target: id(at),
                action: accord::ActionName::parse(action).unwrap(),
                param: None,
                observed: accord::Observation {
                    revision: accord::Snapshot::default().revision(),
                    values: Vec::new(),
                },
                ask: None,
            };
            page.run(&req)
        };
        assert!(ask(&mut page, &format!("{card}.previous"), "activate").is_err());
        assert_eq!(
            ask(&mut page, &format!("{card}.next"), "activate"),
            Ok(After::Nothing)
        );
        assert!(page.groups[0].cards[0].carousel.first() > 0);
        assert_eq!(
            ask(&mut page, &format!("{card}.slide[3]"), "select"),
            Ok(After::Nothing)
        );
        assert_eq!(page.groups[0].cards[0].carousel.selected_slide(), Some(3));
        assert!(ask(&mut page, card, "select").is_err());
    }
}
