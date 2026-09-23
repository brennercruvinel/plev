//! `Limits`, what the ring keeps and how often it keys a track, and
//! `Stats`, what the recorder counted while recording.

use sem::Millis;

/// what the ring keeps, by time and by bytes, and how often it keys the
/// track. both limits cut only at a keyframe, so what the ring keeps
/// always opens at a full snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// the ring keeps at least the last `max_age` milliseconds: a segment
    /// goes when the one after it opened at or before `now - max_age`. it
    /// may keep up to one segment more than that (about `full_every`).
    pub max_age: Millis,
    /// the ring holds at most this many bytes, as the recorder estimates
    /// them (inline sizes plus owned text and collections). when the last
    /// segment alone is over, the next event re-keys the track so the
    /// older one can go, and `Stats::over_budget` counts it.
    pub max_bytes: usize,
    /// the semantic keyframe cadence: a changed snapshot is stored whole
    /// when the segment it would join opened this long ago or more, and
    /// any other event arriving then first re-keys the track with the
    /// state as it stands (an unchanged state shares its snapshot).
    pub full_every: Millis,
}

impl Default for Limits {
    /// the last 20 seconds, in at most 32 MiB, keyed every 2 seconds.
    fn default() -> Self {
        Limits {
            max_age: 20_000,
            max_bytes: 32 << 20,
            full_every: 2_000,
        }
    }
}

/// what the recorder counted since it was made, and what the ring holds
/// now.
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
    /// re-keys of the state as it stands, on the cadence or to let the
    /// ring cut.
    pub rekeys: u64,
    /// times earlier than the last recorded one, recorded as the last one.
    pub clamped: u64,
    /// semantic segments the ring dropped, by time or by bytes.
    pub evicted_segments: u64,
    /// events recorded before any snapshot that the ring dropped one by
    /// one (nothing keyed to cut at).
    pub evicted_unkeyed: u64,
    /// times the newest segment alone was over the byte budget.
    pub over_budget: u64,
    /// segments of the semantic track now held.
    pub segments: usize,
    /// the estimated bytes of the semantic track now held.
    pub semantic_bytes: usize,
    /// the estimated bytes the ring holds now, every track together.
    pub bytes: usize,
}
