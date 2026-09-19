//! The HOFF dark-glass drawing recipes every widget shares: edge-lit
//! pills and surfaces, inset key-light, shadow stacks by elevation, the
//! focus ring, and the rounded-rect shorthands. Every dimension and color
//! comes from the [`Theme`]; a recipe never restates a number the tokens
//! own.
//!
//! Two shapes of API, on purpose: the `*_nodes` functions return scene
//! nodes for widgets that batch and push themselves; the `Compositor`
//! functions push straight to a layer for app views.

use engine::compositor::{Compositor, LayerId, SceneNode};
use engine::theme::{Elevation, ShadowSpec, Theme};

use crate::core::Rect;

// ---------------------------------------------------------------------------
// Rounded-rect node shorthands
// ---------------------------------------------------------------------------

/// Solid rounded rect ([`SceneNode::RoundedRect`] without border). The
/// compositor preserves push order across primitive types, so icons
/// (paths) pushed after this stack on top of it.
pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, radius: f32, color: [f32; 4]) -> SceneNode {
    SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color,
        corner_radius: radius,
        border_width: 0.0,
        border_color: [0.0; 4],
    }
}

/// Border-only rounded rect: transparent fill with an SDF border ring,
/// which composites OVER whatever is underneath, exactly like a stroked
/// path would (the ring sits inside the bounds, like the SDF border).
pub fn rounded_rect_stroke(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: [f32; 4],
    width: f32,
) -> SceneNode {
    SceneNode::RoundedRect {
        x,
        y,
        w,
        h,
        color: [0.0; 4],
        corner_radius: radius,
        border_width: width,
        border_color: color,
    }
}

// ---------------------------------------------------------------------------
// Glass
// ---------------------------------------------------------------------------

/// HOFF edge-light pill: a border that only exists at the top, fading
/// out downward (the CSS original is a masked 165 to 178 degree border).
///
/// Emitted as two SDF nodes: an `edge` to transparent vertical
/// [`GradientRect`] underlay the size of `rect`, then the surface fill
/// inset by `width`. The translucent fill lets the underlay shine
/// through, which doubles as the HOFF inset key-light.
///
/// [`GradientRect`]: SceneNode::GradientRect
pub fn glass_pill(
    rect: Rect,
    radius: f32,
    edge: [f32; 4],
    width: f32,
    fill: [f32; 4],
) -> [SceneNode; 2] {
    [
        SceneNode::GradientRect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            color: edge,
            color2: [edge[0], edge[1], edge[2], 0.0],
            // CSS 180deg: first stop (the lit edge) at the top.
            angle_deg: 180.0,
            corner_radius: radius,
            border_width: 0.0,
            border_color: [0.0; 4],
        },
        SceneNode::RoundedRect {
            x: rect.x + width,
            y: rect.y + width,
            w: (rect.w - width * 2.0).max(0.0),
            h: (rect.h - width * 2.0).max(0.0),
            color: fill,
            corner_radius: (radius - width).max(0.0),
            border_width: 0.0,
            border_color: [0.0; 4],
        },
    ]
}

/// Edge-light rim as a masked border ring (the alternative to
/// [`glass_pill`] when the fill is opaque and cannot let a gradient
/// underlay through): full strength on the top 40% of the height, half
/// strength from 40% to 65%, nothing below. Pushed to `layer`.
pub fn edge_light(
    c: &mut Compositor,
    layer: LayerId,
    rect: Rect,
    radius: f32,
    width: f32,
    color: [f32; 4],
) {
    let ring = |c: &mut Compositor, col: [f32; 4]| {
        c.push_to_layer(
            layer,
            rounded_rect_stroke(rect.x, rect.y, rect.w, rect.h, radius, col, width),
        );
    };
    let pad = width;
    c.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: rect.x - pad,
            y: rect.y - pad,
            w: rect.w + pad * 2.0,
            h: rect.h * 0.40 + pad,
        },
    );
    ring(c, color);
    c.push_to_layer(layer, SceneNode::PopClip);

    c.push_to_layer(
        layer,
        SceneNode::PushClip {
            x: rect.x - pad,
            y: rect.y + rect.h * 0.40,
            w: rect.w + pad * 2.0,
            h: rect.h * 0.25,
        },
    );
    ring(c, [color[0], color[1], color[2], color[3] * 0.5]);
    c.push_to_layer(layer, SceneNode::PopClip);
}

