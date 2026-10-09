//! task-79, the probe behind the scene channel: what a frame of every
//! showcase page weighs when it travels, by the monster path the
//! recorder uses (`monster::raise_scene`, discovery, the v0 encoder) and
//! by a codec of the compositor's own nodes, layer by layer; what a frame
//! with one change weighs on a page with text, one with an image and one
//! with glass; the images and the bindings a frame references.
//!
//! a measurement, not a check: it prints and asserts nothing about the
//! numbers. debug build (the workspace crates at opt-level 0, their
//! dependencies at 2), so the times are an upper bound:
//!
//!     cargo test -p rec --test scene_probe -- --ignored --nocapture
//!
//! the per-layer codec here is the draft the channel's codec grew from:
//! every node a tag byte and its fields as they are (an `f32` as its
//! bits), a text as its key (the string and the typography) and its
//! place, an image as its placement (its pixels counted apart: they would
//! travel once per content), a path as its vertices and indices.

use std::collections::{BTreeMap, HashSet};
use std::time::{Duration, Instant};

use engine::widgets::prelude::WidgetEvent;
use engine::drawing::compositor::{Compositor, Layer, LayerEffect, SceneNode, TextNodeKey};
use engine::drawing::headless::Viewport;
use motion::{AssetBank, DiscoverConfig, LoweredAsset, Node};
use uigallery::shell::stage_page;
use uigallery::views::{PAGES, ShowcaseView};

const VIEWPORT: Viewport = Viewport {
    width: 1280.0,
    height: 800.0,
    scale: 1.0,
};

/// the longest line a client may send the port (`wire::MAX_LINE`,
/// crates/wire/src/port.rs:48); rec does not depend on wire.
const MAX_LINE: usize = 1 << 20;

/// base64 of `n` bytes, as a JSON string carries binary.
fn b64(n: usize) -> usize {
    n.div_ceil(3) * 4
}

struct Buf(Vec<u8>);

