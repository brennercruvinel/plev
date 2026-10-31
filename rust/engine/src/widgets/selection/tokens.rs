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
use crate::styling::theme::hoff_core::SHADES;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonRecipe, ButtonSurface, Theme};
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
    /// The well off and on; `text` is unused.
    pub off: ButtonSurface,
    pub on: ButtonSurface,
    /// The knob (the radio's dot).
    pub knob: ButtonSurface,
    /// Opacity of a disabled switch (the buttons' `opacity-30`; the kit
    /// has no disabled switch).
    pub disabled_alpha: f32,
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

/// A 1px ring around the surface (`0 0 0 1px color`, the css border).
fn ring(color: Color) -> ShadowSpec {
    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, color)
}

const NONE: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

/// WCAG 2.2 AAA for text (1.4.6), the bar the front's text colors keep:
/// a color of the kit or of a theme that falls under it gives way to one
/// that passes (`theme/tests/contrast.rs` measures the pairs).
pub const TEXT_CONTRAST: f32 = 7.0;

/// WCAG 2.2 AA for the parts of a control a person must see (1.4.11):
/// today's ring.
pub const MARK_CONTRAST: f32 = 3.0;

/// `fg` composed over the opaque linear `under`, in linear light.
fn over(fg: Color, under: [f32; 3]) -> [f32; 3] {
    let [r, g, b, a] = fg.to_linear_array();
    let mix = |c: f32, u: f32| c * a + u * (1.0 - a);
    [mix(r, under[0]), mix(g, under[1]), mix(b, under[2])]
}

/// `surface` at its full strength: its fill opaque. The switch's well on
/// is the ink at 70%, where the knob's paper reads 2.8:1; a check must
/// read (1.4.11), so the box takes the ink whole.
fn full(surface: &ButtonSurface) -> ButtonSurface {
    let opaque = |c: Color| Color::rgba(c.0[0], c.0[1], c.0[2], 1.0);
    ButtonSurface {
        fill: surface.fill.map(opaque),
        ..surface.clone()
    }
}

/// WCAG contrast of two linear colors, lighter over darker.
fn ratio(a: [f32; 3], b: [f32; 3]) -> f32 {
    let lum = |[r, g, b]: [f32; 3]| 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// `preferred` where it reads at [`TEXT_CONTRAST`] on `fill` laid over
/// `page`, else `fallback`: the kit's or the theme's quiet text where it
/// is legible, the stronger one where it is not.
pub fn legible(preferred: Color, fallback: Color, fill: Color, page: Color) -> Color {
    legible_at(TEXT_CONTRAST, preferred, fallback, fill, page)
}

/// [`legible`] at the contrast `need`.
pub fn legible_at(need: f32, preferred: Color, fallback: Color, fill: Color, page: Color) -> Color {
    let under = over(fill, over(page, [0.0; 3]));
    if ratio(over(preferred, under), under) >= need {
        preferred
    } else {
        fallback
    }
}

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
        let switch = SwitchTokens {
            off: ButtonSurface {
                fill: [hex(0xECECEC); 2],
                overlay: NONE,
                shadow: vec![ShadowSpec::inset([0.0, 0.0], 1.0, 0.5, ink_a(0.10))],
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
            row_hover: hex(0xF1F1F1),
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
        let button_group = ButtonGroupTokens {
            shell: ButtonSurface {
                fill: [c.surface; 2],
                overlay: NONE,
                shadow: vec![ring(c.divider)],
                text: c.text,
            },
            separator: c.divider,
            ..kit.button_group
        };
        // the kit's call to action is ink on paper, the page's colors
        // turned over: so is a derived theme's
        let chosen = ButtonSurface {
            fill: [c.text; 2],
            overlay: NONE,
            shadow: Vec::new(),
            text: c.bg,
        };
        // the kit's well is a step off the page and its on the ink at
        // 70%: the hover tone ringed by the divider, and the text at the
        // same strength; the knob is the page
        let strength = kit.switch.on.fill[0].0[3];
        let on = Color::rgba(
            c.text.0[0],
            c.text.0[1],
            c.text.0[2],
            c.text.0[3] * strength,
        );
        let switch = SwitchTokens {
            off: ButtonSurface {
                fill: [c.bg_hover; 2],
                overlay: NONE,
                shadow: vec![ring(c.divider)],
                text: c.text,
            },
            on: ButtonSurface {
                fill: [on; 2],
                overlay: NONE,
                shadow: Vec::new(),
                text: c.text,
            },
            knob: ButtonSurface {
                fill: [c.bg; 2],
                overlay: NONE,
                shadow: vec![ring(c.divider)],
                text: c.text,
            },
            ..kit.switch
        };
        let edge = legible_at(MARK_CONTRAST, c.divider, c.text_mid, c.bg_hover, c.bg);
        let checkbox = CheckboxTokens {
            off: ButtonSurface {
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            off_hover: ButtonSurface {
                overlay: c.bg_hover,
                shadow: vec![ring(edge)],
                ..switch.off.clone()
            },
            on: full(&switch.on),
            mark: switch.knob.fill[0],
            label: c.text,
            ..kit.checkbox
        };
        let radio = RadioTokens {
            row_hover: c.bg_hover,
            label: c.text,
            ..kit.radio
        };
        Self {
            segmented: SegmentedTokens {
                text: legible(c.text_mid, c.text, track.fill[0], c.bg),
                track,
                thumb,
                text_hover: c.text,
                ..kit.segmented
            },
            button_group,
            switch,
            checkbox,
            radio,
            calendar: CalendarTokens {
                weekday: legible(c.text_mid, c.text, c.surface, c.bg),
                today: legible_at(MARK_CONTRAST, c.border_active, c.text, c.surface, c.bg),
                today_w: theme.control.edge_width,
                chosen: ButtonRecipe {
                    rest: chosen.clone(),
                    hover: chosen.clone(),
                    active: chosen,
                    ..kit.calendar.chosen
                },
                ..kit.calendar
            },
        }
    }
}
