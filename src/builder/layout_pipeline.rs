use crate::layout::{
    Align as TaffyAlign, Direction as TaffyDirection, Justify as TaffyJustify, LayoutItem,
    LayoutStyle as TaffyLayoutStyle,
};

use super::element::{Element, ElementKind};
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
    };

    index
}

pub(crate) fn to_layout_style(element: &Element) -> TaffyLayoutStyle {
    let lc = &element.layout;

    // Intrinsic size for leaf elements (text, path)
    let width = match lc.width {
        SizeConstraint::Fixed(v) => Some(v),
        SizeConstraint::Auto => match &element.kind {
            ElementKind::Text {
                content,
                font_size,
                truncate_chars,
                ..
            } => {
                let chars = truncate_chars
                    .map(|m| m.min(content.chars().count()))
                    .unwrap_or_else(|| content.chars().count());
                if chars > 0 {
                    Some(chars as f32 * font_size * 0.6)
                } else {
                    None
                }
            }
            ElementKind::Path { data } => {
                let max_x = data
                    .vertices
                    .iter()
                    .map(|v| v.position[0])
                    .fold(0.0_f32, f32::max);
                if max_x > 0.0 { Some(max_x) } else { None }
            }
            ElementKind::Div => None,
        },
    };

    let height = match lc.height {
        SizeConstraint::Fixed(v) => Some(v),
        SizeConstraint::Auto => match &element.kind {
            ElementKind::Text { line_height, .. } => Some(*line_height),
            ElementKind::Path { data } => {
                let max_y = data
                    .vertices
                    .iter()
                    .map(|v| v.position[1])
                    .fold(0.0_f32, f32::max);
                if max_y > 0.0 { Some(max_y) } else { None }
            }
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
