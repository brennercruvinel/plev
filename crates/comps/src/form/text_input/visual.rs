//! One line of text as it reads on screen: a caret stop at every grapheme
//! boundary, in screen order, built from the shaped clusters
//! ([`TextMeasurer::clusters_styled`]). The arrow keys step to the
//! neighbouring stop, the caret draws at a stop's x, a click lands on the
//! nearest stop, and a logical selection draws as the boxes it covers.
//!
//! Where two directions meet, one byte offset touches two screen edges
//! (in `abc |אבג def` the end of the space and the right edge of `א`).
//! The offset takes the edge on the side of the lower bidi level, the
//! paragraph's side: the caret stays after the space, where a Latin
//! letter typed there appears. Then every offset has one stop and every
//! stop one offset. In Latin text the stops are the byte order and every
//! x is the one [`TextMeasurer::cursor_x_styled`] gives.

use engine::text::{ClusterBox, TextMeasurer, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

/// A caret stop: a byte offset, where the caret draws, and its place in
/// screen order (`k` is the left edge of the k-th cluster from the left,
/// `k + f` a boundary a share `f` into it).
#[derive(Clone, Copy, Debug)]
struct Stop {
    pos: usize,
    x: f32,
    order: f32,
}

impl Stop {
    /// Left of `other` on screen; offsets break a tie so the order is total.
    fn before(&self, other: &Stop) -> bool {
        self.order < other.order || (self.order == other.order && self.pos < other.pos)
    }
}

/// A single line of text laid out for the caret (see the module docs).
pub(crate) struct VisualLine<'a> {
    text: &'a str,
    clusters: Vec<ClusterBox>,
    /// Indices into `clusters` in text order (by first byte).
    by_start: Vec<usize>,
    /// Every grapheme boundary, left to right.
    stops: Vec<Stop>,
}

