//! Home page, the hoff core kit's home (`app/page.tsx`, `templates/
//! HomePage`, titled Branding): one group of three cards, a row of them
//! wide, the logo in each, the symbol, the wordmark and both. The logo is
//! `engine::widgets::header::Logo`; every visual value is
//! `theme.header.logo`. The cards are as tall as the kit's narrowest
//! step in narrow content and its widest in wide, on the theme's fluid
//! line between them, never shorter than the logo in the stage's pads:
//! no step at a width (ADR 0073).
//!
//! Projected: the group `home_page.group[0]`, and in it each card's logo
//! as the link home, `home_page.group[0].card[<c>]`, described by its
//! card's chip. A press does what a click does: nothing past the press.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::{Theme, fluid};
use engine::widgets::header::{Logo, LogoKind};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::showcase::{self, GroupLayout};
use super::topbar::PRODUCT;
use super::{After, Component, child_id, root_id, under};

/// The cards' height at the kit's ends (`templates/HomePage/index.tsx:15`,
/// `h-154.5 max-[1419px]:h-80 max-[1023px]:h-60`): `h-60` in its
/// narrowest window, `h-154.5` in its widest. The `h-80` between is a
/// point the line passes near, not a step.
const CARD_NARROW: f32 = 240.0;
const CARD_WIDE: f32 = 618.0;
/// Three to a row (`span={3}`).
const SPAN: usize = 3;

struct Card {
    title: &'static str,
    logo: Logo,
}

pub struct HomePage {
    root: Option<SemId>,
    cards: Vec<Card>,
    focus: Option<usize>,
}

/// `home_page.group[0]`.
fn group_id(root: &SemId) -> Option<SemId> {
    child_id(Some(root), "group").map(|g| g.key("0"))
}

impl HomePage {
    pub fn new() -> Self {
        let root = root_id("home_page");
        let card = |title, kind| Card {
            title,
            logo: Logo::new(PRODUCT).kind(kind),
        };
        let mut cards = vec![
            card("Symbol", LogoKind::Symbol),
            card("Wordmark", LogoKind::Wordmark),
            card("Full", LogoKind::Full),
        ];
        let family = root
            .as_ref()
            .and_then(group_id)
            .and_then(|g| g.child("card").ok());
        if let Some(family) = family {
            for (i, card) in cards.iter_mut().enumerate() {
                let logo = std::mem::replace(&mut card.logo, Logo::new(PRODUCT));
                card.logo = logo.sem(family.key(i.to_string()));
            }
        }
        Self {
            root,
            cards,
            focus: None,
        }
    }

    /// The cards' height in `content`: from the kit's narrow height to
    /// its wide one over the widths the theme's type grows across, and
    /// at least the tallest logo with the stage's pad above and below.
    fn card_h(&self, content: Rect, theme: &Theme) -> f32 {
        let range = theme.typography.roles.range;
        let fluid = fluid(content.w, range, [CARD_NARROW, CARD_WIDE]).round();
        let logo = self
            .cards
            .iter()
            .map(|c| c.logo.preferred_size(theme).1)
            .fold(0.0, f32::max);
        fluid.max(logo + showcase::pad(content.w, theme) * 2.0)
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let h = self.card_h(content, theme);
        showcase::layout_untitled(SPAN, vec![h; self.cards.len()], content, theme)
    }

    fn logo_rect(logo: &Logo, card: Rect, theme: &Theme) -> Rect {
        showcase::centered(logo.preferred_size(theme), card)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
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
        for (card, rect) in self.cards.iter_mut().zip(&layout[0].cards) {
            let at = Self::logo_rect(&card.logo, *rect, theme);
            result = result.merge(card.logo.handle_event(event, at));
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (card, rect) in self.cards.iter().zip(&layout[0].cards) {
            showcase::card(c, *rect, card.title, theme);
            card.logo
                .render(c, Self::logo_rect(&card.logo, *rect, theme), theme);
        }
    }

    fn set_focus(&mut self, at: Option<usize>) {
        self.focus = at;
        for (i, card) in self.cards.iter_mut().enumerate() {
            card.logo.set_focused(at == Some(i));
        }
    }

    fn at(&self, id: &SemId) -> Option<usize> {
        self.cards.iter().position(|c| c.logo.sem_id() == Some(id))
    }
}

impl Default for HomePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for HomePage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(group) = group_id(root) else {
            return;
        };
        b.node(&group, Role::Group, "Branding").parent(root);
        for card in &self.cards {
            if let Some(id) = card.logo.sem_id() {
                card.logo
                    .project(b, id, Some(&group))
                    .description(card.title);
            }
        }
        let focused = self.focus.and_then(|i| self.cards.get(i)?.logo.sem_id());
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// The link's `activate`: it presses, as a click does.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        match (self.at(&req.target), req.action.as_str()) {
            (Some(_), "activate") => Ok(After::Nothing),
            _ => Err(format!(
                "the home page has no {} on {}",
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

    /// The template's three cards in its order, three to a row and as
    /// tall as the kit at its ends, each logo centered in its card.
    #[test]
    fn the_page_is_the_kit_s_branding() {
        let page = HomePage::new();
        let titles: Vec<&str> = page.cards.iter().map(|c| c.title).collect();
        assert_eq!(titles, ["Symbol", "Wordmark", "Full"]);
        let theme = Theme::hoff_core();
        let wide = Rect::new(0.0, 0.0, 1200.0, 900.0);
        let cards = &page.layout(wide, &theme)[0].cards;
        assert_eq!(cards[0].y, cards[2].y, "one row");
        let [_, to] = theme.typography.roles.range;
        let widest = Rect::new(0.0, 0.0, to, 900.0);
        assert_eq!(page.layout(widest, &theme)[0].cards[0].h, CARD_WIDE);
        let phone = Rect::new(0.0, 0.0, 320.0, 900.0);
        assert_eq!(page.layout(phone, &theme)[0].cards[0].h, CARD_NARROW);
        for (card, rect) in page.cards.iter().zip(cards) {
            let r = HomePage::logo_rect(&card.logo, *rect, &theme);
            assert!((r.x + r.w / 2.0 - (rect.x + rect.w / 2.0)).abs() <= 0.5);
            assert!(card.logo.sem_id().is_some());
        }
        let [symbol, word, full] = [0, 1, 2].map(|i| page.cards[i].logo.preferred_size(&theme));
        assert!(full.0 > word.0 && word.0 > symbol.0);
    }

    /// From a phone to a wide window the cards grow a pixel at a time:
    /// no width where the height jumps (the kit's 1419 and 1023 steps
    /// are gone), and never less than the logo in the stage's pads.
    #[test]
    fn the_cards_grow_with_the_width_without_a_step() {
        let page = HomePage::new();
        let theme = Theme::hoff_core();
        let mut last = 0.0;
        for w in (280..=1800).step_by(4) {
            let content = Rect::new(0.0, 0.0, w as f32, 900.0);
            let h = page.layout(content, &theme)[0].cards[0].h;
            assert!(h >= last, "{w}: it only grows");
            if last > 0.0 {
                assert!(h - last <= 2.0, "{w}: {last} to {h}, a step");
            }
            let logo = page.cards[2].logo.preferred_size(&theme).1;
            assert!(h >= logo + showcase::pad(content.w, &theme) * 2.0, "{w}");
            last = h;
        }
    }
}