impl Buf {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32s(&mut self, vs: &[f32]) {
        for v in vs {
            self.u32(v.to_bits());
        }
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
}

fn text_key(b: &mut Buf, k: &TextNodeKey) {
    b.str(&k.text);
    b.u32(k.font_size_bits);
    b.u32(k.line_height_bits);
    match k.max_width_bits {
        Some(w) => {
            b.u8(1);
            b.u32(w);
        }
        None => b.u8(0),
    }
    b.u16(k.font_weight);
    b.u32(k.letter_spacing_bits);
    match &k.font_family {
        Some(f) => {
            b.u8(1);
            b.str(f);
        }
        None => b.u8(0),
    }
}

/// one layer as the draft codec writes it: its properties, then its nodes.
fn encode_layer(layer: &Layer) -> Vec<u8> {
    let mut b = Buf(Vec::new());
    b.u32(layer.id.0);
    b.u32(layer.z_order as u32);
    b.u8(u8::from(layer.visible));
    b.f32s(&[layer.opacity]);
    match layer.clip_rect {
        Some((x, y, w, h)) => {
            b.u8(1);
            for v in [x, y, w, h] {
                b.u32(v);
            }
        }
        None => b.u8(0),
    }
    b.u32(layer.effects().len() as u32);
    for e in layer.effects() {
        match e {
            LayerEffect::Blur { sigma } => {
                b.u8(1);
                b.f32s(&[*sigma]);
            }
            LayerEffect::Shadow { sigma, color } => {
                b.u8(2);
                b.f32s(&[*sigma]);
                b.f32s(color);
            }
        }
    }
    b.u32(layer.nodes().len() as u32);
    for node in layer.nodes() {
        encode_node(&mut b, node);
    }
    b.0
}

/// one node as the draft codec writes it.
fn encode_node(b: &mut Buf, node: &SceneNode) {
    {
        match node {
            SceneNode::Rect { x, y, w, h, color } => {
                b.u8(1);
                b.f32s(&[*x, *y, *w, *h]);
                b.f32s(color);
            }
            SceneNode::RoundedRect {
                x,
                y,
                w,
                h,
                color,
                corner_radius,
                border_width,
                border_color,
            } => {
                b.u8(2);
                b.f32s(&[*x, *y, *w, *h]);
                b.f32s(color);
                b.f32s(&[*corner_radius, *border_width]);
                b.f32s(border_color);
            }
            SceneNode::GradientRect {
                x,
                y,
                w,
                h,
                color,
                color2,
                angle_deg,
                corner_radius,
                border_width,
                border_color,
            } => {
                b.u8(3);
                b.f32s(&[*x, *y, *w, *h]);
                b.f32s(color);
                b.f32s(color2);
                b.f32s(&[*angle_deg, *corner_radius, *border_width]);
                b.f32s(border_color);
            }
            SceneNode::Text { key, x, y, color } => {
                b.u8(4);
                text_key(b, key);
                b.f32s(&[*x, *y]);
                b.f32s(color);
            }
            SceneNode::Path { data } => {
                // the geometry travels once per content, as an asset
                b.u8(5);
                b.0.extend_from_slice(&data.hash.to_le_bytes());
            }
            SceneNode::Image {
                x,
                y,
                w,
                h,
                image,
                corner_radius,
            } => {
                b.u8(6);
                b.f32s(&[*x, *y, *w, *h, *corner_radius]);
                for v in [image.atlas_x, image.atlas_y, image.width, image.height] {
                    b.u32(v);
                }
                // the content key the pixels travel under, once
                b.0.extend_from_slice(&[0; 8]);
            }
            SceneNode::PushClip { x, y, w, h } => {
                b.u8(7);
                b.f32s(&[*x, *y, *w, *h]);
            }
            SceneNode::PopClip => b.u8(8),
            SceneNode::BackdropBlur {
                x,
                y,
                w,
                h,
                corner_radius,
                sigma,
            } => {
                b.u8(9);
                b.f32s(&[*x, *y, *w, *h, *corner_radius, *sigma]);
            }
            SceneNode::Shadow {
                x,
                y,
                w,
                h,
                corner_radius,
                blur_radius,
                offset,
                color,
                inset,
            } => {
                b.u8(10);
                b.f32s(&[*x, *y, *w, *h, *corner_radius, *blur_radius]);
                b.f32s(offset);
                b.f32s(color);
                b.u8(u8::from(*inset));
            }
        }
    }
}

/// what a receiver holding `before` of a layer needs for `after`: the
/// nodes kept at the start and at the end, counted, and the ones between,
/// written out (with the layer's properties). the bytes and the nodes
/// written.
fn patch(before: &[SceneNode], after: &[SceneNode]) -> (usize, usize) {
    let head = before.iter().zip(after).take_while(|(a, b)| a == b).count();
    let rest = before.len().min(after.len()) - head;
    let tail = before
        .iter()
        .rev()
        .zip(after.iter().rev())
        .take(rest)
        .take_while(|(a, b)| a == b)
        .count();
    let mut b = Buf(Vec::new());
    for n in &after[head..after.len() - tail] {
        encode_node(&mut b, n);
    }
    // layer properties (~30 bytes) + head, tail, count
    (30 + 12 + b.0.len(), after.len() - head - tail)
}

/// a frame as the draft channel sends it: the viewport, the clear, each
/// layer's id and content hash, the bodies of the layers the receiver
/// does not hold, and the bindings (id and visible rect).
struct Draft {
    /// per layer: id, hash of its body, its body.
    layers: Vec<(u32, u64, Vec<u8>)>,
    bindings: Vec<u8>,
    /// distinct image placements and the RGBA bytes they would carry.
    images: Vec<(u32, u32, u32, u32)>,
    /// distinct paths by content hash and the bytes of their geometry
    /// (every vertex's position and color, every index, as `f32` and
    /// `u32`): sent once, then named by the hash.
    paths: BTreeMap<u64, usize>,
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn draft(c: &Compositor) -> Draft {
    let layers = c
        .layers()
        .iter()
        .map(|l| {
            let body = encode_layer(l);
            (l.id.0, fnv(&body), body)
        })
        .collect();
    let mut bindings = Buf(Vec::new());
    for (id, rect) in c.visible_bindings() {
        bindings.str(id.as_str());
        bindings.f32s(&rect);
    }
    let mut seen = HashSet::new();
    let mut images = Vec::new();
    let mut paths = BTreeMap::new();
    for l in c.layers() {
        for n in l.nodes() {
            match n {
                SceneNode::Image { image, .. } => {
                    if seen.insert((image.atlas_x, image.atlas_y, image.id)) {
                        images.push((image.atlas_x, image.atlas_y, image.width, image.height));
                    }
                }
                SceneNode::Path { data } => {
                    paths.insert(
                        data.hash,
                        8 + 8 + data.vertices.len() * 24 + data.indices.len() * 4,
                    );
                }
                _ => {}
            }
        }
    }
    Draft {
        layers,
        bindings: bindings.0,
        images,
        paths,
    }
}

/// header: seq u64, viewport 3 f32, clear 4 f32, layer count, per layer
/// id u32 + hash u64 + has-body u8, binding list length.
fn header_bytes(layers: usize) -> usize {
    8 + 12 + 16 + 4 + layers * 13 + 4
}

impl Draft {
    fn pixels(&self) -> usize {
        self.images
            .iter()
            .map(|(_, _, w, h)| (*w as usize) * (*h as usize) * 4)
            .sum()
    }

