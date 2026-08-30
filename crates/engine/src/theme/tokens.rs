// ============================================================================
// Scales
// ============================================================================

use crate::color::Color;
use crate::text::TextStyle;

#[derive(Clone, Debug, PartialEq)]
pub struct SpacingScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
    pub xxl: f32,
}

/// Letter spacing of the body family (`=body`, `=body-2*`, `=hairline`),
/// converted to px per style. The HOFF sass reference tracks `0.025em` —
/// calibrated for Rubik, a narrow face. Inclusive Sans is ~10% wider with
/// generous built-in sidebearings, so the same tracking reads as gaps;
/// the body family tracks `0.0` (font's natural spacing) since the
/// Inclusive Sans swap (2026-08).
pub const BODY_LETTER_SPACING_EM: f32 = 0.0;

#[derive(Clone, Debug, PartialEq)]
pub struct TypographyScale {
    /// `=small`: 10px in HOFF.
    pub small: f32,
    pub caption: f32,
    pub body_sm: f32,
    pub body: f32,
    pub title_sm: f32,
    pub title: f32,
    pub display: f32,
    pub line_height_ratio: f32,
}

impl TypographyScale {
    /// Generic fallback ratio. Prefer the named styles below: each HOFF
    /// mixin has its own exact line-height.
    pub fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }

    fn style(size: f32, lh_factor: f32, weight: u16) -> TextStyle {
        TextStyle::new(size)
            .with_line_height(size * lh_factor)
            .with_weight(weight)
    }

    fn body_style(size: f32, lh_factor: f32, weight: u16) -> TextStyle {
        Self::style(size, lh_factor, weight).with_letter_spacing(BODY_LETTER_SPACING_EM * size)
    }

    // -- The HOFF type ramp (styles/variables.sass:39-127), one method per
    // -- mixin. Line heights and letter spacing are exact, not the generic
    // -- ratio.

    /// `=h4`: display size, line-height 1, weight 500.
    pub fn h4(&self) -> TextStyle {
        Self::style(self.display, 1.0, 500)
    }

    /// `=title`: 20px in HOFF, line-height 1.2, weight 500.
    pub fn title(&self) -> TextStyle {
        Self::style(self.title_sm, 1.2, 500)
    }

    /// `=base-r`: 16px, line-height 1.5.
    pub fn base_r(&self) -> TextStyle {
        Self::style(self.body, 1.5, 400)
    }

    /// `=base-m`: 16px, line-height 1.5, weight 500.
    pub fn base_m(&self) -> TextStyle {
        Self::style(self.body, 1.5, 500)
    }

    /// `=base-2r`: 14px, line-height 1.4.
    pub fn base_2r(&self) -> TextStyle {
        Self::style(self.body_sm, 1.4, 400)
    }

    /// `=base-2m`: 14px, line-height 1.4, weight 500.
    pub fn base_2m(&self) -> TextStyle {
        Self::style(self.body_sm, 1.4, 500)
    }

    /// `=base-2sm`: 14px, line-height 1.4, weight 600.
    pub fn base_2sm(&self) -> TextStyle {
        Self::style(self.body_sm, 1.4, 600)
    }

    /// `=base-2b`: 14px, line-height 1.4, weight 700.
    pub fn base_2b(&self) -> TextStyle {
        Self::style(self.body_sm, 1.4, 700)
    }

    /// `=caption-r`: 12px, line-height 1.33.
    pub fn caption_r(&self) -> TextStyle {
        Self::style(self.caption, 1.33, 400)
    }

    /// `=caption-sm`: 12px, line-height 1.33, weight 600.
    pub fn caption_sm(&self) -> TextStyle {
        Self::style(self.caption, 1.33, 600)
    }

    /// `=small`: 10px, line-height 1.2.
    pub fn small_r(&self) -> TextStyle {
        Self::style(self.small, 1.2, 400)
    }

    /// `=small-sm`: 10px, line-height 1.2, weight 600.
    pub fn small_sm(&self) -> TextStyle {
        Self::style(self.small, 1.2, 600)
    }

    /// `=body`: 16px, line-height 1.5.
    pub fn body(&self) -> TextStyle {
        Self::body_style(self.body, 1.5, 400)
    }

    /// `=body-2r`: 14px, line-height 1.7.
    pub fn body_2r(&self) -> TextStyle {
        Self::body_style(self.body_sm, 1.7, 400)
    }

    /// `=body-2m`: 14px, line-height 1.7, weight 500.
    pub fn body_2m(&self) -> TextStyle {
        Self::body_style(self.body_sm, 1.7, 500)
    }

    /// `=body-2b`: like `=body-2m` (the reference sets weight 500 on both).
    pub fn body_2b(&self) -> TextStyle {
        Self::body_style(self.body_sm, 1.7, 500)
    }

    /// `=hairline`: 12px, line-height 1.65, weight 500.
    pub fn hairline(&self) -> TextStyle {
        Self::body_style(self.caption, 1.65, 500)
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
