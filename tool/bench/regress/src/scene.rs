//! A staged frame in the terms a baseline keeps: every layer under a
//! stable name, with a stable content hash and the bounds of what it
//! draws, and every bound control with its rect.
//!
//! The name. A layer has no name in the compositor and its `LayerId` is
//! the order of creation. The name is taken from what the layer holds
//! that does not move with content: the roots of the `SemId`s bound in it
//! (`todo` for a layer holding `todo`, `todo.input`, `todo.item[3]`), in
//! bind order, joined by `+`. A layer with no binding is named by the
//! z-order the app gave it (`z0`, `z1000`). A name that repeats gets
//! `#2`, `#3` in composite order (z-order, then creation). Adding an item
//! under `todo`, or a layer at another z, renames nothing.
//!
//! The content hash of a layer is [`crate::hash::stable_hash`].

use std::collections::HashMap;

use engine::drawing::compositor::{Compositor, Layer, LayerId, SceneNode, shadow_padding};
use engine::drawing::headless::Viewport;
use engine::drawing::text::{TextMeasurer, TextStyle};
use serde::{Deserialize, Serialize};

use crate::hash::stable_hash;

/// One layer of a frame, as a baseline keeps it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LayerRecord {
    pub name: String,
    pub z: i32,
    /// [`stable_hash`], as 16 hex digits (a JSON number loses a u64).
    #[serde(with = "hex")]
    pub hash: u64,
    pub nodes: usize,
    /// `[x, y, w, h]` in logical px: what the layer draws, within its
    /// clips and the viewport; `None` when it draws nothing there.
    pub bounds: Option<[f32; 4]>,
}

/// One bound control of a frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BindRecord {
    pub id: String,
    /// The name of the layer it is bound in.
    pub layer: String,
    /// `[x, y, w, h]` in logical px, the rect it draws and hit-tests with.
    pub rect: [f32; 4],
    pub clip: Option<[f32; 4]>,
}

/// A frame's layers in composite order and its bindings in bind order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub layers: Vec<LayerRecord>,
    pub bindings: Vec<BindRecord>,
}

/// The scene of `c`. Bounds are measured only `with_bounds` (text is
/// measured to get them); a check measures only the layers that changed
/// ([`bounds_of`]).
pub fn capture(c: &Compositor, viewport: Viewport, with_bounds: bool) -> Scene {
    let named = names(c);
    let layers = named
        .iter()
        .filter_map(|(id, name)| {
            let layer = c.layer(*id)?;
            Some(LayerRecord {
                name: name.clone(),
                z: layer.z_order,
                hash: stable_hash(layer),
                nodes: layer.nodes().len(),
                bounds: if with_bounds {
                    bounds(layer, viewport)
                } else {
                    None
                },
            })
        })
        .collect();
    let bindings = c
        .bindings()
        .filter_map(|(id, b)| {
            let layer = named.iter().find(|(l, _)| *l == b.layer)?;
            Some(BindRecord {
                id: id.to_string(),
                layer: layer.1.clone(),
                rect: b.rect,
                clip: b.clip,
            })
        })
        .collect();
    Scene { layers, bindings }
}

/// The bounds of the layer named `name` in `c`, as [`capture`] measures
/// them.
pub fn bounds_of(c: &Compositor, viewport: Viewport, name: &str) -> Option<[f32; 4]> {
    let (id, _) = names(c).into_iter().find(|(_, n)| n == name)?;
    bounds(c.layer(id)?, viewport)
}

