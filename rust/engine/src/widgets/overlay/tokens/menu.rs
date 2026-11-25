//! The menu tokens (task-104, front B): the hoff core kit's three menus
//! of its MenuPage, each a recipe the context menu reads by its
//! [`MenuStyle`]. On every other theme the three are the menu the theme
//! had, so its menus do not move.

use crate::styling::color::Color;
use crate::styling::theme::scales::ShadowSpec;
use crate::styling::theme::{ButtonSurface, ControlSize, IconSize, Theme};
use crate::styling::theme::{TypeRole, TypeRun};

/// The menubar's bar (the kit's `Toolbar`): a raised surface the titles
/// sit in, padded, a gap between them.
#[derive(Clone, Debug, PartialEq)]
pub struct MenubarTokens {
    /// Its fill, its shadows and its border as a ring.
    pub surface: ButtonSurface,
    pub radius: f32,
    pub pad: f32,
    pub gap: f32,
}

impl MenubarTokens {
    /// `Toolbar/index.tsx:72-73`: `shadow-[0px_1px_4px_-4px_rgba(0,0,0,0.08),
    /// 0px_8px_16px_-12px_rgba(0,0,0,0.13)] border border-[#ececec]
    /// bg-[#fcfcfc] rounded-[1.25rem]`, the actions `flex gap-2 p-2`.
    pub fn hoff_core() -> Self {
        let paper = hex(0xFCFCFC);
        Self {
            surface: ButtonSurface {
                fill: [paper, paper],
                overlay: Color::rgba(0.0, 0.0, 0.0, 0.0),
                shadow: vec![
                    ShadowSpec::new([0.0, 1.0], 4.0, -4.0, Color::rgba(0.0, 0.0, 0.0, 0.08)),
                    ShadowSpec::new([0.0, 8.0], 16.0, -12.0, Color::rgba(0.0, 0.0, 0.0, 0.13)),
                    ShadowSpec::new([0.0, 0.0], 0.0, 1.0, hex(0xECECEC)),
                ],
                text: hex(INK),
            },
            radius: 20.0,
            pad: 8.0,
            gap: 8.0,
        }
    }

    /// The theme's floating surface: the popover glass, the floating
    /// shadow and the soft edge as its ring, the dropdown radius, the
    /// menu's padding, an `xs` gap.
    pub fn derive(theme: &Theme) -> Self {
        let glass = &theme.glass;
        let ring = ShadowSpec::new([0.0, 0.0], 0.0, theme.control.edge_width, glass.edge_soft);
        Self {
            surface: ButtonSurface {
                fill: [glass.popover, glass.popover],
                overlay: Color::rgba(0.0, 0.0, 0.0, 0.0),
                shadow: [theme.shadows.floating.as_slice(), &[ring]].concat(),
                text: theme.colors.text,
            },
            radius: theme.shape.dropdown,
            pad: theme.control.menu_pad,
            gap: theme.spacing.xs,
        }
    }
}

/// Which of the kit's menus a context menu draws as.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MenuStyle {
    /// Rows with a leading icon (`ProfileMenu`, the header's
    /// `UserDropdown`).
    #[default]
    Profile,
    /// Compact rows with a trailing shortcut (`FileMenu`).
    File,
    /// Compact rows with a leading check (`FiltersMenu`).
    Filter,
    /// Tall rows with a leading icon in larger text (`Upload`).
    Upload,
}

