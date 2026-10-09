//! HOFF badges, the two shapes the kit uses:
//! - `Count`: the notification disc (danger color, small-sm number), one
//!   `spacing.xl` wide, as on the sidebar nav link badge.
//! - `Tag`: the neutral chip/tag (surface-hover glass, tooltip radius,
//!   caption-sm label in the text-default tone).
//!
//! Static: badges are read, never pressed. For a pressable pill see
//! [`Chip`](crate::action::Chip).

use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::{TextMeasurer, TextStyle};
use engine::theme::{Intent, Theme};

use crate::core::{Rect, contrast_text, intent_fill};
use crate::recipe::rounded_rect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeKind {
    /// Small colored disc with a number: counters, unread marks.
    Count,
    /// Glass chip: status words, file states, section counts.
    #[default]
    Tag,
}

#[derive(Clone, Debug)]
pub struct Badge {
    pub label: String,
    pub kind: BadgeKind,
    /// `Count` discs take the intent color (Destructive by default, the
    /// HOFF red); `Tag` chips tint their label with it when not Neutral.
    pub intent: Intent,
}

impl Badge {
    pub fn count(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: BadgeKind::Count,
            intent: Intent::Destructive,
        }
    }

    pub fn tag(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            kind: BadgeKind::Tag,
            intent: Intent::Neutral,
        }
    }

    pub fn intent(mut self, intent: Intent) -> Self {
        self.intent = intent;
        self
    }

    /// Label style: small-sm for the disc, caption-sm for the tag. One
    /// style for measuring and drawing.
    pub fn text_style(&self, theme: &Theme) -> TextStyle {
        match self.kind {
            BadgeKind::Count => theme.typography.small_sm(),
            BadgeKind::Tag => theme.typography.caption_sm(),
        }
    }

    /// Intrinsic size from real text measurement. The count disc is a
    /// circle at least `spacing.xl` wide that stretches into a pill for
    /// wide numbers.
    pub fn preferred_size(&self, theme: &Theme) -> (f32, f32) {
        let style = self.text_style(theme);
        let (tw, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        match self.kind {
            BadgeKind::Count => {
                let d = theme.spacing.xl;
                ((tw + theme.spacing.sm).max(d).ceil(), d)
            }
            BadgeKind::Tag => (
                (tw + theme.spacing.sm * 2.0).ceil(),
                style.line_height + theme.spacing.xs + theme.spacing.xs / 2.0,
            ),
        }
    }

    pub fn render(&self, c: &mut Compositor, bounds: Rect, theme: &Theme) {
        self.render_to_layer(c, LayerId::DEFAULT, bounds, theme);
    }

    /// Draw at the top-left of `bounds` at the preferred size.
    pub fn render_to_layer(&self, c: &mut Compositor, layer: LayerId, bounds: Rect, theme: &Theme) {
        let style = self.text_style(theme);
        let (w, h) = self.preferred_size(theme);
        let (tw, _) = TextMeasurer::measure_styled(&self.label, &style, None);
        let (bg, fg, radius) = match self.kind {
            BadgeKind::Count => {
                let bg = intent_fill(theme, self.intent);
                (bg, contrast_text(bg), h / 2.0)
            }
            BadgeKind::Tag => (
                theme.glass.surface_hover.0,
                match self.intent {
                    Intent::Neutral => theme.glass.text_default.0,
                    other => intent_fill(theme, other),
                },
                theme.shape.tooltip,
            ),
        };
        c.push_to_layer(layer, rounded_rect(bounds.x, bounds.y, w, h, radius, bg));
        c.push_to_layer(
            layer,
            SceneNode::Text {
                key: TextNodeKey::from_style(&self.label, &style, None),
                x: bounds.x + (w - tw) / 2.0,
                y: bounds.y + TextMeasurer::vertical_center(&style, h),
                color: fg,
            },
        );
    }
}
