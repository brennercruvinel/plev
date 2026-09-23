//! `Screen`, the read side of the visual track: the chunks a window
//! holds, the assets they reference once each, played back with
//! monster's own player. a time reads the frame taken last at or before
//! it, never a value between two frames: render on demand draws a frame
//! only when something changed, so between two frames the screen held
//! the earlier one.
//!
//! the shape is the file's: chunks with their frame times and asset refs
//! (section I), their encoded bytes (section C), one asset table
//! (section A).

use std::collections::HashMap;
use std::sync::Arc;

use engine::compositor::SceneNode;
use engine::gpu::ImageHandle;
use engine::path::TessellatedPath;
use monster::{LoweredAsset, MonsterPlayer};
use sem::Millis;

use crate::error::ReadError;
use crate::visual::{AssetKey, Visual, seconds};

/// one asset of a window.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Asset {
    /// resolvable in this process: a text style, a path, or an image of
    /// this process's atlas.
    Live(LoweredAsset),
    /// an image read from a file: its size, its pixels when the file
    /// carried them, and the handle they were loaded under in this
    /// process. without pixels there is no handle, and the node draws
    /// nothing rather than another image's pixels.
    Image {
        width: u32,
        height: u32,
        pixels: Option<Arc<[u8]>>,
        handle: Option<ImageHandle>,
    },
}

impl Asset {
    fn lowered(&self) -> LoweredAsset {
        match self {
            Asset::Live(asset) => asset.clone(),
            Asset::Image {
                handle: Some(h), ..
            } => LoweredAsset::Image(*h),
            Asset::Image { handle: None, .. } => lost(),
        }
    }
}

/// one chunk: where it starts, the frames it took (ms from the start),
/// its monster bytes, and the asset behind each of its asset ids.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Shown {
    pub(crate) start: Millis,
    pub(crate) times: Vec<u32>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) refs: Vec<u32>,
}

/// the visual track of a window.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Screen {
    pub(crate) chunks: Vec<Shown>,
    pub(crate) assets: Vec<Asset>,
}

/// what lowering reads for an asset that cannot be drawn: an empty path,
/// which draws nothing and leaves a node of another kind mistyped, so it
/// lowers to nothing, deterministically.
fn lost() -> LoweredAsset {
    LoweredAsset::Path(TessellatedPath {
        vertices: Vec::new(),
        indices: Vec::new(),
        hash: 0,
    })
}

/// an asset table under construction, one entry per content.
#[derive(Default)]
struct Table {
    assets: Vec<Asset>,
    index: HashMap<AssetKey, u32>,
}

impl Table {
    fn add(&mut self, key: AssetKey, asset: impl FnOnce() -> LoweredAsset) -> u32 {
        let next = u32::try_from(self.assets.len()).unwrap_or(u32::MAX);
        *self.index.entry(key).or_insert_with(|| {
            self.assets.push(Asset::Live(asset()));
            next
        })
    }
}

impl Screen {
    /// a copy of what the visual track holds, the open chunk encoded as
    /// it stands.
    pub(crate) fn of(visual: &Visual) -> Screen {
        let mut table = Table::default();
        let mut chunks = Vec::new();
        for c in visual.chunks() {
            let refs = c
                .assets
                .iter()
                .map(|key| table.add((**key).clone(), || visual.resolve(key).unwrap_or_else(lost)))
                .collect();
            chunks.push(Shown {
                start: c.start,
                times: c.times.clone(),
                bytes: c.bytes.clone(),
                refs,
            });
        }
        for (c, assets) in visual.open_encoded() {
            let refs = assets
                .into_iter()
                .map(|asset| table.add(AssetKey::of(&asset), || asset))
                .collect();
            chunks.push(Shown {
                start: c.start,
                times: c.times,
                bytes: c.bytes,
                refs,
            });
        }
        Screen {
            chunks,
            assets: table.assets,
        }
    }

    /// the chunk and the frame time on screen at `t`.
    fn locate(&self, t: Millis) -> Option<(&Shown, u32)> {
        let n = self.chunks.partition_point(|c| c.start <= t);
        let chunk = self.chunks.get(n.checked_sub(1)?)?;
        let rel = u32::try_from(t.saturating_sub(chunk.start)).unwrap_or(u32::MAX);
        let k = chunk.times.partition_point(|&at| at <= rel);
        let at = *chunk.times.get(k.checked_sub(1)?)?;
        Some((chunk, at))
    }

    /// the time of the frame on screen at `t`.
    pub(crate) fn frame_at(&self, t: Millis) -> Option<Millis> {
        self.locate(t)
            .map(|(c, at)| c.start.saturating_add(Millis::from(at)))
    }

    /// the frame on screen at `t`, lowered to compositor nodes.
    pub(crate) fn scene_at(&self, t: Millis) -> Result<Option<Vec<SceneNode>>, ReadError> {
        let Some((chunk, at)) = self.locate(t) else {
            return Ok(None);
        };
        let doc = monster::decode(&chunk.bytes).map_err(|e| ReadError::Chunk(e.to_string()))?;
        let mut player =
            MonsterPlayer::new(doc.timeline).map_err(|e| ReadError::Chunk(e.to_string()))?;
        let assets = chunk
            .refs
            .iter()
            .map(|&r| {
                usize::try_from(r)
                    .ok()
                    .and_then(|r| self.assets.get(r))
                    .map_or_else(lost, Asset::lowered)
            })
            .collect();
        player.set_assets(assets);
        Ok(Some(player.scene_at(seconds(at))))
    }

    /// the visual keyframes: where each chunk starts.
    pub(crate) fn keys(&self) -> Vec<Millis> {
        self.chunks.iter().map(|c| c.start).collect()
    }

    /// the first and the last frame time held.
    pub(crate) fn span(&self) -> Option<(Millis, Millis)> {
        let first = self.chunks.first()?;
        let last = self.chunks.last()?;
        let end = last.start.saturating_add(Millis::from(*last.times.last()?));
        Some((first.start, end))
    }

    /// the frame times held, in order.
    pub(crate) fn times(&self) -> Vec<Millis> {
        self.chunks
            .iter()
            .flat_map(|c| {
                c.times
                    .iter()
                    .map(move |&at| c.start.saturating_add(Millis::from(at)))
            })
            .collect()
    }

    /// the encoded bytes and the asset count of every chunk.
    pub(crate) fn sizes(&self) -> Vec<(usize, usize)> {
        self.chunks
            .iter()
            .map(|c| (c.bytes.len(), c.refs.len()))
            .collect()
    }
}
