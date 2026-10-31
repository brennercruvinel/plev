//! The message of the hoff core kit: an avatar, the author and the time
//! on one line, the body wrapped under them, and, in the quick layout,
//! the like chip under the body. Two layouts of the kit
//! (`theme.message`): the thread row (`Comment/Message`, a hairline
//! under it) and the quick comment (`QuickComment/Comment`, the body
//! clamped to three lines).
//!
//! Sized by its content: the row takes the width it is given and grows
//! as tall as the measured body, never a fixed height. The text column
//! is what the width leaves after the padding and the avatar; the body
//! wraps against it with the one style it is drawn with. On a narrow
//! column the author is ellipsized before the time is, so the time
//! stays whole; a clamped body is cut by `TextMeasurer::clamp_to_lines`
//! and is still read whole by the projection.
//!
//! Nothing visual lives here: the box model and the runs come from
//! `theme.message`, the like chip is the labeled button's outline
//! recipe at its `Xs` step, the avatar is the content avatar.
//!
//! With a [`SemId`] ([`Message::sem`]) the render binds the row it
//! draws, the like chip binds `<id>.reaction`, and [`Message::project`]
//! declares both.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::tokens::{MessageStyle, MessageText};
use crate::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use crate::drawing::gpu::image::ImageHandle;
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Button, ButtonSize, ButtonVariant};
use crate::widgets::content::{Avatar, AvatarSize};
use crate::widgets::icons;
use crate::widgets::recipe::focus_ring_inside;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// Which layout of the kit a message takes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageKind {
    /// The row of a comment thread, a hairline under it.
    #[default]
    Thread,
    /// The compact comment: muted body clamped, the like chip under it.
    Quick,
}

impl MessageKind {
    pub fn style(self, theme: &Theme) -> &MessageStyle {
        match self {
            MessageKind::Thread => &theme.message.thread,
            MessageKind::Quick => &theme.message.quick,
        }
    }
}

/// A run of text where it is drawn: the text as drawn (ellipsized or
/// clamped) and its box. For the body the box width is the width it
/// wraps at.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub text: String,
    pub rect: Rect,
}

/// Where every part of a message lands, from its bounds alone.
#[derive(Clone, Debug, PartialEq)]
pub struct MessageLayout {
    /// The whole row, padding and hairline included: as tall as the
    /// content, as wide as the bounds.
    pub row: Rect,
    pub avatar: Rect,
    pub name: Option<Placed>,
    /// The seal after the name, when the author is verified.
    pub seal: Option<Rect>,
    pub time: Option<Placed>,
    pub body: Option<Placed>,
    pub reaction: Option<Rect>,
    pub divider: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct Message {
    author: String,
    time: String,
    body: String,
    kind: MessageKind,
    avatar: Avatar,
    divider: bool,
    verified: bool,
    reaction: Option<Button>,
    sem: Option<SemId>,
    focused: bool,
}

impl Message {
    pub fn new(
        author: impl Into<String>,
        time: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        let author = author.into();
        Self {
            // The disc is as large as the row's token; `Md` only bounds
            // it, the avatar never exceeds the rect it is given.
            avatar: Avatar::new(author.clone()).size(AvatarSize::Md),
            author,
            time: time.into(),
            body: body.into(),
            kind: MessageKind::default(),
            divider: false,
            verified: false,
            reaction: None,
            sem: None,
            focused: false,
        }
    }

    pub fn kind(mut self, kind: MessageKind) -> Self {
        self.kind = kind;
        self
    }

    /// The hairline under the row (`border-b`), as in a thread.
    pub fn divider(mut self, on: bool) -> Self {
        self.divider = on;
        self
    }

    /// The author verified: the kit's seal after the name (`✪`), drawn
    /// as the `badge-check` icon one name line tall.
    pub fn verified(mut self, on: bool) -> Self {
        self.verified = on;
        self
    }

    pub fn is_verified(&self) -> bool {
        self.verified
    }

    /// The author's photo; without one the avatar shows the initial.
    pub fn image(mut self, image: ImageHandle) -> Self {
        self.avatar = self.avatar.image(image);
        self
    }

    /// The like chip under the body, showing `count`: the kit's
    /// `👍 2`, as the outline button with the thumbs-up icon.
    pub fn reaction(mut self, count: impl Into<String>) -> Self {
        let mut chip = Button::new(count)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Xs)
            .icon("thumbs-up");
        if let Some(id) = self.sem.as_ref().and_then(reaction_id) {
            chip = chip.sem(id);
        }
        self.reaction = Some(chip);
        self
    }

