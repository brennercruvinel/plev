//! The markers of the hoff core kit, what sits on a canvas where a
//! conversation is anchored: the comment pin (`components/CommentPin`),
//! the people of a thread as overlapping avatars on a pill whose bottom
//! start corner is square (the tail points at the spot), its border the
//! hairline once read and the accent while unread; and the comment cursor
//! (`components/CommentCursor`), the pointer that places one, in its four
//! states: point (bordered paper with a faint plus), pinned (dark),
//! uploading (blue with a white plus) and typing (blue with three dots).
//!
//! Sized by what it holds: a pin is as wide as its avatars overlap, a
//! cursor is the kit's box. The engine's rounded rect has one radius; the
//! pin's square tail is the bubble's (`bubble::tailed_pill`), and the
//! cursor's smaller tail corner is a patch over that corner with its own
//! radius, its gradient continued.
//!
//! Behavior apart from the look: a pin opens its thread on a click or a
//! reader's `activate`, which reads it ([`Marker::open`]); a cursor is an
//! indicator and takes nothing. Nothing visual lives here: every value is
//! `theme.message.marker`, the avatars are the content avatar.
//!
//! With a [`SemId`] ([`Marker::sem`]) the render binds the marker, and
//! [`Marker::project`] declares it.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::bubble::{BubbleSide, tailed_pill};
use super::tokens::CursorShape;
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::Press;
use crate::widgets::content::{Avatar, AvatarSize};
use crate::widgets::recipe::{focus_ring, gradient_fill, rounded_rect, shadow_drops};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, mix};

/// The comment cursor's four states.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorState {
    /// Pointing where a comment would go.
    #[default]
    Point,
    /// Placed, waiting for the words.
    Pinned,
    /// A file on its way.
    Uploading,
    /// Someone writing there.
    Typing,
}

impl CursorState {
    pub fn name(self) -> &'static str {
        match self {
            CursorState::Point => "point",
            CursorState::Pinned => "pinned",
            CursorState::Uploading => "uploading",
            CursorState::Typing => "typing",
        }
    }
}

#[derive(Clone, Debug)]
enum Kind {
    Pin { people: Vec<Avatar>, unread: bool },
    Cursor(CursorState),
}

#[derive(Clone, Debug)]
pub struct Marker {
    kind: Kind,
    press: Press,
    sem: Option<SemId>,
}

/// Where a marker lands: its box at the bounds' origin and, for a pin,
/// each avatar's disc.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerLayout {
    pub rect: Rect,
    pub avatars: Vec<Rect>,
}

impl Marker {
    /// A comment pin holding these people, read.
    pub fn pin<S: Into<String>>(people: impl IntoIterator<Item = S>) -> Self {
        let people = people
            .into_iter()
            .map(|name| Avatar::new(name).size(AvatarSize::Md))
            .collect();
        Self::of(Kind::Pin {
            people,
            unread: false,
        })
    }

    pub fn cursor(state: CursorState) -> Self {
        Self::of(Kind::Cursor(state))
    }

    fn of(kind: Kind) -> Self {
        Self {
            kind,
            press: Press::default(),
            sem: None,
        }
    }

