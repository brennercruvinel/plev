//! The quick comment of the hoff core kit (`components/QuickComment`): an
//! object's card over the page. Its header (a close button, the object's
//! name and kind), its preview (the thumbnail stands as a rectangle of the
//! token's color), its comments (the quick layout of the message, with
//! the like chip), then the reply (`QuickComment/Comment`: the replier's
//! avatar and name over the line) and a row of tools with `Send`.
//!
//! Sized by its content: the kit's `w-92` or the column, the preview a
//! square as wide as the kit's picture or the card's inside, the comments
//! as tall as they measure.
//!
//! Behavior apart from the look: the line edits as a field
//! ([`ComposeLine`]); `Send` posts the reply as the replier's comment at
//! the end; the close button, the tools and the like chips press and
//! nothing else happens, as in the kit (the card is not in a modal here).
//! Every visual value is `theme.message.comment.quick`; the rows are the
//! message's, the buttons the kit's.
//!
//! Its parts take `<id>.close`, `.comments[<i>]` (and their chips), `.reply`,
//! `.tools[<i>]` and `.send`.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder, Value};

use super::compose::{ComposeLine, LineLook};
use super::message::{Message, MessageKind};
use super::select_ai::line;
use super::tool::Tool;
use crate::drawing::compositor::{Compositor, LayerId};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{
    Button, ButtonSize, ButtonVariant, IconButton, IconButtonSize, IconButtonVariant,
};
use crate::widgets::content::{Avatar, AvatarSize};
use crate::widgets::icons;
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The tools under the reply (`QuickComment/Comment/index.tsx:9-26`).
pub const TOOLS: [(&str, &str); 4] = [
    ("circle-plus", "Add"),
    ("smile", "Emoji"),
    ("thumbs-up", "Like"),
    ("thumbs-down", "Dislike"),
];

/// A part of the card a reader can land on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuickPart {
    Close,
    Comment(usize),
    Like(usize),
    Reply,
    Tool(usize),
    Send,
}

/// Where every part lands, from the bounds' origin and width.
#[derive(Clone, Debug, PartialEq)]
pub struct QuickLayout {
    pub card: Rect,
    pub close: Rect,
    pub title: Rect,
    pub preview: Rect,
    pub picture: Rect,
    pub comments: Vec<Rect>,
    pub avatar: Rect,
    pub name: Rect,
    pub seal: Option<Rect>,
    pub line: Rect,
    pub tools: Vec<Rect>,
    pub send: Rect,
}

#[derive(Clone, Debug)]
pub struct QuickComment {
    title: String,
    subtitle: String,
    close: IconButton,
    comments: Vec<Message>,
    replier: Avatar,
    verified: bool,
    reply: ComposeLine,
    tools: Vec<Tool>,
    send: Button,
    /// The like chip the keyboard is on (a message keeps no flag to read).
    liked_focus: Option<usize>,
    sem: Option<SemId>,
}

impl QuickComment {
    /// The card of the object `title` of kind `subtitle`, its comments,
    /// the reply written by `replier` and asking `placeholder`.
    pub fn new(
        title: impl Into<String>,
        subtitle: impl Into<String>,
        comments: Vec<Message>,
        replier: &str,
        placeholder: &str,
    ) -> Self {
        Self {
            title: title.into(),
            subtitle: subtitle.into(),
            close: IconButton::new("x")
                .variant(IconButtonVariant::Outline)
                .size(IconButtonSize::Md),
            comments: comments
                .into_iter()
                .map(|m| m.kind(MessageKind::Quick))
                .collect(),
            replier: Avatar::new(replier).size(AvatarSize::Md),
            verified: false,
            reply: ComposeLine::new(placeholder),
            tools: TOOLS
                .iter()
                .map(|(icon, label)| Tool::new(icon, label))
                .collect(),
            send: Button::new("Send")
                .variant(ButtonVariant::Primary)
                .size(ButtonSize::Sm),
            liked_focus: None,
            sem: None,
        }
    }

