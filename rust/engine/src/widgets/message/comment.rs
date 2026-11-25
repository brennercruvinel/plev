//! The comment thread of the hoff core kit (`components/Comment`): the
//! pin of the people who wrote, and beside it the popup with their
//! messages (`Comment/Message`), a dark close disc on its corner and the
//! reply under them (`Comment/NewMessage`: the replier's avatar, the line
//! and the submit button in a well).
//!
//! Sized by its content: the popup is the kit's `w-70` or what the width
//! leaves after the pin, as tall as its messages measure and its reply;
//! closed, the thread is its pin alone.
//!
//! Behavior apart from the look: the pin opens and closes the popup and
//! reads it; the close disc closes it; the line edits as a field
//! ([`ComposeLine`]) and the submit button sends it, a message of the
//! replier's at the end of the thread. The surface is the scroller's
//! tokens (the same popup of the kit), the rows the message's, the pin
//! the marker's, the rest `theme.message.comment`.
//!
//! Its parts take `<id>.pin`, `.close`, `.messages[<i>]`, `.reply` and
//! `.send`; [`CommentThread::project`] declares them and
//! [`CommentThread::act`] runs what a reader or an agent asks.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder, Value};

use super::compose::{ComposeLine, LineLook};
use super::marker::Marker;
use super::message::Message;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{Press, SubmitButton, SubmitButtonSize};
use crate::widgets::content::{Avatar, AvatarSize};
use crate::widgets::hit::hit_rect;
use crate::widgets::icons;
use crate::widgets::recipe::{
    focus_ring, gradient_fill, rounded_rect, shadow_drops, shadow_insets,
};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// A part of the thread a reader can land on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThreadPart {
    Pin,
    Close,
    Message(usize),
    Reply,
    Send,
}

/// Where every part lands, from the bounds' origin and width.
#[derive(Clone, Debug, PartialEq)]
pub struct ThreadLayout {
    pub pin: Rect,
    /// The popup and what is in it, while open.
    pub popup: Option<Rect>,
    pub close: Rect,
    pub rows: Vec<Rect>,
    pub avatar: Rect,
    pub well: Rect,
    pub line: Rect,
    pub send: Rect,
}

#[derive(Clone, Debug)]
pub struct CommentThread {
    pin: Marker,
    messages: Vec<Message>,
    reply: ComposeLine,
    replier: Avatar,
    /// The kit's SubmitButton at its `isMedium` step.
    send: SubmitButton,
    send_focused: bool,
    close: Press,
    open: bool,
    sem: Option<SemId>,
}

impl CommentThread {
    /// A thread of `messages`, the reply written by `replier`.
    pub fn new(messages: Vec<Message>, replier: &str) -> Self {
        let mut thread = Self {
            pin: Marker::pin(Vec::<String>::new()),
            messages: messages.into_iter().map(|m| m.divider(true)).collect(),
            reply: ComposeLine::new("Reply"),
            replier: Avatar::new(replier).size(AvatarSize::Md),
            send: SubmitButton::new().size(SubmitButtonSize::Md),
            send_focused: false,
            close: Press::default(),
            open: true,
            sem: None,
        };
        thread.pin = Marker::pin(thread.people());
        thread
    }

    /// The people who wrote, each once, in order.
    fn people(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for m in &self.messages {
            if !out.iter().any(|p| p == m.author()) {
                out.push(m.author().to_owned());
            }
        }
        out
    }

