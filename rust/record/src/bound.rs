//! where each bound control shows in each frame (task-63 phase d), the map
//! from a point of a replayed frame to a control that the inspector's ui
//! reads (task-64 phase b).
//!
//! a frame taken from a compositor carries its bindings that show
//! (`Compositor::visible_bindings`): each id with the part of its rect
//! inside its clip, on a layer the frame draws, in stacking order, the one
//! on top last, as `Compositor::hit` ranks them. the visual track keeps
//! them in each chunk beside its frames, only when they differ from the
//! frame before. a chunk opens with the whole list, its keyframe, so a
//! ring cut at a chunk loses nothing; after it a step is the whole list
//! again (an id came or went, or the order changed) or only the positions
//! whose rect moved (the same ids in the same order: a scroll, a toast
//! springing in).
//!
//! in the file they are section B (see `format`), which only a session
//! with bindings carries:
//!
//! B := id_count u32, (len u16, utf8)*   every id the steps name, once each
//!      chunk_count u32                   the chunk count of section I
//!      per chunk, in index order: step_count u32, step*
//! step := at u32 (ms from the chunk start, strictly increasing), kind u8,
//!         count u32, then per entry:
//!         kind 0, the whole list: id u32 (into the table), rect
//!         kind 1, moved: position u32 (into the list, increasing), rect
//! rect := x, y, w, h, each an f32 as its bits in a u32; finite, w and h
//!         above zero

use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;

use engine::compositor::{Compositor, Layer};
use sem::{Millis, SemId};

use crate::error::FileError;
use crate::format::{Cur, len32, put_u16, put_u32};

/// where one bound control shows in a frame.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Bound {
    pub id: SemId,
    /// `(x, y, w, h)` in logical window pixels: the part of the rect the
    /// control draws and hit-tests with that is inside its clip
    /// (`engine::compositor::Binding::visible_rect`).
    pub rect: [f32; 4],
}

/// one change of a chunk's bindings.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Shift {
    /// the whole list, in stacking order.
    Whole(Arc<[Bound]>),
    /// the same ids in the same order; the rect at each position given.
    Moved(Vec<(u32, [f32; 4])>),
}

/// a change at `at` milliseconds from the chunk's start.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Step {
    pub(crate) at: u32,
    pub(crate) shift: Shift,
}

impl Step {
    /// the estimated bytes the ring holds for this step.
    pub(crate) fn weight(&self) -> usize {
        size_of::<Step>()
            + match &self.shift {
                Shift::Whole(list) => list
                    .iter()
                    .map(|b| size_of::<Bound>() + b.id.as_str().len())
                    .sum(),
                Shift::Moved(moved) => moved.len() * size_of::<(u32, [f32; 4])>(),
            }
    }
}

/// what a frame's bindings were to the frame before.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Taken {
    Same,
    Whole,
    Moved,
}

/// the bindings of the open chunk as they are recorded.
#[derive(Debug)]
pub(crate) struct Steps {
    pub(crate) steps: Vec<Step>,
    /// the list after the last step, and before it.
    now: Arc<[Bound]>,
    before: Arc<[Bound]>,
    /// the estimated bytes of `steps`.
    pub(crate) bytes: usize,
}

impl Default for Steps {
    fn default() -> Self {
        Steps {
            steps: Vec::new(),
            now: Arc::from(Vec::new()),
            before: Arc::from(Vec::new()),
            bytes: 0,
        }
    }
}

/// the rect's bits: a change is any change, and a NaN never repeats.
fn bits(rect: [f32; 4]) -> [u32; 4] {
    rect.map(f32::to_bits)
}

/// the step from `before` to `now`, none when nothing changed.
fn shift(before: &[Bound], now: &[Bound]) -> Option<Shift> {
    let same_ids = before.len() == now.len() && before.iter().zip(now).all(|(a, b)| a.id == b.id);
    if !same_ids {
        return Some(Shift::Whole(Arc::from(now)));
    }
    let moved: Vec<(u32, [f32; 4])> = before
        .iter()
        .zip(now)
        .enumerate()
        .filter(|(_, (a, b))| bits(a.rect) != bits(b.rect))
        .map(|(at, (_, b))| (u32::try_from(at).unwrap_or(u32::MAX), b.rect))
        .collect();
    (!moved.is_empty()).then_some(Shift::Moved(moved))
}

impl Steps {
    /// the frame taken `at` ms from the chunk's start shows `now`: a step
    /// when it differs from the frame before. a second frame in the same
    /// millisecond replaces the step of the first.
    pub(crate) fn take(&mut self, at: u32, now: &[Bound]) -> Taken {
        if self.steps.last().is_some_and(|s| s.at == at) {
            if let Some(last) = self.steps.pop() {
                self.bytes -= last.weight();
            }
            self.now = self.before.clone();
        }
        let Some(shift) = shift(&self.now, now) else {
            return Taken::Same;
        };
        let taken = match &shift {
            Shift::Whole(list) => {
                self.before = std::mem::replace(&mut self.now, list.clone());
                Taken::Whole
            }
            Shift::Moved(_) => {
                self.before = std::mem::replace(&mut self.now, Arc::from(now));
                Taken::Moved
            }
        };
        let step = Step { at, shift };
        self.bytes += step.weight();
        self.steps.push(step);
        taken
    }
}

