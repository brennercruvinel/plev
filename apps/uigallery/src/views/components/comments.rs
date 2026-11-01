//! Comments page, in the showcase mold, as the hoff core kit's
//! CommentPage: the comment cursor in its four states and the pin read
//! and unread; the comment popups (the new comment, a single comment and
//! a conversation, each thread beside its pin); the new comment in each
//! state its card names (add comment, typing, image uploaded, file
//! uploaded, upload image); the quick comment of an object (the kit's
//! `QuickComment`, shown on its modal page); then the content a thread
//! meets, long text in six scripts with emoji. A card is as tall as what
//! it holds (`showcase::layout_sized`), never less than the kit's `h-50`.
//!
//! The keyboard: Tab walks the parts in the projection's order, the
//! arrows walk a thread's messages, Enter and Space press, Escape lets
//! the focus go; the characters go to the focused line (the page is a
//! text host).
//!
//! Projected: a group per title, `comments_page.group[<g>]`; in it each
//! card as a group named by its chip, `…card[<c>]`; in a card its widget,
//! `…card[<c>].marker`, `.thread`, `.new` or `.quick`, with its parts as
//! the widget declares them.

use accord::{Request, Role, SemId, SnapshotBuilder};
use engine::drawing::compositor::Compositor;
use engine::styling::theme::Theme;
use engine::widgets::content::EmptyState;
use engine::widgets::message::attachment::AttachmentKind;
use engine::widgets::message::comment::CommentThread;
use engine::widgets::message::compose::ComposeLine;
use engine::widgets::message::marker::{CursorState, Marker};
use engine::widgets::message::message::Message;
use engine::widgets::message::new_comment::NewComment;
use engine::widgets::message::quick_comment::QuickComment;
use engine::widgets::prelude::{EventResult, Rect, Wake, WidgetEvent};

use super::{After, Component, EditKey, child_id, root_id, showcase, under};
use crate::views::ImeEvent;
use crate::views::text_input::clipboard::Keys;
use crate::views::text_input::host::TextHost;

mod sample;
#[cfg(test)]
mod tests;

use sample::Sample;

struct Card {
    title: &'static str,
    sample: Sample,
}

struct Group {
    title: &'static str,
    columns: usize,
    cards: Vec<Card>,
}

pub struct CommentsPage {
    root: Option<SemId>,
    groups: Vec<Group>,
}

fn card_id(root: &SemId, g: usize, c: usize) -> Option<SemId> {
    let group = child_id(Some(root), "group")?.key(g.to_string());
    child_id(Some(&group), "card").map(|f| f.key(c.to_string()))
}

const LUNA: &str = "I love this bag, it looks great. Could you make it with leather material? 🤔";
const LONG: &str = "These draggable sliders look really cool. Maybe these could be displayed \
     when you hold shift, to rotate exactly on the X or Y. But by default I don't think we \
     need these controllers, you could just rotate the object by clicking and dragging \
     anywhere as expected on any 3D tool (theoretically).";
const SCRIPTS: &str =
    "Olá! مرحبا بالعالم · שלום עולם · नमस्ते दुनिया · 你好世界 · こんにちは · 안녕하세요 🎉👍🏽";

/// The kit's quick comment (`components/QuickComment`): the shop house
/// icon, Randomdash's comment liked twice, the reply to it. The Modals
/// page shows the same card.
pub(crate) fn kit_quick_comment() -> QuickComment {
    QuickComment::new(
        "Shop House Icon",
        "3D Objects",
        vec![Message::new("Randomdash", "1h", LONG).reaction("2")],
        "Tran Mau Tri Tam",
        "Reply to @randomfash",
    )
    .verified(true)
}

fn luna() -> Message {
    Message::new("Luna Starfield", "2s", LUNA).verified(true)
}

fn card(title: &'static str, sample: Sample) -> Card {
    Card { title, sample }
}

impl Default for CommentsPage {
    fn default() -> Self {
        Self::new()
    }
}

