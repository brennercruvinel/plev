//! Attachment page, in the showcase mold: the attachments of the hoff
//! core kit (its image preview tiles with the remove button, a file tile
//! in the language of its add-files menu, the add button), one tile per
//! state, trays as the kit's comment composer holds them, then the
//! content a tray meets: a long name, many tiles wrapping, six scripts. A
//! card is as tall as its tray measures (`showcase::layout_sized`), never
//! less than the kit's `h-50`.
//!
//! A click on a remove button removes the tile; a click on the add button
//! adds a picture (the page stands in for the picker the kit's menu
//! opens).
//!
//! Projected: a group per title, `attachment_page.group[<g>]`; in it each
//! card as the tray's list, named by its chip, `…card[<c>]`; in the list
//! each attachment, `…card[<c>][<key>]`, its remove button,
//! `…[<key>].remove`, and the add button, `…card[<c>].add`.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::drawing::gpu::image::{ImageHandle, load_image_rgba};
use engine::styling::theme::Theme;
use engine::widgets::message::attachment::{AttachmentKind, AttachmentTray, TrayFocus};
use engine::widgets::prelude::{EventResult, Rect, WidgetEvent};

use super::{After, Component, child_id, root_id, showcase, under};

/// A procedural photo for a tile: a warm diagonal wash, `seed` turning
/// its hue, so the pictures read as different without shipping assets.
fn photo(seed: f32) -> Option<ImageHandle> {
    let n = 60u32;
    let mut pixels = Vec::with_capacity((n * n * 4) as usize);
    for y in 0..n {
        for x in 0..n {
            let t = (x + y) as f32 / (2 * n) as f32;
            let hue = (seed + t * 0.25).fract() * std::f32::consts::TAU;
            let channel = |shift: f32| ((hue + shift).cos() * 0.5 + 0.5) * 0.55 + 0.25;
            let [r, g, b] = [channel(0.0), channel(2.1), channel(4.2)].map(|c| (c * 255.0) as u8);
            pixels.extend_from_slice(&[r, g, b, 255]);
        }
    }
    load_image_rgba(n, n, pixels).ok()
}

fn picture(i: usize) -> AttachmentKind {
    AttachmentKind::Image(photo(i as f32 * 0.17))
}

const LONG: &str = "Minimalist Architecture Scene, final review with the lighting notes.pdf";

struct Card {
    title: &'static str,
    tray: AttachmentTray,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

/// A focusable part of the page: a part of one card's tray.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Spot {
    g: usize,
    c: usize,
    part: TrayFocus,
}

pub struct AttachmentPage {
    root: Option<SemId>,
    groups: Vec<Group>,
    focus: Option<Spot>,
    /// Pictures added so far: the next one's look.
    added: usize,
}

fn group_id(root: &SemId, g: usize) -> Option<SemId> {
    child_id(Some(root), "group").map(|family| family.key(g.to_string()))
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    child_id(group_id(root, g).as_ref(), "card").map(|family| family.key(c.to_string()))
}

fn card(title: &'static str, tray: AttachmentTray) -> Card {
    Card { title, tray }
}

fn pictures(n: usize) -> AttachmentTray {
    (0..n).fold(AttachmentTray::new(), |t, i| {
        t.with(&format!("Uploaded {}", i + 1), "PNG", picture(i))
    })
}

