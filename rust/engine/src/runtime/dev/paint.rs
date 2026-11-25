//! What every view of the dev mode draws with: one layer, the theme's
//! tokens, and a caption line as the unit of height. Nothing here knows a
//! view; a view asks for a text, an icon, a bar or a table row at a place.
//!
//! The panel is dense on purpose, as a tool is: the caption size for
//! words, the same size with tabular figures for numbers (so a column of
//! them lines up and a counting value does not shake its neighbours), and
//! the mono face for what is code (an id, a source position).
//!
//! A view also says where a press does something ([`Painter::region`]):
//! the place and the [`Act`] go into one list the dev mode reads on the
//! next pointer event, and the region the pointer is over is drawn hot.

use super::Act;
use crate::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use crate::drawing::text::{TextFeatures, TextMeasurer, TextStyle};
use crate::runtime::perf::{Level, Phase};
use crate::styling::theme::{Theme, TypeRole};
use crate::widgets::icons;
use crate::widgets::prelude::{Rect, rounded_rect, rounded_rect_stroke};
use crate::widgets::widget::mix;

/// The frame budget at 60 Hz, what a frame's time is read against.
pub(super) const BUDGET_NS: u64 = 16_666_667;

/// How much of its color a mark that is not the one in focus keeps.
const DIM: f32 = 0.45;
/// How much of the text color a track behind a bar takes.
const TRACK: f32 = 0.06;
/// How much of the text color the wash under a hot row takes.
const WASH: f32 = 0.05;
/// How much of its color the body of a chart's column keeps under its cap.
const BODY: f32 = 0.28;
/// How much of its color the ground of a badge keeps.
const TINT: f32 = 0.14;
/// The fewest columns a chart is cut into: a short history draws narrow
/// columns at the right, not a few wide ones.
const CHART_SLOTS: usize = 60;
/// An icon beside a caption line.
pub(super) const ICON: f32 = 14.0;

/// How a frame's time stands against the budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Health {
    Good,
    Tight,
    Over,
}

impl Health {
    /// `nanos` of a frame against the budget: over it, past three fifths
    /// of it, or well inside.
    pub fn of(nanos: u64) -> Health {
        if nanos > BUDGET_NS {
            Health::Over
        } else if nanos * 5 > BUDGET_NS * 3 {
            Health::Tight
        } else {
            Health::Good
        }
    }
}

/// The layer the dev mode draws on, with the theme it draws from, the
/// regions the frame offers the pointer and the one it is over.
pub(super) struct Painter<'a> {
    pub c: &'a mut Compositor,
    pub layer: LayerId,
    pub theme: &'a Theme,
    pub regions: &'a mut Vec<(Rect, Act)>,
    pub hot: Option<Act>,
}