    /// the whole frame: bodies of every layer and the paths they draw,
    /// without image pixels.
    fn whole(&self) -> usize {
        header_bytes(self.layers.len())
            + self.layers.iter().map(|(_, _, b)| b.len()).sum::<usize>()
            + self.bindings.len()
            + self.path_bytes()
    }

    fn path_bytes(&self) -> usize {
        self.paths.values().sum()
    }

    /// what goes to a receiver that holds `before`: the header, the
    /// bodies whose hash moved, the bindings when they moved.
    fn delta(&self, before: &Draft) -> (usize, usize) {
        let held: BTreeMap<u32, u64> = before.layers.iter().map(|(i, h, _)| (*i, *h)).collect();
        let dirty: Vec<&(u32, u64, Vec<u8>)> = self
            .layers
            .iter()
            .filter(|(i, h, _)| held.get(i) != Some(h))
            .collect();
        let bindings = if self.bindings == before.bindings {
            0
        } else {
            self.bindings.len()
        };
        let paths: usize = self
            .paths
            .iter()
            .filter(|(h, _)| !before.paths.contains_key(h))
            .map(|(_, n)| n)
            .sum();
        let bytes = header_bytes(self.layers.len())
            + dirty.iter().map(|(_, _, b)| b.len()).sum::<usize>()
            + bindings
            + paths;
        (bytes, dirty.len())
    }
}

/// the scene in composite order, every layer's nodes one after the other.
fn nodes(c: &Compositor) -> Vec<SceneNode> {
    c.layers()
        .iter()
        .flat_map(|l| l.nodes().iter().cloned())
        .collect()
}

fn discovery() -> DiscoverConfig {
    DiscoverConfig {
        keyframe_every_s: f32::INFINITY,
        scalar_jump: f32::INFINITY,
        color_jump: f32::INFINITY,
    }
}

/// the monster path for `frames`: raised into one bank, discovered and
/// encoded as the recorder's visual track does (crates/rec/src/visual.rs,
/// `encode`), plus what its asset table would hold (a text key as the
/// session file writes it, a path as `monster::asset_path` packs it; an
/// image as its size, the monster table having no pixels).
struct Monster {
    bytes: usize,
    assets: usize,
    skipped: [usize; 4],
    took: Duration,
}

fn monster(frames: &[Vec<SceneNode>]) -> Monster {
    let at = Instant::now();
    let mut bank = AssetBank::new();
    let mut skipped = [0usize; 4];
    let mut samples: Vec<(f32, Vec<Node>)> = Vec::new();
    for (n, frame) in frames.iter().enumerate() {
        let raised = motion::raise_scene(frame, &mut bank).expect("the scene raises");
        for s in &raised.skipped {
            if let Some(k) = motion::Unsupported::ALL.iter().position(|u| *u == s.kind) {
                skipped[k] += 1;
            }
        }
        samples.push((n as f32 / 60.0, raised.nodes));
    }
    let bytes = motion::discover(&samples, &discovery())
        .ok()
        .and_then(|t| motion::encode(&t, &[], &[]).ok())
        .map_or(0, |b| b.len());
    let took = at.elapsed();
    let assets = bank
        .assets()
        .iter()
        .map(|a| match a {
            LoweredAsset::TextStyle(k) => {
                5 + 4
                    + k.text.len()
                    + 4
                    + 4
                    + 5
                    + 2
                    + 4
                    + 3
                    + k.font_family.as_ref().map_or(0, String::len)
            }
            LoweredAsset::Image(_) => 5 + 9,
            LoweredAsset::Path(p) => {
                5 + 4
                    + motion::encoding::asset_path::pack_chunks(p)
                        .iter()
                        .map(|c| 4 + c.len())
                        .sum::<usize>()
            }
        })
        .sum();
    Monster {
        bytes,
        assets,
        skipped: if frames.len() > 1 {
            skipped.map(|s| s / frames.len())
        } else {
            skipped
        },
        took,
    }
}

fn kinds(c: &Compositor) -> [usize; 5] {
    let mut k = [0usize; 5];
    for l in c.layers() {
        for n in l.nodes() {
            match n {
                SceneNode::Text { .. } => k[0] += 1,
                SceneNode::Image { .. } => k[1] += 1,
                SceneNode::BackdropBlur { .. } => k[2] += 1,
                SceneNode::Shadow { .. } => k[3] += 1,
                SceneNode::PushClip { .. } | SceneNode::PopClip => k[4] += 1,
                _ => {}
            }
        }
    }
    k
}

/// the time of `f` over `n` runs, the fastest (debug build).
fn best(n: usize, mut f: impl FnMut()) -> Duration {
    (0..n)
        .map(|_| {
            let at = Instant::now();
            f();
            at.elapsed()
        })
        .min()
        .unwrap_or_default()
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

#[test]
#[ignore = "a measurement: run with --ignored --nocapture"]
fn every_page_weighed_by_both_codecs() {
    println!("\nwhole frame per page at 1280x800 @1 (bytes; b64 = as a JSON string)");
    println!(
        "{:<22} {:<5} {:>3} {:>5} {:>4} {:>3} {:>4} | {:>8} {:>6} {:>8} {:>6} {:>7} | {:>8} {:>7} {:>11} {:>7} | {:>7}",
        "page",
        "theme",
        "lay",
        "nodes",
        "text",
        "img",
        "blur",
        "own",
        "paths",
        "own b64",
        "bind",
        "own ms",
        "monster",
        "assets",
        "skip c/b/s",
        "mon ms",
        "pixels"
    );
    let mut max = (0usize, String::new());
    let mut total = [0usize; 3];
    let mut pages = 0usize;
    for theme in ["core", "hoff"] {
        for p in PAGES {
            let staged = stage_page(VIEWPORT, p.title, theme, None).expect("the page stages");
            let c = staged.compositor();
            let d = draft(c);
            let k = kinds(c);
            let own_t = best(3, || {
                let _ = draft(c);
            });
            let frame = nodes(c);
            let m = monster(std::slice::from_ref(&frame));
            let n: usize = c.layers().iter().map(|l| l.nodes().len()).sum();
            let whole = d.whole();
            println!(
                "{:<22} {:<5} {:>3} {:>5} {:>4} {:>3} {:>4} | {:>8} {:>6} {:>8} {:>6} {:>7.2} | {:>8} {:>7} {:>4}/{:>2}/{:>3} {:>7.2} | {:>7}",
                p.title,
                theme,
                c.layers().len(),
                n,
                k[0],
                k[1],
                k[2],
                whole,
                d.path_bytes(),
                b64(whole),
                d.bindings.len(),
                ms(own_t),
                m.bytes,
                m.assets,
                m.skipped[0] + m.skipped[1],
                m.skipped[2],
                m.skipped[3],
                ms(m.took),
                d.pixels()
            );
            if b64(whole) > max.0 {
                max = (b64(whole), format!("{} / {theme}", p.title));
            }
            total[0] += whole;
            total[1] += m.bytes + m.assets;
            total[2] += d.pixels();
            pages += 1;
        }
    }
    println!(
        "\n{pages} frames: own {} bytes in all (mean {}), monster with assets {} (mean {}), image pixels {}",
        total[0],
        total[0] / pages,
        total[1],
        total[1] / pages,
        total[2]
    );
    println!(
        "largest whole frame as a JSON string: {} bytes ({}), MAX_LINE {MAX_LINE} ({:.1}%)",
        max.0,
        max.1,
        max.0 as f64 * 100.0 / MAX_LINE as f64
    );
    let camera = 640 * 360 * 4;
    println!(
        "a camera frame (task-76 item 10, 640x360 RGBA at 15 fps): {camera} bytes, {} as base64 ({:.2} x MAX_LINE), {:.1} MB/s raw",
        b64(camera),
        b64(camera) as f64 / MAX_LINE as f64,
        camera as f64 * 15.0 / 1e6
    );
}

/// a live gallery page: the view, its compositor, settled.
struct Live {
    view: ShowcaseView,
    c: Compositor,
}

impl Live {
    fn new(page: &str, theme: &str) -> Live {
        let mut view = ShowcaseView::new(VIEWPORT.width, VIEWPORT.height);
        view.jump_to_page(page);
        view.apply_theme(theme);
        view.resize(VIEWPORT.width, VIEWPORT.height, VIEWPORT.scale);
        let mut live = Live {
            view,
            c: Compositor::new(),
        };
        live.settle();
        live
    }

    fn settle(&mut self) {
        let mut dt = 0.0;
        for _ in 0..120 {
            let moving = self.view.tick(dt);
            self.view.render(&mut self.c);
            dt = 1.0 / 60.0;
            if !moving {
                break;
            }
        }
    }

    /// the pointer over the center of each bound control in turn, then
    /// over a 40 px grid of the page column, until a hover changes the
    /// frame; where it rests.
    fn hover_one(&mut self, before: &Draft) -> Option<String> {
        let mut targets: Vec<(String, f32, f32)> = self
            .c
            .visible_bindings()
            .map(|(id, [x, y, w, h])| (id.as_str().to_owned(), x + w / 2.0, y + h / 2.0))
            .collect();
        targets.reverse();
        for y in (100..780).step_by(40) {
            for x in (300..1260).step_by(40) {
                targets.push((format!("({x}, {y})"), x as f32, y as f32));
            }
        }
        for (at, x, y) in targets {
            self.view.handle_event(&WidgetEvent::MouseMove { x, y });
            self.settle();
            if draft(&self.c).delta(before).1 > 0 {
                return Some(at);
            }
        }
        // no hover shows: the page scrolled by 40 px instead
        self.view.handle_event(&WidgetEvent::Scroll {
            x: 800.0,
            y: 400.0,
            delta: 40.0,
        });
        self.settle();
        (draft(&self.c).delta(before).1 > 0).then(|| "a 40 px scroll".to_owned())
    }
}

#[test]
#[ignore = "a measurement: run with --ignored --nocapture"]
fn one_change_weighed_by_both_codecs() {
    println!("\none change (the pointer over one control) on a page with text, an image, glass");
    for (page, theme) in [("Todo", "core"), ("Card", "hoff"), ("Effects", "hoff")] {
        let mut live = Live::new(page, theme);
        let a = draft(&live.c);
        let frame_a = nodes(&live.c);
        let held: BTreeMap<u32, Vec<SceneNode>> = live
            .c
            .layers()
            .iter()
            .map(|l| (l.id.0, l.nodes().to_vec()))
            .collect();
        let Some(control) = live.hover_one(&a) else {
            println!("{page} / {theme}: nothing changes the frame");
            continue;
        };
        let b = draft(&live.c);
        let frame_b = nodes(&live.c);
        let (delta, dirty) = b.delta(&a);
        let moved: Vec<(usize, usize)> = live
            .c
            .layers()
            .iter()
            .filter_map(|l| {
                let before = held.get(&l.id.0)?;
                (before.as_slice() != l.nodes()).then(|| patch(before, l.nodes()))
            })
            .collect();
        let patched = header_bytes(b.layers.len())
            + moved.iter().map(|(bytes, _)| bytes).sum::<usize>()
            + if b.bindings == a.bindings {
                0
            } else {
                b.bindings.len()
            };
        let written: usize = moved.iter().map(|(_, n)| n).sum();
        let (same, _) = b.delta(&b);
        let one = monster(std::slice::from_ref(&frame_a));
        let two = monster(&[frame_a.clone(), frame_b.clone()]);
        let k = kinds(&live.c);
        println!(
            "{page} / {theme} ({} text, {} image, {} blur nodes), over {control}:\n  \
             own: whole {} bytes, delta {delta} ({dirty} of {} layers dirty), as a patch of the \
             dirty layer {patched} ({written} nodes written), unchanged frame {same} \
             (header only; nothing need go)\n  \
             monster: one frame {} + assets {}, two frames {} + assets {}: the second frame costs {} bytes; \
             {} clip nodes, {} blur, {} shadow left out of each frame",
            k[0],
            k[1],
            k[2],
            b.whole(),
            b.layers.len(),
            one.bytes,
            one.assets,
            two.bytes,
            two.assets,
            (two.bytes + two.assets).saturating_sub(one.bytes + one.assets),
            two.skipped[0] + two.skipped[1],
            two.skipped[2],
            two.skipped[3],
        );
    }
}