impl CommentsPage {
    pub fn new() -> Self {
        let cursor = |s| Sample::Marker(Marker::cursor(s));
        let thread = |t: CommentThread| Sample::Thread(Box::new(t));
        let new = |n: NewComment| Sample::New(Box::new(n));
        let mut groups = vec![
            Group {
                title: "Comment cursor, pin",
                columns: 4,
                cards: vec![
                    card("Point", cursor(CursorState::Point)),
                    card("Pinned", cursor(CursorState::Pinned)),
                    card("Uploading", cursor(CursorState::Uploading)),
                    card("Typing", cursor(CursorState::Typing)),
                    card(
                        "Conversation",
                        Sample::Marker(Marker::pin(["Luna Starfield", "Sam"])),
                    ),
                    card(
                        "Single comment",
                        Sample::Marker(Marker::pin(["Luna Starfield"]).unread(true)),
                    ),
                ],
            },
            Group {
                title: "Comment Popup",
                columns: 2,
                cards: vec![
                    card(
                        "Add comment, Typing, Image uploaded, File uploaded, Upload image",
                        new(NewComment::new()),
                    ),
                    card(
                        "Single comment",
                        thread(CommentThread::new(vec![luna()], "Sam").unread(true)),
                    ),
                    card(
                        "Conversation",
                        thread(CommentThread::new(
                            vec![
                                luna(),
                                Message::new("Sam", "1s", "@Luna Thank, I’m working on it"),
                            ],
                            "Sam",
                        )),
                    ),
                    card(
                        "Empty: no comments yet",
                        Sample::Empty(Box::new(
                            EmptyState::new(
                                "No comments yet",
                                "Pin a comment to the scene to start a thread.",
                            )
                            .icon("kit-comment"),
                        )),
                    ),
                ],
            },
            Group {
                title: "New comment",
                columns: 2,
                cards: vec![
                    card("Add comment", new(NewComment::new())),
                    card(
                        "Typing",
                        new(NewComment::new().with_text("Love the lighting")),
                    ),
                    card(
                        "Image uploaded",
                        new(NewComment::new()
                            .with_text("Two angles")
                            .with_file("Front", "PNG", AttachmentKind::Image(None))
                            .with_file("Side", "PNG", AttachmentKind::Image(None))),
                    ),
                    card(
                        "File uploaded",
                        new(NewComment::new().with_text("The brief").with_file(
                            "Brief.pdf",
                            "PDF · 1.2 MB",
                            AttachmentKind::File,
                        )),
                    ),
                    card(
                        "Upload image",
                        new(NewComment::new().uploading(true).with_file(
                            "Render",
                            "PNG",
                            AttachmentKind::Image(None),
                        )),
                    ),
                ],
            },
            Group {
                title: "Quick Comment",
                columns: 2,
                cards: vec![card(
                    "Shop House Icon",
                    Sample::Quick(Box::new(kit_quick_comment())),
                )],
            },
            Group {
                title: "Content",
                columns: 2,
                cards: vec![card(
                    "Long text, six scripts with emoji",
                    thread(CommentThread::new(
                        vec![
                            Message::new("Randomdash", "1h", LONG),
                            Message::new("Ana Lucía Fernández-Oliveira", "now", SCRIPTS),
                        ],
                        "Sam",
                    )),
                )],
            },
        ];
        let root = root_id("comments_page");
        if let Some(root) = &root {
            for (g, group) in groups.iter_mut().enumerate() {
                for (c, card) in group.cards.iter_mut().enumerate() {
                    if let Some(cid) = card_id(root, g, c) {
                        card.sample.identify(&cid);
                    }
                }
            }
        }
        Self { root, groups }
    }

    fn samples(&self) -> impl Iterator<Item = &Sample> {
        self.groups.iter().flat_map(|g| &g.cards).map(|c| &c.sample)
    }

    fn samples_mut(&mut self) -> impl Iterator<Item = &mut Sample> {
        self.groups
            .iter_mut()
            .flat_map(|g| &mut g.cards)
            .map(|c| &mut c.sample)
    }

    /// Per group its layout, and per card where its widget goes.
    fn layout(&self, content: Rect, theme: &Theme) -> Vec<(showcase::GroupLayout, Vec<Rect>)> {
        let avail = |w: f32| (w - theme.spacing.md * 2.0).max(0.0);
        let sized = self.groups.iter().map(|g| {
            let w = showcase::card_width(g.columns, content, theme);
            let cards = g
                .cards
                .iter()
                .map(|card| showcase::card_height(card.sample.size(avail(w), theme).1, theme));
            (g.columns, cards.collect())
        });
        showcase::layout_sized(sized, content, theme)
            .into_iter()
            .zip(&self.groups)
            .map(|(placed, group)| {
                let boxes = placed
                    .cards
                    .iter()
                    .zip(&group.cards)
                    .map(|(rect, card)| {
                        showcase::centered(card.sample.size(avail(rect.w), theme), *rect)
                    })
                    .collect();
                (placed, boxes)
            })
            .collect()
    }

    /// Each widget's box, in page order.
    fn boxes(&self, content: Rect, theme: &Theme) -> Vec<Rect> {
        self.layout(content, theme)
            .into_iter()
            .flat_map(|(_, boxes)| boxes)
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

    pub fn wake(&mut self, dt: f32) -> Wake {
        self.samples_mut()
            .fold(Wake::idle(), |w, s| w.merge(s.wake(dt)))
    }

    /// A press lets the keyboard's focus go (a line pressed takes it back
    /// with its caret); then the widget in the card under the pointer
    /// takes it. Every widget takes a move, so a hover can leave.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        let mut result = EventResult::IGNORED;
        if matches!(event, WidgetEvent::MouseDown { .. }) && self.blur_all() {
            result = EventResult::changed();
        }
        let (px, py) = event.pos();
        let every = matches!(event, WidgetEvent::MouseMove { .. });
        let layout = self.layout(content, theme);
        for (group, (placed, boxes)) in self.groups.iter_mut().zip(&layout) {
            for ((card, rect), at) in group.cards.iter_mut().zip(&placed.cards).zip(boxes) {
                if every || rect.contains(px, py) {
                    result = result.merge(card.sample.handle_event(event, *at, theme));
                }
            }
        }
        result
    }

