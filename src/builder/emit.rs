use crate::compositor::{SceneNode, TextNodeKey};
use crate::layout::ComputedBounds;

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

pub(crate) fn emit_scene_nodes(
    elements: &[&Element],
    bounds: &[ComputedBounds],
    theme: Option<&crate::theme::Theme>,
    out: &mut Vec<SceneNode>,
) {
    for (i, &element) in elements.iter().enumerate() {
        let b = &bounds[i];

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
        }
    }
}

fn emit_div(
    element: &Element,
    b: &ComputedBounds,
    intent_color: Option<crate::color::Color>,
    out: &mut Vec<SceneNode>,
) {
    let has_bg = element.style.bg.is_some() || intent_color.is_some();
    let has_border = element.style.border > 0.0;
    let has_radius = element.style.corner_radius > 0.0;

    if (has_bg || has_border) && (b.width > 0.0 || b.height > 0.0) {
        let bg_color = intent_color
            .or(element.style.bg)
            .unwrap_or(crate::color::Color::TRANSPARENT)
            .to_array();

        if has_radius || has_border {
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
        let weight = if element.style.bold {
            700
        } else {
            element.style.font_weight
        };
        out.push(SceneNode::Text {
            key: TextNodeKey::new(actual, props.font_size, props.line_height, props.max_width)
                .with_weight(weight),
            x: b.x,
            y: b.y,
            color: intent_color
                .map(|c| c.to_array())
                .unwrap_or_else(|| element.style.text_color.to_array()),
        });
    }
}
