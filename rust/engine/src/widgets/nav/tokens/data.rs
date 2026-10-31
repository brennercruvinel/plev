//! The data half of the navigation slot: the stat, the code block, the
//! charts and the graph. The kit draws none of them, so each core value
//! derives from its palette, its type ramp (`TypographyPage`), its radii
//! and its shadows, with the class list it comes from; `derive` writes
//! back what the widget drew with before the slot.

use super::NavText;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::hoff_core::SHADES;

/// One size of a stat: the value, the unit after it, the label under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatLevel {
    pub value: NavText,
    pub unit: NavText,
    pub label: NavText,
}

/// The stat: three sizes, three tones and the spaces between the parts.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatRecipe {
    pub sm: StatLevel,
    pub md: StatLevel,
    pub lg: StatLevel,
    pub value: Color,
    pub unit: Color,
    pub label: Color,
    /// Between the value and its unit.
    pub unit_gap: f32,
    /// Before the delta badge.
    pub delta_gap: f32,
    /// Between the value row and the label, and between the rows when
    /// the delta wraps under the value.
    pub row_gap: f32,
}

impl StatRecipe {
    /// No stat in the kit: the pricing card's price is the nearest
    /// readout (`PriceCard/index.tsx:46-53`): the value `text-[2.5rem]
    /// leading-[3rem] tracking-[-0.03em] font-normal`, the currency
    /// `text-[1.5rem] leading-[2rem] tracking-[-0.03em] font-medium
    /// text-[#7B7B7B]` in `px-0.5 py-1`, the note `p-2 text-[0.75rem]
    /// leading-[1rem] text-[#7B7B7B]`. That is the large stat; the medium
    /// and the small step down the `TypographyPage` ramp (h5 24/32
    /// medium over the 15/24 body; 15/24 semibold over 12/16).
    pub fn hoff_core() -> Self {
        let [_, _, _, _, _, gray, _, _, ink] = SHADES;
        let heading = -0.03;
        Self {
            sm: StatLevel {
                value: NavText::new(15.0, 24.0, 600).tracked(-0.02),
                unit: NavText::new(12.0, 16.0, 500).tracked(-0.01),
                label: NavText::new(11.0, 16.0, 500).tracked(-0.01),
            },
            md: StatLevel {
                value: NavText::new(24.0, 32.0, 500).tracked(heading),
                unit: NavText::new(15.0, 24.0, 500).tracked(-0.02),
                label: NavText::new(12.0, 16.0, 400),
            },
            lg: StatLevel {
                value: NavText::new(40.0, 48.0, 400).tracked(heading),
                unit: NavText::new(24.0, 32.0, 500).tracked(heading),
                label: NavText::new(12.0, 16.0, 400),
            },
            value: ink,
            unit: gray,
            label: gray,
            unit_gap: 2.0,
            delta_gap: 8.0,
            row_gap: 4.0,
        }
    }

    /// The stat of before: base-m over small, title over caption,
    /// headline over base-2r, the unit a ramp step under its value; the
    /// value in the text tone, the unit mid, the label dim.
    pub fn derive(theme: &Theme) -> Self {
        let (ty, colors, sp) = (&theme.typography, &theme.colors, &theme.spacing);
        let of = NavText::of;
        Self {
            sm: StatLevel {
                value: of(&ty.base_m()),
                unit: of(&ty.caption_r()),
                label: of(&ty.small_r()),
            },
            md: StatLevel {
                value: of(&ty.title()),
                unit: of(&ty.base_2r()),
                label: of(&ty.caption_r()),
            },
            lg: StatLevel {
                value: of(&ty.headline()),
                unit: of(&ty.title()),
                label: of(&ty.base_2r()),
            },
            value: colors.text,
            unit: colors.text_mid,
            label: colors.text_dim,
            unit_gap: sp.xs,
            delta_gap: sp.sm,
            row_gap: sp.xs,
        }
    }
}

/// The code block: its box, its text and caption, and the wash behind
/// each kind of a diff's row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CodeRecipe {
    pub text: NavText,
    pub caption: NavText,
    pub pad: f32,
    /// Between the caption and the code.
    pub caption_gap: f32,
    pub radius: f32,
    pub border_width: f32,
    pub fill: Color,
    pub border: Color,
    /// Wrapped code, the caption, and the text of every row (the one
    /// tone that reads over every wash).
    pub code: Color,
    pub caption_color: Color,
    pub row_ink: Color,
    pub added: Color,
    pub removed: Color,
    pub header: Color,
}

impl CodeRecipe {
    /// No code block in the kit: the well of the pricing card
    /// (`PriceCard/index.tsx:43`, `p-3 bg-[#F8F7F7]`) inside the card
    /// shell's rim (`Card/index.tsx:36`, `border-[#ECECEC]`) at the
    /// inputs' radius (`rounded-xl`, `Field:66`); the caption the
    /// typography table's text (`TypographyPage:8`, `text-[0.6875rem]
    /// leading-[1rem] font-medium text-[#7B7B7B]`); the code the
    /// 0.8125rem step on the 1.25rem line in the theme's mono family
    /// (untracked, at its one embedded weight), in the kit's dark gray
    /// `#323232`; a row's text ink. A diff's rows wash in the kit's
    /// green and red at the core glass's wash (10%), a header in
    /// `#F1F1F1`.
    pub fn hoff_core() -> Self {
        use crate::styling::theme::hoff_core::{GREEN, RED};
        let [_, well, wash, line, _, gray, dark, _, ink] = SHADES;
        Self {
            text: NavText::new(13.0, 20.0, 400).in_mono(),
            caption: NavText::new(11.0, 16.0, 500),
            pad: 12.0,
            caption_gap: 4.0,
            radius: 12.0,
            border_width: 1.0,
            fill: well,
            border: line,
            code: dark,
            caption_color: gray,
            row_ink: ink,
            added: tint(GREEN, 0.10),
            removed: tint(RED, 0.10),
            header: wash,
        }
    }

    /// The block of before: the mono step on the glass's field at the nav
    /// radius inside its soft edge, the caption faint, the code mid, the
    /// rows in the text tone over the intent washes.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass) = (&theme.colors, &theme.glass);
        Self {
            text: NavText::of(&theme.typography.mono()).in_mono(),
            caption: NavText::of(&theme.typography.caption_sm()),
            pad: theme.spacing.md,
            caption_gap: theme.spacing.xs,
            radius: theme.shape.nav,
            border_width: theme.control.edge_width,
            fill: glass.field,
            border: glass.edge_soft,
            code: colors.text_mid,
            caption_color: glass.text_faint,
            row_ink: colors.text,
            added: tint(colors.success, glass.wash_alpha),
            removed: tint(colors.danger, glass.wash_alpha),
            header: glass.surface_hover,
        }
    }
}

/// `c` at alpha `a` (the hue kept).
fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], a])
}
