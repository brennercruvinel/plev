//! Panel Header page: the column head every panel starts with, in the
//! showcase mold: alone, with its blurb, with its count badge, and a
//! title longer than its card, cut. The widget is
//! `engine::widgets::nav::PanelHeader`; its measures and tones are the
//! theme's (`theme.disclosure.header`): the kit's Notifications head in
//! the core theme, the glass head elsewhere.
//!
//! Projected: a group per title, `panel_header_page.group[<g>]`; in it
//! each head, `...card[<c>]`, a label named by its title, described by
//! its blurb, its badge as `badge`. A head takes no action and no focus.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::nav::PanelHeader;
use engine::widgets::prelude::{Badge, EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase};

struct Card {
    title: &'static str,
    header: PanelHeader,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct PanelHeaderPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl PanelHeaderPage {
    pub fn new() -> Self {
        let card = |title, header| Card { title, header };
        let groups = vec![
            Group {
                title: "Panel Header",
                columns: 3,
                cards: vec![
                    card("Title", PanelHeader::new("Notifications")),
                    card(
                        "Title and badge",
                        PanelHeader::new("Changes").badge(Badge::tag("12")),
                    ),
                    card(
                        "Title, blurb and badge",
                        PanelHeader::new("Comments")
                            .blurb("Three unread since yesterday")
                            .badge(Badge::tag("3")),
                    ),
                ],
            },
            Group {
                title: "Panel Header, ranges",
                columns: 2,
                cards: vec![card(
                    "Long title: cut before the badge",
                    PanelHeader::new("Every asset of the scene and its animation timeline")
                        .badge(Badge::tag("128")),
                )],
            },
        ];
        Self {
            root: root_id("panel_header_page"),
            groups,
        }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |g, c, _| {
            self.groups[g].cards[c].header.height(theme)
        })
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.layout(content, theme)
            .last()
            .map_or(0.0, |placed| placed.rule_y - content.y)
    }

    pub fn handle_event(
        &mut self,
        _event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, placed)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                let at = Rect::new(body.x, body.y, body.w, card.header.height(theme));
                card.header.render(c, at, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for PanelHeaderPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(at) = group_id(root, g) else {
                continue;
            };
            b.node(&at, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                if let Some(id) = card_id(root, g, c) {
                    card.header.project(b, &id, Some(&at));
                }
            }
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the panel header page has no {} on {}",
            req.action, req.target
        ))
    }

    fn focus(&mut self, _id: &SemId) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::drawing::compositor::{LayerId, SceneNode};
    use engine::drawing::text::TextMeasurer;

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff:
    /// every title ends before its badge and inside its card.
    #[test]
    fn every_head_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = PanelHeaderPage::new();
            let style = PanelHeader::title_style(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                        let at = Rect::new(body.x, body.y, body.w, card.header.height(&theme));
                        assert!(at.bottom() <= rect.bottom() + 0.01, "{w}px: too tall");
                        let mut c = Compositor::new();
                        c.begin_frame();
                        card.header.render(&mut c, at, &theme);
                        let room = card.header.trailing_rect(at, &theme).right();
                        let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
                        let Some(SceneNode::Text { key, x, .. }) = nodes
                            .iter()
                            .find(|n| matches!(n, SceneNode::Text { key, .. } if key.font_size_bits == style.font_size.to_bits()))
                        else {
                            panic!("{w}px: no title");
                        };
                        let (tw, _) = TextMeasurer::measure_styled(&key.text, &style, None);
                        assert!(
                            x + tw <= room + 0.5,
                            "{w}px, {}: the title runs on",
                            card.title
                        );
                    }
                }
            }
        }
    }
}
