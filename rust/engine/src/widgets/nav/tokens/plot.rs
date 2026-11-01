//! The plotted half of the navigation slot: the charts (and their meter
//! and labeled bars). The kit draws no chart, so core derives one from
//! its palette and type: hairlines in the rule gray, labels in the type
//! table's text, the series in two of its accents in a fixed order,
//! blue then orange (the pair that passes the categorical checks: the
//! lightness band, the chroma floor, ΔE 30 under protanopia and 36 in
//! normal vision; blue and purple fail at 4.4), the ring in the ink ramp
//! the kit's grays already are; and the graph's tones, its nodes in the
//! same first series. `derive` writes back what the charts and the graph
//! drew with before the slot.

use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::TypeRole;
use crate::styling::theme::TypeRun;
use crate::styling::theme::hoff_core::{BLUE, ORANGE, SHADES};

/// The charts' recipe: every color, width and text style the chart
/// drawers read (`widgets::charts::draw`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartRecipe {
    /// Tick, value and percent labels.
    pub label: TypeRun,
    pub label_color: Color,
    /// The donut's center figure and its legend.
    pub center: TypeRun,
    pub center_color: Color,
    pub legend: TypeRun,
    pub legend_color: Color,
    /// Grid lines (the vertical ones at `grid_v_alpha` of the grid's
    /// alpha) and the bars' baseline.
    pub grid: Color,
    pub grid_v_alpha: f32,
    pub baseline: Color,
    /// The hairline, the series line, a band's edge, a dot's radius.
    pub hair: f32,
    pub line_w: f32,
    pub edge_w: f32,
    pub dot_r: f32,
    /// The categorical order: the first series, the second.
    pub series: [Color; 2],
    /// The wash under the line.
    pub under: Color,
    /// The stacked bands' fills and edges, by series.
    pub band_fill: [Color; 2],
    pub band_edge: [Color; 2],
    /// Every bar, and the tallest one.
    pub bar: Color,
    pub bar_peak: Color,
    /// The ring's slices: one ink at falling alphas; a percent on a slice
    /// at or past `ring_bright` alpha takes `ring_on`.
    pub ring: Color,
    pub ring_alphas: [f32; 5],
    pub ring_bright: f32,
    pub ring_on: Color,
    pub swatch_radius: f32,
    /// The meter: its track and its fill.
    pub meter_track: Color,
    pub meter_fill: Color,
    /// The labeled bars' text, their names' and values' tones, the
    /// space after a name and before a value.
    pub row: TypeRun,
    pub row_name: Color,
    pub row_value: Color,
    pub row_gap: f32,
    /// A plot's height as a fraction of its width, up to `max_h`.
    pub aspect: f32,
    pub max_h: f32,
}

impl ChartRecipe {
    /// The kit's derivation. Labels are the typography table's text
    /// (`TypographyPage:8`, `text-[0.6875rem] leading-[1rem] font-medium
    /// text-[#7B7B7B]`); the donut's figure its h5 (`text-[1.5rem]
    /// leading-[2rem] tracking-[-0.03em] font-medium`); grid and baseline
    /// the rule `#ECECEC` at 1 px; the series line 2 px with 4 px dots;
    /// the meter the video player's track (`VideoPlayer:48-53`,
    /// `bg-[#7B7B7B]/20`) filled in ink.
    pub fn hoff_core() -> Self {
        let [paper, _, _, line, _, gray, dark, _, ink] = SHADES;
        Self {
            label: TypeRole::Caption.run(),
            label_color: gray,
            center: TypeRole::H5.run(),
            center_color: ink,
            legend: TypeRole::Label.run(),
            legend_color: dark,
            grid: line,
            grid_v_alpha: 0.6,
            baseline: line,
            hair: 1.0,
            line_w: 2.0,
            edge_w: 2.0,
            dot_r: 4.0,
            series: [BLUE, ORANGE],
            under: alpha(BLUE, 0.08),
            band_fill: [alpha(BLUE, 0.16), alpha(ORANGE, 0.16)],
            band_edge: [BLUE, ORANGE],
            bar: alpha(BLUE, 0.32),
            bar_peak: BLUE,
            ring: ink,
            ring_alphas: [0.90, 0.62, 0.40, 0.24, 0.12],
            ring_bright: 0.5,
            ring_on: paper,
            swatch_radius: 3.0,
            meter_track: alpha(gray, 0.20),
            meter_fill: ink,
            row: TypeRole::Caption.run(),
            row_name: dark,
            row_value: gray,
            row_gap: 8.0,
            // Derived, no kit source: half as tall as wide, up to 240
            // (`h-60` on tailwind's scale), so a trend reads wider than
            // it is tall.
            aspect: 0.5,
            max_h: 240.0,
        }
    }

