//! HOFF "dark glass" drawing recipes shared by every component:
//! edge-light borders (top-lit rim that fades out), analytic drop-shadow
//! stacks (`SceneNode::Shadow`), glass surfaces and real-shaping text
//! measurement helpers.

#[cfg(test)]
mod tests;

use crate::theme::ShadowSpec;
use plev::color::Color;
use plev::compositor::{Compositor, LayerId, SceneNode};
use plev::text::{TextMeasurer, TextStyle};

/// Push one CSS-like box-shadow layer. `spread` is emulated by
/// expanding/shrinking the casting rect (plev shadows have no spread).
// Geometry args mirror SceneNode::Shadow's fields; a rect bag would be
// repacked at every call site (same trade-off as plev's card.rs).
#[allow(clippy::too_many_arguments)]
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
// Same geometry-mirrors-SceneNode signature as `shadow` above.
#[allow(clippy::too_many_arguments)]
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
// Geometry args mirror SceneNode::RoundedRect's fields (see card.rs).
#[allow(clippy::too_many_arguments)]
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
// Geometry args mirror SceneNode::RoundedRect's fields (see card.rs).
#[allow(clippy::too_many_arguments)]
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

/// Truncate `s` with an ellipsis so it fits on one line of `avail` px,
/// measured with the SAME [`TextStyle`] the caller draws with (real shaping
/// via [`measure_text`], not a per-char estimate). Keeps rows strictly
/// single-line.
pub fn truncate_to_width(s: &str, avail: f32, style: &TextStyle) -> String {
    if measure_text(s, style) <= avail {
        return s.to_string();
    }
    let chars: Vec<char> = s.chars().collect();
    let candidate = |n: usize| -> String {
        let t: String = chars[..n].iter().collect();
        format!("{}\u{2026}", t.trim_end())
    };
    // Largest prefix whose "prefix…" really fits, by binary search on the
    // char count (each probe is a cached real measurement).
    let (mut lo, mut hi) = (0usize, chars.len().saturating_sub(1));
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if measure_text(&candidate(mid), style) <= avail {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    candidate(lo)
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
