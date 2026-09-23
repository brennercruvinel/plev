//! the visual track: what was on screen, as monster data. every rendered
//! frame (render on demand: a frame only when something changed) is
//! raised to the monster IR (`monster::raise_scene`) and kept in the open
//! chunk; a chunk closes on its cadence, at a frame count or when its
//! asset bank is full, and closing runs monster's delta discovery and
//! encodes the result as a v0 file with an empty asset table. a chunk is
//! the keyframe unit of the track: it opens at a full snapshot of the
//! scene, so the ring cuts between chunks and what it keeps always opens
//! at a visual keyframe.
//!
//! discovery is told that no step is a discontinuity, so a changed
//! value between two frames becomes one segment, not a whole-scene
//! keyframe; the reader evaluates a chunk only at the times a frame was
//! taken (`Screen`), which discovery reproduces exactly on the wire grid,
//! so the segment never shows a value that was not on screen.
//!
//! assets live beside the chunks, once per content (a text key, an image
//! handle, a path hash), counted by the chunks that reference them and
//! dropped with the last one: the ring holds only the assets the window
//! it keeps references.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::mem::size_of;
use std::sync::Arc;

use engine::compositor::{SceneNode, TextNodeKey};
use engine::gpu::ImageHandle;
use engine::path::TessellatedPath;
use monster::{AssetBank, DiscoverConfig, LoweredAsset, Node, RaiseError, Unsupported};
use sem::Millis;

use crate::error::FrameError;

/// what an asset is, by content.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum AssetKey {
    Text(TextNodeKey),
    Image(ImageHandle),
    Path(u64),
}

impl AssetKey {
    fn of(asset: &LoweredAsset) -> AssetKey {
        match asset {
            LoweredAsset::TextStyle(key) => AssetKey::Text(key.clone()),
            LoweredAsset::Image(handle) => AssetKey::Image(*handle),
            LoweredAsset::Path(path) => AssetKey::Path(path.hash),
        }
    }
}

/// the estimated bytes of one asset, as the store holds it.
pub(crate) fn asset_weight(asset: &LoweredAsset) -> usize {
    size_of::<AssetKey>()
        + match asset {
            LoweredAsset::TextStyle(key) => {
                key.text.len() + key.font_family.as_ref().map_or(0, String::len)
            }
            LoweredAsset::Image(_) => 0,
            LoweredAsset::Path(path) => {
                size_of::<TessellatedPath>()
                    + path.vertices.len() * size_of::<engine::compositor::QuadVertex>()
                    + path.indices.len() * size_of::<u32>()
            }
        }
}

fn node_weight(node: &Node) -> usize {
    size_of::<Node>() + node.props.len() * size_of::<(monster::Prop, monster::Value)>()
}

/// one closed chunk: a monster v0 file of the frames it took.
#[derive(Clone, Debug)]
pub(crate) struct Chunk {
    /// the time of its first frame, its keyframe.
    pub(crate) start: Millis,
    /// the times frames were taken, from `start`: the only instants the
    /// reader evaluates.
    pub(crate) times: Vec<u32>,
    /// keyframes and discovered deltas, encoded; asset ids index `assets`.
    pub(crate) bytes: Vec<u8>,
    /// the asset behind each id the chunk's nodes use.
    pub(crate) assets: Vec<Arc<AssetKey>>,
}

impl Chunk {
    fn weight(&self) -> usize {
        size_of::<Chunk>()
            + self.bytes.len()
            + self.times.len() * size_of::<u32>()
            + self.assets.len() * size_of::<Arc<AssetKey>>()
    }
}

/// the chunk being filled.
#[derive(Debug)]
struct Open {
    start: Millis,
    frames: Vec<(Millis, Vec<Node>)>,
    bank: AssetBank,
    bytes: usize,
}

impl Open {
    fn new(start: Millis) -> Self {
        Open {
            start,
            frames: Vec::new(),
            bank: AssetBank::new(),
            bytes: size_of::<Open>(),
        }
    }
}

#[derive(Debug, Default)]
struct Slot {
    refs: usize,
    path: Option<TessellatedPath>,
    bytes: usize,
}

/// how the visual track keys itself.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Cadence {
    pub(crate) every: Millis,
    pub(crate) frames: usize,
}

/// what the visual track counted.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Counts {
    pub(crate) frames: u64,
    pub(crate) unchanged: u64,
    pub(crate) closed: u64,
    pub(crate) skipped: [u64; 4],
    pub(crate) errors: u64,
}

#[derive(Debug, Default)]
pub(crate) struct Visual {
    chunks: VecDeque<Chunk>,
    open: Option<Open>,
    store: HashMap<Arc<AssetKey>, Slot>,
    /// closed chunks and stored assets.
    bytes: usize,
    /// a limit could not cut: the next frame opens a new chunk.
    close_next: bool,
    pub(crate) counts: Counts,
}

