//! Disclosure tokens: what the hoff core kit writes as a tailwind class
//! list for the two things that open and close, as recipes the widgets
//! read. One recipe shape serves both looks: the kit's Accordion (the
//! assets sidebar: a divider above each section, a bare header, the
//! chevron in a box) and its Dropdown (the home sidebar: a bordered,
//! rounded trigger with an icon tile), which the [`Collapsible`] draws.
//!
//! `DisclosureTokens::hoff_core` is the kit verbatim (the private
//! typescript design system: `components/SceneAndAssetsSidebar/Assets/
//! Accordion` and `components/HomeSidebar/Dropdown`). Every other theme
//! takes the same geometry and motion with its own palette through
//! [`DisclosureTokens::derive`]; derived from the core theme, the tokens
//! are the kit, value for value.
//!
//! [`Collapsible`]: super::collapsible::Collapsible

use crate::compose::animation::Easing;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::scales::ShadowSpec;

/// A trigger's surface in one state: its fill and its border color
/// (alpha 0 is none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureSurface {
    pub fill: Color,
    pub border: Color,
}

/// The leading icon tile of the Dropdown look (`size-8 rounded-lg`):
/// flat at rest, raised while its section holds the current page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureTile {
    pub size: f32,
    pub radius: f32,
    /// Space between the tile and the label (`mr-3`).
    pub gap: f32,
    /// The icon's edge inside the tile.
    pub icon: f32,
    pub fill_current: Color,
    pub shadow_current: ShadowSpec,
    pub icon_rest: Color,
    pub icon_current: Color,
}

/// The kit's raised cap (`px-1.5 py-0.5 rounded-md` over a four-layer
/// shadow): the count beside a Dropdown's label and the key caps of its
/// menus and its search. Padding x and y, radius, fill, its css shadow
/// list (first on top), its text.
#[derive(Clone, Debug, PartialEq)]
pub struct CapRecipe {
    pub pad: [f32; 2],
    pub radius: f32,
    pub fill: Color,
    pub shadow: Vec<ShadowSpec>,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub text: Color,
}

/// The Dropdown's list: links one under the other, a rail down their
/// left and a rounded branch from the rail to each link.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureRail {
    /// The rail's x from the list's left, its width, its color.
    pub x: f32,
    pub width: f32,
    pub color: Color,
    /// How far above the list's bottom the rail stops.
    pub end: f32,
    /// A branch: as wide as `branch_w`, as tall as `branch_h` from the
    /// link's top, its bottom-left corner rounded by `branch_radius`.
    pub branch_w: f32,
    pub branch_h: f32,
    pub branch_radius: f32,
    /// A link's height and its padding left and right.
    pub link_h: f32,
    pub link_pad: [f32; 2],
    /// A link's label at rest and when current or hovered.
    pub link_rest: Color,
    pub link_active: Color,
    /// The chevron that marks the current link.
    pub link_chevron: f32,
}

/// The kit's Accordion grid: square thumbnails two to a row, a wash
/// under each, two inset rings on the one under the pointer (the outer
/// gray, the inner paper), the image centered at one of two sizes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridRecipe {
    pub columns: usize,
    pub gap: f32,
    pub pad: f32,
    pub radius: f32,
    pub fill: Color,
    /// The rings: the outer's width and color, the inner's.
    pub ring_outer: (f32, Color),
    pub ring_inner: (f32, Color),
    /// The image's edge, small and large, and the placeholder's color.
    pub image: f32,
    pub image_large: f32,
    pub glyph_color: Color,
}

/// One look of a disclosure: the trigger's box, its states, its label,
/// its chevron, the divider above it and the padding of what it reveals.
/// Paddings are css order: top, right, bottom, left.
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureRecipe {
    pub pad: [f32; 4],
    pub radius: f32,
    pub border_width: f32,
    pub rest: DisclosureSurface,
    pub hover: DisclosureSurface,
    /// While open (the kit fills an open Dropdown as it fills a hovered one).
    pub open: DisclosureSurface,
    /// While the section holds the current page (the Dropdown's `active`).
    pub current: DisclosureSurface,
    /// Label size and line height in px, and weight.
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub label: Color,
    /// The box the chevron is centered in, the chevron's edge, the space
    /// before the box, and the chevron's color.
    pub chevron_box: f32,
    pub chevron: f32,
    pub chevron_gap: f32,
    pub chevron_color: Color,
    /// The leading icon tile, for a look that has one.
    pub tile: Option<DisclosureTile>,
    /// The count beside the label, for a look that has one.
    pub counter: Option<CapRecipe>,
    /// The rail of a list revealed, for a look that has one.
    pub rail: Option<DisclosureRail>,
    /// The thumbnail grid revealed, for a look that has one.
    pub grid: Option<GridRecipe>,
    /// The rule above every section but the first (`border-t`,
    /// `first:border-t-0`); width 0 is none.
    pub divider: Color,
    pub divider_width: f32,
    /// Padding around what the disclosure reveals.
    pub content_pad: [f32; 4],
    /// Text revealed as a body: size, line height, weight, color.
    pub body_size: f32,
    pub body_line_height: f32,
    pub body_weight: u16,
    pub body: Color,
}

