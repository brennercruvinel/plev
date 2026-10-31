//! Selection tokens: the token slot of task-104 front E, selection and
//! dates (#158), for the toggle, toggle group, button group, radio group,
//! calendar and date picker.
//!
//! `SelectionTokens::hoff_core` copies the kit as it is written;
//! [`SelectionTokens::derive`] serves every other theme from the theme already
//! built (its palette and its glass, which the hoff themes write by hand),
//! so the widgets render under all of them (the shape of
//! `crate::widgets::buttons::tokens`). A surface is a [`ButtonSurface`]
//! (fill, css shadow list, wash, text), painted by the buttons'
//! `paint_surface`: one surface model for every pressable thing.
//!
//! The toggle reads the buttons' own `toolbar` recipes and has nothing
//! here.

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonSurface, Theme};

/// The kit's segmented control (`components/Tabs/index.tsx`): a well
/// with a raised thumb that slides under the chosen segment. The toggle
/// group draws it, and the tabs share it.
#[derive(Clone, Debug, PartialEq)]
pub struct SegmentedTokens {
    /// The well: fill, inset shade and ring (`:26`, `:29`).
    pub track: ButtonSurface,
    /// The thumb under the chosen segment, and its label color (`:34`,
    /// `:43`).
    pub thumb: ButtonSurface,
    /// A segment's label when not chosen, and under the pointer (`:40`,
    /// `:43`).
    pub text: Color,
    pub text_hover: Color,
    /// Well and thumb radii (`rounded-xl`, `rounded-lg`).
    pub track_radius: f32,
    pub thumb_radius: f32,
    /// The well's padding around the segments (`p-0.75`) and its border
    /// (`border`, drawn as the ring).
    pub pad: f32,
    pub edge: f32,
    /// The two steps of the kit: `Md` (default, `h-8`, 13px) and `Sm`
    /// (`isMedium`, `h-7`, 12px).
    pub sizes: [SegmentMetrics; 2],
    /// Opacity of a disabled control (the buttons' `opacity-30`).
    pub disabled_alpha: f32,
}

/// One step of the segment ladder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SegmentMetrics {
    /// A segment's height, inside the well's padding.
    pub height: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionTokens {
    pub segmented: SegmentedTokens,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

/// `rgba(50,50,50,a)`, the kit's shade for wells and thumbs.
fn shade(a: f32) -> Color {
    Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, a)
}

/// A 1px ring around the surface (`0 0 0 1px color`, the css border).
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

/// The segment ladder, the same under every theme: the kit's heights
/// and text, `font-semibold`; the line box is the text's own 1.5
/// (tailwind's body leading, which the arbitrary sizes inherit).
fn segment_sizes() -> [SegmentMetrics; 2] {
    [
        SegmentMetrics {
            height: 32.0,
            font_size: 13.0,
            line_height: 19.5,
            font_weight: 600,
        },
        SegmentMetrics {
            height: 28.0,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
        },
    ]
}

impl SelectionTokens {
    /// The hoff core kit, values as written in its class lists.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        // components/Tabs/index.tsx:26 `p-0.75 border border-[#e2e2e2]
        // bg-[#f1f1f1] rounded-xl`, :29 `shadow-[inset_0px_1px_2px_0_
        // rgba(50,50,50,0.10)]`.
        let track = ButtonSurface {
            fill: [hex(0xF1F1F1); 2],
            overlay: NONE,
            shadow: vec![
                ShadowSpec::inset([0.0, 1.0], 2.0, 0.0, shade(0.10)),
                ring(hex(0xE2E2E2)),
            ],
            text: ink,
        };
        // :34 `rounded-lg bg-[#fcfcfc] shadow-[0_1.25px_3px_0px_
        // rgba(50,50,50,0.10)),inset_0px_1.25px_1px_0px_#FFF]`: the list
        // carries a stray `)`, read as written meant (the drop and the
        // white key-light). :43 the chosen label `text-[#121212]`.
        let thumb = ButtonSurface {
            fill: [hex(0xFCFCFC); 2],
            overlay: NONE,
            shadow: vec![
                ShadowSpec::new([0.0, 1.25], 3.0, 0.0, shade(0.10)),
                ShadowSpec::inset([0.0, 1.25], 1.0, 0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
            ],
            text: ink,
        };
        Self {
            segmented: SegmentedTokens {
                track,
                thumb,
                // :43 `text-[#7B7B7B]`, :40 `hover:text-[#121212]`.
                text: hex(0x7B7B7B),
                text_hover: ink,
                track_radius: 12.0,
                thumb_radius: 8.0,
                pad: 3.0,
                edge: 1.0,
                sizes: segment_sizes(),
                disabled_alpha: 0.30,
            },
        }
    }

    /// The recipe of any other theme, from the theme built without it
    /// (`Theme::derive_front_slots` fills the slots last): the well is the
    /// hover tone ringed by the divider, the thumb the background ringed
    /// by the active border, the labels the theme's text and its mid tone.
    pub fn derive(theme: &Theme) -> Self {
        let kit = Self::hoff_core();
        let c = &theme.colors;
        let track = ButtonSurface {
            fill: [c.bg_hover; 2],
            overlay: NONE,
            shadow: vec![ring(c.divider)],
            text: c.text,
        };
        let thumb = ButtonSurface {
            fill: [c.bg; 2],
            overlay: NONE,
            shadow: vec![ring(c.border_active)],
            text: c.text,
        };
        Self {
            segmented: SegmentedTokens {
                track,
                thumb,
                text: c.text_mid,
                text_hover: c.text,
                ..kit.segmented
            },
        }
    }
}
