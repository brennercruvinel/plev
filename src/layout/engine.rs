use taffy::prelude::*;

use super::types::{Align, ComputedBounds, Direction, Justify, LayoutItem, LayoutStyle};

// ---------------------------------------------------------------------------
// LayoutEngine -- owns a TaffyTree, computes layout from a flat list of items
// ---------------------------------------------------------------------------

pub struct LayoutEngine {
    tree: TaffyTree<()>,
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

        // Phase 1: create all nodes with styles (no children yet)
        let node_ids: Vec<NodeId> = items
            .iter()
            .map(|item| {
                self.tree
                    .new_leaf(to_taffy_style(&item.style))
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

        // Phase 3: compute layout
        let root = node_ids[0];
        self.tree
            .compute_layout(
                root,
                taffy::Size {
                    width: AvailableSpace::Definite(viewport_w),
                    height: AvailableSpace::Definite(viewport_h),
                },
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
        width: dim_from_option(style.width),
        height: dim_from_option(style.height),
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