    /// The like chip disabled (no effect without one).
    pub fn reaction_disabled(mut self, on: bool) -> Self {
        self.reaction = self.reaction.map(|b| b.disabled(on));
        self
    }

    /// The logical identity the render binds the row to; the like chip
    /// takes `<id>.reaction`.
    pub fn sem(mut self, id: SemId) -> Self {
        if let Some(rid) = reaction_id(&id) {
            self.reaction = self.reaction.map(|b| b.sem(rid));
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The like chip's id, when the message has a chip and an id.
    pub fn reaction_sem_id(&self) -> Option<&SemId> {
        self.reaction.as_ref()?.sem_id()
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn time(&self) -> &str {
        &self.time
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn message_kind(&self) -> MessageKind {
        self.kind
    }

    pub fn has_reaction(&self) -> bool {
        self.reaction.is_some()
    }

    /// Keyboard focus on the row, driven by the owning view: the focus
    /// ring inside the row.
    pub fn set_focused(&mut self, on: bool) {
        self.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Keyboard focus on the like chip (a disabled chip refuses it).
    pub fn set_reaction_focused(&mut self, on: bool) {
        if let Some(chip) = &mut self.reaction {
            chip.set_focused(on);
        }
    }

    /// The narrowest row whose parts still fit side by side: padding,
    /// avatar, gap and the like chip. Below it the column is empty.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let s = self.kind.style(theme);
        let chip = self
            .reaction
            .as_ref()
            .map_or(0.0, |b| b.preferred_size(theme).0);
        s.pad * 2.0 + s.avatar + s.gap + chip
    }

    /// The row's height at `width`: the content measured, never fixed.
    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, width, 0.0), theme).row.h
    }

    /// Where every part lands in `bounds` (only its origin and width
    /// count; the row is as tall as its content).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> MessageLayout {
        let s = self.kind.style(theme);
        let ty = &theme.typography;
        let top = bounds.y + s.pad;
        let avatar = Rect::new(bounds.x + s.pad, top, s.avatar, s.avatar);
        let col_x = avatar.right() + s.gap;
        let col_w = (bounds.right() - s.pad - col_x).max(0.0);

        // The name line: the time keeps its width, the name the rest.
        let name_style = s.name.style(ty);
        let time_style = s.time.style(ty);
        let time = fit(&self.time, &time_style, col_w);
        let time_w = width(&time, &time_style);
        let time_room = if time.is_empty() {
            0.0
        } else {
            s.time_gap + time_w
        };
        // The seal keeps its place after the name, as the time does.
        let seal_edge = name_style.line_height;
        let seal_room = if self.verified {
            s.seal_gap + seal_edge
        } else {
            0.0
        };
        let seal_fits = self.verified && col_w >= time_room + seal_room;
        let name_room = (col_w - time_room - if seal_fits { seal_room } else { 0.0 }).max(0.0);
        let name = fit(&self.author, &name_style, name_room);
        let name_w = width(&name, &name_style);
        let seal = seal_fits.then(|| {
            let x = if name.is_empty() {
                col_x
            } else {
                col_x + name_w + s.seal_gap
            };
            Rect::new(x, top, seal_edge, seal_edge)
        });
        let after_name = match (&seal, name.is_empty()) {
            (Some(r), _) => Some(r.right()),
            (None, false) => Some(col_x + name_w),
            (None, true) => None,
        };
        let time_x = after_name.map_or(col_x, |x| x + s.time_gap);
        let name = placed(name, col_x, top, name_w, &name_style);
        let time = placed(time, time_x, top, time_w, &time_style);
        let line_h = [&name, &time]
            .into_iter()
            .flatten()
            .map(|p| p.rect.h)
            .chain(seal.map(|r| r.h))
            .fold(0.0, f32::max);
        let mut y = top + line_h;

        // The body, wrapped against the column (and clamped).
        let body_style = s.body.style(ty);
        let text = match s.body_lines {
            _ if col_w <= 0.0 => String::new(),
            Some(lines) => TextMeasurer::clamp_to_lines(&self.body, &body_style, col_w, lines),
            None => self.body.clone(),
        };
        let body = (!text.is_empty()).then(|| {
            if line_h > 0.0 {
                y += s.body_gap;
            }
            let (_, h) = TextMeasurer::measure_styled(&text, &body_style, Some(col_w));
            let rect = Rect::new(col_x, y, col_w, h);
            y += h;
            Placed { text, rect }
        });

        // The like chip under it.
        let reaction = self.reaction.as_ref().map(|chip| {
            if y > top {
                y += s.reaction_gap;
            }
            let (w, h) = chip.preferred_size(theme);
            let rect = Rect::new(col_x, y, w, h);
            y += h;
            rect
        });

        let hairline = if self.divider {
            theme.message.divider_width
        } else {
            0.0
        };
        let bottom = avatar.bottom().max(y) + s.pad + hairline;
        let row = Rect::new(bounds.x, bounds.y, bounds.w, bottom - bounds.y);
        let divider = self
            .divider
            .then(|| Rect::new(bounds.x, bottom - hairline, bounds.w, hairline));
        MessageLayout {
            row,
            avatar,
            name,
            seal,
            time,
            body,
            reaction,
            divider,
        }
    }

