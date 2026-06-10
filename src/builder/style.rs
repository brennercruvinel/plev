use crate::color::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Direction {
    #[default]
    Column,
    Row,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum SizeConstraint {
    #[default]
    Auto,
    Fixed(f32),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Spacing {
    pub const fn all(v: f32) -> Self {
        Spacing {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct LayoutConfig {
    pub direction: Direction,
    pub align: Align,
    pub justify: Justify,
    pub padding: Spacing,
    pub margin: Spacing,
    pub gap: f32,
    pub width: SizeConstraint,
    pub height: SizeConstraint,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub grow: f32,
    pub shrink: f32,
    pub basis: Option<f32>,
    pub wrap: bool,
}

/// Per-side border configuration.
#[derive(Clone, Copy, Debug, Default)]
pub struct BorderConfig {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
    pub color: Color,
}

/// 2-stop linear gradient background (see `Element::bg_linear`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearGradient {
    pub from: Color,
    pub to: Color,
    /// CSS-style angle in degrees (0 = `from` at the bottom, clockwise).
    pub angle_deg: f32,
}

/// Analytic shadow spec, shared by drop (`Element::shadow_drop`) and
/// inset (`Element::shadow_inset`) shadows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DropShadow {
    pub blur: f32,
    pub offset: [f32; 2],
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct Style {
    pub bg: Option<Color>,
    pub bg_gradient: Option<LinearGradient>,
    pub drop_shadow: Option<DropShadow>,
    pub inset_shadow: Option<DropShadow>,
    /// Gaussian sigma for a region backdrop blur under this element
    /// (see `Element::backdrop_blur`).
    pub backdrop_blur: Option<f32>,
    pub clip_children: bool,
    pub text_color: Color,
    pub corner_radius: f32,
    pub shadow: f32,
    pub opacity: f32,
    pub border: f32,
    pub border_color: Color,
    pub border_sides: BorderConfig,
    pub bold: bool,
    pub italic: bool,
    pub font_weight: u16,
    pub letter_spacing: f32,
    pub uppercase: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            bg: None,
            bg_gradient: None,
            drop_shadow: None,
            inset_shadow: None,
            backdrop_blur: None,
            clip_children: false,
            text_color: Color::WHITE,
            corner_radius: 0.0,
            shadow: 0.0,
            opacity: 1.0,
            border: 0.0,
            border_color: Color::rgba(1.0, 1.0, 1.0, 0.2),
            border_sides: BorderConfig::default(),
            bold: false,
            italic: false,
            font_weight: 400,
            letter_spacing: 0.0,
            uppercase: false,
        }
    }
}
