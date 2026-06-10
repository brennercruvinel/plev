use crate::compositor::{SceneNode, TextNodeKey};
use crate::layout::ComputedBounds;
use crate::text::TextStyle;

use super::element::{Element, ElementKind};

// ---------------------------------------------------------------------------
// Emit SceneNodes using Taffy-computed bounds
// ---------------------------------------------------------------------------

/// Text-specific properties extracted from an Element's TextKind variant.
struct TextProps<'a> {
    content: &'a str,
    font_size: f32,
    line_height: f32,
    max_width: Option<f32>,
    truncate_chars: Option<&'a usize>,
}

/// Walk the element tree in the same preorder as `collect_layout_items`
/// (so `bounds[i]` lines up), wrapping children of `clip_children` elements
/// in PushClip/PopClip pairs.
pub(crate) fn emit_scene_nodes(
    root: &Element,
    bounds: &[ComputedBounds],
    theme: Option<&crate::theme::Theme>,
    out: &mut Vec<SceneNode>,
) {
    let mut cursor = 0usize;
    emit_element(root, bounds, theme, &mut cursor, out);
}

fn emit_element(
    element: &Element,
    bounds: &[ComputedBounds],
    theme: Option<&crate::theme::Theme>,
    cursor: &mut usize,
    out: &mut Vec<SceneNode>,
) {
    let b = &bounds[*cursor];
    *cursor += 1;

    // Resolve intent-derived color when theme is available
    let intent_color = element
        .intent
        .and_then(|intent| theme.map(|t| t.intent_color(intent)));

    match &element.kind {
        ElementKind::Div => {
            emit_div(element, b, intent_color, out);
        }
        ElementKind::Text {
            content,
            font_size,
            line_height,
            max_width,
            truncate_chars,
        } => {
            emit_text(
                element,
                b,
                intent_color,
                &TextProps {
                    content,
                    font_size: *font_size,
                    line_height: *line_height,
                    max_width: *max_width,
                    truncate_chars: truncate_chars.as_ref(),
                },
                out,
            );
        }
        ElementKind::Path { data } => {
            out.push(SceneNode::Path { data: data.clone() });
        }
        ElementKind::Image { bytes } => {
            if let Some(bytes) = bytes
                && (b.width > 0.0 || b.height > 0.0)
            {
                // Memoized by content hash; failures are remembered and
                // logged once inside the store.
                if let Ok(handle) = crate::gpu::image::load_image_bytes(bytes) {
                    out.push(SceneNode::Image {
                        x: b.x,
                        y: b.y,
                        w: b.width,
                        h: b.height,
                        image: handle,
                        corner_radius: element.style.corner_radius,
                    });
                }
            }
        }
    }

    let clip = element.style.clip_children && !element.children.is_empty();
    if clip {
        out.push(SceneNode::PushClip {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
        });
    }
    for child in &element.children {
        emit_element(child, bounds, theme, cursor, out);
    }
    if clip {
        out.push(SceneNode::PopClip);
    }
}

