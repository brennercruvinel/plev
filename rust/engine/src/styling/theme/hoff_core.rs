//! The hoff core theme: the light kit of the hoff research design system
//! (the private typescript kit, next + tailwind), transcribed token by
//! token. Nine shades of gray from paper (#FCFCFC) to ink (#121212), six
//! accents, Inter tracked -0.02em, radii 8 / 10 / 12 / 16 / 22 and the
//! measured shadow stacks of its depths page.

use crate::widgets::buttons::tokens::ButtonTokens;
use super::Theme;
use super::intent::MotionPhysics;
use super::scales::{
    ControlTokens, DurationScale, LayoutTokens, ShadowSpec, ShadowTokens, ShapeTokens, SizeTokens,
};
use super::tokens::{
    ColorTokens, EffectTokens, GlassTokens, RadiusScale, SpacingScale, TypographyScale,
};
use crate::styling::color::Color;

/// Shade 1..9 of the colors page, paper to ink.
pub const SHADES: [Color; 9] = [
    Color::hex(0xFCFCFC),
    Color::hex(0xF8F7F7),
    Color::hex(0xF1F1F1),
    Color::hex(0xECECEC),
    Color::hex(0xE2E2E2),
    Color::hex(0x7B7B7B),
    Color::hex(0x323232),
    Color::hex(0x222222),
    Color::hex(0x121212),
];

pub const GREEN: Color = Color::hex(0x49BA61);
pub const BLUE: Color = Color::hex(0x3582FF);
pub const ORANGE: Color = Color::hex(0xE36323);
pub const YELLOW: Color = Color::hex(0xFFB73A);
pub const RED: Color = Color::hex(0xFE5938);
pub const PURPLE: Color = Color::hex(0x8755E9);

fn ink(a: f32) -> Color {
    Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, a)
}

fn black(a: f32) -> Color {
    Color::rgba(0.0, 0.0, 0.0, a)
}

impl TypographyScale {
    /// The core ramp: Inter, tracking -0.02em, 10 / 11 / 13 / 15 / 20 /
    /// 32 / 48 with 16px steps of line height.
    pub fn hoff_core() -> Self {
        Self {
            small: 10.0,
            caption: 11.0,
            body_sm: 13.0,
            body: 15.0,
            title_sm: 20.0,
            title: 32.0,
            display: 48.0,
            line_height_ratio: 1.4,
            family: Some("Inter"),
            tracking_em: -0.02,
        }
    }
}

impl ShadowTokens {
    /// The depths page: surface, subtle, slight hover, modal, tooltip.
    pub fn hoff_core() -> Self {
        Self {
            // Subtle: 0 2px 8px -4px black/25.
            raised: ShadowSpec::new([0.0, 2.0], 8.0, -4.0, black(0.25)),
            // Surface: 0 8px 16px -12px black/13 over 0 1px 4px -4px black/8.
            floating: ShadowSpec::new([0.0, 8.0], 16.0, -12.0, black(0.13)),
            // Modal, four of its layers.
            overlay: [
                ShadowSpec::new([0.0, 16.0], 24.0, -12.0, black(0.10)),
                ShadowSpec::new([0.0, 6.0], 13.0, 0.0, black(0.06)),
                ShadowSpec::new([0.0, 3.0], 3.0, 0.0, black(0.02)),
                ShadowSpec::new([0.0, 1.0], 1.0, 0.0, black(0.02)),
            ],
            // Tooltip: 0 4px 4px -2px black/40.
            hint: ShadowSpec::new([0.0, 4.0], 4.0, -2.0, black(0.40)),
            card: ShadowSpec::new([0.0, 8.0], 16.0, -12.0, black(0.13)),
        }
    }
}

impl Theme {
    /// The hoff core light theme.
    pub fn hoff_core() -> Self {
        let [
            paper,
            shade2,
            shade3,
            shade4,
            shade5,
            shade6,
            shade7,
            _shade8,
            ink9,
        ] = SHADES;
        let colors = ColorTokens {
            bg: paper,
            surface: paper,
            bg_panel: shade2,
            bg_hover: shade3,
            text: ink9,
            text_mid: shade6,
            text_dim: shade6,
            accent: ink9,
            accent_dim: shade7,
            success: GREEN,
            danger: RED,
            warning: ORANGE,
            info: BLUE,
            divider: shade4,
            border_active: shade5,
        };
        let radius = RadiusScale {
            none: 0.0,
            sm: 6.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            full: 9999.0,
        };
        let control = ControlTokens::hoff();
        Self {
            glass: GlassTokens {
                surface: paper,
                surface_hover: shade2,
                surface_active: shade3,
                button: shade3,
                tabs: shade2,
                button_hover: shade4,
                field: paper,
                field_focus_border: shade5,
                popover: paper,
                tooltip: ink9,
                scrim: Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 0.40),
                edge: shade4,
                edge_soft: shade4,
                inset_highlight: Color::rgba(1.0, 1.0, 1.0, 0.33),
                knob_gradient: [paper, shade2],
                text_gradient: [ink9, shade7],
                text_faint: shade6,
                text_placeholder: ink(0.40),
                text_active: ink9,
                text_default: shade6,
                disabled_alpha: 0.30,
                wash_alpha: 0.10,
                wash_hover_alpha: 0.16,
            },
            button: ButtonTokens::hoff_core(),
            colors,
            typography: TypographyScale::hoff_core(),
            spacing: SpacingScale {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
                xxl: 48.0,
            },
            radius,
            motion: MotionPhysics {
                mass: 1.0,
                stiffness: 170.0,
                damping: 26.0,
            },
            effects: EffectTokens {
                shadow_sigma: 4.0,
                shadow_color: black(0.15),
                blur_sigma: 8.0,
            },
            // Rounded-xl (12) on controls, 16 on cards, 22 on wide cards,
            // 8 on chips, 6 on tooltips.
            shape: ShapeTokens {
                pill: 12.0,
                dropdown: 16.0,
                tabs: 12.0,
                card: 16.0,
                block: 10.0,
                item: 8.0,
                nav: 12.0,
                cluster: 10.0,
                tooltip: 6.0,
                micro: 6.0,
            },
            control,
            size: SizeTokens::hoff(),
            duration: DurationScale::hoff(),
            layout: LayoutTokens::hoff(),
            shadows: ShadowTokens::hoff_core(),
        }
    }
}
