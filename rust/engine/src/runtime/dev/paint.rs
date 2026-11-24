//! What every view of the dev mode draws with: one layer, the theme's
//! tokens, and a caption line as the unit of height. Nothing here knows a
//! view; a view asks for a text, a bar or an outline at a place.

use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::runtime::perf::Phase;
use crate::styling::theme::{Theme, TypeRole};
use crate::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};

/// The frame budget at 60 Hz, what a frame's time is read against.
pub(super) const BUDGET_NS: u64 = 16_666_667;

/// How much of its color a mark that is not the one in focus keeps.
const DIM: f32 = 0.45;
/// How much of the text color a track behind a bar takes.
const TRACK: f32 = 0.08;
/// How much of its color the body of a chart's column keeps under its cap.
const BODY: f32 = 0.3;
/// The fewest columns a chart is cut into: a short history draws narrow
/// columns at the right, not a few wide ones.
const CHART_SLOTS: usize = 60;

/// The layer the dev mode draws on, with the theme it draws from.
pub(super) struct Painter<'a> {
    pub c: &'a mut Compositor,
    pub layer: LayerId,
    pub theme: &'a Theme,
}

impl Painter<'_> {
    pub fn caption(&self) -> TextStyle {
        self.theme.typography.run(TypeRole::Caption.regular())
    }

    pub fn strong(&self) -> TextStyle {
        self.theme.typography.run(TypeRole::Caption.strong())
    }

    /// A caption line's height.
    pub fn line(&self) -> f32 {
        self.caption().line_height
    }

    /// A row of a table: a caption line and the gap under it.
    pub fn row(&self) -> f32 {
        self.line() + self.theme.spacing.xs
    }

    pub fn gap(&self) -> f32 {
        self.theme.spacing.sm
    }

    pub fn ink(&self) -> [f32; 4] {
        self.theme.colors.text.0
    }

    pub fn mid(&self) -> [f32; 4] {
        self.theme.colors.text_mid.0
    }

    /// How wide `s` is in `style`.
    pub fn width(&self, s: &str, style: &TextStyle) -> f32 {
        TextMeasurer::measure_styled(s, style, None).0
    }

    /// `s` on one line at `(x, y)`, cut to `room`.
    pub fn text(&mut self, s: &str, x: f32, y: f32, room: f32, color: [f32; 4], style: &TextStyle) {
        if room <= 0.0 || s.is_empty() {
            return;
        }
        let s = TextMeasurer::truncate_to_width(s, style, room);
        let key = TextNodeKey::from_style(&s, style, None);
        self.c
            .push_to_layer(self.layer, SceneNode::Text { key, x, y, color });
    }

    /// `s` ending at `right`, on one line at `y`.
    pub fn text_right(&mut self, s: &str, right: f32, y: f32, color: [f32; 4], style: &TextStyle) {
        let w = self.width(s, style);
        self.text(s, right - w, y, w + 1.0, color, style);
    }

    /// A square-cornered fill.
    pub fn fill(&mut self, r: Rect, color: [f32; 4]) {
        if r.w > 0.0 && r.h > 0.0 {
            let none = self.theme.radius.none;
            self.c
                .push_to_layer(self.layer, rounded_rect(r.x, r.y, r.w, r.h, none, color));
        }
    }

    /// A bar: a fill with the theme's small corners.
    pub fn bar(&mut self, r: Rect, color: [f32; 4]) {
        if r.w > 0.0 && r.h > 0.0 {
            let radius = self.theme.radius.sm.min(r.w / 2.0).min(r.h / 2.0);
            self.c
                .push_to_layer(self.layer, rounded_rect(r.x, r.y, r.w, r.h, radius, color));
        }
    }

    /// The track a bar fills part of.
    pub fn track(&mut self, r: Rect) {
        let color = alpha(self.ink(), TRACK);
        self.bar(r, color);
    }

    /// A ring around `r`, drawn inside it.
    pub fn outline(&mut self, r: Rect, color: [f32; 4]) {
        if r.w > 0.0 && r.h > 0.0 {
            let node = rounded_rect_stroke(
                r.x,
                r.y,
                r.w,
                r.h,
                self.theme.radius.sm,
                color,
                self.theme.spacing.xs / 2.0,
            );
            self.c.push_to_layer(self.layer, node);
        }
    }

    /// `values` as columns in `r`, oldest at the left, filling it from the
    /// right as a history does: each a faint body with a cap in `color`,
    /// measured from zero (a value below zero hangs under the zero line).
    /// The column at `marked` is drawn in the text color. More values than
    /// `r` has room for keeps the latest.
    pub fn chart(&mut self, r: Rect, values: &[f64], color: [f32; 4], marked: Option<usize>) {
        self.track(r);
        let cap = self.theme.spacing.xs / 2.0;
        let room = (r.w / cap).floor().max(1.0) as usize;
        let skip = values.len().saturating_sub(room);
        let values = &values[skip..];
        let marked = marked.and_then(|i| i.checked_sub(skip));
        let slots = values.len().max(CHART_SLOTS).min(room);
        let slot = r.w / slots as f32;
        let low = values.iter().copied().fold(0.0, f64::min);
        let high = values.iter().copied().fold(0.0, f64::max);
        let span = (high - low).max(f64::MIN_POSITIVE);
        let at = |v: f64| r.bottom() - ((v - low) / span) as f32 * r.h;
        let zero = at(0.0);
        let first = slots - values.len();
        for (i, &value) in values.iter().enumerate() {
            let x = r.x + (first + i) as f32 * slot;
            let w = (slot - 1.0).max(1.0);
            let top = at(value).min(zero);
            let h = (at(value) - zero).abs();
            let ink = if marked == Some(i) { self.ink() } else { color };
            self.fill(Rect::new(x, top, w, h), alpha(ink, BODY));
            let edge = if value < 0.0 { top + h - cap } else { top };
            self.fill(Rect::new(x, edge.max(r.y), w, cap), ink);
        }
    }

    /// A chart with its words, in `r`: `name` over it with `value` at the
    /// right, `range` under it. Nothing in a place too short for the
    /// three lines.
    #[allow(clippy::too_many_arguments)]
    pub fn plot(
        &mut self,
        r: Rect,
        name: &str,
        value: &str,
        range: &str,
        values: &[f64],
        color: [f32; 4],
        marked: Option<usize>,
    ) {
        let row = self.row();
        if r.h < row * 3.0 {
            return;
        }
        let (caption, strong) = (self.caption(), self.strong());
        let value_w = self.width(value, &caption);
        self.text(
            name,
            r.x,
            r.y,
            r.w - value_w - self.gap(),
            self.mid(),
            &strong,
        );
        self.text_right(value, r.right(), r.y, self.ink(), &caption);
        let chart = Rect::new(r.x, r.y + row, r.w, r.h - row * 2.0);
        self.chart(chart, values, color, marked);
        let under = r.bottom() - self.line();
        self.text(range, r.x, under, r.w, self.mid(), &caption);
    }

    /// The color a phase of the frame is drawn in, everywhere it shows.
    pub fn phase_color(&self, phase: Phase) -> [f32; 4] {
        let colors = &self.theme.colors;
        match phase {
            Phase::Input => colors.text_mid.0,
            Phase::Tick => colors.text_dim.0,
            Phase::Scene => colors.accent.0,
            Phase::Resolve => colors.info.0,
            Phase::Encode => colors.success.0,
            Phase::Gpu => colors.warning.0,
        }
    }
}

