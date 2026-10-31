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
//! Beside them the front's other pieces: the bubble (`CommentPin`), the
//! scroller (`Comment`), the inline alert (`Notification`, by intent).
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
use crate::styling::theme::hoff_core::{BLUE, RED, SHADES};
use crate::styling::theme::tokens::TypographyScale;
use crate::styling::theme::{Intent, ShadowSpec, Theme};

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
    /// Between the name and its seal (the kit's `✪`, drawn as an icon one
    /// name line tall).
    pub seal_gap: f32,
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

/// The message scroller: the kit's comment popup (`components/Comment`),
/// its messages in a rounded surface that scrolls past a height.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollerTokens {
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    pub radius: f32,
    /// The css `box-shadow` list, first on top (drops and the key-light).
    pub shadow: Vec<ShadowSpec>,
    /// The tallest the scroller grows before its messages scroll.
    pub max_h: f32,
    /// What an empty scroller says, in the muted body tone.
    pub empty: MessageText,
}

/// The fill of each intent's mark, the badge an alert carries.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntentColors {
    pub neutral: Color,
    pub informational: Color,
    pub constructive: Color,
    pub destructive: Color,
}

impl IntentColors {
    pub fn of(&self, intent: Intent) -> Color {
        match intent {
            Intent::Neutral => self.neutral,
            Intent::Informational => self.informational,
            Intent::Constructive => self.constructive,
            Intent::Destructive => self.destructive,
        }
    }
}

/// The inline alert: the kit's notification row
/// (`components/Notification`), a lead disc with the intent's badge on
/// its corner, the title and the time on one line, the detail under them
/// on one line, the content clamped, and the answers in a row.
#[derive(Clone, Debug, PartialEq)]
pub struct AlertTokens {
    /// Padding at the top, the bottom and the start of the row.
    pub pad: f32,
    /// Padding at the end, the dot's side.
    pub pad_end: f32,
    /// The lead disc's edge (the avatar, or the intent's mark alone).
    pub lead: f32,
    /// Between the lead and the text column.
    pub gap: f32,
    /// The badge's edge, its ring included.
    pub badge: f32,
    /// The ring that cuts the badge out of the lead.
    pub badge_ring: f32,
    /// How far the badge passes the lead's bottom end corner.
    pub badge_out: f32,
    pub badge_icon: f32,
    /// The ring's color: the paper the row sits on, opaque.
    pub ring: Color,
    pub intents: IntentColors,
    pub title: MessageText,
    /// How far the title line rises into the column's top.
    pub title_lift: f32,
    pub time: MessageText,
    /// Between the title and the time.
    pub time_gap: f32,
    /// Between the title line and the detail.
    pub title_gap: f32,
    /// One line, ellipsized.
    pub detail: MessageText,
    pub content: MessageText,
    /// Between the detail and the content.
    pub content_gap: f32,
    pub content_lines: usize,
    /// Between the text and the answers, and between two answers.
    pub actions_gap: f32,
    pub action_gap: f32,
    /// The unread dot: its edge, its top in the title line, and the room
    /// the title line keeps for it at the end.
    pub dot: f32,
    pub dot_top: f32,
    pub dot_room: f32,
    pub unread: Color,
}

/// The attachments of a message: the kit's image preview
/// (`NewComment/ImagePreview`), square tiles with a remove button on their
/// top end corner, and, for what is not a picture, a file tile in the
/// language of its add-files menu (`PromptInput/AddFiles`). The add button
/// is the bordered icon button at its 40 step.
#[derive(Clone, Debug, PartialEq)]
pub struct AttachmentTokens {
    /// A picture tile's edge, and every tile's height.
    pub tile: f32,
    pub radius: f32,
    /// Between two tiles, across and down.
    pub gap: f32,
    /// A picture still loading, or none: the tile's wash.
    pub placeholder: Color,
    /// The remove button: its edge, how far it passes the tile's top end
    /// corner, its icon, its fill top to bottom and its icon's color.
    pub remove: f32,
    pub remove_out: f32,
    pub remove_icon: f32,
    pub remove_fill: [Color; 2],
    pub remove_ink: Color,
    /// A file tile: its padding across, its icon and the box it sits in,
    /// the gap to the text, its border and fill.
    pub file_pad: f32,
    pub file_icon: f32,
    pub file_icon_box: f32,
    pub file_gap: f32,
    pub file_icon_color: Color,
    pub file_border: Color,
    pub file_border_width: f32,
    pub file_fill: Color,
    pub name: MessageText,
    pub detail: MessageText,
    /// The widest a file tile grows, as a fraction of the tray.
    pub file_max_fraction: f32,
}

