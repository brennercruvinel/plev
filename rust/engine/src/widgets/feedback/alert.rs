//! The inline alert of the hoff core kit: its notification row
//! (`components/Notification`), in the page's flow rather than over it
//! (the toast is the passing one). A lead disc, the person's avatar with
//! the intent's badge on its bottom end corner or the intent's mark alone;
//! beside it the title and the time on one line, an unread dot at its end,
//! the detail on one line under them, the content clamped to two lines,
//! and the answers in a row.
//!
//! Sized by its content: the row takes the width it is given and grows as
//! tall as what it holds, the text column vertically centered against the
//! lead while it is shorter, as the kit's `min-h-12 justify-center`. On a
//! narrow column the title is ellipsized before the time is; the detail is
//! one line (`truncate`); the content is cut by
//! `TextMeasurer::clamp_to_lines`. The projection reads all of it whole.
//! The answers wrap to another line when the column is too narrow for
//! them side by side.
//!
//! Behavior apart from the look: an alert is read (`open`) by a click on
//! it or a reader's `activate`, which lets its unread dot go, and an
//! answer (`choose`) is a button's click or `activate`, which reads the
//! alert too. Nothing visual lives here: the box model, the runs and the
//! colors are `theme.message.alert`, the answers are the labeled buttons
//! at the kit's `isSmall` step, the avatar is the content avatar.
//!
//! With a [`SemId`] ([`Alert::sem`]) the render binds the row, each answer
//! binds `<id>.action[<i>]`, and [`Alert::project`] declares them.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, SceneNode};
use crate::drawing::text::TextMeasurer;
use crate::styling::theme::{Intent, Theme};
use crate::widgets::buttons::{Button, ButtonSize, ButtonVariant, Press};
use crate::widgets::content::{Avatar, AvatarSize};
use crate::widgets::icons;
use crate::widgets::message::message::{Placed, fit, placed, run, width};
use crate::widgets::recipe::{focus_ring_inside, rounded_rect};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent, contrast_text};

/// Where every part of an alert lands, from its bounds alone.
#[derive(Clone, Debug, PartialEq)]
pub struct AlertLayout {
    /// The whole row, the hairline above it included.
    pub row: Rect,
    /// The hairline above the row (`border-t`), when asked.
    pub divider: Option<Rect>,
    pub lead: Rect,
    /// The intent's badge on the avatar's corner; without an avatar the
    /// lead itself is the intent's mark.
    pub badge: Option<Rect>,
    pub title: Option<Placed>,
    pub time: Option<Placed>,
    pub dot: Option<Rect>,
    pub detail: Option<Placed>,
    pub content: Option<Placed>,
    pub actions: Vec<Rect>,
}

#[derive(Clone, Debug)]
pub struct Alert {
    intent: Intent,
    title: String,
    time: String,
    detail: String,
    content: String,
    avatar: Option<Avatar>,
    unread: bool,
    divider: bool,
    actions: Vec<Button>,
    chosen: Option<usize>,
    press: Press,
    sem: Option<SemId>,
}

