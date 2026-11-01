//! Skeleton page, in the showcase mold: the hoff core kit's scene card
//! skeleton (`SceneCardSkeleton`) as the kit draws it, then the shapes it
//! is made of (lines, a block, a disc on the avatar's step), then the
//! content a skeleton meets: one line and many. They breathe only while a
//! person (or a reader, or an agent) turns it on with a press on any of
//! them, so an idle page settles; with reduced motion they stand still.
//!
//! Projected: a group per title, `skeleton_page.group[<g>]`; in it each
//! card as a group named by its chip, `…card[<c>]`; in it the skeleton,
//! `…card[<c>].skeleton`, a progress bar with no value (busy, as a
//! loading indicator is; not a live region) named by what it stands for,
//! `breathing` or not, whose `activate` turns the breathing on or off as
//! the click does.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::{AvatarSize, Skeleton, SkeletonShape};
use engine::widgets::hit::hit_rect;
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

struct Card {
    title: &'static str,
    /// What it stands for, as a reader hears it.
    stands_for: &'static str,
    skeleton: Skeleton,
    sem: Option<SemId>,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct SkeletonPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    breathing: bool,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, stands_for: &'static str, skeleton: Skeleton) -> Card {
    Card {
        title,
        stands_for,
        skeleton,
        sem: None,
    }
}

fn groups() -> Vec<Group> {
    let lines = |n| Skeleton::new(SkeletonShape::Lines).lines(n);
    vec![
        Group {
            title: "Scene card",
            columns: 2,
            cards: vec![
                card(
                    "Rest, Breathing",
                    "a scene card",
                    Skeleton::new(SkeletonShape::Card),
                ),
                card("Paragraph", "a paragraph", lines(3)),
            ],
        },
        Group {
            title: "Shape",
            columns: 3,
            cards: vec![
                card("Lines", "lines of text", lines(2)),
                card("Block", "an image", Skeleton::new(SkeletonShape::Block)),
                card("Disc", "an avatar", Skeleton::new(SkeletonShape::Disc)),
            ],
        },
        Group {
            title: "Content",
            columns: 2,
            cards: vec![
                card("One line", "a line of text", lines(1)),
                card("Many lines", "a long text", lines(7)),
            ],
        },
    ]
}

