//! The message scroller of the hoff core kit: its comment popup
//! (`components/Comment`), the thread's messages in a rounded paper
//! surface with a hairline and the kit's shadow list, scrolling once they
//! outgrow it.
//!
//! One scroll state, the engine's (`control::input::scroll`), with the
//! engine's fading scrollbar, and at rest the paper fading over an edge
//! the thread runs past (`theme.message.scroller.edge`), the sign that it
//! goes on: the wheel and a touch drag (a synthesized
//! wheel) move it, a reader or the keyboard landing on a message scrolls
//! it into view ([`MessageScroller::reveal`]); a wheel at the end of the
//! thread is left to the page. As tall as its messages measure at the
//! width it is given, up to `theme.message.scroller.max_h`; empty, it
//! says so in one muted line.
//!
//! With a [`SemId`] ([`MessageScroller::sem`]) each message takes
//! `<id>[<i>]` and [`MessageScroller::project`] declares the thread as a
//! list of them, every one of them, read in order.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use super::message::Message;
use crate::control::input::scroll::ScrollState;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::styling::theme::Theme;
use crate::widgets::feedback::Scrollbar;
use crate::widgets::recipe::{shadow_drops, shadow_insets};
use crate::widgets::widget::{EventResult, Rect, WidgetEvent};

#[derive(Clone, Debug)]
pub struct MessageScroller {
    label: String,
    empty: String,
    messages: Vec<Message>,
    scroll: ScrollState,
    bar: Scrollbar,
    sem: Option<SemId>,
}

impl MessageScroller {
    /// A thread named `label` (what a reader hears), saying `empty` when
    /// it has no message.
    /// The surface's border closes the last message: its hairline goes.
    pub fn new(label: impl Into<String>, empty: impl Into<String>, messages: Vec<Message>) -> Self {
        let last = messages.len().saturating_sub(1);
        let messages = messages
            .into_iter()
            .enumerate()
            .map(|(i, m)| if i == last { m.divider(false) } else { m })
            .collect();
        Self {
            label: label.into(),
            empty: empty.into(),
            messages,
            scroll: ScrollState::new(),
            bar: Scrollbar::new(),
            sem: None,
        }
    }

    /// The thread's id; message `i` takes `<id>[<i>]`.
    pub fn sem(mut self, id: SemId) -> Self {
        self.messages = std::mem::take(&mut self.messages)
            .into_iter()
            .enumerate()
            .map(|(i, m)| m.sem(id.key(i.to_string())))
            .collect();
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut [Message] {
        &mut self.messages
    }

    pub fn offset(&self) -> f32 {
        self.scroll.offset()
    }

    /// The surface's inside, where the messages scroll.
    fn inner(bounds: Rect, theme: &Theme) -> Rect {
        bounds.inset(theme.message.scroller.border_width)
    }

    fn empty_h(theme: &Theme) -> f32 {
        let s = &theme.message.thread;
        theme.message.scroller.empty.line_height + s.pad * 2.0
    }

    /// The messages' height, all of them, in a surface `width` wide.
    pub fn content_h(&self, width: f32, theme: &Theme) -> f32 {
        let w = (width - theme.message.scroller.border_width * 2.0).max(0.0);
        if self.messages.is_empty() {
            return Self::empty_h(theme);
        }
        self.messages.iter().map(|m| m.height(w, theme)).sum()
    }

    /// The scroller's height in a column `width` wide: its messages, up
    /// to the cap, inside its border.
    pub fn height(&self, width: f32, theme: &Theme) -> f32 {
        let t = &theme.message.scroller;
        self.content_h(width, theme)
            .min(t.max_h - t.border_width * 2.0)
            + t.border_width * 2.0
    }

    /// The scroll state over `bounds`: its viewport and its content.
    fn sync(&mut self, bounds: Rect, theme: &Theme) {
        self.scroll.set_viewport(Self::inner(bounds, theme).h);
        self.scroll.set_content(self.content_h(bounds.w, theme));
    }

    /// Each message's rect in `bounds`, at the current offset (some above
    /// or below the surface, which clips them).
    pub fn rows(&self, bounds: Rect, theme: &Theme) -> Vec<Rect> {
        let inner = Self::inner(bounds, theme);
        let mut y = inner.y - self.scroll.offset();
        self.messages
            .iter()
            .map(|m| {
                let r = Rect::new(inner.x, y, inner.w, m.height(inner.w, theme));
                y += r.h;
                r
            })
            .collect()
    }

    /// Scrolls the least that shows message `i` whole (its top, when it
    /// is taller than the surface). `true` when the offset moved.
    pub fn reveal(&mut self, i: usize, bounds: Rect, theme: &Theme) -> bool {
        self.sync(bounds, theme);
        let Some(row) = self.rows(bounds, theme).get(i).copied() else {
            return false;
        };
        let inner = Self::inner(bounds, theme);
        let old = self.scroll.offset();
        if row.y < inner.y || row.h > inner.h {
            self.scroll.scroll_by(row.y - inner.y);
        } else if row.bottom() > inner.bottom() {
            self.scroll.scroll_by(row.bottom() - inner.bottom());
        }
        let moved = self.scroll.offset() != old;
        if moved {
            self.bar.notify_scroll();
        }
        moved
    }

    /// The wheel scrolls the thread while it can (handled, so the page
    /// does not scroll too); the bar drags it; pointer events reach the
    /// messages shown.
    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        bounds: Rect,
        theme: &Theme,
    ) -> EventResult {
        self.sync(bounds, theme);
        let inner = Self::inner(bounds, theme);
        let bar = self.bar.handle_event(event, inner, &mut self.scroll, theme);
        if bar.handled {
            return bar;
        }
        if let WidgetEvent::Scroll { x, y, delta } = *event {
            if !inner.contains(x, y) {
                return EventResult::IGNORED;
            }
            let old = self.scroll.offset();
            self.scroll.scroll_by(delta);
            if self.scroll.offset() == old {
                return EventResult::IGNORED;
            }
            self.bar.notify_scroll();
            return EventResult::changed();
        }
        let (px, py) = event.pos();
        let inside = inner.contains(px, py);
        let rows = self.rows(bounds, theme);
        let mut result = bar;
        for (m, r) in self.messages.iter_mut().zip(rows) {
            // Outside the surface a message is hidden: it hears no press,
            // only the moves that let its hover go.
            if inside || matches!(event, WidgetEvent::MouseMove { .. }) {
                let ev = if inside {
                    *event
                } else {
                    WidgetEvent::MouseMove {
                        x: f32::MIN,
                        y: f32::MIN,
                    }
                };
                result = result.merge(m.handle_event(&ev, r, theme));
            }
        }
        result
    }

