//! `Screen`, the read side of the visual track: the chunks a window
//! holds, each with the assets it references resolved, played back with
//! monster's own player. a time reads the frame taken last at or before
//! it, never a value between two frames: render on demand draws a frame
//! only when something changed, so between two frames the screen held
//! the earlier one.

use engine::compositor::SceneNode;
use engine::path::TessellatedPath;
use monster::{LoweredAsset, MonsterPlayer};
use sem::Millis;

use crate::error::ReadError;
use crate::visual::{Visual, seconds};

#[derive(Clone, Debug, PartialEq)]
struct Shown {
    start: Millis,
    times: Vec<u32>,
    bytes: Vec<u8>,
    assets: Vec<LoweredAsset>,
}

/// the visual track of a window.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Screen {
    chunks: Vec<Shown>,
}

/// what lowering reads for an asset the store no longer holds, which
/// the reference counts never let happen: an empty path, which draws
/// nothing and leaves a node of another kind mistyped, so it lowers to
/// nothing, deterministically.
fn lost() -> LoweredAsset {
    LoweredAsset::Path(TessellatedPath {
        vertices: Vec::new(),
        indices: Vec::new(),
        hash: 0,
    })
}

impl Screen {
    /// a copy of what the visual track holds, the open chunk encoded as
    /// it stands.
    pub(crate) fn of(visual: &Visual) -> Screen {
        let mut chunks: Vec<Shown> = visual
            .chunks()
            .iter()
            .map(|c| Shown {
                start: c.start,
                times: c.times.clone(),
                bytes: c.bytes.clone(),
                assets: c
                    .assets
                    .iter()
                    .map(|key| visual.resolve(key).unwrap_or_else(lost))
                    .collect(),
            })
            .collect();
        for (c, assets) in visual.open_encoded() {
            chunks.push(Shown {
                start: c.start,
                times: c.times,
                bytes: c.bytes,
                assets,
            });
        }
        Screen { chunks }
    }

    /// the chunk and the frame time on screen at `t`.
    fn locate(&self, t: Millis) -> Option<(&Shown, u32)> {
        let n = self.chunks.partition_point(|c| c.start <= t);
        let chunk = self.chunks.get(n.checked_sub(1)?)?;
        let rel = u32::try_from(t - chunk.start).unwrap_or(u32::MAX);
        let k = chunk.times.partition_point(|&at| at <= rel);
        let at = *chunk.times.get(k.checked_sub(1)?)?;
        Some((chunk, at))
    }

    /// the time of the frame on screen at `t`.
    pub(crate) fn frame_at(&self, t: Millis) -> Option<Millis> {
        self.locate(t).map(|(c, at)| c.start + Millis::from(at))
    }

    /// the frame on screen at `t`, lowered to compositor nodes.
    pub(crate) fn scene_at(&self, t: Millis) -> Result<Option<Vec<SceneNode>>, ReadError> {
        let Some((chunk, at)) = self.locate(t) else {
            return Ok(None);
        };
        let doc = monster::decode(&chunk.bytes).map_err(|e| ReadError::Chunk(e.to_string()))?;
        let mut player =
            MonsterPlayer::new(doc.timeline).map_err(|e| ReadError::Chunk(e.to_string()))?;
        player.set_assets(chunk.assets.clone());
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
        let end = last.start + Millis::from(*last.times.last()?);
        Some((first.start, end))
    }

    /// the frame times held, in order.
    pub(crate) fn times(&self) -> Vec<Millis> {
        self.chunks
            .iter()
            .flat_map(|c| c.times.iter().map(move |&at| c.start + Millis::from(at)))
            .collect()
    }

    /// the encoded bytes and the asset count of every chunk.
    pub(crate) fn sizes(&self) -> Vec<(usize, usize)> {
        self.chunks
            .iter()
            .map(|c| (c.bytes.len(), c.assets.len()))
            .collect()
    }
}
