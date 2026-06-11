//! Counter lifecycle + color palette + layout constants.

use plev::component::Lifecycle;
use plev::compositor::{SceneNode, TextNodeKey};
use plev::view::ViewContext;

// --- Color palette --------------------------------------------------------

pub const BG: [f32; 4] = [0.06, 0.06, 0.10, 1.0];
pub const HEADER_BG: [f32; 4] = [0.08, 0.08, 0.14, 1.0];
pub const SURFACE: [f32; 4] = [0.10, 0.10, 0.16, 1.0];
pub const ACCENT_DIM: [f32; 4] = [0.20, 0.35, 0.70, 1.0];
pub const CYAN: [f32; 4] = [0.0, 0.85, 0.95, 1.0];
pub const ORANGE: [f32; 4] = [1.0, 0.55, 0.10, 1.0];
pub const TEXT: [f32; 4] = [0.93, 0.93, 0.96, 1.0];
pub const TEXT_DIM: [f32; 4] = [0.55, 0.55, 0.65, 1.0];
pub const DIVIDER: [f32; 4] = [0.18, 0.18, 0.25, 1.0];
pub const FOOTER_BG: [f32; 4] = [0.07, 0.07, 0.12, 1.0];

// --- Layout constants -----------------------------------------------------

pub const HEADER_H: f32 = 70.0;
pub const FOOTER_H: f32 = 32.0;
pub const MARGIN: f32 = 32.0;
pub const ACCENT_BAR_H: f32 = 2.0;

// --- Counter Lifecycle ----------------------------------------------------

pub struct Counter;

impl Lifecycle for Counter {
    type State = u64;

    fn initial_state(&self) -> u64 {
        0
    }

    fn on_update(&self, count: &mut u64) {
        *count += 1;
    }

    fn render(&self, count: &u64, cx: &mut ViewContext) -> Vec<SceneNode> {
        let w = cx.width;
        let content_w = w - MARGIN * 2.0;
        let card_w = content_w * 0.55;
        let card_x = MARGIN;
        let card_y = HEADER_H + MARGIN;
        let card_h = 180.0;

        let mut nodes = Vec::with_capacity(12);

        nodes.push(SceneNode::Rect {
            x: card_x,
            y: card_y,
            w: card_w,
            h: card_h,
            color: SURFACE,
        });
        nodes.push(SceneNode::Rect {
            x: card_x,
            y: card_y,
            w: card_w,
            h: ACCENT_BAR_H,
            color: ACCENT_DIM,
        });
        nodes.push(SceneNode::Text {
            key: TextNodeKey::new("FRAME COUNTER", 11.0, 14.3, Some(card_w - 32.0)),
            x: card_x + 16.0,
            y: card_y + 16.0,
            color: ORANGE,
        });

        let count_str = format!("{}", count);
        nodes.push(SceneNode::Text {
            key: TextNodeKey::new(&count_str, 38.0, 49.4, Some(card_w - 32.0)),
            x: card_x + 16.0,
            y: card_y + 44.0,
            color: ORANGE,
        });
        nodes.push(SceneNode::Text {
            key: TextNodeKey::new("frames", 11.0, 14.3, Some(card_w - 32.0)),
            x: card_x + 16.0,
            y: card_y + 100.0,
            color: TEXT_DIM,
        });
        nodes.push(SceneNode::Rect {
            x: card_x + 16.0,
            y: card_y + 124.0,
            w: card_w - 32.0,
            h: 1.0,
            color: DIVIDER,
        });
        nodes.push(SceneNode::Text {
            key: TextNodeKey::new(
                "on_update() increments every frame",
                10.0,
                13.0,
                Some(card_w - 32.0),
            ),
            x: card_x + 16.0,
            y: card_y + 140.0,
            color: TEXT_DIM,
        });

        nodes
    }
}
