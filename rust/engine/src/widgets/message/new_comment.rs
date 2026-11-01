//! The new comment of the hoff core kit (`components/NewComment`): the
//! comment cursor where the comment goes, and beside it the raised box it
//! is written in. Empty, the box is its line and the cursor is the
//! pinned one; written in, the cursor types, and the box opens on the
//! attachments (`NewComment/ImagePreview`, the attachment tray) and a row
//! of tools over a hairline: add a picture, mention, an emoji, and the
//! submit button.
//!
//! Sized by its content: the box is the kit's `w-70` or what the width
//! leaves after the cursor, as tall as its line, its tray and its tools.
//!
//! Behavior apart from the look: the line edits as a field
//! ([`ComposeLine`]); the add tool attaches a picture (the gallery stands
//! in for the picker), the mention tool writes an `@`, the emoji tool an
//! emoji; a tile's remove button removes it; the submit button posts the
//! comment (it counts it) and empties the box. While a picture uploads,
//! the cursor is the uploading one. Every visual value is
//! `theme.message.comment.new`, the cursor the marker's, the tiles the
//! attachment's.
//!
//! Its parts take `<id>.cursor`, `.text`, `.files` (the tray's ids under
//! it), `.tools[<i>]` and `.send`.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder, Value};

use super::attachment::{AttachmentKind, AttachmentTray, TrayFocus};
use super::compose::{ComposeLine, LineLook};
use super::marker::{CursorState, Marker};
use super::tool::Tool;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::Theme;
use crate::widgets::buttons::{SubmitButton, SubmitButtonSize};
use crate::widgets::recipe::{rounded_rect, rounded_rect_stroke, shadow_drops, shadow_insets};
use crate::widgets::wake::Wake;
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

/// The tools under the line, their icon and what a reader hears.
pub const TOOLS: [(&str, &str); 3] = [
    ("circle-plus", "Add a picture"),
    ("at-sign", "Mention"),
    ("smile", "Emoji"),
];

/// A part of the new comment a reader can land on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NewCommentPart {
    Text,
    /// A tile of the tray, or its remove button.
    Tray(TrayFocus),
    Tool(usize),
    Send,
}

/// Where every part lands, from the bounds' origin and width.
#[derive(Clone, Debug, PartialEq)]
pub struct NewCommentLayout {
    pub cursor: Rect,
    pub frame: Rect,
    pub line: Rect,
    /// The attachments, the tool row's hairline, its tools and the submit
    /// button, while open.
    pub tray: Option<Rect>,
    pub rule: Option<Rect>,
    pub tools: Vec<Rect>,
    pub send: Option<Rect>,
}

#[derive(Clone, Debug)]
pub struct NewComment {
    line: ComposeLine,
    tray: AttachmentTray,
    tools: Vec<Tool>,
    /// The kit's SubmitButton at its `isMedium` step.
    send: SubmitButton,
    send_focused: bool,
    uploading: bool,
    posted: usize,
    /// Pictures the add tool attached, for their names.
    pictures: usize,
    sem: Option<SemId>,
}

impl Default for NewComment {
    fn default() -> Self {
        Self::new()
    }
}

impl NewComment {
    pub fn new() -> Self {
        Self {
            line: ComposeLine::new("Add comment..."),
            tray: AttachmentTray::new(),
            tools: TOOLS
                .iter()
                .map(|(icon, label)| Tool::new(icon, label))
                .collect(),
            send: SubmitButton::new().size(SubmitButtonSize::Md),
            send_focused: false,
            uploading: false,
            posted: 0,
            pictures: 0,
            sem: None,
        }
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.line = self.line.with_text(text);
        self
    }

    /// An attachment already in the box.
    pub fn with_file(mut self, name: &str, detail: &str, kind: AttachmentKind) -> Self {
        self.tray = self.tray.with(name, detail, kind);
        self
    }

