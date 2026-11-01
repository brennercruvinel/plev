//! Modals page, the hoff core kit's ModalPage (`templates/ModalPage`,
//! the `modal` route; task-104 front B, onda 3): every modal of the
//! template, in its groups and in the state it shows. Variations, the
//! shared post and profile, the shared file, the quick comment (front C's)
//! beside the delete confirm, Export with its images, video and 3D
//! object, Settings with its five sections. Each is a widget of
//! `engine::widgets::overlay::modals` on `theme.overlay.modals` (the
//! delete confirm is the confirmation `Modal` with its art); the page
//! lays them out in the showcase's groups and cards, each modal centered
//! in its card at the width the card gives it, on the kit's
//! `bg-[#F8F7F7]` where the template fills the card.
//!
//! Projected: a group per title, `modals_page.group[<g>]`, and in it each
//! modal, `modals_page.group[<g>].card[<c>]`, a group named by its title
//! with its controls under it, as the engine projects them. The person's
//! pointer, a reader and an agent reach the same controls; Tab walks
//! them, the arrows move in a tab list and a group of choices, Escape
//! closes an open select and then lets the focus go.

mod cards;
#[cfg(test)]
mod tests;
mod text;

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::Theme;
use engine::widgets::overlay::modals::export::Export;
use engine::widgets::overlay::modals::settings::Settings;
use engine::widgets::overlay::modals::share::{SharePost, ShareProfile};
use engine::widgets::overlay::modals::share_file::{Person, ShareFile};
use engine::widgets::overlay::modals::variations::Variations;
use engine::widgets::overlay::modals::{self, ModalBody};
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::showcase::{self, CARD_H, GroupLayout};
use super::{After, Component, child_id, root_id, under};
use cards::{Confirm, Pending};

/// A modal in its card: the chip naming the states it shows, whether the
/// template fills the card, the widget.
struct Shown {
    chip: &'static str,
    filled: bool,
    body: Box<dyn ModalBody>,
}

struct Group {
    title: &'static str,
    /// Cards per row at full width.
    span: usize,
    cards: Vec<Shown>,
}

pub struct ModalsPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    /// The node a reader or the keyboard focused, held for the projection.
    focus: Option<SemId>,
}