/// A menu (the context menu, the dropdown menu's panel): its surface, its
/// rows and its separators. A row's check, when it has one, takes the
/// icon's slot, size and color; its shortcut sits at the row's end, `gap`
/// after the label.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuTokens {
    pub fill: Color,
    /// Drop shadows under the surface, first on top.
    pub shadow: Vec<ShadowSpec>,
    /// The border, its color and width; alpha 0 is none.
    pub border: Color,
    pub border_w: f32,
    /// The hoff glass's inset key-light on the surface.
    pub keylight: bool,
    pub radius: f32,
    /// Between the surface's edge and its rows.
    pub pad: f32,
    /// The narrowest it gets; it grows to its widest row.
    pub min_w: f32,
    pub item_h: f32,
    /// Between a row's highlight edge and its icon or label.
    pub item_pad_x: f32,
    pub item_radius: f32,
    pub icon: f32,
    /// Between the icon and the label, and between the label and the
    /// shortcut.
    pub gap: f32,
    pub label: TypeRun,
    pub text: Color,
    pub text_hover: Color,
    pub icon_color: Color,
    pub icon_hover: Color,
    pub hover_fill: Color,
    /// A separator row: its height, its line, and whether the line runs
    /// the surface's whole width (the kit's `-left-2 -right-2`).
    pub sep_h: f32,
    pub sep_color: Color,
    pub sep_full: bool,
    pub disabled_alpha: f32,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

/// `shadow-2xl` (tailwind v4 theme.css): 0 25px 50px -12px black/25.
fn shadow_2xl() -> Vec<ShadowSpec> {
    vec![ShadowSpec::new(
        [0.0, 25.0],
        50.0,
        -12.0,
        Color::rgba(0.0, 0.0, 0.0, 0.25),
    )]
}

/// The body's text (`app/layout.tsx:108`, `text-[#121212]`), which the
/// kit's menu rows inherit.
const INK: u32 = 0x121212;

impl MenuTokens {
    /// The kit's profile menu (`ProfileMenu/index.tsx:30`, `:33`, `:37`;
    /// `Layout/Header/UserDropdown`): `w-55 p-2 rounded-[1.25rem] border
    /// border-[#E5E5E5] bg-[#FCFCFC] shadow-2xl`, rows `h-10 px-2.5
    /// rounded-xl gap-3.5 text-[0.75rem] font-semibold fill-[#7B7B7B]
    /// hover:bg-[#F1F1F1] hover:fill-[#000]`, icons `size-5`, a separator
    /// `mb-4` with a full-width `h-0.25 bg-[#F4F4F4]` line in its middle.
    /// Disabled rows fade as the kit's disabled controls (`opacity-30`).
    pub fn hoff_core() -> Self {
        let ink = hex(INK);
        Self {
            fill: hex(0xFCFCFC),
            shadow: shadow_2xl(),
            border: hex(0xE5E5E5),
            border_w: 1.0,
            keylight: false,
            radius: 20.0,
            pad: 8.0,
            min_w: 220.0,
            item_h: 40.0,
            item_pad_x: 10.0,
            item_radius: 12.0,
            icon: 20.0,
            gap: 14.0,
            // No leading: the preflight's 1.5.
            label: TypeRole::BodyStrong.run(),
            text: ink,
            text_hover: ink,
            icon_color: hex(0x7B7B7B),
            icon_hover: hex(0x000000),
            hover_fill: hex(0xF1F1F1),
            sep_h: 16.0,
            sep_color: hex(0xF4F4F4),
            sep_full: true,
            disabled_alpha: 0.30,
        }
    }

    /// The kit's file menu (`FileMenu/index.tsx:33`, `:36`): `w-51 p-2
    /// rounded-[1.25rem] bg-[#FCFCFC] border border-[#ECECEC] shadow-2xl`,
    /// rows `justify-between gap-2.5 h-9 px-2 rounded-xl text-[0.75rem]
    /// leading-[1rem] font-medium hover:bg-[#F1F1F1]`, the shortcut a key
    /// cap at the row's end (`:42`), the separator `mb-4` with a
    /// full-width `h-0.25 bg-[#ECECEC]` line. It has no icon; a check
    /// would be the filter menu's (`FiltersMenu/index.tsx:19`, `size-4`).
    /// This menu with its labels in `label`: a menu style's role over
    /// another theme's menu.
    pub fn labelled(self, label: TypeRun) -> Self {
        Self { label, ..self }
    }