/// A list's item, the kit's two: the Folder row (an icon box and a
/// label, a wash on hover) and the AssetItem tile (a square thumbnail in
/// a bordered well, an add button that shows on hover).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemRecipe {
    /// The row: padding, radius, border width, its surfaces.
    pub row_pad: f32,
    pub row_radius: f32,
    pub row_border: f32,
    pub row_rest: DisclosureSurface,
    pub row_hover: DisclosureSurface,
    pub row_selected: DisclosureSurface,
    /// The icon's box, the icon in it, the space before the label.
    pub icon_box: f32,
    pub icon: f32,
    pub gap: f32,
    pub icon_color: Color,
    /// The label, and the line under it in the dim tone.
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub label: Color,
    pub meta_size: f32,
    pub meta_line_height: f32,
    pub meta_weight: u16,
    pub meta: Color,
    /// The tile: its edge, radius, border width, its surfaces, the
    /// thumbnail's edge inside it.
    pub tile: f32,
    pub tile_radius: f32,
    pub tile_border: f32,
    pub tile_rest: DisclosureSurface,
    pub tile_hover: DisclosureSurface,
    pub tile_selected: DisclosureSurface,
    pub thumb: f32,
    /// The tile's add button: its edge, its inset from the corner, its
    /// gradient top and bottom, its glyph's edge and color.
    pub add: f32,
    pub add_inset: f32,
    pub add_fill: [Color; 2],
    pub add_glyph: f32,
    pub add_glyph_color: Color,
}

/// The kit's AssetCard well, the frame an aspect ratio draws: padding,
/// radius, border width, surfaces at rest and hovered, the share of the
/// inner square a placeholder glyph takes, and the shade that rises from
/// the bottom on hover (its height, its color at the bottom).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRecipe {
    pub pad: f32,
    pub radius: f32,
    pub border: f32,
    pub rest: DisclosureSurface,
    pub hover: DisclosureSurface,
    pub glyph: f32,
    pub glyph_color: Color,
    pub shade_h: f32,
    pub shade: Color,
}

/// How a disclosure moves: the revealed height and the chevron's half
/// turn, each a duration in seconds and an easing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DisclosureMotion {
    pub reveal: f32,
    pub reveal_easing: Easing,
    pub turn: f32,
    pub turn_easing: Easing,
}

impl DisclosureMotion {
    /// The kit: `AnimateHeight duration={500}` with the library's default
    /// `ease`, and tailwind 4's `transition-transform` (150 ms,
    /// `cubic-bezier(0.4, 0, 0.2, 1)`) on the chevron.
    pub const fn hoff_core() -> Self {
        Self {
            reveal: 0.5,
            reveal_easing: Easing::CubicBezier(0.25, 0.1, 0.25, 1.0),
            turn: 0.15,
            turn_easing: Easing::CubicBezier(0.4, 0.0, 0.2, 1.0),
        }
    }
}

impl Default for DisclosureMotion {
    fn default() -> Self {
        Self::hoff_core()
    }
}

/// The disclosure slot of a theme (`theme.disclosure`).
#[derive(Clone, Debug, PartialEq)]
pub struct DisclosureTokens {
    /// The kit's Accordion.
    pub accordion: DisclosureRecipe,
    /// The kit's Dropdown, the collapsible's look.
    pub collapsible: DisclosureRecipe,
    pub motion: DisclosureMotion,
    /// The Item tab's row and tile.
    pub item: ItemRecipe,
    /// The key cap, small (a menu's shortcut) and medium (the search's).
    pub kbd: [CapRecipe; 2],
    /// The frame of an aspect ratio.
    pub aspect: AspectRecipe,
}

fn hex(v: u32) -> Color {
    Color::hex(v)
}

