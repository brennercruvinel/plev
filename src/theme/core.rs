// ============================================================================
// Theme -- the root struct (RULE-08)
// ============================================================================

use super::intent::{Intent, MotionPhysics};
use super::tokens::{
    ColorTokens, EffectTokens, GlassTokens, RadiusScale, SpacingScale, TypographyScale,
};
use crate::color::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub colors: ColorTokens,
    pub typography: TypographyScale,
    pub spacing: SpacingScale,
    pub radius: RadiusScale,
    pub motion: MotionPhysics,
    pub effects: EffectTokens,
    /// Translucent surface recipe (HOFF dark-glass language).
    pub glass: GlassTokens,
}

/// The default plev theme is HOFF (see [`Theme::hoff`]).
impl Default for Theme {
    fn default() -> Self {
        Self::hoff()
    }
}

impl Theme {
    /// Dark theme matching the pre-HOFF showcase palette.
    pub fn dark() -> Self {
        let colors = ColorTokens {
            bg: Color::hex(0x000000),
            surface: Color::hex(0x0a0a0a),
            bg_panel: Color::hex(0x111111),
            bg_hover: Color::hex(0x1a1a1a),
            text: Color::hex(0xe0e0e0),
            text_dim: Color::hex(0x555555),
            text_mid: Color::hex(0x888888),
            accent: Color::hex(0xffffff),
            accent_dim: Color::hex(0x444444),
            success: Color::rgb(0.20, 0.80, 0.45),
            danger: Color::rgb(1.0, 0.30, 0.25),
            warning: Color::rgb(1.0, 0.85, 0.20),
            info: Color::rgb(0.0, 0.85, 0.95),
            divider: Color::hex(0x222222),
            border_active: Color::hex(0x444444),
        };
        Self {
            glass: GlassTokens::derive(&colors),
            colors,
            typography: TypographyScale {
                caption: 11.0,
                body_sm: 13.0,
                body: 16.0,
                title_sm: 20.0,
                title: 28.0,
                display: 36.0,
                line_height_ratio: 1.3,
            },
            spacing: SpacingScale {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
                xxl: 48.0,
            },
            radius: RadiusScale {
                none: 0.0,
                sm: 2.0,
                md: 4.0,
                lg: 8.0,
                xl: 12.0,
                full: 9999.0,
            },
            motion: MotionPhysics {
                mass: 1.0,
                stiffness: 170.0,
                damping: 26.0,
            },
            effects: EffectTokens {
                shadow_sigma: 4.0,
                shadow_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
                blur_sigma: 8.0,
            },
        }
    }

    /// Light theme.
    pub fn light() -> Self {
        let colors = ColorTokens {
            bg: Color::rgb(0.98, 0.98, 0.99),
            surface: Color::rgb(1.0, 1.0, 1.0),
            bg_panel: Color::hex(0xf0f0f0),
            bg_hover: Color::hex(0xe8e8ec),
            text: Color::rgba(0.10, 0.10, 0.12, 1.0),
            text_dim: Color::rgba(0.45, 0.45, 0.50, 1.0),
            text_mid: Color::rgba(0.30, 0.30, 0.35, 1.0),
            accent: Color::rgb(0.22, 0.45, 0.95),
            accent_dim: Color::rgb(0.15, 0.30, 0.65),
            success: Color::rgb(0.15, 0.65, 0.35),
            danger: Color::rgb(0.90, 0.22, 0.18),
            warning: Color::rgb(0.85, 0.65, 0.05),
            info: Color::rgb(0.0, 0.65, 0.80),
            divider: Color::rgba(0.85, 0.85, 0.88, 1.0),
            border_active: Color::hex(0x9699a3),
        };
        Self {
            glass: GlassTokens::derive(&colors),
            colors,
            ..Self::dark()
        }
    }

