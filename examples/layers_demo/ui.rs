//! Scene-building helpers: header, info card, footer, foreground card.

use plev::compositor::{LayerId, SceneNode, TextNodeKey};

use crate::palette::*;

pub fn build_header(comp: &mut plev::compositor::Compositor, w: f32, margin: f32) {
    let header_h = 70.0;
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w,
            h: header_h,
            color: HEADER_BG,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: 0.0,
            y: header_h - 1.0,
            w,
            h: 1.0,
            color: DIVIDER,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "LAYER SYSTEM",
                &title_style(24.0, 30.0),
                Some(w - margin * 2.0),
            ),
            x: margin,
            y: 16.0,
            color: TEXT,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "Per-layer dirty tracking + offscreen composition",
                &body_style(12.0, 16.0),
                Some(w - margin * 2.0),
            ),
            x: margin,
            y: 48.0,
            color: TEXT_DIM,
        },
    );
}

pub fn build_info_card(comp: &mut plev::compositor::Compositor, _w: f32, margin: f32) {
    let header_h = 70.0;
    let card_x = margin;
    let card_y = header_h + 20.0;
    let card_w = 340.0;
    let card_h = 180.0;

    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card_x,
            y: card_y,
            w: card_w,
            h: card_h,
            color: SURFACE,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card_x + 1.0,
            y: card_y,
            w: card_w - 2.0,
            h: 2.0,
            color: GREEN,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "LAYER INFO",
                &card_title_style(13.0, 17.0),
                Some(card_w - 32.0),
            ),
            x: card_x + 16.0,
            y: card_y + 14.0,
            color: GREEN,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card_x + 16.0,
            y: card_y + 36.0,
            w: card_w - 32.0,
            h: 1.0,
            color: DIVIDER,
        },
    );

    let info_x = card_x + 16.0;
    let info_max_w = card_w - 32.0;
    let mut iy = card_y + 50.0;

    let layer_lines: &[(&str, [f32; 4])] = &[
        ("Layer -1 (bg): static dot grid, z_order=-1", TEXT_MID),
        ("Layer  0 (default): content + UI, z_order=0", TEXT_MID),
        ("Layer  1 (fg): animated, z_order=1, opacity=80%", TEXT_MID),
    ];

    for (line, color) in layer_lines {
        comp.push_to_layer(
            LayerId::DEFAULT,
            SceneNode::Rect {
                x: info_x,
                y: iy + 4.0,
                w: 4.0,
                h: 4.0,
                color: GREEN,
            },
        );
        comp.push_to_layer(
            LayerId::DEFAULT,
            SceneNode::Text {
                key: TextNodeKey::from_style(
                    line,
                    &code_style(12.0, 16.0),
                    Some(info_max_w - 16.0),
                ),
                x: info_x + 12.0,
                y: iy,
                color: *color,
            },
        );
        iy += 28.0;
    }

    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card_x + 16.0,
            y: iy + 4.0,
            w: card_w - 32.0,
            h: 1.0,
            color: DIVIDER,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "Unchanged layers = zero GPU work per frame",
                &body_style(11.0, 14.0),
                Some(info_max_w),
            ),
            x: info_x,
            y: iy + 14.0,
            color: ACCENT_DIM,
        },
    );
}

pub fn build_footer(comp: &mut plev::compositor::Compositor, w: f32, h: f32, margin: f32) {
    let footer_h = 32.0;
    let footer_y = h - footer_h;
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: 0.0,
            y: footer_y - 1.0,
            w,
            h: 1.0,
            color: DIVIDER,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: 0.0,
            y: footer_y,
            w,
            h: footer_h,
            color: FOOTER_BG,
        },
    );
    comp.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "Per-layer offscreen textures  |  FxHash dirty tracking  |  Composite pass",
                &body_style(11.0, 15.0),
                Some(w - margin * 2.0),
            ),
            x: margin,
            y: footer_y + 9.0,
            color: TEXT_DIM,
        },
    );
}

pub fn build_foreground(
    comp: &mut plev::compositor::Compositor,
    fg_layer: plev::compositor::LayerId,
    frame_count: u64,
    w: f32,
    h: f32,
) {
    let phase = (frame_count / 60) % 4;
    let (fx, fy) = match phase {
        0 => (50.0, 120.0),
        1 => (w / 2.0 - 100.0, 120.0),
        2 => (w / 2.0 - 100.0, h / 2.0 - 40.0),
        _ => (50.0, h / 2.0 - 40.0),
    };

    let fg_card_w = 300.0;
    let fg_card_h = 80.0;

    comp.push_to_layer(
        fg_layer,
        SceneNode::Rect {
            x: fx,
            y: fy,
            w: fg_card_w,
            h: fg_card_h,
            color: SURFACE,
        },
    );
    comp.push_to_layer(
        fg_layer,
        SceneNode::Rect {
            x: fx + 1.0,
            y: fy,
            w: fg_card_w - 2.0,
            h: 2.0,
            color: ACCENT,
        },
    );
    comp.push_to_layer(
        fg_layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                &format!("FOREGROUND (phase {})", phase),
                &card_title_style(14.0, 18.0),
                Some(fg_card_w - 20.0),
            ),
            x: fx + 10.0,
            y: fy + 14.0,
            color: ACCENT,
        },
    );
    comp.push_to_layer(
        fg_layer,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                "Dynamic layer, 80% opacity, z_order=1",
                &body_style(11.0, 14.0),
                Some(fg_card_w - 20.0),
            ),
            x: fx + 10.0,
            y: fy + 40.0,
            color: TEXT_DIM,
        },
    );
    comp.push_to_layer(
        fg_layer,
        SceneNode::Rect {
            x: fx + 10.0,
            y: fy + fg_card_h - 12.0,
            w: 60.0,
            h: 3.0,
            color: [0.0, 0.85, 0.95, 1.0],
        },
    );
}
