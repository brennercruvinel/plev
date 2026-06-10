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
// GlassTokens -- translucent "dark glass" surface recipe (HOFF)
// ============================================================================

/// Layered-glass surface tokens. The HOFF design language builds every
/// surface from a handful of white/graphite alphas; other themes derive
/// equivalents from their palette so widgets stay theme-agnostic.
#[derive(Clone, Debug, PartialEq)]
pub struct GlassTokens {
    /// Card/list-row surface at rest (HOFF: rgba(248,248,248,.02)).
    pub surface: Color,
    /// Hovered surface (HOFF: rgba(248,248,248,.05)).
    pub surface_hover: Color,
    /// Active/selected surface (HOFF: rgba(248,248,248,.10)).
    pub surface_active: Color,
    /// Default pill-button fill (HOFF: rgba(40,40,40,.70)).
    pub button: Color,
    /// Hovered pill-button fill (HOFF: rgba(248,248,248,.10)).
    pub button_hover: Color,
    /// Input/field background (HOFF: rgba(248,248,248,.05)).
    pub field: Color,
    /// Focused field border (HOFF: rgba(248,248,248,.25)).
    pub field_focus_border: Color,
    /// Solid popover/dropdown body (HOFF: #3b3b3b).
    pub popover: Color,
    /// Solid tooltip body (HOFF: #262626).
    pub tooltip: Color,
    /// Modal backdrop scrim (HOFF: rgba(35,34,34,.9)).
    pub scrim: Color,
    /// Edge-light border, strong (HOFF: rgba(255,255,255,.10)).
    pub edge: Color,
    /// Edge-light border, soft (HOFF: rgba(255,255,255,.05)).
    pub edge_soft: Color,
    /// Universal inset key-light (HOFF: rgba(248,248,248,.06)).
    pub inset_highlight: Color,
    /// Knob/handle vertical gradient, top then bottom
    /// (HOFF: rgba(248,248,248,.90) -> rgba(248,248,248,.30)).
    pub knob_gradient: [Color; 2],
    /// Headline gradient-text pair
    /// (HOFF: rgba(248,248,248,.9) -> rgba(248,248,248,.5)).
    pub text_gradient: [Color; 2],
    /// Inactive text / default icon fill (HOFF: rgba(248,248,248,.40)).
    pub text_faint: Color,
    /// Placeholders, list dates (HOFF: rgba(248,248,248,.25)).
    pub text_placeholder: Color,
}

impl GlassTokens {
    /// Derive a glass recipe from a plain palette so non-HOFF themes keep
    /// working with the HOFF-styled widgets.
    pub fn derive(colors: &ColorTokens) -> Self {
        let t = colors.text.0;
        let tint = |a: f32| Color([t[0], t[1], t[2], a]);
        Self {
            surface: tint(0.02),
            surface_hover: tint(0.05),
            surface_active: tint(0.10),
            button: colors.bg_panel,
            button_hover: tint(0.10),
            field: tint(0.05),
            field_focus_border: tint(0.25),
            popover: colors.bg_panel,
            tooltip: colors.bg_panel,
            scrim: Color([0.0, 0.0, 0.0, 0.45]),
            edge: tint(0.10),
            edge_soft: tint(0.05),
            inset_highlight: tint(0.06),
            knob_gradient: [tint(0.90), tint(0.30)],
            text_gradient: [tint(0.90), tint(0.50)],
            text_faint: tint(0.40),
            text_placeholder: tint(0.25),
        }
    }
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
