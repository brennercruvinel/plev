use crate::layout::LayoutStyle;

// ---------------------------------------------------------------------------
// Visual -- rendering variant per node
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) enum Visual {
    None,
    Box {
        bg: [f32; 4],
        border_color: [f32; 4],
        border_width: f32,
        corner_radius: f32,
    },
    Text {
        content: String,
        size: f32,
        line_height: f32,
        weight: u16,
        color: [f32; 4],
        family: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// UiNode -- internal node stored per layout node
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub(crate) struct UiNode {
    pub layout: LayoutStyle,
    pub visual: Visual,
    pub children: Vec<usize>,
    pub click_id: Option<u64>,
}

// ---------------------------------------------------------------------------
// UiHitRect -- collected after layout
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct UiHitRect {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
