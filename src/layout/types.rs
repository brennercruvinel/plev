// ---------------------------------------------------------------------------
// plev layout types -- wrapping Taffy as an implementation detail
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone)]
pub struct LayoutStyle {
    pub direction: Direction,
    pub align: Align,
    pub justify: Justify,
    pub padding: [f32; 4], // [top, right, bottom, left]
    pub margin: [f32; 4],  // [top, right, bottom, left]
    pub gap: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Option<f32>,
    pub flex_wrap: bool,
}

impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            direction: Direction::Column,
            align: Align::Stretch,
            justify: Justify::Start,
            padding: [0.0; 4],
            margin: [0.0; 4],
            gap: 0.0,
            width: None,
            height: None,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            flex_wrap: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ComputedBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for ComputedBounds {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

// ---------------------------------------------------------------------------
// LayoutItem -- input for layout computation (style + child indices)
// ---------------------------------------------------------------------------

/// Text content + style for real measurement of text leaf nodes. When set on
/// a `LayoutItem`, the engine measures the node with the `TextMeasurer`
/// (taffy measure function) instead of treating it as an empty leaf.
#[derive(Debug, Clone)]
pub struct TextMeasureSpec {
    pub content: String,
    pub style: crate::text::TextStyle,
    /// Element-level wrap width, combined with the available space.
    pub max_width: Option<f32>,
}

pub struct LayoutItem {
    pub style: LayoutStyle,
    pub children: Vec<usize>,
    /// Present for text leaf nodes that need intrinsic-size measurement.
    pub text: Option<TextMeasureSpec>,
}
