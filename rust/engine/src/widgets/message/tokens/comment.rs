//! The comment popups' tokens: the kit's `components/Comment` (a thread
//! beside its pin, with the close disc and the reply of
//! `Comment/NewMessage`), `components/NewComment` (the cursor and the box
//! a new comment is written in) and `components/QuickComment` (the
//! object's card with its comments).
//!
//! The thread's surface is the message scroller's (`theme.message.
//! scroller`, the same popup of the kit); the rows are the message's
//! (`thread`, `quick`), the pin and the cursor the marker's.
//!
//! [`CommentTokens::hoff_core`] is the kit as written, the file:line of
//! each value beside it; [`CommentTokens::derive`] lays the same geometry
//! over another theme's palette.

use super::MessageText;
use super::prompt::{raised_shadow, shadow_2xl};
use crate::styling::color::Color;
use crate::styling::theme::TypeRole;
use crate::styling::theme::hoff_core::SHADES;
use crate::styling::theme::{ShadowSpec, Theme};

/// A square icon button the kit draws bare: nothing at rest, a wash on
/// hover.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ToolTokens {
    pub size: f32,
    pub radius: f32,
    pub icon: f32,
    pub color: Color,
    pub hover: Color,
    /// Between two of them.
    pub gap: f32,
}

/// The thread's close button: a dark disc on its top end corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CloseTokens {
    pub size: f32,
    /// From the popup's top and end edges.
    pub inset: f32,
    pub icon: f32,
    /// Top to bottom.
    pub fill: [Color; 2],
    pub ink: Color,
}

/// The reply under a thread (`Comment/NewMessage`): the replier's avatar,
/// then the line and the submit button in a well.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReplyTokens {
    pub pad: f32,
    pub avatar: f32,
    /// How far the avatar sits under the row's top.
    pub avatar_top: f32,
    pub gap: f32,
    pub well: Color,
    pub well_radius: f32,
    pub well_pad: f32,
    /// Around the line inside the well (the textarea's `p-2`).
    pub line_pad: f32,
    pub text: MessageText,
    pub placeholder: Color,
}

/// The box a new comment is written in, beside its cursor.
#[derive(Clone, Debug, PartialEq)]
pub struct NewCommentTokens {
    /// Between the cursor and the box.
    pub gap: f32,
    /// How far the cursor sits under the box's top.
    pub cursor_top: f32,
    pub w: f32,
    pub radius: f32,
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    pub shadow: Vec<ShadowSpec>,
    pub pad: f32,
    /// Around the line (the textarea's `p-2`).
    pub line_pad: f32,
    pub text: MessageText,
    pub placeholder: Color,
    /// The attachments' inset across and under them.
    pub tray_pad_x: f32,
    pub tray_pad_bottom: f32,
    /// The tool row: the space over it, its padding and its hairline.
    pub tools_top: f32,
    pub tools_pad: f32,
    pub tool: ToolTokens,
}

