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
use crate::styling::theme::hoff_core::{BLUE, SHADES};
use crate::styling::theme::tokens::TypographyScale;
use crate::styling::theme::{ShadowSpec, Theme};

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

/// The chat bubble: the kit's comment pin (`components/CommentPin`), a
/// pill with its bottom-start corner square, holding text instead of
/// avatars.
#[derive(Clone, Debug, PartialEq)]
pub struct BubbleTokens {
    /// Inside the border, around the text.
    pub pad_x: f32,
    pub pad_y: f32,
    pub border_width: f32,
    /// Opaque, so the square tail corner composes without a seam.
    pub fill: Color,
    /// The border of a read bubble.
    pub border: Color,
    /// The border of an unread one (the kit's accent).
    pub unread: Color,
    pub shadow: ShadowSpec,
    pub text: MessageText,
    /// The widest a bubble grows, as a fraction of the width it is given.
    pub max_fraction: f32,
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
    pub bubble: BubbleTokens,
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
            // components/CommentPin/index.tsx
            bubble: BubbleTokens {
                // the pin pads avatars by `p-0.25`; around text the
                // bubble takes the quick comment's `p-3`
                // (QuickComment/Comment/index.tsx:29) across and half
                // of it down
                pad_x: 12.0,
                pad_y: 6.0,
                // :10 `bg-[#FCFCFC] border-[1.5px]`
                border_width: 1.5,
                fill: SHADES[0],
                // :11 read `border-[#ECECEC]`, unread `border-[#3582FF]`
                border: shade4,
                unread: BLUE,
                // :10 `shadow-[0px_2px_2px_0px_rgba(0,0,0,0.15)]`
                shadow: ShadowSpec::new([0.0, 2.0], 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                // the thread's body run (Comment/Message/index.tsx:30)
                text: MessageText {
                    size: 13.0,
                    line_height: 19.52,
                    weight: 400,
                    color: ink,
                },
                // the kit draws no text bubble; four fifths of the row
                // leaves the other side's bubbles their lane
                max_fraction: 0.8,
            },
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
            bubble: BubbleTokens {
                fill: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                border: colors.divider,
                unread: colors.info,
                text: kit.bubble.text.recolor(colors.text),
                ..kit.bubble
            },
        }
    }
}
