//! rulers measured in-session (stat -f %z; gzip -9 -c | wc -c):
//!   explosion json 698654 -> 207434 gz, webm 38642;
//!   girl json 742135 -> 122861 gz, webm 111344.
//! run: cargo test -p monster --test bench_lottie -- --nocapture

use crate::lot::{Scan, f, scan_layer};
use monster::{
    Asset, AssetKind, Keyframe, Node, NodeKind, OptimizeCfg, Props, Timeline, decode, encode,
    optimize,
};
use serde_json::Value as J;

/// v/in/out times 2 coords times i32 twips, the starfish precedent.
pub const VERT_BYTES: usize = 24;
/// Static paint weight: rgba8 plus width or gradient extras.
pub const PAINT_BYTES: usize = 8;

const REF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../ref/lottie");
const EXPLOSION_JSON_BYTES: usize = 698654;
const EXPLOSION_GZIP_BYTES: usize = 207434;
const EXPLOSION_WEBM_BYTES: usize = 38642;

pub fn load(name: &str) -> J {
    let path = format!("{REF}/{name}");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("dense bench reads {path}: {e}"));
    serde_json::from_slice(&bytes).expect("ref lottie json parses")
}

/// The dense benches measure against large lottie json under `ref/lottie`,
/// which is gitignored study material (AGENTS.md). when it is absent (a fresh
/// clone, CI, a sandbox), the bench has nothing to weigh, so it skips with a
/// notice instead of failing: a missing study fixture is not a regression.
/// returns true (and prints the skip line) when the fixture is not present.
pub fn fixture_missing(name: &str) -> bool {
    let path = format!("{REF}/{name}");
    if std::path::Path::new(&path).exists() {
        return false;
    }
    eprintln!("skip: ref lottie fixture absent ({path}); populate ref/lottie to run this bench");
    true
}

pub struct Dense {
    pub name: &'static str,
    pub raw: usize,
    pub opt: usize,
    pub duration_s: f32,
    pub json: usize,
    pub gzip: usize,
    pub webm: usize,
}

pub fn report(b: &Dense) {
    let bps = |n: usize| n as f32 / b.duration_s;
    println!("== {} ==", b.name);
    println!("  monster raw {:>7} B  {:>9.1} B/s", b.raw, bps(b.raw));
    let delta = b.opt as i64 - b.raw as i64;
    println!(
        "  monster opt {:>7} B  {:>9.1} B/s  (optimizer {delta:+} B)",
        b.opt,
        bps(b.opt)
    );
    println!("  json    {:>7} B  {:>9.1} B/s", b.json, bps(b.json));
    println!("  gzip-9  {:>7} B  {:>9.1} B/s", b.gzip, bps(b.gzip));
    println!("  webm    {:>7} B  {:>9.1} B/s", b.webm, bps(b.webm));
    println!(
        "  monster/json {:.3}x  monster/gzip {:.3}x  monster/webm {:.3}x  gate(monster<=gzip): {}",
        b.opt as f32 / b.json as f32,
        b.opt as f32 / b.gzip as f32,
        b.opt as f32 / b.webm as f32,
        if b.opt <= b.gzip { "PASS" } else { "FAIL" }
    );
}

/// Path node over a fresh asset of `verts` quantized vertices.
pub fn path_node(slot: u16, verts: usize, assets: &mut Vec<Asset>) -> Node {
    let id = assets.len() as u16;
    assets.push(Asset {
        kind: AssetKind::Path,
        data: vec![0; verts * VERT_BYTES],
    });
    Node {
        id: slot,
        depth: slot,
        kind: NodeKind::Path { path: id },
        props: Props::new(),
    }
}

/// (raw bytes, optimized bytes, optimized timeline): encode as
/// authored, then through the default optimizer, then prove the
/// optimized file decodes.
pub fn encode_pair(tl: &Timeline, assets: &[Asset]) -> (usize, usize, Timeline) {
    let raw = encode(tl, assets, &[]).expect("dense timeline encodes");
    let opt_tl = optimize(tl, &OptimizeCfg::default()).expect("dense timeline optimizes");
    let opt = encode(&opt_tl, assets, &[]).expect("optimized timeline encodes");
    decode(&opt).expect("optimized monster decodes back");
    (raw.len(), opt.len(), opt_tl)
}

#[test]
fn bench_explosion_vs_lottie() {
    if fixture_missing("explosion/Explosion.json") {
        return;
    }
    let doc = load("explosion/Explosion.json");
    let fr = f(&doc["fr"]);
    let dur = (f(&doc["op"]) - f(&doc["ip"])) / fr;
    let mut cels: Vec<Scan> = doc["layers"]
        .as_array()
        .expect("explosion layers")
        .iter()
        .map(|l| scan_layer(l, fr, 0.0, 0.0, dur))
        .collect();
    cels.sort_by(|a, b| a.ip_s.total_cmp(&b.ip_s));
    assert!(
        cels.iter()
            .all(|c| c.nums.is_empty() && c.morphs.is_empty()),
        "explosion is pure cel animation; an animated prop means the mapping went stale"
    );
    let mut assets = Vec::new();
    let mut fresh: u16 = 0;
    let mut keyframes = Vec::new();
    for cel in &cels {
        let mut snapshot = Vec::new();
        for verts in &cel.static_paths {
            snapshot.push(path_node(fresh, *verts, &mut assets));
            fresh += 1;
        }
        if cel.paints > 0 {
            assets.push(Asset {
                kind: AssetKind::Path,
                data: vec![0; cel.paints * PAINT_BYTES],
            });
        }
        keyframes.push(Keyframe {
            t: cel.ip_s,
            snapshot,
        });
    }
    let timeline = Timeline {
        duration_s: dur,
        fps_hint: fr as u16,
        keyframes,
        ..Timeline::default()
    };
    let (raw, opt, _) = encode_pair(&timeline, &assets);
    println!(
        "  [explosion] cels {}  path nodes {}  vertices {}  assets {}",
        cels.len(),
        cels.iter().map(|c| c.static_paths.len()).sum::<usize>(),
        cels.iter().flat_map(|c| &c.static_paths).sum::<usize>(),
        assets.len()
    );
    report(&Dense {
        name: "explosion (cel animation: 17 snapshots of static path geometry)",
        raw,
        opt,
        duration_s: dur,
        json: EXPLOSION_JSON_BYTES,
        gzip: EXPLOSION_GZIP_BYTES,
        webm: EXPLOSION_WEBM_BYTES,
    });
}