/// The object's card with its comments (`components/QuickComment`).
#[derive(Clone, Debug, PartialEq)]
pub struct QuickCommentTokens {
    pub w: f32,
    pub pad: f32,
    pub radius: f32,
    pub fill: Color,
    pub shadow: Vec<ShadowSpec>,
    /// The header: its padding, the gap after the close button, the
    /// space under it.
    pub header_pad: f32,
    pub header_gap: f32,
    pub header_bottom: f32,
    pub title: MessageText,
    pub subtitle: MessageText,
    /// The object's preview: its well, its hairline, its corner, the
    /// picture's edge in it (a thumbnail stands as a rectangle of this
    /// color), the space under it.
    pub preview_fill: Color,
    pub preview_border: Color,
    pub preview_radius: f32,
    pub picture: f32,
    pub picture_fill: Color,
    pub preview_bottom: f32,
    /// The comments: the space over them and their padding.
    pub comments_top: f32,
    pub comments_pad: f32,
    /// The reply: the space over it, the replier's name, the line.
    pub reply_top: f32,
    pub name: MessageText,
    pub text: MessageText,
    pub placeholder: Color,
    /// Between the name and the line.
    pub line_gap: f32,
    /// The space over the tool row.
    pub tools_top: f32,
    pub tool: ToolTokens,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommentTokens {
    /// The thread's popup: its width, and the gap after its pin.
    pub popup_w: f32,
    pub pin_gap: f32,
    pub close: CloseTokens,
    pub reply: ReplyTokens,
    pub new: NewCommentTokens,
    pub quick: QuickCommentTokens,
}

impl CommentTokens {
    /// The kit as written.
    pub fn hoff_core() -> Self {
        let [paper, _, shade3, shade4, shade5, shade6, shade7, _, ink] = SHADES;
        let comment = MessageText {
            run: TypeRole::BodyLg.run(),
            color: ink,
        };
        Self {
            // components/Comment/index.tsx:16 `gap-3.5`, :21 `w-70`
            popup_w: 280.0,
            pin_gap: 14.0,
            // :22 `top-2 right-2 size-5 from-[#323232] to-[#121212]
            // rounded-full`, :24 `size-3 fill-white`
            close: CloseTokens {
                size: 20.0,
                inset: 8.0,
                icon: 12.0,
                fill: [shade7, ink],
                ink: Color::WHITE,
            },
            // Comment/NewMessage/index.tsx:10 `p-4`, :11 `mt-1.5`, :13
            // `size-8`, :20 `pl-4`, :21 `p-1.5 bg-[#F1F1F1] rounded-xl`,
            // :23 `h-8 p-2 text-[0.8125rem] leading-[1rem]`, the kit's
            // `placeholder:text-secondary` (#7B7B7B)
            reply: ReplyTokens {
                pad: 16.0,
                avatar: 32.0,
                avatar_top: 6.0,
                gap: 16.0,
                well: shade3,
                well_radius: 12.0,
                well_pad: 6.0,
                line_pad: 8.0,
                text: comment,
                placeholder: shade6,
            },
            // components/NewComment/index.tsx:12 `gap-4`, :14 `mt-2.5`,
            // :18 `w-70 rounded-xl bg-[#FCFCFC] border border-[#ECECEC]`
            // and the raised shadow list, :19 `p-2`, :21 `h-8 p-2
            // text-[0.8125rem] leading-[1rem]`; ImagePreview/index.tsx:9
            // `px-4 pb-2`; :36 `mt-2 p-2 border-t`, :37 `gap-1`, :38 `size-8
            // rounded-[0.625rem] hover:bg-[#F1F1F1]`, :40 `size-5
            // fill-[#7B7B7B]`
            new: NewCommentTokens {
                gap: 16.0,
                cursor_top: 10.0,
                w: 280.0,
                radius: 12.0,
                fill: paper,
                border: shade4,
                border_width: 1.0,
                shadow: raised_shadow(),
                pad: 8.0,
                line_pad: 8.0,
                text: comment,
                placeholder: shade6,
                tray_pad_x: 16.0,
                tray_pad_bottom: 8.0,
                tools_top: 8.0,
                tools_pad: 8.0,
                tool: ToolTokens {
                    size: 32.0,
                    radius: 10.0,
                    icon: 20.0,
                    color: shade6,
                    hover: shade3,
                    gap: 4.0,
                },
            },
            // components/QuickComment/index.tsx:6 `w-92 p-1 bg-[#FCFCFC]
            // rounded-[1.25rem] shadow-2xl`; :7 `gap-3 mb-2 p-3`; :19
            // `text-[0.875rem] leading-[1.25rem] font-semibold`; :22
            // `text-[0.6875rem] leading-[1rem] font-medium
            // text-[#7B7B7B]` (4.1:1 on paper: shade 7 for AAA); :27 `mb-2
            // bg-[#F1F1F1] border border-[#E2E2E2] rounded-2xl`, :30
            // `width={264}`
            quick: QuickCommentTokens {
                w: 368.0,
                pad: 4.0,
                radius: 20.0,
                fill: paper,
                shadow: shadow_2xl(),
                header_pad: 12.0,
                header_gap: 12.0,
                header_bottom: 8.0,
                title: MessageText {
                    run: TypeRole::Heading.strong(),
                    color: ink,
                },
                subtitle: MessageText {
                    run: TypeRole::Caption.run(),
                    color: shade7,
                },
                preview_fill: shade3,
                preview_border: shade5,
                preview_radius: 16.0,
                picture: 264.0,
                // the 3D object's thumbnail: a rectangle in the well's
                // hairline gray, a shade over the well
                picture_fill: shade4,
                preview_bottom: 8.0,
                // QuickComment/Comment/index.tsx:29 `mt-2 p-3`; :62
                // `mt-4`; :73 `text-[0.8125rem] leading-[1rem]
                // font-semibold`; :77 `mt-1 text-[0.8125rem]
                // leading-[1.25rem] placeholder:text-[#7B7B7B]/80` (2.9:1:
                // a hint, the full muted gray as the prompt's); :86 `mt-4`,
                // :87 `gap-1`, :90 `size-9 rounded-lg hover:bg-[#F1F1F1]`,
                // :94 `size-5 fill-[#7B7B7B]/70` (2.6:1, under the 3:1 an
                // icon needs: the full muted gray)
                comments_top: 8.0,
                comments_pad: 12.0,
                reply_top: 16.0,
                name: MessageText {
                    run: TypeRole::BodyLg.strong(),
                    color: ink,
                },
                text: MessageText {
                    run: TypeRole::Paragraph.run(),
                    color: ink,
                },
                placeholder: shade6,
                line_gap: 4.0,
                tools_top: 16.0,
                tool: ToolTokens {
                    size: 36.0,
                    radius: 8.0,
                    icon: 20.0,
                    color: shade6,
                    hover: shade3,
                    gap: 4.0,
                },
            },
        }
    }

