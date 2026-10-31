//! Disclosure tokens: what the hoff core kit writes as a tailwind class
//! list for the two things that open and close, as recipes the widgets
//! read. One recipe shape serves both looks: the kit's Accordion (the
//! assets sidebar: a divider above each section, a bare header, the
//! chevron in a box) and its Dropdown (the home sidebar: a bordered,
//! rounded trigger with an icon tile), which the [`Collapsible`] draws.
//!
//! `DisclosureTokens::hoff_core` is the kit verbatim (the private
//! typescript design system: `components/SceneAndAssetsSidebar/Assets/
//! Accordion` and `components/HomeSidebar/Dropdown`). Every other theme
//! takes the same geometry and motion with its own palette through
//! [`DisclosureTokens::derive`]; derived from the core theme, the tokens
//! are the kit, value for value.
//!
//! [`Collapsible`]: super::collapsible::Collapsible

use crate::compose::animation::Easing;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::scales::ShadowSpec;

/// A trigger's surface in one state: its fill and its border color
/// (alpha 0 is none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureSurface {
    pub fill: Color,
    pub border: Color,
}

/// The leading icon tile of the Dropdown look (`size-8 rounded-lg`):
/// flat at rest, raised while its section holds the current page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureTile {
    pub size: f32,
    pub radius: f32,
    /// Space between the tile and the label (`mr-3`).
    pub gap: f32,
    /// The icon's edge inside the tile.
    pub icon: f32,
    pub fill_current: Color,
    pub shadow_current: ShadowSpec,
    pub icon_rest: Color,
    pub icon_current: Color,
}

/// One look of a disclosure: the trigger's box, its states, its label,
/// its chevron, the divider above it and the padding of what it reveals.
/// Paddings are css order: top, right, bottom, left.
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureRecipe {
    pub pad: [f32; 4],
    pub radius: f32,
    pub border_width: f32,
    pub rest: DisclosureSurface,
    pub hover: DisclosureSurface,
    /// While open (the kit fills an open Dropdown as it fills a hovered one).
    pub open: DisclosureSurface,
    /// While the section holds the current page (the Dropdown's `active`).
    pub current: DisclosureSurface,
    /// Label size and line height in px, and weight.
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub label: Color,
    /// The box the chevron is centered in, the chevron's edge, the space
    /// before the box, and the chevron's color.
    pub chevron_box: f32,
    pub chevron: f32,
    pub chevron_gap: f32,
    pub chevron_color: Color,
    /// The leading icon tile, for a look that has one.
    pub tile: Option<DisclosureTile>,
    /// The rule above every section but the first (`border-t`,
    /// `first:border-t-0`); width 0 is none.
    pub divider: Color,
    pub divider_width: f32,
    /// Padding around what the disclosure reveals.
    pub content_pad: [f32; 4],
    /// Text revealed as a body: size, line height, weight, color.
    pub body_size: f32,
    pub body_line_height: f32,
    pub body_weight: u16,
    pub body: Color,
}

/// How a disclosure moves: the revealed height and the chevron's half
/// turn, each a duration in seconds and an easing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureMotion {
    pub reveal: f32,
    pub reveal_easing: Easing,
    pub turn: f32,
    pub turn_easing: Easing,
}

impl DisclosureMotion {
    /// The kit: `AnimateHeight duration={500}` with the library's default
    /// `ease`, and tailwind 4's `transition-transform` (150 ms,
    /// `cubic-bezier(0.4, 0, 0.2, 1)`) on the chevron.
    pub const fn hoff_core() -> Self {
        Self {
            reveal: 0.5,
            reveal_easing: Easing::CubicBezier(0.25, 0.1, 0.25, 1.0),
            turn: 0.15,
            turn_easing: Easing::CubicBezier(0.4, 0.0, 0.2, 1.0),
        }
    }
}

impl Default for DisclosureMotion {
    fn default() -> Self {
        Self::hoff_core()
    }
}

/// The disclosure slot of a theme (`theme.disclosure`).
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureTokens {
    /// The kit's Accordion.
    pub accordion: DisclosureRecipe,
    /// The kit's Dropdown, the collapsible's look.
    pub collapsible: DisclosureRecipe,
    pub motion: DisclosureMotion,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], a])
}

const NONE: Color = Color::TRANSPARENT;

