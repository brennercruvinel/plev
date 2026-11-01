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
    /// `w-60`: a sidebar on its own (`HomeSidebar`, `GuidelineSidebar`).
    pub width: f32,
    /// The app shell's sidebar, the kit's `Layout` (`!w-55`,
    /// `components/Layout/index.tsx:20`): the dock's panel
    /// (`navigation.dock.width`).
    pub shell_w: f32,
    /// `min-h-219`; 0 where a theme lets the sidebar be as tall as its rows.
    pub min_h: f32,
    /// `bg-[#FCFCFC]`, the rim `border-[#ECECEC]`.
    pub fill: Color,
    pub edge: Color,
    pub edge_width: f32,
    /// The canvas sidebars' `rounded-[1.25rem]`.
    pub canvas_radius: f32,
    /// The head's padding round the logo (`header::Logo`, front F): the
    /// kit's `p-6` less the logo's own `p-1` (`theme.header.logo.pad`),
    /// so the mark stands where the kit's does and the head is its 80.
    pub head_pad: f32,
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
    /// The design and animation sidebar's parts.
    pub design: DesignRecipe,
}

/// `DesignAndAnimationSidebar`: its head (`p-3 pl-4`, the avatars'
/// `size-8 border-2 -ml-2`), a group (`min-h-12 px-4 py-3`, the add
/// button's `size-6 rounded-md`, `p-4 pt-0` round its body), the fields
/// and value chips (`h-9 rounded-[0.625rem] bg-[#F1F1F1]`), the range
/// (`rounded-lg`, filled `rgba(123,123,123,0.3)`), the loop's strip
/// (`h-15 rounded-xl`, its span `rgba(18,18,18,0.35)` between 2 px
/// handles), the tiles (`rounded-2xl`, the effects' `rounded-xl`).
#[derive(Clone, Debug, PartialEq)]
pub struct DesignRecipe {
    pub head_pad: f32,
    pub head_pad_start: f32,
    pub avatar: f32,
    pub avatar_ring: f32,
    pub avatar_overlap: f32,
    pub avatars: [Color; 2],
    pub header_h: f32,
    pub header: NavText,
    pub add: f32,
    pub add_icon: f32,
    pub gap: f32,
    pub stack_gap: f32,
    pub field_h: f32,
    pub field_radius: f32,
    pub field_fill: Color,
    pub field_text: NavText,
    pub field_text_x: f32,
    pub field_prefix_x: f32,
    pub unit_w: f32,
    pub chip_w: f32,
    pub chip_w_narrow: f32,
    pub chip_icon: f32,
    pub range_radius: f32,
    pub range_fill: Color,
    pub strip_h: f32,
    pub strip_radius: f32,
    pub strip_span: Color,
    pub strip_handle: f32,
    pub tile_radius: f32,
    pub tile_radius_small: f32,
}

impl SidebarsRecipe {
    pub fn hoff_core() -> Self {
        let [paper, _, wash, line, _, _, shade7, _, _] = SHADES;
        Self {
            width: 240.0,
            shell_w: 220.0,
            min_h: 876.0,
            fill: paper,
            edge: line,
            edge_width: 1.0,
            canvas_radius: 20.0,
            // `p-6` (24) less the logo's `p-1` (4)
            head_pad: 20.0,
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
            // one sidebar width in a theme of its own
            shell_w: theme.size.sidebar_w,
            min_h: 0.0,
            fill: colors.surface,
            edge: glass.edge_soft,
            edge_width: control.edge_width,
            canvas_radius: theme.radius.lg,
            // the mark at the `lg` step, the logo's own pad inside it
            // (every theme's logo keeps the kit's, `HeaderTokens::derive`)
            head_pad: (space.lg - theme.header.logo.pad).max(0.0),
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
            design: DesignRecipe::hoff_core(),
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
            design: DesignRecipe::derive(theme),
        }
    }
}

impl DesignRecipe {
    pub fn hoff_core() -> Self {
        use crate::styling::theme::hoff_core::{BLUE, ORANGE};
        let [_, _, wash, _, _, gray, _, _, ink] = SHADES;
        Self {
            head_pad: 12.0,
            head_pad_start: 16.0,
            avatar: 32.0,
            avatar_ring: 2.0,
            avatar_overlap: 8.0,
            avatars: [ORANGE, BLUE],
            header_h: 48.0,
            header: NavText::new(12.0, 16.0, 600),
            add: 24.0,
            add_icon: 16.0,
            gap: 8.0,
            stack_gap: 6.0,
            field_h: 36.0,
            field_radius: 10.0,
            field_fill: wash,
            field_text: NavText::new(12.0, 16.0, 500),
            field_text_x: 30.0,
            field_prefix_x: 10.0,
            unit_w: 24.0,
            chip_w: 78.0,
            chip_w_narrow: 56.0,
            chip_icon: 16.0,
            range_radius: 8.0,
            range_fill: tint(gray, 0.3),
            strip_h: 60.0,
            strip_radius: 12.0,
            strip_span: tint(ink, 0.35),
            strip_handle: 2.0,
            tile_radius: 16.0,
            tile_radius_small: 12.0,
        }
    }

    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass, control, space) =
            (&theme.colors, &theme.glass, &theme.control, &theme.spacing);
        let core = Self::hoff_core();
        let field = control.height(ControlSize::Md);
        Self {
            head_pad: space.sm,
            head_pad_start: space.md,
            avatar: control.height(ControlSize::Xs),
            avatar_ring: control.edge_width * 2.0,
            avatar_overlap: space.sm,
            avatars: [colors.warning, colors.info],
            header_h: control.height(ControlSize::Lg) + space.sm,
            header: NavText::of(&theme.typography.caption_r()),
            add: field * core.add / core.field_h,
            add_icon: control.icon(IconSize::Sm),
            gap: space.sm,
            stack_gap: space.xs,
            field_h: field,
            field_radius: theme.shape.nav,
            field_fill: glass.surface_hover,
            field_text: NavText::of(&theme.typography.caption_r()),
            field_text_x: space.xl,
            field_prefix_x: space.sm,
            unit_w: space.lg,
            chip_w: field * core.chip_w / core.field_h,
            chip_w_narrow: field * core.chip_w_narrow / core.field_h,
            chip_icon: control.icon(IconSize::Sm),
            range_radius: theme.shape.micro,
            range_fill: glass.surface_active,
            strip_h: field * core.strip_h / core.field_h,
            strip_radius: theme.shape.nav,
            strip_span: glass.surface_active,
            strip_handle: control.edge_width * 2.0,
            tile_radius: theme.radius.lg,
            tile_radius_small: theme.shape.nav,
        }
    }
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], c.0[3] * a])
}
