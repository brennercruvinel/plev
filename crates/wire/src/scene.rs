//! the scene channel (task-79, docs/ADR/scene-travels-as-data.md): a
//! client watches the scene of one of the app's windows and gets its
//! frames, the first whole and then only what changed, on the same
//! connection and in protocol v1, as replies to its `scene` request.
//!
//! ```text
//! -> {"v":1,"id":"9","type":"scene","window":"main"}
//! <- {"v":1,"id":"9","type":"scene_blob","key":"8a1f...","at":0,"total":24012,"bytes":"AQAAAA..."}
//! <- {"v":1,"id":"9","type":"scene","seq":1,"head":"AAAgRA...","layers":[{"key":0,"hash":"3c9e...","body":"AAAAAA..."},...],"bounds":[...]}
//! <- {"v":1,"id":"9","type":"scene","seq":2,"layers":[{"key":0,"hash":"77d0...","patch":"AAAAAA..."},{"key":1,"hash":"51aa..."}]}
//! ```
//!
//! the transport does not read a frame: the app's codec (`rec::scene`)
//! writes each layer with a hash, a body and maybe a patch against the
//! frame before, and each asset a frame references under a key; the
//! server keeps, per watch, the hash of each layer and the assets it
//! sent, and sends a layer only when its hash moved (the patch when the
//! watch holds the patch's base, the body otherwise), an asset only when
//! the watch does not hold it, the head and the bindings (`bounds`, where
//! each control shows, the form an observation serves) only when they
//! changed. a frame where nothing moved for a watch sends nothing.
//!
//! binary parts travel as base64 (`b64`). an asset, or a body that would
//! not fit a frame's line, goes before the frame in `scene_blob` lines of
//! at most `SCENE_PART` bytes, each line under `MAX_LINE`, the limit a
//! client's line has; a layer whose line carries neither body nor patch
//! is the one the watch holds or the blob just sent under its hash. the
//! watch holds an asset while each frame references it, the rule the
//! receiver keeps too. a watch lasts as long as its connection.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::b64;
use crate::proto::{Bound, ProtoError, Reply, ServerMsg};
use crate::server::{ConnId, Outgoing};

/// the longest line a client may send, in bytes; the scene channel
/// keeps every line it writes under it too.
pub const MAX_LINE: usize = 1 << 20;

/// the most bytes of a blob one `scene_blob` line carries: 384 KiB, 512
/// KiB as base64, half a line.
pub const SCENE_PART: usize = 3 << 17;

/// the window a `scene` request without one watches.
pub const MAIN_WINDOW: &str = "main";

/// the longest window name, in characters.
pub const MAX_WINDOW: usize = 32;

/// a watch of one window's scene.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneAsk {
    #[serde(default = "main_window")]
    pub window: String,
}

fn main_window() -> String {
    MAIN_WINDOW.to_owned()
}

/// one frame for one watch. `seq` counts the frames sent to it, from 1.
/// `head` (the codec's: size, scale, clear) and `bounds` come when they
/// changed, and always in the first frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneReply {
    pub seq: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "b64::opt_bytes"
    )]
    pub head: Option<Vec<u8>>,
    pub layers: Vec<SceneLayer>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Vec<Bound>>,
}

/// one layer of a frame, in composite order: the app's layer id, the hash
/// of its body, and what the watch needs of it (nothing when it holds
/// that hash, or when the body came as a blob under the hash).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SceneLayer {
    pub key: u32,
    #[serde(with = "b64::hex")]
    pub hash: u64,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "b64::opt_bytes"
    )]
    pub body: Option<Vec<u8>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "b64::opt_bytes"
    )]
    pub patch: Option<Vec<u8>>,
}

/// a part of a blob: an asset, or a layer's body too large for its
/// frame's line, `total` bytes under `key`, this part from byte `at`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlobReply {
    #[serde(with = "b64::hex")]
    pub key: u64,
    pub at: u64,
    pub total: u64,
    #[serde(with = "b64::bytes")]
    pub bytes: Vec<u8>,
}

