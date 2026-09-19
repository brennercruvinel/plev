//! Design tokens for the scene3d HUD, read from the engine theme (the
//! HOFF default) instead of a private grayscale ladder. The HUD is
//! denser than an app screen (a ramp of small readouts over a 3D scene),
//! so it keeps its own font ladder below the theme's `small` step; every
//! color and spacing comes from `Theme`.

use std::sync::LazyLock;

use engine::color::Color;
use engine::theme::Theme;

pub struct HudTheme {
    pub surface_1: Color,
    pub surface_3: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_accent: Color,
    pub chip_essencial: Color,
    pub chip_recomendado: Color,
    pub chip_opcional: Color,
    pub chip_presente: Color,
    pub chip_config: Color,
    pub chip_basico: Color,
    pub white_70: Color,
    pub white_30: Color,
    pub space_xs: f32,
    pub space_sm: f32,
}

impl HudTheme {
    fn from_theme(t: &Theme) -> Self {
        let text = t.colors.text;
        let tint = |a: f32| Color([text.0[0], text.0[1], text.0[2], a]);
        Self {
            surface_1: t.colors.surface,
            surface_3: t.colors.divider,
            text_primary: t.colors.text,
            text_muted: t.colors.text_dim,
            text_accent: t.colors.accent,
            // Chip variants: the theme's intent colors, the only chromatics.
            chip_essencial: t.colors.danger,
            chip_recomendado: t.colors.warning,
            chip_opcional: t.colors.info,
            chip_presente: t.colors.success,
            chip_config: t.colors.accent,
            chip_basico: t.colors.text_mid,
            white_70: tint(0.7),
            white_30: tint(0.3),
            space_xs: t.spacing.xs,
            space_sm: t.spacing.sm,
        }
    }
}

static HUD: LazyLock<HudTheme> = LazyLock::new(|| HudTheme::from_theme(&Theme::default()));

/// The HUD tokens, resolved once from the default theme.
pub fn hud() -> &'static HudTheme {
    &HUD
}

// ---------------------------------------------------------------------------
// HUD font ladder (px): readouts over a 3D scene run smaller than the
// app ramp; the top steps meet the theme's caption / body / title.
// ---------------------------------------------------------------------------

pub const FONT_2XS: f32 = 7.0;
pub const FONT_XS: f32 = 8.0;
pub const FONT_SM: f32 = 9.0;
pub const FONT_BASE: f32 = 11.0;
pub const FONT_2XL: f32 = 22.0;

/// Chip background: the chip color at the wash alpha.
pub fn chip_bg(color: Color) -> Color {
    Color::rgba(color.0[0], color.0[1], color.0[2], 0.2)
}
