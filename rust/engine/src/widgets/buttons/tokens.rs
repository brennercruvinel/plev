//! Button tokens: what the hoff core design system writes as a tailwind
//! class list per button, as a recipe the widgets read. A recipe is the
//! css box model of one surface (fill gradient, box-shadow list with its
//! rings and insets, text color) at rest, hovered and pressed. The sizes
//! are the height ladder of the same kit (28 / 36 / 40 / 44 for labeled
//! buttons, 24 / 36 / 40 for icon-only ones).
//!
//! `ButtonTokens::hoff_core` is the measured kit (the private typescript
//! design system: its Button, SubmitButton and buttons page). Every
//! other theme derives flat recipes from its palette through
//! [`ButtonTokens::derive`], so the widgets render under all of them.

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::tokens::ColorTokens;

/// One surface state: fill, shadow list, label color.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonSurface {
    /// Vertical gradient, top then bottom; equal stops are a flat fill,
    /// alpha 0 is no fill.
    pub fill: [Color; 2],
    /// A wash painted over the fill (css `after:bg-white/40`); alpha 0
    /// is none.
    pub overlay: Color,
    /// The css `box-shadow` list, first entry on top. Rings are spread-1
    /// blur-0 layers, key-lights are inset layers.
    pub shadow: Vec<ShadowSpec>,
    /// Label and icon color.
    pub text: Color,
}

/// A button variant: its surface at rest, hovered and pressed, and how
/// far a disabled one fades.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonRecipe {
    pub rest: ButtonSurface,
    pub hover: ButtonSurface,
    pub active: ButtonSurface,
    /// Opacity of the whole button when disabled (css `opacity-30`).
    pub disabled_alpha: f32,
}

/// Geometry of one step of the labeled-button ladder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonMetrics {
    pub height: f32,
    pub pad_x: f32,
    pub radius: f32,
    /// Label size and line height in px (14/20, 12/16...).
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// Leading icon edge.
    pub icon: f32,
    /// Gap between icon and label.
    pub gap: f32,
}

