// ============================================================================
// Scales
// ============================================================================

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;

use super::roles::TypeRoles;
use super::scales::ShadowSpec;

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

/// The embedded code face (JetBrains Mono Regular, the Latin range), the
/// mono family every built-in theme names.
pub const MONO_FAMILY: &str = "JetBrains Mono";

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
    /// Face every style resolves to; `None` is the engine default
    /// (Inclusive Sans). The hoff core theme points this at Inter.
    pub family: Option<&'static str>,
    /// Face of the code styles ([`Self::mono`], [`Self::exact_mono`]):
    /// [`MONO_FAMILY`] in every built-in theme.
    pub mono_family: &'static str,
    /// Tracking applied to every style, in em (css `tracking-[-0.02em]`
    /// on the core body). Zero for the HOFF ramp.
    pub tracking_em: f32,
    /// The scale by role ([`super::roles`]): what every recipe resolves
    /// its text against.
    pub roles: TypeRoles,
}

impl TypographyScale {
    /// Generic fallback ratio. Prefer the named styles below: each HOFF
    /// mixin has its own exact line-height.
    pub fn line_height(&self, font_size: f32) -> f32 {
        font_size * self.line_height_ratio
    }

    fn style(&self, size: f32, lh_factor: f32, weight: u16) -> TextStyle {
        self.exact(size, size * lh_factor, weight)
    }

    /// A style with an explicit line height in px, in this scale's
    /// family and tracking. The core ramp is written this way (14/20,
    /// 12/16...).
    pub fn exact(&self, size: f32, line_height: f32, weight: u16) -> TextStyle {
        let style = TextStyle::new(size)
            .with_line_height(line_height)
            .with_weight(weight)
            .with_letter_spacing(self.tracking_em * size);
        match self.family {
            Some(family) => style.with_family(family),
            None => style,
        }
    }

    /// A code style with an explicit line height in px: the mono family
    /// at its own spacing, the scale's tracking left out (the tracking is
    /// the UI face's, and a mono face keeps its grid).
    pub fn exact_mono(&self, size: f32, line_height: f32, weight: u16) -> TextStyle {
        TextStyle::new(size)
            .with_line_height(line_height)
            .with_weight(weight)
            .with_family(self.mono_family)
    }

    fn body_style(&self, size: f32, lh_factor: f32, weight: u16) -> TextStyle {
        let style = self.style(size, lh_factor, weight);
        let tracking = (self.tracking_em + BODY_LETTER_SPACING_EM) * size;
        style.with_letter_spacing(tracking)
    }

    // -- The HOFF type ramp (styles/variables.sass:39-127), one method per
    // -- mixin. Line heights and letter spacing are exact, not the generic
    // -- ratio.

    /// `=h4`: display size, line-height 1, weight 500.
    pub fn h4(&self) -> TextStyle {
        self.style(self.display, 1.0, 500)
    }

    /// `=title`: 20px in HOFF, line-height 1.2, weight 500.
    pub fn title(&self) -> TextStyle {
        self.style(self.title_sm, 1.2, 500)
    }

    /// `=base-r`: 16px, line-height 1.5.
    pub fn base_r(&self) -> TextStyle {
        self.style(self.body, 1.5, 400)
    }

    /// `=base-m`: 16px, line-height 1.5, weight 500.
    pub fn base_m(&self) -> TextStyle {
        self.style(self.body, 1.5, 500)
    }

    /// `=base-2r`: 14px, line-height 1.4.
    pub fn base_2r(&self) -> TextStyle {
        self.style(self.body_sm, 1.4, 400)
    }

    /// `=base-2m`: 14px, line-height 1.4, weight 500.
    pub fn base_2m(&self) -> TextStyle {
        self.style(self.body_sm, 1.4, 500)
    }

    /// `=base-2sm`: 14px, line-height 1.4, weight 600.
    pub fn base_2sm(&self) -> TextStyle {
        self.style(self.body_sm, 1.4, 600)
    }

    /// `=base-2b`: 14px, line-height 1.4, weight 700.
    pub fn base_2b(&self) -> TextStyle {
        self.style(self.body_sm, 1.4, 700)
    }

    /// `=caption-r`: 12px, line-height 1.33.
    pub fn caption_r(&self) -> TextStyle {
        self.style(self.caption, 1.33, 400)
    }

    /// `=caption-sm`: 12px, line-height 1.33, weight 600.
    pub fn caption_sm(&self) -> TextStyle {
        self.style(self.caption, 1.33, 600)
    }

