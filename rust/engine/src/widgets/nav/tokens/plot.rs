//! The plotted half of the navigation slot: the charts (and their meter
//! and labeled bars). The kit draws no chart, so core derives one from
//! its palette and type: hairlines in the rule gray, labels in the type
//! table's text, the series in two of its accents in a fixed order,
//! blue then orange (the pair that passes the categorical checks: the
//! lightness band, the chroma floor, ΔE 30 under protanopia and 36 in
//! normal vision; blue and purple fail at 4.4), the ring in the ink ramp
//! the kit's grays already are. `derive` writes back what the charts
//! drew with before the slot.

use super::NavText;
use crate::styling::color::Color;
use crate::styling::theme::Theme;
use crate::styling::theme::hoff_core::{BLUE, ORANGE, SHADES};

/// The charts' recipe: every color, width and text style the chart
/// drawers read (`widgets::charts::draw`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartRecipe {
    /// Tick, value and percent labels.
    pub label: NavText,
    pub label_color: Color,
    /// The donut's center figure and its legend.
    pub center: NavText,
    pub center_color: Color,
    pub legend: NavText,
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
    pub row: NavText,
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
            label: NavText::new(11.0, 16.0, 500),
            label_color: gray,
            center: NavText::new(24.0, 32.0, 500).tracked(-0.03),
            center_color: ink,
            legend: NavText::new(12.0, 16.0, 500),
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
            row: NavText::new(11.0, 16.0, 500),
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
        let (colors, glass, control, ty) = (
            &theme.colors,
            &theme.glass,
            &theme.control,
            &theme.typography,
        );
        let text = colors.text;
        Self {
            label: NavText::of(&ty.small_sm()),
            label_color: glass.text_faint,
            center: NavText::of(&ty.title()),
            center_color: text,
            legend: NavText::of(&ty.caption_r()),
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
            row: NavText::of(&ty.caption_r()),
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
