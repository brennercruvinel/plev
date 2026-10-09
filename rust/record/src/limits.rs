//! `Limits`, what the ring keeps and how often it keys a track, and
//! `Stats`, what the recorder counted while recording.

use accord::Millis;

/// what the ring keeps, by time and by bytes, and how often it keys each
/// track. both limits cut only at a keyframe, so what the ring keeps
/// always opens at a full snapshot on the semantic track and at a visual
/// keyframe on the visual one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// the ring keeps at least the last `max_age` milliseconds: a segment
    /// or a chunk goes when the one after it opened at or before `now -
    /// max_age`. it may keep up to one segment or chunk more than that.
    pub max_age: Millis,
    /// the ring holds at most this many bytes, every track together, as
    /// the recorder estimates them (inline sizes plus owned text and
    /// collections; a closed chunk counts its encoded bytes). when only
    /// the newest segment and chunk are left and still over, the next
    /// event re-keys so the older ones can go, and `Stats::over_budget`
    /// counts it.
    pub max_bytes: usize,
    /// the semantic keyframe cadence: a changed snapshot is stored whole
    /// when the segment it would join opened this long ago or more, and
    /// any other event arriving then first re-keys the track with the
    /// state as it stands (an unchanged state shares its snapshot).
    pub full_every: Millis,
    /// the visual keyframe cadence: a frame this long after the open
    /// chunk started closes it and opens the next.
    pub chunk_every: Millis,
    /// ... or a frame arriving when the open chunk holds this many.
    pub chunk_frames: usize,
}

impl Default for Limits {
    /// the last 20 seconds, in at most 32 MiB, keyed every 2 seconds on
    /// both tracks, at most 120 frames per chunk.
    fn default() -> Self {
        Limits {
            max_age: 20_000,
            max_bytes: 32 << 20,
            full_every: 2_000,
            chunk_every: 2_000,
            chunk_frames: 120,
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
    /// times the newest segment and chunk alone were over the budget.
    pub over_budget: u64,
    /// frames taken.
    pub frames: u64,
    /// frames equal to the one before, not stored.
    pub frames_unchanged: u64,
    /// frames the IR limits refused (see `FrameError`).
    pub frame_errors: u64,
    /// compositor nodes the monster IR v0 has no node for, per kind, in
    /// the order of `monster::Unsupported::ALL` (clip push, clip pop,
    /// backdrop blur, shadow): the replay draws without them.
    pub skipped: [u64; 4],
    /// layers drawn with an opacity, an effect or a clip rect the IR v0
    /// does not carry: their nodes are recorded, the layer look is not.
    pub layers_approximated: u64,
    /// visual chunks closed.
    pub chunks_closed: u64,
    /// frames whose bindings were recorded whole: the first of a chunk
    /// that binds anything, and any whose ids or their order changed.
    pub bindings_whole: u64,
    /// frames whose bindings were recorded as the rects that moved (the
    /// same ids in the same order); a frame that binds as the one before
    /// records nothing.
    pub bindings_moved: u64,
    /// visual chunks the ring dropped, by time or by bytes.
    pub evicted_chunks: u64,
    /// segments of the semantic track now held.
    pub segments: usize,
    /// visual chunks now held, the open one included.
    pub chunks: usize,
    /// the estimated bytes of the semantic track now held.
    pub semantic_bytes: usize,
    /// the estimated bytes of the visual track now held: encoded chunks,
    /// the open chunk and the assets.
    pub visual_bytes: usize,
    /// the part of `visual_bytes` that is assets.
    pub asset_bytes: usize,
    /// the estimated bytes the ring holds now, every track together.
    pub bytes: usize,
}