/// Geometry of one step of the icon-button ladder (square).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IconButtonMetrics {
    pub size: f32,
    pub radius: f32,
    pub icon: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonTokens {
    /// The pale raised button (`isPrimary`).
    pub primary: ButtonRecipe,
    /// The dark call-to-action (`isSecondary`).
    pub secondary: ButtonRecipe,
    /// The orange warning button (`isOrange`).
    pub warning: ButtonRecipe,
    /// Bordered, flat: the like button and the bordered icon button.
    pub outline: ButtonRecipe,
    /// No border, a wash on hover: the plain icon buttons.
    pub ghost: ButtonRecipe,
    /// Toolbar item at rest (a transparent border keeps the size).
    pub toolbar: ButtonRecipe,
    /// Toolbar item when its tool is selected: bordered, filled, pressed in.
    pub toolbar_active: ButtonRecipe,
    /// Labeled ladder: Xs 28 / Sm 36 / Md 40 / Lg 44.
    pub sizes: [ButtonMetrics; 4],
    /// Icon-only ladder: Sm 24 / Md 36 / Lg 40.
    pub icon_sizes: [IconButtonMetrics; 3],
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn white(a: f32) -> Color {
    Color::rgba(1.0, 1.0, 1.0, a)
}

fn black(a: f32) -> Color {
    Color::rgba(0.0, 0.0, 0.0, a)
}

/// A 1px ring around the surface (`0 0 0 1px color`).
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

fn flat(color: Color) -> [Color; 2] {
    [color, color]
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl ButtonTokens {
    /// The hoff core kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let paper = hex(0xFCFCFC);
        let shade2 = hex(0xF8F7F7);
        let shade3 = hex(0xF1F1F1);
        let shade5 = hex(0xE2E2E2);

        // isPrimary: gradient E5E5E5 -> E2E2E2, shadow 0 3px 4px -1px
        // black/15, inset 0 1px 0 white/33, ring D4D4D4; hover lifts the
        // shadow (0 3px 8px -2px black/30, inset white/70, ring E6E6E6)
        // and washes white/40 over; press drops the wash.
        let primary_fill = [hex(0xE5E5E5), hex(0xE2E2E2)];
        let primary_rest = ButtonSurface {
            fill: primary_fill,
            overlay: NONE,
            shadow: vec![
                ShadowSpec::new([0.0, 3.0], 4.0, -1.0, black(0.15)),
                ShadowSpec::inset([0.0, 1.0], 0.0, 0.0, white(0.33)),
                ring(hex(0xD4D4D4)),
            ],
            text: ink,
        };
        let primary_hover = ButtonSurface {
            fill: primary_fill,
            overlay: white(0.40),
            shadow: vec![
                ShadowSpec::new([0.0, 3.0], 8.0, -2.0, black(0.30)),
                ShadowSpec::inset([0.0, 1.0], 0.0, 0.0, white(0.70)),
                ring(hex(0xE6E6E6)),
            ],
            text: ink,
        };
        let primary_active = ButtonSurface {
            overlay: NONE,
            ..primary_hover.clone()
        };
        let primary = ButtonRecipe {
            rest: primary_rest,
            hover: primary_hover,
            active: primary_active,
            disabled_alpha: 0.30,
        };

        // isSecondary: dark gradient 323232 -> 222222 under a white/15
        // top key-light, 0 2px 4px -1px #0D0D0D/50, a bottom inset of
        // #121212 and a #333 ring; hover fades the gradient to 90%.
        let secondary_shadow = vec![
            ShadowSpec::inset([0.0, 0.5], 1.0, 0.0, white(0.15)),
            ShadowSpec::new(
                [0.0, 2.0],
                4.0,
                -1.0,
                Color::rgba(13.0 / 255.0, 13.0 / 255.0, 13.0 / 255.0, 0.50),
            ),
            ShadowSpec::inset([0.0, -1.0], 1.2, 0.35, ink),
            ring(hex(0x333333)),
        ];
        let secondary_rest = ButtonSurface {
            fill: [hex(0x323232), hex(0x222222)],
            overlay: NONE,
            shadow: secondary_shadow.clone(),
            text: paper,
        };
        let secondary_hover = ButtonSurface {
            fill: [
                Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, 0.90),
                Color::rgba(34.0 / 255.0, 34.0 / 255.0, 34.0 / 255.0, 0.90),
            ],
            ..secondary_rest.clone()
        };
        let secondary = ButtonRecipe {
            rest: secondary_rest.clone(),
            hover: secondary_hover,
            active: secondary_rest,
            disabled_alpha: 0.30,
        };

        // isOrange: E36323 -> DF5A18, top key-light white/33, ring
        // BF4A0F and an orange glow 0 3px 4px -1px rgba(252,96,16,.95);
        // hover washes white/10.
        let warning_rest = ButtonSurface {
            fill: [hex(0xE36323), hex(0xDF5A18)],
            overlay: NONE,
            shadow: vec![
                ShadowSpec::inset([0.0, 1.0], 0.0, 0.0, white(0.33)),
                ring(hex(0xBF4A0F)),
                ShadowSpec::new(
                    [0.0, 3.0],
                    4.0,
                    -1.0,
                    Color::rgba(252.0 / 255.0, 96.0 / 255.0, 16.0 / 255.0, 0.95),
                ),
            ],
            text: paper,
        };
        let warning_hover = ButtonSurface {
            overlay: white(0.10),
            ..warning_rest.clone()
        };
        let warning = ButtonRecipe {
            rest: warning_rest.clone(),
            hover: warning_hover,
            active: warning_rest,
            disabled_alpha: 0.30,
        };

        // The like button and the bordered icon button: E2E2E2 border,
        // paper fill, F8F7F7 / F1F1F1 on hover, pressed in by an inset
        // 0 0 4px black/15.
        let outline_rest = ButtonSurface {
            fill: flat(paper),
            overlay: NONE,
            shadow: vec![ring(shade5)],
            text: ink,
        };
        let outline_hover = ButtonSurface {
            fill: flat(shade2),
            ..outline_rest.clone()
        };
        let outline_active = ButtonSurface {
            fill: flat(paper),
            shadow: vec![
                ShadowSpec::inset([0.0, 0.0], 4.0, 0.0, black(0.15)),
                ring(shade5),
            ],
            ..outline_rest.clone()
        };
        let outline = ButtonRecipe {
            rest: outline_rest,
            hover: outline_hover,
            active: outline_active,
            disabled_alpha: 0.50,
        };

        // The plain icon buttons: nothing at rest, F1F1F1 on hover, a
        // faint inset 0 0 2.1px black/15 when pressed; icon in shade 6.
        let ghost_rest = ButtonSurface {
            fill: flat(NONE),
            overlay: NONE,
            shadow: Vec::new(),
            text: hex(0x7B7B7B),
        };
        let ghost_hover = ButtonSurface {
            fill: flat(shade3),
            ..ghost_rest.clone()
        };
        let ghost_active = ButtonSurface {
            fill: flat(shade3),
            shadow: vec![ShadowSpec::inset([0.0, 0.0], 2.1, 0.0, black(0.15))],
            ..ghost_rest.clone()
        };
        let ghost = ButtonRecipe {
            rest: ghost_rest,
            hover: ghost_hover,
            active: ghost_active,
            disabled_alpha: 0.50,
        };

        // Toolbar: like ghost but the icon is ink and disabled fades to
        // 40%; the selected tool gets the E2E2E2 border, the F1F1F1 fill
        // and the pressed-in pair of insets.
        let toolbar_rest = ButtonSurface {
            fill: flat(NONE),
            overlay: NONE,
            shadow: Vec::new(),
            text: ink,
        };
        let toolbar_hover = ButtonSurface {
            fill: flat(shade3),
            ..toolbar_rest.clone()
        };
        let toolbar = ButtonRecipe {
            rest: toolbar_rest,
            hover: toolbar_hover.clone(),
            active: toolbar_hover,
            disabled_alpha: 0.40,
        };
        let toolbar_active_surface = ButtonSurface {
            fill: flat(shade3),
            overlay: NONE,
            shadow: vec![
                ShadowSpec::inset(
                    [0.0, -1.0],
                    3.0,
                    0.0,
                    Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.15),
                ),
                ShadowSpec::inset([0.0, 1.25], 1.0, 0.0, white(1.0)),
                ring(shade5),
            ],
            text: ink,
        };
        let toolbar_active = ButtonRecipe {
            rest: toolbar_active_surface.clone(),
            hover: toolbar_active_surface.clone(),
            active: toolbar_active_surface,
            disabled_alpha: 0.40,
        };

        Self {
            primary,
            secondary,
            warning,
            outline,
            ghost,
            toolbar,
            toolbar_active,
            sizes: [
                // Xs: the like-comment chip, h-7 px-2 rounded-lg 13/16.
                ButtonMetrics {
                    height: 28.0,
                    pad_x: 8.0,
                    radius: 8.0,
                    font_size: 13.0,
                    line_height: 16.0,
                    font_weight: 600,
                    icon: 16.0,
                    gap: 8.0,
                },
                // Sm: isSmall, h-9 px-5 rounded-[0.625rem] 12/16.
                ButtonMetrics {
                    height: 36.0,
                    pad_x: 20.0,
                    radius: 10.0,
                    font_size: 12.0,
                    line_height: 16.0,
                    font_weight: 600,
                    icon: 20.0,
                    gap: 8.0,
                },
                // Md: the default, h-10 px-6 rounded-xl 14/20.
                ButtonMetrics {
                    height: 40.0,
                    pad_x: 24.0,
                    radius: 12.0,
                    font_size: 14.0,
                    line_height: 20.0,
                    font_weight: 600,
                    icon: 20.0,
                    gap: 8.0,
                },
                // Lg: the like button, h-11 px-3.5 rounded-xl 12/16.
                ButtonMetrics {
                    height: 44.0,
                    pad_x: 14.0,
                    radius: 12.0,
                    font_size: 12.0,
                    line_height: 16.0,
                    font_weight: 600,
                    icon: 16.0,
                    gap: 8.0,
                },
            ],
            icon_sizes: [
                // Sm: size-6 rounded-lg, 16px icon.
                IconButtonMetrics {
                    size: 24.0,
                    radius: 8.0,
                    icon: 16.0,
                },
                // Md: size-9 rounded-[0.625rem], 20px icon.
                IconButtonMetrics {
                    size: 36.0,
                    radius: 10.0,
                    icon: 20.0,
                },
                // Lg: size-10 rounded-xl, 16 to 20px icon.
                IconButtonMetrics {
                    size: 40.0,
                    radius: 12.0,
                    icon: 20.0,
                },
            ],
        }
    }

    /// Flat recipes from a palette, for the themes that are not the core
    /// kit, in the kit's order of strength: the panel tone as the primary
    /// (a step off the page), the accent as the secondary (the call to
    /// action, the strongest), the warning color as the warning, dividers
    /// as rings. Same ladder as the kit.
    pub fn derive(colors: &ColorTokens) -> Self {
        let kit = Self::hoff_core();
        let text_on = |fill: Color| {
            // The page's text or the page's own tone, whichever contrasts
            // more with the fill as the page shows it (a translucent fill
            // is mostly the page).
            let lum = |c: [f32; 4]| 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            let (f, b, a) = (fill.0, colors.bg.0, fill.0[3]);
            let seen = lum([0, 1, 2, 3].map(|i| f[i] * a + b[i] * (1.0 - a)));
            let ratio = |c: Color| {
                let l = lum(c.0);
                (l.max(seen) + 0.05) / (l.min(seen) + 0.05)
            };
            if ratio(colors.bg) > ratio(colors.text) {
                colors.bg
            } else {
                colors.text
            }
        };
        let tint = |c: Color, a: f32| Color([c.0[0], c.0[1], c.0[2], a]);
        let solid = |fill: Color, hover: Color, disabled_alpha: f32| {
            let rest = ButtonSurface {
                fill: flat(fill),
                overlay: NONE,
                shadow: vec![ring(colors.divider)],
                text: text_on(fill),
            };
            let hovered = ButtonSurface {
                fill: flat(hover),
                text: text_on(hover),
                ..rest.clone()
            };
            ButtonRecipe {
                rest: rest.clone(),
                hover: hovered,
                active: rest,
                disabled_alpha,
            }
        };
        let quiet = |ring_color: Color, text: Color, disabled_alpha: f32| {
            let rest = ButtonSurface {
                fill: flat(NONE),
                overlay: NONE,
                shadow: if ring_color.0[3] > 0.0 {
                    vec![ring(ring_color)]
                } else {
                    Vec::new()
                },
                text,
            };
            let hovered = ButtonSurface {
                fill: flat(colors.bg_hover),
                ..rest.clone()
            };
            ButtonRecipe {
                rest,
                hover: hovered.clone(),
                active: hovered,
                disabled_alpha,
            }
        };
        let toolbar_active_surface = ButtonSurface {
            fill: flat(colors.bg_hover),
            overlay: NONE,
            shadow: vec![ring(colors.border_active)],
            text: colors.text,
        };
        Self {
            // The kit's order of strength, whatever the palette: the pale
            // `isPrimary` is a step off the page, the dark `isSecondary`
            // (the call to action: a modal's confirm) the most contrasted
            // fill the theme has.
            primary: solid(colors.bg_panel, colors.bg_hover, 0.30),
            secondary: solid(colors.accent, tint(colors.accent, 0.85), 0.30),
            warning: solid(colors.warning, tint(colors.warning, 0.85), 0.30),
            outline: quiet(colors.divider, colors.text, 0.50),
            ghost: quiet(NONE, colors.text_mid, 0.50),
            toolbar: quiet(NONE, colors.text, 0.40),
            toolbar_active: ButtonRecipe {
                rest: toolbar_active_surface.clone(),
                hover: toolbar_active_surface.clone(),
                active: toolbar_active_surface,
                disabled_alpha: 0.40,
            },
            sizes: kit.sizes,
            icon_sizes: kit.icon_sizes,
        }
    }
}