    /// The charts of before: the small step for ticks and values in the
    /// faint glass text, the divider grid, the glass edge under the bars,
    /// the accent's line, the success hue on the tallest bar and the
    /// second band's edge, the text tone at fixed alphas for the rest.
    pub fn derive(theme: &Theme) -> Self {
        let (colors, glass, control) = (&theme.colors, &theme.glass, &theme.control);
        let text = colors.text;
        Self {
            label: TypeRole::Small.strong(),
            label_color: glass.text_faint,
            center: TypeRole::Title.run(),
            center_color: text,
            legend: TypeRole::Caption.regular(),
            legend_color: colors.text_mid,
            grid: colors.divider,
            grid_v_alpha: 0.6,
            baseline: glass.edge,
            hair: control.edge_width,
            line_w: control.edge_width * 2.0,
            edge_w: control.edge_width_strong,
            dot_r: control.edge_width_strong * 2.0,
            series: [colors.accent, colors.success],
            under: alpha(colors.accent, 0.08),
            band_fill: [alpha(text, 0.16), alpha(text, 0.30)],
            band_edge: [alpha(text, 0.55), alpha(colors.success, 0.90)],
            bar: alpha(text, 0.28),
            bar_peak: alpha(colors.success, 0.90),
            ring: text,
            ring_alphas: [0.90, 0.62, 0.40, 0.24, 0.12],
            ring_bright: 0.5,
            ring_on: colors.bg,
            swatch_radius: theme.shape.micro / 2.0,
            meter_track: glass.surface_active,
            meter_fill: colors.accent,
            row: TypeRole::Caption.regular(),
            row_name: colors.text_mid,
            row_value: colors.text_dim,
            row_gap: theme.spacing.sm,
            // The chart page's panels of before: 0.58 of the width, up
            // to 300.
            aspect: 0.58,
            max_h: 300.0,
        }
    }
}

/// `c` at alpha `a` (the hue kept, its own alpha replaced).
fn alpha(c: Color, a: f32) -> Color {
    Color([c.0[0], c.0[1], c.0[2], a])
}

/// The graph's tones (adr-0081: the look names a tone, the theme resolves
/// it when the graph draws): what each `Tone` of a node or a group and
/// each `EdgeTone` of an edge is, the alphas of the cool and the hot
/// pass, the ring on a hovered or selected node and the frame around a
/// canvas with a selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphRecipe {
    pub accent: Color,
    pub text: Color,
    pub text_mid: Color,
    pub text_dim: Color,
    pub border_active: Color,
    pub divider: Color,
    pub fill: Color,
    pub panel: Color,
    pub edge_dim: Color,
    pub edge_info: Color,
    pub edge_success: Color,
    pub edge_danger: Color,
    pub edge_cool_alpha: f32,
    pub edge_hot_alpha: f32,
    /// The thinnest an edge draws, however far the view zooms out.
    pub edge_min_w: f32,
    pub node_rest_alpha: f32,
    pub ring: Color,
    pub frame: Color,
    pub frame_radius: f32,
}

impl GraphRecipe {
    /// The kit's derivation: nodes in the charts' first series (blue),
    /// a second kind of edge in their second (orange), the structural
    /// edges in the kit's gray; text in ink and gray, rules `#ECECEC`,
    /// the active border `#E2E2E2`, fills `#F1F1F1` and the panel
    /// `#F8F7F7`; the ring in ink, the frame the card shell's rim at the
    /// card radius (`Card/index.tsx:36`, `rounded-2xl`).
    pub fn hoff_core() -> Self {
        use crate::styling::theme::hoff_core::{GREEN, RED};
        let [_, well, wash, line, active, gray, _, _, ink] = SHADES;
        Self {
            accent: BLUE,
            text: ink,
            text_mid: gray,
            text_dim: gray,
            border_active: active,
            divider: line,
            fill: wash,
            panel: well,
            edge_dim: gray,
            edge_info: ORANGE,
            edge_success: GREEN,
            edge_danger: RED,
            // On the paper a 35% gray hairline all but vanished: the
            // edges at 60% and never under 1.5 px.
            edge_cool_alpha: 0.6,
            edge_hot_alpha: 0.9,
            edge_min_w: 1.5,
            node_rest_alpha: 0.65,
            ring: ink,
            frame: line,
            frame_radius: 16.0,
        }
    }

    /// The graph of before: every tone the theme's color of that name,
    /// the structural edges the glass edge, the frame the glass edge at
    /// the nav radius.
    pub fn derive(theme: &Theme) -> Self {
        let c = &theme.colors;
        Self {
            accent: c.accent,
            text: c.text,
            text_mid: c.text_mid,
            text_dim: c.text_dim,
            border_active: c.border_active,
            divider: c.divider,
            fill: c.bg_hover,
            panel: c.bg_panel,
            edge_dim: theme.glass.edge,
            edge_info: c.info,
            edge_success: c.success,
            edge_danger: c.danger,
            edge_cool_alpha: 0.35,
            edge_hot_alpha: 0.9,
            edge_min_w: 0.5,
            node_rest_alpha: 0.65,
            ring: c.text,
            frame: theme.glass.edge,
            frame_radius: theme.shape.nav,
        }
    }
}