/// one frame of a window as the app offers it to the watches: the head,
/// every layer in composite order, every asset the layers reference, and
/// where each bound control shows.
#[derive(Clone, Debug, Default)]
pub struct Offer {
    pub head: Arc<[u8]>,
    pub layers: Vec<OfferLayer>,
    pub assets: Vec<(u64, Arc<[u8]>)>,
    pub bounds: Vec<Bound>,
}

/// one layer offered: its key, its hash, its body and, when the codec
/// wrote one, a patch against the layer at hash `.0`.
#[derive(Clone, Debug)]
pub struct OfferLayer {
    pub key: u32,
    pub hash: u64,
    pub body: Arc<[u8]>,
    pub patch: Option<(u64, Arc<[u8]>)>,
}

/// the watches of one server.
#[derive(Debug, Default)]
pub(crate) struct Watches {
    by_conn: HashMap<ConnId, Watch>,
}

/// what one watch was sent.
#[derive(Debug)]
struct Watch {
    /// the id of the `scene` request.
    msg: String,
    window: String,
    seq: u64,
    head: Option<Arc<[u8]>>,
    layers: Vec<(u32, u64)>,
    assets: HashSet<u64>,
    bounds: Option<Vec<Bound>>,
}

impl Watches {
    /// `conn` watches `ask.window`, one watch per connection.
    pub(crate) fn watch(
        &mut self,
        conn: ConnId,
        msg: &str,
        ask: SceneAsk,
    ) -> Result<(), ProtoError> {
        let name = &ask.window;
        let fits = (1..=MAX_WINDOW).contains(&name.chars().count())
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !fits {
            return Err(ProtoError::BadMessage {
                reason: format!(
                    "a window is named by 1 to {MAX_WINDOW} characters of [A-Za-z0-9_-]; \
                     got {name:?}"
                ),
            });
        }
        if let Some(w) = self.by_conn.get(&conn) {
            return Err(ProtoError::BadMessage {
                reason: format!(
                    "this connection already watches the scene of window {:?} (request {:?}); \
                     one watch per connection",
                    w.window, w.msg
                ),
            });
        }
        self.by_conn.insert(
            conn,
            Watch {
                msg: msg.to_owned(),
                window: ask.window,
                seq: 0,
                head: None,
                layers: Vec::new(),
                assets: HashSet::new(),
                bounds: None,
            },
        );
        Ok(())
    }

    pub(crate) fn watching(&self, window: &str) -> bool {
        self.by_conn.values().any(|w| w.window == window)
    }

    pub(crate) fn close(&mut self, conn: ConnId) {
        self.by_conn.remove(&conn);
    }

    /// what each watch of `window` needs of `offer`, the watches in the
    /// order of their connections.
    pub(crate) fn offer(&mut self, window: &str, offer: &Offer) -> Vec<Outgoing> {
        let mut conns: Vec<ConnId> = self
            .by_conn
            .iter()
            .filter(|(_, w)| w.window == window)
            .map(|(c, _)| *c)
            .collect();
        conns.sort_unstable();
        let mut out = Vec::new();
        for conn in conns {
            if let Some(w) = self.by_conn.get_mut(&conn) {
                out.extend(w.next(offer).into_iter().map(|msg| Outgoing { conn, msg }));
            }
        }
        out
    }
}