impl AttachmentPage {
    pub fn new() -> Self {
        let file = || AttachmentTray::new().with("Brief.pdf", "PDF, 3.4 MB", AttachmentKind::File);
        let mut groups = vec![
            Group {
                title: "Tile",
                columns: 2,
                cards: vec![
                    card("Picture, Focus", pictures(1)),
                    card("Picture, Hover, Touch", pictures(1).show_removes(true)),
                    card("File", file()),
                    card(
                        "3D object",
                        AttachmentTray::new().with(
                            "Classic Car.glb",
                            "3D object, 12 MB",
                            AttachmentKind::Object,
                        ),
                    ),
                ],
            },
            Group {
                title: "Tray",
                columns: 2,
                cards: vec![
                    card("Pictures, Add", pictures(3).with_add()),
                    card(
                        "Mixed, Add",
                        pictures(1)
                            .with("Brief.pdf", "PDF, 3.4 MB", AttachmentKind::File)
                            .with("Classic Car.glb", "3D object", AttachmentKind::Object)
                            .with_add(),
                    ),
                    card("Empty, Add", AttachmentTray::new().with_add()),
                    card(
                        "Loading picture",
                        AttachmentTray::new().with(
                            "Uploading 1",
                            "PNG, 40%",
                            AttachmentKind::Image(None),
                        ),
                    ),
                ],
            },
            Group {
                title: "Content",
                columns: 2,
                cards: vec![
                    card(
                        "Long name",
                        AttachmentTray::new().with(LONG, "PDF, 3.4 MB", AttachmentKind::File),
                    ),
                    card("Many, wrapping", pictures(12).with_add()),
                    card(
                        "Mixed scripts",
                        AttachmentTray::new()
                            .with("تقرير المشروع.pdf", "PDF · مستند", AttachmentKind::File)
                            .with("设计稿 🎨.png", "PNG · 图片", AttachmentKind::File),
                    ),
                ],
            },
        ];
        let root = root_id("attachment_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(id) = card_id(root, g, c) {
                        card.tray = std::mem::take(&mut card.tray).sem(id);
                    }
                }
            }
        }
        Self {
            root,
            groups,
            focus: None,
            added: 0,
        }
    }

    fn at(&self, id: &SemId) -> Option<Spot> {
        self.groups.iter().enumerate().find_map(|(g, group)| {
            group
                .cards
                .iter()
                .enumerate()
                .find_map(|(c, card)| card.tray.target(id).map(|part| Spot { g, c, part }))
        })
    }

    fn tray_mut(&mut self, g: usize, c: usize) -> Option<&mut AttachmentTray> {
        Some(&mut self.groups.get_mut(g)?.cards.get_mut(c)?.tray)
    }

    /// What the add button asks for: a picture at the end of the tray.
    fn add_to(&mut self, g: usize, c: usize) -> bool {
        let n = self.added;
        let Some(tray) = self.tray_mut(g, c) else {
            return false;
        };
        if !tray.has_add() {
            return false;
        }
        tray.push(format!("Added {}", n + 1), "PNG", picture(n + 5));
        self.added += 1;
        true
    }

    fn set_focus(&mut self, at: Option<Spot>) {
        self.focus = at;
        for (g, group) in self.groups.iter_mut().enumerate() {
            for (c, card) in group.cards.iter_mut().enumerate() {
                let part = at.filter(|s| (s.g, s.c) == (g, c)).map(|s| s.part);
                card.tray.set_focus(part);
            }
        }
    }

    /// The width a card's tray takes: the card's inside, up to a readable
    /// column.
    fn tray_w(card_w: f32, theme: &Theme) -> f32 {
        showcase::room(card_w, theme).min(theme.size.readable_max_w)
    }

    /// Per group its layout, and per card the tray's rect, centered.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let sized = self.groups.iter().map(|g| {
            let w = Self::tray_w(showcase::card_width(g.columns, content, theme), theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(card.tray.height(w, theme), theme));
            (g.title, g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let trays = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        let w = Self::tray_w(rect.w, theme);
                        showcase::centered((w, card.tray.height(w, theme)), *rect)
                    })
                    .collect();
                (placed, trays)
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

    /// A click lets the reader's focus go; each tray takes the click at
    /// its rect, and an add button's click adds a picture.
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
        let mut asked = Vec::new();
        for (g, (group, (_, trays))) in self.groups.iter_mut().zip(&layout).enumerate() {
            for (c, (card, r)) in group.cards.iter_mut().zip(trays).enumerate() {
                result = result.merge(card.tray.handle_event(event, *r, theme));
                if card.tray.take_add_request() {
                    asked.push((g, c));
                }
            }
        }
        for (g, c) in asked {
            self.add_to(g, c);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, trays))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(trays) {
                showcase::card(c, *rect, card.title, theme);
                card.tray.render(c, *r, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }
}