    /// A picture uploading: the cursor is the uploading one.
    pub fn uploading(mut self, on: bool) -> Self {
        self.uploading = on;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        if let Ok(c) = id.child("text") {
            self.line = self.line.sem(c);
        }
        if let Ok(c) = id.child("files") {
            self.tray = self.tray.sem(c);
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

    pub fn line(&self) -> &ComposeLine {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut ComposeLine {
        &mut self.line
    }

    pub fn tray(&self) -> &AttachmentTray {
        &self.tray
    }

    pub fn tray_mut(&mut self) -> &mut AttachmentTray {
        &mut self.tray
    }

    pub fn posted(&self) -> usize {
        self.posted
    }

    /// Written in, or holding an attachment: the box shows its tray and
    /// its tools.
    pub fn is_open(&self) -> bool {
        !self.line.is_empty() || !self.tray.items().is_empty()
    }

    /// The cursor's state: uploading, typing while written in, pinned at
    /// rest.
    pub fn cursor_state(&self) -> CursorState {
        if self.uploading {
            CursorState::Uploading
        } else if self.line.is_empty() {
            CursorState::Pinned
        } else {
            CursorState::Typing
        }
    }

    fn cursor(&self) -> Marker {
        let m = Marker::cursor(self.cursor_state());
        match self.sem.as_ref().and_then(|id| id.child("cursor").ok()) {
            Some(id) => m.sem(id),
            None => m,
        }
    }

    /// Runs tool `i`: a picture attached, an `@` or an emoji written.
    pub fn use_tool(&mut self, i: usize) -> bool {
        match i {
            0 => {
                self.pictures += 1;
                let name = format!("Picture {}", self.pictures);
                self.tray.push(name, "PNG", AttachmentKind::Image(None));
                true
            }
            1 | 2 => {
                let mark = if i == 1 { "@" } else { "\u{1f642}" };
                let text = format!("{}{mark}", self.line.text());
                self.line.set_value(&text)
            }
            _ => false,
        }
    }

    /// Posts the comment and empties the box. `false` with nothing in it.
    pub fn post(&mut self) -> bool {
        if !self.is_open() {
            return false;
        }
        self.posted += 1;
        self.line.clear();
        let keys: Vec<u64> = self.tray.items().iter().map(|a| a.key()).collect();
        for key in keys {
            self.tray.remove(key);
        }
        self.uploading = false;
        true
    }

    fn look(theme: &Theme) -> LineLook {
        let t = &theme.message.comment.new;
        LineLook {
            text: t.text,
            placeholder: t.placeholder,
            alpha: 1.0,
        }
    }

    /// The new comment's size in a column `avail` wide.
    pub fn size(&self, avail: f32, theme: &Theme) -> (f32, f32) {
        let l = self.layout(Rect::new(0.0, 0.0, avail, 0.0), theme);
        (
            l.frame.right().max(l.cursor.right()),
            l.frame.bottom().max(l.cursor.bottom()),
        )
    }

    /// The narrowest box whose tool row still holds the tools and the
    /// submit button side by side.
    pub fn min_box_w(&self, theme: &Theme) -> f32 {
        let t = &theme.message.comment.new;
        let n = self.tools.len() as f32;
        let send = self.send.preferred_size(theme).0;
        (t.border_width + t.tools_pad) * 2.0 + n * (t.tool.size + t.tool.gap) + send
    }

    /// The box beside the cursor, as wide as the kit's or what is left;
    /// under it, as wide as the column, where beside it would be narrower
    /// than [`min_box_w`](Self::min_box_w).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> NewCommentLayout {
        let t = &theme.message.comment.new;
        let (cw, ch) = self.cursor().size(theme);
        let beside = bounds.w - cw - t.gap >= self.min_box_w(theme);
        let cursor = if beside {
            Rect::new(bounds.x, bounds.y + t.cursor_top, cw, ch)
        } else {
            Rect::new(bounds.x, bounds.y, cw, ch)
        };
        let (x, top) = if beside {
            (cursor.right() + t.gap, bounds.y)
        } else {
            (bounds.x, cursor.bottom() + t.gap)
        };
        let w = t.w.min(bounds.right() - x).max(0.0);
        let bounds = Rect::new(bounds.x, top, bounds.w, bounds.h);
        let edge = t.border_width + t.pad + t.line_pad;
        let line_h = ComposeLine::height(&Self::look(theme), theme);
        let line = Rect::new(x + edge, bounds.y + edge, (w - edge * 2.0).max(0.0), line_h);
        let mut y = line.bottom() + t.line_pad + t.pad;
        let (mut tray, mut rule, mut tools, mut send) = (None, None, Vec::new(), None);
        if self.is_open() {
            let tw = (w - (t.border_width + t.tray_pad_x) * 2.0).max(0.0);
            if !self.tray.items().is_empty() {
                let r = Rect::new(
                    x + t.border_width + t.tray_pad_x,
                    y,
                    tw,
                    self.tray.height(tw, theme),
                );
                y = r.bottom() + t.tray_pad_bottom;
                tray = Some(r);
            }
            y += t.tools_top;
            let inner_w = (w - t.border_width * 2.0).max(0.0);
            rule = Some(Rect::new(x + t.border_width, y, inner_w, t.border_width));
            let top = y + t.border_width + t.tools_pad;
            let tool = &t.tool;
            let (sw, sh) = self.send.preferred_size(theme);
            let row_h = tool.size.max(sh);
            let mut tx = x + t.border_width + t.tools_pad;
            for _ in &self.tools {
                tools.push(Rect::new(
                    tx,
                    top + (row_h - tool.size) / 2.0,
                    tool.size,
                    tool.size,
                ));
                tx += tool.size + tool.gap;
            }
            let sx = x + w - t.border_width - t.tools_pad - sw;
            send = Some(Rect::new(sx, top + (row_h - sh) / 2.0, sw, sh));
            y = top + row_h + t.tools_pad;
        }
        let frame = Rect::new(x, bounds.y, w, y + t.border_width - bounds.y);
        NewCommentLayout {
            cursor,
            frame,
            line,
            tray,
            rule,
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
        let look = Self::look(theme);
        let mut result = self.line.handle_event(event, l.line, &look, theme);
        if let Some(r) = l.tray {
            result = result.merge(self.tray.handle_event(event, r, theme));
        }
        let mut used = None;
        for (i, (tool, r)) in self.tools.iter_mut().zip(&l.tools).enumerate() {
            let r = tool.handle_event(event, *r);
            if r.clicked {
                used = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = used {
            self.use_tool(i);
        }
        if let Some(at) = l.send {
            let r = self.send.handle_event(event, at);
            if r.clicked {
                self.post();
            }
            result = result.merge(r);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let layer = LayerId::DEFAULT;
        let t = &theme.message.comment.new;
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            let f = l.frame;
            let (w, h) = (f.right() - bounds.x, f.bottom() - bounds.y);
            c.bind(id, [bounds.x, bounds.y, w, h]);
        }
        self.cursor().render(c, l.cursor, theme);
        let f = l.frame;
        shadow_drops(c, layer, f, t.radius, &t.shadow);
        c.push(rounded_rect(f.x, f.y, f.w, f.h, t.radius, t.fill.0));
        c.push(rounded_rect_stroke(
            f.x,
            f.y,
            f.w,
            f.h,
            t.radius,
            t.border.0,
            t.border_width,
        ));
        shadow_insets(c, layer, f, t.radius, &t.shadow);
        self.line
            .render(c, layer, l.line, &Self::look(theme), theme);
        if let Some(r) = l.tray {
            self.tray.render(c, r, theme);
        }
        if let Some(r) = l.rule {
            c.push(SceneNode::Rect {
                x: r.x,
                y: r.y,
                w: r.w,
                h: r.h,
                color: t.border.0,
            });
        }
        for (tool, r) in self.tools.iter().zip(&l.tools) {
            tool.render(c, layer, *r, &t.tool, theme);
        }
        if let Some(r) = l.send {
            self.send.render(c, r, theme);
        }
    }

    pub fn wake(&mut self, dt: f32) -> Wake {
        self.line.wake(dt)
    }

    pub fn ime_area(&self, bounds: Rect, theme: &Theme) -> Rect {
        let l = self.layout(bounds, theme);
        self.line.ime_area(l.line, &Self::look(theme), theme)
    }

    pub fn part_id(&self, part: NewCommentPart) -> Option<SemId> {
        let id = self.sem.as_ref()?;
        match part {
            NewCommentPart::Text => id.child("text").ok(),
            NewCommentPart::Tray(f) => {
                let tray = self.tray.sem_id()?;
                match f {
                    TrayFocus::Tile(k) => Some(tray.key(k.to_string())),
                    TrayFocus::Remove(k) => tray.key(k.to_string()).child("remove").ok(),
                    TrayFocus::Add => tray.child("add").ok(),
                }
            }
            NewCommentPart::Tool(i) => id.child("tools").ok().map(|f| f.key(i.to_string())),
            NewCommentPart::Send => id.child("send").ok(),
        }
    }

    pub fn part_of(&self, id: &SemId) -> Option<NewCommentPart> {
        if let Some(f) = self.tray.target(id) {
            return Some(NewCommentPart::Tray(f));
        }
        let root = self.sem.as_ref()?;
        let rest = id.as_str().strip_prefix(root.as_str())?.strip_prefix('.')?;
        Some(match rest {
            "text" => NewCommentPart::Text,
            "send" => NewCommentPart::Send,
            _ => {
                let i = rest.strip_prefix("tools[")?.strip_suffix(']')?;
                NewCommentPart::Tool(i.parse().ok()?)
            }
        })
    }

    pub fn focused(&self) -> Option<NewCommentPart> {
        if self.line.is_focused() {
            return Some(NewCommentPart::Text);
        }
        if self.send_focused {
            return Some(NewCommentPart::Send);
        }
        if let Some(f) = self.tray.focus() {
            return Some(NewCommentPart::Tray(f));
        }
        self.tools
            .iter()
            .position(Tool::is_focused)
            .map(NewCommentPart::Tool)
    }

    pub fn focus(&mut self, part: Option<NewCommentPart>) {
        let is = |p: NewCommentPart| part == Some(p);
        self.line.set_focused(is(NewCommentPart::Text));
        let open = self.is_open();
        for (i, t) in self.tools.iter_mut().enumerate() {
            t.set_focused(is(NewCommentPart::Tool(i)) && open);
        }
        self.send.set_focused(is(NewCommentPart::Send));
        self.send_focused = is(NewCommentPart::Send) && open;
        self.tray.set_focus(match part {
            Some(NewCommentPart::Tray(f)) => Some(f),
            _ => None,
        });
    }

    pub fn act(
        &mut self,
        part: NewCommentPart,
        action: &str,
        param: Option<&Value>,
    ) -> Result<(), String> {
        let open = self.is_open();
        let done = match (part, action, param) {
            (NewCommentPart::Text, "set_value", Some(Value::Text(t))) => self.line.set_value(t),
            (NewCommentPart::Tray(TrayFocus::Remove(key)), "activate", _) => self.tray.remove(key),
            (NewCommentPart::Tool(i), "activate", _) => open && self.use_tool(i),
            (NewCommentPart::Send, "activate", _) => self.post(),
            _ => false,
        };
        if done {
            Ok(())
        } else {
            Err(format!("the new comment does not {action} {part:?} now"))
        }
    }

    /// Declares the new comment as `id`: a `Role::Group` named `label`
    /// with the comments it `posted`; its cursor (an image of its state),
    /// its line, and while open its attachments (the tray's list), its
    /// tools and its submit button. Returns the group's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        label: impl Into<String>,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let cursor = self.cursor();
        if let Some(cid) = cursor.sem_id() {
            cursor.project(b, cid, Some(id));
        }
        if let Some(tid) = self.part_id(NewCommentPart::Text) {
            self.line.project(b, &tid, "Comment", Some(id));
        }
        if self.is_open() {
            if let Some(fid) = self.tray.sem_id() {
                self.tray.project(b, fid, "Attachments", Some(id));
            }
            for t in &self.tools {
                if let Some(tid) = t.sem_id() {
                    t.project(b, tid, Some(id));
                }
            }
            if let Some(sid) = self.part_id(NewCommentPart::Send) {
                self.send.project(b, &sid, "Post", Some(id));
            }
        }
        let mut node = b.node(id, Role::Group, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("posted", i64::try_from(self.posted).unwrap_or(i64::MAX));
        node
    }
}
