//! Topbar page, the hoff core kit's TopbarPage (`templates/TopbarPage`):
//! one group of grey cards, the row's width each (`px-10 py-20 isGray`),
//! a header in each: Simple, Asset, Minimal, Simple with Filter and Show
//! search. The headers are `engine::widgets::header::Header`; every visual
//! value is `theme.header`, the search the form's input group.
//!
//! Projected: the group `topbar_page.group[0]`, and in it each card's
//! header as a toolbar, `topbar_page.group[0].card[<c>]`, named by the
//! card's chip, its parts under it. A press does what a click does (a
//! thumbnail becomes the current asset, the rest only press); the search
//! takes the text the person types. The page owns the keyboard focus as
//! the form pages do: Tab walks the parts, the arrows move along the
//! thumbnails, Escape blurs.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::Theme;
use engine::widgets::form::TextField;
use engine::widgets::header::Header;
use engine::widgets::prelude::{
    EventResult, Rect, Wake, WidgetEvent, rounded_rect, rounded_rect_stroke,
};

use super::showcase::{self, GroupLayout};
use super::{After, Component, EditKey, child_id, root_id, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

/// The product the kit's logo spells.
pub(crate) const PRODUCT: &str = "Brainwave";

/// A card's padding around its header, `py-20` and `px-10`: page values
/// of the template, as the showcase's own.
const CARD_PAD_Y: f32 = 80.0;
const CARD_PAD_X: f32 = 40.0;

struct Card {
    title: &'static str,
    header: Header,
}

pub struct TopbarPage {
    root: Option<SemId>,
    cards: Vec<Card>,
}

/// `topbar_page.group[0]`.
fn group_id(root: &SemId) -> Option<SemId> {
    child_id(Some(root), "group").map(|g| g.key("0"))
}

/// A grey card (`isGray`): the panel tone, its hairline, and the chip in
/// the page's tone with an edge (`bg-[#FCFCFC] border-[#ECECEC]`).
pub(crate) fn gray_card(c: &mut Compositor, rect: Rect, title: &str, theme: &Theme) {
    let (radius, edge) = (theme.shape.card, theme.colors.divider.0);
    c.push(rounded_rect(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        radius,
        theme.colors.bg_panel.0,
    ));
    c.push(rounded_rect_stroke(
        rect.x, rect.y, rect.w, rect.h, radius, edge, 1.0,
    ));
    let inset = theme.spacing.sm;
    let text = showcase::chip_fit(title, rect.w - inset * 2.0, theme);
    let at = showcase::chip_rect(rect.x + inset, rect.y + inset, &text, theme);
    c.push(rounded_rect(
        at.x,
        at.y,
        at.w,
        at.h,
        theme.shape.nav,
        theme.colors.bg.0,
    ));
    c.push(rounded_rect_stroke(
        at.x,
        at.y,
        at.w,
        at.h,
        theme.shape.nav,
        edge,
        1.0,
    ));
    let style = showcase::chip_style(theme);
    let (_, th) = TextMeasurer::measure_styled(&text, &style, None);
    c.push(SceneNode::Text {
        key: TextNodeKey::from_style(&text, &style, None),
        x: at.x + theme.spacing.md,
        y: at.y + (at.h - th.max(style.line_height)) / 2.0,
        color: theme.colors.text.0,
    });
}

impl TopbarPage {
    pub fn new() -> Self {
        let root = root_id("topbar_page");
        let mut cards = vec![
            Card {
                title: "Simple",
                header: Header::simple(),
            },
            Card {
                title: "Asset",
                header: Header::asset(5, 2),
            },
            Card {
                title: "Minimal",
                header: Header::minimal(PRODUCT),
            },
            Card {
                title: "Simple with Filter",
                header: Header::simple_filters(),
            },
            Card {
                title: "Show search",
                header: Header::simple_search(PRODUCT),
            },
        ];
        if let Some(group) = root.as_ref().and_then(group_id) {
            for (i, card) in cards.iter_mut().enumerate() {
                let Ok(family) = group.child("card") else {
                    continue;
                };
                let header = std::mem::replace(&mut card.header, Header::simple());
                card.header = header.sem(family.key(i.to_string()));
            }
        }
        Self { root, cards }
    }

    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let heights = self
            .cards
            .iter()
            .map(|c| c.header.height(theme) + CARD_PAD_Y * 2.0)
            .collect();
        showcase::layout_sized([(1, heights)], content, theme)
    }

    /// The header's bar inside its card.
    fn bar(card: Rect, header: &Header, theme: &Theme) -> Rect {
        let h = header.height(theme);
        Rect::new(
            card.x + CARD_PAD_X,
            card.y + (card.h - h) / 2.0,
            (card.w - CARD_PAD_X * 2.0).max(0.0),
            h,
        )
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
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur_parts() {
            result = EventResult::changed();
        }
        let layout = self.layout(content, theme);
        for (card, rect) in self.cards.iter_mut().zip(&layout[0].cards) {
            let bar = Self::bar(*rect, &card.header, theme);
            result = result.merge(card.header.handle_event(event, bar, theme));
        }
        result
    }

    /// The caret blink of the focused search; nothing while none is.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.focused_text().map_or(Wake::idle(), |t| t.wake(dt))
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        for (card, rect) in self.cards.iter().zip(&layout[0].cards) {
            gray_card(c, *rect, card.title, theme);
            card.header
                .render(c, Self::bar(*rect, &card.header, theme), theme);
        }
    }

    /// The search whose box has the caret.
    fn focused_search(&mut self) -> Option<&mut Header> {
        self.cards
            .iter_mut()
            .map(|c| &mut c.header)
            .find(|h| h.search().is_some_and(|g| g.input.is_focused()))
    }

    fn focused_text(&mut self) -> Option<&mut TextField> {
        Some(&mut self.focused_search()?.search_mut()?.input)
    }

    /// A reader's focus off every part but the carets. `true` when one
    /// had it.
    fn blur_parts(&mut self) -> bool {
        let mut had = false;
        for card in &mut self.cards {
            if card.header.focus().is_some() {
                card.header.set_focus(None);
                had = true;
            }
        }
        had
    }
}