impl Component for AttachmentPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = group_id(root, g) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for card in &group.cards {
                if let Some(id) = card.tray.sem_id() {
                    card.tray.project(b, id, card.title, Some(&gid));
                }
            }
        }
        let focused = self.focus.and_then(|s| {
            let tray = self.groups.get(s.g)?.cards.get(s.c)?.tray.sem_id()?;
            Some(match s.part {
                TrayFocus::Add => child_id(Some(tray), "add")?,
                TrayFocus::Tile(k) => tray.key(k.to_string()),
                TrayFocus::Remove(k) => child_id(Some(&tray.key(k.to_string())), "remove")?,
            })
        });
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    /// A remove button's `activate` removes its tile; the add button's
    /// adds a picture, as their clicks do.
    fn run(&mut self, req: &Request) -> Result<After, String> {
        let spot = self
            .at(&req.target)
            .filter(|_| req.action.as_str() == "activate");
        let done = match spot {
            Some(Spot {
                g,
                c,
                part: TrayFocus::Remove(k),
            }) => self.tray_mut(g, c).is_some_and(|t| t.remove(k)),
            Some(Spot {
                g,
                c,
                part: TrayFocus::Add,
            }) => self.add_to(g, c),
            _ => false,
        };
        if done {
            Ok(After::Nothing)
        } else {
            Err(format!(
                "the attachment page has no {} on {}",
                req.action, req.target
            ))
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
    use accord::Actor;

    const EPS: f32 = 0.5;

    fn within(inner: Rect, outer: Rect) -> bool {
        inner.x >= outer.x - EPS
            && inner.y >= outer.y - EPS
            && inner.right() <= outer.right() + EPS
            && inner.bottom() <= outer.bottom() + EPS
    }

    /// The width sweep, 280 to 1600 every 8 px: every tray and every tile
    /// in its card, every card in the content, the groups down the page.
    #[test]
    fn the_page_holds_over_the_whole_width_range() {
        let page = AttachmentPage::new();
        for theme in [Theme::hoff_core(), Theme::hoff()] {
            for w in (280..=1600).step_by(8) {
                let content = Rect::new(24.0, 0.0, w as f32, 800.0);
                let mut prev = content.y;
                for (g, (group, (placed, trays))) in page
                    .groups
                    .iter()
                    .zip(page.layout(content, &theme))
                    .enumerate()
                {
                    assert!(placed.title_y >= prev, "{w}: group {g} climbs");
                    prev = placed.rule_y;
                    for ((card, rect), r) in group.cards.iter().zip(&placed.cards).zip(&trays) {
                        let lane = Rect::new(content.x, rect.y, content.w, rect.h);
                        assert!(
                            within(*rect, lane),
                            "{w}: {} out of the content",
                            card.title
                        );
                        assert!(
                            within(*r, *rect),
                            "{w}: {} tray out of its card",
                            card.title
                        );
                        assert!(r.w >= card.tray.min_width(&theme), "{w}: {}", card.title);
                        let l = card.tray.layout(*r, &theme);
                        for t in &l.tiles {
                            assert!(within(t.rect, *rect), "{w}: {} tile out", card.title);
                            assert!(within(t.remove, *rect), "{w}: {} remove out", card.title);
                        }
                    }
                }
            }
        }
    }

    /// The ids parse and map back; the reader's focus lands and goes; a
    /// remove's `activate` removes and the add's adds; a tile runs
    /// nothing.
    #[test]
    fn remove_and_add_run_from_the_projection() {
        let mut page = AttachmentPage::new();
        let root = page.root().cloned().expect("the root parses");
        let tray = page.groups[1].cards[0].tray.sem_id().cloned().unwrap();
        let tile = tray.key("0");
        let remove = tile.child("remove").unwrap();
        let add = tray.child("add").unwrap();
        assert!(under(&root, &remove));
        assert!(page.focus(&remove));
        assert!(page.blur());

        let seen = {
            let mut b = SnapshotBuilder::new();
            b.node(&root, Role::Region, "Attachment");
            page.project(&mut b, &root);
            b.finish().expect("the projection validates")
        };
        let ask = |target: &SemId| {
            let actor = Actor::Test {
                name: "attachment page".to_owned(),
            };
            Request::observe(&seen, actor, target.as_str(), "activate", None)
        };
        assert_eq!(page.run(&ask(&remove).unwrap()), Ok(After::Nothing));
        assert_eq!(page.groups[1].cards[0].tray.items().len(), 2);
        assert_eq!(page.run(&ask(&add).unwrap()), Ok(After::Nothing));
        assert_eq!(page.groups[1].cards[0].tray.items().len(), 3);
        assert!(ask(&tile).is_err(), "a tile declares no action");
        let mut stray = ask(&add).unwrap();
        stray.target = page.groups[0].cards[0].tray.sem_id().cloned().unwrap();
        assert!(page.run(&stray).is_err());
    }
}
