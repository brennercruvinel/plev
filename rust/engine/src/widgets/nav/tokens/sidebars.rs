//! The SidebarPage's sidebars (`templates/SidebarPage`): the shell the
//! four share and the parts of each. `SidebarsRecipe::hoff_core` is the
//! kit as its class lists write it; [`SidebarsRecipe::derive`] reads the
//! same roles from any other theme.
//!
//! Text that fails WCAG AAA in the kit's own gray (`#7B7B7B`, and its
//! 70% and 50% washes) takes shade 7 (`#323232`), as the page head and
//! the table do (task-104 front D); a mark keeps the kit's gray, which
//! reads at 3:1.

use crate::styling::color::Color;
use crate::styling::theme::hoff_core::SHADES;
use crate::styling::theme::{ControlSize, IconSize, Theme};

use super::NavText;

/// The shell, the head with its logo, the captions and the new-folder
/// field of the home and guideline sidebars.
#[derive(Clone, Debug, PartialEq)]
pub struct SidebarsRecipe {
    /// `w-60`.
    pub width: f32,
    /// `min-h-219`; 0 where a theme lets the sidebar be as tall as its rows.
    pub min_h: f32,
    /// `bg-[#FCFCFC]`, the rim `border-[#ECECEC]`.
    pub fill: Color,
    pub edge: Color,
    pub edge_width: f32,
    /// The canvas sidebars' `rounded-[1.25rem]`.
    pub canvas_radius: f32,
    /// The head's `p-6` and the logo's 145 x 32.
    pub head_pad: f32,
    pub logo_w: f32,
    pub logo_h: f32,
    /// The logo's mark and wordmark until the Logo (front F) is here.
    pub logo_mark: Color,
    pub logo_text: NavText,
    /// The rows' `px-5 pb-5`.
    pub body_pad: f32,
    /// A group's caption: `p-2.5`, `mb-0.5`, its run and tone.
    pub caption: NavText,
    pub caption_pad: f32,
    pub caption_gap: f32,
    pub caption_color: Color,
    /// `mb-3` after a group.
    pub group_gap: f32,
    /// `gap-0.5` / `mt-0.5` between rows.
    pub row_gap: f32,
    /// The new folder's field, open: `h-10 pl-12 rounded-xl`, its fill
    /// and rim, the folder's mark, the placeholder.
    pub field_h: f32,
    pub field_radius: f32,
    pub field_text_x: f32,
    pub field_icon_x: f32,
    pub field_fill: Color,
    pub field_edge: Color,
    pub field_mark: Color,
    pub field_placeholder: Color,
}

impl SidebarsRecipe {
    pub fn hoff_core() -> Self {
        let [paper, _, wash, line, _, _, shade7, _, ink] = SHADES;
        Self {
            width: 240.0,
            min_h: 876.0,
            fill: paper,
            edge: line,
            edge_width: 1.0,
            canvas_radius: 20.0,
            head_pad: 24.0,
            logo_w: 145.0,
            logo_h: 32.0,
            logo_mark: ink,
            logo_text: NavText::new(15.0, 24.0, 600),
            body_pad: 20.0,
            // `p-2.5 text-[0.75rem] leading-[1rem] font-medium
            // text-[#7b7b7b]/70` (GuidelineSidebar:59): shade 7 for AAA.
            caption: NavText::new(12.0, 16.0, 500),
            caption_pad: 10.0,
            caption_gap: 2.0,
            caption_color: shade7,
            group_gap: 12.0,
            row_gap: 2.0,
            // NewFolder:43: `h-10 pl-12 rounded-xl bg-[#f1f1f1]
            // border-[#ececec]`, the folder `#7240F1` at `left-2.5`, the
            // placeholder `#7b7b7b/50` (shade 7 for AAA).
            field_h: 40.0,
            field_radius: 12.0,
            field_text_x: 48.0,
            field_icon_x: 10.0,
            field_fill: wash,
            field_edge: line,
            field_mark: Color::hex(0x7240F1),
            field_placeholder: shade7,
        }
    }

    /// The sidebar's own width and the surface it stands on, the soft
    /// edge, the caption in the mid text, the field a hovered control.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass, control, space) =
            (&theme.colors, &theme.glass, &theme.control, &theme.spacing);
        let row = control.height(ControlSize::Lg);
        let slot = control.height(ControlSize::Xs);
        Self {
            width: theme.size.sidebar_w,
            min_h: 0.0,
            fill: colors.surface,
            edge: glass.edge_soft,
            edge_width: control.edge_width,
            canvas_radius: theme.radius.lg,
            head_pad: space.lg,
            logo_w: theme.size.sidebar_w - space.lg * 2.0,
            logo_h: slot,
            logo_mark: colors.text,
            logo_text: NavText::of(&theme.typography.base_2sm()),
            body_pad: space.md,
            caption: NavText::of(&theme.typography.caption_r()),
            caption_pad: space.sm,
            caption_gap: space.xs / 2.0,
            caption_color: colors.text_mid,
            group_gap: space.sm,
            row_gap: space.xs / 2.0,
            field_h: row,
            field_radius: theme.shape.nav,
            field_text_x: slot + space.md,
            field_icon_x: (slot - control.icon(IconSize::Md)) / 2.0 + space.xs,
            field_fill: glass.surface_hover,
            field_edge: glass.edge_soft,
            field_mark: colors.accent,
            field_placeholder: colors.text_mid,
        }
    }
}
