//! Render logic for the layers demo: each window's scene, built into its
//! host's compositor and presented through the engine's shared frame tail
//! (`WindowHost::present`, the `present_frame` every app uses).

use engine::drawing::compositor::{LayerId, SceneNode, TextNodeKey};
use engine::winit::window::WindowId;

use crate::LayersDemoApp;
use crate::palette::*;
use crate::ui;

fn clear_linear() -> [f32; 4] {
    engine::styling::color::Color::rgb(pal().bg[0], pal().bg[1], pal().bg[2]).to_linear_array()
}

/// The animated demo: a static dot grid on the background layer, the
/// chrome on the default layer, a card that moves every 60 frames on the
/// foreground layer.
pub fn render_layers(app: &mut LayersDemoApp, id: WindowId) {
    let Some(host) = app.windows.get_mut(id) else {
        return;
    };
    host.begin_frame();
    let (w, h) = host.logical_size();
    let c = host.compositor_mut();
    c.begin_frame();

    let margin = 32.0;

    // Background layer (static dot grid)
    let dot_spacing = 40.0;
    let cols = (w / dot_spacing) as i32 + 1;
    let rows = (h / dot_spacing) as i32 + 1;
    for row in 0..rows {
        for col in 0..cols {
            c.push_to_layer(
                app.bg_layer,
                SceneNode::Rect {
                    x: col as f32 * dot_spacing,
                    y: row as f32 * dot_spacing,
                    w: 1.0,
                    h: 1.0,
                    color: pal().dot,
                },
            );
        }
    }

    // Default layer (UI chrome + info card)
    ui::build_header(c, w, margin);
    ui::build_info_card(c, w, margin);
    ui::build_footer(c, w, h, margin);

    // Foreground layer (dynamic -- changes every 60 frames)
    ui::build_foreground(c, app.fg_layer, app.frame_count, w, h);
    app.frame_count += 1;

    host.present(clear_linear(), "layers");
}

/// The other window: a card at rest, drawn once and then only when the
/// window changes (a resize).
pub fn render_side(app: &mut LayersDemoApp, id: WindowId) {
    let Some(host) = app.windows.get_mut(id) else {
        return;
    };
    host.begin_frame();
    let frames = host.frames();
    let (w, h) = host.logical_size();
    let c = host.compositor_mut();
    c.begin_frame();

    let margin = 24.0;
    let card = (margin, margin, w - margin * 2.0, h - margin * 2.0);
    c.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card.0,
            y: card.1,
            w: card.2,
            h: card.3,
            color: pal().surface,
        },
    );
    c.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Rect {
            x: card.0 + 1.0,
            y: card.1,
            w: card.2 - 2.0,
            h: 2.0,
            color: pal().accent,
        },
    );
    let lines: [(&str, [f32; 4]); 4] = [
        ("THE OTHER WINDOW", pal().accent),
        (
            "a second window of the same app, on its own context",
            pal().text,
        ),
        (
            "at rest it draws nothing while the first one animates",
            pal().text_dim,
        ),
        (
            "close either window: the other stays. close both: the app exits",
            pal().text_dim,
        ),
    ];
    let mut y = card.1 + 18.0;
    for (i, (line, color)) in lines.iter().enumerate() {
        let style = if i == 0 {
            card_title_style(14.0, 18.0)
        } else {
            body_style(12.0, 16.0)
        };
        c.push_to_layer(
            LayerId::DEFAULT,
            SceneNode::Text {
                key: TextNodeKey::from_style(line, &style, Some(card.2 - 32.0)),
                x: card.0 + 16.0,
                y,
                color: *color,
            },
        );
        y += if i == 0 { 30.0 } else { 22.0 };
    }
    c.push_to_layer(
        LayerId::DEFAULT,
        SceneNode::Text {
            key: TextNodeKey::from_style(
                &format!("frames drawn by this window: {}", frames + 1),
                &code_style(11.0, 14.0),
                Some(card.2 - 32.0),
            ),
            x: card.0 + 16.0,
            y: card.1 + card.3 - 30.0,
            color: pal().accent_dim,
        },
    );

    host.present(clear_linear(), "layers_side");
}