/// HOFF inset key-light: `inset 2px 4px 16px rgba(248,248,248,.06)`, a
/// soft highlight bleeding in from the top-left, the glint that makes a
/// glass surface read as lit rather than painted.
pub fn inset_keylight(c: &mut Compositor, layer: LayerId, rect: Rect, radius: f32, theme: &Theme) {
    let g = &theme.glass;
    c.push_to_layer(
        layer,
        SceneNode::Shadow {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            corner_radius: radius,
            blur_radius: theme.spacing.lg,
            offset: [theme.spacing.xs / 2.0, theme.spacing.xs],
            color: g.inset_highlight.0,
            inset: true,
        },
    );
}

/// Glass surface pushed to `layer`: translucent fill with the top-lit rim
/// and the inset key-light, the HOFF card / panel / sheet shell.
/// `fill` is usually `theme.glass.surface` (rest) or `surface_hover`.
pub fn glass_surface(
    c: &mut Compositor,
    layer: LayerId,
    rect: Rect,
    radius: f32,
    fill: [f32; 4],
    theme: &Theme,
) {
    let edge = theme.glass.edge_soft.0;
    for node in glass_pill(rect, radius, edge, theme.control.edge_width, fill) {
        c.push_to_layer(layer, node);
    }
    inset_keylight(c, layer, rect, radius, theme);
}

// ---------------------------------------------------------------------------
// Shadows
// ---------------------------------------------------------------------------

/// One css-like box-shadow layer as a scene node, with the spec's spread
/// folded into the casting rect. `None` when spread ate the surface.
pub fn shadow_node(rect: Rect, radius: f32, spec: &ShadowSpec) -> Option<SceneNode> {
    let (x, y, w, h) = spec.cast_rect(rect.x, rect.y, rect.w, rect.h)?;
    Some(SceneNode::Shadow {
        x,
        y,
        w,
        h,
        corner_radius: (radius + spec.spread).max(0.0),
        blur_radius: spec.blur,
        offset: spec.offset,
        color: spec.color.0,
        inset: false,
    })
}

/// The shadow stack a surface at `elevation` casts, bottom layer first,
/// pushed to `layer` before the surface itself.
pub fn shadow_stack(
    c: &mut Compositor,
    layer: LayerId,
    rect: Rect,
    radius: f32,
    elevation: Elevation,
    theme: &Theme,
) {
    for spec in theme.shadows.stack(elevation) {
        if let Some(node) = shadow_node(rect, radius, spec) {
            c.push_to_layer(layer, node);
        }
    }
}

/// Floating-menu drop shadow as a single node (dropdowns, selects,
/// context menus): `theme.shadows.floating`.
pub fn menu_shadow(rect: Rect, radius: f32, theme: &Theme) -> SceneNode {
    shadow_node(rect, radius, &theme.shadows.floating)
        .unwrap_or_else(|| rounded_rect(rect.x, rect.y, 0.0, 0.0, 0.0, [0.0; 4]))
}

// ---------------------------------------------------------------------------
// Focus
// ---------------------------------------------------------------------------

/// Keyboard-focus ring: a `control.focus_ring_width` stroke in the theme
/// accent, floating `control.focus_ring_offset` outside the control's
/// visual rect and following its corner radius. Every form widget draws
/// this from `render` when focused, so focus reads identically across the
/// kit.
pub fn focus_ring(rect: Rect, radius: f32, theme: &Theme) -> SceneNode {
    let inflate = theme.control.focus_ring_offset + theme.control.focus_ring_width;
    rounded_rect_stroke(
        rect.x - inflate,
        rect.y - inflate,
        rect.w + inflate * 2.0,
        rect.h + inflate * 2.0,
        radius + inflate,
        theme.colors.accent.0,
        theme.control.focus_ring_width,
    )
}
