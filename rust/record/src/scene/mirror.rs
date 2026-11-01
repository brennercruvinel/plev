//! the viewer's side: the layers and the assets a receiver holds, a frame
//! applied to them, and the scene drawn into a compositor of its own.
//!
//! a fixed image enters this process's store by `load_image_rgba`, once
//! per content (the store's dedupe answers the same handle when it comes
//! again). a stream image (a camera) never comes with its pixels: in its
//! place the receiver draws a box that says the live image is not
//! mirrored.

use std::collections::{HashMap, HashSet};

use engine::drawing::compositor::{Compositor, LayerId, SceneNode, TextNodeKey};
use engine::drawing::gpu::{ImageHandle, load_image_rgba};
use engine::drawing::path::TessellatedPath;
use engine::drawing::text::OverflowWrap;

use super::SceneError;
use super::asset::{self, Asset};
use super::bytes::fnv;
use super::node::{self, Drawn, Head, Props};

/// what one layer of a frame brings.
#[derive(Clone, Copy, Debug)]
pub enum Brought<'a> {
    /// nothing: the receiver holds the layer at this hash.
    Held,
    /// the layer whole.
    Body(&'a [u8]),
    /// the layer against the version the receiver holds.
    Patch(&'a [u8]),
}

/// one layer of a frame, in composite order.
#[derive(Clone, Copy, Debug)]
pub struct Incoming<'a> {
    pub key: u32,
    pub hash: u64,
    pub brought: Brought<'a>,
}

/// what the receiver did with the images it was sent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageCounts {
    /// fixed images loaded (`load_image_rgba`), one per asset received.
    pub loads: u64,
    /// stream images received, as their size and place: each draws the
    /// box that says it is not mirrored.
    pub live: u64,
    /// fixed images that came without pixels, or that this store
    /// refused: their nodes draw nothing.
    pub missing: u64,
}

#[derive(Debug)]
struct Held {
    key: u32,
    hash: u64,
    props: Props,
    nodes: Vec<Drawn>,
}

/// an image as the receiver holds it.
#[derive(Clone, Copy, Debug)]
enum Shown {
    /// a fixed image, in this process's store.
    Here(ImageHandle),
    /// a stream image, never mirrored.
    Live,
}

/// the scene a receiver holds.
#[derive(Debug, Default)]
pub struct Mirror {
    head: Option<Head>,
    layers: Vec<Held>,
    images: HashMap<u64, Shown>,
    paths: HashMap<u64, TessellatedPath>,
    /// the app's layer ids and the compositor layers they draw into.
    ids: HashMap<u32, LayerId>,
    counts: ImageCounts,
}

impl Mirror {
    pub fn new() -> Mirror {
        Mirror::default()
    }

    /// the head of the last frame: the size, the scale, the clear.
    pub fn head(&self) -> Option<Head> {
        self.head
    }

    /// the layers held, in composite order: key, hash, node count.
    pub fn layers(&self) -> Vec<(u32, u64, usize)> {
        self.layers
            .iter()
            .map(|l| (l.key, l.hash, l.nodes.len()))
            .collect()
    }

    pub fn image_counts(&self) -> ImageCounts {
        self.counts
    }

    /// assets held: images and paths.
    pub fn assets(&self) -> (usize, usize) {
        (self.images.len(), self.paths.len())
    }

    /// an asset that came before the frame that draws it.
    pub fn asset(&mut self, key: u64, bytes: &[u8]) -> Result<(), SceneError> {
        match asset::read_asset(bytes)? {
            Asset::Path(path) => {
                self.paths.insert(key, path);
            }
            Asset::Image { live: true, .. } => {
                self.counts.live += 1;
                self.images.insert(key, Shown::Live);
            }
            Asset::Image {
                handle,
                pixels: Some(pixels),
                ..
            } => {
                self.counts.loads += 1;
                match load_image_rgba(handle.width, handle.height, pixels) {
                    Ok(here) => {
                        self.images.insert(key, Shown::Here(here));
                    }
                    Err(_) => self.counts.missing += 1,
                }
            }
            Asset::Image { pixels: None, .. } => self.counts.missing += 1,
        }
        Ok(())
    }

