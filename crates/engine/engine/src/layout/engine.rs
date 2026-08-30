use taffy::prelude::*;

use crate::text::TextMeasurer;

use super::types::{
    Align, ComputedBounds, Direction, Justify, LayoutItem, LayoutStyle, TextMeasureSpec,
};

// ---------------------------------------------------------------------------
// LayoutEngine -- owns a TaffyTree, computes layout from a flat list of items
// ---------------------------------------------------------------------------

pub struct LayoutEngine {
    tree: TaffyTree<TextMeasureSpec>,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            tree: TaffyTree::new(),
        }
    }

    /// Compute layout from a flat list of LayoutItems.
    /// Returns ComputedBounds for each item (absolute screen coordinates).
    /// Item 0 is the root.
    pub fn compute(
        &mut self,
        items: &[LayoutItem],
        viewport_w: f32,
        viewport_h: f32,
    ) -> Vec<ComputedBounds> {
        self.tree.clear();

        if items.is_empty() {
            return Vec::new();
        }

        // Phase 1: create all nodes with styles (no children yet). Text leaf
        // nodes carry their measure spec as taffy node context.
        let node_ids: Vec<NodeId> = items
            .iter()
            .map(|item| {
                let style = to_taffy_style(&item.style);
                match &item.text {
                    Some(spec) => self.tree.new_leaf_with_context(style, spec.clone()),
                    None => self.tree.new_leaf(style),
                }
                .expect("failed to create taffy node")
            })
            .collect();

        // Phase 2: set children for each node
        for (i, item) in items.iter().enumerate() {
            if !item.children.is_empty() {
                let child_ids: Vec<NodeId> =
                    item.children.iter().map(|&idx| node_ids[idx]).collect();
                self.tree
                    .set_children(node_ids[i], &child_ids)
                    .expect("failed to set children");
            }
        }

        // Phase 3: compute layout, measuring text leaves with the real shaper
        let root = node_ids[0];
        self.tree
            .compute_layout_with_measure(
                root,
                taffy::Size {
                    width: AvailableSpace::Definite(viewport_w),
                    height: AvailableSpace::Definite(viewport_h),
                },
                measure_text_node,
            )
            .expect("layout computation failed");

        // Phase 4: collect absolute positions via DFS
        let mut bounds = vec![ComputedBounds::default(); items.len()];
        self.collect_bounds(&node_ids, items, 0, 0.0, 0.0, &mut bounds);

        bounds
    }

    fn collect_bounds(
        &self,
        node_ids: &[NodeId],
        items: &[LayoutItem],
        index: usize,
        parent_x: f32,
        parent_y: f32,
        bounds: &mut [ComputedBounds],
    ) {
        let layout = self
            .tree
            .layout(node_ids[index])
            .expect("failed to read layout");

        let abs_x = parent_x + layout.location.x;
        let abs_y = parent_y + layout.location.y;

        bounds[index] = ComputedBounds {
            x: abs_x,
            y: abs_y,
            width: layout.size.width,
            height: layout.size.height,
        };

        for &child_idx in &items[index].children {
            self.collect_bounds(node_ids, items, child_idx, abs_x, abs_y, bounds);
        }
    }
}

// ---------------------------------------------------------------------------
// Text measure function (taffy leaf nodes with a TextMeasureSpec context)
// ---------------------------------------------------------------------------

fn measure_text_node(
    known: taffy::Size<Option<f32>>,
    available: taffy::Size<AvailableSpace>,
    _node: NodeId,
    context: Option<&mut TextMeasureSpec>,
    _style: &Style,
) -> taffy::Size<f32> {
    let Some(spec) = context else {
        return taffy::Size::ZERO;
    };
    if spec.content.is_empty() {
        return taffy::Size::ZERO;
    }

    let available_width = known.width.or(match available.width {
        AvailableSpace::Definite(w) => Some(w),
        // Min-content: wrap as tightly as possible (widest unbreakable word).
        AvailableSpace::MinContent => Some(0.0),
        AvailableSpace::MaxContent => None,
    });
    let wrap_width = match (available_width, spec.max_width) {
        (Some(a), Some(b)) => Some(a.min(b)),
        (a, b) => a.or(b),
    };

    let (width, height) = TextMeasurer::measure_styled(&spec.content, &spec.style, wrap_width);
    taffy::Size {
        width: known.width.unwrap_or(width),
        height: known.height.unwrap_or(height),
    }
}