/// One shape of the comment cursor: its box, its corners (the bottom
/// start one, the tail, smaller), its drop, its fill top to bottom and the
/// mark inside.
#[derive(Clone, Debug, PartialEq)]
pub struct CursorShape {
    pub w: f32,
    pub h: f32,
    pub radius: f32,
    pub tail_radius: f32,
    pub shadow: ShadowSpec,
    pub fill: [Color; 2],
    /// The plus's edge, and its color.
    pub mark: f32,
    pub ink: Color,
}

/// The markers on a canvas: the kit's comment pin (`components/CommentPin`,
/// the people of a thread on a tailed pill, read or unread) and its
/// comment cursor (`components/CommentCursor`: point, pinned, uploading,
/// typing).
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerTokens {
    /// The pin: around the avatars, its border read and unread, its fill
    /// and drop; each avatar's edge, the paper ring that cuts it from the
    /// one before, and how far it slides under that one.
    pub pin_pad: f32,
    pub pin_border_width: f32,
    pub pin_fill: Color,
    pub pin_read: Color,
    pub pin_unread: Color,
    pub pin_shadow: ShadowSpec,
    pub avatar: f32,
    pub avatar_ring: f32,
    pub avatar_overlap: f32,
    /// The cursor at a point: bordered paper.
    pub point: CursorShape,
    pub point_border_width: f32,
    pub point_border: Color,
    /// Pinned: the dark one.
    pub pinned: CursorShape,
    /// Uploading and typing: the blue one.
    pub active: CursorShape,
    /// The typing dots: edge and gap.
    pub dot: f32,
    pub dot_gap: f32,
    /// The plus's bar, as a share of its edge (`1.5` of the kit's `8`).
    pub mark_bar: f32,
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
    pub scroller: ScrollerTokens,
    pub alert: AlertTokens,
    pub attachment: AttachmentTokens,
    pub marker: MarkerTokens,
}