/// Every layer of `c` with its name, in composite order: z-order, then
/// creation (the compositor sorts by z-order, stably, before it
/// composites).
pub fn names(c: &Compositor) -> Vec<(LayerId, String)> {
    let mut order: Vec<&Layer> = c.layers().iter().collect();
    order.sort_by_key(|l| l.z_order);
    let mut roots: HashMap<LayerId, Vec<&str>> = HashMap::new();
    for (id, binding) in c.bindings() {
        let at = roots.entry(binding.layer).or_default();
        let root = root(id.as_str());
        if !at.contains(&root) {
            at.push(root);
        }
    }
    let mut seen: HashMap<String, usize> = HashMap::new();
    order
        .into_iter()
        .map(|layer| {
            let base = match roots.get(&layer.id) {
                Some(r) => r.join("+"),
                None => format!("z{}", layer.z_order),
            };
            let n = seen.entry(base.clone()).or_insert(0);
            *n += 1;
            let name = if *n == 1 { base } else { format!("{base}#{n}") };
            (layer.id, name)
        })
        .collect()
}

/// The first segment of a `SemId`'s canonical text: a name segment is
/// `[a-z0-9_]+`, so it ends at the first `.` or `[`.
fn root(id: &str) -> &str {
    id.split(['.', '[']).next().unwrap_or(id)
}

/// What the layer draws, in logical px: the union of every node's
/// extent, each cut by the clips open at it, then by the layer's own
/// scissor and the viewport. Text is measured with the style its key
/// carries (the shaper the frame draws from) and padded by a quarter of
/// its size for ink past the line box; a drop shadow covers its blur
/// and offset. A superset is safe: it only bounds where pixels are
/// compared.
pub fn bounds(layer: &Layer, viewport: Viewport) -> Option<[f32; 4]> {
    let mut clips: Vec<[f32; 4]> = Vec::new();
    let mut all: Option<[f32; 4]> = None;
    for node in layer.nodes() {
        let extent = match node {
            SceneNode::PushClip { x, y, w, h } => {
                let rect = [*x, *y, *w, *h];
                let open = clips.last().map_or(Some(rect), |c| intersect(*c, rect));
                clips.push(open.unwrap_or([*x, *y, 0.0, 0.0]));
                continue;
            }
            SceneNode::PopClip => {
                clips.pop();
                continue;
            }
            SceneNode::Rect { x, y, w, h, .. }
            | SceneNode::RoundedRect { x, y, w, h, .. }
            | SceneNode::GradientRect { x, y, w, h, .. }
            | SceneNode::Image { x, y, w, h, .. }
            | SceneNode::BackdropBlur { x, y, w, h, .. } => [*x, *y, *w, *h],
            SceneNode::Shadow {
                x,
                y,
                w,
                h,
                blur_radius,
                offset,
                inset,
                ..
            } => {
                if *inset {
                    [*x, *y, *w, *h]
                } else {
                    let pad = shadow_padding(*blur_radius);
                    union(
                        [*x, *y, *w, *h],
                        [
                            x + offset[0] - pad,
                            y + offset[1] - pad,
                            w + 2.0 * pad,
                            h + 2.0 * pad,
                        ],
                    )
                }
            }
            SceneNode::Text { key, x, y, .. } => {
                let style = TextStyle {
                    font_size: f32::from_bits(key.font_size_bits),
                    line_height: f32::from_bits(key.line_height_bits),
                    font_weight: key.font_weight,
                    letter_spacing: f32::from_bits(key.letter_spacing_bits),
                    font_family: key.font_family.clone(),
                    features: key.features,
                    overflow_wrap: key.overflow_wrap,
                };
                let max = key.max_width_bits.map(f32::from_bits);
                let (w, h) = TextMeasurer::measure_styled(&key.text, &style, max);
                let pad = style.font_size * 0.25;
                [x - pad, y - pad, w + 2.0 * pad, h + 2.0 * pad]
            }
            SceneNode::Path { data } => {
                let mut it = data.vertices.iter().map(|v| v.position);
                let Some([x0, y0]) = it.next() else {
                    continue;
                };
                let (mut lo, mut hi) = ([x0, y0], [x0, y0]);
                for [x, y] in it {
                    lo = [lo[0].min(x), lo[1].min(y)];
                    hi = [hi[0].max(x), hi[1].max(y)];
                }
                [lo[0], lo[1], hi[0] - lo[0], hi[1] - lo[1]]
            }
        };
        let drawn = match clips.last() {
            Some(clip) => intersect(*clip, extent),
            None => Some(extent),
        };
        if let Some(r) = drawn.filter(|r| r[2] > 0.0 && r[3] > 0.0) {
            all = Some(all.map_or(r, |a| union(a, r)));
        }
    }
    let mut out = all?;
    if let Some((x, y, w, h)) = layer.clip_rect {
        let s = viewport.scale;
        out = intersect(
            out,
            [x as f32 / s, y as f32 / s, w as f32 / s, h as f32 / s],
        )?;
    }
    intersect(out, [0.0, 0.0, viewport.width, viewport.height])
}