fn black(a: f32) -> Color {
    Color::rgba(0.0, 0.0, 0.0, a)
}

fn white(a: f32) -> Color {
    Color::rgba(1.0, 1.0, 1.0, a)
}

fn tint(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], a])
}

const NONE: Color = Color::TRANSPARENT;

const CLEAR: DisclosureSurface = DisclosureSurface {
    fill: NONE,
    border: NONE,
};

impl DisclosureTokens {
    /// The kit, class list by class list.
    pub fn hoff_core() -> Self {
        let ink = hex(0x121212);
        let gray = hex(0x7B7B7B);
        let line = hex(0xECECEC);
        let wash = hex(0xF1F1F1);
        let paper = hex(0xFCFCFC);

        // Accordion/index.tsx: the section `border-t border-[#ECECEC]
        // first:border-t-0` (:22); the header `px-4 py-3 text-[0.75rem]
        // font-semibold text-[#121212]` (:27), no fill in any state; the
        // chevron `size-4 fill-[#7B7B7B]` in a `size-6` box, pushed to the
        // end by `justify-between` (:31-33); the body `p-4 pt-0` (:45).
        // Line height 18: the 0.75rem label inherits the preflight's 1.5.
        let accordion = DisclosureRecipe {
            pad: [12.0, 16.0, 12.0, 16.0],
            radius: 0.0,
            border_width: 0.0,
            rest: CLEAR,
            hover: CLEAR,
            open: CLEAR,
            current: CLEAR,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 24.0,
            chevron: 16.0,
            chevron_gap: 0.0,
            chevron_color: gray,
            tile: None,
            counter: None,
            rail: None,
            // The grid: `flex flex-wrap -mt-2 -mx-1`, each thumbnail
            // `w-[calc(50%-0.5rem)] mx-1 mt-2 aspect-square p-1.5
            // bg-[#F1F1F1] rounded-2xl`, hovered `shadow-[0_0_0_1.5px
            // rgba(123,123,123,0.5)_inset,0px_0px_0px_4px_#fcfcfc_inset]`,
            // its image `size-16` or `size-22` (:46-55).
            grid: Some(GridRecipe {
                columns: 2,
                gap: 8.0,
                pad: 6.0,
                radius: 16.0,
                fill: wash,
                ring_outer: (1.5, tint(gray, 0.5)),
                ring_inner: (4.0, paper),
                image: 64.0,
                image_large: 88.0,
                glyph_color: gray,
            }),
            divider: line,
            divider_width: 1.0,
            content_pad: [0.0, 16.0, 16.0, 16.0],
            // The kit's sections hold thumbnails, not text: a text body
            // takes the Dropdown's list text (Dropdown :73-76).
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        // The counter `px-1.5 py-0.5 bg-[#f1f1f1] rounded-md
        // text-[0.6875rem] font-medium text-[#7b7b7b]` and its shadow
        // `0 0 0 1px black/11, inset 0 2px 0.8px white/27, inset 0 -1px
        // 0.6px black/20, 0 1px 4.2px -1px black/25` (:52).
        let cap = CapRecipe {
            pad: [6.0, 2.0],
            radius: 6.0,
            fill: wash,
            shadow: vec![
                ShadowSpec::new([0.0, 0.0], 0.0, 1.0, black(0.11)),
                ShadowSpec::inset([0.0, 2.0], 0.8, 0.0, white(0.27)),
                ShadowSpec::inset([0.0, -1.0], 0.6, 0.0, black(0.20)),
                ShadowSpec::new([0.0, 1.0], 4.2, -1.0, black(0.25)),
            ],
            font_size: 11.0,
            line_height: 16.5,
            font_weight: 500,
            text: gray,
        };

        // Dropdown/index.tsx: the trigger `p-0.75 pr-3 border rounded-xl
        // text-[0.75rem] font-semibold text-[#121212] hover:bg-[#f1f1f1]`,
        // filled while open, `bg-[#f1f1f1] border-[#ececec]` while
        // current, `border-transparent` otherwise (:25-30); the tile
        // `size-8 mr-3 rounded-lg`, raised `bg-[#fcfcfc]
        // shadow-[0_0_4px_0_rgba(18,18,18,0.10)] fill-[#121212]` while
        // current, `fill-[#7b7b7b]` otherwise, its icon `size-5` (:35-42);
        // the chevron `size-4 ml-3 fill-[#7b7b7b]` (:58); the list
        // `pt-0.5` (:70), its items `text-[0.75rem] font-semibold
        // text-[#7b7b7b]` (:73-76).
        let collapsible = DisclosureRecipe {
            pad: [3.0, 12.0, 3.0, 3.0],
            radius: 12.0,
            border_width: 1.0,
            rest: CLEAR,
            hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            open: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            current: DisclosureSurface {
                fill: wash,
                border: line,
            },
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            chevron_box: 16.0,
            chevron: 16.0,
            chevron_gap: 12.0,
            chevron_color: gray,
            tile: Some(DisclosureTile {
                size: 32.0,
                radius: 8.0,
                gap: 12.0,
                icon: 20.0,
                fill_current: paper,
                shadow_current: ShadowSpec::new([0.0, 0.0], 4.0, 0.0, tint(ink, 0.10)),
                icon_rest: gray,
                icon_current: ink,
            }),
            counter: Some(cap.clone()),
            grid: None,
            // The list: the rail `before:left-4.75 before:bottom-8
            // before:w-[1.5px] before:bg-[#ececec]` (:70); each link `h-10
            // pl-12 pr-3`, `text-[#121212]` when current or hovered,
            // `text-[#7b7b7b]` otherwise, its branch `before:left-4.75
            // before:bottom-5 before:w-4 before:border-l-[1.5px]
            // before:border-b-[1.5px] before:rounded-bl-lg` (:73), the
            // current one's chevron `size-4 -rotate-90` (:83).
            rail: Some(DisclosureRail {
                x: 19.0,
                width: 1.5,
                color: line,
                end: 32.0,
                branch_w: 16.0,
                branch_h: 20.0,
                branch_radius: 8.0,
                link_h: 40.0,
                link_pad: [48.0, 12.0],
                link_rest: gray,
                link_active: ink,
                link_chevron: 16.0,
            }),
            divider: NONE,
            divider_width: 0.0,
            content_pad: [2.0, 0.0, 0.0, 0.0],
            body_size: 12.0,
            body_line_height: 18.0,
            body_weight: 600,
            body: gray,
        };

        // Folder/index.tsx: `p-0.75 rounded-xl text-[0.75rem]
        // font-semibold border border-transparent hover:bg-[#f1f1f1]`, the
        // icon in a `size-8 mr-3` box (:23-24); selected takes the
        // Dropdown's current look (Dropdown :28). AssetItem/index.tsx: the
        // tile `size-15 rounded-[0.625rem] bg-[#F1F1F1] border
        // border-[#ECECEC] hover:border-[#C4C4C4] hover:bg-[#fcfcfc]`, its
        // image `size-11` (:12-19); the add button `size-4 top-0.5
        // right-0.5 rounded-full from-[#323232] to-[#121212]`, its glyph
        // `size-3 fill-white` (:22-24).
        let item = ItemRecipe {
            row_pad: 3.0,
            row_radius: 12.0,
            row_border: 1.0,
            row_rest: CLEAR,
            row_hover: DisclosureSurface {
                fill: wash,
                border: NONE,
            },
            row_selected: DisclosureSurface {
                fill: wash,
                border: line,
            },
            icon_box: 32.0,
            icon: 20.0,
            gap: 12.0,
            icon_color: gray,
            font_size: 12.0,
            line_height: 18.0,
            font_weight: 600,
            label: ink,
            meta_size: 11.0,
            meta_line_height: 16.5,
            meta_weight: 500,
            meta: gray,
            tile: 60.0,
            tile_radius: 10.0,
            tile_border: 1.0,
            tile_rest: DisclosureSurface {
                fill: wash,
                border: line,
            },
            tile_hover: DisclosureSurface {
                fill: paper,
                border: hex(0xC4C4C4),
            },
            tile_selected: DisclosureSurface {
                fill: paper,
                border: ink,
            },
            thumb: 44.0,
            add: 16.0,
            add_inset: 2.0,
            add_fill: [hex(0x323232), ink],
            add_glyph: 12.0,
            add_glyph_color: hex(0xFFFFFF),
        };

        // The key caps: FileMenu/index.tsx's `text-[0.6875rem]
        // leading-[1rem]` (:42) and Search/index.tsx's `text-[0.75rem]
        // leading-[1rem]` (:42), on the counter's cap.
        let kbd = [
            CapRecipe {
                line_height: 16.0,
                ..cap.clone()
            },
            CapRecipe {
                font_size: 12.0,
                line_height: 16.0,
                ..cap
            },
        ];

        // AssetCard/index.tsx: `aspect-square p-6 bg-[#F8F7F7] border
        // border-[#ECECEC] rounded-[1.25rem] hover:bg-[#F1F1F1]` (:10);
        // the shade `h-24 bg-gradient-to-t from-[#171717]/100
        // group-hover:opacity-30` (:20). The image fills the well: a
        // placeholder glyph takes half of it, in the dim tone.
        let aspect = AspectRecipe {
            pad: 24.0,
            radius: 20.0,
            border: 1.0,
            rest: DisclosureSurface {
                fill: hex(0xF8F7F7),
                border: line,
            },
            hover: DisclosureSurface {
                fill: wash,
                border: line,
            },
            glyph: 0.5,
            glyph_color: gray,
            shade_h: 96.0,
            shade: tint(hex(0x171717), 0.30),
        };

        Self {
            accordion,
            collapsible,
            motion: DisclosureMotion::hoff_core(),
            item,
            kbd,
            aspect,
        }
    }

    /// The kit's geometry and motion in a theme's palette: ink is the
    /// text, the gray is the dim text, the line is the divider, the wash
    /// is the hover tone, the paper is the surface.
    pub fn derive(theme: &Theme) -> Self {
        let colors = &theme.colors;
        let kit = Self::hoff_core();
        let wash = DisclosureSurface {
            fill: colors.bg_hover,
            border: NONE,
        };
        let accordion = DisclosureRecipe {
            label: colors.text,
            chevron_color: colors.text_dim,
            divider: colors.divider,
            body: colors.text_dim,
            grid: kit.accordion.grid.map(|grid| GridRecipe {
                fill: colors.bg_hover,
                ring_outer: (grid.ring_outer.0, tint(colors.text_dim, 0.5)),
                ring_inner: (grid.ring_inner.0, colors.surface),
                glyph_color: colors.text_dim,
                ..grid
            }),
            ..kit.accordion
        };
        let collapsible = DisclosureRecipe {
            hover: wash,
            open: wash,
            current: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            label: colors.text,
            chevron_color: colors.text_dim,
            tile: kit.collapsible.tile.map(|tile| DisclosureTile {
                fill_current: colors.surface,
                shadow_current: ShadowSpec {
                    color: tint(colors.text, 0.10),
                    ..tile.shadow_current
                },
                icon_rest: colors.text_dim,
                icon_current: colors.text,
                ..tile
            }),
            // The counter's shadows are black and white washes and stay;
            // its fill and its text follow the palette.
            counter: kit.collapsible.counter.clone().map(|counter| CapRecipe {
                fill: colors.bg_hover,
                text: colors.text_dim,
                ..counter
            }),
            rail: kit.collapsible.rail.map(|rail| DisclosureRail {
                color: colors.divider,
                link_rest: colors.text_dim,
                link_active: colors.text,
                ..rail
            }),
            body: colors.text_dim,
            ..kit.collapsible
        };
        // The hover border sits between the line and the dim text.
        let between = Color([
            (colors.divider.0[0] + colors.text_dim.0[0]) / 2.0,
            (colors.divider.0[1] + colors.text_dim.0[1]) / 2.0,
            (colors.divider.0[2] + colors.text_dim.0[2]) / 2.0,
            (colors.divider.0[3] + colors.text_dim.0[3]) / 2.0,
        ]);
        let item = ItemRecipe {
            row_hover: wash,
            row_selected: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            icon_color: colors.text_dim,
            label: colors.text,
            meta: colors.text_dim,
            tile_rest: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            tile_hover: DisclosureSurface {
                fill: colors.surface,
                border: between,
            },
            tile_selected: DisclosureSurface {
                fill: colors.surface,
                border: colors.text,
            },
            add_fill: [colors.accent_dim, colors.accent],
            add_glyph_color: colors.bg,
            ..kit.item
        };
        let kbd = kit.kbd.map(|cap| CapRecipe {
            fill: colors.bg_hover,
            text: colors.text_dim,
            ..cap
        });
        let aspect = AspectRecipe {
            rest: DisclosureSurface {
                fill: colors.bg_panel,
                border: colors.divider,
            },
            hover: DisclosureSurface {
                fill: colors.bg_hover,
                border: colors.divider,
            },
            glyph_color: colors.text_dim,
            ..kit.aspect
        };
        Self {
            accordion,
            collapsible,
            motion: kit.motion,
            item,
            kbd,
            aspect,
        }
    }
}