// ---------------------------------------------------------------------------
// Conversion: LayoutStyle -> taffy::Style
// ---------------------------------------------------------------------------

fn to_taffy_style(style: &LayoutStyle) -> Style {
    let flex_direction = match style.direction {
        Direction::Row => FlexDirection::Row,
        Direction::Column => FlexDirection::Column,
    };

    let align_items = match style.align {
        Align::Start => Some(AlignItems::FlexStart),
        Align::Center => Some(AlignItems::Center),
        Align::End => Some(AlignItems::FlexEnd),
        Align::Stretch => Some(AlignItems::Stretch),
    };

    let justify_content = match style.justify {
        Justify::Start => Some(JustifyContent::FlexStart),
        Justify::Center => Some(JustifyContent::Center),
        Justify::End => Some(JustifyContent::FlexEnd),
        Justify::SpaceBetween => Some(JustifyContent::SpaceBetween),
        Justify::SpaceAround => Some(JustifyContent::SpaceAround),
        Justify::SpaceEvenly => Some(JustifyContent::SpaceEvenly),
    };

    let [pt, pr, pb, pl] = style.padding;
    let padding = taffy::Rect {
        top: LengthPercentage::length(pt),
        right: LengthPercentage::length(pr),
        bottom: LengthPercentage::length(pb),
        left: LengthPercentage::length(pl),
    };

    let [mt, mr, mb, ml] = style.margin;
    let margin = taffy::Rect {
        top: LengthPercentageAuto::length(mt),
        right: LengthPercentageAuto::length(mr),
        bottom: LengthPercentageAuto::length(mb),
        left: LengthPercentageAuto::length(ml),
    };

    let gap = taffy::Size {
        width: LengthPercentage::length(style.gap),
        height: LengthPercentage::length(style.gap),
    };

    let size = taffy::Size {
        width: dim_from(style.width, style.width_percent),
        height: dim_from(style.height, style.height_percent),
    };

    let min_size = taffy::Size {
        width: dim_from_option(style.min_width),
        height: dim_from_option(style.min_height),
    };

    let max_size = taffy::Size {
        width: dim_from_option(style.max_width),
        height: dim_from_option(style.max_height),
    };

    let flex_basis = match style.flex_basis {
        Some(v) => Dimension::length(v),
        None => Dimension::auto(),
    };

    let flex_wrap = if style.flex_wrap {
        FlexWrap::Wrap
    } else {
        FlexWrap::NoWrap
    };

    Style {
        display: Display::Flex,
        flex_direction,
        align_items,
        justify_content,
        padding,
        margin,
        gap,
        size,
        min_size,
        max_size,
        flex_grow: style.flex_grow,
        flex_shrink: style.flex_shrink,
        flex_basis,
        flex_wrap,
        ..Default::default()
    }
}

fn dim_from_option(val: Option<f32>) -> Dimension {
    match val {
        Some(v) => Dimension::length(v),
        None => Dimension::auto(),
    }
}

/// Resolve a dimension from parallel pixel/percent fields. Percent (a
/// fraction, `0.5` = 50% of the parent) wins over a fixed pixel value;
/// neither set means auto.
fn dim_from(px: Option<f32>, percent: Option<f32>) -> Dimension {
    match (percent, px) {
        (Some(p), _) => Dimension::percent(p),
        (None, Some(v)) => Dimension::length(v),
        (None, None) => Dimension::auto(),
    }
}
