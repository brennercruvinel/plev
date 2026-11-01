//! The data half of the navigation slot: the stat, the code block, the
//! charts and the graph. The kit draws none of them, so each core value
//! derives from its palette, its type ramp (`TypographyPage`), its radii
//! and its shadows, with the class list it comes from; `derive` writes
//! back what the widget drew with before the slot.

use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::styling::theme::TypeRun;
use crate::styling::theme::hoff_core::SHADES;

/// One size of a stat: the value, the unit after it, the label under it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StatLevel {
    pub value: TypeRun,
    pub unit: TypeRun,
    pub label: TypeRun,
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
        Self {
            sm: StatLevel {
                value: TypeRole::Title.strong(),
                unit: TypeRole::Label.run(),
                label: TypeRole::Caption.run(),
            },
            md: StatLevel {
                value: TypeRole::H5.run(),
                unit: TypeRole::Title.run(),
                label: TypeRole::Body.run(),
            },
            lg: StatLevel {
                value: TypeRole::H2.run(),
                unit: TypeRole::H5.run(),
                label: TypeRole::Body.run(),
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
        let (colors, sp) = (&theme.colors, &theme.spacing);
        Self {
            sm: StatLevel {
                value: TypeRole::BodyLg.medium(),
                unit: TypeRole::Caption.regular(),
                label: TypeRole::Small.run(),
            },
            md: StatLevel {
                value: TypeRole::Title.run(),
                unit: TypeRole::Body.run(),
                label: TypeRole::Caption.regular(),
            },
            lg: StatLevel {
                value: TypeRole::H3.run(),
                unit: TypeRole::Title.run(),
                label: TypeRole::Body.run(),
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
    pub text: TypeRun,
    pub caption: TypeRun,
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
            text: TypeRole::Code.run(),
            caption: TypeRole::Caption.run(),
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
            text: TypeRole::Code.run(),
            caption: TypeRole::Caption.strong(),
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
