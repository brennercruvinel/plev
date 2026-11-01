//! Resizable page: split panes with a draggable divider, in the showcase
//! mold: two panes side by side, and two stacked. The widget is
//! `engine::widgets::nav::SplitPane`; the divider's tones are the
//! theme's (`theme.disclosure.split`): the kit's line, its gray while
//! hovered or dragged, in the core theme; the glass edge elsewhere. Each
//! pane keeps its minimum: a narrow card moves the divider, never past
//! it.
//!
//! Projected (task-75 phase 2b): a group per title,
//! `resizable_page.group[<g>]`; the first split keeps its id,
//! `resizable_page.split`, the stacked one is
//! `resizable_page.group[1].card[0]`; each named by its chip and
//! described by the first pane's share; `set_ratio` moves the divider
//! where a drag would. A reader has no way to ask it yet: the tree sets a
//! value only through `set_value` (and a select's `select`), and the
//! contract has no splitter role.

use accord::{Request, Role, SemId, SnapshotBuilder, Value};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::styling::theme::Theme;
use engine::styling::theme::TypeRole;
use engine::widgets::prelude::{EventResult, Rect, SplitDirection, SplitPane, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase, under};

struct Card {
    title: &'static str,
    split: SplitPane,
}

pub struct ResizablePage {
    root: Option<SemId>,
    /// Group and card of each split.
    cards: Vec<(usize, Card)>,
}

const GROUPS: [&str; 2] = ["Resizable", "Resizable, vertical"];

impl ResizablePage {
    pub fn new(theme: &Theme) -> Self {
        let root = root_id("resizable_page");
        let stacked = child_id(
            child_id(root.as_ref(), "group")
                .map(|g| g.key("1"))
                .as_ref(),
            "card",
        )
        .map(|c| c.key("0"));
        let ids = [child_id(root.as_ref(), "split"), stacked];
        let splits = [
            (
                0,
                "Default, Hover, Drag: two panes",
                SplitPane::new(SplitDirection::Horizontal, 0.35, theme),
            ),
            (
                1,
                "Stacked: a pane over another",
                SplitPane::new(SplitDirection::Vertical, 0.5, theme),
            ),
        ];
        let cards = splits
            .into_iter()
            .zip(ids)
            .map(|((g, title, split), id)| {
                let split = match id {
                    Some(id) => split.sem(id),
                    None => split,
                };
                (g, Card { title, split })
            })
            .collect();
        Self { root, cards }
    }

    /// A split's height: one card's worth.
    fn split_h(theme: &Theme) -> f32 {
        theme.size.card_min_w
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans = [(1, 1), (1, 1)];
        fit_layout(&spans, content, theme, |_, _, _| Self::split_h(theme))
    }

    /// Each split's rect, in the order of the cards.
    fn rects(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        let layout = self.layout(content, theme);
        self.cards
            .iter()
            .map(|(g, _)| {
                let (_, body) = layout[*g].cards[0];
                Rect::new(body.x, body.y, body.w, Self::split_h(theme))
            })
            .collect()
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
        let rects = self.rects(content, theme);
        let mut result = EventResult::IGNORED;
        for ((_, card), at) in self.cards.iter_mut().zip(rects) {
            result = result.merge(card.split.handle_event(event, at));
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let rects = self.rects(content, theme);
        let style = theme.typography.role(TypeRole::Body);
        let pad = theme.spacing.lg;
        for (g, placed) in layout.iter().enumerate() {
            showcase::title(c, GROUPS[g], content, placed.title_y, theme);
            if g + 1 < layout.len() {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
        for ((g, card), at) in self.cards.iter().zip(rects) {
            let (rect, _) = layout[*g].cards[0];
            showcase::card(c, rect, card.title, theme);
            let first = card.split.first_rect(at);
            let second = card.split.second_rect(at);
            let size = match card.split.direction {
                SplitDirection::Horizontal => first.w,
                SplitDirection::Vertical => first.h,
            };
            let lines = [
                (first, format!("first pane: {size:.0}px")),
                (second, "second pane".to_string()),
            ];
            for (pane, line) in lines {
                let wrap = Some((pane.w - pad * 2.0).max(0.0));
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(&line, &style, wrap),
                    x: pane.x + pad,
                    y: pane.y + pad,
                    color: theme.colors.text.0,
                });
            }
            card.split.render(c, at, theme);
        }
    }
}

impl Component for ResizablePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let groups: Vec<_> = (0..GROUPS.len())
            .map(|g| child_id(Some(root), "group").map(|f| f.key(g.to_string())))
            .collect();
        for (g, title) in GROUPS.iter().enumerate() {
            if let Some(at) = &groups[g] {
                b.node(at, Role::Group, *title).parent(root);
            }
        }
        for (g, card) in &self.cards {
            let (Some(id), Some(at)) = (card.split.sem_id(), &groups[*g]) else {
                continue;
            };
            let share = (card.split.ratio() * 100.0).round();
            card.split
                .project(b, id, card.title, Some(at))
                .description(format!("the first pane takes {share}%"));
        }
    }

    /// `set_ratio` moves the divider, as a drag does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let on = self
            .cards
            .iter_mut()
            .find(|(_, card)| card.split.sem_id() == Some(&req.target));
        match (req.action.as_str(), &req.param, on) {
            ("set_ratio", Some(Value::Num(r)), Some((_, card))) => {
                card.split.set_ratio(*r as f32);
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the resizable page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    /// Nothing of the split takes focus; the page's nodes are its own all
    /// the same.
    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_drags() {
        let theme = Theme::hoff();
        let mut page = ResizablePage::new(&theme);
        let content = Rect::new(288.0, 80.0, 1272.0, 900.0);
        let at = page.rects(content, &theme)[0];
        let d = page.cards[0].1.split.divider_rect(at);
        let (x, y) = (d.x + 1.0, d.y + 20.0);
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        assert!(page.cards[0].1.split.is_dragging());
        page.handle_event(&WidgetEvent::MouseMove { x: x + 100.0, y }, content, &theme);
        assert!(page.cards[0].1.split.ratio() > 0.35);
    }

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// split inside its card, both panes and the divider inside the split.
    #[test]
    fn every_split_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = ResizablePage::new(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 900.0);
                for ((_, card), at) in page.cards.iter().zip(page.rects(content, &theme)) {
                    let parts = [
                        card.split.first_rect(at),
                        card.split.divider_rect(at),
                        card.split.second_rect(at),
                    ];
                    for part in parts {
                        assert!(
                            part.x >= at.x - 0.01 && part.right() <= at.right() + 0.01,
                            "{w}px"
                        );
                        assert!(
                            part.y >= at.y - 0.01 && part.bottom() <= at.bottom() + 0.01,
                            "{w}px"
                        );
                    }
                }
            }
        }
    }
}