impl Watch {
    /// the lines of one frame for this watch; none when nothing moved.
    fn next(&mut self, offer: &Offer) -> Vec<ServerMsg> {
        let held: HashMap<u32, u64> = self.layers.iter().copied().collect();
        let mut blobs: Vec<(u64, &[u8])> = offer
            .assets
            .iter()
            .filter(|(key, _)| !self.assets.contains(key))
            .map(|(key, bytes)| (*key, &bytes[..]))
            .collect();
        let mut layers: Vec<SceneLayer> = Vec::with_capacity(offer.layers.len());
        let mut inline: Vec<(usize, usize, u64, &[u8])> = Vec::new();
        for (at, l) in offer.layers.iter().enumerate() {
            let had = held.get(&l.key).copied();
            let mut layer = SceneLayer {
                key: l.key,
                hash: l.hash,
                body: None,
                patch: None,
            };
            if had != Some(l.hash) {
                match &l.patch {
                    Some((base, patch)) if had == Some(*base) => {
                        layer.patch = Some(patch.to_vec());
                        inline.push((patch.len(), at, l.hash, &l.body));
                    }
                    _ => {
                        layer.body = Some(l.body.to_vec());
                        inline.push((l.body.len(), at, l.hash, &l.body));
                    }
                }
            }
            layers.push(layer);
        }
        let order: Vec<(u32, u64)> = offer.layers.iter().map(|l| (l.key, l.hash)).collect();
        let head = (self.head.as_deref() != Some(&offer.head[..])).then(|| offer.head.to_vec());
        let bounds = (self.bounds.as_ref() != Some(&offer.bounds)).then(|| offer.bounds.clone());
        if order == self.layers && head.is_none() && bounds.is_none() && blobs.is_empty() {
            return Vec::new();
        }
        // a frame's line stays under MAX_LINE: the layers that weigh most
        // in it go before it, whole, as blobs under their hash
        let mut size = line_size(&layers, head.as_deref(), bounds.as_deref());
        inline.sort_by_key(|a| std::cmp::Reverse(a.0));
        for (len, at, hash, body) in inline {
            if size <= MAX_LINE {
                break;
            }
            layers[at].body = None;
            layers[at].patch = None;
            size -= len.div_ceil(3) * 4;
            blobs.push((hash, body));
        }
        let mut out = Vec::new();
        for (key, bytes) in blobs {
            out.extend(self.blob(key, bytes));
        }
        self.seq += 1;
        out.push(self.reply(Reply::Scene(SceneReply {
            seq: self.seq,
            head,
            layers,
            bounds,
        })));
        self.layers = order;
        self.head = Some(offer.head.clone());
        self.assets = offer.assets.iter().map(|(k, _)| *k).collect();
        self.bounds = Some(offer.bounds.clone());
        out
    }

    /// `bytes` under `key`, in lines of at most `SCENE_PART` bytes.
    fn blob(&self, key: u64, bytes: &[u8]) -> Vec<ServerMsg> {
        let total = bytes.len() as u64;
        if bytes.is_empty() {
            return vec![self.reply(Reply::SceneBlob(BlobReply {
                key,
                at: 0,
                total,
                bytes: Vec::new(),
            }))];
        }
        bytes
            .chunks(SCENE_PART)
            .enumerate()
            .map(|(n, part)| {
                self.reply(Reply::SceneBlob(BlobReply {
                    key,
                    at: (n * SCENE_PART) as u64,
                    total,
                    bytes: part.to_vec(),
                }))
            })
            .collect()
    }

    fn reply(&self, reply: Reply) -> ServerMsg {
        ServerMsg {
            id: Some(self.msg.clone()),
            reply,
        }
    }
}

/// an upper bound of a frame's line, in bytes: the base64 it carries and
/// a generous allowance for the JSON around each piece.
fn line_size(layers: &[SceneLayer], head: Option<&[u8]>, bounds: Option<&[Bound]>) -> usize {
    let b64 = |n: usize| n.div_ceil(3) * 4;
    let blob = |v: &Option<Vec<u8>>| v.as_ref().map_or(0, |v| b64(v.len()));
    256 + head.map_or(0, |h| b64(h.len()))
        + layers
            .iter()
            .map(|l| 96 + blob(&l.body) + blob(&l.patch))
            .sum::<usize>()
        + bounds.map_or(0, |b| {
            b.iter()
                .map(|b| 160 + 2 * b.id.as_str().len())
                .sum::<usize>()
        })
}
