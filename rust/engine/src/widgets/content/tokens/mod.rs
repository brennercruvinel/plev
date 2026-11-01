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
//! The rows of a list and a table live here too (`list`, `table`): the
//! kit has no list nor table widget, so the core takes the kit's language
//! (its row, its one table), and `derive` is the look every other theme
//! drew before the kit, so no app outside the core theme changes.
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

/// The kit's slides and the bar that pages through them: Variations'
/// square tiles (a ring on the hovered and the chosen one, a "k of N"
/// counter) and ViewController's pill (two arrows, a tick per page).
#[derive(Clone, Debug, PartialEq)]
pub struct CarouselRecipe {
    /// A slide's least edge: as many fit in a row as the width allows.
    pub slide_min: f32,
    pub gap: f32,
    pub radius: f32,
    pub fill: Color,
    pub glyph: f32,
    pub glyph_color: Color,
    /// The rings: the outer's width and color, the inner's.
    pub ring_outer: (f32, Color),
    pub ring_inner: (f32, Color),
    /// The counter: size, line height, color; the space under it.
    pub counter_size: f32,
    pub counter_line_height: f32,
    pub counter: Color,
    pub counter_gap: f32,
    /// The bar: height, padding x, gap, fill, border, css shadow list,
    /// the space above it.
    pub bar_h: f32,
    pub bar_pad: f32,
    pub bar_gap: f32,
    pub bar_fill: Color,
    pub bar_border: Color,
    pub bar_shadow: Vec<ShadowSpec>,
    pub bar_space: f32,
    /// An arrow's edge, and its color where it leads somewhere or not.
    pub arrow: f32,
    pub arrow_on: Color,
    pub arrow_off: Color,
    /// A tick: width, height at rest and current, the gap, the colors.
    pub tick_w: f32,
    pub tick_h: f32,
    pub tick_h_on: f32,
    pub tick_gap: f32,
    pub tick_on: Color,
    pub tick_off: Color,
}

/// Page numbers in cells, derived from the kit's language (it has a
/// counter, not a pager): the Dropdown's `size-8 rounded-lg` cell, a wash
/// on hover, its current look on the current page, the 12 px semibold
/// label, the ViewController's arrows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PagerRecipe {
    pub cell: f32,
    pub radius: f32,
    pub gap: f32,
    pub border: f32,
    pub rest: DisclosureSurface,
    pub hover: DisclosureSurface,
    pub current: DisclosureSurface,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub label: Color,
    pub label_dim: Color,
    pub arrow: f32,
}

/// A row of a list or of a table: the surface under it hovered and
/// selected, the radius of that surface and of the focus ring drawn in
/// it, and the rule between two rows (width 0 is none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RowRecipe {
    pub radius: f32,
    pub border: f32,
    pub hover: DisclosureSurface,
    pub selected: DisclosureSurface,
    pub ring_radius: f32,
    pub rule: f32,
    pub rule_color: Color,
}

/// A table, derived from the one table the kit writes (the spec sheet of
/// its typography page): the frame around it, the header band and its
/// titles, the rule under the header, the cells' padding, text and two
/// tones (the lead column and the others), and its rows.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TableRecipe {
    pub frame: f32,
    pub frame_color: Color,
    pub header_fill: Color,
    pub header_radius: f32,
    pub header_size: f32,
    pub header_line_height: f32,
    pub header_weight: u16,
    pub header_text: Color,
    pub rule: f32,
    pub rule_color: Color,
    pub cell_pad: f32,
    pub cell_size: f32,
    pub cell_line_height: f32,
    pub cell_weight: u16,
    pub cell: Color,
    pub cell_dim: Color,
    pub rows: RowRecipe,
}

/// The icon tile a tree row's icon sits in (the kit's `size-8
/// rounded-lg` square), at rest and selected.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeTile {
    pub size: f32,
    pub radius: f32,
    pub rest: Color,
    pub selected: Color,
    pub shadow_selected: ShadowSpec,
}

/// A tree's row: its height, the padding before the first glyph, the
/// indent per depth, the chevron and the gap after it, the icon (in a
/// tile, for a look that has one) and the gap before the label, the
/// surface under a hovered or selected row (inset by `seam` x and y so
/// neighbors keep a seam), the label's text and its two tones, the
/// chevron's and the icon's colors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TreeRecipe {
    pub row_h: f32,
    pub pad_x: f32,
    pub indent: f32,
    pub chevron: f32,
    pub chevron_gap: f32,
    pub icon: f32,
    pub tile: Option<TreeTile>,
    pub label_gap: f32,
    pub radius: f32,
    pub border: f32,
    pub seam: [f32; 2],
    pub hover: DisclosureSurface,
    pub selected: DisclosureSurface,
    pub font_size: f32,
    pub line_height: f32,
    pub font_weight: u16,
    pub label: Color,
    pub label_selected: Color,
    pub chevron_color: Color,
    pub icon_color: Color,
    pub icon_selected: Color,
}

/// A panel: its radius, the padding around its content, and its group
/// label (size, line height, weight, color).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelRecipe {
    pub radius: f32,
    pub pad: f32,
    pub label_size: f32,
    pub label_line_height: f32,
    pub label_weight: u16,
    pub label: Color,
}

/// A head: padding left and right and above and below, the title row's
/// height (the title, the badge and the trailing actions centered in it
/// and its padding), the space between that row and the blurb; the
/// title's and the blurb's text; the gap before the badge; the rule under
/// the head (width 0 is none).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeaderRecipe {
    pub pad_x: f32,
    pub pad_y: f32,
    pub row: f32,
    pub blurb_gap: f32,
    pub title_size: f32,
    pub title_line_height: f32,
    pub title_weight: u16,
    pub title: Color,
    pub blurb_size: f32,
    pub blurb_line_height: f32,
    pub blurb_weight: u16,
    pub blurb: Color,
    pub badge_gap: f32,
    pub rule: f32,
    pub rule_color: Color,
}

/// A page's head, the one the shell draws: its look narrow and wide, the
/// head width from which the wide one shows, and whether it spans the
/// page column (its rule edge to edge, its title on the page's gutter)
/// rather than sitting inside the gutter.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageHeadRecipe {
    pub narrow: HeaderRecipe,
    pub wide: HeaderRecipe,
    pub wide_from: f32,
    pub bleed: bool,
}

impl PageHeadRecipe {
    /// The look of a head `width` wide.
    pub fn at(&self, width: f32) -> HeaderRecipe {
        if width >= self.wide_from {
            self.wide
        } else {
            self.narrow
        }
    }
}

/// A split's divider: its color at rest and while hovered or dragged.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitRecipe {
    pub rest: Color,
    pub active: Color,
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
    /// The carousel's slides and bar.
    pub carousel: CarouselRecipe,
    /// The pager's cells.
    pub pager: PagerRecipe,
    /// A virtual list's rows.
    pub list: RowRecipe,
    /// A table.
    pub table: TableRecipe,
    /// A tree's rows.
    pub tree: TreeRecipe,
    /// A panel.
    pub panel: PanelRecipe,
    /// A panel's head.
    pub header: HeaderRecipe,
    /// A page's head.
    pub page: PageHeadRecipe,
    /// A card's chart and legend marks.
    pub card: CardRecipe,
    /// The kit's catalog cards (its CardsPage).
    pub catalog: CatalogTokens,
    /// A split's divider.
    pub split: SplitRecipe,
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

mod card;
mod catalog;
mod derive;
mod kit;

pub use card::CardRecipe;
pub use catalog::{
    CatalogPalette, CatalogTokens, ExploreRecipe, PriceRecipe, TileRecipe, TypeSpec,
};