impl Default for TopbarPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for TopbarPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let Some(group) = group_id(root) else {
            return;
        };
        b.node(&group, Role::Group, "Topbar").parent(root);
        for card in &self.cards {
            card.header.project(b, card.title, &group);
        }
        if let Some(id) = self.cards.iter().find_map(|c| c.header.focused_id()) {
            b.focus(&id);
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let card = self
            .cards
            .iter_mut()
            .find(|c| c.header.sem_id().is_some_and(|h| under(h, &req.target)))
            .ok_or_else(|| format!("the topbar page has no {}", req.target))?;
        card.header.run(req).map(|()| After::Nothing)
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        for card in &mut self.cards {
            let part = card.header.part(id);
            card.header.set_focus(part);
        }
        true
    }

    fn blur(&mut self) -> bool {
        let had =
            self.focused_text().is_some() || self.cards.iter().any(|c| c.header.focus().is_some());
        for card in &mut self.cards {
            card.header.set_focus(None);
        }
        had
    }
}

impl TextHost for TopbarPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.focused_text().is_some_and(|t| t.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.focused_text().is_some_and(|t| t.edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for card in &mut self.cards {
                    if let Some(g) = card.header.search_mut() {
                        dropped |= g.input.end_compose();
                    }
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.focused_text()
            .is_some_and(|t| t.compose(committed, preedit, cursor))
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let layout = self.layout(content, theme);
        self.cards
            .iter()
            .zip(&layout[0].cards)
            .find_map(|(card, rect)| {
                let g = card.header.search().filter(|g| g.input.is_focused())?;
                let bar = Self::bar(*rect, &card.header, theme);
                let at = card.header.layout(bar, theme);
                let i = card.header.slots.iter().position(|s| s.key == "search")?;
                Some(g.ime_area((*at.slots.get(i)?)?, theme))
            })
    }

    /// A right-click on a search box: it takes the keys for its menu.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let layout = self.layout(content, theme);
        for (card, rect) in self.cards.iter_mut().zip(&layout[0].cards) {
            let bar = Self::bar(*rect, &card.header, theme);
            let at = card.header.layout(bar, theme);
            let Some(i) = card.header.slots.iter().position(|s| s.key == "search") else {
                continue;
            };
            let Some(Some(r)) = at.slots.get(i).copied() else {
                continue;
            };
            let on_box = card.header.search().is_some_and(|g| {
                g.layout(r, theme).input.contains(x, y) && !g.on_action(x, y, r, theme)
            });
            if on_box {
                card.header
                    .handle_event(&WidgetEvent::MouseDown { x, y }, bar, theme);
                card.header
                    .handle_event(&WidgetEvent::MouseUp { x, y }, bar, theme);
                return true;
            }
        }
        false
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.focused_text() {
            Some(t) => Keys::Field(t),
            None => Keys::Nobody,
        }
    }

    /// Escape blurs the search with the caret, or lets go of the part a
    /// reader focused.
    fn handle_escape(&mut self) -> bool {
        Component::blur(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The template's five cards in its order, each a header the row's
    /// width less the card's padding, centered in its card.
    #[test]
    fn the_page_is_the_kit_s_five_headers() {
        let page = TopbarPage::new();
        let titles: Vec<&str> = page.cards.iter().map(|c| c.title).collect();
        assert_eq!(
            titles,
            [
                "Simple",
                "Asset",
                "Minimal",
                "Simple with Filter",
                "Show search"
            ]
        );
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1100.0, 900.0);
        for (card, rect) in page
            .cards
            .iter()
            .zip(&page.layout(content, &theme)[0].cards)
        {
            let bar = TopbarPage::bar(*rect, &card.header, &theme);
            assert_eq!(bar.w, rect.w - CARD_PAD_X * 2.0);
            assert!((bar.y + bar.h / 2.0 - (rect.y + rect.h / 2.0)).abs() < 0.01);
            assert!(card.header.sem_id().is_some(), "{}", card.title);
        }
        assert!(page.content_height(content, &theme) > 5.0 * (CARD_PAD_Y * 2.0));
    }

    /// A click in a search puts the caret there and the typed text goes
    /// to it; Escape takes the caret away.
    #[test]
    fn a_search_takes_the_caret_and_the_text() {
        let mut page = TopbarPage::new();
        let theme = Theme::hoff_core();
        let content = Rect::new(0.0, 0.0, 1100.0, 900.0);
        let card = page.layout(content, &theme)[0].cards[0];
        let header = &page.cards[0].header;
        let bar = TopbarPage::bar(card, header, &theme);
        let i = header.slots.iter().position(|s| s.key == "search").unwrap();
        let r = header.layout(bar, &theme).slots[i].unwrap();
        let (x, y) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
        page.handle_event(&WidgetEvent::MouseDown { x, y }, content, &theme);
        page.handle_event(&WidgetEvent::MouseUp { x, y }, content, &theme);
        assert!(page.handle_text("chairs"));
        let text = page.cards[0]
            .header
            .search()
            .map(|g| g.input.text().to_string());
        assert_eq!(text.as_deref(), Some("chairs"));
        assert!(page.handle_escape());
        assert!(!page.handle_text("x"), "no caret after Escape");
    }
}
