//! Clip stack support: `SceneNode::PushClip`/`PopClip` scope nodes to the
//! intersection of all active clip rects. Geometry builders group emitted
//! primitives into [`DrawRange`]s so the encoder can issue one
//! `set_scissor_rect` per group instead of one render pass per clip.

/// Clip rect as (x, y, w, h) in logical pixels.
pub type ClipRect = [f32; 4];

/// A contiguous run of indices in a layer's index buffer sharing one clip.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrawRange {
    pub first_index: u32,
    pub index_count: u32,
    /// Intersection of the clip stack when these primitives were emitted.
    /// `None` = unclipped (full viewport scissor).
    pub clip: Option<ClipRect>,
}

/// Intersection of two clip rects. May produce a degenerate rect
/// (w or h <= 0) when the inputs are disjoint.
pub fn intersect_rects(a: ClipRect, b: ClipRect) -> ClipRect {
    let x0 = a[0].max(b[0]);
    let y0 = a[1].max(b[1]);
    let x1 = (a[0] + a[2]).min(b[0] + b[2]);
    let y1 = (a[1] + a[3]).min(b[1] + b[3]);
    [x0, y0, x1 - x0, y1 - y0]
}

/// Convert a clip rect to a scissor rect clamped to the viewport.
/// Returns `None` when the visible area is empty (skip the draw).
pub fn clip_to_scissor(clip: ClipRect, vw: u32, vh: u32) -> Option<(u32, u32, u32, u32)> {
    let x0 = clip[0].max(0.0).floor() as u32;
    let y0 = clip[1].max(0.0).floor() as u32;
    let x1 = ((clip[0] + clip[2]).ceil().max(0.0) as u32).min(vw);
    let y1 = ((clip[1] + clip[3]).ceil().max(0.0) as u32).min(vh);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some((x0, y0, x1 - x0, y1 - y0))
}

/// Intersection of two scissor rects (x, y, w, h). `None` when empty.
pub fn intersect_scissors(
    a: (u32, u32, u32, u32),
    b: (u32, u32, u32, u32),
) -> Option<(u32, u32, u32, u32)> {
    let x0 = a.0.max(b.0);
    let y0 = a.1.max(b.1);
    let x1 = (a.0 + a.2).min(b.0 + b.2);
    let y1 = (a.1 + a.3).min(b.1 + b.3);
    if x0 >= x1 || y0 >= y1 {
        return None;
    }
    Some((x0, y0, x1 - x0, y1 - y0))
}

/// Tracks the active clip while walking scene nodes in paint order.
/// Pushes store the running intersection so `current` is O(1).
#[derive(Default)]
pub(crate) struct ClipStack {
    stack: Vec<ClipRect>,
}

impl ClipStack {
    pub(crate) fn push(&mut self, rect: ClipRect) {
        let combined = match self.current() {
            Some(cur) => intersect_rects(cur, rect),
            None => rect,
        };
        self.stack.push(combined);
    }

    /// Pop the most recent clip. Unbalanced pops are ignored (and logged)
    /// rather than panicking on malformed scenes.
    pub(crate) fn pop(&mut self) {
        if self.stack.pop().is_none() {
            log::warn!("PopClip without matching PushClip -- ignored");
        }
    }

    pub(crate) fn current(&self) -> Option<ClipRect> {
        self.stack.last().copied()
    }

    /// Whether the current clip has no visible area (everything inside it
    /// can be skipped entirely).
    pub(crate) fn is_empty_clip(&self) -> bool {
        matches!(self.current(), Some([_, _, w, h]) if w <= 0.0 || h <= 0.0)
    }
}

/// Append `count` indices starting at `first_index` to the range list,
/// merging into the previous range when the clip is unchanged.
pub(crate) fn record_range(
    ranges: &mut Vec<DrawRange>,
    first_index: u32,
    index_count: u32,
    clip: Option<ClipRect>,
) {
    if index_count == 0 {
        return;
    }
    if let Some(last) = ranges.last_mut()
        && last.clip == clip
        && last.first_index + last.index_count == first_index
    {
        last.index_count += index_count;
        return;
    }
    ranges.push(DrawRange {
        first_index,
        index_count,
        clip,
    });
}

/// Merge per-group text geometry (as produced by
/// `TextSystem::resolve_for_layer` per group) into single vertex/index
/// buffers plus draw ranges. Indices are rebased onto the merged buffer.
/// Exactly one range is emitted per group -- even empty ones -- so ranges
/// stay 1:1 with the layer's `Text` draw commands (see
/// `Layer::assign_text_ranges`).
pub fn merge_text_groups(
    groups: Vec<(Vec<crate::text::TextVertex>, Vec<u32>, Option<ClipRect>)>,
) -> (Vec<crate::text::TextVertex>, Vec<u32>, Vec<DrawRange>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut ranges = Vec::new();

    for (group_vertices, group_indices, clip) in groups {
        let vertex_base = vertices.len() as u32;
        let first_index = indices.len() as u32;
        let index_count = group_indices.len() as u32;
        vertices.extend(group_vertices);
        indices.extend(group_indices.into_iter().map(|i| i + vertex_base));
        ranges.push(DrawRange {
            first_index,
            index_count,
            clip,
        });
    }

    (vertices, indices, ranges)
}