impl MessageTokens {
    /// The kit as written (the private typescript design system). The
    /// file:line of each value is in the comment beside it.
    pub fn hoff_core() -> Self {
        let [paper, _, shade3, shade4, shade5, shade6, shade7, _, ink] = SHADES;
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
                // :66 of templates/CommentPage `Luna Starfield ✪`: one
                // space at the name's 12 px, as `ml-1`
                seal_gap: 4.0,
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
                // QuickComment/Comment/index.tsx:74 `Tran Mau Tri Tam ✪`
                seal_gap: 4.0,
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
            // components/Comment/index.tsx:21 `bg-[#FCFCFC] border
            // border-[#ECECEC] rounded-2xl` and its shadow list
            scroller: ScrollerTokens {
                fill: SHADES[0],
                border: shade4,
                border_width: 1.0,
                radius: 16.0,
                shadow: vec![
                    ShadowSpec::new([0.0, 18.0], 24.0, -20.0, Color::rgba(0.0, 0.0, 0.0, 0.13)),
                    ShadowSpec::inset([0.0, 2.0], 0.0, 0.0, Color::rgba(1.0, 1.0, 1.0, 1.0)),
                    ShadowSpec::new([0.0, 8.0], 16.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.08)),
                ],
                // the kit's popup has no cap; this is the select panel's
                // (`size.dropdown_max_h`), the tallest a panel grows
                // before it scrolls
                max_h: 320.0,
                // the quick comment's muted body (QuickComment/Comment:49)
                empty: MessageText {
                    size: 13.0,
                    line_height: 20.0,
                    weight: 400,
                    color: shade6,
                },
            },
            // components/Notification/index.tsx
            alert: AlertTokens {
                // :18 `p-5 pr-4`
                pad: 20.0,
                pad_end: 16.0,
                // :25 `size-12 rounded-full`
                lead: 48.0,
                // :63 `pl-4`
                gap: 16.0,
                // :32 `-right-1 -bottom-1 size-5.5 border-2
                // border-[#FCFCFC]`
                badge: 22.0,
                badge_ring: 2.0,
                badge_out: 4.0,
                // :45 `size-3`
                badge_icon: 12.0,
                ring: paper,
                // :33-41, the badge per kind of notification, read as
                // intents: the comment's violet the neutral one, the
                // video's blue the informational, the invite's green the
                // constructive, the like's red the destructive
                intents: IntentColors {
                    neutral: Color::hex(0x8651F8),
                    informational: Color::hex(0x1BA4F3),
                    constructive: Color::hex(0x55B93E),
                    destructive: RED,
                },
                // :65 `text-[0.8125rem] leading-[1rem] font-semibold`
                title: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 600,
                    color: ink,
                },
                // :64 `-mt-0.5 mb-1.5`
                title_lift: 2.0,
                title_gap: 6.0,
                // :68 `ml-1 text-[0.75rem] leading-[1rem] font-medium
                // text-[#7B7B7B]/50`. That is 1.9:1 on paper; the time
                // takes shade 7, the darkest gray under ink that still
                // reads as secondary, and passes AAA (task-104 contrast)
                time: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade7,
                },
                time_gap: 4.0,
                // :75 `truncate text-[0.8125rem] leading-[1rem]`
                detail: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 400,
                    color: ink,
                },
                // :99 `mt-2 line-clamp-2 text-[0.8125rem]
                // leading-[1.2188rem] text-[#7B7B7B]/80`; shade 7 for
                // AAA, as the time
                content: MessageText {
                    size: 13.0,
                    line_height: 19.5,
                    weight: 400,
                    color: shade7,
                },
                content_gap: 8.0,
                content_lines: 2,
                // :104 `mt-4 gap-2`
                actions_gap: 16.0,
                action_gap: 8.0,
                // :72 `top-2.5 size-2`, in the title line's `pr-4`. The
                // kit's green dot (#55b93e) is 2.4:1 on paper, under the
                // 3:1 a state mark needs; the dot takes the kit's unread
                // accent, the comment pin's blue
                dot: 8.0,
                dot_top: 10.0,
                dot_room: 16.0,
                unread: BLUE,
            },
            // components/NewComment/ImagePreview/index.tsx
            attachment: AttachmentTokens {
                // :13 `size-15 rounded-xl`, :9 `gap-2`
                tile: 60.0,
                radius: 12.0,
                gap: 8.0,
                // the add-files row's hover wash
                // (PromptInput/AddFiles/index.tsx:53 `hover:bg-[#f1f1f1]`)
                placeholder: shade3,
                // :20 `-top-1.5 -right-1.5 size-5 bg-gradient-to-b
                // from-[#323232] to-[#121212]`, :22 `size-4 fill-white`
                remove: 20.0,
                remove_out: 6.0,
                remove_icon: 16.0,
                remove_fill: [shade7, ink],
                remove_ink: Color::WHITE,
                // PromptInput/AddFiles/index.tsx:53 `gap-2.5 px-2`,
                // :58 `size-6`, :60 `size-5 fill-[#7b7b7b]`; the tile
                // pads by the image preview's `p-3` cousin, `px-3`
                file_pad: 12.0,
                file_icon: 20.0,
                file_icon_box: 24.0,
                file_gap: 10.0,
                file_icon_color: shade6,
                // Upload/index.tsx:21 `bg-[#FCFCFC] border
                // border-[#E2E2E2]`
                file_border: shade5,
                file_border_width: 1.0,
                file_fill: paper,
                // :53 `text-[0.8125rem] font-medium text-[#121212]`, at
                // the kit's 16 line
                name: MessageText {
                    size: 13.0,
                    line_height: 16.0,
                    weight: 500,
                    color: ink,
                },
                // the notification's time run (Notification:68), in
                // shade 7 for AAA as there
                detail: MessageText {
                    size: 12.0,
                    line_height: 16.0,
                    weight: 500,
                    color: shade7,
                },
                // the kit draws no file tile: it grows with its name up
                // to the tray's width, where the name is ellipsized
                file_max_fraction: 1.0,
            },
            marker: MarkerTokens {
                // components/CommentPin/index.tsx:10 `p-0.25
                // border-[1.5px] bg-[#FCFCFC]
                // shadow-[0px_2px_2px_0px_rgba(0,0,0,0.15)]`, :11 read
                // `#ECECEC`, unread `#3582FF`
                pin_pad: 1.0,
                pin_border_width: 1.5,
                pin_fill: paper,
                pin_read: shade4,
                pin_unread: BLUE,
                pin_shadow: ShadowSpec::new([0.0, 2.0], 2.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                // :16 `-ml-2.5 border-[1.5px] border-[#FCFCFC]`, :20
                // `size-6`
                avatar: 24.0,
                avatar_ring: 1.5,
                avatar_overlap: 10.0,
                // components/CommentCursor/index.tsx:12 `w-5 h-4.5
                // border-[1.5px] border-[#121212] rounded-md
                // rounded-bl-[1px] bg-[#FCFCFC]
                // shadow-[0px_2px_4px_rgba(0,0,0,0.15)]`, :28 `size-1.75
                // fill-[#7B7B7B]/50`. That plus is 1.9:1 on paper; it
                // takes the full muted gray, 4.1:1
                point: CursorShape {
                    w: 20.0,
                    h: 18.0,
                    radius: 6.0,
                    tail_radius: 1.0,
                    shadow: ShadowSpec::new([0.0, 2.0], 4.0, 0.0, Color::rgba(0.0, 0.0, 0.0, 0.15)),
                    fill: [paper, paper],
                    mark: 7.0,
                    ink: shade6,
                },
                point_border_width: 1.5,
                point_border: ink,
                // :13 `w-7.5 h-7 rounded-[10px] rounded-bl-[2px]
                // shadow-[0px_4px_4px_rgba(18,18,18,0.15)]`, :14
                // `from-[#323232] to-[#121212]`, :29 `size-2.25
                // fill-[#7B7B7B]`
                pinned: CursorShape {
                    w: 30.0,
                    h: 28.0,
                    radius: 10.0,
                    tail_radius: 2.0,
                    shadow: ShadowSpec::new(
                        [0.0, 4.0],
                        4.0,
                        0.0,
                        Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.15),
                    ),
                    fill: [shade7, ink],
                    mark: 9.0,
                    ink: shade6,
                },
                // :16 `from-[#34A7FF] to-[#0D7DFD]`, :30 `fill-[#FFFFFF]`.
                // White is 2.6:1 on #34A7FF; the top takes the kit's
                // accent blue, 3.6:1, the bottom stays
                active: CursorShape {
                    w: 30.0,
                    h: 28.0,
                    radius: 10.0,
                    tail_radius: 2.0,
                    shadow: ShadowSpec::new(
                        [0.0, 4.0],
                        4.0,
                        0.0,
                        Color::rgba(18.0 / 255.0, 18.0 / 255.0, 18.0 / 255.0, 0.15),
                    ),
                    fill: [BLUE, Color::hex(0x0D7DFD)],
                    mark: 9.0,
                    ink: Color::WHITE,
                },
                // :21 `gap-0.5 [&_span]:size-0.5`
                dot: 2.0,
                dot_gap: 2.0,
                // :36, the plus path: bars 1.5 wide on an 8 box
                mark_bar: 1.5 / 8.0,
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
            scroller: ScrollerTokens {
                fill: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                border: colors.divider,
                // the key-light is the kit's paper; elsewhere the drops
                shadow: kit
                    .scroller
                    .shadow
                    .iter()
                    .filter(|s| !s.inset)
                    .cloned()
                    .collect(),
                max_h: theme.size.dropdown_max_h,
                empty: kit.scroller.empty.recolor(colors.text_mid),
                ..kit.scroller
            },
            // titles and details in the text tone, the time and the
            // content in the muted one, the badges in the intents' colors
            // (the toast's), the dot in the info accent
            alert: AlertTokens {
                ring: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                intents: IntentColors {
                    neutral: colors.accent,
                    informational: colors.info,
                    constructive: colors.success,
                    destructive: colors.danger,
                },
                title: kit.alert.title.recolor(colors.text),
                time: kit.alert.time.recolor(colors.text_mid),
                detail: kit.alert.detail.recolor(colors.text),
                content: kit.alert.content.recolor(colors.text_mid),
                unread: colors.info,
                ..kit.alert
            },
            // the wash and the border in the hover and the divider tones,
            // the remove button in the text tone over the page
            attachment: AttachmentTokens {
                placeholder: colors.bg_hover,
                remove_fill: [colors.text, colors.text],
                remove_ink: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                file_icon_color: colors.text_mid,
                file_border: colors.divider,
                file_fill: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                name: kit.attachment.name.recolor(colors.text),
                detail: kit.attachment.detail.recolor(colors.text_mid),
                ..kit.attachment
            },
            // the pin on the page's paper, read in the divider, unread in
            // the info accent; the cursors bordered in the text tone, the
            // pinned one in it, the active ones in the info accent
            // (each composed over the paper: a shape and the patch over its
            // tail corner overlap, and a translucent fill would show it)
            marker: {
                let paper = Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]);
                let solid = |c: Color| solid(c, paper);
                let (text, info) = (solid(colors.text), solid(colors.info));
                let k = kit.marker;
                MarkerTokens {
                    pin_fill: paper,
                    pin_read: solid(colors.divider),
                    pin_unread: info,
                    point: CursorShape {
                        fill: [paper, paper],
                        ink: colors.text_mid,
                        ..k.point
                    },
                    point_border: text,
                    pinned: CursorShape {
                        fill: [text, text],
                        ink: paper,
                        ..k.pinned
                    },
                    active: CursorShape {
                        fill: [info, info],
                        ink: Color(crate::widgets::widget::contrast_text(info.0)),
                        ..k.active
                    },
                    ..k
                }
            },
        }
    }
}

/// `c` composed over the opaque `under`: the same color, opaque.
fn solid(c: Color, under: Color) -> Color {
    let a = c.0[3];
    let mix = |i: usize| c.0[i] * a + under.0[i] * (1.0 - a);
    Color([mix(0), mix(1), mix(2), 1.0])
}