    /// The edges the thread runs past, `(top, bottom)`: above while it is
    /// scrolled down, below while it has not reached its end.
    pub fn cut_edges(&self, bounds: Rect, theme: &Theme) -> (bool, bool) {
        let inner = Self::inner(bounds, theme);
        let max = (self.content_h(bounds.w, theme) - inner.h).max(0.0);
        let off = self.scroll.offset().clamp(0.0, max);
        (off > 0.5, max > 0.5 && off < max - 0.5)
    }

    /// The paper fading over each cut edge of the surface, inside the clip.
    fn render_edges(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let inner = Self::inner(bounds, theme);
        let t = &theme.message.scroller;
        let edge = t.edge.min(inner.h / 2.0);
        let paper = t.fill.0;
        let clear = [paper[0], paper[1], paper[2], 0.0];
        let (top, bottom) = self.cut_edges(bounds, theme);
        let radius = (t.radius - t.border_width).max(0.0);
        // css 180deg: the first color at the top
        let fade = |c: &mut Compositor, y: f32, from: [f32; 4], to: [f32; 4]| {
            c.push(SceneNode::GradientRect {
                x: inner.x,
                y,
                w: inner.w,
                h: edge,
                color: from,
                color2: to,
                angle_deg: 180.0,
                corner_radius: radius.min(edge),
                border_width: 0.0,
                border_color: [0.0; 4],
            });
        };
        if top {
            fade(c, inner.y, paper, clear);
        }
        if bottom {
            fade(c, inner.bottom() - edge, clear, paper);
        }
    }

    /// The bar's fade. `true` while it animates.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.bar.tick(dt)
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        let t = &theme.message.scroller;
        if let Some(id) = &self.sem {
            c.bind(id, [bounds.x, bounds.y, bounds.w, bounds.h]);
        }
        shadow_drops(c, LayerId::DEFAULT, bounds, t.radius, &t.shadow);
        c.push(SceneNode::RoundedRect {
            x: bounds.x,
            y: bounds.y,
            w: bounds.w,
            h: bounds.h,
            color: t.fill.0,
            corner_radius: t.radius,
            border_width: t.border_width,
            border_color: t.border.0,
        });
        shadow_insets(c, LayerId::DEFAULT, bounds, t.radius, &t.shadow);
        let inner = Self::inner(bounds, theme);
        if self.messages.is_empty() {
            let style = t.empty.style(&theme.typography);
            let pad = theme.message.thread.pad;
            c.push(SceneNode::Text {
                key: TextNodeKey::from_style(
                    &self.empty,
                    &style,
                    Some((inner.w - pad * 2.0).max(0.0)),
                ),
                x: inner.x + pad,
                y: inner.y + pad,
                color: t.empty.color.0,
            });
            return;
        }
        c.push_clip(inner.x, inner.y, inner.w, inner.h);
        for (m, r) in self.messages.iter().zip(self.rows(bounds, theme)) {
            if r.bottom() > inner.y && r.y < inner.bottom() {
                m.render(c, r, theme);
            }
        }
        self.render_edges(c, bounds, theme);
        c.pop_clip();
        self.bar.render(c, inner, &self.scroll, theme);
    }

    /// Declares the thread as `id`: a `Role::List` named by its label,
    /// with its `count`, and each message `<id>[<i>]` under it, the ones
    /// scrolled away too (a reader landing on one scrolls it in). Returns
    /// the list's node.
    #[track_caller]
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        for m in &self.messages {
            if let Some(mid) = m.sem_id() {
                m.project(b, mid, Some(id));
            }
        }
        let mut node = b.node(id, Role::List, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state(
            "count",
            i64::try_from(self.messages.len()).unwrap_or(i64::MAX),
        );
        node
    }
}
