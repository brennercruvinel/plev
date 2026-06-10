// ============================================================================
// Scales
// ============================================================================

use crate::color::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct SpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypographyScale {
    pub caption: f32,
    pub body_sm: f32,
    pub body: f32,
    pub title_sm: f32,
    pub title: f32,
    pub display: f32,
    pub line_height_ratio: f32,
}

impl TypographyScale {
    pub fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadiusScale {
    pub none: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub full: f32,
}

// ============================================================================
// ColorTokens -- semantic color palette
// ============================================================================

#[derive(Clone, Debug)]
pub struct ColorTokens {
    pub bg: Color,
    pub surface: Color,
    pub bg_panel: Color,
    pub bg_hover: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_mid: Color,
    pub accent: Color,
    pub accent_dim: Color,
    pub success: Color,
    pub danger: Color,
    pub warning: Color,
    pub info: Color,
    pub divider: Color,
    pub border_active: Color,
}

// ============================================================================
// EffectTokens
// ============================================================================

#[derive(Clone, Debug)]
pub struct EffectTokens {
    pub shadow_sigma: f32,
    pub shadow_color: Color,
    pub blur_sigma: f32,
}