/// The smallest rect holding both.
pub fn union(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let x0 = a[0].min(b[0]);
    let y0 = a[1].min(b[1]);
    let x1 = (a[0] + a[2]).max(b[0] + b[2]);
    let y1 = (a[1] + a[3]).max(b[1] + b[3]);
    [x0, y0, x1 - x0, y1 - y0]
}

/// The overlap, `None` when it is empty.
pub fn intersect(a: [f32; 4], b: [f32; 4]) -> Option<[f32; 4]> {
    let x0 = a[0].max(b[0]);
    let y0 = a[1].max(b[1]);
    let x1 = (a[0] + a[2]).min(b[0] + b[2]);
    let y1 = (a[1] + a[3]).min(b[1] + b[3]);
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
}

/// A `u64` as 16 lowercase hex digits, and back.
mod hex {
    use serde::{Deserialize, Deserializer, Serializer, de::Error};

    pub fn serialize<S: Serializer>(v: &u64, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{v:016x}"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
        let text = String::deserialize(d)?;
        u64::from_str_radix(&text, 16).map_err(D::Error::custom)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use accord::SemId;

    pub(crate) const VIEW: Viewport = Viewport {
        width: 400.0,
        height: 300.0,
        scale: 1.0,
    };

    fn id(text: &str) -> SemId {
        match SemId::parse(text) {
            Ok(id) => id,
            Err(e) => panic!("{text}: {e}"),
        }
    }

    pub(crate) fn rect(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> SceneNode {
        SceneNode::Rect { x, y, w, h, color }
    }

    pub(crate) const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    pub(crate) const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

    /// A small page: a panel with two rows bound under `list`, a
    /// sidebar layer with nothing bound, a toast layer.
    pub(crate) fn page(rows: &[&str]) -> Compositor {
        let mut c = Compositor::new();
        let sidebar = c.create_layer(1000);
        let toast = c.create_layer(1200);
        c.begin_frame();
        c.push(rect(100.0, 20.0, 280.0, 260.0, BLUE));
        c.bind(&id("list"), [100.0, 20.0, 280.0, 260.0]);
        for (i, key) in rows.iter().enumerate() {
            let y = 30.0 + i as f32 * 30.0;
            c.push(rect(110.0, y, 260.0, 24.0, RED));
            c.bind(&id("list").key(*key), [110.0, y, 260.0, 24.0]);
        }
        c.push_to_layer(sidebar, rect(0.0, 0.0, 90.0, 300.0, BLUE));
        c.push_to_layer(toast, rect(300.0, 260.0, 90.0, 30.0, RED));
        c
    }

    fn named(c: &Compositor) -> Vec<String> {
        names(c).into_iter().map(|(_, n)| n).collect()
    }

    #[test]
    fn a_layer_is_named_by_the_roots_bound_in_it_else_by_its_z_order() {
        let c = page(&["a", "b"]);
        assert_eq!(named(&c), ["list", "z1000", "z1200"]);
    }

    #[test]
    fn content_under_a_root_and_a_new_layer_elsewhere_rename_nothing() {
        let before = named(&page(&["a"]));
        let mut c = page(&["a", "b", "c"]);
        assert_eq!(named(&c), before);
        // a layer created later, below every other: the others keep
        // their names, the new one takes its z
        let under = c.create_layer(-5);
        c.push_to_layer(under, rect(0.0, 0.0, 10.0, 10.0, RED));
        assert_eq!(named(&c), ["z-5", "list", "z1000", "z1200"]);
    }

    #[test]
    fn two_roots_join_in_bind_order_and_a_repeated_name_is_numbered() {
        let mut c = Compositor::new();
        let a = c.create_layer(5);
        let b = c.create_layer(5);
        let d = c.create_layer(9);
        c.begin_frame();
        c.bind(&id("nav.home"), [0.0; 4]);
        c.bind(&id("form.name"), [0.0; 4]);
        c.bind(&id("nav.back"), [0.0; 4]);
        c.push_to_layer(a, rect(0.0, 0.0, 1.0, 1.0, RED));
        c.push_to_layer(b, rect(0.0, 0.0, 1.0, 1.0, RED));
        c.bind_to_layer(d, &id("form.ok"), [0.0; 4]);
        assert_eq!(named(&c), ["nav+form", "z5", "z5#2", "form"]);
    }

    #[test]
    fn bounds_are_the_drawn_extent_within_clips_and_the_viewport() {
        let c = page(&["a"]);
        let scene = capture(&c, VIEW, true);
        assert_eq!(scene.layers[0].bounds, Some([100.0, 20.0, 280.0, 260.0]));
        let mut clipped = Compositor::new();
        clipped.push(SceneNode::PushClip {
            x: 0.0,
            y: 0.0,
            w: 50.0,
            h: 50.0,
        });
        clipped.push(rect(20.0, 20.0, 100.0, 100.0, RED));
        clipped.push(SceneNode::PopClip);
        clipped.push(rect(390.0, 290.0, 50.0, 50.0, RED));
        let Some(layer) = clipped.layer(LayerId::DEFAULT) else {
            panic!("no default layer");
        };
        // the first rect is cut by its clip, the second by the viewport
        assert_eq!(bounds(layer, VIEW), Some([20.0, 20.0, 380.0, 280.0]));
        assert_eq!(
            capture(&Compositor::new(), VIEW, true).layers[0].bounds,
            None
        );
    }

    #[test]
    fn text_bounds_come_from_the_shaper_the_frame_draws_with() {
        let mut c = Compositor::new();
        let style = TextStyle::new(20.0);
        c.push(SceneNode::Text {
            key: engine::drawing::compositor::TextNodeKey::from_style("Walk the dog", &style, None),
            x: 40.0,
            y: 60.0,
            color: RED,
        });
        let (w, h) = TextMeasurer::measure_styled("Walk the dog", &style, None);
        let Some(layer) = c.layer(LayerId::DEFAULT) else {
            panic!("no default layer");
        };
        assert_eq!(
            bounds(layer, VIEW),
            Some([35.0, 55.0, w + 10.0, h + 10.0]),
            "{w}x{h}"
        );
        assert!(w > 60.0 && h >= 20.0, "{w}x{h}");
    }

    #[test]
    fn bindings_carry_the_name_of_their_layer() {
        let scene = capture(&page(&["a", "b"]), VIEW, false);
        let ids: Vec<(&str, &str)> = scene
            .bindings
            .iter()
            .map(|b| (b.id.as_str(), b.layer.as_str()))
            .collect();
        assert_eq!(
            ids,
            [("list", "list"), ("list[a]", "list"), ("list[b]", "list")]
        );
        assert_eq!(scene.bindings[2].rect, [110.0, 60.0, 260.0, 24.0]);
    }

    #[test]
    fn a_record_survives_json() {
        let scene = capture(&page(&["a"]), VIEW, true);
        let text = match serde_json::to_string(&scene) {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        };
        assert!(text.contains(&format!("\"{:016x}\"", scene.layers[0].hash)));
        match serde_json::from_str::<Scene>(&text) {
            Ok(back) => assert_eq!(back, scene),
            Err(e) => panic!("{e}\n{text}"),
        }
    }
}
