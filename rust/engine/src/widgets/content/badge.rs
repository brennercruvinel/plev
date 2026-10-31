//! Badges, the three shapes of the hoff core kit (`theme.message.badge`):
//! - `Tag`: the kit's chip label (`Card`), a run in a rounded box with a
//!   hairline; an intent puts its disc before the label, and the label
//!   stays in the ink, so it reads at AAA on every intent.
//! - `Count`: a number in a pill on the chip's run, a disc for one digit,
//!   filled by its intent with the numeral at 7:1.
//! - `Dot`: the kit's notification mark (`HeaderSimple`), a disc on its
//!   host's top end corner, no text.
//!
//! Static: a badge is read, never pressed (for a pressable pill see
//! [`Chip`](crate::action::Chip)). It measures its text, takes the width
//! it is given and ellipsizes a label that does not fit; a reader can land
//! on it (`project`), and it says its label and intent.

use accord::{NodeBuilder, Role, SemId, SnapshotBuilder};

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::styling::theme::{Intent, Theme};

use crate::widgets::recipe::{focus_ring, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::Rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeKind {
    /// A number in a pill: counters, unread counts.
    Count,
    /// The chip: status words, file states, section counts.
    #[default]
    Tag,
    /// The notification mark: something new, no number.
    Dot,
}

#[derive(Clone, Debug)]
pub struct Badge {
    pub label: String,
    pub kind: BadgeKind,
    /// A count's fill, a tag's disc, a dot's color. A count is
    /// Destructive by default (the kit's red), a tag Neutral (no disc).
    pub intent: Intent,
    /// A tag on a tinted surface: paper with a hairline (the kit's gray
    /// card).
    pub raised: bool,
    sem: Option<SemId>,
    focused: bool,
}

impl Badge {
    fn of(kind: BadgeKind, label: String, intent: Intent) -> Self {
        Self {
            label,
            kind,
            intent,
            raised: false,
            sem: None,
            focused: false,
        }
    }

    pub fn count(label: impl Into<String>) -> Self {
        Self::of(BadgeKind::Count, label.into(), Intent::Destructive)
    }

    pub fn tag(label: impl Into<String>) -> Self {
        Self::of(BadgeKind::Tag, label.into(), Intent::Neutral)
    }

    /// The mark, named for a reader by `label` (it draws no text).
    pub fn dot(label: impl Into<String>) -> Self {
        Self::of(BadgeKind::Dot, label.into(), Intent::Destructive)
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    pub fn raised(mut self, raised: bool) -> Self {
        self.raised = raised;
        self
    }

    pub fn sem(mut self, id: SemId) -> Self {
        self.sem = Some(id);
        self
    }

    pub fn sem_id(&self) -> Option<&SemId> {
        self.sem.as_ref()
    }

    pub fn set_focused(&mut self, on: bool) {
        self.focused = on;
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// The one style the label is measured and drawn with: the chip's
    /// run, for the tag and the count alike.
    pub fn text_style(&self, theme: &Theme) -> TextStyle {
        theme.message.badge.tag.text.style(&theme.typography)
    }

    /// The tag's disc and its gap, when its intent shows one.
    fn lead(&self, theme: &Theme) -> f32 {
        let t = &theme.message.badge.tag;
        match (self.kind, self.intent) {
            (BadgeKind::Tag, Intent::Neutral) => 0.0,
            (BadgeKind::Tag, _) => t.mark + t.mark_gap,
            _ => 0.0,
        }
    }

    /// Across the label: the box's padding and border, and the disc.
    fn frame_w(&self, theme: &Theme) -> f32 {
        let b = &theme.message.badge;
        match self.kind {
            BadgeKind::Tag => (b.tag.pad_x + b.tag.border_width) * 2.0 + self.lead(theme),
            BadgeKind::Count => b.count_pad_x * 2.0,
            BadgeKind::Dot => 0.0,
        }
    }

    /// The intrinsic size from the measured label: the tag's run with its
    /// padding and border; the count as tall as the run and its padding,
    /// never narrower than tall; the dot its disc.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let b = &theme.message.badge;
        let style = self.text_style(theme);
        let line = style.line_height;
        let tw = || TextMeasurer::measure_styled(&self.label, &style, None).0;
        match self.kind {
            BadgeKind::Tag => (
                (tw() + self.frame_w(theme)).ceil(),
                line + (b.tag.pad_y + b.tag.border_width) * 2.0,
            ),
            BadgeKind::Count => {
                let h = line + b.tag.pad_y * 2.0;
                ((tw() + self.frame_w(theme)).max(h).ceil(), h)
            }
            BadgeKind::Dot => (b.dot, b.dot),
        }
    }

    /// The box drawn at the top-left of `bounds`: the preferred size, no
    /// wider than `bounds` (never narrower than tall); a `bounds` with no
    /// width places the badge at its preferred size.
    pub fn rect(&self, bounds: Rect, theme: &Theme) -> Rect {
        let (w, h) = self.preferred_size(theme);
        let w = if bounds.w > 0.0 {
            w.min(bounds.w.max(h))
        } else {
            w
        };
        Rect::new(bounds.x, bounds.y, w, h)
    }

    /// Where a dot sits on `host`: its top end corner, in by the inset.
    pub fn dot_on(host: Rect, theme: &Theme) -> Rect {
        let b = &theme.message.badge;
        Rect::new(
            host.right() - b.dot_inset - b.dot,
            host.y + b.dot_inset,
            b.dot,
            b.dot,
        )
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// Draw at the top-left of `bounds`, at the preferred size or the
    /// width of `bounds`, whichever is less; the label ellipsized to fit.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let b = &theme.message.badge;
        let r = self.rect(bounds, theme);
        let radius = match self.kind {
            BadgeKind::Tag => b.tag.radius,
            BadgeKind::Count | BadgeKind::Dot => r.h / 2.0,
        };
        let fill = match self.kind {
            BadgeKind::Tag if self.raised => b.tag.raised_fill,
            BadgeKind::Tag => b.tag.fill,
            BadgeKind::Count => b.count_fill.of(self.intent),
            BadgeKind::Dot => b.dots.of(self.intent),
        };
        c.push_to_layer(layer, rounded_rect(r.x, r.y, r.w, r.h, radius, fill.0));
        if self.kind == BadgeKind::Tag {
            let border = if self.raised {
                b.tag.raised_border
            } else {
                b.tag.border
            };
            if border.0[3] > 0.0 {
                let bw = b.tag.border_width;
                c.push_to_layer(
                    layer,
                    rounded_rect_stroke(r.x, r.y, r.w, r.h, radius, border.0, bw),
                );
            }
        }
        if self.focused {
            c.push_to_layer(layer, focus_ring(r, radius, theme));
        }
        if self.kind == BadgeKind::Dot {
            return;
        }
        let lead = self.lead(theme);
        let x = r.x
            + match self.kind {
                BadgeKind::Tag => b.tag.pad_x + b.tag.border_width,
                _ => b.count_pad_x,
            };
        if lead > 0.0 {
            let d = b.tag.mark;
            let y = r.y + (r.h - d) / 2.0;
            let mark = b.marks.of(self.intent);
            c.push_to_layer(layer, rounded_rect(x, y, d, d, d / 2.0, mark.0));
        }
        let style = self.text_style(theme);
        let room = (r.w - self.frame_w(theme)).max(0.0);
        let label = TextMeasurer::truncate_to_width(&self.label, &style, room);
        let (tw, _) = TextMeasurer::measure_styled(&label, &style, None);
        let ink = match self.kind {
            BadgeKind::Count => b.count_ink.of(self.intent),
            _ => b.tag.text.color,
        };
        // a count centers its numeral; a tag starts after its disc
        let tx = match self.kind {
            BadgeKind::Count => r.x + (r.w - tw) / 2.0,
            _ => x + lead,
        };
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&label, &style, None),
                x: tx,
                y: r.y + TextMeasurer::vertical_center(&style, r.h),
                color: ink.0,
            },
        );
    }

    /// A reader's view: a status named by its label, with its intent, and
    /// focusable so a reader can land on it. It declares no action.
    pub fn project<'b>(
        &self,
        b: &'b mut SnapshotBuilder,
        id: &SemId,
        parent: Option<&SemId>,
    ) -> NodeBuilder<'b> {
        let mut node = b.node(id, Role::Status, self.label.as_str());
        if let Some(parent) = parent {
            node.parent(parent);
        }
        node.state("intent", intent_word(self.intent));
        node.focusable(true);
        node
    }
}

/// The intent as a reader hears it.
fn intent_word(intent: Intent) -> &'static str {
    match intent {
        Intent::Neutral => "neutral",
        Intent::Informational => "informational",
        Intent::Constructive => "constructive",
        Intent::Destructive => "destructive",
    }
}
