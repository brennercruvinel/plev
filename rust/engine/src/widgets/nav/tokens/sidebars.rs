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
use crate::styling::theme::scales::ShadowSpec;
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
    /// The canvas sidebars' parts.
    pub canvas: CanvasRecipe,
}

/// The canvas sidebars (`SceneAndAssetsSidebar`,
/// `DesignAndAnimationSidebar`): the head, the tab band, the scene's
/// rows and their toggles, the asset tiles.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasRecipe {
    /// The head's `p-4 pb-3`, `mb-3` under its top row.
    pub pad: f32,
    pub head_pad_b: f32,
    pub head_gap: f32,
    /// The head's icon button (`size-8 rounded-md`, a `size-5` icon in
    /// the gray, `hover:bg-[#F8F7F7]`).
    pub button: f32,
    pub button_radius: f32,
    pub button_icon: f32,
    pub button_hover: Color,
    /// A quiet mark: the kit's gray, 3:1 on the surface.
    pub mark: Color,
    /// The project's title (`text-[0.9375rem] leading-[1.5rem]
    /// font-semibold`, `gap-2.5`) and its chevron's `size-6` tile.
    pub title: NavText,
    pub title_gap: f32,
    pub chevron_tile: f32,
    pub chevron: f32,
    /// The project's kind (`py-0.5`, `#7B7B7B/80`: shade 7 for AAA).
    pub subtitle: NavText,
    pub subtitle_pad: f32,
    pub ink: Color,
    pub subtle: Color,
    /// The tab band's `px-4 py-3`.
    pub band_pad_y: f32,
    /// The scene's list (`p-3`, `gap-1`) and a row (`p-0.75 border
    /// rounded-xl`, `gap-3`): its tile (`size-8 rounded-lg`, a `size-4`
    /// icon), the selected row's fill and rim, the tile's rise.
    pub list_pad: f32,
    pub row_gap: f32,
    pub row_h: f32,
    pub row_radius: f32,
    pub row_pad: f32,
    pub row_fill: Color,
    pub row_edge: Color,
    pub tile: f32,
    pub tile_radius: f32,
    pub tile_icon: f32,
    pub tile_fill: Color,
    pub tile_raised: Color,
    pub tile_shadow: ShadowSpec,
    pub label: NavText,
    pub label_gap: f32,
    /// The row's toggles: `size-4`, `gap-3`, `right-3`; gray at rest,
    /// ink when on.
    pub toggle: f32,
    pub toggle_gap: f32,
    /// The asset tiles (`mx-1 mt-2`, `p-1.5 rounded-2xl`; the section's
    /// `p-4 pt-0` is the accordion's),
    /// the picture's sizes (`size-22`, `size-16`) and its stand-in.
    pub grid_gap: f32,
    pub cell_radius: f32,
    pub cell_pad: f32,
    pub picture_large: f32,
    pub picture: f32,
    pub picture_fill: [Color; 2],
    /// The search's `p-3` band.
    pub search_pad: f32,
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
            canvas: CanvasRecipe::hoff_core(),
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
            canvas: CanvasRecipe::derive(theme),
        }
    }
}

impl CanvasRecipe {
    pub fn hoff_core() -> Self {
        let [paper, shade2, wash, _, shade5, gray, shade7, _, ink] = SHADES;
        Self {
            pad: 16.0,
            head_pad_b: 12.0,
            head_gap: 12.0,
            button: 32.0,
            button_radius: 6.0,
            button_icon: 20.0,
            button_hover: shade2,
            mark: gray,
            title: NavText::new(15.0, 24.0, 600),
            title_gap: 10.0,
            chevron_tile: 24.0,
            chevron: 16.0,
            subtitle: NavText::new(12.0, 16.0, 500),
            subtitle_pad: 2.0,
            ink,
            subtle: shade7,
            band_pad_y: 12.0,
            list_pad: 12.0,
            row_gap: 4.0,
            row_h: 40.0,
            row_radius: 12.0,
            row_pad: 3.0,
            row_fill: wash,
            row_edge: shade5,
            tile: 32.0,
            tile_radius: 8.0,
            tile_icon: 16.0,
            tile_fill: wash,
            tile_raised: paper,
            tile_shadow: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
            label: NavText::new(12.0, 16.0, 500),
            label_gap: 12.0,
            toggle: 16.0,
            toggle_gap: 12.0,
            grid_gap: 8.0,
            cell_radius: 16.0,
            cell_pad: 6.0,
            picture_large: 88.0,
            picture: 64.0,
            picture_fill: [shade5, tint(gray, 0.5)],
            search_pad: 12.0,
        }
    }

    /// The control scale, the spacing steps and the surface's tones.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass, control, space) =
            (&theme.colors, &theme.glass, &theme.control, &theme.spacing);
        let core = Self::hoff_core();
        let row = control.height(ControlSize::Lg);
        let slot = control.height(ControlSize::Xs);
        Self {
            pad: space.md,
            head_pad_b: space.sm,
            head_gap: space.sm,
            button: slot,
            button_radius: theme.shape.micro,
            button_icon: control.icon(IconSize::Md),
            button_hover: glass.surface_hover,
            mark: colors.text_mid,
            title: NavText::of(&theme.typography.base_2sm()),
            title_gap: space.sm,
            chevron_tile: slot * core.chevron_tile / core.button,
            chevron: control.icon(IconSize::Sm),
            subtitle: NavText::of(&theme.typography.caption_r()),
            subtitle_pad: space.xs / 2.0,
            ink: colors.text,
            subtle: colors.text_mid,
            band_pad_y: space.sm,
            list_pad: space.sm,
            row_gap: space.xs,
            row_h: row,
            row_radius: theme.shape.nav,
            row_pad: (row - slot) / 2.0 - control.edge_width,
            row_fill: glass.surface_hover,
            row_edge: glass.edge,
            tile: slot,
            tile_radius: theme.shape.micro,
            tile_icon: control.icon(IconSize::Sm),
            tile_fill: glass.surface_hover,
            tile_raised: colors.surface,
            tile_shadow: core.tile_shadow,
            label: NavText::of(&theme.typography.caption_r()),
            label_gap: space.sm,
            toggle: control.icon(IconSize::Sm),
            toggle_gap: space.sm,
            grid_gap: space.sm,
            cell_radius: theme.radius.lg,
            cell_pad: space.xs,
            picture_large: core.picture_large,
            picture: core.picture,
            picture_fill: [colors.bg_hover, colors.divider],
            search_pad: space.sm,
        }
    }
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], c.0[3] * a])
}