const CLEAR: DisclosureSurface = DisclosureSurface {
    fill: NONE,
    border: NONE,
};

impl DisclosureTokens {
    /// The kit, class list by class list.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let gray = hex(0x7B7B7B);
        let line = hex(0xECECEC);
        let wash = hex(0xF1F1F1);
        let paper = hex(0xFCFCFC);

        // Accordion/index.tsx: the section `border-t border-[#ECECEC]
        // first:border-t-0` (:22); the header `px-4 py-3 text-[0.75rem]
        // font-semibold text-[#121212]` (:27), no fill in any state; the
        // chevron `size-4 fill-[#7B7B7B]` in a `size-6` box, pushed to the
        // end by `justify-between` (:31-33); the body `p-4 pt-0` (:45).
        // Line height 18: the 0.75rem label inherits the preflight's 1.5.
        let accordion = DisclosureRecipe {
            pad: [12.0, 16.0, 12.0, 16.0],
            radius: 0.0,
            border_width: 0.0,
            rest: CLEAR,
            hover: CLEAR,
            open: CLEAR,
            current: CLEAR,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 24.0,
            chevron: 16.0,
            chevron_gap: 0.0,
            chevron_color: gray,
            tile: None,
            divider: line,
            divider_width: 1.0,
            content_pad: [0.0, 16.0, 16.0, 16.0],
            // The kit's sections hold thumbnails, not text: a text body
            // takes the Dropdown's list text (Dropdown :73-76).
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        // Dropdown/index.tsx: the trigger `p-0.75 pr-3 border rounded-xl
        // text-[0.75rem] font-semibold text-[#121212] hover:bg-[#f1f1f1]`,
        // filled while open, `bg-[#f1f1f1] border-[#ececec]` while
        // current, `border-transparent` otherwise (:25-30); the tile
        // `size-8 mr-3 rounded-lg`, raised `bg-[#fcfcfc]
        // shadow-[0_0_4px_0_rgba(18,18,18,0.10)] fill-[#121212]` while
        // current, `fill-[#7b7b7b]` otherwise, its icon `size-5` (:35-42);
        // the chevron `size-4 ml-3 fill-[#7b7b7b]` (:58); the list
        // `pt-0.5` (:70), its items `text-[0.75rem] font-semibold
        // text-[#7b7b7b]` (:73-76).
        let collapsible = DisclosureRecipe {
            pad: [3.0, 12.0, 3.0, 3.0],
            radius: 12.0,
            border_width: 1.0,
            rest: CLEAR,
            hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            open: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            current: DisclosureSurface {
                fill: wash,
                border: line,
            },
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 16.0,
            chevron: 16.0,
            chevron_gap: 12.0,
            chevron_color: gray,
            tile: Some(DisclosureTile {
                size: 32.0,
                radius: 8.0,
                gap: 12.0,
                icon: 20.0,
                fill_current: paper,
                shadow_current: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
                icon_rest: gray,
                icon_current: ink,
            }),
            divider: NONE,
            divider_width: 0.0,
            content_pad: [2.0, 0.0, 0.0, 0.0],
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        Self {
            accordion,
            collapsible,
            motion: DisclosureMotion::hoff_core(),
        }
    }

    /// The kit's geometry and motion in a theme's palette: ink is the
    /// text, the gray is the dim text, the line is the divider, the wash
    /// is the hover tone, the paper is the surface.
    pub fn derive(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let kit = Self::hoff_core();
        let wash = DisclosureSurface {
            fill: colors.bg_hover,
            border: NONE,
        };
        let accordion = DisclosureRecipe {
            label: colors.text,
            chevron_color: colors.text_dim,
            divider: colors.divider,
            body: colors.text_dim,
            ..kit.accordion
        };
        let collapsible = DisclosureRecipe {
            hover: wash,
            open: wash,
            current: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            label: colors.text,
            chevron_color: colors.text_dim,
            tile: kit.collapsible.tile.map(|tile| DisclosureTile {
                fill_current: colors.surface,
                shadow_current: ShadowSpec {
                    color: tint(colors.text, 0.10),
                    ..tile.shadow_current
                },
                icon_rest: colors.text_dim,
                icon_current: colors.text,
                ..tile
            }),
            body: colors.text_dim,
            ..kit.collapsible
        };
        Self {
            accordion,
            collapsible,
            motion: kit.motion,
        }
    }
}
