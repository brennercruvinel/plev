//! girl fixture of the dense lottie benches; shared infrastructure in
//! `dense.rs`, json scan in `lot.rs`.
//!
//! mapping honesty notes (read the json before touching the numbers):
//! girl/json.json (742135 B, 60 fps, 95 f = 1.5833 s, 44 root layers
//! plus 3 precomps of 11/40/16 layers, each instanced once and
//! materialized at its instance offset; skateboard P2 rides at
//! st = 52 f) carries 287 animated properties and 2223 keyframes. monster
//! v0 has no parenting, rotation, scale, opacity or path-vertex
//! animation, so the mapping preserves the information density, not
//! the visual: every animated numeric property becomes segment chains
//! with the authored boundaries, values and bezier easing on slot
//! props of gradient_rect host nodes (1d -> one scalar track; 2d/3d ->
//! x and y tracks, the constant z filler dropped; values that all sit
//! in [0,1] -> one rgba color track, alpha padded); each animated path
//! becomes a path node plus one asset per keyframe (verts x 24 B)
//! swapped by replace ops (v0 morph = asset swap); static paths are
//! single assets; paints weigh 8 B each in one style asset; the text
//! layer is a text node plus its measured document bytes. layer in/out
//! windows become place/remove ops for path nodes (hosts stay in the
//! t=0 snapshot so their tracks validate); keyframes outside
//! [0, duration] or coincident after clamping collapse to the later
//! keyframe, without bezier slicing (v0 cannot slice a curve), which
//! also drops morph keyframes outside their layer's visibility.

use crate::dense::{Dense, encode_pair, fixture_missing, load, path_node, report};
use crate::lot::{NumKf, Scan, f, scan_layer};
use monster::{
    Asset, AssetKind, Keyframe, Node, NodeKind, PlaceNode, Prop, Props, RemoveNode, ReplaceNode,
    Segment, Timeline, Track, Value,
};

const GIRL_JSON_BYTES: usize = 742135;
const GIRL_GZIP_BYTES: usize = 122861;
const GIRL_WEBM_BYTES: usize = 111344;
/// Static paint weight, mirrored from `dense.rs`.
const PAINT_BYTES: usize = 8;

const SCALAR_SLOTS: [Prop; 6] = [
    Prop::X,
    Prop::Y,
    Prop::W,
    Prop::H,
    Prop::CornerRadius,
    Prop::BorderWidth,
];
const COLOR_SLOTS: [Prop; 3] = [Prop::Color, Prop::BorderColor, Prop::Color2];

/// gradient_rect hosts for the animated numeric properties: 6 scalar
/// and 3 color slots per host, filled greedily; the snapshot node
/// carries each chain's base value.
#[derive(Default)]
struct Hosts {
    nodes: Vec<Node>,
    scalars: usize,
    colors: usize,
}

impl Hosts {
    fn alloc(&mut self, fresh: &mut u16, init: Value) -> (u16, Prop) {
        let color = matches!(init, Value::Color(_));
        let cap = if color {
            COLOR_SLOTS.len()
        } else {
            SCALAR_SLOTS.len()
        };
        let used = if color { self.colors } else { self.scalars };
        if self.nodes.is_empty() || used >= cap {
            self.nodes.push(Node {
                id: *fresh,
                depth: *fresh,
                kind: NodeKind::GradientRect,
                props: Props::new(),
            });
            *fresh += 1;
            self.scalars = 0;
            self.colors = 0;
        }
        let prop = if color {
            self.colors += 1;
            COLOR_SLOTS[self.colors - 1]
        } else {
            self.scalars += 1;
            SCALAR_SLOTS[self.scalars - 1]
        };
        let host = self.nodes.last_mut().expect("a host exists");
        host.props.set(prop, init);
        (host.id, prop)
    }
}