    /// Named palette lookup.
    pub fn named(name: &str) -> Option<Self> {
        let base = || ColorTokens {
            success: Color::rgb(0.20, 0.80, 0.45),
            danger: Color::rgb(1.0, 0.30, 0.25),
            warning: Color::rgb(1.0, 0.85, 0.20),
            info: Color::rgb(0.0, 0.85, 0.95),
            ..Self::dark().colors
        };
        let colors = match name {
            "hoff" => return Some(Self::hoff()),
            "plev" => return Some(Self::dark()),
            "catppuccin" => ColorTokens {
                bg: Color::hex(0x11111b),
                surface: Color::hex(0x181825),
                bg_panel: Color::hex(0x1e1e2e),
                bg_hover: Color::hex(0x313244),
                text: Color::hex(0xcdd6f4),
                text_mid: Color::hex(0xbac2de),
                text_dim: Color::hex(0x6c7086),
                accent: Color::hex(0xcba6f7),
                accent_dim: Color::hex(0x585b70),
                divider: Color::hex(0x313244),
                border_active: Color::hex(0x585b70),
                ..base()
            },
            "dracula" => ColorTokens {
                bg: Color::hex(0x141119),
                surface: Color::hex(0x1c1d26),
                bg_panel: Color::hex(0x282a36),
                bg_hover: Color::hex(0x504364),
                text: Color::hex(0xf8f8f2),
                text_mid: Color::hex(0xa186c7),
                text_dim: Color::hex(0x6272a4),
                accent: Color::hex(0xbd93f9),
                accent_dim: Color::hex(0x3c324b),
                divider: Color::hex(0x3c324b),
                border_active: Color::hex(0x6272a4),
                ..base()
            },
            "tokyo-night" => ColorTokens {
                bg: Color::hex(0x1a1b26),
                surface: Color::hex(0x1e2030),
                bg_panel: Color::hex(0x24283b),
                bg_hover: Color::hex(0x292e42),
                text: Color::hex(0xc0caf5),
                text_mid: Color::hex(0xa9b1d6),
                text_dim: Color::hex(0x565f89),
                accent: Color::hex(0x7aa2f7),
                accent_dim: Color::hex(0x3b4261),
                divider: Color::hex(0x3b4261),
                border_active: Color::hex(0x545c7e),
                ..base()
            },
            "rose-pine" => ColorTokens {
                bg: Color::hex(0x191724),
                surface: Color::hex(0x1f1d2e),
                bg_panel: Color::hex(0x26233a),
                bg_hover: Color::hex(0x2a2738),
                text: Color::hex(0xe0def4),
                text_mid: Color::hex(0x908caa),
                text_dim: Color::hex(0x6e6a86),
                accent: Color::hex(0xc4a7e7),
                accent_dim: Color::hex(0x403d52),
                divider: Color::hex(0x403d52),
                border_active: Color::hex(0x524f67),
                ..base()
            },
            "nord" => ColorTokens {
                bg: Color::hex(0x2e3440),
                surface: Color::hex(0x2e3440),
                bg_panel: Color::hex(0x3b4252),
                bg_hover: Color::hex(0x434c5e),
                text: Color::hex(0xeceff4),
                text_mid: Color::hex(0xd8dee9),
                text_dim: Color::hex(0x4c566a),
                accent: Color::hex(0x88c0d0),
                accent_dim: Color::hex(0x3b4252),
                divider: Color::hex(0x3b4252),
                border_active: Color::hex(0x4c566a),
                ..base()
            },
            "gruvbox" => ColorTokens {
                bg: Color::hex(0x1d2021),
                surface: Color::hex(0x282828),
                bg_panel: Color::hex(0x292828),
                bg_hover: Color::hex(0x3c3836),
                text: Color::hex(0xd4be98),
                text_mid: Color::hex(0xc5b18d),
                text_dim: Color::hex(0x7c6f64),
                accent: Color::hex(0x89b482),
                accent_dim: Color::hex(0x444343),
                divider: Color::hex(0x444343),
                border_active: Color::hex(0x665c54),
                ..base()
            },
            "github-dark" => ColorTokens {
                bg: Color::hex(0x010409),
                surface: Color::hex(0x0d1117),
                bg_panel: Color::hex(0x0d1117),
                bg_hover: Color::hex(0x161b22),
                text: Color::hex(0xe6edf3),
                text_mid: Color::hex(0xcbced3),
                text_dim: Color::hex(0x484f58),
                accent: Color::hex(0x58a6ff),
                accent_dim: Color::hex(0x30363d),
                divider: Color::hex(0x30363d),
                border_active: Color::hex(0x484f58),
                ..base()
            },
            "one-dark" => ColorTokens {
                bg: Color::hex(0x1e2227),
                surface: Color::hex(0x23272e),
                bg_panel: Color::hex(0x23272e),
                bg_hover: Color::hex(0x2c313a),
                text: Color::hex(0xabb2bf),
                text_mid: Color::hex(0x828997),
                text_dim: Color::hex(0x5c6370),
                accent: Color::hex(0x61afef),
                accent_dim: Color::hex(0x3e4452),
                divider: Color::hex(0x3e4452),
                border_active: Color::hex(0x4b5263),
                ..base()
            },
            "kanagawa" => ColorTokens {
                bg: Color::hex(0x181820),
                surface: Color::hex(0x1a1a22),
                bg_panel: Color::hex(0x1f1f28),
                bg_hover: Color::hex(0x2a2a37),
                text: Color::hex(0xdcd7ba),
                text_mid: Color::hex(0x9cabca),
                text_dim: Color::hex(0x727169),
                accent: Color::hex(0x7e9cd8),
                accent_dim: Color::hex(0x2a2a37),
                divider: Color::hex(0x2a2a37),
                border_active: Color::hex(0x4f4f66),
                ..base()
            },
            "moonlight" => ColorTokens {
                bg: Color::hex(0x1e2030),
                surface: Color::hex(0x212337),
                bg_panel: Color::hex(0x212337),
                bg_hover: Color::hex(0x373c5c),
                text: Color::hex(0xc8d3f5),
                text_mid: Color::hex(0x828bb8),
                text_dim: Color::hex(0x758096),
                accent: Color::hex(0x82aaff),
                accent_dim: Color::hex(0x191a2a),
                divider: Color::hex(0x191a2a),
                border_active: Color::hex(0x444a73),
                ..base()
            },
            _ => return None,
        };
        Some(Self {
            glass: GlassTokens::derive(&colors),
            colors,
            ..Self::dark()
        })
    }

    /// Resolve intent to a semantic color.
    pub fn intent_color(&self, intent: Intent) -> Color {
        match intent {
            Intent::Neutral => self.colors.text,
            Intent::Constructive => self.colors.success,
            Intent::Destructive => self.colors.danger,
            Intent::Informational => self.colors.info,
        }
    }

    /// Resolve intent to motion physics.
    pub fn intent_motion(&self, intent: Intent) -> MotionPhysics {
        self.motion.for_intent(intent)
    }

    /// Resolve intent to an AccessKit role hint.
    #[cfg(feature = "accessibility")]
    pub fn intent_role(&self, intent: Intent) -> accesskit::Role {
        match intent {
            Intent::Neutral => accesskit::Role::GenericContainer,
            Intent::Constructive => accesskit::Role::Button,
            Intent::Destructive => accesskit::Role::Button,
            Intent::Informational => accesskit::Role::Label,
        }
    }
}