impl Painter<'_> {
    pub fn caption(&self) -> TextStyle {
        self.theme.typography.run(TypeRole::Caption.regular())
    }

    pub fn strong(&self) -> TextStyle {
        self.theme.typography.run(TypeRole::Caption.strong())
    }

    /// The caption with tabular figures: every number of the panel.
    pub fn num(&self) -> TextStyle {
        self.caption().with_features(TextFeatures::TABULAR)
    }

    pub fn num_strong(&self) -> TextStyle {
        self.strong().with_features(TextFeatures::TABULAR)
    }

    /// The mono face at the caption's size: an id, a source position.
    pub fn mono(&self) -> TextStyle {
        let caption = self.caption();
        let mut code = self.theme.typography.run(TypeRole::Code.regular());
        code.font_size = caption.font_size;
        code.line_height = caption.line_height;
        code
    }

    /// The smallest words: an axis, a unit under a number.
    pub fn small(&self) -> TextStyle {
        self.theme
            .typography
            .run(TypeRole::Small.regular())
            .with_features(TextFeatures::TABULAR)
    }

    /// A caption line's height.
    pub fn line(&self) -> f32 {
        self.caption().line_height
    }

    /// A row of a table: a caption line and the room around it.
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

    pub fn faint(&self) -> [f32; 4] {
        self.theme.colors.text_dim.0
    }

    pub fn rule(&self) -> [f32; 4] {
        self.theme.colors.divider.0
    }

    /// The color a level is said in.
    pub fn level_color(&self, level: Level) -> [f32; 4] {
        match level {
            Level::Info => self.theme.colors.info.0,
            Level::Warn => self.theme.colors.warning.0,
            Level::Error => self.theme.colors.danger.0,
        }
    }

    /// The icon a level is said with.
    pub fn level_icon(level: Level) -> &'static str {
        match level {
            Level::Info => "info",
            Level::Warn => "alert-triangle",
            Level::Error => "circle-x",
        }
    }

    pub fn health_color(&self, health: Health) -> [f32; 4] {
        match health {
            Health::Good => self.theme.colors.success.0,
            Health::Tight => self.theme.colors.warning.0,
            Health::Over => self.theme.colors.danger.0,
        }
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
        self.c.push_to_layer(
            self.layer,
            SceneNode::Text {
                key,
                x: x.round(),
                y: y.round(),
                color,
            },
        );
    }

    /// `s` in lines no wider than `w`, from `(x, y)` down; how tall it
    /// came out.
    pub fn text_wrapped(
        &mut self,
        s: &str,
        x: f32,
        y: f32,
        w: f32,
        color: [f32; 4],
        style: &TextStyle,
    ) -> f32 {
        if w <= 0.0 || s.is_empty() {
            return 0.0;
        }
        let key = TextNodeKey::from_style(s, style, Some(w));
        self.c.push_to_layer(
            self.layer,
            SceneNode::Text {
                key,
                x: x.round(),
                y: y.round(),
                color,
            },
        );
        self.wrapped_height(s, w, style)
    }

    /// How tall `s` is in lines no wider than `w`; nothing where there
    /// is no width to wrap in, as [`Painter::text_wrapped`] draws nothing.
    pub fn wrapped_height(&self, s: &str, w: f32, style: &TextStyle) -> f32 {
        if w <= 0.0 || s.is_empty() {
            return 0.0;
        }
        TextMeasurer::measure_styled(s, style, Some(w)).1
    }

    /// Draw `draw` cut to `r`: nothing of it shows outside.
    pub fn clipped(&mut self, r: Rect, draw: impl FnOnce(&mut Self)) {
        let clip = SceneNode::PushClip {
            x: r.x,
            y: r.y,
            w: r.w.max(0.0),
            h: r.h.max(0.0),
        };
        self.c.push_to_layer(self.layer, clip);
        draw(self);
        self.c.push_to_layer(self.layer, SceneNode::PopClip);
    }

    /// `s` ending at `right`, on one line at `y`.
    pub fn text_right(&mut self, s: &str, right: f32, y: f32, color: [f32; 4], style: &TextStyle) {
        let w = self.width(s, style);
        self.text(s, right - w, y, w + 1.0, color, style);
    }

    /// `s` on one line of `r`, centered both ways, cut to it.
    pub fn text_in(&mut self, s: &str, r: Rect, color: [f32; 4], style: &TextStyle) {
        let w = self.width(s, style).min(r.w);
        let y = r.y + (r.h - style.line_height) / 2.0;
        self.text(s, r.x + (r.w - w) / 2.0, y, r.w, color, style);
    }

    /// The icon `name`, `size` on a side, its top-left at `(x, y)`.
    pub fn icon(&mut self, name: &str, x: f32, y: f32, size: f32, color: [f32; 4]) {
        if let Some(node) = icons::icon_at(name, size, color, x.round(), y.round()) {
            self.c.push_to_layer(self.layer, node);
        }
    }

    /// The icon `name` in the middle of `r`.
    pub fn icon_in(&mut self, name: &str, r: Rect, size: f32, color: [f32; 4]) {
        self.icon(
            name,
            r.x + (r.w - size) / 2.0,
            r.y + (r.h - size) / 2.0,
            size,
            color,
        );
    }

    fn rounded(&mut self, r: Rect, radius: f32, color: [f32; 4]) {
        if r.w > 0.0 && r.h > 0.0 {
            let radius = radius.min(r.w / 2.0).min(r.h / 2.0);
            self.c
                .push_to_layer(self.layer, rounded_rect(r.x, r.y, r.w, r.h, radius, color));
        }
    }

    /// A square-cornered fill.
    pub fn fill(&mut self, r: Rect, color: [f32; 4]) {
        self.rounded(r, self.theme.radius.none, color);
    }

    /// A bar: a fill with the theme's small corners.
    pub fn bar(&mut self, r: Rect, color: [f32; 4]) {
        self.rounded(r, self.theme.radius.sm / 2.0, color);
    }

    /// A fill with the theme's small corners whole: a chip, a hot button.
    pub fn chip(&mut self, r: Rect, color: [f32; 4]) {
        self.rounded(r, self.theme.radius.sm, color);
    }

    /// A fill round at its ends: a dot, a badge.
    pub fn pill(&mut self, r: Rect, color: [f32; 4]) {
        self.rounded(r, r.h / 2.0, color);
    }

    /// A one pixel line along the top of `r`'s width, or down its height.
    pub fn rule_h(&mut self, x: f32, y: f32, w: f32) {
        let color = self.rule();
        self.fill(Rect::new(x, y.round(), w, 1.0), color);
    }

    pub fn rule_v(&mut self, x: f32, y: f32, h: f32) {
        let color = self.rule();
        self.fill(Rect::new(x.round(), y, 1.0, h), color);
    }

    /// The track a bar fills part of.
    pub fn track(&mut self, r: Rect) {
        let color = alpha(self.ink(), TRACK);
        self.bar(r, color);
    }

    /// The wash under a row the pointer is over, or one chosen.
    pub fn wash(&mut self, r: Rect, strength: f32) {
        let color = alpha(self.ink(), WASH * strength);
        self.fill(r, color);
    }

    /// A ring around `r`, drawn inside it.
    pub fn outline(&mut self, r: Rect, color: [f32; 4], width: f32) {
        if r.w > 0.0 && r.h > 0.0 {
            let node = rounded_rect_stroke(
                r.x,
                r.y,
                r.w,
                r.h,
                self.theme.radius.sm.min(r.h / 2.0),
                color,
                width,
            );
            self.c.push_to_layer(self.layer, node);
        }
    }

    /// Offer `r` to the pointer for `act`; `true` when the pointer is on
    /// it now.
    pub fn region(&mut self, r: Rect, act: Act) -> bool {
        let hot = self.hot.as_ref() == Some(&act);
        self.regions.push((r, act));
        hot
    }

    /// `text` on a tinted ground, its left at `x` on the caption line at
    /// `y`; how wide it came out.
    pub fn badge(&mut self, text: &str, x: f32, y: f32, color: [f32; 4]) -> f32 {
        let style = self.small();
        let pad = self.theme.spacing.xs + 2.0;
        let w = self.width(text, &style) + pad * 2.0;
        let h = style.line_height + 2.0;
        let r = Rect::new(x, y + (self.line() - h) / 2.0, w, h);
        self.pill(r, alpha(color, TINT));
        let ink = mix(color, self.ink(), 0.35);
        self.text_in(text, r, ink, &style);
        w
    }

    /// The head of a part of a view: its name in small capitals' place
    /// (strong, dimmer), and `note` at the right. Returns the y under it.
    pub fn title(&mut self, name: &str, note: &str, r: Rect) -> f32 {
        let (strong, num) = (self.strong(), self.num());
        let note_w = self.width(note, &num);
        self.text(
            name,
            r.x,
            r.y,
            r.w - note_w - self.gap(),
            self.ink(),
            &strong,
        );
        self.text_right(note, r.right(), r.y, self.mid(), &num);
        r.y + self.row()
    }

    /// A view with nothing to show yet: an icon and why, in the middle
    /// of `r`.
    pub fn empty(&mut self, r: Rect, icon: &str, why: &str) {
        let caption = self.caption();
        let size = ICON * 1.5;
        let w = self.width(why, &caption).min(r.w);
        let top = r.y + ((r.h - size - self.gap() - self.line()) / 2.0).max(0.0);
        self.icon(icon, r.x + (r.w - size) / 2.0, top, size, self.faint());
        let y = top + size + self.gap();
        self.text(why, r.x + (r.w - w) / 2.0, y, r.w, self.mid(), &caption);
    }

    /// The ground of a table's head in `r`, with a line under it.
    pub fn table_head(&mut self, r: Rect) {
        self.fill(r, self.theme.colors.bg_panel.0);
        self.rule_h(r.x, r.bottom() - 1.0, r.w);
    }

    /// `values` as columns in `r`, oldest at the left, filling it from the
    /// right as a history does: each a faint body with a cap in `color`,
    /// measured from zero (a value below zero hangs under the zero line).
    /// The column at `marked` is drawn in the text color. More values than
    /// `r` has room for keeps the latest.
    pub fn chart(&mut self, r: Rect, values: &[f64], color: [f32; 4], marked: Option<usize>) {
        self.track(r);
        let cap = (self.theme.spacing.xs / 2.0).max(1.0);
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
        let (caption, num, small) = (self.caption(), self.num_strong(), self.small());
        let value_w = self.width(value, &num);
        let dot = (self.line() / 2.0).round();
        self.pill(
            Rect::new(r.x, r.y + (self.line() - dot) / 2.0, dot, dot),
            color,
        );
        let name_x = r.x + dot + self.gap() / 2.0;
        self.text(
            name,
            name_x,
            r.y,
            r.right() - name_x - value_w - self.gap(),
            self.mid(),
            &caption,
        );
        self.text_right(value, r.right(), r.y, self.ink(), &num);
        let chart = Rect::new(r.x, r.y + row, r.w, r.h - row - small.line_height - 2.0);
        self.chart(chart, values, color, marked);
        let under = r.bottom() - small.line_height;
        self.text(range, r.x, under, r.w, self.faint(), &small);
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

    /// One of the theme's telling colors for `name`, the same every
    /// time: what sets one zone apart from the next in a timeline.
    pub fn name_color(&self, name: &str) -> [f32; 4] {
        let colors = &self.theme.colors;
        let palette = [
            colors.info.0,
            colors.success.0,
            colors.warning.0,
            colors.danger.0,
            mix(colors.info.0, colors.danger.0, 0.5),
            mix(colors.success.0, colors.info.0, 0.5),
        ];
        let hash = name
            .bytes()
            .fold(0u32, |h, b| h.wrapping_mul(31).wrapping_add(u32::from(b)));
        palette[hash as usize % palette.len()]
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
