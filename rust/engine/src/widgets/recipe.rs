//! The drawing recipes every widget shares: rimmed pills and surfaces,
//! the inset key-light, shadow stacks by elevation, the focus ring, and
//! the rounded-rect shorthands. How a rim runs and whether a surface is
//! lit are theme tokens (`glass.rim_fade`, `glass.keylight`): the HOFF
//! dark glass and the core kit draw through the same recipes. Every dimension and color
//! comes from the [`Theme`]; a recipe never restates a number the tokens
//! own.
//!
//! Two shapes of API, on purpose: the `*_nodes` functions return scene
//! nodes for widgets that batch and push themselves; the `Compositor`
//! functions push straight to a layer for app views.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode};
use crate::styling::theme::{Elevation, ShadowSpec, Theme};

use crate::widgets::widget::Rect;

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

/// Rimmed pill: a `width` rim of `edge` around a `fill`. How the rim
/// runs down the sides is the theme's `glass.rim_fade`: the HOFF glass
/// (0) keeps it only at the top, fading out downward (the CSS original is
/// a masked 165 to 178 degree border); the core kit (1) keeps it whole,
/// a plain `border`.
///
/// Emitted as two SDF nodes: an `edge` to faded vertical
/// [`GradientRect`] underlay the size of `rect`, then the surface fill
/// inset by `width`. A translucent fill lets the underlay shine through,
/// which doubles as the HOFF inset key-light.
///
/// [`GradientRect`]: SceneNode::GradientRect
pub fn glass_pill(
    rect: Rect,
    radius: f32,
    edge: [f32; 4],
    width: f32,
    fill: [f32; 4],
    theme: &Theme,
) -> [SceneNode; 2] {
    let fade = theme.glass.rim_fade.clamp(0.0, 1.0);
    [
        SceneNode::GradientRect {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            color: edge,
            color2: [edge[0], edge[1], edge[2], edge[3] * fade],
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

/// The theme's inset key-light (`glass.keylight`) over `rect`: in the
/// HOFF glass `inset 2px 4px 16px rgba(248,248,248,.06)`, a soft
/// highlight bleeding in from the top-left, the glint that makes a glass
/// surface read as lit rather than painted. A theme without one (the core
/// kit) draws nothing.
pub fn inset_keylight(c: &mut Compositor, layer: LayerId, rect: Rect, radius: f32, theme: &Theme) {
    let Some(spec) = theme.glass.keylight.as_ref() else {
        return;
    };
    if let Some(node) = shadow_node(rect, radius, spec) {
        c.push_to_layer(layer, node);
    }
}

/// Surface pushed to `layer`: the fill inside the theme's rim
/// ([`glass_pill`]) under its key-light ([`inset_keylight`]). The HOFF
/// card / panel / sheet shell is a translucent fill with a top-lit rim;
/// the core kit's, a plain fill inside a 1px `border` (Card :36).
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
    for node in glass_pill(rect, radius, edge, theme.control.edge_width, fill, theme) {
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
    if spec.inset {
        // Inset: the surface itself is the mask; spread only moves the
        // inner edge by a hair, which the sdf blur absorbs.
        return Some(SceneNode::Shadow {
            x: rect.x,
            y: rect.y,
            w: rect.w,
            h: rect.h,
            corner_radius: radius,
            blur_radius: spec.blur,
            offset: spec.offset,
            color: spec.color.0,
            inset: true,
        });
    }
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

/// A css `box-shadow` list: every layer, first in the list drawn last
/// (on top), like the browser does. Inset layers go after the surface;
/// drop layers before it. Call `shadow_drops` before the fill and
/// `shadow_insets` after.
pub fn shadow_drops(
    c: &mut Compositor,
    layer: LayerId,
    rect: Rect,
    radius: f32,
    specs: &[ShadowSpec],
) {
    for spec in specs.iter().rev().filter(|s| !s.inset) {
        if let Some(node) = shadow_node(rect, radius, spec) {
            c.push_to_layer(layer, node);
        }
    }
}

/// The inset layers of a css `box-shadow` list, pushed over the surface.
pub fn shadow_insets(
    c: &mut Compositor,
    layer: LayerId,
    rect: Rect,
    radius: f32,
    specs: &[ShadowSpec],
) {
    for spec in specs.iter().rev().filter(|s| s.inset) {
        if let Some(node) = shadow_node(rect, radius, spec) {
            c.push_to_layer(layer, node);
        }
    }
}

/// Vertical two-stop gradient fill (css `bg-gradient-to-b`), rounded.
pub fn gradient_fill(rect: Rect, radius: f32, top: [f32; 4], bottom: [f32; 4]) -> SceneNode {
    SceneNode::GradientRect {
        x: rect.x,
        y: rect.y,
        w: rect.w,
        h: rect.h,
        color: top,
        color2: bottom,
        angle_deg: 180.0,
        corner_radius: radius,
        border_width: 0.0,
        border_color: [0.0; 4],
    }
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

/// The floating-menu shadow (dropdowns, selects, context menus):
/// `theme.shadows.floating`, every layer, in the order to push them
/// before the surface.
pub fn menu_shadow(rect: Rect, radius: f32, theme: &Theme) -> Vec<SceneNode> {
    theme
        .shadows
        .floating
        .iter()
        .filter_map(|spec| shadow_node(rect, radius, spec))
        .collect()
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

/// The keyboard-focus ring drawn inside `rect` instead of around it: the
/// same stroke, its outer edge on the rect's edge and its outer corners at
/// `radius`. For a row or a segment packed edge to edge with its
/// neighbors (a tree row, a tab of a strip), where the ring around it
/// would cross into the next one.
pub fn focus_ring_inside(rect: Rect, radius: f32, theme: &Theme) -> SceneNode {
    let inflate = theme.control.focus_ring_offset + theme.control.focus_ring_width;
    focus_ring(rect.inset(inflate), (radius - inflate).max(0.0), theme)
}