    /// The replier verified: the kit's seal after the name.
    pub fn verified(mut self, on: bool) -> Self {
        self.verified = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(c) = id.child("close") {
            self.close = self.close.sem(c);
        }
        if let Ok(family) = id.child("comments") {
            self.comments = std::mem::take(&mut self.comments)
                .into_iter()
                .enumerate()
                .map(|(i, m)| m.sem(family.key(i.to_string())))
                .collect();
        }
        if let Ok(c) = id.child("reply") {
            self.reply = self.reply.sem(c);
        }
        if let Ok(family) = id.child("tools") {
            for (i, t) in self.tools.iter_mut().enumerate() {
                *t = t.clone().sem(family.key(i.to_string()));
            }
        }
        if let Ok(c) = id.child("send") {
            self.send = self.send.sem(c);
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn comments(&self) -> &[Message] {
        &self.comments
    }

    pub fn line(&self) -> &ComposeLine {
        &self.reply
    }

    pub fn line_mut(&mut self) -> &mut ComposeLine {
        &mut self.reply
    }

    /// Posts the reply as the replier's comment. `false` with nothing in
    /// the line.
    pub fn send(&mut self) -> bool {
        if self.reply.is_empty() {
            return false;
        }
        let i = self.comments.len();
        let mut m = Message::new(self.replier.name.clone(), "now", self.reply.text())
            .kind(MessageKind::Quick)
            .verified(self.verified);
        if let Some(id) = self.part_id(QuickPart::Comment(i)) {
            m = m.sem(id);
        }
        self.comments.push(m);
        self.reply.clear();
        true
    }

    fn look(theme: &Theme) -> LineLook {
        let t = &theme.message.comment.quick;
        LineLook {
            text: t.text,
            placeholder: t.placeholder,
            alpha: 1.0,
        }
    }

    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        let card = self.layout(Rect::new(0.0, 0.0, avail, 0.0), theme).card;
        (card.w, card.h)
    }

    pub fn layout(&self, bounds: Rect, theme: &Theme) -> QuickLayout {
        let t = &theme.message.comment.quick;
        let w = t.w.min(bounds.w).max(0.0);
        let inner = Rect::new(
            bounds.x + t.pad,
            bounds.y + t.pad,
            (w - t.pad * 2.0).max(0.0),
            0.0,
        );
        let (cs, _) = self.close.preferred_size(theme);
        let close = Rect::new(inner.x + t.header_pad, inner.y + t.header_pad, cs, cs);
        let tx = close.right() + t.header_gap;
        let head_h =
            t.title.line_height(&theme.typography) + t.subtitle.line_height(&theme.typography);
        let title = Rect::new(
            tx,
            close.y + (cs - head_h).max(0.0) / 2.0,
            (inner.right() - t.header_pad - tx).max(0.0),
            head_h,
        );
        let mut y = close.bottom().max(title.bottom()) + t.header_pad + t.header_bottom;
        let edge = theme.message.divider_width;
        let pic = t.picture.min(inner.w - edge * 2.0).max(0.0);
        let preview = Rect::new(inner.x, y, inner.w, pic + edge * 2.0);
        let picture = Rect::new(
            preview.x + (preview.w - pic) / 2.0,
            preview.y + edge,
            pic,
            pic,
        );
        y = preview.bottom() + t.preview_bottom + t.comments_top;
        let quick = &theme.message.quick;
        let comments: Vec<Rect> = self
            .comments
            .iter()
            .map(|m| {
                let r = Rect::new(inner.x, y, inner.w, m.height(inner.w, theme));
                y = r.bottom() - quick.pad;
                r
            })
            .collect();
        let cx = inner.x + t.comments_pad;
        let cw = (inner.w - t.comments_pad * 2.0).max(0.0);
        let top = if comments.is_empty() {
            y + t.comments_pad
        } else {
            y + t.reply_top
        };
        let avatar = Rect::new(cx, top, quick.avatar, quick.avatar);
        let col = avatar.right() + quick.gap;
        let col_w = (cx + cw - col).max(0.0);
        let name = Rect::new(col, top, col_w, t.name.line_height(&theme.typography));
        let seal = self.verified.then(|| {
            let style = t.name.style(&theme.typography);
            let w = super::message::width(self.replier.name.as_str(), &style);
            let edge = t.name.line_height(&theme.typography);
            Rect::new(col + w + quick.seal_gap, top, edge, edge)
        });
        let line_h = ComposeLine::height(&Self::look(theme), theme);
        let line = Rect::new(col, name.bottom() + t.line_gap, col_w, line_h);
        let row_y = line.bottom().max(avatar.bottom()) + t.tools_top;
        let tool = &t.tool;
        let (sw, sh) = self.send.preferred_size(theme);
        let row_h = tool.size.max(sh);
        let tools = (0..self.tools.len())
            .map(|i| {
                let x = cx + i as f32 * (tool.size + tool.gap);
                Rect::new(x, row_y + (row_h - tool.size) / 2.0, tool.size, tool.size)
            })
            .collect();
        let send = Rect::new(cx + cw - sw, row_y + (row_h - sh) / 2.0, sw, sh);
        let bottom = row_y + row_h + t.comments_pad + t.pad;
        QuickLayout {
            card: Rect::new(bounds.x, bounds.y, w, bottom - bounds.y),
            close,
            title,
            preview,
            picture,
            comments,
            avatar,
            name,
            seal,
            line,
            tools,
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
        let mut result = self.close.handle_event(event, l.close);
        for (m, r) in self.comments.iter_mut().zip(&l.comments) {
            result = result.merge(m.handle_event(event, *r, theme));
        }
        let look = Self::look(theme);
        result = result.merge(self.reply.handle_event(event, l.line, &look, theme));
        for (tool, r) in self.tools.iter_mut().zip(&l.tools) {
            result = result.merge(tool.handle_event(event, *r));
        }
        let r = self.send.handle_event(event, l.send);
        if r.clicked {
            self.send();
        }
        result.merge(r)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let t = &theme.message.comment.quick;
        let l = self.layout(bounds, theme);
        let k = l.card;
        if let Some(id) = &self.sem {
            c.bind(id, [k.x, k.y, k.w, k.h]);
        }
        shadow_drops(c, layer, k, t.radius, &t.shadow);
        c.push(rounded_rect(k.x, k.y, k.w, k.h, t.radius, t.fill.0));
        self.close.render(c, l.close, theme);
        let room = l.title.w;
        line(
            c,
            layer,
            &self.title,
            &t.title,
            l.title.x,
            l.title.y,
            room,
            theme,
        );
        let sub_y = l.title.y + t.title.line_height(&theme.typography);
        line(
            c,
            layer,
            &self.subtitle,
            &t.subtitle,
            l.title.x,
            sub_y,
            room,
            theme,
        );
        let p = l.preview;
        let edge = theme.message.divider_width;
        c.push(rounded_rect(
            p.x,
            p.y,
            p.w,
            p.h,
            t.preview_radius,
            t.preview_fill.0,
        ));
        c.push(rounded_rect_stroke(
            p.x,
            p.y,
            p.w,
            p.h,
            t.preview_radius,
            t.preview_border.0,
            edge,
        ));
        let pic = l.picture;
        c.push(rounded_rect(
            pic.x,
            pic.y,
            pic.w,
            pic.h,
            0.0,
            t.picture_fill.0,
        ));
        for (m, r) in self.comments.iter().zip(&l.comments) {
            m.render(c, *r, theme);
        }
        self.replier.render(c, l.avatar, theme);
        let n = l.name;
        let name_w = line(c, layer, &self.replier.name, &t.name, n.x, n.y, n.w, theme);
        if let Some(s) = l.seal.filter(|s| s.x >= n.x + name_w)
            && let Some(node) = icons::icon_at("badge-check", s.w, t.name.color.0, s.x, s.y)
        {
            c.push(node);
        }
        self.reply
            .render(c, layer, l.line, &Self::look(theme), theme);
        for (tool, r) in self.tools.iter().zip(&l.tools) {
            tool.render(c, layer, *r, &t.tool, theme);
        }
        self.send.render(c, l.send, theme);
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        self.reply.wake(dt)
    }

    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        self.reply.ime_area(l.line, &Self::look(theme), theme)
    }

    pub fn part_id(&self, part: QuickPart) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        let indexed = |name: &str, i: usize| id.child(name).ok().map(|f| f.key(i.to_string()));
        match part {
            QuickPart::Close => id.child("close").ok(),
            QuickPart::Comment(i) => indexed("comments", i),
            QuickPart::Like(i) => indexed("comments", i).and_then(|c| c.child("reaction").ok()),
            QuickPart::Reply => id.child("reply").ok(),
            QuickPart::Tool(i) => indexed("tools", i),
            QuickPart::Send => id.child("send").ok(),
        }
    }