/// `color` at `a` of its opacity.
pub(super) fn alpha(color: [f32; 4], a: f32) -> [f32; 4] {
    [color[0], color[1], color[2], color[3] * a]
}

/// `color` as a mark out of focus.
pub(super) fn dim(color: [f32; 4]) -> [f32; 4] {
    alpha(color, DIM)
}

/// A time for a table: milliseconds from a tenth of one up, microseconds
/// under that.
pub(super) fn time(nanos: u64) -> String {
    if nanos >= 100_000 {
        format!("{:.2} ms", nanos as f64 / 1e6)
    } else {
        format!("{} us", (nanos + 500) / 1_000)
    }
}

/// A byte count for a table.
pub(super) fn bytes(n: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * KB;
    if n >= MB {
        format!("{:.1} MB", n as f64 / MB as f64)
    } else if n >= KB {
        format!("{:.1} KB", n as f64 / KB as f64)
    } else {
        format!("{n} B")
    }
}

/// A count for a table, its thousands apart.
pub(super) fn count(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// A signed byte count: what a frame added to the heap or took from it.
pub(super) fn signed_bytes(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "+" };
    format!("{sign}{}", bytes(n.unsigned_abs()))
}

/// `part` of `whole` as a whole percent; zero of nothing.
pub(super) fn percent(part: u64, whole: u64) -> u64 {
    (part * 100 + whole / 2).checked_div(whole).unwrap_or(0)
}
