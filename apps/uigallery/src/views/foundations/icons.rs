//! Icons page, the hoff core kit's IconographyPage: a group per icon size
//! the theme's controls carry (`control.icons`, the kit's "Icon 16" and
//! "Icon 20"), each a row of square cards with the icon centered and its
//! name in the chip, then the kit's "Notification Icons": the badge of
//! each kind of notification, a white icon on its color. The set is the
//! Lucide one the engine ships (`engine::widgets::icons`), not the kit's
//! own drawings.

use engine::drawing::compositor::Compositor;
use engine::styling::color::Color;
use engine::styling::theme::{IconSize, Intent, Theme};
use engine::widgets::icons;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent, rounded_rect};

use super::super::components::showcase::{self, Grid, GroupLayout};

/// Cards to a row at full width (the kit's `span={6}`).
const SPAN: usize = 6;
/// The sizes shown, one group each: the kit's two.
const SIZES: [IconSize; 2] = [IconSize::Sm, IconSize::Md];
/// Notification badges to a row (the kit's `span={5}`).
const BADGE_SPAN: usize = 5;
/// The kit's notification badges (IconographyPage :2334, Notification
/// :33-49), in its order: the kind, its icon, and the intent whose badge
/// color the alert tokens give it (`theme.message.alert.intents`). The
/// image's badge (#AD9000) has no intent: it takes the theme's warning.
const BADGES: [(&str, &str, Option<Intent>); 5] = [
    ("comment", "message-circle", Some(Intent::Neutral)),
    ("like", "heart", Some(Intent::Destructive)),
    ("video", "video", Some(Intent::Informational)),
    ("image", "image", None),
    ("invite", "user-plus", Some(Intent::Constructive)),
];

/// A badge's color under `theme`.
fn badge_color(intent: Option<Intent>, theme: &Theme) -> Color {
    match intent {
        Some(i) => theme.message.alert.intents.of(i),
        None => theme.colors.warning,
    }
}

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

    /// A group per size, then the badges, in square cards
    /// (`aspect-square`).
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let min_w = Self::min_w(theme);
        let side = showcase::card_width_at(SPAN, min_w, content, theme);
        let badge_side = showcase::card_width_at(BADGE_SPAN, min_w, content, theme);
        let grids = SIZES
            .map(|_| Grid::new(SPAN, min_w, vec![side; self.names.len()]))
            .into_iter()
            .chain([Grid::new(BADGE_SPAN, min_w, vec![badge_side; BADGES.len()])]);
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
        if let Some(badges) = layout.get(SIZES.len()) {
            showcase::title(c, "Notification Icons", content, badges.title_y, theme);
            let a = &theme.message.alert;
            let disc = a.badge - a.badge_ring * 2.0;
            for ((kind, icon, intent), card) in BADGES.iter().zip(&badges.cards) {
                showcase::card(c, *card, kind, theme);
                let at = showcase::centered((disc, disc), *card);
                let color = badge_color(*intent, theme);
                c.push(rounded_rect(at.x, at.y, disc, disc, disc / 2.0, color.0));
                let glyph = showcase::centered((a.badge_icon, a.badge_icon), at);
                if let Some(node) = icons::icon_at(icon, a.badge_icon, a.ring.0, glyph.x, glyph.y) {
                    c.push(node);
                }
            }
        }
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
                assert_eq!(layout.len(), SIZES.len() + 1);
                for (g, group) in layout.iter().enumerate() {
                    let n = if g < SIZES.len() {
                        page.names.len()
                    } else {
                        BADGES.len()
                    };
                    assert_eq!(group.cards.len(), n);
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
        assert_eq!(paths, page.names.len() * SIZES.len() + BADGES.len());
        assert_eq!(theme.control.icon(IconSize::Sm), 16.0);
    }

    /// The kit's five notification badges, each icon in the set, the four
    /// with an intent in the alert tokens' colors.
    #[test]
    fn the_notification_badges_are_the_kits() {
        let theme = Theme::hoff_core();
        for (kind, icon, _) in BADGES {
            assert!(icons::has_icon(icon), "{kind}: {icon}");
        }
        let hex = |i: Option<Intent>| badge_color(i, &theme);
        assert_eq!(hex(Some(Intent::Neutral)), Color::hex(0x8651F8));
        assert_eq!(hex(Some(Intent::Informational)), Color::hex(0x1BA4F3));
    }
}
