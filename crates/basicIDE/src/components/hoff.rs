//! HOFF "dark glass" drawing recipes shared by every component:
//! edge-light borders (top-lit rim that fades out), analytic drop-shadow
//! stacks (`SceneNode::Shadow`) and glass surfaces.

use crate::theme::ShadowSpec;
use plev::color::Color;
use plev::compositor::{Compositor, LayerId, SceneNode};
use plev::text::{TextMeasurer, TextStyle};

/// Push one CSS-like box-shadow layer. `spread` is emulated by
/// expanding/shrinking the casting rect (plev shadows have no spread).
pub fn shadow(
    compositor: &mut Compositor,
    layer: LayerId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    spec: &ShadowSpec,
) {
    let s = spec.spread;
    let (w2, h2) = ((w + 2.0 * s).max(0.0), (h + 2.0 * s).max(0.0));
    if w2 <= 0.0 || h2 <= 0.0 {
        return;
    }
    compositor.push_to_layer(
        layer,
        SceneNode::Shadow {
            x: x - s,
            y: y - s,
            w: w2,
            h: h2,
            corner_radius: (radius + s).max(0.0),
            blur_radius: spec.blur,
            offset: spec.offset,
            color: spec.color.to_array(),
            inset: false,
        },
    );
}

/// Push a whole box-shadow stack (e.g. [`crate::theme::SHADOW_MODAL`]).
pub fn shadow_stack(
    compositor: &mut Compositor,
    layer: LayerId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    specs: &[ShadowSpec],
) {
    for spec in specs {
        shadow(compositor, layer, x, y, w, h, radius, spec);
    }
}

/// Edge-light border — the HOFF signature: a 1–1.5px white-alpha border
/// that only exists at the top, fading away by ~half the height
/// (`border + mask-image: linear-gradient(175deg, black, transparent 50%)`).
///
/// Emulated with a border-only rounded rect drawn twice under clip rects:
/// full strength on the top 40%, half strength from 40% to 65%.
pub fn edge_light(
    compositor: &mut Compositor,
    layer: LayerId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    border_width: f32,
    color: Color,
) {
    let c = color.to_array();
    let ring = |comp: &mut Compositor, col: [f32; 4]| {
        comp.push_to_layer(
            layer,
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color: [0.0; 4],
                corner_radius: radius,
                border_width,
                border_color: col,
            },
        );
    };

    // Top 40%: full strength.
    compositor.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: x - 1.0,
            y: y - 1.0,
            w: w + 2.0,
            h: h * 0.40 + 1.0,
        },
    );
    ring(compositor, c);
    compositor.push_to_layer(layer, SceneNode::PopClip);

    // 40%..65%: faded tail of the mask.
    compositor.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: x - 1.0,
            y: y + h * 0.40,
            w: w + 2.0,
            h: h * 0.25,
        },
    );
    ring(compositor, [c[0], c[1], c[2], c[3] * 0.5]);
    compositor.push_to_layer(layer, SceneNode::PopClip);
}

/// HOFF inset key-light: `inset 2px 4px 16px rgba(248,248,248,.06)` — a soft
/// highlight bleeding in from the top-left, the glint that makes a glass
/// surface read as lit rather than painted (the same recipe plev's Card
/// widget uses on its social shell).
pub fn inset_keylight(
    compositor: &mut Compositor,
    layer: LayerId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
) {
    compositor.push_to_layer(
        layer,
        SceneNode::Shadow {
            x,
            y,
            w,
            h,
            corner_radius: radius,
            blur_radius: 16.0,
            offset: [2.0, 4.0],
            color: [248.0 / 255.0, 248.0 / 255.0, 248.0 / 255.0, 0.06],
            inset: true,
        },
    );
}

/// Glass surface: background fill + optional edge-light rim.
/// `edge` is `(border_width, color)`.
pub fn glass(
    compositor: &mut Compositor,
    layer: LayerId,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    bg: Color,
    edge: Option<(f32, Color)>,
) {
    compositor.push_to_layer(
        layer,
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color: bg.to_array(),
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        },
    );
    if let Some((bw, color)) = edge {
        edge_light(compositor, layer, x, y, w, h, radius, bw, color);
    }
}