    pub fn part_of(&self, id: &SemId) -> Option<QuickPart> {
        let root = self.sem.as_ref()?;
        let rest = id.as_str().strip_prefix(root.as_str())?.strip_prefix('.')?;
        let index = |name: &str| -> Option<(usize, &str)> {
            let tail = rest.strip_prefix(name)?.strip_prefix('[')?;
            let (i, after) = tail.split_once(']')?;
            Some((i.parse().ok()?, after))
        };
        Some(match rest {
            "close" => QuickPart::Close,
            "reply" => QuickPart::Reply,
            "send" => QuickPart::Send,
            _ => match (index("comments"), index("tools")) {
                (Some((i, "")), _) => QuickPart::Comment(i),
                (Some((i, ".reaction")), _) => QuickPart::Like(i),
                (_, Some((i, ""))) => QuickPart::Tool(i),
                _ => return None,
            },
        })
    }

    pub fn focused(&self) -> Option<QuickPart> {
        if self.reply.is_focused() {
            return Some(QuickPart::Reply);
        }
        if self.close.is_focused() {
            return Some(QuickPart::Close);
        }
        if self.send.is_focused() {
            return Some(QuickPart::Send);
        }
        if let Some(i) = self.tools.iter().position(Tool::is_focused) {
            return Some(QuickPart::Tool(i));
        }
        self.comments
            .iter()
            .position(Message::is_focused)
            .map(QuickPart::Comment)
            .or_else(|| self.liked_focus.map(QuickPart::Like))
    }