/// Keyframe boundaries clamped into [0, dur], strictly increasing; a
/// boundary clamped onto another keeps the later keyframe.
fn boundaries(kfs: &[NumKf], dur: f32) -> Vec<(f32, &NumKf)> {
    let mut out: Vec<(f32, &NumKf)> = Vec::with_capacity(kfs.len());
    for kf in kfs {
        let t = kf.t.clamp(0.0, dur);
        match out.last_mut() {
            Some(last) if last.0 >= t => *last = (t, kf),
            _ => out.push((t, kf)),
        }
    }
    out
}

fn push_track(
    bnds: &[(f32, &NumKf)],
    val: &dyn Fn(&NumKf) -> Value,
    hosts: &mut Hosts,
    fresh: &mut u16,
    tracks: &mut Vec<Track>,
) {
    let (node_id, prop) = hosts.alloc(fresh, val(bnds[0].1));
    let segments = bnds
        .windows(2)
        .map(|w| Segment {
            target: val(w[1].1),
            easing: w[0].1.ez,
            dur_s: w[1].0 - w[0].0,
        })
        .collect();
    tracks.push(Track {
        node_id,
        prop,
        start_t: bnds[0].0,
        segments,
    });
}

fn scan_all(doc: &serde_json::Value, fr: f32, dur: f32) -> Vec<Scan> {
    let layers = doc["layers"].as_array().expect("girl layers");
    let empty = Vec::new();
    let mut scans: Vec<Scan> = layers
        .iter()
        .map(|l| scan_layer(l, fr, 0.0, 0.0, dur))
        .collect();
    for asset in doc["assets"].as_array().unwrap_or(&empty) {
        let id = asset["id"].as_str().unwrap_or_default();
        let inst = layers.iter().find(|l| l["refId"].as_str() == Some(id));
        let (off, lo, hi) = inst.map_or((0.0, 0.0, dur), |l| {
            let win = |k: &str| (f(&l[k]) / fr).clamp(0.0, dur);
            (f(&l["st"]) / fr, win("ip"), win("op"))
        });
        for layer in asset["layers"].as_array().unwrap_or(&empty) {
            scans.push(scan_layer(layer, fr, off, lo, hi));
        }
    }
    scans
}

