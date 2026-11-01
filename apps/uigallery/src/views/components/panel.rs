//! Panel page: the surface a screen groups content on, in the showcase
//! mold, at each elevation it takes. The widget is
//! `engine::widgets::content::Panel`; its radius, padding and label are
//! the theme's (`theme.disclosure.panel`), its surface the shared recipe:
//! the kit's bordered inspector panel in the core theme, the glass
//! elsewhere.
//!
//! Projected: a group per title, `panel_page.group[<g>]`; in it each
//! panel, `...card[<c>]`, a group named by its label. A panel takes no
//! action and no focus.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::{TextMeasurer, TextStyle};
use engine::styling::theme::TypeRole;
use engine::styling::theme::{Elevation, Theme};
use engine::widgets::content::Panel;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::accordion::{Fitted, fit_layout};
use super::{After, Component, child_id, root_id, showcase};

/// What every panel holds: one line of body text.
const BODY: &str = "Camera, lights and the loop of the scene.";

struct Card {
    title: &'static str,
    panel: Panel,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct PanelPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

impl PanelPage {
    pub fn new() -> Self {
        let card = |title, label: &str, elevation| Card {
            title,
            panel: Panel::new().label(label).elevation(elevation),
        };
        let groups = vec![
            Group {
                title: "Panel",
                columns: 3,
                cards: vec![
                    card("Flat: on the page", "Scene", Elevation::Base),
                    card("Raised: a block over it", "Camera", Elevation::Raised),
                    card("Overlay: a sheet", "Export", Elevation::Overlay),
                ],
            },
            Group {
                title: "Panel, ranges",
                columns: 1,
                cards: vec![card(
                    "Wide: the padding holds at any width",
                    "Design and animation",
                    Elevation::Base,
                )],
            },
        ];
        Self {
            root: root_id("panel_page"),
            groups,
        }
    }

    /// A panel's height at the width `w`: its padding, its label band
    /// and the body, wrapped to what the padding leaves.
    fn panel_h(theme: &Theme, w: f32) -> f32 {
        let probe = Panel::new().label("x");
        let inner = probe.content_rect(Rect::new(0.0, 0.0, w, 0.0), theme);
        let wrap = Some(inner.w.max(0.0));
        let (_, body) = TextMeasurer::measure_styled(BODY, &Self::body_style(theme), wrap);
        inner.y + body + Panel::pad(theme)
    }

    fn body_style(theme: &Theme) -> TextStyle {
        theme.typography.role(TypeRole::Body)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<Fitted> {
        let spans: Vec<_> = self
            .groups
            .iter()
            .map(|g| (g.columns, g.cards.len()))
            .collect();
        fit_layout(&spans, content, theme, |_, _, w| Self::panel_h(theme, w))
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
        let style = Self::body_style(theme);
        for (i, (group, placed)) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                showcase::card(c, *rect, card.title, theme);
                let at = Rect::new(body.x, body.y, body.w, Self::panel_h(theme, body.w));
                card.panel.render(c, at, theme);
                let inner = card.panel.content_rect(at, theme);
                c.push(SceneNode::Text {
                    key: TextNodeKey::from_style(BODY, &style, Some(inner.w)),
                    x: inner.x,
                    y: inner.y,
                    color: theme.colors.text.0,
                });
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for PanelPage {
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
                    card.panel.project(b, &id, card.title, Some(&at));
                }
            }
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the panel page has no {} on {}",
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

    /// The width sweep, 280 to 1600 every 8 px, under core and hoff: each
    /// panel inside its card, its body line inside the panel.
    #[test]
    fn every_panel_fits_its_card_from_280_to_1600() {
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            let page = PanelPage::new();
            let style = PanelPage::body_style(&theme);
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                for (group, placed) in page.groups.iter().zip(page.layout(content, &theme)) {
                    for (card, (rect, body)) in group.cards.iter().zip(&placed.cards) {
                        let what = format!("{w}px, {}", card.title);
                        let at =
                            Rect::new(body.x, body.y, body.w, PanelPage::panel_h(&theme, body.w));
                        assert!(at.right() <= rect.right() && at.bottom() <= rect.bottom() + 0.01);
                        let inner = card.panel.content_rect(at, &theme);
                        let (_, h) = TextMeasurer::measure_styled(BODY, &style, Some(inner.w));
                        assert!(inner.y + h <= at.bottom() + 0.01, "{what}: the body spills");
                    }
                }
            }
        }
    }
}