    pub fn focus(&mut self, part: Option<QuickPart>) {
        let is = |p: QuickPart| part == Some(p);
        self.close.set_focused(is(QuickPart::Close));
        self.reply.set_focused(is(QuickPart::Reply));
        self.send.set_focused(is(QuickPart::Send));
        for (i, t) in self.tools.iter_mut().enumerate() {
            t.set_focused(is(QuickPart::Tool(i)));
        }
        for (i, m) in self.comments.iter_mut().enumerate() {
            m.set_focused(is(QuickPart::Comment(i)));
            m.set_reaction_focused(is(QuickPart::Like(i)));
        }
        self.liked_focus = match part {
            Some(QuickPart::Like(i)) => Some(i),
            _ => None,
        };
    }

    /// Runs `action` on `part`: `Send` posts, the line takes a text; the
    /// close button, a tool and a like chip press and nothing else.
    pub fn act(
        &mut self,
        part: QuickPart,
        action: &str,
        param: Option<&Value>,
    ) -> Result<(), String> {
        let done = match (part, action, param) {
            (QuickPart::Reply, "set_value", Some(Value::Text(t))) => self.reply.set_value(t),
            (QuickPart::Send, "activate", _) => {
                self.send();
                true
            }
            (QuickPart::Close | QuickPart::Tool(_), "activate", _) => true,
            (QuickPart::Like(i), "activate", _) => {
                self.comments.get(i).is_some_and(Message::has_reaction)
            }
            _ => false,
        };
        if done {
            Ok(())
        } else {
            Err(format!("the quick comment does not {action} {part:?}"))
        }
    }

    /// Declares the card as `id`: a `Role::Group` named by the object's
    /// name and described by its kind; its close button, its comments (a
    /// list with its `count`, each with its like chip), the reply line, the
    /// tools and `Send`. Returns the group's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        if let Some(cid) = self.part_id(QuickPart::Close) {
            self.close.project(b, &cid, "Close", Some(id));
        }
        if let Ok(list) = id.child("comments") {
            b.node(&list, Role::List, "Comments").parent(id).state(
                "count",
                i64::try_from(self.comments.len()).unwrap_or(i64::MAX),
            );
            for m in &self.comments {
                if let Some(mid) = m.sem_id() {
                    m.project(b, mid, Some(&list));
                }
            }
        }
        if let Some(rid) = self.part_id(QuickPart::Reply) {
            self.reply.project(b, &rid, "Reply", Some(id));
        }
        for t in &self.tools {
            if let Some(tid) = t.sem_id() {
                t.project(b, tid, Some(id));
            }
        }
        if let Some(sid) = self.send.sem_id() {
            self.send.project(b, sid, Some(id));
        }
        let mut node = b.node(id, Role::Group, self.title.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.description(self.subtitle.as_str());
        node
    }
}