/// Real single-line text width via the engine's shaper
/// ([`plev::text::TextMeasurer`]): same `FontSystem`, faces (Rubik default
/// family) and cache as the rasterizer, so a shape sized with this never
/// disagrees with the glyphs drawn on top of it.
///
/// Golden rule: build ONE [`TextStyle`] per label and use it BOTH here and
/// in the draw call (`TextNodeKey::from_style` with the same style), so
/// measurement == rendering by construction.
pub fn measure_text(text: &str, style: &TextStyle) -> f32 {
    TextMeasurer::measure_styled(text, style, None).0
}

/// Scrollbar thumb — 4px wide, rgba($n2,.25), rounded; no track.
pub fn draw_scrollbar(
    compositor: &mut Compositor,
    theme: &crate::theme::Theme,
    x: f32,
    y: f32,
    h: f32,
    scroll: &plev::scroll::ScrollState,
) {
    let thumb_h = (h * scroll.thumb_ratio()).max(24.0);
    let thumb_y = y + (h - thumb_h) * scroll.thumb_position();
    compositor.push(SceneNode::RoundedRect {
        x,
        y: thumb_y,
        w: 4.0,
        h: thumb_h,
        color: theme.text_placeholder.to_array(),
        corner_radius: 2.0,
        border_width: 0.0,
        border_color: [0.0; 4],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{SHADOW_MENU, SHADOW_MODAL};

    fn nodes(compositor: &Compositor) -> &[SceneNode] {
        compositor.layer(LayerId::DEFAULT).unwrap().nodes()
    }

    #[test]
    fn shadow_emulates_negative_spread_by_shrinking_rect() {
        let mut c = Compositor::new();
        c.begin_frame();
        // SHADOW_MENU: 0 24px 32px -12px rgba(18,18,18,.10)
        shadow(
            &mut c,
            LayerId::DEFAULT,
            100.0,
            50.0,
            240.0,
            180.0,
            24.0,
            &SHADOW_MENU,
        );
        let n = nodes(&c);
        assert_eq!(n.len(), 1);
        match &n[0] {
            SceneNode::Shadow {
                x,
                y,
                w,
                h,
                corner_radius,
                blur_radius,
                offset,
                color,
                ..
            } => {
                // spread -12 shrinks the casting rect by 12px each side
                assert_eq!((*x, *y), (112.0, 62.0));
                assert_eq!((*w, *h), (216.0, 156.0));
                assert_eq!(*corner_radius, 12.0);
                assert_eq!(*blur_radius, 32.0);
                assert_eq!(*offset, [0.0, 24.0]);
                assert!((color[3] - 0.10).abs() < 1e-6);
            }
            other => panic!("expected Shadow node, got {other:?}"),
        }
    }

    #[test]
    fn shadow_with_spread_collapsing_rect_is_skipped() {
        let mut c = Compositor::new();
        c.begin_frame();
        shadow(
            &mut c,
            LayerId::DEFAULT,
            0.0,
            0.0,
            20.0,
            20.0,
            8.0,
            &SHADOW_MODAL[0],
        );
        // spread -16 on a 20px rect collapses it -> nothing drawn
        assert!(nodes(&c).is_empty());
    }

    #[test]
    fn shadow_stack_pushes_one_node_per_spec() {
        let mut c = Compositor::new();
        c.begin_frame();
        shadow_stack(
            &mut c,
            LayerId::DEFAULT,
            0.0,
            0.0,
            400.0,
            216.0,
            32.0,
            &SHADOW_MODAL,
        );
        let count = nodes(&c)
            .iter()
            .filter(|n| matches!(n, SceneNode::Shadow { .. }))
            .count();
        assert_eq!(count, SHADOW_MODAL.len());
    }

    #[test]
    fn edge_light_draws_two_masked_rings() {
        let mut c = Compositor::new();
        c.begin_frame();
        let color = Color::rgba(1.0, 1.0, 1.0, 0.10);
        edge_light(
            &mut c,
            LayerId::DEFAULT,
            0.0,
            0.0,
            100.0,
            40.0,
            12.0,
            1.5,
            color,
        );
        let n = nodes(&c);
        // clip + ring + pop, twice
        assert_eq!(n.len(), 6);
        assert!(matches!(n[0], SceneNode::PushClip { .. }));
        assert!(matches!(n[2], SceneNode::PopClip));
        assert!(matches!(n[3], SceneNode::PushClip { .. }));
        assert!(matches!(n[5], SceneNode::PopClip));
        let ring_alpha = |node: &SceneNode| match node {
            SceneNode::RoundedRect {
                border_width,
                border_color,
                color,
                ..
            } => {
                assert_eq!(*border_width, 1.5);
                assert_eq!(color[3], 0.0, "ring must be border-only");
                border_color[3]
            }
            other => panic!("expected RoundedRect ring, got {other:?}"),
        };
        let full = ring_alpha(&n[1]);
        let faded = ring_alpha(&n[4]);
        assert!((full - 0.10).abs() < 1e-6);
        assert!((faded - 0.05).abs() < 1e-6, "tail ring is half strength");
    }

    #[test]
    fn glass_fills_background_then_rims() {
        let mut c = Compositor::new();
        c.begin_frame();
        let bg = Color::rgba(40.0 / 255.0, 40.0 / 255.0, 40.0 / 255.0, 0.7);
        let edge = Color::rgba(1.0, 1.0, 1.0, 0.05);
        glass(
            &mut c,
            LayerId::DEFAULT,
            0.0,
            0.0,
            240.0,
            120.0,
            24.0,
            bg,
            Some((1.0, edge)),
        );
        let n = nodes(&c);
        // 1 fill + 6 edge-light nodes
        assert_eq!(n.len(), 7);
        match &n[0] {
            SceneNode::RoundedRect {
                color,
                corner_radius,
                border_width,
                ..
            } => {
                assert!((color[3] - 0.7).abs() < 1e-6);
                assert_eq!(*corner_radius, 24.0);
                assert_eq!(*border_width, 0.0);
            }
            other => panic!("expected fill RoundedRect, got {other:?}"),
        }
    }

    #[test]
    fn measure_text_uses_real_shaping_not_per_char_heuristic() {
        // "Commit" at 14px weight 600 (the button label style). The old
        // per-char heuristic (`chars * size * 0.58`) gave 48.72px; the real
        // Rubik SemiBold shaping is ~53.7px (verified against the
        // rasterizer's FontSystem). Sizing pills with the heuristic made
        // labels overflow their shapes by up to ~10%.
        let style = TextStyle::new(14.0).with_weight(600);
        let measured = measure_text("Commit", &style);
        let old_heuristic = "Commit".chars().count() as f32 * 14.0 * 0.58;
        assert!(
            (measured - old_heuristic).abs() > 1.0,
            "measure_text must differ from the old heuristic: real {measured} vs heuristic {old_heuristic}"
        );
        assert!(
            (50.0..60.0).contains(&measured),
            "Commit @14/600 should shape to ~53.7px, got {measured}"
        );
    }

    #[test]
    fn measure_text_respects_font_weight() {
        // Weight is part of the style: SemiBold advances are wider than
        // Regular for the same string — the heuristic could not see this.
        let regular = measure_text("MODIFIED", &TextStyle::new(14.0));
        let semibold = measure_text("MODIFIED", &TextStyle::new(14.0).with_weight(600));
        assert!(
            semibold > regular,
            "600 ({semibold}) must measure wider than 400 ({regular})"
        );
    }

    #[test]
    fn glass_without_edge_is_a_single_fill() {
        let mut c = Compositor::new();
        c.begin_frame();
        glass(
            &mut c,
            LayerId::DEFAULT,
            0.0,
            0.0,
            100.0,
            44.0,
            16.0,
            Color::rgba(1.0, 1.0, 1.0, 0.02),
            None,
        );
        assert_eq!(nodes(&c).len(), 1);
    }
}
