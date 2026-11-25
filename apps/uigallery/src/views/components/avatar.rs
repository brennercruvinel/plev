//! Avatar page, in the showcase mold: the hoff core kit's avatar on its
//! ladder (the pin's 24 to the profile's 80), without a photo on the
//! upload disc (`UploadAvatar`); offline and online, the dot at every
//! size; the kit's stack of people (`DesignAndAnimationSidebar/Head`),
//! two as the kit draws it and more until the card counts the rest; then
//! the names an avatar meets: six scripts and emoji, and a long one.
//!
//! Projected: a group per title, `avatar_page.group[<g>]`; in it each
//! card as a list named by its chip, `…card[<c>]`; in the list each
//! avatar, `…card[<c>].avatar[<i>]`, an image named by the person with
//! its presence, or a stack, `…card[<c>].stack`, a list of its people
//! (`…stack.avatar[<i>]`). An avatar does nothing; a reader lands on it.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{Avatar, AvatarSize, AvatarStack};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, flow, root_id, showcase, under};

/// The kit's people (`templates/CommentPage`, `QuickComment`).
const PEOPLE: [&str; 12] = [
    "Luna Starfield",
    "Randomdash",
    "Tran Mau Tri Tam",
    "Marina",
    "Joyce",
    "Orion",
    "Kai",
    "Sol",
    "Iris",
    "Noah",
    "Vera",
    "Theo",
];

/// A name far longer than its initial: a reader hears it whole.
const LONG: &str = "Maximiliana Alexandrina von Hohenzollern-Sigmaringen";

/// Six scripts and emoji, one name each (adr-0071).
const SCRIPTS: [&str; 7] = [
    "Ólafur",
    "مريم",
    "שירה",
    "नमिता",
    "王芳",
    "안나",
    "🎉 Party",
];

enum Shown {
    One(Vec<Avatar>),
    Stack(AvatarStack),
}

struct Card {
    title: &'static str,
    shown: Shown,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct AvatarPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The avatar a reader focused: `(group, card, avatar)`.
    focus: Option<(usize, usize, usize)>,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn ones(avatars: impl IntoIterator<Item = Avatar>) -> Shown {
    Shown::One(avatars.into_iter().collect())
}

fn stack(n: usize) -> Shown {
    Shown::Stack(AvatarStack::new(
        PEOPLE
            .iter()
            .take(n)
            .enumerate()
            .map(|(i, p)| Avatar::new(*p).online(i == 0)),
    ))
}

fn card(title: &'static str, shown: Shown) -> Card {
    Card { title, shown }
}

fn groups() -> Vec<Group> {
    let sized = |s: AvatarSize, name: &str| ones([Avatar::new(name).size(s)]);
    let online = AvatarSize::ALL.map(|s| Avatar::new(PEOPLE[0]).size(s).online(true));
    vec![
        Group {
            title: "Size",
            columns: 3,
            cards: vec![
                card("Pin, Xs", sized(AvatarSize::Xs, PEOPLE[0])),
                card("Thread, Sm", sized(AvatarSize::Sm, PEOPLE[1])),
                card("Header, Md", sized(AvatarSize::Md, PEOPLE[2])),
                card("Upload, Lg", sized(AvatarSize::Lg, PEOPLE[3])),
                card("Profile, Xl", sized(AvatarSize::Xl, PEOPLE[4])),
            ],
        },
        Group {
            title: "Presence",
            columns: 3,
            cards: vec![
                card(
                    "Offline",
                    ones([Avatar::new(PEOPLE[5]).size(AvatarSize::Lg)]),
                ),
                card(
                    "Online",
                    ones([Avatar::new(PEOPLE[6]).size(AvatarSize::Lg).online(true)]),
                ),
                card("Online, every size", ones(online)),
            ],
        },
        Group {
            title: "Stack",
            columns: 3,
            cards: vec![
                card("Two", stack(2)),
                card("Five", stack(5)),
                card("Twelve, counted", stack(12)),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card(
                    "Mixed scripts",
                    ones(SCRIPTS.map(|n| Avatar::new(n).size(AvatarSize::Lg))),
                ),
                card(
                    "Long name",
                    ones([Avatar::new(LONG).size(AvatarSize::Lg).online(true)]),
                ),
            ],
        },
    ]
}

impl Shown {
    fn avatars(&self) -> &[Avatar] {
        match self {
            Shown::One(a) => a,
            Shown::Stack(s) => s.people(),
        }
    }