    /// `=small`: 10px, line-height 1.2.
    pub fn small_r(&self) -> TextStyle {
        self.style(self.small, 1.2, 400)
    }

    /// `=small-sm`: 10px, line-height 1.2, weight 600.
    pub fn small_sm(&self) -> TextStyle {
        self.style(self.small, 1.2, 600)
    }

    /// `=body`: 16px, line-height 1.5.
    pub fn body(&self) -> TextStyle {
        self.body_style(self.body, 1.5, 400)
    }

    /// `=body-2r`: 14px, line-height 1.7.
    pub fn body_2r(&self) -> TextStyle {
        self.body_style(self.body_sm, 1.7, 400)
    }

    /// `=body-2m`: 14px, line-height 1.7, weight 500.
    pub fn body_2m(&self) -> TextStyle {
        self.body_style(self.body_sm, 1.7, 500)
    }

    /// `=body-2b`: like `=body-2m` (the reference sets weight 500 on both).
    pub fn body_2b(&self) -> TextStyle {
        self.body_style(self.body_sm, 1.7, 500)
    }

    /// `=hairline`: 12px, line-height 1.65, weight 500.
    pub fn hairline(&self) -> TextStyle {
        self.body_style(self.caption, 1.65, 500)
    }

    /// `=headline`: 32px, line-height 1.25, weight 500. The stat card
    /// value and page headlines.
    pub fn headline(&self) -> TextStyle {
        self.style(self.title, 1.25, 500)
    }

    /// Code: body-sm at line-height 1.4 in the mono family. One
    /// definition instead of the `code_style` helper each gallery section
    /// grew.
    pub fn mono(&self) -> TextStyle {
        self.exact_mono(self.body_sm, self.body_sm * 1.4, 400)
    }

    /// Small readout under a specimen or a chart axis: `small` at 1.5.
    pub fn readout(&self) -> TextStyle {
        self.style(self.small, 1.5, 400)
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

/// A group of a design system's named colors, as its colors page lists
/// them (`Shades`, `Colors`), each with the name the system gives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwatchGroup {
    pub title: &'static str,
    pub colors: &'static [(&'static str, Color)],
}

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
    /// The design system's own palette, named, beyond the roles above (a
    /// shade or an accent no role holds); empty for a theme that is only
    /// its roles.
    pub swatches: &'static [SwatchGroup],
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
    /// Tabs strip container (HOFF: rgba(40,40,40,.60)).
    pub tabs: Color,
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
    /// Active / hovered label and icon (HOFF: rgba(248,248,248,.76)).
    pub text_active: Color,
    /// Menu item / option label at rest, inactive titles
    /// (HOFF: rgba(248,248,248,.56)).
    pub text_default: Color,
    /// Alpha multiplier applied to a disabled control's fill, rim and
    /// label.
    pub disabled_alpha: f32,
    /// Alpha of an intent-tinted wash behind a selected chip or row at
    /// rest, and hovered. The hue comes from the intent; only the alphas
    /// are tokens.
    pub wash_alpha: f32,
    pub wash_hover_alpha: f32,
    /// How much of the rim's top color [`glass_pill`] keeps at the
    /// bottom: 0 fades the rim out downward (the HOFF top-lit edge), 1
    /// keeps it whole (a plain border, the core kit's `border`).
    ///
    /// [`glass_pill`]: crate::widgets::recipe::glass_pill
    pub rim_fade: f32,
    /// The inset key-light [`inset_keylight`] lays over a glass surface,
    /// a soft highlight bleeding in from the top-left (HOFF: `inset 2px
    /// 4px 16px rgba(248,248,248,.06)`); `None` draws none (the core kit
    /// lights no surface).
    ///
    /// [`inset_keylight`]: crate::widgets::recipe::inset_keylight
    pub keylight: Option<ShadowSpec>,
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
            tabs: Color([
                colors.bg_panel.0[0],
                colors.bg_panel.0[1],
                colors.bg_panel.0[2],
                colors.bg_panel.0[3] * 0.85,
            ]),
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
            text_active: tint(0.76),
            text_default: tint(0.56),
            disabled_alpha: 0.5,
            wash_alpha: 0.14,
            wash_hover_alpha: 0.22,
            rim_fade: 0.0,
            // The dark ladder these themes are built on: the key-light
            // offset `xs / 2, xs` and blurred by `lg` (4 and 24).
            keylight: Some(ShadowSpec::inset([2.0, 4.0], 24.0, 0.0, tint(0.06))),
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
