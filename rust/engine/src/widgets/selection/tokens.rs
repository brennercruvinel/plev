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

pub use super::legible::{
    MARK_CONTRAST, TEXT_CONTRAST, hue_sextant, legible, legible_at, legible_on,
};
use super::legible::{full, reads_on};
use crate::styling::color::Color;
use crate::styling::theme::ButtonRecipe;
use crate::styling::theme::ButtonSurface;
use crate::styling::theme::hoff_core::{BLUE, SHADES};
use crate::styling::theme::scales::ShadowSpec;
use crate::widgets::buttons::tokens::ButtonTokens;

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

/// The kit's toolbar shell (`components/Toolbar/index.tsx:72-73`): a
/// raised capsule that holds actions and toggles in a row, sections
/// split by a hairline. The button group draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct ButtonGroupTokens {
    /// The shell: fill, the two soft drops and the border ring (`:72`).
    pub shell: ButtonSurface,
    /// `rounded-[1.25rem]`.
    pub radius: f32,
    /// `p-2` around the items and `gap-2` between them (`:73`).
    pub pad: f32,
    pub gap: f32,
    /// The section hairline (`w-0.25 bg-[#ECECEC]`, `:97`).
    pub separator: Color,
    pub separator_w: f32,
}

/// The kit's switch (`components/Switch/index.tsx`): a well, light off
/// and dark on, and a raised knob. Its measures are the theme's
/// (`control.switch_size`, `switch_knob`); the radio's disc is drawn with
/// the same three surfaces.
#[derive(Clone, Debug, PartialEq)]
pub struct SwitchTokens {
    /// The well off and on; `text` is unused. Off, it is ringed where it
    /// reads at [`MARK_CONTRAST`] on the page and on a row's hover (the
    /// radio's disc is this well): the kit draws it with shades alone.
    pub off: ButtonSurface,
    pub on: ButtonSurface,
    /// The knob (the radio's dot).
    pub knob: ButtonSurface,
    /// Opacity of a disabled switch (the buttons' `opacity-30`; the kit
    /// has no disabled switch).
    pub disabled_alpha: f32,
}

/// The kit's range (`app/globals.css:22-55`, the `.range-slider` thumb;
/// `components/Export/Images/Compression/index.tsx:24` the track and its
/// fill): a tall well that fills to the value, and a raised block thumb.
#[derive(Clone, Debug, PartialEq)]
pub struct SliderTokens {
    /// The well (`h-9 bg-[#f1f1f1] rounded-lg`); `text` is unused.
    pub track: ButtonSurface,
    /// The part up to the value (`rgba(123,123,123,0.3)`).
    pub fill: Color,
    /// The thumb (`#f8f7f7`, its three shadows), ringed where it reads
    /// at [`MARK_CONTRAST`] on the well and on the fill it borders.
    pub thumb: ButtonSurface,
    /// The well's height and radius, the thumb's width and radius.
    pub height: f32,
    pub radius: f32,
    pub thumb_w: f32,
    pub thumb_radius: f32,
    pub disabled_alpha: f32,
}

/// The kit's chip: the card's state tag (`components/Card/index.tsx:56`,
/// `px-3 py-1 border rounded-xl`, 11/16 medium; `bg-[#F1F1F1]`, or
/// `bg-[#FCFCFC] border-[#ECECEC]` on gray) and the notifications'
/// filter pill (`components/Notifications/index.tsx:33`, chosen
/// `bg-[#F1F1F1] text-[#000]`, else `text-[#7B7B7B]`, `hover:text-
/// [#000]`). A quiet chip is the bordered tag, a selected one the filled.
#[derive(Clone, Debug, PartialEq)]
pub struct ChipTokens {
    pub rest: ButtonSurface,
    pub rest_hover: ButtonSurface,
    pub on: ButtonSurface,
    pub on_hover: ButtonSurface,
    /// The hue of an informational chip: the theme's info, where it
    /// stands apart from its success (a constructive chip) on the color
    /// wheel.
    pub info: Color,
    /// `px-3`, `py-1`, `rounded-xl`, the `border`.
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    pub edge: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
}

/// The checkbox. The kit has none: its box is the switch's well, off
/// and on, ringed where it is off by an edge that reads (1.4.11), its
/// mark the knob's paper, its label the radio's list line color. The box
/// and the radius are the theme's (`control.box_size`, `shape.micro`).
#[derive(Clone, Debug, PartialEq)]
pub struct CheckboxTokens {
    pub off: ButtonSurface,
    /// Off under the pointer: the well a shade deeper.
    pub off_hover: ButtonSurface,
    pub on: ButtonSurface,
    /// The check on a box that is on.
    pub mark: Color,
    /// The check on a disabled box that is on, drawn whole: the box fades
    /// to `disabled_alpha`, and the knob's paper would vanish on it, so
    /// the check takes the first gray that still reads at
    /// [`MARK_CONTRAST`] there.
    pub mark_disabled: Color,
    pub label: Color,
    pub disabled_alpha: f32,
}