fn emit_div(
    element: &Element,
    b: &ComputedBounds,
    intent_color: Option<crate::color::Color>,
    out: &mut Vec<SceneNode>,
) {
    let has_bg = element.style.bg.is_some() || intent_color.is_some();
    let has_gradient = element.style.bg_gradient.is_some();
    let has_border = element.style.border > 0.0;
    let has_radius = element.style.corner_radius > 0.0;

    // Drop shadow first so the rect paints on top of it.
    if let Some(shadow) = element.style.drop_shadow
        && (b.width > 0.0 || b.height > 0.0)
    {
        out.push(SceneNode::Shadow {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
            corner_radius: element.style.corner_radius,
            blur_radius: shadow.blur,
            offset: shadow.offset,
            color: shadow.color.to_array(),
            inset: false,
        });
    }

    // Backdrop blur sits under the background fill: frosted glass is the
    // blurred backdrop showing through the translucent bg above it.
    if let Some(sigma) = element.style.backdrop_blur
        && (b.width > 0.0 || b.height > 0.0)
    {
        out.push(SceneNode::BackdropBlur {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
            corner_radius: element.style.corner_radius,
            sigma,
        });
    }

    if (has_bg || has_gradient || has_border) && (b.width > 0.0 || b.height > 0.0) {
        let bg_color = intent_color
            .or(element.style.bg)
            .unwrap_or(crate::color::Color::TRANSPARENT)
            .to_array();

        if let Some(gradient) = element.style.bg_gradient {
            out.push(SceneNode::GradientRect {
                x: b.x,
                y: b.y,
                w: b.width,
                h: b.height,
                color: gradient.from.to_array(),
                color2: gradient.to.to_array(),
                angle_deg: gradient.angle_deg,
                corner_radius: element.style.corner_radius,
                border_width: element.style.border,
                border_color: element.style.border_color.to_array(),
            });
        } else if has_radius || has_border {
            out.push(SceneNode::RoundedRect {
                x: b.x,
                y: b.y,
                w: b.width,
                h: b.height,
                color: bg_color,
                corner_radius: element.style.corner_radius,
                border_width: element.style.border,
                border_color: element.style.border_color.to_array(),
            });
        } else {
            out.push(SceneNode::Rect {
                x: b.x,
                y: b.y,
                w: b.width,
                h: b.height,
                color: bg_color,
            });
        }
    }

    // Inset shadow AFTER the fill: it composites over the background and
    // under the children (draw order follows push order).
    if let Some(shadow) = element.style.inset_shadow
        && (b.width > 0.0 || b.height > 0.0)
    {
        out.push(SceneNode::Shadow {
            x: b.x,
            y: b.y,
            w: b.width,
            h: b.height,
            corner_radius: element.style.corner_radius,
            blur_radius: shadow.blur,
            offset: shadow.offset,
            color: shadow.color.to_array(),
            inset: true,
        });
    }

    // Emit per-side borders as thin Rect nodes
    let bs = &element.style.border_sides;
    let bc = bs.color.to_array();
    if bs.top > 0.0 {
        out.push(SceneNode::Rect {
            x: b.x,
            y: b.y,
            w: b.width,
            h: bs.top,
            color: bc,
        });
    }
    if bs.bottom > 0.0 {
        out.push(SceneNode::Rect {
            x: b.x,
            y: b.y + b.height - bs.bottom,
            w: b.width,
            h: bs.bottom,
            color: bc,
        });
    }
    if bs.left > 0.0 {
        out.push(SceneNode::Rect {
            x: b.x,
            y: b.y,
            w: bs.left,
            h: b.height,
            color: bc,
        });
    }
    if bs.right > 0.0 {
        out.push(SceneNode::Rect {
            x: b.x + b.width - bs.right,
            y: b.y,
            w: bs.right,
            h: b.height,
            color: bc,
        });
    }
}

/// Resolve the final display string for a text element: merged-children
/// content, uppercase transform, and truncation with ellipsis. Shared by
/// scene emission and layout measurement so both see the exact same text.
pub(crate) fn resolve_display_text<'a>(
    element: &'a Element,
    content: &'a str,
    truncate_chars: Option<usize>,
) -> std::borrow::Cow<'a, str> {
    // Resolve content (check merged children)
    let resolved = if content.is_empty() && !element.children.is_empty() {
        element
            .children
            .iter()
            .find_map(|c| {
                if let ElementKind::Text {
                    content: ref cc, ..
                } = c.kind
                    && !cc.is_empty()
                {
                    return Some(cc.as_str());
                }
                None
            })
            .unwrap_or("")
    } else {
        content
    };

    // Apply uppercase transform
    let after_case: std::borrow::Cow<'a, str> = if element.style.uppercase {
        resolved.to_uppercase().into()
    } else {
        resolved.into()
    };

    match truncate_chars {
        Some(max) if after_case.chars().count() > max => (after_case
            .chars()
            .take(max.saturating_sub(1))
            .collect::<String>()
            + "\u{2026}")
            .into(),
        _ => after_case,
    }
}

/// Resolve the effective [`TextStyle`] of a text element's run — the single
/// source of truth for typography (size, line height, weight, letter
/// spacing). Layout measurement (`text_measure_spec`) and scene emission
/// (`emit_text`) both build from this, so the renderer always draws exactly
/// what layout measured; diverging copies are what made `.tracking()` text
/// overflow its container.
pub(crate) fn resolved_text_style(element: &Element, font_size: f32, line_height: f32) -> TextStyle {
    TextStyle {
        font_size,
        line_height,
        font_weight: if element.style.bold {
            700
        } else {
            element.style.font_weight
        },
        letter_spacing: element.style.letter_spacing,
        // The builder Style has no font-family modifier yet; when it grows
        // one, threading it here propagates to measure AND draw at once.
        font_family: None,
    }
}

fn emit_text(
    element: &Element,
    b: &ComputedBounds,
    intent_color: Option<crate::color::Color>,
    props: &TextProps<'_>,
    out: &mut Vec<SceneNode>,
) {
    let actual = resolve_display_text(element, props.content, props.truncate_chars.copied());
    let actual: &str = &actual;

    if !actual.is_empty() {
        let style = resolved_text_style(element, props.font_size, props.line_height);
        out.push(SceneNode::Text {
            key: TextNodeKey::from_style(actual, &style, props.max_width),
            x: b.x,
            y: b.y,
            color: intent_color
                .map(|c| c.to_array())
                .unwrap_or_else(|| element.style.text_color.to_array()),
        });
    }
}
