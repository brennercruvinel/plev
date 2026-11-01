//! A text laid out in a width, line by line, for the caret: each `\n`
//! starts a paragraph and each paragraph wraps where the shaper wraps it
//! ([`TextMeasurer::clusters_styled`] with the width), so the lines here
//! are the lines the renderer draws. A line is a byte range of the text,
//! without its `\n`; an empty paragraph is one empty line. Inside a line
//! the caret, a click and a selection are its [`VisualLine`]'s, so mixed
//! directions read as they do in a single-line field.
//!
//! Where a paragraph wraps, the end of a line is the start of the next:
//! that offset belongs to the next line, where a typed letter appears.

use crate::drawing::text::{TextMeasurer, TextStyle};
use crate::widgets::form::text_input::visual::VisualLine;

/// One line: its bytes and whether a `\n` ends it (the last line of a
/// paragraph) or a wrap does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

/// A text's lines in a width.
pub(crate) struct Lines<'a> {
    text: &'a str,
    style: &'a TextStyle,
    width: f32,
    pub(crate) lines: Vec<Line>,
}

impl<'a> Lines<'a> {
    pub(crate) fn new(text: &'a str, style: &'a TextStyle, width: f32) -> Self {
        let mut lines = Vec::new();
        let mut at = 0;
        for para in text.split('\n') {
            let clusters = TextMeasurer::clusters_styled(para, style, Some(width.max(1.0)));
            let first = lines.len();
            let mut top = None;
            for c in &clusters {
                if top != Some(c.line_top) {
                    top = Some(c.line_top);
                    lines.push(Line {
                        start: at + c.start,
                        end: at + c.end,
                    });
                }
                if let Some(l) = lines.last_mut() {
                    l.start = l.start.min(at + c.start);
                    l.end = l.end.max(at + c.end);
                }
            }
            if lines.len() == first {
                lines.push(Line { start: at, end: at });
            }
            // The lines of a paragraph touch: each ends where the next
            // starts (the space a wrap hangs, which has no cluster, stays
            // on the line before), the first starts at the paragraph's
            // start, the last ends at its end.
            lines[first].start = at;
            for i in first + 1..lines.len() {
                lines[i - 1].end = lines[i].start;
            }
            if let Some(l) = lines.last_mut() {
                l.end = at + para.len();
            }
            at += para.len() + 1;
        }
        Self {
            text,
            style,
            width,
            lines,
        }
    }

    pub(crate) fn count(&self) -> usize {
        self.lines.len()
    }

    /// The line the offset `pos` is on (at a wrap, the later line).
    pub(crate) fn line_of(&self, pos: usize) -> usize {
        self.lines.iter().rposition(|l| l.start <= pos).unwrap_or(0)
    }

    fn visual(&self, i: usize) -> VisualLine<'a> {
        let l = self.lines[i];
        VisualLine::new(&self.text[l.start..l.end], self.style, Some(self.width))
    }

    /// Where the caret at `pos` draws: its x from the text's left and its
    /// line.
    pub(crate) fn caret(&self, pos: usize) -> (f32, usize) {
        let i = self.line_of(pos);
        let l = self.lines[i];
        (
            self.visual(i).caret_x(pos.clamp(l.start, l.end) - l.start),
            i,
        )
    }

    /// The offset nearest `x` on line `i` (clamped to the lines).
    pub(crate) fn hit(&self, x: f32, i: usize) -> usize {
        let i = i.min(self.count().saturating_sub(1));
        self.lines[i].start + self.visual(i).hit(x)
    }

    /// The selection `lo..hi` as boxes, `(x, w, line)`.
    pub(crate) fn boxes(&self, lo: usize, hi: usize) -> Vec<(f32, f32, usize)> {
        let mut out = Vec::new();
        for (i, l) in self.lines.iter().enumerate() {
            let (a, b) = (lo.max(l.start), hi.min(l.end));
            if a < b {
                let v = self.visual(i);
                out.extend(
                    v.boxes(a - l.start, b - l.start)
                        .into_iter()
                        .map(|(x, w)| (x, w, i)),
                );
            }
            // A selected line break shows as a sliver at the line's end.
            if lo <= l.end && hi > l.end && self.text.as_bytes().get(l.end) == Some(&b'\n') {
                let x = self.visual(i).caret_x(l.end - l.start);
                out.push((x, self.style.font_size / 3.0, i));
            }
        }
        out
    }

    /// One caret stop left or right of `pos` on screen, crossing to the
    /// line before or after at a line's edge; `pos` at the text's ends.
    pub(crate) fn step(&self, pos: usize, rightward: bool) -> usize {
        let i = self.line_of(pos);
        let l = self.lines[i];
        let v = self.visual(i);
        let local = pos.clamp(l.start, l.end) - l.start;
        let next = if rightward {
            v.right_of(local)
        } else {
            v.left_of(local)
        };
        if let Some(n) = next {
            return l.start + n;
        }
        // At a wrap the offset is already the next line's, so going right
        // past a line's end always crosses a line break.
        if rightward {
            self.lines.get(i + 1).map_or(pos, |n| n.start)
        } else if i == 0 {
            pos
        } else {
            let p = self.lines[i - 1];
            if p.end == l.start {
                // a wrap: the stop before the shared offset
                let v = self.visual(i - 1);
                v.left_of(p.end - p.start).map_or(p.end, |s| p.start + s)
            } else {
                p.end
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> TextStyle {
        TextStyle::new(13.0).with_line_height(16.0)
    }

    /// Each `\n` starts a line, an empty paragraph is an empty line, and
    /// the lines of a wrapped paragraph touch, covering it whole.
    #[test]
    fn the_lines_cover_the_text_paragraph_by_paragraph() {
        let s = style();
        let l = Lines::new("ab\n\ncd", &s, 400.0);
        let ranges: Vec<_> = l.lines.iter().map(|l| (l.start, l.end)).collect();
        assert_eq!(ranges, [(0, 2), (3, 3), (4, 6)]);
        let long = "one two three four five six seven eight nine ten eleven twelve";
        let l = Lines::new(long, &s, 80.0);
        assert!(l.count() > 2, "it wraps");
        assert_eq!(
            (l.lines[0].start, l.lines.last().unwrap().end),
            (0, long.len())
        );
        for pair in l.lines.windows(2) {
            assert_eq!(pair[0].end, pair[1].start, "a wrap leaves no gap");
        }
        let wrap = l.lines[1].start;
        assert_eq!(l.line_of(wrap), 1, "the wrap's offset is the next line's");
        assert_eq!(Lines::new("", &s, 80.0).count(), 1);
    }

    /// Left and Right cross a line break and a wrap one stop at a time.
    #[test]
    fn the_caret_steps_across_the_lines() {
        let s = style();
        let l = Lines::new("ab\ncd", &s, 400.0);
        assert_eq!(
            l.step(2, true),
            3,
            "past the end of a line, the next's start"
        );
        assert_eq!(l.step(3, false), 2, "and back");
        assert_eq!(l.step(0, false), 0);
        assert_eq!(l.step(5, true), 5);
        let long = "one two three four five six seven eight nine ten";
        let l = Lines::new(long, &s, 80.0);
        let wrap = l.lines[1].start;
        assert_eq!(
            l.step(wrap, false),
            wrap - 1,
            "before the wrap, not stuck on it"
        );
        assert_eq!(l.step(wrap - 1, true), wrap);
        let boxes = l.boxes(0, long.len());
        assert_eq!(
            boxes
                .iter()
                .map(|b| b.2)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            l.count(),
            "a box on every line"
        );
    }
}