    /// applies one frame: `head` when it changed, every layer in
    /// composite order. a body is checked against its hash, a patch by
    /// the hash of what it makes; a layer held is taken as it is. then
    /// the assets no layer references any more are let go, as the sender
    /// forgets them.
    pub fn frame(
        &mut self,
        head: Option<&[u8]>,
        layers: &[Incoming<'_>],
    ) -> Result<(), SceneError> {
        let head = head.map(Head::read).transpose()?;
        let mut before: HashMap<u32, Held> = self.layers.drain(..).map(|l| (l.key, l)).collect();
        let mut now = Vec::with_capacity(layers.len());
        for l in layers {
            let held = match l.brought {
                Brought::Held => before
                    .remove(&l.key)
                    .filter(|h| h.hash == l.hash)
                    .ok_or(SceneError::NotHeld(l.key))?,
                Brought::Body(bytes) => {
                    if fnv(&[bytes]) != l.hash {
                        return Err(SceneError::Hash(l.key));
                    }
                    let (key, props, nodes) = node::read_body(bytes)?;
                    if key != l.key {
                        return Err(SceneError::Invalid("a body under another layer's key"));
                    }
                    Held {
                        key,
                        hash: l.hash,
                        props,
                        nodes,
                    }
                }
                Brought::Patch(bytes) => {
                    let base = before.remove(&l.key).ok_or(SceneError::NotHeld(l.key))?;
                    let (props, nodes) = node::apply_patch(bytes, &base.nodes)?;
                    if fnv(&[&node::body(l.key, &props, &nodes)]) != l.hash {
                        return Err(SceneError::Hash(l.key));
                    }
                    Held {
                        key: l.key,
                        hash: l.hash,
                        props,
                        nodes,
                    }
                }
            };
            now.push(held);
        }
        self.layers = now;
        if head.is_some() {
            self.head = head;
        }
        let used: HashSet<u64> = self
            .layers
            .iter()
            .flat_map(|l| l.nodes.iter())
            .filter_map(|n| match n {
                Drawn::Image { key, .. } | Drawn::Path { key } => Some(*key),
                Drawn::Node(_) => None,
            })
            .collect();
        self.images.retain(|k, _| used.contains(k));
        self.paths.retain(|k, _| used.contains(k));
        Ok(())
    }

    /// the scene held, drawn into `c` as the app's compositor held it:
    /// one layer per app layer, in the same composite order, with the
    /// same properties and nodes. `c` starts a frame here
    /// (`begin_frame`), so push anything else after. an image or a path
    /// this receiver does not hold draws nothing.
    pub fn draw(&mut self, c: &mut Compositor) {
        self.arrange(c);
        c.begin_frame();
        for held in &self.layers {
            let Some(&id) = self.ids.get(&held.key) else {
                continue;
            };
            for n in &held.nodes {
                let node = match n {
                    Drawn::Node(node) => node.clone(),
                    Drawn::Image {
                        x,
                        y,
                        w,
                        h,
                        corner_radius,
                        key,
                    } => match self.images.get(key) {
                        Some(Shown::Here(image)) => SceneNode::Image {
                            x: *x,
                            y: *y,
                            w: *w,
                            h: *h,
                            image: *image,
                            corner_radius: *corner_radius,
                        },
                        Some(Shown::Live) => {
                            for node in live_box([*x, *y, *w, *h], *corner_radius) {
                                c.push_to_layer(id, node);
                            }
                            continue;
                        }
                        None => continue,
                    },
                    Drawn::Path { key } => match self.paths.get(key) {
                        Some(path) => SceneNode::Path { data: path.clone() },
                        None => continue,
                    },
                };
                c.push_to_layer(id, node);
            }
        }
    }

    /// forgets which compositor layers draw which app layers: the next
    /// `draw` makes them anew. for a receiver that draws into another
    /// compositor from now on (a viewer that remade its window).
    pub fn forget_layers(&mut self) {
        self.ids.clear();
    }

    /// the compositor's layers made to match the ones held: made, set,
    /// removed, and remade in order when the compositor would stack them
    /// otherwise (layers of one z-order stack in the order they were
    /// made).
    fn arrange(&mut self, c: &mut Compositor) {
        if self.ids.values().any(|id| c.layer(*id).is_none()) {
            // another compositor than the one drawn into before
            self.ids.clear();
        }
        let wanted: Vec<u32> = self.layers.iter().map(|l| l.key).collect();
        let gone: Vec<u32> = self
            .ids
            .keys()
            .copied()
            .filter(|k| !wanted.contains(k))
            .collect();
        for key in gone {
            if let Some(id) = self.ids.remove(&key) {
                drop_layer(c, id);
            }
        }
        for held in &self.layers {
            if !self.ids.contains_key(&held.key) {
                let taken = self.ids.values().any(|id| *id == LayerId::DEFAULT);
                let id = if held.key == LayerId::DEFAULT.0 && !taken {
                    LayerId::DEFAULT
                } else {
                    c.create_layer(held.props.z)
                };
                self.ids.insert(held.key, id);
            }
        }
        self.set(c);
        if self.stacked(c) != wanted {
            for id in self.ids.values() {
                drop_layer(c, *id);
            }
            self.ids.clear();
            for held in &self.layers {
                let id = c.create_layer(held.props.z);
                self.ids.insert(held.key, id);
            }
            self.set(c);
        }
    }

    fn set(&self, c: &mut Compositor) {
        for held in &self.layers {
            let Some(&id) = self.ids.get(&held.key) else {
                continue;
            };
            let p = &held.props;
            c.set_layer_z_order(id, p.z);
            c.set_layer_opacity(id, p.opacity);
            c.set_layer_visible(id, p.visible);
            c.set_layer_effects(id, p.effects.clone());
            c.set_layer_clip_rect(id, p.clip);
        }
    }

    /// the app keys in the order `c` will composite their layers: its
    /// layers as they stand, stably sorted by z-order.
    fn stacked(&self, c: &Compositor) -> Vec<u32> {
        let key_of: HashMap<LayerId, u32> = self.ids.iter().map(|(k, id)| (*id, *k)).collect();
        let mut order: Vec<(i32, u32)> = c
            .layers()
            .iter()
            .filter_map(|l| key_of.get(&l.id).map(|k| (l.z_order, *k)))
            .collect();
        order.sort_by_key(|(z, _)| *z);
        order.into_iter().map(|(_, k)| k).collect()
    }
}

/// a layer let go: removed, or, for the default one, which a compositor
/// always keeps, left empty and plain.
fn drop_layer(c: &mut Compositor, id: LayerId) {
    if id == LayerId::DEFAULT {
        c.set_layer_z_order(id, 0);
        c.set_layer_opacity(id, 1.0);
        c.set_layer_visible(id, true);
        c.set_layer_effects(id, Vec::new());
        c.set_layer_clip_rect(id, None);
    } else {
        c.remove_layer(id);
    }
}

/// what a receiver says of a live image in its place.
pub const LIVE_LABEL: &str = "live image, not mirrored";

/// the box drawn where a stream image (a camera) is: a dark rounded rect
/// with its edge, and [`LIVE_LABEL`] inside it.
fn live_box([x, y, w, h]: [f32; 4], corner_radius: f32) -> [SceneNode; 4] {
    [
        SceneNode::RoundedRect {
            x,
            y,
            w,
            h,
            color: [0.06, 0.06, 0.07, 1.0],
            corner_radius,
            border_width: 1.0,
            border_color: [0.42, 0.42, 0.46, 1.0],
        },
        SceneNode::PushClip { x, y, w, h },
        SceneNode::Text {
            key: TextNodeKey {
                text: LIVE_LABEL.to_owned(),
                font_size_bits: 13.0f32.to_bits(),
                line_height_bits: 18.0f32.to_bits(),
                max_width_bits: Some((w - 24.0).max(1.0).to_bits()),
                font_weight: 500,
                letter_spacing_bits: 0.0f32.to_bits(),
                font_family: None,
                overflow_wrap: OverflowWrap::Normal,
            },
            x: x + 12.0,
            y: y + 12.0,
            color: [0.78, 0.78, 0.8, 1.0],
        },
        SceneNode::PopClip,
    ]
}
