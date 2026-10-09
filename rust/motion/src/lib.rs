//! monster: binary animation codec for plev. the poetic frame: h264 for
//! vectors; keyframes are I-frames, interframes are discovered deltas,
//! the renderer interpolates (docs/ref/monster-format-v0.md).
//!
//! v0 ground work, backend before ui:
//! - [`ir`]: the codec's own node and timeline model, decoupled from
//!   `engine::SceneNode` so the frozen format never chases the internal
//!   enum; the player lowers `timeline::Node` to `SceneNode` at render time
//! - [`easing`]: presets mirroring `engine::animation::Easing` plus the
//!   1-byte wire ids and the custom cubic bezier escape (0xFF)
//! - [`quant`]: pure quantization primitives, f32 in memory and
//!   integers in the file (twips, rgba8, u16 fixed, bezier u8)
//!
//! - [`container`]: the frozen v0 wire layout (header, asset and easing
//!   tables, section index with sha256, K/D/T payloads), LE byte-aligned
//! - [`write`]: encoder mode A, lowering an authored timeline; golden
//!   fixture frozen at fixtures/golden_v0_minimal.monster
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
//! - [`raise`]: the mirror of lower, `SceneNode` -> IR for the kinds
//!   the IR carries, the others reported; feeds a recorder's frames to
//!   discover
//! - [`asset_path`]: the payload layout behind `AssetKind::Path`
//!   (uniform color, twips vertices, u16 indices; deterministic chunk
//!   split under the u16 asset limit), so importers pack and players
//!   unpack tessellated geometry through one codec
//!
//! the `script` cargo feature is reserved for the rhai sidecar and is
//! intentionally empty in v0.

pub mod discover;
pub mod encoding;
pub mod mappings;
pub mod optimize;
pub mod play;
pub mod timeline;

pub use discover::fit::quantize_value;
pub use discover::{DiscoverConfig, DiscoverError, discover};
pub use encoding::container::{Asset, AssetKind, Desc, stage_size};
pub use encoding::read::{Document, ReadError, decode};
pub use encoding::write::{WriteError, encode};
pub use mappings::lower::LoweredAsset;
pub use mappings::raise::{
    AssetBank, MAX_RAISED, RaiseError, Raised, Skipped, Unsupported, raise_node, raise_scene,
};
pub use optimize::{OptimizeCfg, optimize};
pub use play::MonsterPlayer;
pub use timeline::easing::Easing;
pub use timeline::{
    AssetId, Depth, IrError, Keyframe, Node, NodeId, NodeKind, PlaceNode, Prop, Props, RemoveNode,
    ReplaceNode, Segment, Timeline, Track, Value,
};
