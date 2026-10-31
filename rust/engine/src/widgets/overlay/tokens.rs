//! Overlay tokens (task-104, front B): what the hoff core kit writes as
//! tailwind classes on its anchored overlays, as recipes the widgets
//! read. One field per overlay; the popover is the first.
//!
//! `OverlayTokens::hoff_core` is the kit as written (its Toolbar
//! `ZoomControl` and `ExportFiles` popovers and the header's
//! `NotificationsDropdown`). Every other theme derives its recipes from
//! the theme already built through [`OverlayTokens::derive`], so the
//! overlays render under all of them and stay coherent with the theme's
//! other floating surfaces (the context menu's `glass.popover`).

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonSurface, Theme};

/// The tokens of front B's overlays.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayTokens {
    pub popover: PopoverTokens,
}

impl OverlayTokens {
    /// The hoff core kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        Self {
            popover: PopoverTokens::hoff_core(),
        }
    }

    /// Recipes from a theme that is not the core kit, read once the rest
    /// of the theme is built.
    pub fn derive(theme: &Theme) -> Self {
        Self {
            popover: PopoverTokens::derive(theme),
        }
    }
}

/// Distances an anchored overlay keeps: from its anchor along the main
/// axis, along the cross axis, and from the edge of the container that
/// receives it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clearance {
    /// Between the anchor and the panel (`--anchor-gap`).
    pub gap: f32,
    /// Skid along the cross axis (`--anchor-offset`, floating-ui's
    /// `crossAxis`: positive is toward the end).
    pub offset: f32,
    /// Smallest distance from the container's edge (`--anchor-padding`).
    pub edge: f32,
}

/// The kit's popover trigger (`ZoomControl`'s button): a value and a
/// chevron in a bordered pill, raised while open.
#[derive(Clone, Debug, PartialEq)]
pub struct TriggerTokens {
    pub rest: ButtonSurface,
    pub hover: ButtonSurface,
    /// `data-open`: while its panel is open.
    pub open: ButtonSurface,
    pub height: f32,
    pub min_w: f32,
    pub pad_x: f32,
    /// Between the value and the chevron.
    pub gap: f32,
    pub radius: f32,
    /// Value size and line height in px, and weight.
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    /// The chevron's edge and color; it turns half a turn while open.
    pub chevron: f32,
    pub chevron_color: Color,
}

/// The popover: the panel's surface and geometry, how it keeps clear of
/// its anchor and its container, the kit's width ladder, the trigger.
#[derive(Clone, Debug, PartialEq)]
pub struct PopoverTokens {
    /// The panel: fill, the css shadow list (the border is its ring).
    pub panel: ButtonSurface,
    pub radius: f32,
    /// The smaller radius of the kit's menu-sized panel (`rounded-2xl`),
    /// kept for the Dropdown Menu.
    pub radius_menu: f32,
    pub clearance: Clearance,
    /// Preferred panel widths, small then large (`w-51`, `w-96`); the
    /// container caps them.
    pub widths: [f32; 2],
    /// Space between the panel's edge and its content (`p-2`).
    pub pad: f32,
    pub trigger: TriggerTokens,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn flat(color: Color) -> [Color; 2] {
    [color, color]
}

/// A 1px ring around the surface (`0 0 0 1px color`, a css border as
/// the button recipes write it).
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

impl PopoverTokens {
    /// The hoff core kit, values as written in the tailwind classes.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let paper = hex(0xFCFCFC);
        let shade3 = hex(0xF1F1F1);
        let shade4 = hex(0xECECEC);
        let shade5 = hex(0xE2E2E2);

        // The panel (ZoomControl:19, ExportFiles:15, Notifications:35):
        // bg FCFCFC, border ECECEC, shadow-2xl (0 25px 50px -12px
        // black/25, tailwind v4 theme.css).
        let panel = ButtonSurface {
            fill: flat(paper),
            overlay: NONE,
            shadow: vec![
                ShadowSpec::new([0.0, 25.0], 50.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.25)),
                ring(shade4),
            ],
            text: ink,
        };

        // The trigger (ZoomControl:7): bg F1F1F1 border ECECEC; hover
        // bg FCFCFC border E2E2E2; open border 121212/10 and an inset
        // 0 0 2px 2px white. The text is the body's 121212.
        let rest = ButtonSurface {
            fill: flat(shade3),
            overlay: NONE,
            shadow: vec![ring(shade4)],
            text: ink,
        };
        let hover = ButtonSurface {
            fill: flat(paper),
            shadow: vec![ring(shade5)],
            ..rest.clone()
        };
        let open = ButtonSurface {
            shadow: vec![
                ShadowSpec::inset([0.0, 0.0], 2.0, 2.0, hex(0xFFFFFF)),
                ring(Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.10)),
            ],
            ..rest.clone()
        };

        Self {
            panel,
            // rounded-[1.25rem] (ExportFiles:15, Notifications:35).
            radius: 20.0,
            // rounded-2xl (ZoomControl:19).
            radius_menu: 16.0,
            clearance: Clearance {
                // [--anchor-gap:0.75rem] [--anchor-offset:0.5rem]; no
                // --anchor-padding, headlessui's default 0.
                gap: 12.0,
                offset: 8.0,
                edge: 0.0,
            },
            // w-51 (ZoomControl:19), w-96 (Notifications:35).
            widths: [204.0, 384.0],
            // p-2 (Zoom:54).
            pad: 8.0,
            trigger: TriggerTokens {
                rest,
                hover,
                open,
                // h-10 min-w-22 px-3 gap-2 rounded-xl.
                height: 40.0,
                min_w: 88.0,
                pad_x: 12.0,
                gap: 8.0,
                radius: 12.0,
                // text-[0.875rem] font-medium; the line height is the
                // preflight's 1.5.
                font_size: 14.0,
                line_height: 21.0,
                font_weight: 500,
                // size-4 fill-[#7b7b7b].
                chevron: 16.0,
                chevron_color: hex(0x7B7B7B),
            },
        }
    }

    /// The kit's geometry over the theme's own floating surface: the
    /// context menu's popover glass with its soft edge and the floating
    /// shadow, the button glass for the trigger, the theme's dropdown
    /// radius.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        let glass = &theme.glass;
        let colors = &theme.colors;
        let panel = ButtonSurface {
            fill: flat(glass.popover),
            overlay: NONE,
            shadow: vec![theme.shadows.floating, ring(glass.edge_soft)],
            text: colors.text,
        };
        let rest = ButtonSurface {
            fill: flat(glass.button),
            overlay: NONE,
            shadow: vec![ring(colors.divider)],
            text: colors.text,
        };
        let hover = ButtonSurface {
            fill: flat(glass.button_hover),
            shadow: vec![ring(colors.border_active)],
            ..rest.clone()
        };
        let open = ButtonSurface {
            shadow: vec![ring(colors.border_active)],
            ..rest.clone()
        };
        Self {
            panel,
            radius: theme.shape.dropdown,
            radius_menu: theme.shape.dropdown,
            trigger: TriggerTokens {
                rest,
                hover,
                open,
                chevron_color: colors.text_mid,
                ..kit.trigger
            },
            ..kit
        }
    }
}