    /// A pin not read yet: the accent border. Nothing on a cursor.
    pub fn unread(mut self, on: bool) -> Self {
        if let Kind::Pin { unread, .. } = &mut self.kind {
            *unread = on;
        }
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn is_pin(&self) -> bool {
        matches!(self.kind, Kind::Pin { .. })
    }

    pub fn is_unread(&self) -> bool {
        matches!(self.kind, Kind::Pin { unread: true, .. })
    }

    pub fn cursor_state(&self) -> Option<CursorState> {
        match self.kind {
            Kind::Cursor(s) => Some(s),
            Kind::Pin { .. } => None,
        }
    }

    pub fn is_hovered(&self) -> bool {
        self.press.hovered
    }

    /// Opens a pin's thread, which reads it. `false` for a cursor or a pin
    /// read already.
    pub fn open(&mut self) -> bool {
        match &mut self.kind {
            Kind::Pin { unread, .. } => std::mem::replace(unread, false),
            Kind::Cursor(_) => false,
        }
    }

    /// Keyboard focus, on a pin (a cursor takes none).
    pub fn set_focused(&mut self, on: bool) {
        self.press.focused = on && self.is_pin();
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    /// The marker's box: a pin as wide as its avatars overlap, a cursor
    /// the kit's.
    pub fn size(&self, theme: &Theme) -> (f32, f32) {
        let t = &theme.message.marker;
        match &self.kind {
            Kind::Pin { people, .. } => {
                let frame = (t.pin_pad + t.pin_border_width) * 2.0;
                let slot = t.avatar + t.avatar_ring * 2.0;
                let n = people.len().max(1) as f32;
                (
                    frame + n * slot - (n - 1.0) * t.avatar_overlap,
                    frame + slot,
                )
            }
            Kind::Cursor(s) => {
                let shape = Self::shape(*s, theme);
                (shape.w, shape.h)
            }
        }
    }

    fn shape(state: CursorState, theme: &Theme) -> &CursorShape {
        let t = &theme.message.marker;
        match state {
            CursorState::Point => &t.point,
            CursorState::Pinned => &t.pinned,
            CursorState::Uploading | CursorState::Typing => &t.active,
        }
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> MarkerLayout {
        let t = &theme.message.marker;
        let (w, h) = self.size(theme);
        let rect = Rect::new(bounds.x, bounds.y, w, h);
        let avatars = match &self.kind {
            Kind::Pin { people, .. } => {
                let inset = t.pin_pad + t.pin_border_width + t.avatar_ring;
                let step = t.avatar + t.avatar_ring * 2.0 - t.avatar_overlap;
                (0..people.len())
                    .map(|i| {
                        Rect::new(
                            rect.x + inset + i as f32 * step,
                            rect.y + inset,
                            t.avatar,
                            t.avatar,
                        )
                    })
                    .collect()
            }
            Kind::Cursor(_) => Vec::new(),
        };
        MarkerLayout { rect, avatars }
    }

    /// A pin takes hover and the click that opens it; a cursor nothing.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        if !self.is_pin() {
            return EventResult::IGNORED;
        }
        let rect = self.layout(bounds, theme).rect;
        let r = self.press.handle(event, rect, false);
        if r.clicked {
            self.open();
        }
        r
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.marker;
        let l = self.layout(bounds, theme);
        let r = l.rect;
        if let Some(id) = &self.sem {
            c.bind(id, [r.x, r.y, r.w, r.h]);
        }
        match &self.kind {
            Kind::Pin { people, unread } => {
                let radius = r.h / 2.0;
                shadow_drops(
                    c,
                    LayerId::DEFAULT,
                    r,
                    radius,
                    std::slice::from_ref(&t.pin_shadow),
                );
                let border = if *unread { t.pin_unread } else { t.pin_read };
                tailed_pill(c, r, radius, border.0, BubbleSide::Start);
                let bw = t.pin_border_width;
                tailed_pill(c, r.inset(bw), radius - bw, t.pin_fill.0, BubbleSide::Start);
                for (person, disc) in people.iter().zip(&l.avatars) {
                    let ring = disc.outset(t.avatar_ring);
                    c.push(rounded_rect(
                        ring.x,
                        ring.y,
                        ring.w,
                        ring.h,
                        ring.w / 2.0,
                        t.pin_fill.0,
                    ));
                    person.render(c, *disc, theme);
                }
                if self.press.focused {
                    c.push(focus_ring(r, radius, theme));
                }
            }
            Kind::Cursor(state) => {
                let s = Self::shape(*state, theme);
                shadow_drops(
                    c,
                    LayerId::DEFAULT,
                    r,
                    s.radius,
                    std::slice::from_ref(&s.shadow),
                );
                if *state == CursorState::Point {
                    let b = t.point_border.0;
                    let bw = t.point_border_width;
                    box_shape(c, r, s.radius, s.tail_radius, [b, b]);
                    let inner = r.inset(bw);
                    let fill = s.fill.map(|f| f.0);
                    box_shape(c, inner, s.radius - bw, (s.tail_radius - bw).max(0.0), fill);
                } else {
                    box_shape(c, r, s.radius, s.tail_radius, s.fill.map(|f| f.0));
                }
                let (cx, cy) = r.center();
                if *state == CursorState::Typing {
                    let total = t.dot * 3.0 + t.dot_gap * 2.0;
                    for i in 0..3 {
                        let x = cx - total / 2.0 + i as f32 * (t.dot + t.dot_gap);
                        let y = cy - t.dot / 2.0;
                        c.push(rounded_rect(x, y, t.dot, t.dot, t.dot / 2.0, s.ink.0));
                    }
                } else {
                    let bar = s.mark * t.mark_bar;
                    let (x0, y0) = (cx - s.mark / 2.0, cy - s.mark / 2.0);
                    c.push(rounded_rect(
                        x0,
                        cy - bar / 2.0,
                        s.mark,
                        bar,
                        bar / 2.0,
                        s.ink.0,
                    ));
                    c.push(rounded_rect(
                        cx - bar / 2.0,
                        y0,
                        bar,
                        s.mark,
                        bar / 2.0,
                        s.ink.0,
                    ));
                }
            }
        }
    }

    /// Declares this marker as `id`. A pin: a `Role::Button` named "Comment
    /// by" its people, with its `unread` state and `activate` (open it),
    /// which reads it. A cursor: a `Role::Image` named "Comment cursor",
    /// described by its state. Returns the node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = match &self.kind {
            Kind::Pin { people, unread } => {
                let names: Vec<&str> = people.iter().map(|a| a.name.as_str()).collect();
                let label = if names.is_empty() {
                    "Comment".to_owned()
                } else {
                    format!("Comment by {}", names.join(", "))
                };
                let mut node = b.node(id, Role::Button, label);
                node.state("unread", *unread);
                node.focusable(true);
                node.action("activate").reads(format!("{id}:unread"));
                node
            }
            Kind::Cursor(state) => {
                let mut node = b.node(id, Role::Image, "Comment cursor");
                node.description(state.name());
                node
            }
        };
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node
    }
}

/// The cursor's box `r`: corners at `radius`, the bottom start one at
/// `tail`, filled top to bottom. The tail corner is a patch the size of
/// the big corner, at its own radius, its gradient the slice of the body's
/// it covers.
fn box_shape(c: &mut Compositor, r: Rect, radius: f32, tail: f32, [top, bottom]: [[f32; 4]; 2]) {
    let radius = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    c.push(gradient_fill(r, radius, top, bottom));
    if radius > tail && r.h > 0.0 {
        let patch = Rect::new(r.x, r.bottom() - radius, radius, radius);
        let from = mix(top, bottom, (r.h - radius) / r.h);
        c.push(gradient_fill(patch, tail, from, bottom));
    }
}
