//! Marker page, in the showcase mold of the hoff core kit's comment page
//! ("Comment cursor, pin"): the comment cursor in its four states, the
//! comment pin read and unread with one, two, many people and names in
//! six scripts, then the markers on a canvas, each pin's tail on the spot
//! it is anchored to, the spots given as fractions of the canvas so they
//! keep their place at any width.
//!
//! A click on a pin opens its thread, which reads it (the unread border
//! goes); a cursor is an indicator and takes nothing.
//!
//! Projected: a group per title, `marker_page.group[<g>]`; in it each card
//! as a list named by its chip, `…card[<c>]`; in the list each marker,
//! `…card[<c>].marker[<i>]`: a pin a button with `activate` (open), a
//! cursor an image.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::message::marker::{CursorState, Marker};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect};

use super::{After, Component, child_id, root_id, showcase, under};

/// Where a marker sits in its card: centered with the others in a row,
/// or anchored at a spot of the canvas, as fractions of it.
#[derive(Clone, Copy)]
enum Place {
    Row,
    At(f32, f32),
}

struct Card {
    title: &'static str,
    markers: Vec<(Marker, Place)>,
    /// A canvas: its height as a share of its width.
    canvas: Option<f32>,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct MarkerPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The pin a reader focused, `(group, card, marker)`.
    focus: Option<(usize, usize, usize)>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn marker_id(card: &SemId, i: usize) -> Option<SemId> {
    child_id(Some(card), "marker").map(|family| family.key(i.to_string()))
}

fn card(title: &'static str, markers: Vec<Marker>) -> Card {
    Card {
        title,
        markers: markers.into_iter().map(|m| (m, Place::Row)).collect(),
        canvas: None,
    }
}

impl MarkerPage {
    pub fn new() -> Self {
        let cursor = |s| vec![Marker::cursor(s)];
        let mut groups = vec![
            Group {
                title: "Cursor",
                columns: 4,
                cards: vec![
                    card("Point", cursor(CursorState::Point)),
                    card("Pinned", cursor(CursorState::Pinned)),
                    card("Uploading", cursor(CursorState::Uploading)),
                    card("Typing", cursor(CursorState::Typing)),
                ],
            },
            Group {
                title: "Pin",
                columns: 2,
                cards: vec![
                    card("Conversation, Read", vec![Marker::pin(["Luna", "Sam"])]),
                    card(
                        "Single comment, Unread, Hover, Press, Focus",
                        vec![Marker::pin(["Luna"]).unread(true)],
                    ),
                    card(
                        "Many people",
                        vec![Marker::pin(["Luna", "Sam", "Ada", "Kai", "Noor"])],
                    ),
                    card(
                        "Mixed scripts",
                        vec![Marker::pin(["سارة", "李华", "Ана", "Ömer"]).unread(true)],
                    ),
                ],
            },
            Group {
                title: "On a canvas",
                columns: 1,
                cards: vec![Card {
                    title: "Pins anchored to their spots, someone typing",
                    markers: vec![
                        (Marker::pin(["Luna", "Sam"]), Place::At(0.2, 0.45)),
                        (Marker::pin(["Ada"]).unread(true), Place::At(0.55, 0.75)),
                        (Marker::cursor(CursorState::Typing), Place::At(0.4, 0.35)),
                        (Marker::pin(["Kai"]), Place::At(0.78, 0.4)),
                    ],
                    canvas: Some(0.4),
                }],
            },
        ];
        let root = root_id("marker_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    for (i, (m, place)) in std::mem::take(&mut card.markers).into_iter().enumerate()
                    {
                        let m = match marker_id(&cid, i) {
                            Some(id) => m.sem(id),
                            None => m,
                        };
                        card.markers.push((m, place));
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

    fn at(&self, id: &SemId) -> Option<(usize, usize, usize)> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group.cards.iter().enumerate().find_map(|(c, card)| {
                let i = card
                    .markers
                    .iter()
                    .position(|(m, _)| m.sem_id() == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    fn marker_mut(&mut self, (g, c, i): (usize, usize, usize)) -> Option<&mut Marker> {
        Some(
            &mut self
                .groups
                .get_mut(g)?
                .cards
                .get_mut(c)?
                .markers
                .get_mut(i)?
                .0,
        )
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, (m, _)) in card.markers.iter_mut().enumerate() {
                    m.set_focused(at == Some((g, c, i)));
                }
            }
        }
    }

    /// A card's inside: what its markers lay out in.
    fn inside(card: Rect, theme: &Theme) -> Rect {
        card.inset(theme.spacing.sm)
    }

    /// The canvas of a card `w` wide: its share of the width, as tall as
    /// the share says.
    fn canvas_h(card: &Card, w: f32, theme: &Theme) -> f32 {
        let inside = (w - theme.spacing.sm * 2.0).max(0.0);
        card.canvas.map_or(0.0, |share| inside * share)
    }

    /// The tallest marker of a row card.
    fn row_h(card: &Card, theme: &Theme) -> f32 {
        card.markers
            .iter()
            .map(|(m, _)| m.size(theme).1)
            .fold(0.0, f32::max)
    }

    /// Per group its layout, and per card each marker's box.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Vec<Rect>>)> {
        let sized = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.columns, content, theme);
            let cards = g.cards.iter().map(|card| {
                let h = Self::canvas_h(card, w, theme).max(Self::row_h(card, theme));
                showcase::card_height(h, theme)
            });
            (g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let boxes = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| Self::place(card, *rect, theme))
                    .collect();
                (placed, boxes)
            })
            .collect()
    }

    /// The canvas of a card at `rect`, centered in it (empty for a row
    /// card).
    fn canvas(card: &Card, rect: Rect, theme: &Theme) -> Rect {
        let w = Self::inside(rect, theme).w;
        showcase::centered((w, Self::canvas_h(card, rect.w, theme)), rect)
    }