    /// Not read yet: the pin's accent border.
    pub fn unread(mut self, on: bool) -> Self {
        self.pin = self.pin.unread(on);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(c) = id.child("pin") {
            self.pin = self.pin.sem(c);
        }
        if let Ok(c) = id.child("reply") {
            self.reply = self.reply.sem(c);
        }
        if let Ok(c) = id.child("send") {
            self.send = self.send.sem(c);
        }
        if let Ok(family) = id.child("messages") {
            self.messages = std::mem::take(&mut self.messages)
                .into_iter()
                .enumerate()
                .map(|(i, m)| m.sem(family.key(i.to_string())))
                .collect();
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn is_unread(&self) -> bool {
        self.pin.is_unread()
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn reply(&self) -> &ComposeLine {
        &self.reply
    }

    pub fn reply_mut(&mut self) -> &mut ComposeLine {
        &mut self.reply
    }

    /// Opens or closes the popup; opening reads the thread.
    pub fn set_open(&mut self, on: bool) {
        self.open = on;
        if on {
            self.pin.open();
        } else {
            self.reply.set_focused(false);
        }
    }

    /// Sends the reply: a message of the replier's at the end, the line
    /// cleared, the pin holding them. `false` with nothing to send.
    pub fn send(&mut self) -> bool {
        if self.reply.is_empty() || !self.open {
            return false;
        }
        let i = self.messages.len();
        let mut m = Message::new(self.replier.name.clone(), "now", self.reply.text()).divider(true);
        if let Some(id) = self.part_id(ThreadPart::Message(i)) {
            m = m.sem(id);
        }
        self.messages.push(m);
        self.reply.clear();
        let mut pin = Marker::pin(self.people());
        if let Some(id) = self.pin.sem_id() {
            pin = pin.sem(id.clone());
        }
        self.pin = pin;
        true
    }

    fn look(theme: &Theme) -> LineLook {
        let t = &theme.message.comment.reply;
        LineLook {
            text: t.text,
            placeholder: t.placeholder,
            alpha: 1.0,
        }
    }

    /// The narrowest popup whose reply still holds the avatar, a line and
    /// the submit button side by side.
    pub fn min_popup_w(&self, theme: &Theme) -> f32 {
        let r = &theme.message.comment.reply;
        let s = &theme.message.scroller;
        let send = self.send.preferred_size(theme).0;
        (s.border_width + r.pad + r.well_pad + r.line_pad) * 2.0 + r.avatar + r.gap + send
    }

    /// Where the popup goes in a column `avail` wide: beside the pin, as
    /// wide as the kit's or what is left; under the pin, as wide as the
    /// column, where beside it would be narrower than
    /// [`min_popup_w`](Self::min_popup_w). `(beside, width)`.
    fn popup_w(&self, avail: f32, theme: &Theme) -> (bool, f32) {
        let t = &theme.message.comment;
        let pin = self.pin.size(theme).0;
        let beside = avail - pin - t.pin_gap;
        if beside >= self.min_popup_w(theme) {
            (true, t.popup_w.min(beside))
        } else {
            (false, t.popup_w.min(avail).max(0.0))
        }
    }

    /// The thread's size in a column `avail` wide.
    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        let l = self.layout(Rect::new(0.0, 0.0, avail, 0.0), theme);
        let pin = l.pin;
        match l.popup {
            Some(p) => (p.right().max(pin.right()), p.bottom().max(pin.bottom())),
            None => (pin.w, pin.h),
        }
    }

    /// Where every part lands in `bounds` (its origin and width).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> ThreadLayout {
        let t = &theme.message.comment;
        let s = &theme.message.scroller;
        let r = &t.reply;
        let (pw, ph) = self.pin.size(theme);
        let pin = Rect::new(bounds.x, bounds.y, pw, ph);
        let (beside, w) = self.popup_w(bounds.w, theme);
        let (x, top) = if beside {
            (pin.right() + t.pin_gap, bounds.y)
        } else {
            (bounds.x, pin.bottom() + t.pin_gap)
        };
        let inner_w = (w - s.border_width * 2.0).max(0.0);
        let mut y = top + s.border_width;
        let rows: Vec<Rect> = self
            .messages
            .iter()
            .map(|m| {
                let row = Rect::new(x + s.border_width, y, inner_w, m.height(inner_w, theme));
                y = row.bottom();
                row
            })
            .collect();
        let reply_y = y + r.pad;
        let avatar = Rect::new(
            x + s.border_width + r.pad,
            reply_y + r.avatar_top,
            r.avatar,
            r.avatar,
        );
        let well_x = avatar.right() + r.gap;
        let well_w = (x + w - s.border_width - r.pad - well_x).max(0.0);
        let (send_w, send_h) = self.send.preferred_size(theme);
        let line_h = ComposeLine::height(&Self::look(theme), theme);
        let well_h = r.well_pad * 2.0 + (r.line_pad * 2.0 + line_h).max(send_h);
        let well = Rect::new(well_x, reply_y, well_w, well_h);
        let send = Rect::new(
            well.right() - r.well_pad - send_w,
            well.y + r.well_pad,
            send_w,
            send_h,
        );
        let line = Rect::new(
            well.x + r.well_pad + r.line_pad,
            well.y + r.well_pad + r.line_pad,
            (send.x - well.x - r.well_pad - r.line_pad * 2.0).max(0.0),
            line_h,
        );
        let bottom = well.bottom().max(avatar.bottom()) + r.pad + s.border_width;
        let popup = Rect::new(x, top, w, bottom - top);
        let c = &t.close;
        let close = Rect::new(
            popup.right() - c.inset - c.size,
            popup.y + c.inset,
            c.size,
            c.size,
        );
        ThreadLayout {
            pin,
            popup: self.open.then_some(popup),
            close,
            rows,
            avatar,
            well,
            line,
            send,
        }
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        let mut result = self.pin.handle_event(event, l.pin, theme);
        if result.clicked {
            self.set_open(!self.open);
            return result;
        }
        if l.popup.is_none() {
            return result;
        }
        let r = self.close.handle(event, hit_rect(l.close, theme), false);
        if r.clicked {
            self.set_open(false);
            return result.merge(r);
        }
        result = result.merge(r);
        let look = Self::look(theme);
        result = result.merge(self.reply.handle_event(event, l.line, &look, theme));
        let r = self.send.handle_event(event, l.send);
        if r.clicked {
            self.send();
        }
        result.merge(r)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            let all = l.popup.map_or(l.pin, |p| {
                Rect::new(l.pin.x, l.pin.y, p.right() - l.pin.x, p.h.max(l.pin.h))
            });
            c.bind(id, [all.x, all.y, all.w, all.h]);
        }
        self.pin.render(c, l.pin, theme);
        let Some(p) = l.popup else {
            return;
        };
        let s = &theme.message.scroller;
        shadow_drops(c, layer, p, s.radius, &s.shadow);
        c.push(SceneNode::RoundedRect {
            x: p.x,
            y: p.y,
            w: p.w,
            h: p.h,
            color: s.fill.0,
            corner_radius: s.radius,
            border_width: s.border_width,
            border_color: s.border.0,
        });
        shadow_insets(c, layer, p, s.radius, &s.shadow);
        for (m, r) in self.messages.iter().zip(&l.rows) {
            m.render(c, *r, theme);
        }
        let t = &theme.message.comment;
        self.replier.render(c, l.avatar, theme);
        let w = l.well;
        c.push(rounded_rect(
            w.x,
            w.y,
            w.w,
            w.h,
            t.reply.well_radius,
            t.reply.well.0,
        ));
        self.reply
            .render(c, layer, l.line, &Self::look(theme), theme);
        self.send.render(c, l.send, theme);
        let k = &t.close;
        let r = l.close;
        if let Some(id) = self.part_id(ThreadPart::Close) {
            // the 20 px dot presses in the minimum target (`widgets::hit`)
            let hit = hit_rect(r, theme);
            c.bind(&id, [hit.x, hit.y, hit.w, hit.h]);
        }
        c.push(gradient_fill(r, r.w / 2.0, k.fill[0].0, k.fill[1].0));
        if self.close.focused {
            c.push(focus_ring(r, r.w / 2.0, theme));
        }
        let (cx, cy) = r.center();
        if let Some(node) =
            icons::icon_at("x", k.icon, k.ink.0, cx - k.icon / 2.0, cy - k.icon / 2.0)
        {
            c.push(node);
        }
    }

