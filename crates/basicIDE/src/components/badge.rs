//! Badges — the two HOFF badge shapes:
//! * `Notification` — the 20x20 red circle (#BD3027) with a small-sm
//!   (10/600) number, as on the Sidebar NavLink badge.
//! * `Tag` — chip/tag: bg rgba($n2,.05), radius 8, caption-sm (12/600)
//!   at rgba($n2,.56), padding 4px 8px 2px.

use super::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode, TextNodeKey};

/// Badge kind.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BadgeKind {
    /// 20px red circle — counters/notifications.
    Notification,
    /// Glass chip/tag — neutral labels.
    Tag,
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
            let text_w = hoff::text_width(label, FONT_SIZE);
            compositor.push(SceneNode::Text {
                key: TextNodeKey::new(label, FONT_SIZE, LINE_H, None).with_weight(600),
                x: x + (SIZE - text_w) / 2.0,
                y: y + (SIZE - LINE_H) / 2.0,
                color: theme.text_primary.to_array(),
            });
            SIZE
        }
        BadgeKind::Tag => {
            const FONT_SIZE: f32 = 12.0;
            const LINE_H: f32 = 12.0 * 1.33;
            const PAD_X: f32 = 8.0;
            let text_w = hoff::text_width(label, FONT_SIZE);
            let badge_w = text_w + PAD_X * 2.0;
            let badge_h = LINE_H + 6.0; // chips pad 4px 8px 2px
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
                key: TextNodeKey::new(label, FONT_SIZE, LINE_H, None).with_weight(600),
                x: x + PAD_X,
                y: y + (badge_h - LINE_H) / 2.0,
                color: theme.text_default.to_array(),
            });
            badge_w
        }
    }
}