    fn avatars_mut(&mut self) -> &mut [Avatar] {
        match self {
            Shown::One(a) => a,
            Shown::Stack(s) => s.people_mut(),
        }
    }
}

impl AvatarPage {
    pub fn new() -> Self {
        let mut groups = groups();
        let root = root_id("avatar_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    let Some(cid) = card_id(root, g, c) else {
                        continue;
                    };
                    card.shown = match std::mem::replace(&mut card.shown, ones([])) {
                        Shown::One(avatars) => Shown::One(
                            avatars
                                .into_iter()
                                .enumerate()
                                .map(|(i, a)| match child_id(Some(&cid), "avatar") {
                                    Some(f) => a.sem(f.key(i.to_string())),
                                    None => a,
                                })
                                .collect(),
                        ),
                        Shown::Stack(s) => match child_id(Some(&cid), "stack") {
                            Some(id) => Shown::Stack(s.sem(id)),
                            None => Shown::Stack(s),
                        },
                    };
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
                    .shown
                    .avatars()
                    .iter()
                    .position(|a| a.sem_id() == Some(id))?;
                Some((g, c, i))
            })
        })
    }

    fn set_focus(&mut self, at: Option<(usize, usize, usize)>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                for (i, a) in card.shown.avatars_mut().iter_mut().enumerate() {
                    a.set_focused(at == Some((g, c, i)));
                }
            }
        }
    }

    /// A card's avatars flowed across a body `w` wide from the origin, or
    /// its stack as wide as it fits the body.
    fn flowed(card: &Card, w: f32, theme: &Theme) -> (Vec<Rect>, f32) {
        match &card.shown {
            Shown::One(avatars) => {
                let body = Rect::new(0.0, 0.0, w, 0.0);
                let sizes = avatars.iter().map(|a| a.preferred_size(theme));
                flow(body, 0.0, theme.spacing.sm, sizes)
            }
            Shown::Stack(s) => {
                let (sw, sh) = s.preferred_size(theme);
                let fit = s.layout(Rect::new(0.0, 0.0, sw.min(w), sh), theme).rect;
                (vec![Rect::new(0.0, 0.0, fit.w, sh)], sh)
            }
        }
    }

    fn body_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w
    }

    /// Per group: its layout (each card as tall as its avatars), and per
    /// card its avatars' rects (a stack's one), centered in the card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Vec<Rect>>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::body_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(Self::flowed(card, w, theme).1, theme));
            (g.title, g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let rects = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let w = Self::body_w(rect.w, theme);
                        let (rects, h) = Self::flowed(card, w, theme);
                        let used = rects.iter().map(|r| r.right()).fold(0.0, f32::max);
                        let block = showcase::centered((used, h), *rect);
                        rects
                            .into_iter()
                            .map(|r| Rect::new(block.x + r.x, block.y + r.y, r.w, r.h))
                            .collect()
                    })
                    .collect();
                (placed, rects)
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

    /// An avatar takes no pointer; a click lets the reader's focus go.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        _content: Rect,
        _theme: &Theme,
    ) -> EventResult {
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur() {
            return EventResult::changed();
        }
        EventResult::IGNORED
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), rects) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                match &card.shown {
                    Shown::One(avatars) => {
                        for (a, r) in avatars.iter().zip(rects) {
                            a.render(c, *r, theme);
                        }
                    }
                    Shown::Stack(s) => {
                        if let Some(r) = rects.first() {
                            s.render(c, *r, theme);
                        }
                    }
                }
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for AvatarPage {
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
                // a card of avatars is a list of them; a stack's card a
                // group around the stack, the list
                let role = match card.shown {
                    Shown::One(_) => Role::List,
                    Shown::Stack(_) => Role::Group,
                };
                b.node(&cid, role, card.title).parent(&gid);
                match &card.shown {
                    Shown::One(avatars) => {
                        for a in avatars {
                            if let Some(id) = a.sem_id() {
                                a.project(b, id, Some(&cid));
                            }
                        }
                    }
                    Shown::Stack(s) => {
                        if let Some(id) = s.sem_id() {
                            s.project(b, id, Some(&cid), card.title);
                        }
                    }
                }
            }
        }
        let focused = self.focus.and_then(|(g, c, i)| {
            self.groups
                .get(g)?
                .cards
                .get(c)?
                .shown
                .avatars()
                .get(i)?
                .sem_id()
        });
        if let Some(id) = focused {
            b.focus(id);
        }
    }

    /// An avatar declares no action.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        Err(format!(
            "the avatar page has no {} on {}",
            req.action, req.target
        ))
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

    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    /// The width sweep, 280 to 1600 every 8 px, under the core and hoff
    /// and a theme whose controls are larger: every avatar inside its
    /// card's body, every stack in it holding everyone, the cards inside
    /// the content, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = AvatarPage::new();
        let mut large = Theme::hoff_core();
        large.control.heights = large.control.heights.map(|h| h * 1.5);
        for theme in [Theme::hoff_core(), Theme::hoff(), large] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (g, (group, (placed, rects))) in page
                    .groups
                    .iter()
                    .zip(page.layout(content, &theme))
                    .enumerate()
                {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for ((card, rect), rects) in group.cards.iter().zip(&placed.cards).zip(&rects) {
                        let lane = Rect::new(content.x, rect.y, content.w, rect.h);
                        assert!(within(*rect, lane), "{w}: {} out", card.title);
                        let body = showcase::body(*rect, &theme);
                        for r in rects {
                            assert!(within(*r, *rect), "{w}: {} out of its card", card.title);
                            assert!(r.w <= body.w + EPS, "{w}: {} wider", card.title);
                        }
                        if let (Shown::Stack(s), Some(r)) = (&card.shown, rects.first()) {
                            let l = s.layout(*r, &theme);
                            let held = l.people.len() + l.more.map_or(0, |m| m.1);
                            assert_eq!(held, s.people().len(), "{w}: {}", card.title);
                            assert!(within(l.rect, *rect), "{w}: {}", card.title);
                        }
                    }
                }
            }
        }
    }

    /// The ids parse, the reader's focus lands on an avatar (one alone and
    /// one in a stack) and goes, and an avatar is an image with no action.
    #[test]
    fn an_avatar_takes_the_reader_s_focus_and_runs_nothing() {
        let mut page = AvatarPage::new();
        let root = page.root().cloned().expect("the root parses");
        let one = page.groups[0].cards[0].shown.avatars()[0]
            .sem_id()
            .cloned()
            .unwrap();
        let stacked = page.groups[2].cards[1].shown.avatars()[3]
            .sem_id()
            .cloned()
            .unwrap();
        for id in [&one, &stacked] {
            assert!(under(&root, id));
            assert!(page.focus(id));
            assert_eq!(page.at(id), page.focus);
            assert!(page.blur());
        }
        let mut b = SnapshotBuilder::new();
        b.node(&root, Role::Region, "Avatar");
        page.project(&mut b, &root);
        let seen = b.finish().expect("the projection validates");
        for id in [&one, &stacked] {
            let node = seen.get(id).expect("the avatar");
            assert_eq!(node.role, Role::Image);
            assert!(node.actions.is_empty());
        }
        assert_eq!(seen.get(&one).unwrap().label, PEOPLE[0]);
    }
}
