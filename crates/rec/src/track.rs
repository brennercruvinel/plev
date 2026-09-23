//! the semantic track: events in segments, each segment opening at a full
//! snapshot, the keyframe of the track. the state at any instant of a
//! segment is its opening snapshot plus the changes after it, so a reader
//! never looks before the segment that governs a time. only the first
//! segment of a recording may open without a snapshot (the events that
//! came before the app gave one).

use sem::Millis;
use std::collections::VecDeque;

use crate::event::{Event, Kind};
use crate::weigh;

/// one keyframe and what followed it.
#[derive(Clone, Debug, Default)]
pub(crate) struct Segment {
    pub(crate) events: Vec<Event>,
    /// the weight of the events, the opening snapshot included.
    pub(crate) bytes: usize,
}

impl Segment {
    /// the time of the first event.
    pub(crate) fn start(&self) -> Option<Millis> {
        self.events.first().map(|e| e.t)
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
        let base = match &event.kind {
            Kind::Snapshot { snapshot, .. } => weigh::snapshot(snapshot),
            _ => 0,
        };
        let bytes = weigh::event(&event) + base;
        self.bytes += bytes;
        self.segments.push_back(Segment {
            events: vec![event],
            bytes,
        });
    }

    pub(crate) fn segments(&self) -> &VecDeque<Segment> {
        &self.segments
    }

    /// the start of the last segment, where the next re-key is measured
    /// from.
    pub(crate) fn last_start(&self) -> Option<Millis> {
        self.segments.back().and_then(Segment::start)
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }

    /// every event, in order.
    pub(crate) fn events(&self) -> impl Iterator<Item = &Event> + '_ {
        self.segments.iter().flat_map(|s| s.events.iter())
    }
}
