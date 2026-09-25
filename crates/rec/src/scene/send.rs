//! the app's side: the frame a compositor holds, written layer by layer,
//! each layer with its hash, its body and, when the frame before had it,
//! a patch against that one; the assets its nodes reference, each built
//! once while consecutive frames reference it.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

use engine::compositor::{Compositor, Layer, SceneNode};

use super::asset::{self, Pixels};
use super::bytes::fnv;
use super::mask::{self, Mask};
use super::node::{self, Drawn, Head, Props};

/// one layer of a frame as the sender wrote it.
#[derive(Clone, Debug)]
pub struct LayerOut {
    /// the layer's id in the app's compositor.
    pub key: u32,
    /// FNV-1a 64 of `body`.
    pub hash: u64,
    pub body: Arc<[u8]>,
    /// the layer written against the frame before's: that one's hash and
    /// the patch, when the layer was there and the patch is the smaller.
    pub patch: Option<(u64, Arc<[u8]>)>,
}

/// one frame as the sender wrote it: the head, every layer in composite
/// order (by z-order, a tie in the order the compositor holds them, as
/// it composites them), every asset the layers reference (each once, in
/// the order they are first drawn).
#[derive(Clone, Debug)]
pub struct Frame {
    pub head: Arc<[u8]>,
    pub layers: Vec<LayerOut>,
    pub assets: Vec<(u64, Arc<[u8]>)>,
}

/// writes a window's frames. it keeps the frame before's layers, for the
/// patches, and its assets, so an asset is built (an image's pixels
/// copied out of the store) once while consecutive frames reference it.
#[derive(Debug, Default)]
pub struct Sender {
    layers: HashMap<u32, (u64, Vec<Drawn>)>,
    assets: HashMap<u64, Arc<[u8]>>,
}

impl Sender {
    pub fn new() -> Sender {
        Sender::default()
    }

    /// the frame `compositor` holds (the one on screen between two
    /// frames), laid out and cleared as `head` says.
    ///
    /// a fixed image travels with the pixels its slot holds
    /// (`engine::gpu::image_pixels`), once while frames reference it; a
    /// stream image (`engine::gpu::image_is_stream`: a camera's frames)
    /// travels as its size and place only, never its pixels, under a key
    /// its frames share, so a new frame of it moves nothing.
    pub fn frame(&mut self, compositor: &Compositor, head: Head) -> Frame {
        self.frame_masked(compositor, head, &[])
    }

    /// [`Sender::frame`], with the drawing of every place `masks` names
    /// left out of its layer and a box in its stead (`scene::mask`): what
    /// a secret draws is in no body, patch, hash or asset of the frame.
    pub fn frame_masked(&mut self, compositor: &Compositor, head: Head, masks: &[Mask]) -> Frame {
        let mut assets: Vec<(u64, Arc<[u8]>)> = Vec::new();
        let mut kept: HashMap<u64, Arc<[u8]>> = HashMap::new();
        let mut layers = Vec::with_capacity(compositor.layers().len());
        let mut held = HashMap::with_capacity(compositor.layers().len());
        // composite order: by z-order, a tie in the order the compositor
        // holds them, as its next resolve sorts them
        let mut stack: Vec<&Layer> = compositor.layers().iter().collect();
        stack.sort_by_key(|l| l.z_order);
        for layer in stack {
            let key = layer.id.0;
            let hidden: Vec<[f32; 4]> = masks
                .iter()
                .filter(|m| m.layer == key)
                .map(|m| m.rect)
                .collect();
            let nodes: Vec<Drawn> = mask::hide(layer.nodes(), &hidden)
                .iter()
                .map(|n| self.drawn(n, &mut kept, &mut assets))
                .collect();
            let props = Props::of(layer);
            let body: Arc<[u8]> = node::body(key, &props, &nodes).into();
            let hash = fnv(&[&body]);
            let patch = match self.layers.get(&key) {
                Some((base, before)) if *base != hash => {
                    let patch = node::patch(&props, before, &nodes);
                    (patch.len() < body.len()).then(|| (*base, patch.into()))
                }
                _ => None,
            };
            layers.push(LayerOut {
                key,
                hash,
                body,
                patch,
            });
            held.insert(key, (hash, nodes));
        }
        self.layers = held;
        self.assets = kept;
        Frame {
            head: head.write().into(),
            layers,
            assets,
        }
    }

    /// a node as it travels; an image or a path enters the frame's
    /// assets the first time it is drawn.
    fn drawn(
        &self,
        node: &SceneNode,
        kept: &mut HashMap<u64, Arc<[u8]>>,
        assets: &mut Vec<(u64, Arc<[u8]>)>,
    ) -> Drawn {
        let (key, build): (u64, &dyn Fn() -> Arc<[u8]>) = match node {
            SceneNode::Image { image, .. } if engine::gpu::image_is_stream(image) => {
                (asset::image_key(image, true), &|| {
                    asset::image_asset(image, Pixels::Live)
                })
            }
            SceneNode::Image { image, .. } => (asset::image_key(image, false), &|| {
                let pixels = engine::gpu::image_pixels(image);
                let pixels = pixels.as_deref().map_or(Pixels::Missing, Pixels::Here);
                asset::image_asset(image, pixels)
            }),
            SceneNode::Path { data } => (asset::path_key(data), &|| asset::path_asset(data)),
            other => return Drawn::Node(other.clone()),
        };
        if let Entry::Vacant(slot) = kept.entry(key) {
            let bytes = self.assets.get(&key).cloned().unwrap_or_else(build);
            slot.insert(bytes.clone());
            assets.push((key, bytes));
        }
        match node {
            SceneNode::Image {
                x,
                y,
                w,
                h,
                corner_radius,
                ..
            } => Drawn::Image {
                x: *x,
                y: *y,
                w: *w,
                h: *h,
                corner_radius: *corner_radius,
                key,
            },
            _ => Drawn::Path { key },
        }
    }
}
