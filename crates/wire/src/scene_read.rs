//! the watcher's side of the scene channel: a watch's lines put back
//! together into frames. a blob's parts come in order and add up to its
//! total, at most `MAX_BLOB` bytes; a frame takes, for each layer that
//! carries neither body nor patch, the blob just sent under its hash, and
//! hands every other blob that came before it on as an asset.

use std::collections::HashMap;

use crate::proto::{Bound, ProtoError, Reply};
use crate::scene::{BlobReply, SceneReply};

/// the largest blob a reader takes: an image the engine's atlas holds
/// (8190 x 8190 RGBA8) and its header.
pub const MAX_BLOB: u64 = 8192 * 8192 * 4 + 64;

/// what one layer of a frame brings.
#[derive(Clone, Debug, PartialEq)]
pub enum LayerBody {
    /// nothing: the watcher holds the layer at this hash.
    Held,
    Body(Vec<u8>),
    Patch(Vec<u8>),
}

/// one layer of a frame, in composite order.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneLayerIn {
    pub key: u32,
    pub hash: u64,
    pub body: LayerBody,
}

/// one frame put back together.
#[derive(Clone, Debug, PartialEq)]
pub struct SceneFrame {
    pub seq: u64,
    pub head: Option<Vec<u8>>,
    pub layers: Vec<SceneLayerIn>,
    /// the assets that came before it, in order, under their keys.
    pub assets: Vec<(u64, Vec<u8>)>,
    pub bounds: Option<Vec<Bound>>,
}

/// puts a watch's lines back together.
#[derive(Debug, Default)]
pub struct SceneReader {
    partial: Option<(u64, u64, Vec<u8>)>,
    done: Vec<(u64, Vec<u8>)>,
}

impl SceneReader {
    pub fn new() -> SceneReader {
        SceneReader::default()
    }

    /// takes one reply of the watch: a frame when it completes one,
    /// `None` for a blob's part (and for any reply that is not the
    /// scene's). an error for parts out of order or past `MAX_BLOB`.
    pub fn take(&mut self, reply: Reply) -> Result<Option<SceneFrame>, ProtoError> {
        match reply {
            Reply::SceneBlob(part) => self.part(part).map(|()| None),
            Reply::Scene(frame) => self.frame(frame).map(Some),
            _ => Ok(None),
        }
    }

    fn part(&mut self, p: BlobReply) -> Result<(), ProtoError> {
        let bad = |reason: String| ProtoError::BadMessage { reason };
        if p.total > MAX_BLOB {
            return Err(bad(format!(
                "a blob of {} bytes is past the {MAX_BLOB} a reader takes",
                p.total
            )));
        }
        let (key, total, mut bytes) = match self.partial.take() {
            Some((key, total, bytes)) if key == p.key => (key, total, bytes),
            Some((key, ..)) => {
                return Err(bad(format!(
                    "blob {key:016x} was cut by blob {:016x}",
                    p.key
                )));
            }
            None => (p.key, p.total, Vec::new()),
        };
        if p.total != total || p.at != bytes.len() as u64 {
            return Err(bad(format!(
                "blob {key:016x}: a part at {} of {} after {} of {total} bytes",
                p.at,
                p.total,
                bytes.len()
            )));
        }
        bytes.extend_from_slice(&p.bytes);
        match (bytes.len() as u64).cmp(&total) {
            std::cmp::Ordering::Less => self.partial = Some((key, total, bytes)),
            std::cmp::Ordering::Equal => self.done.push((key, bytes)),
            std::cmp::Ordering::Greater => {
                return Err(bad(format!("blob {key:016x} runs past its {total} bytes")));
            }
        }
        Ok(())
    }

    fn frame(&mut self, f: SceneReply) -> Result<SceneFrame, ProtoError> {
        if let Some((key, ..)) = &self.partial {
            return Err(ProtoError::BadMessage {
                reason: format!("frame {} came inside blob {key:016x}", f.seq),
            });
        }
        let mut blobs: HashMap<u64, Vec<u8>> = HashMap::new();
        let mut order = Vec::new();
        for (key, bytes) in self.done.drain(..) {
            order.push(key);
            blobs.insert(key, bytes);
        }
        let layers = f
            .layers
            .into_iter()
            .map(|l| {
                let body = match (l.body, l.patch) {
                    (Some(body), _) => LayerBody::Body(body),
                    (None, Some(patch)) => LayerBody::Patch(patch),
                    (None, None) => blobs
                        .remove(&l.hash)
                        .map_or(LayerBody::Held, LayerBody::Body),
                };
                SceneLayerIn {
                    key: l.key,
                    hash: l.hash,
                    body,
                }
            })
            .collect();
        let assets = order
            .into_iter()
            .filter_map(|key| blobs.remove(&key).map(|bytes| (key, bytes)))
            .collect();
        Ok(SceneFrame {
            seq: f.seq,
            head: f.head,
            layers,
            assets,
            bounds: f.bounds,
        })
    }
}