impl Alert {
    pub fn new(intent: Intent, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            intent,
            title: title.into(),
            time: String::new(),
            detail: detail.into(),
            content: String::new(),
            avatar: None,
            unread: false,
            divider: false,
            actions: Vec::new(),
            chosen: None,
            press: Press::default(),
            sem: None,
        }
    }

    /// When it happened, after the title (`1h ago`).
    pub fn time(mut self, time: impl Into<String>) -> Self {
        self.time = time.into();
        self
    }

    /// The longer text under the detail, clamped to the kit's two lines.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Who it is about: their avatar leads, the intent's badge on it.
    pub fn avatar(mut self, name: impl Into<String>) -> Self {
        // `Lg` only bounds the disc; the lead token sizes it
        self.avatar = Some(Avatar::new(name).size(AvatarSize::Lg));
        self
    }

    /// Not read yet: the dot at the end of the title line.
    pub fn unread(mut self, on: bool) -> Self {
        self.unread = on;
        self
    }

    /// The hairline above the row, between two alerts of a list (the
    /// kit's `border-t`; the owner leaves it off the first).
    pub fn divider(mut self, on: bool) -> Self {
        self.divider = on;
        self
    }

    /// An answer, in the row under the text: the kit's invite has
    /// `Decline` as the pale button and `Accept` as the dark one.
    pub fn action(mut self, label: impl Into<String>, variant: ButtonVariant) -> Self {
        let mut button = Button::new(label).variant(variant).size(ButtonSize::Sm);
        if let Some(id) = self
            .sem
            .as_ref()
            .and_then(|id| action_id(id, self.actions.len()))
        {
            button = button.sem(id);
        }
        self.actions.push(button);
        self
    }

    /// The logical identity the render binds the row to; the answers take
    /// `<id>.action[<i>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        for (i, b) in std::mem::take(&mut self.actions).into_iter().enumerate() {
            self.actions.push(match action_id(&id, i) {
                Some(aid) => b.sem(aid),
                None => b,
            });
        }
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    /// The answers' ids, in order.
    pub fn action_ids(&self) -> impl Iterator<Item = Option<&SemId>> {
        self.actions.iter().map(Button::sem_id)
    }

    pub fn intent(&self) -> Intent {
        self.intent
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn is_unread(&self) -> bool {
        self.unread
    }

    /// The answer given, by its index.
    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Read: the unread dot goes. `false` when it was read already.
    pub fn open(&mut self) -> bool {
        std::mem::replace(&mut self.unread, false)
    }

    /// Answer with the action at `index`, which reads the alert too.
    /// `false` for an index it does not have.
    pub fn choose(&mut self, index: usize) -> bool {
        if index >= self.actions.len() {
            return false;
        }
        self.chosen = Some(index);
        self.unread = false;
        true
    }

    /// Keyboard focus on the row, driven by the owning view.
    pub fn set_focused(&mut self, on: bool) {
        self.press.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.press.focused
    }

    /// Keyboard focus on the answer at `index`, or on none.
    pub fn set_action_focused(&mut self, index: Option<usize>) {
        for (i, b) in self.actions.iter_mut().enumerate() {
            b.set_focused(index == Some(i));
        }
    }

    /// The narrowest row whose parts still fit side by side: the padding,
    /// the lead, the gap and the widest answer. Below it the column is
    /// empty.
    pub fn min_width(&self, theme: &Theme) -> f32 {
        let t = &theme.message.alert;
        let widest = self
            .actions
            .iter()
            .map(|b| b.preferred_size(theme).0)
            .fold(0.0, f32::max);
        t.pad + t.lead + t.gap + widest + t.pad_end
    }

    /// The row's height at `width`: the content measured, never fixed.
    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        self.layout(Rect::new(0.0, 0.0, width, 0.0), theme).row.h
    }

    /// Where every part lands in `bounds` (only its origin and width
    /// count; the row is as tall as its content).
    pub fn layout(&self, bounds: Rect, theme: &Theme) -> AlertLayout {
        let t = &theme.message.alert;
        let ty = &theme.typography;
        let hairline = if self.divider {
            theme.message.divider_width
        } else {
            0.0
        };
        let top = bounds.y + hairline + t.pad;
        let lead = Rect::new(bounds.x + t.pad, top, t.lead, t.lead);
        let badge = self.avatar.as_ref().map(|_| {
            let at = t.lead + t.badge_out - t.badge;
            Rect::new(lead.x + at, lead.y + at, t.badge, t.badge)
        });
        let col_x = lead.right() + t.gap;
        let col_r = (bounds.right() - t.pad_end).max(col_x);
        let col_w = col_r - col_x;

        // What the column holds, measured first: it centers against the
        // lead while it is shorter.
        let title_style = t.title.style(ty);
        let time_style = t.time.style(ty);
        let line_room = (col_w - t.dot_room).max(0.0);
        let time = fit(&self.time, &time_style, line_room);
        let time_w = width(&time, &time_style);
        let time_room = if time.is_empty() {
            0.0
        } else {
            t.time_gap + time_w
        };
        let title = fit(&self.title, &title_style, (line_room - time_room).max(0.0));
        let title_w = width(&title, &title_style);
        let has_line = !title.is_empty() || !time.is_empty() || self.unread;
        let line_h = if has_line {
            title_style.line_height.max(time_style.line_height)
        } else {
            0.0
        };
        let detail_style = t.detail.style(ty);
        let detail = fit(&self.detail, &detail_style, col_w);
        let content_style = t.content.style(ty);
        let content = if col_w > 0.0 {
            TextMeasurer::clamp_to_lines(&self.content, &content_style, col_w, t.content_lines)
        } else {
            String::new()
        };
        let content_h = if content.is_empty() {
            0.0
        } else {
            TextMeasurer::measure_styled(&content, &content_style, Some(col_w)).1
        };
        let sizes: Vec<(f32, f32)> = self
            .actions
            .iter()
            .map(|b| b.preferred_size(theme))
            .collect();
        let (flowed, actions_h) = flow(col_x, col_r, t.action_gap, &sizes);

        // Each block after another one takes the kit's gap above it.
        let blocks = [
            (line_h, 0.0),
            (
                detail_style.line_height * f32::from(!detail.is_empty()),
                t.title_gap,
            ),
            (content_h, t.content_gap),
            (actions_h, t.actions_gap),
        ];
        let mut offsets = [0.0; 4];
        let mut y = 0.0;
        for (i, (h, gap)) in blocks.iter().enumerate() {
            if *h <= 0.0 {
                offsets[i] = y;
                continue;
            }
            if y > 0.0 {
                y += gap;
            }
            offsets[i] = y;
            y += h;
        }
        let lift = if has_line { t.title_lift } else { 0.0 };
        let col_h = (y - lift).max(0.0);
        let col_top = top + ((t.lead - col_h) / 2.0).max(0.0) - lift;

        let line_y = col_top + offsets[0];
        let title = placed(title, col_x, line_y, title_w, &title_style);
        let time_x = if self.title.is_empty() || title.is_none() {
            col_x
        } else {
            col_x + title_w + t.time_gap
        };
        let time = placed(time, time_x, line_y, time_w, &time_style);
        let dot = self
            .unread
            .then(|| Rect::new(col_r - t.dot, line_y + t.dot_top, t.dot, t.dot));
        let detail_w = width(&detail, &detail_style);
        let detail = placed(detail, col_x, col_top + offsets[1], detail_w, &detail_style);
        let content = (!content.is_empty()).then(|| Placed {
            text: content,
            rect: Rect::new(col_x, col_top + offsets[2], col_w, content_h),
        });
        let actions_y = col_top + offsets[3];
        let actions = flowed
            .into_iter()
            .map(|r| Rect::new(r.x, actions_y + r.y, r.w, r.h))
            .collect();

        let bottom = lead.bottom().max(col_top + lift + col_h) + t.pad;
        AlertLayout {
            row: Rect::new(bounds.x, bounds.y, bounds.w, bottom - bounds.y),
            divider: self
                .divider
                .then(|| Rect::new(bounds.x, bounds.y, bounds.w, hairline)),
            lead,
            badge,
            title,
            time,
            dot,
            detail,
            content,
            actions,
        }
    }

    /// A click on an answer chooses it; a click elsewhere on the row reads
    /// the alert. Hover and press go to the answer under the pointer.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        let l = self.layout(bounds, theme);
        let mut result = EventResult::IGNORED;
        let mut chose = None;
        for (i, (b, r)) in self.actions.iter_mut().zip(&l.actions).enumerate() {
            let r = b.handle_event(event, *r);
            if r.clicked {
                chose = Some(i);
            }
            result = result.merge(r);
        }
        if let Some(i) = chose {
            self.choose(i);
            return result;
        }
        // The row takes what no answer is under.
        let on_answer = |x: f32, y: f32| l.actions.iter().any(|r| r.contains(x, y));
        let row_event = match *event {
            WidgetEvent::MouseDown { x, y } if on_answer(x, y) => None,
            _ => Some(event),
        };
        if let Some(event) = row_event {
            let r = self.press.handle(event, l.row, false);
            if r.clicked {
                self.open();
            }
            result = result.merge(r);
        }
        result
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let l = self.layout(bounds, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [l.row.x, l.row.y, l.row.w, l.row.h]);
        }
        let t = &theme.message.alert;
        let ty = &theme.typography;
        if let Some(d) = l.divider {
            c.push(SceneNode::Rect {
                x: d.x,
                y: d.y,
                w: d.w,
                h: d.h,
                color: theme.message.divider.0,
            });
        }
        let fill = t.intents.of(self.intent).0;
        match (&self.avatar, l.badge) {
            (Some(avatar), Some(badge)) => {
                avatar.render(c, l.lead, theme);
                disc(c, badge, t.ring.0);
                let inner = badge.inset(t.badge_ring);
                disc(c, inner, fill);
                self.mark(c, inner, t.badge_icon, fill);
            }
            _ => {
                disc(c, l.lead, fill);
                // the icon keeps the badge's share of its disc
                self.mark(c, l.lead, l.lead.w * t.badge_icon / t.badge, fill);
            }
        }
        run(c, l.title.as_ref(), &t.title, ty, None);
        run(c, l.time.as_ref(), &t.time, ty, None);
        if let Some(d) = l.dot {
            disc(c, d, t.unread.0);
        }
        run(c, l.detail.as_ref(), &t.detail, ty, None);
        let wrap = l.content.as_ref().map(|p| p.rect.w);
        run(c, l.content.as_ref(), &t.content, ty, wrap);
        for (b, r) in self.actions.iter().zip(&l.actions) {
            b.render(c, *r, theme);
        }
        if self.press.focused {
            c.push(focus_ring_inside(l.row, theme.shape.micro, theme));
        }
    }

    /// The intent's icon centered in `disc`, in whichever of ink and paper
    /// reads on `fill`.
    fn mark(&self, c: &mut Compositor, disc: Rect, size: f32, fill: [f32; 4]) {
        let name = match self.intent {
            Intent::Neutral | Intent::Informational => "info",
            Intent::Constructive => "check",
            Intent::Destructive => "alert-triangle",
        };
        let (cx, cy) = disc.center();
        let at = (cx - size / 2.0, cy - size / 2.0);
        if let Some(node) = icons::icon_at(name, size, contrast_text(fill), at.0, at.1) {
            c.push(node);
        }
    }

    /// Declares this alert as `id`: a `Role::Alert` when destructive and a
    /// `Role::Status` otherwise, as the toast, named by its title and its
    /// time, described by the detail and the content whole, with its
    /// `unread` state and `activate` (read it), which reads it; each
    /// answer as its button, `<id>.action[<i>]`, `selected` once chosen.
    /// Pass the id given to [`Alert::sem`]. Returns the row's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for (i, button) in self.actions.iter().enumerate() {
            if let Some(aid) = button.sem_id() {
                button
                    .project(b, aid, Some(id))
                    .state("selected", self.chosen == Some(i));
            }
        }
        let role = if self.intent == Intent::Destructive {
            Role::Alert
        } else {
            Role::Status
        };
        let label = join([self.title.as_str(), self.time.as_str()], ", ");
        let mut node = b.node(id, role, label);
        if let Some(parent) = parent {
            node.parent(parent);
        }
        let description = join([self.detail.as_str(), self.content.as_str()], ". ");
        if !description.is_empty() {
            node.description(description);
        }
        node.state("unread", self.unread);
        node.focusable(true);
        node.action("activate").reads(format!("{id}:unread"));
        node
    }
}

/// `<id>.action[<i>]`, an answer's id under its alert.
fn action_id(id: &SemId, i: usize) -> Option<SemId> {
    id.child("action")
        .ok()
        .map(|family| family.key(i.to_string()))
}

fn join<'a>(parts: impl IntoIterator<Item = &'a str>, sep: &str) -> String {
    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(sep)
}

fn disc(c: &mut Compositor, r: Rect, color: [f32; 4]) {
    c.push(rounded_rect(r.x, r.y, r.w, r.h, r.w / 2.0, color));
}

/// Sizes left to right from `x` to `right`, wrapping; rects relative to
/// the top of the first line, and the height they take.
fn flow(x: f32, right: f32, gap: f32, sizes: &[(f32, f32)]) -> (Vec<Rect>, f32) {
    let mut out = Vec::with_capacity(sizes.len());
    let (mut at, mut y, mut line_h) = (x, 0.0_f32, 0.0_f32);
    for &(w, h) in sizes {
        if at > x && at + w > right {
            at = x;
            y += line_h + gap;
            line_h = 0.0;
        }
        out.push(Rect::new(at, y, w, h));
        at += w + gap;
        line_h = line_h.max(h);
    }
    (out, if sizes.is_empty() { 0.0 } else { y + line_h })
}