/// the bindings a chunk's steps give at `at` ms from its start: empty
/// before its first step.
pub(crate) fn replay(steps: &[Step], at: u32) -> Vec<Bound> {
    let mut list: Vec<Bound> = Vec::new();
    for step in steps.iter().take_while(|s| s.at <= at) {
        match &step.shift {
            Shift::Whole(whole) => list = whole.to_vec(),
            Shift::Moved(moved) => {
                for (position, rect) in moved {
                    let slot = usize::try_from(*position)
                        .ok()
                        .and_then(|p| list.get_mut(p));
                    if let Some(bound) = slot {
                        bound.rect = *rect;
                    }
                }
            }
        }
    }
    list
}

/// the steps of the part of a chunk from `from` to before `to` (ms from
/// its start), rebased on `from`: the list at `from`, whole, then every
/// step after it.
pub(crate) fn part(steps: &[Step], from: u32, to: Option<u32>) -> Vec<Step> {
    let open = replay(steps, from);
    let mut out = Vec::new();
    if !open.is_empty() {
        out.push(Step {
            at: 0,
            shift: Shift::Whole(Arc::from(open)),
        });
    }
    out.extend(
        steps
            .iter()
            .filter(|s| s.at > from && to.is_none_or(|to| s.at < to))
            .map(|s| Step {
                at: s.at - from,
                shift: s.shift.clone(),
            }),
    );
    out
}

/// the bindings of the frame `compositor` holds that show on a layer the
/// frame draws (`layers`, in composite order), in stacking order: by the
/// layer's place in the composite, then by bind order within it, the one
/// on top last. a rect that is not finite is left out.
pub(crate) fn stacked(compositor: &Compositor, layers: &[&Layer]) -> Vec<Bound> {
    let mut out: Vec<((usize, u32), Bound)> = compositor
        .visible_bindings()
        .filter_map(|(id, rect)| {
            let binding = compositor.binding(id)?;
            let rank = layers.iter().position(|l| l.id == binding.layer)?;
            rect.iter().all(|v| v.is_finite()).then(|| {
                let bound = Bound {
                    id: id.clone(),
                    rect,
                };
                ((rank, binding.order), bound)
            })
        })
        .collect();
    out.sort_by_key(|(key, _)| *key);
    out.into_iter().map(|(_, bound)| bound).collect()
}

/// the control on top at `(x, y)` in a list in stacking order: the last
/// one whose rect holds the point, edges included, like `Compositor::hit`.
pub(crate) fn hit(list: &[Bound], x: f32, y: f32) -> Option<&SemId> {
    list.iter()
        .rev()
        .find(|b| {
            let [bx, by, w, h] = b.rect;
            x >= bx && x <= bx + w && y >= by && y <= by + h
        })
        .map(|b| &b.id)
}

fn put_rect(buf: &mut Vec<u8>, rect: [f32; 4]) {
    for v in rect {
        put_u32(buf, v.to_bits());
    }
}

/// section B for chunks whose steps are `chunks`, in index order; `None`
/// when no chunk has a step, so a session without bindings is written as
/// before them.
pub(crate) fn section(chunks: &[&[Step]]) -> Result<Option<Vec<u8>>, FileError> {
    if chunks.iter().all(|steps| steps.is_empty()) {
        return Ok(None);
    }
    let mut ids: Vec<&SemId> = Vec::new();
    let mut index: HashMap<&SemId, u32> = HashMap::new();
    let mut body = Vec::new();
    put_u32(&mut body, len32(chunks.len(), "bound chunk count")?);
    for steps in chunks {
        put_u32(&mut body, len32(steps.len(), "binding steps")?);
        for step in *steps {
            put_u32(&mut body, step.at);
            match &step.shift {
                Shift::Whole(list) => {
                    body.push(0);
                    put_u32(&mut body, len32(list.len(), "bindings")?);
                    for bound in list.iter() {
                        let next = len32(ids.len(), "bound ids")?;
                        let at = *index.entry(&bound.id).or_insert_with(|| {
                            ids.push(&bound.id);
                            next
                        });
                        put_u32(&mut body, at);
                        put_rect(&mut body, bound.rect);
                    }
                }
                Shift::Moved(moved) => {
                    body.push(1);
                    put_u32(&mut body, len32(moved.len(), "moved bindings")?);
                    for (position, rect) in moved {
                        put_u32(&mut body, *position);
                        put_rect(&mut body, *rect);
                    }
                }
            }
        }
    }
    let mut out = Vec::with_capacity(body.len());
    put_u32(&mut out, len32(ids.len(), "bound ids")?);
    for id in ids {
        let text = id.as_str();
        let len = u16::try_from(text.len()).map_err(|_| FileError::TooLarge("a bound id"))?;
        put_u16(&mut out, len);
        out.extend_from_slice(text.as_bytes());
    }
    out.extend_from_slice(&body);
    Ok(Some(out))
}

