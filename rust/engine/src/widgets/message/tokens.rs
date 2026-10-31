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
use crate::styling::theme::tokens::TypographyScale;
use crate::styling::theme::{Intent, ShadowSpec, Theme};

/// `MessageTokens::hoff_core`, the kit's values with their file:line.
mod kit;
/// The status pieces: the badge, the avatar's online dot.
mod status;

pub use status::{AvatarTokens, BadgePalette, BadgeTokens, TagTokens};

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

/// The badge's tone per kind of notification (the kit's
/// `Notification:33-41`), apart from the row's intent: a like is red and
/// still only informs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KindColors {
    pub comment: Color,
    pub video: Color,
    pub invite: Color,
    pub like: Color,
    pub image: Color,
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
    /// The badge's tone when the row names its kind.
    pub kinds: KindColors,
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

/// The toast's surface: the kit's notifications dropdown
/// (`templates/NotificationsPage`), a paper card with a hairline and a
/// deep drop; inside, the alert's row (`MessageTokens::alert`).
#[derive(Clone, Debug, PartialEq)]
pub struct ToastTokens {
    pub radius: f32,
    pub fill: Color,
    pub border: Color,
    pub border_width: f32,
    pub shadow: ShadowSpec,
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
    pub toast: ToastTokens,
    pub badge: BadgeTokens,
    pub avatar: AvatarTokens,
}

impl MessageTokens {
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
            // (the toast's), the dot in the info accent where it reads on
            // the paper (`unread_mark`)
            alert: AlertTokens {
                ring: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                intents: IntentColors {
                    neutral: colors.accent,
                    informational: colors.info,
                    constructive: colors.success,
                    destructive: colors.danger,
                },
                kinds: KindColors {
                    comment: colors.accent,
                    video: colors.info,
                    invite: colors.success,
                    like: colors.danger,
                    image: colors.warning,
                },
                title: kit.alert.title.recolor(colors.text),
                time: kit.alert.time.recolor(colors.text_mid),
                detail: kit.alert.detail.recolor(colors.text),
                content: kit.alert.content.recolor(colors.text_mid),
                unread: unread_mark(theme),
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
                    pin_unread: unread_mark(theme),
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
            // the page's paper and its divider around the alert's row
            toast: ToastTokens {
                fill: Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                border: colors.divider,
                ..kit.toast
            },
            // the chip on the hover glass with a hairline in the divider,
            // the ink in the text tone; the intents' colors from the palette
            badge: {
                let k = kit.badge;
                let paper = Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]);
                BadgeTokens::new(
                    TagTokens {
                        text: k.tag.text.recolor(colors.text),
                        fill: theme.glass.surface_hover,
                        border: colors.divider,
                        raised_fill: paper,
                        raised_border: colors.divider,
                        ..k.tag
                    },
                    k.count_pad_x,
                    k.dot,
                    k.dot_inset,
                    BadgePalette {
                        intents: IntentColors {
                            neutral: colors.text_mid,
                            informational: colors.info,
                            constructive: colors.success,
                            destructive: colors.danger,
                        },
                        surface: status::over(colors.surface, paper),
                        well: colors.bg_panel,
                        paper,
                        ink: colors.text,
                    },
                )
            },
            // the online dot in the success green, ringed by the paper
            avatar: AvatarTokens::new(
                kit.avatar.online,
                kit.avatar.online_ring,
                colors.success,
                Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]),
                colors.text,
            ),
        }
    }
}

/// `c` composed over the opaque `under`: the same color, opaque.
fn solid(c: Color, under: Color) -> Color {
    let a = c.0[3];
    let mix = |i: usize| c.0[i] * a + under.0[i] * (1.0 - a);
    Color([mix(0), mix(1), mix(2), 1.0])
}

/// What a state mark needs against what it sits on (WCAG 1.4.11).
const MARK_CONTRAST: f32 = 3.0;

/// The unread mark of a theme: the info accent, or, where it does not
/// reach [`MARK_CONTRAST`] on the theme's paper, the first of the accent
/// and the text tone that does; opaque, composed over the paper.
fn unread_mark(theme: &Theme) -> Color {
    let colors = &theme.colors;
    let paper = Color([colors.bg.0[0], colors.bg.0[1], colors.bg.0[2], 1.0]);
    let candidates = [colors.info, colors.accent, colors.text].map(|c| solid(c, paper));
    candidates
        .into_iter()
        .find(|c| contrast(*c, paper) >= MARK_CONTRAST)
        .unwrap_or(candidates[2])
}

/// WCAG's contrast ratio of two opaque colors.
fn contrast(a: Color, b: Color) -> f32 {
    let lum = |c: Color| {
        let lin = |v: f32| {
            if v <= 0.040_45 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.0[0]) + 0.7152 * lin(c.0[1]) + 0.0722 * lin(c.0[2])
    };
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}
