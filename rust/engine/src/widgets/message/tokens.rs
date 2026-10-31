//! Message tokens: the hoff core kit's message rows as numbers the
//! widget reads. Two layouts of the kit:
//!
//! - `thread`, its `Comment/Message`: the row of a comment thread
//!   (avatar 32, name and time on one line, the body under them, a
//!   hairline under the row);
//! - `quick`, its `QuickComment/Comment`: the compact comment of the
//!   quick-comment popup (avatar 36, a semibold name, the body in the
//!   muted tone clamped to three lines, the like chip under it).
//!
//! Text runs are written as the kit writes them, a size, a line height
//! and a weight in px, and become a style through
//! [`TypographyScale::exact`], so the family and the tracking stay the
//! theme's. The like chip is not restated here: it is the labeled
//! button's outline recipe at the `Xs` step (`theme.button`), the same
//! surface the kit gives it.
//!
//! `MessageTokens::hoff_core` is the kit verbatim; every other theme
//! derives the same geometry over its own palette through
//! [`MessageTokens::derive`], so the widget renders under all of them.

use crate::drawing::text::TextStyle;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::hoff_core::{BLUE, SHADES};
use crate::styling::theme::tokens::TypographyScale;

/// One text run of a message: size, line height and weight in px, and
/// its color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageText {
    pub size: f32,
    pub line_height: f32,
    pub weight: u16,
    pub color: Color,
}

impl MessageText {
    /// The one style this run is measured and drawn with.
    pub fn style(&self, typography: &TypographyScale) -> TextStyle {
        typography.exact(self.size, self.line_height, self.weight)
    }

    fn recolor(self, color: Color) -> Self {
        Self { color, ..self }
    }
}

/// One message layout of the kit: the row's box model and its runs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MessageStyle {
    /// Padding on every side of the row.
    pub pad: f32,
    /// Between the avatar and the text column.
    pub gap: f32,
    /// Avatar edge (a disc).
    pub avatar: f32,
    pub name: MessageText,
    pub time: MessageText,
    /// Between the end of the name and the time.
    pub time_gap: f32,
    pub body: MessageText,
    /// Between the name line and the body.
    pub body_gap: f32,
    /// Lines the body is clamped to (css `line-clamp`); `None` keeps
    /// every line.
    pub body_lines: Option<usize>,
    /// Between the body and the like chip.
    pub reaction_gap: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MessageTokens {
    /// The row of a comment thread.
    pub thread: MessageStyle,
    /// The compact comment of the quick-comment popup.
    pub quick: MessageStyle,
    /// The hairline under a row (`border-b`).
    pub divider: Color,
    pub divider_width: f32,
    /// A mention in the body (`[&_span]:text-[#3582FF]`). Read once the
    /// text engine draws a color per run; until then the body is one run.
    pub mention: Color,
}

impl MessageTokens {
    /// The kit as written (the private typescript design system). The
    /// file:line of each value is in the comment beside it.
    pub fn hoff_core() -> Self {
        let [_, _, _, shade4, _, shade6, _, _, ink] = SHADES;
        Self {
            // components/Comment/Message/index.tsx
            thread: MessageStyle {
                // :13 `flex gap-4 p-4`
                pad: 16.0,
                gap: 16.0,
                // :16 `size-8 rounded-full`
                avatar: 32.0,
                // :24 `text-[0.75rem] leading-[1rem] font-medium`, the
                // ink of app/layout.tsx:108
                name: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                // :26 `ml-2 text-[0.6875rem] text-[#7B7B7B]`, leading
                // and weight inherited from :24
                time: MessageText {
                    size: 11.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade6,
                },
                time_gap: 8.0,
                // :30 `mt-1 text-[0.8125rem] leading-[1.22rem]`, in ink
                body: MessageText {
                    size: 13.0,
                    line_height: 19.52,
                    weight: 400,
                    color: ink,
                },
                body_gap: 4.0,
                body_lines: None,
                // the thread row has no like chip
                reaction_gap: 0.0,
            },
            // components/QuickComment/Comment/index.tsx
            quick: MessageStyle {
                // :29 `p-3`, :30 `gap-3`
                pad: 12.0,
                gap: 12.0,
                // :33 `size-9 rounded-full`
                avatar: 36.0,
                // :42 `text-[0.8125rem] leading-[1rem] font-semibold
                // text-[#121212]`
                name: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 600,
                    color: ink,
                },
                // :45 `ml-1 text-[0.75rem] leading-[1rem]`. The kit
                // writes `text-[#7B7B7B]/50`; that is 1.9:1 on paper, so
                // the time keeps the full muted tone (4.13:1) until the
                // contrast decision of task-104.
                time: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 400,
                    color: shade6,
                },
                time_gap: 4.0,
                // :49 `mt-1 line-clamp-3 text-[0.8125rem]
                // leading-[1.25rem] text-[#7B7B7B]`
                body: MessageText {
                    size: 13.0,
                    line_height: 20.0,
                    weight: 400,
                    color: shade6,
                },
                body_gap: 4.0,
                body_lines: Some(3),
                // :57 `mt-2` on the like chip
                reaction_gap: 8.0,
            },
            // components/Comment/Message/index.tsx:13 `border-b
            // border-[#ECECEC]`
            divider: shade4,
            divider_width: 1.0,
            // components/Comment/Message/index.tsx:30
            mention: BLUE,
        }
    }

    /// The kit's geometry over another theme's palette: names and thread
    /// bodies in the text tone, times and quick bodies in the muted one,
    /// the hairline in the divider, mentions in the info accent. Reads
    /// only `theme.colors`.
    pub fn derive(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let kit = Self::hoff_core();
        let restyle = |s: MessageStyle, body: Color| MessageStyle {
            name: s.name.recolor(colors.text),
            time: s.time.recolor(colors.text_mid),
            body: s.body.recolor(body),
            ..s
        };
        Self {
            thread: restyle(kit.thread, colors.text),
            quick: restyle(kit.quick, colors.text_mid),
            divider: colors.divider,
            divider_width: kit.divider_width,
            mention: colors.info,
        }
    }
}
