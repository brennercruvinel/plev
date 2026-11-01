//! The notifications dropdown's tokens: the kit's
//! `components/Notifications` (a header with the title and the All and
//! Unread filters over a hairline, then the notifications scrolling), on
//! the toast's surface (`theme.message.toast`, the same card of the kit's
//! `templates/NotificationsPage`), its rows the alert's
//! (`theme.message.alert`).
//!
//! [`NotificationsTokens::hoff_core`] is the kit as written, the file:line
//! of each value beside it; [`NotificationsTokens::derive`] lays the same
//! geometry over another theme's palette.

use super::MessageText;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::hoff_core::SHADES;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NotificationsTokens {
    /// The dropdown's width and the tallest it grows before its list
    /// scrolls.
    pub w: f32,
    pub max_h: f32,
    /// A notification on its own (the page's variations).
    pub row_w: f32,
    pub header_pad_x: f32,
    pub header_pad_y: f32,
    pub title: MessageText,
    /// A filter: its padding, its corner, the gap between two, its run.
    pub tab_pad_x: f32,
    pub tab_pad_y: f32,
    pub tab_radius: f32,
    pub tab_gap: f32,
    pub tab: MessageText,
    /// The chosen filter's wash and ink, and the ink of one hovered.
    pub tab_fill: Color,
    pub tab_active: Color,
    /// The well the page sets the dropdown on (the card's `bg`).
    pub well: Color,
}

impl NotificationsTokens {
    /// The kit as written.
    pub fn hoff_core() -> Self {
        let [_, shade2, shade3, _, _, _, shade7, _, ink] = SHADES;
        Self {
            // templates/NotificationsPage/index.tsx:115 `w-96 h-200`, :19
            // `w-96.5`, :113 `bg-[#F8F7F7]`
            w: 384.0,
            max_h: 800.0,
            row_w: 386.0,
            // components/Notifications/index.tsx:26 `px-5 py-4 border-b`,
            // :27 `text-[0.875rem] leading-[1.25rem] font-semibold`
            header_pad_x: 20.0,
            header_pad_y: 16.0,
            title: MessageText {
                size: 14.0,
                line_height: 20.0,
                weight: 600,
                color: ink,
            },
            // :30 `gap-1`, :33 `text-[0.75rem] leading-[1rem] font-medium
            // px-2 py-1 rounded-[0.625rem] hover:text-[#000]`, :35
            // `bg-[#F1F1F1] text-[#000]`, :36 `text-[#7B7B7B]`. That gray
            // is 4.1:1 on paper; the filter at rest takes shade 7 for AAA,
            // the chosen one keeps its wash
            tab_pad_x: 8.0,
            tab_pad_y: 4.0,
            tab_radius: 10.0,
            tab_gap: 4.0,
            tab: MessageText {
                size: 12.0,
                line_height: 16.0,
                weight: 500,
                color: shade7,
            },
            tab_fill: shade3,
            tab_active: Color::BLACK,
            well: shade2,
        }
    }

    /// The kit's geometry over `theme`'s palette: the title and the chosen
    /// filter in its text tone on its hover wash, a filter at rest in its
    /// muted tone, the well its panel's. Reads only `theme.colors`.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        let kit = Self::hoff_core();
        Self {
            title: MessageText {
                color: c.text,
                ..kit.title
            },
            tab: MessageText {
                color: c.text_mid,
                ..kit.tab
            },
            tab_fill: c.bg_hover,
            tab_active: c.text,
            well: c.bg_panel,
            ..kit
        }
    }
}
