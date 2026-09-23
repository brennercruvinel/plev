//! `Limits`, what the recorder keeps and how often it keys a track, and
//! `Stats`, what it counted while recording.

use sem::Millis;

/// how the recorder keys its track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// the semantic keyframe cadence: a changed snapshot is stored whole
    /// when the segment it would join opened this long ago or more, and
    /// any other event arriving then first re-keys the track with the
    /// state as it stands (an unchanged state shares its snapshot).
    pub full_every: Millis,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { full_every: 2_000 }
    }
}

/// what the recorder counted since it was made.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    /// semantic events recorded.
    pub events: u64,
    /// snapshots stored whole: the first, the structural ones, the ones on
    /// the cadence and the ones asked for.
    pub snapshots_full: u64,
    /// snapshots stored as their changes.
    pub snapshots_changed: u64,
    /// snapshots equal to the state before, not stored.
    pub snapshots_unchanged: u64,
    /// re-keys of an unchanged state on the cadence.
    pub rekeys: u64,
    /// times earlier than the last recorded one, recorded as the last one.
    pub clamped: u64,
    /// segments of the semantic track now held.
    pub segments: usize,
    /// the estimated bytes of the semantic track now held.
    pub semantic_bytes: usize,
}
