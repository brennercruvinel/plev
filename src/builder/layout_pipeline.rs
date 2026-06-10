use crate::layout::{
    Align as TaffyAlign, Direction as TaffyDirection, Justify as TaffyJustify, LayoutItem,
    LayoutStyle as TaffyLayoutStyle, TextMeasureSpec,
};
use crate::text::TextStyle;

use super::element::{Element, ElementKind};
use super::emit::resolve_display_text;
use super::style::*;

// ---------------------------------------------------------------------------
// Element tree -> flat LayoutItem list for Taffy
// ---------------------------------------------------------------------------

pub(crate) fn collect_layout_items<'a>(
    element: &'a Element,
    items: &mut Vec<LayoutItem>,
    elements: &mut Vec<&'a Element>,
) -> usize {
    let index = items.len();
    // Reserve slot with placeholder
    items.push(LayoutItem {
        style: TaffyLayoutStyle::default(),
        children: vec![],
        text: None,
    });
    elements.push(element);

    // Recurse children
    let child_indices: Vec<usize> = element
        .children
        .iter()
        .map(|child| collect_layout_items(child, items, elements))
        .collect();

    // Fill in real style and children
    items[index] = LayoutItem {
        style: to_layout_style(element),
        children: child_indices,
        text: text_measure_spec(element),
    };

    index
}

/// Build the measure spec for text leaf nodes, mirroring exactly the string
/// transformations `emit_text` applies (children merge, uppercase, truncate),
/// so layout measures the same text the renderer shapes.
fn text_measure_spec(element: &Element) -> Option<TextMeasureSpec> {
    let ElementKind::Text {
        content,
        font_size,
        line_height,
        max_width,
        truncate_chars,
    } = &element.kind
    else {
        return None;
    };

    let resolved = resolve_display_text(element, content, *truncate_chars);
    if resolved.is_empty() {
        return None;
    }

    let font_weight = if element.style.bold {
        700
    } else {
        element.style.font_weight
    };

    Some(TextMeasureSpec {
        content: resolved.into_owned(),
        style: TextStyle {
            font_size: *font_size,
            line_height: *line_height,
            font_weight,
            font_family: None,
        },
        max_width: *max_width,
    })
}

pub(crate) fn to_layout_style(element: &Element) -> TaffyLayoutStyle {
    let lc = &element.layout;

    // Intrinsic size for leaf elements. Text nodes have no fixed size here:
    // they are measured for real by the layout engine (TextMeasurer).
    // Natural size of an image element: the decoded dimensions (the load
    // is memoized by content, so layout and emit share one decode).
    let image_size = match &element.kind {
        ElementKind::Image { bytes: Some(bytes) } => {
            crate::gpu::image::load_image_bytes(bytes).ok()
        }
        _ => None,
    };

    let width = match lc.width {
        SizeConstraint::Fixed(v) => Some(v),
        SizeConstraint::Auto => match &element.kind {
            ElementKind::Text { .. } => None,
            ElementKind::Path { data } => {
                let max_x = data
                    .vertices
                    .iter()
                    .map(|v| v.position[0])
                    .fold(0.0_f32, f32::max);
                if max_x > 0.0 { Some(max_x) } else { None }
            }
            ElementKind::Image { .. } => image_size.map(|h| h.width as f32),
            ElementKind::Div => None,
        },
    };

    let height = match lc.height {
        SizeConstraint::Fixed(v) => Some(v),
        SizeConstraint::Auto => match &element.kind {
            ElementKind::Text { .. } => None,
            ElementKind::Path { data } => {
                let max_y = data
                    .vertices
                    .iter()
                    .map(|v| v.position[1])
                    .fold(0.0_f32, f32::max);
                if max_y > 0.0 { Some(max_y) } else { None }
            }
            ElementKind::Image { .. } => image_size.map(|h| h.height as f32),
            ElementKind::Div => None,
        },
    };

    TaffyLayoutStyle {
        direction: match lc.direction {
            Direction::Row => TaffyDirection::Row,
            Direction::Column => TaffyDirection::Column,
        },
        align: match lc.align {
            Align::Start => TaffyAlign::Start,
            Align::Center => TaffyAlign::Center,
            Align::End => TaffyAlign::End,
            Align::Stretch => TaffyAlign::Stretch,
        },
        justify: match lc.justify {
            Justify::Start => TaffyJustify::Start,
            Justify::Center => TaffyJustify::Center,
            Justify::End => TaffyJustify::End,
            Justify::SpaceBetween => TaffyJustify::SpaceBetween,
        },
        padding: [
            lc.padding.top,
            lc.padding.right,
            lc.padding.bottom,
            lc.padding.left,
        ],
        margin: [
            lc.margin.top,
            lc.margin.right,
            lc.margin.bottom,
            lc.margin.left,
        ],
        gap: lc.gap,
        width,
        height,
        min_width: lc.min_width,
        min_height: lc.min_height,
        max_width: lc.max_width,
        max_height: lc.max_height,
        flex_grow: lc.grow,
        flex_shrink: lc.shrink,
        flex_basis: lc.basis,
        flex_wrap: lc.wrap,
    }
}