fn read_rect(
    c: &mut Cur<'_>,
    bad: impl Fn(&'static str) -> FileError,
) -> Result<[f32; 4], FileError> {
    let mut rect = [0f32; 4];
    for v in &mut rect {
        *v = f32::from_bits(c.u32("a bound rect")?);
    }
    let [_, _, w, h] = rect;
    if !rect.iter().all(|v| v.is_finite()) || w <= 0.0 || h <= 0.0 {
        return Err(bad("a rect is not finite or has no area"));
    }
    Ok(rect)
}

/// reads section B, strictly, for chunks that start at `starts`: the
/// steps of each chunk, in index order.
pub(crate) fn read_section(payload: &[u8], starts: &[Millis]) -> Result<Vec<Vec<Step>>, FileError> {
    let mut c = Cur::new(payload);
    let count = c.count(2, "bound id count")?;
    let mut ids: Vec<SemId> = Vec::with_capacity(count);
    for at in 0..count {
        let len = usize::from(c.u16("bound id length")?);
        let text = c.text(len, "a bound id")?;
        let id = SemId::parse(&text).map_err(|e| FileError::BoundId {
            at,
            reason: e.to_string(),
        })?;
        if ids.contains(&id) {
            return Err(FileError::BoundId {
                at,
                reason: "it is in the table twice".to_owned(),
            });
        }
        ids.push(id);
    }
    let chunks = c.count(4, "bound chunk count")?;
    if chunks != starts.len() {
        return Err(FileError::Bindings {
            chunk: chunks,
            reason: "section B does not have one entry per chunk of the index",
        });
    }
    let mut out = Vec::with_capacity(chunks);
    for (chunk, &start) in starts.iter().enumerate() {
        let next = starts.get(chunk + 1).copied();
        out.push(read_chunk(&mut c, &ids, chunk, start, next)?);
    }
    c.done("the bindings")?;
    Ok(out)
}

/// the steps of one chunk that starts at `start`, the next at `next`.
fn read_chunk(
    c: &mut Cur<'_>,
    ids: &[SemId],
    chunk: usize,
    start: Millis,
    next: Option<Millis>,
) -> Result<Vec<Step>, FileError> {
    let bad = |reason| FileError::Bindings { chunk, reason };
    let count = c.count(9, "binding steps")?;
    let mut steps: Vec<Step> = Vec::with_capacity(count);
    let mut len: Option<usize> = None;
    for _ in 0..count {
        let at = c.u32("a step time")?;
        if steps.last().is_some_and(|s| at <= s.at) {
            return Err(bad("step times do not increase"));
        }
        let t = start
            .checked_add(Millis::from(at))
            .ok_or(bad("a step time overflows"))?;
        if next.is_some_and(|next| t >= next) {
            return Err(bad("a step comes at or after the next chunk's start"));
        }
        let kind = c.u8("a step kind")?;
        let n = c.count(20, "a step's bindings")?;
        let shift = match kind {
            0 => {
                let mut list: Vec<Bound> = Vec::with_capacity(n);
                for _ in 0..n {
                    let at = usize::try_from(c.u32("a bound id index")?).unwrap_or(usize::MAX);
                    let id = ids.get(at).ok_or(bad("an id index is past the table"))?;
                    if list.iter().any(|b| b.id == *id) {
                        return Err(bad("an id shows twice in one list"));
                    }
                    let rect = read_rect(c, bad)?;
                    list.push(Bound {
                        id: id.clone(),
                        rect,
                    });
                }
                len = Some(list.len());
                Shift::Whole(Arc::from(list))
            }
            1 => {
                let len = len.ok_or(bad("rects move before a whole list"))?;
                if n == 0 {
                    return Err(bad("a move step moves nothing"));
                }
                let mut moved: Vec<(u32, [f32; 4])> = Vec::with_capacity(n);
                for _ in 0..n {
                    let position = c.u32("a moved position")?;
                    if moved.last().is_some_and(|(p, _)| position <= *p)
                        || usize::try_from(position).map_or(true, |p| p >= len)
                    {
                        return Err(bad("a moved position is out of order or past the list"));
                    }
                    moved.push((position, read_rect(c, bad)?));
                }
                Shift::Moved(moved)
            }
            _ => return Err(bad("unknown step kind")),
        };
        steps.push(Step { at, shift });
    }
    Ok(steps)
}