impl Visual {
    /// takes one frame at `t` (already ordered by the recorder's clock).
    pub(crate) fn frame<'a>(
        &mut self,
        t: Millis,
        scene: impl Iterator<Item = &'a SceneNode> + Clone,
        cadence: Cadence,
    ) -> Result<(), FrameError> {
        let due = self.open.as_ref().is_some_and(|open| {
            self.close_next
                || t.saturating_sub(open.start) >= cadence.every
                || open.frames.len() >= cadence.frames
        });
        if due {
            self.close();
        }
        let raised = match self.raise(t, scene.clone()) {
            Err(RaiseError::TooManyAssets) => {
                // a full bank: the chunk closes and the frame opens the next.
                self.close();
                self.raise(t, scene)
            }
            other => other,
        };
        let raised = match raised {
            Ok(raised) => raised,
            Err(e) => {
                if self.open.as_ref().is_some_and(|o| o.frames.is_empty()) {
                    self.open = None;
                }
                self.counts.errors += 1;
                return Err(e.into());
            }
        };
        for skipped in &raised.skipped {
            if let Some(at) = Unsupported::ALL.iter().position(|k| *k == skipped.kind) {
                self.counts.skipped[at] += 1;
            }
        }
        self.counts.frames += 1;
        let Some(open) = self.open.as_mut() else {
            return Ok(());
        };
        let nodes = raised.nodes;
        match open.frames.last_mut() {
            Some((_, last)) if *last == nodes => self.counts.unchanged += 1,
            Some((at, last)) if *at == t => {
                open.bytes -= last.iter().map(node_weight).sum::<usize>();
                open.bytes += nodes.iter().map(node_weight).sum::<usize>();
                *last = nodes;
            }
            _ => {
                open.bytes += size_of::<(Millis, Vec<Node>)>();
                open.bytes += nodes.iter().map(node_weight).sum::<usize>();
                open.frames.push((t, nodes));
            }
        }
        Ok(())
    }

    /// raises a frame into the open chunk's bank, opening a chunk at `t`
    /// when none is open.
    fn raise<'a>(
        &mut self,
        t: Millis,
        scene: impl Iterator<Item = &'a SceneNode>,
    ) -> Result<monster::Raised, RaiseError> {
        let open = self.open.get_or_insert_with(|| Open::new(t));
        let before = open.bank.len();
        let raised = monster::raise_scene(scene, &mut open.bank)?;
        let added: usize = open.bank.assets()[before..].iter().map(asset_weight).sum();
        open.bytes += added;
        Ok(raised)
    }

    /// closes the open chunk: discovery, encoding, the assets into the
    /// store.
    pub(crate) fn close(&mut self) {
        self.close_next = false;
        let Some(open) = self.open.take() else {
            return;
        };
        if open.frames.is_empty() {
            return;
        }
        let assets: Vec<Arc<AssetKey>> = open
            .bank
            .into_assets()
            .into_iter()
            .map(|asset| self.hold(asset))
            .collect();
        let pieces = encode(&open.frames);
        if pieces.is_empty() {
            self.counts.errors += 1;
        }
        for (start, times, bytes) in pieces {
            let chunk = Chunk {
                start,
                times,
                bytes,
                assets: assets.clone(),
            };
            for key in &chunk.assets {
                self.retain(key);
            }
            self.bytes += chunk.weight();
            self.chunks.push_back(chunk);
            self.counts.closed += 1;
        }
        for key in &assets {
            self.release(key);
        }
    }

    /// stores an asset (or finds it stored) and holds one reference.
    fn hold(&mut self, asset: LoweredAsset) -> Arc<AssetKey> {
        let weight = asset_weight(&asset);
        match self.store.entry(Arc::new(AssetKey::of(&asset))) {
            Entry::Occupied(mut slot) => {
                slot.get_mut().refs += 1;
                slot.key().clone()
            }
            Entry::Vacant(slot) => {
                let key = slot.key().clone();
                let path = match asset {
                    LoweredAsset::Path(path) => Some(path),
                    LoweredAsset::TextStyle(_) | LoweredAsset::Image(_) => None,
                };
                self.bytes += weight;
                slot.insert(Slot {
                    refs: 1,
                    path,
                    bytes: weight,
                });
                key
            }
        }
    }

    fn retain(&mut self, key: &Arc<AssetKey>) {
        if let Some(slot) = self.store.get_mut(key) {
            slot.refs += 1;
        }
    }

    /// drops one reference; the asset goes with the last one.
    fn release(&mut self, key: &Arc<AssetKey>) {
        let gone = match self.store.get_mut(key) {
            Some(slot) => {
                slot.refs = slot.refs.saturating_sub(1);
                slot.refs == 0
            }
            None => false,
        };
        if gone {
            if let Some(slot) = self.store.remove(key) {
                self.bytes -= slot.bytes;
            }
        }
    }

    /// the asset a key names, as lowering reads it.
    pub(crate) fn resolve(&self, key: &AssetKey) -> Option<LoweredAsset> {
        match key {
            AssetKey::Text(text) => Some(LoweredAsset::TextStyle(text.clone())),
            AssetKey::Image(handle) => Some(LoweredAsset::Image(*handle)),
            AssetKey::Path(_) => self
                .store
                .get(key)
                .and_then(|slot| slot.path.clone())
                .map(LoweredAsset::Path),
        }
    }

    /// drops the oldest closed chunk and the assets only it referenced.
    pub(crate) fn pop_front(&mut self) -> bool {
        let Some(chunk) = self.chunks.pop_front() else {
            return false;
        };
        self.bytes -= chunk.weight();
        for key in &chunk.assets {
            self.release(key);
        }
        true
    }

    /// the start of the chunk after the oldest one, the open one
    /// included: the oldest may go once this is old enough.
    pub(crate) fn second_start(&self) -> Option<Millis> {
        self.chunks
            .get(1)
            .map(|c| c.start)
            .or_else(|| self.chunks.front().and(self.open.as_ref()).map(|o| o.start))
    }

    /// the start of the oldest chunk held, the open one included.
    pub(crate) fn first_start(&self) -> Option<Millis> {
        self.chunks
            .front()
            .map(|c| c.start)
            .or_else(|| self.open.as_ref().map(|o| o.start))
    }

    /// asks the next frame to open a new chunk, so the ring can cut.
    pub(crate) fn close_soon(&mut self) {
        self.close_next = true;
    }

    /// closed chunks and the open one.
    pub(crate) fn chunk_count(&self) -> usize {
        self.chunks.len() + usize::from(self.open.is_some())
    }

    pub(crate) fn bytes(&self) -> usize {
        self.bytes + self.open.as_ref().map_or(0, |o| o.bytes)
    }

    pub(crate) fn asset_bytes(&self) -> usize {
        self.store.values().map(|s| s.bytes).sum()
    }

    pub(crate) fn chunks(&self) -> &VecDeque<Chunk> {
        &self.chunks
    }

    /// the open chunk encoded as it stands, with its assets, for a
    /// window read now; the track itself is left as it is.
    pub(crate) fn open_encoded(&self) -> Vec<(Chunk, Vec<LoweredAsset>)> {
        let Some(open) = &self.open else {
            return Vec::new();
        };
        encode(&open.frames)
            .into_iter()
            .map(|(start, times, bytes)| {
                let chunk = Chunk {
                    start,
                    times,
                    bytes,
                    assets: Vec::new(),
                };
                (chunk, open.bank.assets().to_vec())
            })
            .collect()
    }
}