    /// Forwards pointer events to the like chip at its rect.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let Some(rect) = self.layout(bounds, theme).reaction else {
            return EventResult::IGNORED;
        };
        match &mut self.reaction {
            Some(chip) => chip.handle_event(event, rect),
            None => EventResult::IGNORED,
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [l.row.x, l.row.y, l.row.w, l.row.h]);
        }
        let s = self.kind.style(theme);
        let ty = &theme.typography;
        self.avatar.render(c, l.avatar, theme);
        run(c, l.name.as_ref(), &s.name, ty, None);
        if let Some(r) = l.seal
            && let Some(node) = icons::icon_at("badge-check", r.w, s.name.color.0, r.x, r.y)
        {
            c.push(node);
        }
        run(c, l.time.as_ref(), &s.time, ty, None);
        let wrap = l.body.as_ref().map(|b| b.rect.w);
        run(c, l.body.as_ref(), &s.body, ty, wrap);
        if let (Some(chip), Some(rect)) = (&self.reaction, l.reaction) {
            chip.render(c, rect, theme);
        }
        if let Some(d) = l.divider {
            c.push(SceneNode::Rect {
                x: d.x,
                y: d.y,
                w: d.w,
                h: d.h,
                color: theme.message.divider.0,
            });
        }
        if self.focused {
            c.push(focus_ring_inside(l.row, theme.shape.micro, theme));
        }
    }

    /// Declares this message as `id`: a `Role::ListItem` named by the
    /// author and the time, described by the whole body (a clamped body
    /// is read whole), focusable so a reader can land on it; with a like
    /// chip, its button `<id>.reaction` under it, described "like". Pass
    /// the id given to [`Message::sem`] so the nodes and the bindings
    /// name the same parts. Returns the row's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        if let (Some(chip), Some(rid)) = (&self.reaction, self.reaction_sem_id()) {
            chip.project(b, rid, Some(id)).description("like");
        }
        let label = [self.author.as_str(), self.time.as_str()]
            .into_iter()
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        let mut node = b.node(id, Role::ListItem, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        if !self.body.is_empty() {
            node.description(self.body.as_str());
        }
        if self.verified {
            node.state("verified", true);
        }
        node.focusable(true);
        node
    }
}

/// `<id>.reaction`, the like chip's id under its message.
fn reaction_id(id: &SemId) -> Option<SemId> {
    id.child("reaction").ok()
}

pub(crate) fn width(text: &str, style: &TextStyle) -> f32 {
    TextMeasurer::measure_styled(text, style, None).0
}

/// `text` on one line of `room`, ellipsized; empty when not even the
/// ellipsis fits, so nothing is drawn past the column.
pub(crate) fn fit(text: &str, style: &TextStyle, room: f32) -> String {
    if text.is_empty() || room <= 0.0 {
        return String::new();
    }
    let out = TextMeasurer::truncate_to_width(text, style, room);
    if width(&out, style) <= room {
        out
    } else {
        String::new()
    }
}

pub(crate) fn placed(text: String, x: f32, y: f32, w: f32, style: &TextStyle) -> Option<Placed> {
    (!text.is_empty()).then(|| Placed {
        text,
        rect: Rect::new(x, y, w, style.line_height),
    })
}

/// One run drawn with the style it was measured with.
pub(crate) fn run(
    c: &mut Compositor,
    at: Option<&Placed>,
    spec: &MessageText,
    ty: &crate::styling::theme::TypographyScale,
    wrap: Option<f32>,
) {
    if let Some(p) = at {
        let style = spec.style(ty);
        c.push(SceneNode::Text {
            key: TextNodeKey::from_style(&p.text, &style, wrap),
            x: p.rect.x,
            y: p.rect.y,
            color: spec.color.0,
        });
    }
}