    /// The caret's blink while the reply is focused.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.reply.wake(dt)
    }

    /// Where an input method puts its candidate window: the reply's caret.
    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        self.reply.ime_area(l.line, &Self::look(theme), theme)
    }

    pub fn part_id(&self, part: ThreadPart) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match part {
            ThreadPart::Pin => id.child("pin").ok(),
            ThreadPart::Close => id.child("close").ok(),
            ThreadPart::Message(i) => id.child("messages").ok().map(|f| f.key(i.to_string())),
            ThreadPart::Reply => id.child("reply").ok(),
            ThreadPart::Send => id.child("send").ok(),
        }
    }

    pub fn part_of(&self, id: &SemId) -> Option<ThreadPart> {
        let root = self.sem.as_ref()?;
        let rest = id.as_str().strip_prefix(root.as_str())?.strip_prefix('.')?;
        Some(match rest {
            "pin" => ThreadPart::Pin,
            "close" => ThreadPart::Close,
            "reply" => ThreadPart::Reply,
            "send" => ThreadPart::Send,
            _ => {
                let i = rest.strip_prefix("messages[")?.strip_suffix(']')?;
                ThreadPart::Message(i.parse().ok()?)
            }
        })
    }

    pub fn focused(&self) -> Option<ThreadPart> {
        if self.reply.is_focused() {
            return Some(ThreadPart::Reply);
        }
        if self.pin.is_focused() {
            return Some(ThreadPart::Pin);
        }
        if self.close.focused {
            return Some(ThreadPart::Close);
        }
        if self.send_focused {
            return Some(ThreadPart::Send);
        }
        self.messages
            .iter()
            .position(Message::is_focused)
            .map(ThreadPart::Message)
    }

    pub fn focus(&mut self, part: Option<ThreadPart>) {
        let is = |p: ThreadPart| part == Some(p);
        self.pin.set_focused(is(ThreadPart::Pin));
        self.close.focused = is(ThreadPart::Close) && self.open;
        self.reply.set_focused(is(ThreadPart::Reply) && self.open);
        self.send.set_focused(is(ThreadPart::Send));
        self.send_focused = is(ThreadPart::Send) && self.open;
        for (i, m) in self.messages.iter_mut().enumerate() {
            m.set_focused(is(ThreadPart::Message(i)) && self.open);
        }
    }

    /// Runs `action` on `part`, as a click would (or, with a text, as
    /// typing). `Err` when the part does not take it.
    pub fn act(
        &mut self,
        part: ThreadPart,
        action: &str,
        param: Option<&Value>,
    ) -> Result<(), String> {
        let done = match (part, action, param) {
            (ThreadPart::Pin, "activate", _) => {
                self.set_open(!self.open);
                true
            }
            (ThreadPart::Close, "activate", _) => {
                let was = self.open;
                self.set_open(false);
                was
            }
            (ThreadPart::Reply, "set_value", Some(Value::Text(t))) => {
                self.open && self.reply.set_value(t)
            }
            (ThreadPart::Send, "activate", _) => {
                self.send();
                self.open
            }
            _ => false,
        };
        if done {
            Ok(())
        } else {
            Err(format!("the comment thread does not {action} {part:?} now"))
        }
    }

    /// Declares the thread as `id`: a `Role::Group` named `label` with
    /// its `open` state; its pin; while open its close button, its
    /// messages (a list with its `count`), the reply line and the submit
    /// button. Returns the group's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        if let Some(pid) = self.pin.sem_id() {
            self.pin.project(b, pid, Some(id));
        }
        if self.open {
            if let Some(cid) = self.part_id(ThreadPart::Close) {
                b.node(&cid, Role::Button, "Close")
                    .parent(id)
                    .action("activate")
                    .reads(format!("{id}:open"));
            }
            if let Ok(list) = id.child("messages") {
                b.node(&list, Role::List, "Messages").parent(id).state(
                    "count",
                    i64::try_from(self.messages.len()).unwrap_or(i64::MAX),
                );
                for m in &self.messages {
                    if let Some(mid) = m.sem_id() {
                        m.project(b, mid, Some(&list));
                    }
                }
            }
            if let Some(rid) = self.part_id(ThreadPart::Reply) {
                self.reply.project(b, &rid, "Reply", Some(id));
            }
            if let Some(sid) = self.part_id(ThreadPart::Send) {
                self.send.project(b, &sid, "Send", Some(id));
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("open", self.open);
        node
    }
}