/// discovery with every step a segment and the chunk as the only
/// keyframe: the reader evaluates only at frame times, where the
/// segments reproduce the frames exactly on the wire grid.
fn config() -> DiscoverConfig {
    DiscoverConfig {
        keyframe_every_s: f32::INFINITY,
        scalar_jump: f32::INFINITY,
        color_jump: f32::INFINITY,
    }
}

/// encodes frames as one chunk, or, when the codec refuses them (a u16
/// count over), as two halves, each encoded the same way. every piece
/// comes back with its start and its frame times.
fn encode(frames: &[(Millis, Vec<Node>)]) -> Vec<(Millis, Vec<u32>, Vec<u8>)> {
    let Some((start, _)) = frames.first() else {
        return Vec::new();
    };
    let start = *start;
    let rel = |t: Millis| u32::try_from(t.saturating_sub(start)).unwrap_or(u32::MAX);
    let samples: Vec<(f32, Vec<Node>)> = frames
        .iter()
        .map(|(t, nodes)| (seconds(rel(*t)), nodes.clone()))
        .collect();
    let encoded = monster::discover(&samples, &config())
        .ok()
        .and_then(|timeline| monster::encode(&timeline, &[], &[]).ok());
    match encoded {
        Some(bytes) => vec![(start, frames.iter().map(|(t, _)| rel(*t)).collect(), bytes)],
        None if frames.len() > 1 => {
            let (a, b) = frames.split_at(frames.len() / 2);
            let mut out = encode(a);
            out.extend(encode(b));
            out
        }
        None => Vec::new(),
    }
}

/// a chunk-relative time in the seconds the codec works in; the reader
/// evaluates at exactly these values.
pub(crate) fn seconds(ms: u32) -> f32 {
    ms as f32 / 1000.0
}