    pub fn hoff_core_file() -> Self {
        let profile = Self::hoff_core();
        let gray = hex(0x7B7B7B);
        Self {
            border: hex(0xECECEC),
            min_w: 204.0,
            item_h: 36.0,
            item_pad_x: 8.0,
            icon: 16.0,
            gap: 10.0,
            label: TypeRole::Label.run(),
            icon_hover: gray,
            sep_color: hex(0xECECEC),
            ..profile
        }
    }

    /// The kit's filter menu (`FiltersMenu/index.tsx:38`, `:15`, `:19`,
    /// `:47`): `w-45 p-2 bg-[#FCFCFC] border border-[#ECECEC] rounded-2xl
    /// shadow-2xl`, rows `h-9 px-2.5 rounded-[0.625rem] text-[0.75rem]
    /// font-medium hover:bg-[#f1f1f1]`, the check `size-4 mr-2
    /// fill-[#7B7B7B]` (shown or not, its slot kept), the separator
    /// `-mx-2 my-2 h-0.25 bg-[#E2E2E2]`.
    pub fn hoff_core_filter() -> Self {
        let profile = Self::hoff_core();
        let gray = hex(0x7B7B7B);
        Self {
            border: hex(0xECECEC),
            radius: 16.0,
            min_w: 180.0,
            item_h: 36.0,
            item_radius: 10.0,
            icon: 16.0,
            gap: 8.0,
            // No leading: the preflight's 1.5.
            label: TypeRole::Label.run(),
            icon_hover: gray,
            sep_h: 17.0,
            sep_color: hex(0xE2E2E2),
            ..profile
        }
    }

    /// The kit's upload menu (`Upload/index.tsx:30`, `:33`, `:37`): `w-51
    /// p-2 rounded-[1.25rem] bg-[#FCFCFC] border border-[#E2E2E2]
    /// shadow-2xl`, rows `gap-2.5 h-10 px-2 rounded-xl text-[0.8125rem]
    /// font-medium hover:bg-[#F1F1F1]`, the icon `size-5 fill-[#7B7B7B]
    /// group-hover:fill-[#000]` centered in a `size-6` slot: the slot's
    /// two pixels on each side go to the row's padding and the gap.
    pub fn hoff_core_upload() -> Self {
        let profile = Self::hoff_core();
        let slot = 24.0;
        let icon = 20.0;
        let side = (slot - icon) / 2.0;
        Self {
            border: hex(0xE2E2E2),
            min_w: 204.0,
            item_pad_x: 8.0 + side,
            icon,
            gap: 10.0 + side,
            // No leading: the preflight's 1.5.
            label: TypeRole::BodyLg.medium(),
            ..profile
        }
    }

    /// Every other theme keeps the menu it had, for all three styles: the
    /// popover glass with its soft edge, the floating shadow and the
    /// key-light, `Md` rows at the item radius, base-2sm labels,
    /// text-default at rest and text-active on hover, the icon in the
    /// label's color.
    pub fn derive(theme: &Theme) -> Self {
        let glass = &theme.glass;
        let control = &theme.control;
        Self {
            fill: glass.popover,
            shadow: theme.shadows.floating.clone(),
            border: glass.edge_soft,
            border_w: control.edge_width,
            keylight: true,
            radius: theme.shape.pill,
            pad: control.menu_pad,
            min_w: theme.size.menu_w,
            item_h: control.height(ControlSize::Md),
            item_pad_x: control.menu_pad,
            item_radius: theme.shape.item,
            icon: control.icon(IconSize::Sm),
            gap: control.inline_gap,
            label: TypeRole::BodyStrong.run(),
            text: glass.text_default,
            text_hover: glass.text_active,
            icon_color: glass.text_default,
            icon_hover: glass.text_active,
            hover_fill: glass.surface_active,
            sep_h: theme.spacing.xs * 2.0 + control.edge_width,
            sep_color: glass.surface_active,
            sep_full: false,
            disabled_alpha: glass.disabled_alpha,
        }
    }
}
