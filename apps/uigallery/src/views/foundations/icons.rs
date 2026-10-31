//! Icons page, the hoff core kit's IconographyPage: a group per icon size
//! the theme's controls carry (`control.icons`, the kit's "Icon 16" and
//! "Icon 20"), each a row of square cards with the icon centered and its
//! name in the chip. The set is the Lucide one the engine ships
//! (`engine::widgets::icons`), not the kit's own drawings.

use engine::drawing::compositor::Compositor;
use engine::styling::theme::{IconSize, Theme};
use engine::widgets::icons;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::super::components::showcase::{self, Grid, GroupLayout};

/// Cards to a row at full width (the kit's `span={6}`).
const SPAN: usize = 6;
/// The sizes shown, one group each: the kit's two.
const SIZES: [IconSize; 2] = [IconSize::Sm, IconSize::Md];

pub struct IconsPage {
    names: Vec<&'static str>,
}

impl IconsPage {
    pub fn new() -> Self {
        Self {
            names: icons::icon_names(),
        }
    }

    /// The narrowest a card gets: half the theme's narrowest card.
    fn min_w(theme: &Theme) -> f32 {
        theme.size.card_min_w / 2.0
    }

    /// A group per size, square cards (`aspect-square`).
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let side = showcase::card_width_at(SPAN, Self::min_w(theme), content, theme);
        let grids = SIZES.map(|_| Grid {
            span: SPAN,
            min_w: Self::min_w(theme),
            heights: vec![side; self.names.len()],
        });
        showcase::layout_grids(grids, content, theme)
    }

    /// Natural height of the icon grid (page scrolling needs it).
    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// A specimen page: nothing to hit.
    pub fn handle_event(&mut self, _event: &WidgetEvent, _content: Rect) -> EventResult {
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = layout.len().saturating_sub(1);
        for (g, (group, size)) in layout.iter().zip(SIZES).enumerate() {
            let edge = theme.control.icon(size);
            showcase::title(c, &format!("Icon {edge}"), content, group.title_y, theme);
            for (name, card) in self.names.iter().zip(&group.cards) {
                showcase::card(c, *card, name, theme);
                let at = showcase::centered((edge, edge), *card);
                if let Some(node) = icons::icon_at(name, edge, theme.colors.text.0, at.x, at.y) {
                    c.push(node);
                }
            }
            if g < last {
                showcase::rule(c, content, group.rule_y, theme);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::drawing::compositor::{LayerId, SceneNode};

    /// Every icon of the set, at each size, in a square card inside the
    /// content, from 280 to 1600 wide.
    #[test]
    fn square_cards_fit_from_narrow_to_wide() {
        let page = IconsPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(0.0, 0.0, w as f32, 800.0);
                let layout = page.layout(content, &theme);
                assert_eq!(layout.len(), SIZES.len());
                for group in &layout {
                    assert_eq!(group.cards.len(), page.names.len());
                    for r in &group.cards {
                        assert!(r.right() <= content.right() + 0.5, "{w}: {r:?}");
                        assert!((r.w - r.h).abs() < 0.5, "{w}: not square {r:?}");
                    }
                }
            }
        }
    }

    /// One path per icon per size, at the theme's icon edges.
    #[test]
    fn every_icon_draws_at_each_size() {
        let page = IconsPage::new();
        let theme = Theme::hoff_core();
        let mut c = Compositor::new();
        page.render(&mut c, Rect::new(0.0, 0.0, 1000.0, 800.0), &theme);
        let paths = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .filter(|n| matches!(n, SceneNode::Path { .. }))
            .count();
        assert_eq!(paths, page.names.len() * SIZES.len());
        assert_eq!(theme.control.icon(IconSize::Sm), 16.0);
    }
}
