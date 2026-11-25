//! Aspect Ratio page: the kit's AssetCard well (`aspect-square`) and the
//! other proportions a frame takes, in the showcase mold. Each card holds
//! one frame as wide as the card and as tall as its proportion makes it,
//! so a frame grows and shrinks with its column, never with the window.
//! The widgets are `engine::widgets::content::{AspectRatio, AspectFrame}`;
//! every visual value comes from the theme (`theme.disclosure.aspect`).
//!
//! Projected: a group per title, `aspect_ratio_page.group[<g>]`; in it
//! each frame, `...card[<c>]`, an image named by the card's chip and
//! described by its proportion. A frame is looked at, not operated.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{AspectFrame, AspectRatio};
use engine::widgets::{EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase};

struct Card {
    /// The chip: which proportion and states this card shows.
    title: &'static str,
    frame: AspectFrame,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct AspectRatioPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

/// `aspect_ratio_page.group[<g>]`.
fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

/// `aspect_ratio_page.group[<g>].card[<c>]`.
fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, w: f32, h: f32, glyph: &'static str) -> Card {
    Card {
        title,
        frame: AspectFrame::new(title, AspectRatio::new(w, h)).glyph(glyph),
    }
}

impl AspectRatioPage {
    pub fn new() -> Self {
        let mut groups = vec![
            Group {
                title: "Aspect ratio",
                columns: 2,
                cards: vec![
                    card("1:1, the kit's square: Default, Hover", 1.0, 1.0, "square"),
                    card("16:9", 16.0, 9.0, "play"),
                ],
            },
            Group {
                title: "Aspect ratio, ranges",
                columns: 3,
                cards: vec![
                    card("4:3", 4.0, 3.0, "layers"),
                    card("3:4", 3.0, 4.0, "file"),
                    card("21:9", 21.0, 9.0, "layout-grid"),
                ],
            },
        ];
        let root = root_id("aspect_ratio_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.frame = card.frame.clone().sem(id);
                    }
                }
            }
        }
        Self { root, groups }
    }

    /// Every group placed in `content`: each frame as wide as its card and
    /// as tall as its proportion, a row as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.title, g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, w| {
            self.groups[g].cards[c].frame.height(w)
        })
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    /// The pointer over a frame shades it.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let layout = self.layout(content, theme);
        let mut result = EventResult::IGNORED;
        for (group, placed) in self.groups.iter_mut().zip(&layout) {
            for (card, (_, body)) in group.cards.iter_mut().zip(&placed.cards) {
                result = result.merge(card.frame.handle_event(event, *body));
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
                card.frame.render(c, *body, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for AspectRatioPage {
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
                if let Some(id) = card.frame.sem_id() {
                    card.frame.project(b, id, Some(&at));
                }
            }
        }
    }

    /// A frame has nothing to run.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the aspect ratio page has no {} on {}",
            req.action, req.target
        ))
    }

    /// No frame takes the focus.
    fn focus(&mut self, _id: &SemId) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// frame in its card, keeping its proportion, cards apart.
    #[test]
    fn every_frame_keeps_its_proportion_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = AspectRatioPage::new();
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (k, (card, (rect, body))) in
                        group.cards.iter().zip(&placed.cards).enumerate()
                    {
                        let what = format!("{w}px, {}", card.title);
                        assert!(rect.right() <= content.right() + 0.01, "{what}");
                        assert!(
                            body.x >= rect.x && body.right() <= rect.right() + 0.01,
                            "{what}"
                        );
                        assert!(body.bottom() <= rect.bottom() + 0.01, "{what}");
                        let ratio = card.frame.ratio().ratio();
                        assert!((body.w / body.h - ratio).abs() < 1e-3, "{what}: proportion");
                        for (other, _) in &placed.cards[k + 1..] {
                            let apart = rect.right() <= other.x || other.bottom() <= rect.y;
                            assert!(apart || rect.bottom() <= other.y, "{what}: overlap");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn each_frame_is_an_image_described_by_its_proportion() {
        let page = AspectRatioPage::new();
        let root = page.root.clone().expect("the root parses");
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Aspect Ratio");
        page.project(&mut b, &root);
        let seen = b.finish().expect("a valid tree");
        let id = SemId::parse("aspect_ratio_page.group[0].card[1]").unwrap();
        let node = seen.get(&id).expect("the 16:9 frame");
        assert_eq!(node.role, Role::Image);
        assert_eq!(node.description.as_deref(), Some("16:9"));
    }
}