fn group_id(root: Option<&SemId>, g: usize) -> Option<SemId> {
    child_id(root, "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: Option<&SemId>, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn shown(chip: &'static str, filled: bool, body: impl ModalBody + 'static) -> Shown {
    Shown {
        chip,
        filled,
        body: Box::new(body),
    }
}

/// The kit's shared file: two people who can view and edit, the owner.
fn share_file(id: Option<&SemId>) -> ShareFile {
    let people = vec![
        Person::new("Sam Dy", "sam@hoffresearch.com", Some(0)),
        Person::new("Ellie Joy", "ellie@hoffresearch.com", Some(1)),
        Person::new("Hellen", "helen@hoffresearch.com", None),
    ];
    ShareFile::new(people, "https://research-app.co/file/k373nH", id)
}

impl ModalsPage {
    pub fn new() -> Self {
        let root = root_id("modals_page");
        let r = root.as_ref();
        let id = |g, c| card_id(r, g, c);
        let groups = vec![
            Group {
                title: "Variations",
                span: 1,
                cards: vec![shown(
                    "Default, Hover",
                    true,
                    Variations::new(4, id(0, 0).as_ref()),
                )],
            },
            Group {
                title: "Share",
                span: 2,
                cards: vec![
                    shown(
                        "Share a post",
                        true,
                        SharePost::new(
                            "Cute Monster",
                            "randomdash",
                            "3 Jun, 2025",
                            id(1, 0).as_ref(),
                        ),
                    ),
                    shown(
                        "Share profile",
                        true,
                        ShareProfile::new("Sophie Bennett ®", "159 scenes", id(1, 1).as_ref()),
                    ),
                ],
            },
            Group {
                title: "Share file",
                span: 1,
                cards: vec![shown("Share file", false, share_file(id(2, 0).as_ref()))],
            },
            Group {
                title: "Quick comment & Delete confirm",
                span: 2,
                cards: vec![
                    shown(
                        "Comment",
                        true,
                        Pending {
                            name: "QuickComment",
                            owner: "front C, the Comments tab",
                        },
                    ),
                    shown("Confirm", true, Confirm::new(id(3, 1).as_ref())),
                ],
            },
            Group {
                title: "Export",
                span: 1,
                cards: vec![shown(
                    "Export images, Export video, Export 3D",
                    true,
                    Export::new("Robot 2.0", id(4, 0).as_ref()),
                )],
            },
            Group {
                title: "Settings",
                span: 1,
                cards: vec![shown(
                    "General, Profile, Security, Notifications, Subscription",
                    true,
                    Settings::new(id(5, 0).as_ref()),
                )],
            },
        ];
        Self {
            root,
            groups,
            focus: None,
        }
    }

    /// The frame a card keeps around its widget: the chip band at the top
    /// and the body's margin at the bottom.
    fn frame(theme: &Theme) -> f32 {
        CARD_H - showcase::body(Rect::new(0.0, 0.0, 0.0, CARD_H), theme).h
    }

    /// Each card as tall as its modal at the width its body has; a row is
    /// as tall as its tallest card.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<GroupLayout> {
        let frame = Self::frame(theme);
        let sized = self.groups.iter().map(|g| {
            let card_w = showcase::card_width(g.span, content, theme);
            let body_w = showcase::body(Rect::new(0.0, 0.0, card_w, CARD_H), theme).w;
            let heights = g
                .cards
                .iter()
                .map(|s| CARD_H.max(frame + s.body.size(body_w, theme).1))
                .collect();
            (g.span, heights)
        });
        showcase::layout_sized(sized, content, theme)
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        showcase::content_height(&self.layout(content, theme), content)
    }

    /// Where each modal sits: centered in its card's body.
    fn rects(&self, content: Rect, theme: &Theme) -> Vec<Vec<(Rect, Rect)>> {
        let layout = self.layout(content, theme);
        self.groups
            .iter()
            .zip(&layout)
            .map(|(g, laid)| {
                g.cards
                    .iter()
                    .zip(&laid.cards)
                    .map(|(s, card)| {
                        let body = showcase::body(*card, theme);
                        let (w, h) = s.body.size(body.w, theme);
                        (*card, showcase::centered((w, h), body))
                    })
                    .collect()
            })
            .collect()
    }

    fn each_mut(&mut self) -> impl Iterator<Item = &mut Shown> {
        self.groups.iter_mut().flat_map(|g| g.cards.iter_mut())
    }

    /// The pointer: a modal with an open select has it alone; otherwise
    /// every modal sees it at its rect. A press drops a reader's focus.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let rects: Vec<Rect> = self
            .rects(content, theme)
            .into_iter()
            .flatten()
            .map(|(_, m)| m)
            .collect();
        let mut r = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.focus.is_some() {
            self.set_focus(None);
            r = EventResult::changed();
        }
        let open = self.each_mut().position(|s| s.body.has_open());
        for (i, (s, rect)) in self.each_mut().zip(rects).enumerate() {
            if open.is_some_and(|o| o != i) {
                continue;
            }
            r = r.merge(s.body.handle_event(event, rect, theme));
        }
        r
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        let mut moving = false;
        for s in self.each_mut() {
            moving |= s.body.tick(dt);
        }
        Wake::moving(moving)
    }

    pub fn render(&self, c: &mut Compositor, overlay: LayerId, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = layout.len().saturating_sub(1);
        for (g, at) in layout.iter().enumerate() {
            showcase::title(c, self.groups[g].title, content, at.title_y, theme);
            if g != last {
                showcase::rule(c, content, at.rule_y, theme);
            }
        }
        let well = modals::tokens(theme).ink.well;
        for (group, rects) in self.groups.iter().zip(self.rects(content, theme)) {
            for (s, (card, rect)) in group.cards.iter().zip(rects) {
                if s.filled {
                    modals::boxed(c, card, theme.shape.card, well, None);
                }
                showcase::card(c, card, s.chip, theme);
                s.body.render(c, rect, theme);
                s.body.render_overlay(c, overlay, rect, theme);
            }
        }
    }

    fn set_focus(&mut self, id: Option<SemId>) {
        for s in self.each_mut() {
            s.body.focus(id.as_ref());
        }
        self.focus = id;
    }
}

impl Component for ModalsPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = group_id(Some(root), g) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, s) in group.cards.iter().enumerate() {
                if let Some(cid) = card_id(Some(root), g, c) {
                    s.body.project(b, &cid, &gid);
                }
            }
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str().to_owned();
        for s in self.each_mut() {
            if let Some(done) = s.body.run(&req.target, &action, req.param.as_ref()) {
                return done.map(|_| After::Nothing);
            }
        }
        Err(format!("the modals page has no {action} on {}", req.target))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.set_focus(Some(id.clone()));
        true
    }

    /// Escape: an open select closes first; then the focus goes.
    fn blur(&mut self) -> bool {
        let mut closed = false;
        for s in self.each_mut() {
            closed |= s.body.close();
        }
        if closed {
            return true;
        }
        let had = self.focus.is_some();
        self.set_focus(None);
        had
    }
}
