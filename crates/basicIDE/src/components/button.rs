//! HOFF pill button (`styles/blocks/button.sass`): radius-32 pill,
//! bg rgba(40,40,40,.70), edge-light 1.5px rgba(255,255,255,.1) top-lit,
//! label base-2sm (14/600) `$text-secondary`; hover bg rgba(248,248,248,.10)
//! and text/icon .76.

use super::hoff;
use crate::theme::Theme;
use plev::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};

/// Button visual style variant.
// Catálogo de design: variantes ainda sem uso nas views ficam disponíveis.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonKind {
    /// Default glass pill.
    Glass,
    /// Transparent at rest; chip bg on hover (social chip).
    Ghost,
    /// Glass pill with #BD3027 label (unfollow / destructive).
    Danger,
}

/// Button size — heights from the spec: tabs buttons 36, button 44, medium 52.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

impl ButtonSize {
    fn height(self) -> f32 {
        match self {
            ButtonSize::Sm => 36.0,
            ButtonSize::Md => 44.0,
            ButtonSize::Lg => 52.0,
        }
    }
    fn pad_x(self) -> f32 {
        match self {
            ButtonSize::Sm => 16.0,
            ButtonSize::Md => 24.0,
            ButtonSize::Lg => 32.0,
        }
    }
}

/// Label style: base-2sm (14px / 1.4 / 600).
const FONT_SIZE: f32 = 14.0;
const LINE_H: f32 = 14.0 * 1.4;
const WEIGHT: u16 = 600;

/// Draw a button on the default layer and return its bounding box.
pub fn draw(
    compositor: &mut Compositor,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
    kind: ButtonKind,
    size: ButtonSize,
    hovered: bool,
    disabled: bool,
) -> (f32, f32, f32, f32) {
    draw_to_layer(
        compositor,
        LayerId::DEFAULT,
        theme,
        x,
        y,
        label,
        kind,
        size,
        hovered,
        disabled,
    )
}

/// Draw a button on an arbitrary layer (overlays) and return its bounding box.
#[allow(clippy::too_many_arguments)]
pub fn draw_to_layer(
    compositor: &mut Compositor,
    layer: LayerId,
    theme: &Theme,
    x: f32,
    y: f32,
    label: &str,
    kind: ButtonKind,
    size: ButtonSize,
    hovered: bool,
    disabled: bool,
) -> (f32, f32, f32, f32) {
    let btn_h = size.height();
    let pad_x = size.pad_x();
    let btn_w = hoff::text_width(label, FONT_SIZE) + pad_x * 2.0;
    let radius = theme.radius_pill.min(btn_h / 2.0);
    let hovered = hovered && !disabled;

    let (bg, text_col) = match kind {
        ButtonKind::Glass => (
            if hovered {
                theme.button_hover_bg
            } else {
                theme.button_bg
            },
            if hovered {
                theme.text_active
            } else {
                theme.text_secondary
            },
        ),
        ButtonKind::Ghost => (
            if hovered {
                theme.surface_hover
            } else {
                plev::color::Color::TRANSPARENT
            },
            if hovered {
                theme.text_active
            } else {
                theme.text_default
            },
        ),
        ButtonKind::Danger => (
            if hovered {
                theme.button_hover_bg
            } else {
                theme.button_bg
            },
            theme.accent_red,
        ),
    };

    let mut bg = bg.to_array();
    let mut text_col = text_col.to_array();
    if disabled {
        bg[3] *= 0.45;
        text_col[3] *= 0.45;
    }

    compositor.push_to_layer(
        layer,
        SceneNode::RoundedRect {
            x,
            y,
            w: btn_w,
            h: btn_h,
            color: bg,
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        },
    );

    // Edge-light rim: 1.5px rgba(255,255,255,.1), mask 175deg -> 50%.
    if kind != ButtonKind::Ghost && !disabled {
        hoff::edge_light(
            compositor,
            layer,
            x,
            y,
            btn_w,
            btn_h,
            radius,
            1.5,
            theme.edge_strong,
        );
    }

    compositor.push_to_layer(
        layer,
        SceneNode::Text {
            key: TextNodeKey::new(label, FONT_SIZE, LINE_H, None).with_weight(WEIGHT),
            x: x + pad_x,
            y: y + (btn_h - LINE_H) / 2.0,
            color: text_col,
        },
    );

    (x, y, btn_w, btn_h)
}
