//! anm: binary animation codec for plev. the poetic frame: h264 for
//! vectors; keyframes are I-frames, interframes are discovered deltas,
//! the renderer interpolates (doc/anm-format-v0.md).
//!
//! v0 ground work, backend before ui:
//! - [`ir`]: the codec's own node and timeline model, decoupled from
//!   `plev::SceneNode` so the frozen format never chases the internal
//!   enum; the player lowers `ir::Node` to `SceneNode` at render time
//! - [`easing`]: presets mirroring `plev::animation::Easing` plus the
//!   1-byte wire ids and the custom cubic bezier escape (0xFF)
//! - [`quant`]: pure quantization primitives, f32 in memory and
//!   integers in the file (twips, rgba8, u16 fixed, bezier u8)
//!
//! - [`container`]: the frozen v0 wire layout (header, asset and easing
//!   tables, section index with sha256, K/D/T payloads), LE byte-aligned
//! - [`write`]: encoder mode A, lowering an authored timeline; golden
//!   fixture frozen at fixtures/golden_v0_minimal.anm
//! - [`discover`]: encoder mode B, delta discovery from a sampled frame
//!   sequence: snapshots in, structural ops, linear segment chains and
//!   random-access keyframes out; the result feeds [`write::encode`]
//! - [`optimize`]: encoder-side passes over any timeline (authored or
//!   discovered): static track collapse, RDP keyframe reduction and
//!   collinear segment fusion, tolerances in wire quantization steps
//! - [`read`]: strict decoder back to the IR; typed errors, never a
//!   panic on malformed input, round-trips the encoder output
//! - [`play`]: deterministic player per the spec's player contract:
//!   driven by `AnimationTick`, windowed evaluation, O(1) seek,
//!   reactive play/pause/scrub surface via plev signals
//! - [`lower`]: IR scene -> `SceneNode` mapping plus the
//!   `LoweredAsset` bank for asset-backed nodes
//!
//! the `script` cargo feature is reserved for the rhai sidecar and is
//! intentionally empty in v0.

pub mod container;
pub mod discover;
mod discover_fit;
pub mod easing;
pub mod ir;
pub mod lower;
pub mod optimize;
pub mod play;
mod play_eval;
pub mod quant;
pub mod read;
mod read_sec;
mod validate;
pub mod write;

pub use container::{Asset, AssetKind, Desc};
pub use discover::{DiscoverConfig, DiscoverError, discover};
pub use discover_fit::quantize_value;
pub use easing::Easing;
pub use ir::{
    AssetId, Depth, IrError, Keyframe, Node, NodeId, NodeKind, PlaceNode, Prop, Props, RemoveNode,
    ReplaceNode, Segment, Timeline, Track, Value,
};
pub use lower::LoweredAsset;
pub use optimize::{OptimizeCfg, optimize};
pub use play::AnmPlayer;
pub use read::{Document, ReadError, decode};
pub use write::{WriteError, encode};

#[cfg(test)]
mod tests_discover;
#[cfg(test)]
mod tests_discover_ops;
#[cfg(test)]
mod tests_easing;
#[cfg(test)]
mod tests_golden;
#[cfg(test)]
mod tests_ir;
#[cfg(test)]
mod tests_lower;
#[cfg(test)]
mod tests_ops;
#[cfg(test)]
mod tests_optimize;
#[cfg(test)]
mod tests_optimize_pipe;
#[cfg(test)]
mod tests_play;
#[cfg(test)]
mod tests_play_ops;
#[cfg(test)]
mod tests_prop;
#[cfg(test)]
mod tests_quant;
#[cfg(test)]
mod tests_read;
#[cfg(test)]
mod tests_read_malformed;
#[cfg(test)]
mod tests_write;
