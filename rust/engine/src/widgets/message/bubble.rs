//! The chat bubble of the hoff core kit: its comment pin
//! (`components/CommentPin`), a pill on paper with a 1.5 px border and a
//! small drop shadow, its bottom corner on the speaker's side square (the
//! tail), here holding text. Read, the border is the hairline; unread, the
//! kit's accent; an owner may give a person's color instead.
//!
//! Sized by its content: the text wraps against the widest the bubble may
//! grow, a fraction of the row it is given (`theme.message.bubble`), never
//! a width in px, and the bubble is as wide and tall as the text then
//! measures. It sits at the start of its row (the other person), at the
//! end (oneself) or centered (a note). The corner radius is half a
//! one-line bubble, so a short bubble is a pill and a long one keeps the
//! same corners.
//!
//! The engine's rounded rect has one radius for its four corners
//! (docs/research/makepad-gap-analysis.md); the square tail is the pill
//! with a square patch over that corner, the border's and the fill's, so
//! the fill is opaque.
//!
//! With a [`SemId`] ([`Bubble::sem`]) the render binds the bubble's rect,
//! and [`Bubble::project`] declares it.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::widgets::recipe::{focus_ring, rounded_rect, shadow_drops};
use crate::widgets::widget::Rect;

/// Where a bubble sits in its row, and which bottom corner is its tail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleSide {
    /// The other person: at the start, the tail at the bottom start.
    #[default]
    Start,
    /// Oneself: at the end, the tail at the bottom end.
    End,
    /// A note: centered, no tail.
    Center,
}

#[derive(Clone, Debug)]
pub struct Bubble {
    text: String,
    side: BubbleSide,
    unread: bool,
    accent: Option<Color>,
    sem: Option<SemId>,
    focused: bool,
}

impl Bubble {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            side: BubbleSide::default(),
            unread: false,
            accent: None,
            sem: None,
            focused: false,
        }
    }

    pub fn side(mut self, side: BubbleSide) -> Self {
        self.side = side;
        self
    }

    /// Not read yet: the border takes the accent.
    pub fn unread(mut self, on: bool) -> Self {
        self.unread = on;
        self
    }

    /// A person's color for the border, in place of the read hairline and
    /// the unread accent.
    pub fn accent(mut self, color: Color) -> Self {
        self.accent = Some(color);
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn is_unread(&self) -> bool {
        self.unread
    }

    pub fn bubble_side(&self) -> BubbleSide {
        self.side
    }

    pub fn set_focused(&mut self, on: bool) {
        self.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    fn style(theme: &Theme) -> TextStyle {
        theme.message.bubble.text.style(&theme.typography)
    }

    /// Border plus padding, on each side across and down.
    fn frame(theme: &Theme) -> (f32, f32) {
        let t = &theme.message.bubble;
        (t.pad_x + t.border_width, t.pad_y + t.border_width)
    }

    /// The widest the text may wrap at in a row `width` wide; `None` when
    /// the row leaves no room for a glyph.
    fn wrap(width: f32, theme: &Theme) -> Option<f32> {
        let (fx, _) = Self::frame(theme);
        let room = width * theme.message.bubble.max_fraction - fx * 2.0;
        (room >= Self::style(theme).font_size).then_some(room)
    }

    /// The text's measured size in a row `width` wide (zero when it does
    /// not fit at all).
    fn text_size(&self, width: f32, theme: &Theme) -> (f32, f32) {
        match Self::wrap(width, theme) {
            Some(room) if !self.text.is_empty() => {
                TextMeasurer::measure_styled(&self.text, &Self::style(theme), Some(room))
            }
            _ => (0.0, 0.0),
        }
    }

    /// The bubble's size in a row `width` wide: the text measured, with
    /// the padding and the border around it; never under one line.
    pub fn size(&self, width: f32, theme: &Theme) -> (f32, f32) {
        let (fx, fy) = Self::frame(theme);
        let (tw, th) = self.text_size(width, theme);
        let line = Self::style(theme).line_height;
        (tw + fx * 2.0, th.max(line) + fy * 2.0)
    }

    /// The bubble's rect in `row`, at its side.
    pub fn rect(&self, row: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.size(row.w, theme);
        let x = match self.side {
            BubbleSide::Start => row.x,
            BubbleSide::End => row.right() - w,
            BubbleSide::Center => row.x + (row.w - w) / 2.0,
        };
        Rect::new(x, row.y, w, h)
    }

    /// Half a one-line bubble, never more than the bubble allows.
    fn radius(r: Rect, theme: &Theme) -> f32 {
        let (_, fy) = Self::frame(theme);
        ((Self::style(theme).line_height + fy * 2.0) / 2.0)
            .min(r.h / 2.0)
            .min(r.w / 2.0)
    }

    /// The pill `r` at `radius`, its tail corner square.
    fn shape(&self, c: &mut Compositor, r: Rect, radius: f32, color: [f32; 4]) {
        c.push(rounded_rect(r.x, r.y, r.w, r.h, radius, color));
        let tail_x = match self.side {
            BubbleSide::Start => Some(r.x),
            BubbleSide::End => Some(r.right() - radius),
            BubbleSide::Center => None,
        };
        if let Some(x) = tail_x {
            c.push(SceneNode::Rect {
                x,
                y: r.bottom() - radius,
                w: radius,
                h: radius,
                color,
            });
        }
    }

    /// Draws the bubble at its side of `row`.
    pub fn render(&self, c: &mut Compositor, row: Rect, theme: &Theme) {
        let t = &theme.message.bubble;
        let r = self.rect(row, theme);
        if let Some(id) = &self.sem {
            c.bind(id, [r.x, r.y, r.w, r.h]);
        }
        let radius = Self::radius(r, theme);
        shadow_drops(
            c,
            LayerId::DEFAULT,
            r,
            radius,
            std::slice::from_ref(&t.shadow),
        );
        let border = match (self.accent, self.unread) {
            (Some(accent), _) => accent,
            (None, true) => t.unread,
            (None, false) => t.border,
        };
        self.shape(c, r, radius, border.0);
        let inner = r.inset(t.border_width);
        let inner_radius = (radius - t.border_width).max(0.0);
        self.shape(c, inner, inner_radius, t.fill.0);
        let (tw, _) = self.text_size(row.w, theme);
        if tw > 0.0 {
            let (fx, fy) = Self::frame(theme);
            // Wrapped at its own measured width (the same breaks as at the
            // widest, every line already fits): a right-to-left paragraph
            // aligns to the end of the text, not of the widest box.
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(&self.text, &Self::style(theme), Some(tw.ceil())),
                x: r.x + fx,
                y: r.y + fy,
                color: t.text.color.0,
            });
        }
        if self.focused {
            c.push(focus_ring(r, radius, theme));
        }
    }

    /// Declares this bubble as `id`: a `Role::ListItem` named by its
    /// text, with its `unread` state, focusable so a reader can land on
    /// it. Returns the node for the owner to add to.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::ListItem, self.text.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("unread", self.unread);
        node.focusable(true);
        node
    }
}
