// ============================================================================
// HOFF -- the default plev design language
//
// Monochromatic "dark glass": every surface is white (#F8F8F8) or graphite
// (#282828) at one of a handful of canonical alphas, lit from the top by an
// edge-light border and an inset key-light. The only chromatic accents are
// a red, two greens and an orange. Values are taken 1:1 from the HOFF
// reference (styles/variables.sass + component modules).
// ============================================================================

use super::core::Theme;
use super::intent::MotionPhysics;
use super::tokens::{
    ColorTokens, EffectTokens, GlassTokens, RadiusScale, SpacingScale, TypographyScale,
};
use crate::color::Color;

const N2_F: f32 = 248.0 / 255.0;

/// Base white `$n1: #ffffff`.
pub const N1: Color = Color::hex(0xffffff);
/// Base warm white `$n2: #f8f8f8` — every white alpha derives from this.
pub const N2: Color = Color::hex(0xf8f8f8);
/// Base graphite `$n3: #282828`.
pub const N3: Color = Color::hex(0x282828);
/// Base near-black `$n4: #121212`.
pub const N4: Color = Color::hex(0x121212);

/// `$n2` at an arbitrary alpha — the workhorse of the HOFF palette.
pub const fn n2(alpha: f32) -> Color {
    Color::rgba(N2_F, N2_F, N2_F, alpha)
}

/// `$n3` (#282828) at an arbitrary alpha.
pub const fn n3(alpha: f32) -> Color {
    let g = 40.0 / 255.0;
    Color::rgba(g, g, g, alpha)
}

/// Notification badge / unfollow / active like: `#BD3027`.
pub const RED: Color = Color::hex(0xBD3027);
/// Online/new dot, topic status: `#55F08B`.
pub const GREEN: Color = Color::hex(0x55F08B);
/// Active repost: `rgba(124, 255, 176, .7)` (#7CFFB0).
pub const GREEN_LIGHT: Color = Color::rgba(124.0 / 255.0, 1.0, 176.0 / 255.0, 0.7);
/// Message reaction heart: `rgba(255, 77, 0, .9)` (#FF4D00).
pub const ORANGE: Color = Color::rgba(1.0, 77.0 / 255.0, 0.0, 0.9);

/// Global page frame behind the app (`body` in common.sass): `#444444`.
pub const PAGE_BG: Color = Color::hex(0x444444);
/// Central feed container `$bg-surface: rgba(40, 40, 40, .7)`.
pub const BG_SURFACE: Color = n3(0.7);
/// Sidebars: `rgba(40, 40, 40, .8)`.
pub const BG_SIDEBAR: Color = n3(0.8);
/// Modal overlay: `rgba(35, 34, 34, .9)`.
pub const SCRIM: Color = Color::rgba(35.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 0.9);
/// Actions dropdown body: solid `#3b3b3b`.
pub const POPOVER: Color = Color::hex(0x3b3b3b);
/// Tooltip body: solid `#262626`.
pub const TOOLTIP: Color = Color::hex(0x262626);

/// Card-shell overlay (hoff cards): `rgba(40, 40, 40, .8)`.
pub const CARD_OVERLAY: Color = n3(0.8);
/// Deep drop shadow under cards: `0 32px 24px -16px rgba(0, 0, 0, .40)`.
pub const CARD_SHADOW: Color = Color::rgba(0.0, 0.0, 0.0, 0.40);
/// Floating menu shadow: `0 24px 32px -12px rgba(18, 18, 18, .10)`.
pub const MENU_SHADOW: Color = Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10);

impl Theme {
    /// The HOFF dark-glass theme — plev's default.
    pub fn hoff() -> Self {
        Self {
            colors: ColorTokens {
                bg: PAGE_BG,
                surface: BG_SURFACE,
                bg_panel: POPOVER,
                bg_hover: n2(0.05),
                // $text-primary / $text-secondary / $text-tertiary.
                text: n2(0.95),
                text_mid: n2(0.70),
                text_dim: n2(0.50),
                accent: n2(0.95),
                accent_dim: n2(0.40),
                success: GREEN,
                danger: RED,
                warning: ORANGE,
                info: GREEN_LIGHT,
                divider: Color::rgba(1.0, 1.0, 1.0, 0.05),
                border_active: Color::rgba(1.0, 1.0, 1.0, 0.10),
            },
            // caption-r 12 / base-2r 14 / base-r 16 / title 20 /
            // headline 32 / h4 36.
            typography: TypographyScale {
                caption: 12.0,
                body_sm: 14.0,
                body: 16.0,
                title_sm: 20.0,
                title: 32.0,
                display: 36.0,
                line_height_ratio: 1.4,
            },
            spacing: SpacingScale {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
                xxl: 32.0,
            },
            // 8 tooltips/tags · 12 nav/field/switch · 20 cards ·
            // 32 pills/modals.
            radius: RadiusScale {
                none: 0.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
                full: 9999.0,
            },
            // Near-critically damped, settles in ~200ms — the HOFF global
            // `transition .2s` expressed as spring physics.
            motion: MotionPhysics {
                mass: 1.0,
                stiffness: 380.0,
                damping: 39.0,
            },
            effects: EffectTokens {
                // `0 6px 12px rgba(5, 5, 5, .10)` from the global frame stack.
                shadow_sigma: 12.0,
                shadow_color: Color::rgba(5.0 / 255.0, 5.0 / 255.0, 5.0 / 255.0, 0.10),
                // Canonical backdrop blur for buttons/selects/nav.
                blur_sigma: 50.0,
            },
            glass: GlassTokens {
                surface: n2(0.02),
                surface_hover: n2(0.05),
                surface_active: n2(0.10),
                button: n3(0.70),
                button_hover: n2(0.10),
                field: n2(0.05),
                field_focus_border: n2(0.25),
                popover: POPOVER,
                tooltip: TOOLTIP,
                scrim: SCRIM,
                edge: Color::rgba(1.0, 1.0, 1.0, 0.10),
                edge_soft: Color::rgba(1.0, 1.0, 1.0, 0.05),
                inset_highlight: n2(0.06),
                knob_gradient: [n2(0.90), n2(0.30)],
                text_gradient: [n2(0.90), n2(0.50)],
                text_faint: n2(0.40),
                text_placeholder: n2(0.25),
            },
        }
    }
}