impl<'a> VisualLine<'a> {
    /// Lay `text` out with `style`, unwrapped. `width` is the box the text
    /// is drawn in, when the owner draws it with one: a right-to-left
    /// paragraph hangs from the box's right edge, as the renderer aligns
    /// it.
    pub(crate) fn new(text: &'a str, style: &TextStyle, width: Option<f32>) -> Self {
        let mut clusters = TextMeasurer::clusters_styled(text, style, None);
        let rtl = clusters.first().is_some_and(|c| c.paragraph_rtl);
        if rtl && let Some(width) = width {
            let line_w = clusters.iter().fold(0.0_f32, |w, c| w.max(c.x + c.w));
            let shift = (width - line_w).max(0.0);
            for c in &mut clusters {
                c.x += shift;
            }
        }
        let mut by_start: Vec<usize> = (0..clusters.len()).collect();
        by_start.sort_by_key(|&k| clusters[k].start);
        let mut line = Self {
            text,
            clusters,
            by_start,
            stops: Vec::new(),
        };
        let mut stops: Vec<Stop> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()))
            .map(|pos| line.stop(pos))
            .collect();
        stops.sort_by(|a, b| a.order.total_cmp(&b.order).then(a.pos.cmp(&b.pos)));
        line.stops = stops;
        line
    }

    /// Where the caret for byte offset `pos` draws, px from the origin.
    pub(crate) fn caret_x(&self, pos: usize) -> f32 {
        self.stop(pos).x
    }

    /// The offset one stop left of `pos` on screen; `None` at the left end.
    pub(crate) fn left_of(&self, pos: usize) -> Option<usize> {
        let here = self.stop(pos);
        self.stops
            .iter()
            .rev()
            .find(|s| s.before(&here))
            .map(|s| s.pos)
    }

    /// The offset one stop right of `pos` on screen; `None` at the right end.
    pub(crate) fn right_of(&self, pos: usize) -> Option<usize> {
        let here = self.stop(pos);
        self.stops.iter().find(|s| here.before(s)).map(|s| s.pos)
    }

    /// The offset whose stop is nearest to `x`; halfway between two stops
    /// goes to the right one, as a click on a glyph's right half does.
    pub(crate) fn hit(&self, x: f32) -> usize {
        let mut best: Option<(f32, usize)> = None;
        for s in &self.stops {
            let d = (s.x - x).abs();
            if best.is_none_or(|(bd, _)| d <= bd) {
                best = Some((d, s.pos));
            }
        }
        best.map_or(0, |(_, pos)| pos)
    }

    /// The logical range `lo..hi` as the boxes it covers, `(x, w)` left to
    /// right; boxes that touch merge, so Latin text gives one box.
    pub(crate) fn boxes(&self, lo: usize, hi: usize) -> Vec<(f32, f32)> {
        let mut out: Vec<(f32, f32)> = Vec::new();
        for c in &self.clusters {
            let (s, e) = (lo.max(c.start), hi.min(c.end));
            if s >= e {
                continue;
            }
            let (x, w) = if s == c.start && e == c.end {
                (c.x, c.w)
            } else {
                let (f0, f1) = (self.share(c, s), self.share(c, e));
                let (f0, f1) = if c.is_rtl() {
                    (1.0 - f1, 1.0 - f0)
                } else {
                    (f0, f1)
                };
                (c.x + c.w * f0, c.w * (f1 - f0))
            };
            match out.last_mut() {
                Some(last) if (last.0 + last.1 - x).abs() < 0.01 => last.1 = x + w - last.0,
                _ => out.push((x, w)),
            }
        }
        out.retain(|&(_, w)| w > 0.0);
        out
    }

    /// The stop of byte offset `pos`: inside a cluster (a ligature over
    /// several graphemes) a share of its width; on a cluster edge, the
    /// edge of the lower-level neighbour (the module docs).
    fn stop(&self, pos: usize) -> Stop {
        let n = self
            .by_start
            .partition_point(|&k| self.clusters[k].start < pos);
        // The last cluster starting before `pos`, and the one starting at it.
        let prev = n.checked_sub(1).map(|i| self.by_start[i]);
        let next = self.by_start.get(n).copied();
        if let Some(k) = prev
            && pos < self.clusters[k].end
        {
            let c = &self.clusters[k];
            let f = self.share(c, pos);
            let f = if c.is_rtl() { 1.0 - f } else { f };
            return Stop {
                pos,
                x: c.x + c.w * f,
                order: k as f32 + f,
            };
        }
        let before = prev.filter(|&k| self.clusters[k].end == pos);
        let after = next.filter(|&k| self.clusters[k].start == pos);
        let (x, order) = match (before, after) {
            (Some(b), Some(a)) if self.clusters[b].level < self.clusters[a].level => {
                self.trailing(b)
            }
            (_, Some(a)) => self.leading(a),
            (Some(b), None) => self.trailing(b),
            // Bytes no glyph covers (a line break): the nearest cluster
            // before in text order, else the first after.
            (None, None) => match (prev, next) {
                (Some(b), _) => self.trailing(b),
                (None, Some(a)) => self.leading(a),
                (None, None) => (0.0, 0.0),
            },
        };
        Stop { pos, x, order }
    }

    /// `(x, order)` of the edge where cluster `k` begins in text order.
    fn leading(&self, k: usize) -> (f32, f32) {
        let c = &self.clusters[k];
        if c.is_rtl() {
            (c.x + c.w, k as f32 + 1.0)
        } else {
            (c.x, k as f32)
        }
    }

    /// `(x, order)` of the edge where cluster `k` ends in text order.
    fn trailing(&self, k: usize) -> (f32, f32) {
        let c = &self.clusters[k];
        if c.is_rtl() {
            (c.x, k as f32)
        } else {
            (c.x + c.w, k as f32 + 1.0)
        }
    }

    /// Share of cluster `c`'s graphemes that come before `pos`.
    fn share(&self, c: &ClusterBox, pos: usize) -> f32 {
        let total = self.text[c.start..c.end].graphemes(true).count().max(1);
        let before = self.text[c.start..pos.clamp(c.start, c.end)]
            .graphemes(true)
            .count();
        before as f32 / total as f32
    }
}