    fn blur_all(&mut self) -> bool {
        let mut had = false;
        for s in self.samples_mut() {
            had |= s.blur();
        }
        had
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        let layout = self.layout(content, theme);
        let last = self.groups.len().saturating_sub(1);
        for (i, (group, (placed, boxes))) in self.groups.iter().zip(&layout).enumerate() {
            showcase::title(c, group.title, content, placed.title_y, theme);
            for ((card, rect), at) in group.cards.iter().zip(&placed.cards).zip(boxes) {
                showcase::card(c, *rect, card.title, theme);
                card.sample.render(c, *at, theme);
            }
            if i != last {
                showcase::rule(c, content, placed.rule_y, theme);
            }
        }
    }

    /// The sample whose line has the keys, with its box.
    fn typing(&mut self) -> Option<&mut Sample> {
        self.samples_mut()
            .find(|s| s.line().is_some_and(ComposeLine::is_focused))
    }

    fn typing_line(&mut self) -> Option<&mut ComposeLine> {
        self.typing().and_then(Sample::line_mut)
    }
}

impl TextHost for CommentsPage {
    fn handle_text(&mut self, s: &str) -> bool {
        self.typing_line().is_some_and(|l| l.insert(s))
    }

    fn handle_edit_key(&mut self, key: EditKey) -> bool {
        if key == EditKey::Tab {
            return false;
        }
        self.typing_line().is_some_and(|l| l.edit(key.into()))
    }

    fn handle_ime(&mut self, ime: ImeEvent<'_>) -> bool {
        let (committed, preedit, cursor) = match ime {
            ImeEvent::Disabled => {
                let mut dropped = false;
                for s in self.samples_mut() {
                    dropped |= s.line_mut().is_some_and(|l| l.end_compose());
                }
                return dropped;
            }
            ImeEvent::Preedit(text, cursor) => ("", text, cursor),
            ImeEvent::Commit(text) => (text, "", None),
        };
        self.typing_line()
            .is_some_and(|l| l.compose(committed, preedit, cursor))
    }

    fn ime_area(&self, content: Rect, theme: &Theme) -> Option<Rect> {
        let boxes = self.boxes(content, theme);
        self.samples()
            .zip(boxes)
            .find(|(s, _)| s.line().is_some_and(ComposeLine::is_focused))
            .and_then(|(s, at)| s.ime_area(at, theme))
    }

    /// A right-click on a line: it takes the keys for its menu, keeping a
    /// selection it has, or taking a caret at the click.
    fn right_click(&mut self, x: f32, y: f32, content: Rect, theme: &Theme) -> bool {
        let boxes = self.boxes(content, theme);
        let hit = self
            .samples()
            .zip(&boxes)
            .position(|(s, at)| s.line_rect(*at, theme).is_some_and(|r| r.contains(x, y)));
        let Some(i) = hit else {
            return false;
        };
        let at = boxes[i];
        let Some(s) = self.samples_mut().nth(i) else {
            return false;
        };
        if s.line()
            .is_some_and(|l| l.is_focused() && l.has_selection())
        {
            return true;
        }
        s.handle_event(&WidgetEvent::MouseDown { x, y }, at, theme);
        s.handle_event(&WidgetEvent::MouseUp { x, y }, at, theme);
        true
    }

    fn keys(&mut self) -> Keys<'_> {
        match self.typing_line() {
            Some(l) => Keys::Field(l.field_mut()),
            None => Keys::Nobody,
        }
    }

    fn handle_escape(&mut self) -> bool {
        self.blur_all()
    }
}

impl Component for CommentsPage {
    fn root(&self) -> Option<&SemId> {
        self.root.as_ref()
    }

    fn project(&self, b: &mut SnapshotBuilder, root: &SemId) {
        let mut focused = None;
        for (g, group) in self.groups.iter().enumerate() {
            let Some(gid) = child_id(Some(root), "group").map(|f| f.key(g.to_string())) else {
                continue;
            };
            b.node(&gid, Role::Group, group.title).parent(root);
            for (c, card) in group.cards.iter().enumerate() {
                let Some(cid) = card_id(root, g, c) else {
                    continue;
                };
                b.node(&cid, Role::Group, card.title).parent(&gid);
                card.sample.project(b, &cid);
                focused = focused.or_else(|| card.sample.focused_id());
            }
        }
        if let Some(id) = focused {
            b.focus(&id);
        }
    }

    fn run(&mut self, req: &Request) -> Result<After, String> {
        let action = req.action.as_str();
        for s in self.samples_mut() {
            if let Some(done) = s.act(&req.target, action, req.param.as_ref()) {
                return done.map(|()| After::Nothing);
            }
        }
        Err(format!(
            "the comments page has no {action} on {}",
            req.target
        ))
    }

    fn focus(&mut self, id: &SemId) -> bool {
        if !self.root.as_ref().is_some_and(|root| under(root, id)) {
            return false;
        }
        self.blur_all();
        for s in self.samples_mut() {
            s.focus(id);
        }
        true
    }

    fn blur(&mut self) -> bool {
        self.blur_all()
    }
}
