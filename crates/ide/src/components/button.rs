//! HOFF pill button (`styles/blocks/button.sass`): radius-32 pill,
//! bg rgba(40,40,40,.70), edge-light 1.5px rgba(255,255,255,.1) top-lit,
//! label base-2sm (14/600) `$text-secondary`; hover bg rgba(248,248,248,.10)
//! and text/icon .76.

use super::hoff;
use crate::theme::Theme;
use engine::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::text::TextStyle;

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

/// The one label style — used for BOTH measuring and drawing so the pill
/// is always sized by the same shaping that rasterizes the label.
fn label_style() -> TextStyle {
    TextStyle::new(FONT_SIZE)
        .with_line_height(LINE_H)
        .with_weight(WEIGHT)
}

/// Width the pill takes for `label` at `size` — the exact measurement
/// `draw_to_layer` uses, exposed so callers (modal, header) can position
/// buttons without re-deriving widths from heuristics.
pub fn width_for(label: &str, size: ButtonSize) -> f32 {
    hoff::measure_text(label, &label_style()) + size.pad_x() * 2.0
}

/// Draw a button on the default layer and return its bounding box.
// Thin forwarder to `draw_to_layer`; shares its flat parameter list.
#[allow(clippy::too_many_arguments)]
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
    let style = label_style();
    let btn_w = hoff::measure_text(label, &style) + pad_x * 2.0;
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
                engine::color::Color::TRANSPARENT
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
            key: TextNodeKey::from_style(label, &style, None),
            x: x + pad_x,
            y: y + (btn_h - LINE_H) / 2.0,
            color: text_col,
        },
    );

    (x, y, btn_w, btn_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::compositor::LayerId;
    use engine::text::TextMeasurer;

    /// Regression for the "label leaks out of the pill" bug: the drawn
    /// shape must be at least the REAL measured label width plus both
    /// paddings. With the old per-char heuristic, "Commit" @14/600
    /// (real ~53.7px, estimated 48.7px) overflowed a Md pill.
    #[test]
    fn pill_is_wide_enough_for_real_shaped_label() {
        let mut c = Compositor::new();
        c.begin_frame();
        let (_, _, w, _) = draw(
            &mut c,
            &crate::theme::DARK,
            0.0,
            0.0,
            "Commit",
            ButtonKind::Glass,
            ButtonSize::Md,
            false,
            false,
        );
        let (text_w, _) = TextMeasurer::measure_styled("Commit", &label_style(), None);
        let pad = ButtonSize::Md.pad_x();
        assert!(
            w >= text_w + 2.0 * pad - 1e-3,
            "pill width {w} must fit measured label {text_w} + 2*{pad} padding"
        );
        // And the drawn Text node must carry the same style we measured
        // with (weight 600), or measurement != rendering.
        let text_key = c
            .layer(LayerId::DEFAULT)
            .unwrap()
            .nodes()
            .iter()
            .find_map(|n| match n {
                SceneNode::Text { key, .. } => Some(key.clone()),
                _ => None,
            })
            .expect("button must draw its label");
        assert_eq!(text_key.font_weight, WEIGHT);
    }

    /// `width_for` must agree exactly with the width `draw` produces —
    /// modal/header position buttons with it.
    #[test]
    fn width_for_matches_drawn_width() {
        for size in [ButtonSize::Sm, ButtonSize::Md, ButtonSize::Lg] {
            let mut c = Compositor::new();
            c.begin_frame();
            let (_, _, w, _) = draw(
                &mut c,
                &crate::theme::DARK,
                0.0,
                0.0,
                "Discard changes",
                ButtonKind::Danger,
                size,
                false,
                false,
            );
            assert_eq!(w, width_for("Discard changes", size));
        }
    }
}