#[test]
fn bench_girl_vs_lottie() {
    if fixture_missing("girl/json.json") {
        return;
    }
    let doc = load("girl/json.json");
    let fr = f(&doc["fr"]);
    let dur = (f(&doc["op"]) - f(&doc["ip"])) / fr;
    let scans = scan_all(&doc, fr, dur);
    let mut fresh: u16 = 0;
    let mut hosts = Hosts::default();
    let mut assets = Vec::new();
    let (mut tracks, mut snapshot) = (Vec::new(), Vec::new());
    let (mut places, mut replaces, mut removes) = (Vec::new(), Vec::new(), Vec::new());
    for scan in &scans {
        for kfs in &scan.nums {
            let n = kfs.first().map_or(0, |kf| kf.v.len());
            let bnds = boundaries(kfs, dur);
            if n == 0 || bnds.len() < 2 {
                continue;
            }
            let in_unit = |c: &f32| (-0.0001..=1.0001).contains(c);
            if n >= 3 && kfs.iter().all(|kf| kf.v.iter().all(in_unit)) {
                let val = |kf: &NumKf| {
                    let c = |i: usize| kf.v.get(i).copied().unwrap_or(1.0);
                    Value::Color([c(0), c(1), c(2), if n >= 4 { c(3) } else { 1.0 }])
                };
                push_track(&bnds, &val, &mut hosts, &mut fresh, &mut tracks);
            } else {
                for dim in 0..n.min(2) {
                    let val = |kf: &NumKf| Value::Scalar(kf.v.get(dim).copied().unwrap_or(0.0));
                    push_track(&bnds, &val, &mut hosts, &mut fresh, &mut tracks);
                }
            }
        }
        for morph in &scan.morphs {
            let mut bnds: Vec<(f32, usize)> = Vec::new();
            for kf in morph {
                let t = kf.t.clamp(scan.ip_s, scan.op_s);
                match bnds.last_mut() {
                    Some(last) if last.0 >= t => *last = (t, kf.verts),
                    _ => bnds.push((t, kf.verts)),
                }
            }
            if bnds.is_empty() {
                continue;
            }
            let slot = fresh;
            fresh += 1;
            let start = bnds.iter().rposition(|(t, _)| *t <= scan.ip_s).unwrap_or(0);
            let node = path_node(slot, bnds[start].1, &mut assets);
            if scan.ip_s <= 0.0 {
                snapshot.push(node);
            } else {
                places.push(PlaceNode { t: scan.ip_s, node });
            }
            for (t, verts) in &bnds[start + 1..] {
                let node = path_node(slot, *verts, &mut assets);
                replaces.push(ReplaceNode {
                    t: *t,
                    depth: slot,
                    node,
                });
            }
            if scan.op_s < dur - 1e-6 {
                removes.push(RemoveNode {
                    t: scan.op_s,
                    depth: slot,
                });
            }
        }
        for verts in &scan.static_paths {
            let slot = fresh;
            fresh += 1;
            let node = path_node(slot, *verts, &mut assets);
            if scan.ip_s <= 0.0 {
                snapshot.push(node);
            } else {
                places.push(PlaceNode { t: scan.ip_s, node });
            }
            if scan.op_s < dur - 1e-6 {
                removes.push(RemoveNode {
                    t: scan.op_s,
                    depth: slot,
                });
            }
        }
        if scan.text_bytes > 0 {
            let style = assets.len() as u16;
            assets.push(Asset {
                kind: AssetKind::TextStyle,
                data: vec![0; scan.text_bytes],
            });
            let slot = fresh;
            fresh += 1;
            snapshot.push(Node {
                id: slot,
                depth: slot,
                kind: NodeKind::Text { style },
                props: Props::new()
                    .with(Prop::X, Value::Scalar(0.0))
                    .with(Prop::Y, Value::Scalar(0.0))
                    .with(Prop::Color, Value::Color([1.0, 1.0, 1.0, 1.0])),
            });
        }
    }
    let paints: usize = scans.iter().map(|s| s.paints).sum();
    if paints > 0 {
        assets.push(Asset {
            kind: AssetKind::Path,
            data: vec![0; paints * PAINT_BYTES],
        });
    }
    snapshot.extend(hosts.nodes.iter().cloned());
    let timeline = Timeline {
        duration_s: dur,
        fps_hint: fr as u16,
        keyframes: vec![Keyframe { t: 0.0, snapshot }],
        tracks,
        places,
        replaces,
        removes,
    };
    let (raw, opt, opt_tl) = encode_pair(&timeline, &assets);
    let segs = |tl: &Timeline| tl.tracks.iter().map(|t| t.segments.len()).sum::<usize>();
    println!(
        "  [girl] nodes {} (hosts {})  tracks {}  segments {}  place/replace/remove {}/{}/{}  assets {}",
        timeline.keyframes[0].snapshot.len() + timeline.places.len(),
        hosts.nodes.len(),
        timeline.tracks.len(),
        segs(&timeline),
        timeline.places.len(),
        timeline.replaces.len(),
        timeline.removes.len(),
        assets.len()
    );
    println!(
        "  [girl] optimizer: tracks {} -> {}, segments {} -> {}",
        timeline.tracks.len(),
        opt_tl.tracks.len(),
        segs(&timeline),
        segs(&opt_tl)
    );
    report(&Dense {
        name: "girl (rig: 287 animated props as tracks, 72 path morphs as asset swaps)",
        raw,
        opt,
        duration_s: dur,
        json: GIRL_JSON_BYTES,
        gzip: GIRL_GZIP_BYTES,
        webm: GIRL_WEBM_BYTES,
    });
}