impl SkeletonPage {
    pub fn new() -> Self {
        let mut groups = groups();
        let root = root_id("skeleton_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    card.sem = child_id(card_id(root, g, c).as_ref(), "skeleton");
                }
            }
        }
        Self {
            root,
            groups,
            breathing: false,
        }
    }

    /// A skeleton's size in a body `w` wide: a scene card as wide as the
    /// kit's (`w-65`, its image block over the kit's aspect) or the body;
    /// lines across the body; a block at the image's aspect; a disc on
    /// the avatar's `Lg` step.
    fn size(sk: &Skeleton, w: f32, theme: &Theme) -> (f32, f32) {
        let t = &theme.message.skeleton;
        match sk.shape {
            SkeletonShape::Card => {
                let cw = Self::card_w(theme).min(w);
                (cw, sk.height_for(cw, theme).unwrap_or(0.0))
            }
            SkeletonShape::Lines => (w, sk.preferred_height(theme).unwrap_or(0.0)),
            SkeletonShape::Block => {
                let bw = Self::card_w(theme).min(w);
                (bw, bw / t.image_aspect)
            }
            SkeletonShape::Disc => {
                let d = AvatarSize::Lg.px(theme).min(w);
                (d, d)
            }
        }
    }

    /// The kit's scene card width (`w-65`).
    fn card_w(theme: &Theme) -> f32 {
        theme.message.skeleton.card_w
    }

    fn body_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::body(Rect::new(0.0, 0.0, card_w, 0.0), theme).w
    }

    /// Per group: its layout (each card as tall as its skeleton), and per
    /// card its skeleton's rect, centered.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::body_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(Self::size(&card.skeleton, w, theme).1, theme));
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
                        showcase::centered(Self::size(&card.skeleton, w, theme), *rect)
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

    /// A press on any skeleton turns the breathing on or off.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        if let WidgetEvent::MouseDown { x, y } = *event
            && self
                .layout(content, theme)
                .iter()
                .flat_map(|(_, rects)| rects)
                .any(|r| hit_rect(*r, theme).contains(x, y))
        {
            self.breathing = !self.breathing;
            return EventResult::changed();
        }
        EventResult::IGNORED
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.breathing {
            return false;
        }
        // with reduced motion on each stands still and asks for nothing;
        // every one is ticked (no short circuit)
        let mut moved = false;
        for card in self.groups.iter_mut().flat_map(|g| &mut g.cards) {
            moved |= card.skeleton.tick(dt);
        }
        moved
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, rects))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(rects) {
                showcase::card(c, *rect, card.title, theme);
                if let Some(id) = &card.sem {
                    // a line's skeleton is shorter than a target: where a
                    // press lands grows around it (WCAG 2.5.8)
                    let h = hit_rect(*r, theme);
                    c.bind(id, [h.x, h.y, h.w, h.h]);
                }
                card.skeleton.render(c, *r, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for SkeletonPage {
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
                b.node(&cid, Role::Group, card.title).parent(&gid);
                if let Some(id) = &card.sem {
                    b.node(
                        id,
                        Role::ProgressBar,
                        format!("Loading {}", card.stands_for),
                    )
                    .parent(&cid)
                    .state("busy", true)
                    .state("breathing", self.breathing)
                    .action("activate");
                }
            }
        }
    }

    /// A skeleton's `activate` turns the breathing on or off, as a click.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let on_skeleton = self
            .groups
            .iter()
            .flat_map(|g| &g.cards)
            .any(|c| c.sem.as_ref() == Some(&req.target));
        match (on_skeleton, req.action.as_str()) {
            (true, "activate") => {
                self.breathing = !self.breathing;
                Ok(After::Nothing)
            }
            _ => Err(format!(
                "the skeleton page has no {} on {}",
                req.action, req.target
            )),
        }
    }

    fn focus(&mut self, id: &SemId) -> bool {
        self.root.as_ref().is_some_and(|root| under(root, id))
    }

    fn blur(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;
    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    /// Started, the page asks for every frame; with the system's "reduce
    /// motion" on (task-75 phase 2a) it asks for none, and asks again once
    /// the preference is off.
    #[test]
    fn the_page_asks_for_no_frame_with_reduced_motion() {
        let mut page = SkeletonPage::new();
        assert!(!page.tick(FRAME), "at rest until the person starts it");
        page.breathing = true;
        assert!(page.tick(FRAME), "with motion");
        engine::compose::animation::set_reduced_motion(true);
        let moved = page.tick(FRAME);
        engine::compose::animation::set_reduced_motion(false);
        assert!(!moved, "reduced: it stands still");
        assert!(page.tick(FRAME), "motion again");
    }

    /// The width sweep, 280 to 1600 every 8 px, under the core and hoff:
    /// every skeleton inside its card's body, the scene card as tall as
    /// its width keeps the image's aspect, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = SkeletonPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (group, (placed, rects)) in page.groups.iter().zip(page.layout(content, &theme))
                {
                    assert!(placed.title_y >= prev, "{w}: {} climbs", group.title);
                    prev = placed.rule_y;
                    for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(&rects) {
                        let body = showcase::body(*rect, &theme);
                        assert!(within(*r, *rect), "{w}: {} out of its card", card.title);
                        assert!(r.w <= body.w + EPS, "{w}: {} wider", card.title);
                        if card.skeleton.shape == SkeletonShape::Card {
                            assert_eq!(Some(r.h), card.skeleton.height_for(r.w, &theme));
                        }
                    }
                }
            }
        }
    }

    /// Each skeleton is an indeterminate progress bar named by what it
    /// stands for; its
    /// `activate` turns the breathing on, which every one then reports.
    #[test]
    fn a_skeleton_is_busy_and_its_activate_turns_the_breathing() {
        let mut page = SkeletonPage::new();
        let root = page.root().cloned().expect("the root parses");
        let snap = |page: &SkeletonPage| {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Skeleton");
            page.project(&mut b, &root);
            b.finish().expect("the projection validates")
        };
        let id = page.groups[0].cards[0].sem.clone().expect("an id");
        let seen = snap(&page);
        let node = seen.get(&id).expect("the scene card");
        assert_eq!(node.role, Role::ProgressBar);
        assert_eq!(node.label, "Loading a scene card");
        assert!(node.state.get("value").is_none(), "indeterminate");
        let req = Request {
            actor: accord::Actor::Test {
                name: "skeleton".into(),
            },
            target: id.clone(),
            action: accord::ActionName::parse("activate").expect("an action name"),
            param: None,
            observed: accord::Observation {
                revision: seen.revision(),
                values: Vec::new(),
            },
            ask: None,
        };
        assert!(page.run(&req).is_ok());
        assert!(page.breathing);
        let breathing = |seen: &accord::Snapshot| {
            seen.get(&id)
                .and_then(|n| n.state.get("breathing").cloned())
        };
        assert_eq!(breathing(&seen), Some(accord::Value::Bool(false)));
        assert_eq!(breathing(&snap(&page)), Some(accord::Value::Bool(true)));
    }
}
