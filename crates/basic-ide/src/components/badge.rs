//! Badges — the two HOFF badge shapes:
//! * `Notification` — the 20x20 red circle (#BD3027) with a small-sm
//!   (10/600) number, as on the Sidebar NavLink badge.
//! * `Tag` — chip/tag: bg rgba($n2,.05), radius 8, caption-sm (12/600)
//!   at rgba($n2,.56), padding 4px 8px 2px.

use super::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};
use plev::text::TextStyle;

/// Badge kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeKind {
    /// 20px red circle — counters/notifications.
    Notification,
    /// Glass chip/tag — neutral labels.
    Tag,
}

/// Tag chip metrics — shared with callers that pre-compute the chip
/// footprint (panel headers, the Changes count chip).
const TAG_FONT_SIZE: f32 = 12.0;
const TAG_LINE_H: f32 = 12.0 * 1.33;
const TAG_PAD_X: f32 = 8.0;

/// The caption-sm (12/600) style a `Tag` badge draws with.
pub fn tag_style() -> TextStyle {
    TextStyle::new(TAG_FONT_SIZE)
        .with_line_height(TAG_LINE_H)
        .with_weight(600)
}

/// Width a `Tag` badge takes for `label` — the exact measurement `draw`
/// uses (real shaped width + 2*8px padding).
pub fn tag_width(label: &str) -> f32 {
    hoff::measure_text(label, &tag_style()) + TAG_PAD_X * 2.0
}

/// Draw a badge; returns its width.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
    kind: BadgeKind,
) -> f32 {
    match kind {
        BadgeKind::Notification => {
            const SIZE: f32 = 20.0;
            const FONT_SIZE: f32 = 10.0;
            const LINE_H: f32 = 10.0 * 1.2;
            // One style for measuring AND drawing (small-sm 10/600).
            let style = TextStyle::new(FONT_SIZE)
                .with_line_height(LINE_H)
                .with_weight(600);
            compositor.push(SceneNode::RoundedRect {
                x,
                y,
                w: SIZE,
                h: SIZE,
                color: theme.accent_red.to_array(),
                corner_radius: SIZE / 2.0,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            let text_w = hoff::measure_text(label, &style);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: x + (SIZE - text_w) / 2.0,
                y: y + (SIZE - LINE_H) / 2.0,
                color: theme.text_primary.to_array(),
            });
            SIZE
        }
        BadgeKind::Tag => {
            let style = tag_style();
            let text_w = hoff::measure_text(label, &style);
            let badge_w = text_w + TAG_PAD_X * 2.0;
            let badge_h = TAG_LINE_H + 6.0; // chips pad 4px 8px 2px
            compositor.push(SceneNode::RoundedRect {
                x,
                y,
                w: badge_w,
                h: badge_h,
                color: theme.chip.to_array(),
                corner_radius: theme.radius_tooltip,
                border_width: 0.0,
                border_color: [0.0; 4],
            });
            compositor.push(SceneNode::Text {
                key: TextNodeKey::from_style(label, &style, None),
                x: x + TAG_PAD_X,
                y: y + (badge_h - TAG_LINE_H) / 2.0,
                color: theme.text_default.to_array(),
            });
            badge_w
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plev::compositor::LayerId;

    /// Regression: the chip rect must fit the REAL shaped label plus both
    /// paddings. "MODIFIED" @12/600 was one of the labels the old per-char
    /// heuristic mis-sized, letting the text leak out of the chip.
    #[test]
    fn tag_chip_fits_real_shaped_label() {
        let mut c = Compositor::new();
        c.begin_frame();
        let w = draw(
            &mut c,
            &crate::theme::DARK,
            0.0,
            0.0,
            "MODIFIED",
            BadgeKind::Tag,
        );
        let text_w = hoff::measure_text("MODIFIED", &tag_style());
        assert!(
            w >= text_w + 2.0 * 8.0 - 1e-3,
            "chip width {w} must fit measured label {text_w} + 2*8 padding"
        );
        // The background rect drawn must be exactly the returned width.
        let rect_w = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .find_map(|n| match n {
                SceneNode::RoundedRect { w, .. } => Some(*w),
                _ => None,
            })
            .expect("tag draws a chip rect");
        assert_eq!(rect_w, w);
        // And `tag_width` (used by panel headers) matches the drawn chip.
        assert_eq!(tag_width("MODIFIED"), w);
    }
}