    /// The kit's geometry over `theme`'s palette: the surfaces on its
    /// paper with its divider, the text in its text tone, the hints and
    /// the icons in its muted one, the washes in its hover and panel ones,
    /// the close disc in its text tone with the paper as its ink. Reads
    /// only `theme.colors`.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let kit = Self::hoff_core();
        let paper = Color([c.bg.0[0], c.bg.0[1], c.bg.0[2], 1.0]);
        let ink = |run: MessageText| MessageText {
            color: c.text,
            ..run
        };
        let drops = |list: &[ShadowSpec]| list.iter().filter(|s| !s.inset).copied().collect();
        let tool = |t: ToolTokens| ToolTokens {
            color: c.text_mid,
            hover: c.bg_hover,
            ..t
        };
        Self {
            close: CloseTokens {
                fill: [c.text, c.text],
                ink: paper,
                ..kit.close
            },
            reply: ReplyTokens {
                well: c.bg_hover,
                text: ink(kit.reply.text),
                placeholder: c.text_mid,
                ..kit.reply
            },
            new: NewCommentTokens {
                fill: paper,
                border: c.divider,
                shadow: drops(&kit.new.shadow),
                text: ink(kit.new.text),
                placeholder: c.text_mid,
                tool: tool(kit.new.tool),
                ..kit.new.clone()
            },
            quick: QuickCommentTokens {
                fill: paper,
                title: ink(kit.quick.title),
                subtitle: MessageText {
                    color: c.text_mid,
                    ..kit.quick.subtitle
                },
                preview_fill: c.bg_hover,
                preview_border: c.divider,
                picture_fill: c.bg_panel,
                name: ink(kit.quick.name),
                text: ink(kit.quick.text),
                placeholder: c.text_mid,
                tool: tool(kit.quick.tool),
                ..kit.quick.clone()
            },
            ..kit
        }
    }
}
