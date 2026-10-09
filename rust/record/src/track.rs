//! the semantic track: events in segments, each segment opening at a full
//! snapshot, the keyframe of the track. the state at any instant of a
//! segment is its opening snapshot plus the changes after it, so a reader
//! never looks before the segment that governs a time, and the ring cuts
//! only between segments: what it keeps always opens at a full snapshot.
//! only the first segment of a recording may open without a snapshot (the
//! events that came before the app gave one); having no keyframe to
//! respect, it is trimmed event by event.

use std::collections::VecDeque;
use std::sync::Arc;

use accord::{Millis, Snapshot};

use crate::event::{Event, Kind};
use crate::weigh;

/// one keyframe and what followed it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Segment {
    pub(crate) events: Vec<Event>,
    /// the weight of the events, the opening snapshot excluded.
    pub(crate) bytes: usize,
    /// the weight of the opening snapshot charged to this segment: its
    /// whole weight, or zero when the segment before opens at the same
    /// snapshot (a re-key of an unchanged state shares it), so a shared
    /// snapshot is counted once.
    pub(crate) base: usize,
}

impl Segment {
    /// the time of the first event.
    pub(crate) fn start(&self) -> Option<Millis> {
        self.events.first().map(|e| e.t)
    }

    /// the full snapshot the segment opens at; `None` for the events
    /// recorded before the first one.
    pub(crate) fn opening(&self) -> Option<&Arc<Snapshot>> {
        match self.events.first().map(|e| &e.kind) {
            Some(Kind::Snapshot { snapshot, .. }) => Some(snapshot),
            _ => None,
        }
    }

    pub(crate) fn weight(&self) -> usize {
        self.bytes + self.base
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Track {
    segments: VecDeque<Segment>,
    bytes: usize,
}

impl Track {
    /// appends an event to the last segment; the first event of a
    /// recording opens a segment without a snapshot.
    pub(crate) fn push(&mut self, event: Event) {
        if self.segments.is_empty() {
            self.segments.push_back(Segment::default());
        }
        let w = weigh::event(&event);
        if let Some(last) = self.segments.back_mut() {
            last.events.push(event);
            last.bytes += w;
            self.bytes += w;
        }
    }

    /// opens a segment at a full snapshot event (`Kind::Snapshot`).
    pub(crate) fn key(&mut self, event: Event) {
        let shared = match (&event.kind, self.segments.back().and_then(Segment::opening)) {
            (Kind::Snapshot { snapshot, .. }, Some(before)) => Arc::ptr_eq(snapshot, before),
            _ => false,
        };
        let base = match &event.kind {
            Kind::Snapshot { snapshot, .. } if !shared => weigh::snapshot(snapshot),
            _ => 0,
        };
        let bytes = weigh::event(&event);
        self.bytes += bytes + base;
        self.segments.push_back(Segment {
            events: vec![event],
            bytes,
            base,
        });
    }

    /// drops the oldest segment. when the next one opens at the same
    /// snapshot, the snapshot stays alive there, and so does its weight.
    pub(crate) fn pop_front(&mut self) -> Option<Segment> {
        let gone = self.segments.pop_front()?;
        let mut freed = gone.weight();
        if let (Some(snapshot), Some(next)) = (gone.opening(), self.segments.front_mut()) {
            if next.base == 0 && next.opening().is_some_and(|s| Arc::ptr_eq(s, snapshot)) {
                next.base = gone.base;
                freed -= gone.base;
            }
        }
        self.bytes -= freed;
        Some(gone)
    }

    /// drops the oldest event of a first segment that opens without a
    /// snapshot, when it is the only segment (nothing keyed to cut at).
    /// `false` when there is no such event.
    pub(crate) fn trim_unkeyed(&mut self) -> bool {
        if self.segments.len() != 1 {
            return false;
        }
        let Some(front) = self.segments.front_mut() else {
            return false;
        };
        if front.opening().is_some() || front.events.is_empty() {
            return false;
        }
        let gone = front.events.remove(0);
        let w = weigh::event(&gone);
        front.bytes -= w;
        self.bytes -= w;
        true
    }

    pub(crate) fn segments(&self) -> &VecDeque<Segment> {
        &self.segments
    }

    /// the start of the last segment, where the next re-key is measured
    /// from.
    pub(crate) fn last_start(&self) -> Option<Millis> {
        self.segments.back().and_then(Segment::start)
    }

    /// the start of the second segment: the first one may go once this is
    /// old enough.
    pub(crate) fn second_start(&self) -> Option<Millis> {
        self.segments.get(1).and_then(Segment::start)
    }

    /// the time of the first event held.
    pub(crate) fn first_t(&self) -> Option<Millis> {
        self.segments.front().and_then(Segment::start)
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// every event, in order.
    pub(crate) fn events(&self) -> impl Iterator<Item = &Event> + '_ {
        self.segments.iter().flat_map(|s| s.events.iter())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use accord::{Role, SnapshotBuilder};

    use super::Track;
    use crate::event::{Event, InputKind, Kind};
    use crate::weigh;

    fn key(t: u64, snapshot: &Arc<accord::Snapshot>) -> Event {
        Event {
            t,
            kind: Kind::Snapshot {
                snapshot: snapshot.clone(),
                changes: Vec::new(),
            },
        }
    }

    fn input(t: u64) -> Event {
        Event {
            t,
            kind: Kind::Input {
                kind: InputKind::Key,
                at: None,
                target: None,
            },
        }
    }

    /// the bytes the track claims, recomputed from what it holds with
    /// every distinct snapshot counted once.
    fn recount(track: &Track) -> usize {
        let mut seen: Vec<&Arc<accord::Snapshot>> = Vec::new();
        let mut total = 0;
        for segment in track.segments() {
            total += segment.events.iter().map(weigh::event).sum::<usize>();
            if let Some(s) = segment.opening() {
                if !seen.iter().any(|x| Arc::ptr_eq(x, s)) {
                    seen.push(s);
                    total += weigh::snapshot(s);
                }
            }
        }
        total
    }

    #[test]
    fn a_shared_snapshot_is_counted_once_and_moves_on_eviction() {
        let mut b = SnapshotBuilder::new();
        b.node("list", Role::List, "a list with a long enough label");
        let Ok(snapshot) = b.finish() else {
            panic!("valid projection");
        };
        let (a, other) = (Arc::new(snapshot.clone()), Arc::new(snapshot));
        let mut track = Track::default();
        track.push(input(0));
        track.key(key(1, &a));
        track.push(input(2));
        track.key(key(3, &a));
        track.key(key(4, &a));
        track.key(key(5, &other));
        assert_eq!(track.bytes(), recount(&track));
        let one = weigh::snapshot(&a);
        assert_eq!(track.segments()[2].base, 0, "shared with the one before");
        assert_eq!(track.segments()[4].base, one, "a distinct snapshot");

        assert!(!track.trim_unkeyed(), "four keyed segments follow");
        while track.segments().len() > 1 {
            track.pop_front();
            assert_eq!(track.bytes(), recount(&track));
        }
        assert_eq!(track.first_t(), Some(5));
    }

    #[test]
    fn an_unkeyed_recording_is_trimmed_event_by_event() {
        let mut track = Track::default();
        for t in 0..4 {
            track.push(input(t));
        }
        assert!(track.trim_unkeyed());
        assert_eq!(track.first_t(), Some(1));
        assert_eq!(track.bytes(), recount(&track));
        while track.trim_unkeyed() {}
        assert_eq!(track.bytes(), 0);
    }
}