    /// Each marker's box in its card: a row centered in it, or, on a
    /// canvas, its bottom start corner (the pin's tail) at its spot, kept
    /// inside the canvas.
    fn place(card: &Card, rect: Rect, theme: &Theme) -> Vec<Rect> {
        let sizes: Vec<_> = card.markers.iter().map(|(m, _)| m.size(theme)).collect();
        let gap = theme.spacing.sm;
        let row_w = sizes.iter().map(|s| s.0).sum::<f32>() + gap * (sizes.len().max(1) - 1) as f32;
        let row = showcase::centered((row_w, Self::row_h(card, theme)), rect);
        let canvas = Self::canvas(card, rect, theme);
        let mut x = row.x;
        card.markers
            .iter()
            .zip(sizes)
            .map(|((_, place), (w, h))| match place {
                Place::Row => {
                    let r = Rect::new(x, row.y + (row.h - h) / 2.0, w, h);
                    x += w + gap;
                    r
                }
                Place::At(fx, fy) => {
                    let ax = canvas.x + canvas.w * fx;
                    let ay = canvas.y + canvas.h * fy;
                    let x = ax.min(canvas.right() - w).max(canvas.x);
                    let y = (ay - h).min(canvas.bottom() - h).max(canvas.y);
                    Rect::new(x, y, w, h)
                }
            })
            .collect()
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        let layout: Vec<_> = self
            .layout(content, theme)
            .into_iter()
            .map(|(g, _)| g)
            .collect();
        showcase::content_height(&layout, content)
    }

    /// A click lets the reader's focus go; each pin takes the click at its
    /// box and opens.
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
        for (group, (_, boxes)) in self.groups.iter_mut().zip(&layout) {
            for (card, boxes) in group.cards.iter_mut().zip(boxes) {
                for ((m, _), r) in card.markers.iter_mut().zip(boxes) {
                    result = result.merge(m.handle_event(event, *r, theme));
                }
            }
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, boxes))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), boxes) in group.cards.iter().zip(&placed.cards).zip(boxes) {
                showcase::card(c, *rect, card.title, theme);
                if card.canvas.is_some() {
                    // the canvas the markers sit on: the panel wash
                    let r = Self::canvas(card, *rect, theme);
                    let wash = theme.colors.bg_panel.0;
                    c.push(rounded_rect(r.x, r.y, r.w, r.h, theme.shape.item, wash));
                }
                for ((m, _), r) in card.markers.iter().zip(boxes) {
                    m.render(c, *r, theme);
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for MarkerPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = group_id(root, g) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(cid) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&cid, Role::List, card.title).parent(&gid);
                for (m, _) in &card.markers {
                    if let Some(id) = m.sem_id() {
                        m.project(b, id, Some(&cid));
                    }
                }
            }
        }
        let focused = self
            .focus
            .and_then(|(g, c, i)| self.groups.get(g)?.cards.get(c)?.markers.get(i)?.0.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// A pin's `activate` opens it, as a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let pin = self
            .at(&req.target)
            .filter(|_| req.action.as_str() == "activate")
            .and_then(|at| self.marker_mut(at))
            .filter(|m| m.is_pin());
        match pin {
            Some(m) => {
                m.open();
                Ok(After::Nothing)
            }
            None => Err(format!(
                "the marker page has no {} on {}",
                req.action, req.target
            )),
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
    use accord::Actor;

    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    /// The width sweep, 280 to 1600 every 8 px: every marker in its card
    /// and clear of the chip's band, every card in the content, the
    /// groups down the page; a canvas pin's tail on its spot while there
    /// is room.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = MarkerPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (g, (group, (placed, boxes))) in page
                    .groups
                    .iter()
                    .zip(page.layout(content, &theme))
                    .enumerate()
                {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for ((card, rect), boxes) in group.cards.iter().zip(&placed.cards).zip(&boxes) {
                        let lane = Rect::new(content.x, rect.y, content.w, rect.h);
                        assert!(
                            within(*rect, lane),
                            "{w}: {} out of the content",
                            card.title
                        );
                        for r in boxes {
                            assert!(
                                within(*r, rect.inset(theme.spacing.sm)),
                                "{w}: {}",
                                card.title
                            );
                        }
                    }
                }
            }
        }
    }

    /// The ids parse; a reader's focus lands on a pin and goes; a pin's
    /// `activate` reads it; a cursor declares nothing.
    #[test]
    fn a_pin_opens_from_the_projection_and_a_cursor_runs_nothing() {
        let mut page = MarkerPage::new();
        let root = page.root().cloned().expect("the root parses");
        let pin = page.groups[1].cards[1].markers[0]
            .0
            .sem_id()
            .cloned()
            .unwrap();
        let cursor = page.groups[0].cards[0].markers[0]
            .0
            .sem_id()
            .cloned()
            .unwrap();
        assert!(page.focus(&pin));
        assert!(page.groups[1].cards[1].markers[0].0.is_focused());
        assert!(page.blur());
        let seen = {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Marker");
            page.project(&mut b, &root);
            b.finish().expect("the projection validates")
        };
        let ask = |target: &SemId| {
            let actor = Actor::Test {
                name: "marker page".to_owned(),
            };
            Request::observe(&seen, actor, target.as_str(), "activate", None)
        };
        let open = ask(&pin).expect("a pin declares activate");
        assert_eq!(page.run(&open), Ok(After::Nothing));
        assert!(!page.groups[1].cards[1].markers[0].0.is_unread());
        assert!(ask(&cursor).is_err(), "a cursor declares no action");
        let mut stray = open.clone();
        stray.target = cursor;
        assert!(page.run(&stray).is_err());
    }
}
