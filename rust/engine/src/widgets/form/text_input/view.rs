//! What a single-line field shows of its text: the text itself, or a
//! mask over it (a password: one bullet per grapheme), and the part of
//! the line a box too narrow for it lets through.
//!
//! A mask keeps the editing on the real text: the caret, a click and a
//! selection are byte offsets of the buffer, and [`Shown`] maps them to
//! the bullets and back. Bullets run left to right whatever the text's
//! direction, so the arrow keys move through a password in text order.
//!
//! A line wider than its box is never wrapped: it is laid out unbounded
//! and drawn under a clip of the box. Focused, it slides left so the
//! caret stays inside ([`LineFit::scroll`]); blurred, the owner draws it
//! cut with an ellipsis (`TextMeasurer::truncate_to_width`).

use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;

/// The bullet a masked grapheme draws as.
pub const MASK: char = '\u{2022}';

/// A text as a field draws it, with the byte map back to the text.
pub(crate) struct Shown<'a> {
    text: Cow<'a, str>,
    /// Masked: the byte offset in the source of every grapheme boundary,
    /// the end included. `None`: the text is drawn as it is.
    starts: Option<Vec<usize>>,
}

impl<'a> Shown<'a> {
    pub(crate) fn new(text: &'a str, masked: bool) -> Self {
        if !masked {
            return Self {
                text: Cow::Borrowed(text),
                starts: None,
            };
        }
        let starts: Vec<usize> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain(std::iter::once(text.len()))
            .collect();
        let bullets = std::iter::repeat_n(MASK, starts.len() - 1).collect();
        Self {
            text: Cow::Owned(bullets),
            starts: Some(starts),
        }
    }

    /// What is drawn.
    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    /// A byte offset of the source as an offset of what is drawn: the
    /// bullet of the grapheme it starts (the end after the last).
    pub(crate) fn shown_pos(&self, pos: usize) -> usize {
        match &self.starts {
            None => pos,
            Some(starts) => starts.partition_point(|&s| s < pos) * MASK.len_utf8(),
        }
    }

    /// An offset of what is drawn as a byte offset of the source.
    pub(crate) fn source_pos(&self, pos: usize) -> usize {
        match &self.starts {
            None => pos,
            Some(starts) => starts[(pos / MASK.len_utf8()).min(starts.len() - 1)],
        }
    }
}

/// How a line sits in a box `box_w` wide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LineFit {
    /// The width the line is laid out in: the box when the line fits (a
    /// right-to-left paragraph hangs from the box's right edge, as the
    /// renderer aligns it), `None` (unbounded, never wrapped) when not.
    pub width: Option<f32>,
    /// How far the line slides left: 0 unless it overflows and the caret
    /// would leave the box, then the caret sits on the box's trailing
    /// edge.
    pub scroll: f32,
    /// The line is wider than the box: it draws under a clip, or cut.
    pub overflows: bool,
}

impl LineFit {
    /// A line `line_w` wide in a box `box_w` wide, with the caret
    /// `caret_w` wide at `caret_x` (laid out unbounded) when focused.
    pub fn new(line_w: f32, box_w: f32, caret: Option<(f32, f32)>) -> Self {
        let overflows = line_w > box_w;
        if !overflows {
            return Self {
                width: Some(box_w),
                scroll: 0.0,
                overflows,
            };
        }
        let scroll = caret.map_or(0.0, |(x, w)| {
            (x + w - box_w).clamp(0.0, (line_w + w - box_w).max(0.0))
        });
        Self {
            width: None,
            scroll,
            overflows,
        }
    }
}