/// The radio group. The kit has no radio: the row is its single-choice
/// list line (`components/Zoom/index.tsx:15`, `:19`, `:32`) and the disc
/// is its on/off control, the switch ([`SwitchTokens`]): the well off,
/// the dark well on, the knob as the dot.
#[derive(Clone, Debug, PartialEq)]
pub struct RadioTokens {
    /// The disc's and the dot's diameters.
    pub disc: f32,
    pub dot_d: f32,
    /// The slot the disc sits in (`size-6`) and the space after it
    /// (`mr-1`).
    pub slot: f32,
    pub slot_gap: f32,
    /// A row: height, side padding, radius, hover wash (`h-9 px-2
    /// rounded-[0.625rem] hover:bg-[#F1F1F1]`).
    pub row_h: f32,
    pub row_pad_x: f32,
    pub row_radius: f32,
    pub row_hover: Color,
    /// The label (`text-[0.75rem] leading-[1rem] font-medium
    /// text-[#000]`).
    pub label: Color,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub disabled_alpha: f32,
}

/// The calendar. The kit has none: its days press like the toolbar
/// actions (`theme.button.toolbar`), the chosen day is the dark call to
/// action (`theme.button.secondary`), today is ringed as the outline
/// button; what is here is its own geometry and text, from the kit's
/// ladders.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarTokens {
    /// A day cell's edge: a seventh of the width, between the kit's
    /// `h-7` (the small segment) and `size-10` (the toolbar action).
    pub cell_min: f32,
    pub cell_max: f32,
    /// A day's radius, the kit's `rounded-[0.625rem]`.
    pub cell_radius: f32,
    /// Between cells and rows (`gap-1`).
    pub gap: f32,
    /// The month title (the medium button's 14/20 semibold).
    pub title_size: f32,
    pub title_line: f32,
    pub title_weight: u16,
    /// The weekday initials (the small segment's 12px, medium, in the
    /// kit's secondary text `#7B7B7B`).
    pub weekday_size: f32,
    pub weekday_line: f32,
    pub weekday_weight: u16,
    pub weekday: Color,
    /// Today's ring and its width. The outline button's ring
    /// (`#E2E2E2`) is 1.3:1 on paper, too faint to mark a day; the kit's
    /// secondary gray passes 1.4.11.
    pub today: Color,
    pub today_w: f32,
    /// The chosen day: the kit's dark call to action; in a derived theme
    /// the page's text and background turned over.
    pub chosen: ButtonRecipe,
    /// A day's number (the segment's 13px, medium).
    pub day_size: f32,
    pub day_line: f32,
    pub day_weight: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectionTokens {
    pub segmented: SegmentedTokens,
    pub switch: SwitchTokens,
    pub checkbox: CheckboxTokens,
    pub slider: SliderTokens,
    pub chip: ChipTokens,
    pub button_group: ButtonGroupTokens,
    pub radio: RadioTokens,
    pub calendar: CalendarTokens,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

/// `rgba(50,50,50,a)`, the kit's shade for wells and thumbs.
fn shade(a: f32) -> Color {
    Color::rgba(50.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0, a)
}

/// A 1px border inside the surface (`border`, which the box holds).
pub(super) fn border(color: Color) -> ShadowSpec {
    ShadowSpec::inset([0.0, 0.0], 0.0, 1.0, color)
}

/// A 1px ring around the surface (`0 0 0 1px color`, the css border).
pub(super) fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

/// The first of `steps` that reads at [`MARK_CONTRAST`] on every one of
/// `grounds` (each its layers from the bottom up), else the last: the
/// ring or the mark a person must see keeps (1.4.11).
pub(super) fn mark_on(steps: &[Color], grounds: &[&[Color]]) -> Color {
    let last = steps.last().copied().unwrap_or(NONE);
    steps
        .iter()
        .copied()
        .find(|&c| reads_on(MARK_CONTRAST, c, grounds))
        .unwrap_or(last)
}

/// `c` at `alpha` of its own strength: a surface a disabled control
/// fades.
pub(super) fn faded(c: Color, alpha: f32) -> Color {
    Color::rgba(c.0[0], c.0[1], c.0[2], c.0[3] * alpha)
}

pub(super) const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

/// The segment ladder, the same under every theme: the kit's heights
/// and text, `font-semibold`; the line box is the text's own 1.5
/// (tailwind's body leading, which the arbitrary sizes inherit).
pub(super) fn segment_sizes() -> [SegmentMetrics; 2] {
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
        // components/Toolbar/index.tsx:72 `shadow-[0px_1px_4px_-4px_
        // rgba(0,0,0,0.08),0px_8px_16px_-12px_rgba(0,0,0,0.13)] border
        // border-[#ececec] bg-[#fcfcfc] rounded-[1.25rem]`, :73 `gap-2
        // p-2`, :97 the separator `w-0.25 bg-[#ECECEC]`.
        let black = |a: f32| Color::rgba(0.0, 0.0, 0.0, a);
        let button_group = ButtonGroupTokens {
            shell: ButtonSurface {
                fill: [hex(0xFCFCFC); 2],
                overlay: NONE,
                shadow: vec![
                    ShadowSpec::new([0.0, 1.0], 4.0, -4.0, black(0.08)),
                    ShadowSpec::new([0.0, 8.0], 16.0, -12.0, black(0.13)),
                    ring(hex(0xECECEC)),
                ],
                text: ink,
            },
            radius: 20.0,
            pad: 8.0,
            gap: 8.0,
            separator: hex(0xECECEC),
            separator_w: 1.0,
        };
        // components/Switch/index.tsx:11 the well `bg-[#ececec]
        // shadow-[0px_0px_1px_0.5px_rgba(18,18,18,0.10)_inset]`, on
        // `shadow-[0_1px_0.6px_0px_rgba(18,18,18,0.30)_inset]
        // !bg-[#121212]/70`; :17 the knob `size-4.5 bg-[#f8f7f7]
        // shadow-[0px_1px_0.5px_0px_rgba(255,255,255,0.82)_inset,0px_0px_
        // 2.6px_0px_rgba(0,0,0,0.25),0px_1px_4px_0px_rgba(0,0,0,0.14)]`.
        // components/Zoom/index.tsx:15 the line, :19 the `size-6 mr-1`
        // slot, :32 the title.
        let ink_a = |a: f32| Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, a);
        // The well and the thumb are shades on the paper, under 3:1
        // (1.4.11): each keeps the kit's drawing and gains a ring of the
        // kit's border (`#E2E2E2`, Tabs :26) where it reads, else of its
        // secondary gray, else of its graphite.
        let edges = [SHADES[4], SHADES[5], SHADES[6]];
        let well_off = hex(0xECECEC);
        // components/Zoom/index.tsx:15 `hover:bg-[#F1F1F1]`, the row the
        // radio's disc sits on
        let row_hover = hex(0xF1F1F1);
        let well_edge = mark_on(
            &edges,
            &[
                &[SHADES[0]],
                &[SHADES[0], row_hover],
                &[SHADES[0], well_off],
            ],
        );
        let switch = SwitchTokens {
            off: ButtonSurface {
                fill: [well_off; 2],
                overlay: NONE,
                shadow: vec![
                    ShadowSpec::inset([0.0, 0.0], 1.0, 0.5, ink_a(0.10)),
                    ring(well_edge),
                ],
                text: ink,
            },
            on: ButtonSurface {
                fill: [ink_a(0.70); 2],
                overlay: NONE,
                shadow: vec![ShadowSpec::inset([0.0, 1.0], 0.6, 0.0, ink_a(0.30))],
                text: ink,
            },
            knob: ButtonSurface {
                fill: [hex(0xF8F7F7); 2],
                overlay: NONE,
                shadow: vec![
                    ShadowSpec::new([0.0, 0.0], 2.6, 0.0, black(0.25)),
                    ShadowSpec::new([0.0, 1.0], 4.0, 0.0, black(0.14)),
                    ShadowSpec::inset([0.0, 1.0], 0.5, 0.0, Color::rgba(1.0, 1.0, 1.0, 0.82)),
                ],
                text: ink,
            },
            disabled_alpha: 0.30,
        };
        // app/globals.css:22 the thumb `height: 36px; width: 24px;
        // border-radius: 6px; background: #f8f7f7; box-shadow: 0 -1px 4px
        // -2px rgba(0,0,0,.2) inset, 0 0 2.6px -1px rgba(0,0,0,.17), 0 1px
        // 4px 0 rgba(0,0,0,.14)`; Compression :24 `h-9 bg-[#f1f1f1]
        // rounded-lg` and the fill `rgba(123,123,123,0.3)` to the value.
        let range_well = hex(0xF1F1F1);
        let range_fill = Color::rgba(123.0 / 255.0, 123.0 / 255.0, 123.0 / 255.0, 0.3);
        let thumb_edge = mark_on(
            &edges,
            &[
                &[SHADES[0], range_well],
                &[SHADES[0], range_well, range_fill],
            ],
        );
        let slider = SliderTokens {
            track: ButtonSurface {
                fill: [range_well; 2],
                overlay: NONE,
                shadow: Vec::new(),
                text: ink,
            },
            fill: range_fill,
            thumb: ButtonSurface {
                fill: [hex(0xF8F7F7); 2],
                overlay: NONE,
                shadow: vec![
                    ShadowSpec::new([0.0, 0.0], 2.6, -1.0, black(0.17)),
                    ShadowSpec::new([0.0, 1.0], 4.0, 0.0, black(0.14)),
                    ShadowSpec::inset([0.0, -1.0], 4.0, -2.0, black(0.2)),
                    ring(thumb_edge),
                ],
                text: ink,
            },
            height: 36.0,
            radius: 8.0,
            thumb_w: 24.0,
            thumb_radius: 6.0,
            disabled_alpha: switch.disabled_alpha,
        };
        let quiet = legible(hex(0x7B7B7B), SHADES[6], SHADES[0], SHADES[0]);
        let tag = |fill: Color, edge: Color, text: Color| ButtonSurface {
            fill: [fill; 2],
            overlay: NONE,
            shadow: vec![border(edge)],
            text,
        };
        let chip = ChipTokens {
            rest: tag(SHADES[0], SHADES[3], quiet),
            rest_hover: tag(SHADES[0], SHADES[3], hex(0x000000)),
            on: tag(SHADES[2], NONE, hex(0x000000)),
            // a shade deeper under the pointer, as the toolbar's hover
            on_hover: tag(SHADES[3], NONE, hex(0x000000)),
            // the kit theme's info (styling/theme/hoff_core.rs:188)
            info: BLUE,
            pad_x: 12.0,
            pad_y: 4.0,
            radius: 12.0,
            edge: 1.0,
            font_size: 11.0,
            line_height: 16.0,
            font_weight: 500,
        };
        let edge = legible_at(
            MARK_CONTRAST,
            SHADES[4],
            SHADES[5],
            switch.off.fill[0],
            SHADES[0],
        );
        let checkbox = CheckboxTokens {
            off: ButtonSurface {
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            off_hover: ButtonSurface {
                fill: [SHADES[4]; 2],
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            on: full(&switch.on),
            mark: switch.knob.fill[0],
            mark_disabled: mark_on(
                &[switch.knob.fill[0], SHADES[5], SHADES[6]],
                &[&[
                    SHADES[0],
                    faded(full(&switch.on).fill[0], switch.disabled_alpha),
                ]],
            ),
            // the list line's `text-[#000]` (components/Zoom/index.tsx:32),
            // as the radio's label
            label: hex(0x000000),
            disabled_alpha: switch.disabled_alpha,
        };
        let radio = RadioTokens {
            // The knob's size-4.5 as the disc; the dot is the knob drawn
            // at the switch's padding ratio (p-0.5 of h-5.5) inside it.
            disc: 18.0,
            dot_d: 8.0,
            slot: 24.0,
            slot_gap: 4.0,
            row_h: 36.0,
            row_pad_x: 8.0,
            row_radius: 10.0,
            row_hover,
            label: hex(0x000000),
            font_size: 12.0,
            line_height: 16.0,
            font_weight: 500,
            disabled_alpha: 0.30,
        };
        let calendar = CalendarTokens {
            cell_min: 28.0,
            cell_max: 40.0,
            cell_radius: 10.0,
            gap: 4.0,
            title_size: 14.0,
            title_line: 20.0,
            title_weight: 600,
            weekday_size: 12.0,
            weekday_line: 16.0,
            weekday_weight: 500,
            // the kit's secondary #7B7B7B reads 4.1:1 on its paper; shade 7
            // passes AAA, as front C's quiet text (message/tokens.rs)
            weekday: legible(hex(0x7B7B7B), SHADES[6], SHADES[0], SHADES[0]),
            today: legible_at(MARK_CONTRAST, SHADES[5], SHADES[8], SHADES[0], SHADES[0]),
            today_w: 1.0,
            chosen: ButtonTokens::hoff_core().secondary,
            day_size: 13.0,
            day_line: 16.0,
            day_weight: 500,
        };
        Self {
            button_group,
            switch,
            checkbox,
            slider,
            chip,
            radio,
            calendar,
            segmented: SegmentedTokens {
                // :43 `text-[#7B7B7B]`, :40 `hover:text-[#121212]`. The
                // kit's gray reads 3.8:1 on its well; shade 7 passes AAA.
                text: legible(hex(0x7B7B7B), SHADES[6], track.fill[0], SHADES[0]),
                track,
                thumb,
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
}
