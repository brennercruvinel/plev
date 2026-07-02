//! bench vs lottie (spec gate, kdb/adr/monster-format-v0.md): IR timelines that
//! reproduce the animation semantics of the two lightest samples in
//! refs/anim/lottie-samples, encoded and measured against the json and
//! its gzip -9 (the spec ruler: monster born at or below the gzip size).
//!
//! mapping honesty notes (read the json before touching the numbers):
//! - ripple.json (11458 B, 361 f @ 60 fps = 6.0167 s, 6 authored
//!   keyframes): 2 precomp instances x 23 "grow bar" instances of one
//!   comp (bar line + 2 dots + slider null), phase-shifted 3 frames
//!   each (st), driven by 6 expressions and loopOut('cycle'). monster v0
//!   has no instancing, no expressions, no loop, so the rendered scene
//!   is materialized: 138 rect nodes (46 bars + 92 dots), one sweep
//!   track each, the 2 s cycle unrolled across the 6.0167 s duration
//!   with per-instance phase. easing (0.4,0,0.6,1) on every authored
//!   segment, deduped to one table entry. partial first/last cycle
//!   segments reuse the same curve (v0 cannot slice a bezier).
//! - starfish.json (24780 B, 301 f @ 60 fps = 5.0167 s, 42 authored
//!   keyframes): body path (10 vertices, static geometry), 5 animated
//!   limb nulls (31 position kfs) parented to a "main" null (4 kfs,
//!   x only), eye blink (mask slide, 3 kfs, retimed by a 2-kf identity
//!   time remap), scale pop (2 kfs), 2 extra shapes. monster v0 has no
//!   parenting and no path-vertex animation, so: body = static path
//!   node + 240 B path asset (10 pts x 24 B quantized); limbs = rects
//!   whose X merges own motion with main's (boundary union, summed
//!   values); main's ride baked into each riding node's X; blink =
//!   eyelid rect Y track. lottie ae-default (.167,.167,.833,.833)
//!   maps to the ae-default preset byte, holds to the hold preset,
//!   everything else stays a custom bezier (deduped table).
//!
//! rulers measured in-session (gzip -9 -c file | wc -c):
//!   ripple.json 11458 -> 1617 gz; starfish.json 24780 -> 3144 gz.
//! run: cargo test -p monster --test bench_lottie -- --nocapture

mod dense;
mod girl;
mod kfs;
mod lot;

use kfs::{Kfs, cb, rect, starfish_limbs};
use monster::{
    Asset, AssetKind, Easing, Keyframe, Node, NodeKind, OptimizeCfg, Prop, Props, Timeline, decode,
    encode, optimize,
};

const RIPPLE_JSON_BYTES: usize = 11458;
const RIPPLE_GZIP_BYTES: usize = 1617;
const RIPPLE_DURATION_S: f32 = 361.0 / 60.0;
const STARFISH_JSON_BYTES: usize = 24780;
const STARFISH_GZIP_BYTES: usize = 3144;
const STARFISH_DURATION_S: f32 = 301.0 / 60.0;

struct Bench {
    name: &'static str,
    monster_bytes: usize,
    duration_s: f32,
    json_bytes: usize,
    gzip_bytes: usize,
    nodes: usize,
    tracks: usize,
    segments: usize,
}

fn report(b: &Bench) {
    let bps = b.monster_bytes as f32 / b.duration_s;
    let json_bps = b.json_bytes as f32 / b.duration_s;
    let gz_bps = b.gzip_bytes as f32 / b.duration_s;
    println!("== {} ==", b.name);
    println!(
        "  nodes {}  tracks {}  segments {}",
        b.nodes, b.tracks, b.segments
    );
    println!("  monster    {:>6} B  {:>8.1} B/s", b.monster_bytes, bps);
    println!("  json   {:>6} B  {:>8.1} B/s", b.json_bytes, json_bps);
    println!("  gzip-9 {:>6} B  {:>8.1} B/s", b.gzip_bytes, gz_bps);
    println!(
        "  monster/json {:.3}x   monster/gzip {:.3}x   gate(monster<=gzip): {}",
        b.monster_bytes as f32 / b.json_bytes as f32,
        b.monster_bytes as f32 / b.gzip_bytes as f32,
        if b.monster_bytes <= b.gzip_bytes {
            "PASS"
        } else {
            "FAIL"
        }
    );
}

/// Encode again through the default optimizer and print the delta;
/// these timelines are authored with eased (non-linear) segments, so
/// the passes can only act where a chain is static or linear.
fn print_optimized(name: &str, timeline: &Timeline, assets: &[Asset], raw: usize) {
    let opt_tl = optimize(timeline, &OptimizeCfg::default()).expect("bench timeline optimizes");
    let opt = encode(&opt_tl, assets, &[]).expect("optimized bench timeline encodes");
    let segs = |tl: &Timeline| tl.tracks.iter().map(|t| t.segments.len()).sum::<usize>();
    println!(
        "  [{name}] optimizer: {raw} B -> {} B ({:+} B); tracks {} -> {}, segments {} -> {}",
        opt.len(),
        opt.len() as i64 - raw as i64,
        timeline.tracks.len(),
        opt_tl.tracks.len(),
        segs(timeline),
        segs(&opt_tl)
    );
}

/// The ripple sweep: gradient end x 396 -> -7 over 60 f, back over the
/// next 60 f, loopOut('cycle'), instance phase-shifted by 3 f. This
/// unrolls one instance's cycle over the whole duration from its phase.
fn sweep_kfs(phase_frames: f32) -> Kfs {
    let ease = cb(0.4, 0.0, 0.6, 1.0);
    let (a, b) = (396.0, -7.0);
    let half = |pos: f32| {
        if pos < 1.0 {
            (a, b, pos)
        } else {
            (b, a, pos - 1.0)
        }
    };
    let pos0 = (phase_frames / 60.0) % 2.0;
    let (from, to, frac) = half(pos0);
    let v0 = from + (to - from) * ease.sample(frac);
    let mut t = vec![0.0f32];
    let mut v = vec![v0];
    let mut e = vec![ease];
    let mut acc = if frac > 0.0 { 1.0 - frac } else { 1.0 };
    let mut target_half = if pos0 < 1.0 { 1.0 } else { 0.0 }; // half ending the 1st seg
    loop {
        let clamped = acc.min(RIPPLE_DURATION_S);
        t.push(clamped);
        let (hf, ht, _) = half(target_half);
        if clamped < acc {
            // last partial segment: eased value at the cut, same curve
            let k = 1.0 - (acc - clamped);
            v.push(hf + (ht - hf) * ease.sample(k));
        } else {
            v.push(ht);
        }
        if clamped >= RIPPLE_DURATION_S {
            break;
        }
        e.push(ease);
        target_half = if target_half == 1.0 { 0.0 } else { 1.0 };
        acc += 1.0;
    }
    let frames: Vec<f32> = t.iter().map(|s| s * 60.0).collect();
    Kfs::frames(&frames, &v, &e)
}

#[test]
fn bench_ripple_vs_lottie() {
    let teal = [0.0, 0.6, 0.8, 1.0];
    let white = [1.0, 1.0, 1.0, 1.0];
    let mut snapshot = Vec::new();
    let mut tracks = Vec::new();
    for unit in 0..46u16 {
        let bar_i = unit % 23;
        let x = 22.0 + 34.4545 * bar_i as f32;
        let y = if unit < 23 { 221.0 } else { 387.0 };
        let phase = (120.0 - 3.0 * bar_i as f32) % 120.0;
        let kfs = sweep_kfs(phase);
        let (bar_id, top_id, bot_id) = (1 + unit * 3, 2 + unit * 3, 3 + unit * 3);
        let (w0, bar_track) = kfs.track(bar_id, Prop::W);
        snapshot.push(rect(bar_id, unit * 3, x, y, w0, 2.0, teal));
        tracks.push(bar_track);
        let (dx0, top_track) = kfs.track(top_id, Prop::X);
        snapshot.push(rect(top_id, 1 + unit * 3, dx0, y - 6.0, 12.0, 12.0, white));
        tracks.push(top_track);
        let (bx0, bot_track) = kfs.track(bot_id, Prop::X);
        snapshot.push(rect(bot_id, 2 + unit * 3, bx0, y + 6.0, 12.0, 12.0, white));
        tracks.push(bot_track);
    }
    let timeline = Timeline {
        duration_s: RIPPLE_DURATION_S,
        fps_hint: 60,
        keyframes: vec![Keyframe { t: 0.0, snapshot }],
        tracks,
        ..Timeline::default()
    };
    let bytes = encode(&timeline, &[], &[]).expect("ripple timeline encodes");
    decode(&bytes).expect("ripple monster decodes back");
    print_optimized("ripple", &timeline, &[], bytes.len());
    report(&Bench {
        name: "ripple.json (materialized: no instancing/expressions/loop in v0)",
        monster_bytes: bytes.len(),
        duration_s: RIPPLE_DURATION_S,
        json_bytes: RIPPLE_JSON_BYTES,
        gzip_bytes: RIPPLE_GZIP_BYTES,
        nodes: timeline.keyframes[0].snapshot.len(),
        tracks: timeline.tracks.len(),
        segments: timeline.tracks.iter().map(|t| t.segments.len()).sum(),
    });

    // context: the authored unit cell (1 bar + 2 dots, one 2 s cycle),
    // what one comp_79 instance costs before materialization.
    let kfs = sweep_kfs(0.0);
    let cell_kfs = Kfs {
        t: kfs.t[..3].to_vec(),
        v: kfs.v[..3].to_vec(),
        e: kfs.e[..2].to_vec(),
    };
    let mut snapshot = Vec::new();
    let mut tracks = Vec::new();
    for (i, prop) in [(0u16, Prop::W), (1, Prop::X), (2, Prop::X)] {
        let (v0, track) = cell_kfs.track(i + 1, prop);
        let node = if prop == Prop::W {
            rect(i + 1, i, 22.0, 221.0, v0, 2.0, teal)
        } else {
            rect(i + 1, i, v0, 215.0 + 12.0 * i as f32, 12.0, 12.0, white)
        };
        snapshot.push(node);
        tracks.push(track);
    }
    let cell = Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe { t: 0.0, snapshot }],
        tracks,
        ..Timeline::default()
    };
    let cell_bytes = encode(&cell, &[], &[]).expect("unit cell encodes");
    println!(
        "  context: authored unit cell (3 nodes, 1 cycle) = {} B ({:.1} B/s)",
        cell_bytes.len(),
        cell_bytes.len() as f32 / 2.0
    );
}

#[test]
fn bench_starfish_vs_lottie() {
    let orange = [0.95, 0.6, 0.2, 1.0];
    // main null: slides in from the right, holds, slides out (x only;
    // its y is constant in the json so no y track exists anywhere).
    let main_x = Kfs::frames(
        &[0.0, 69.0, 215.0, 296.0],
        &[1228.969, 421.969, 421.969, 1228.969],
        &[
            cb(0.89, 0.0, 0.833, 0.833),
            Easing::Hold,
            cb(0.97, 0.0, 0.833, 0.833),
        ],
    );
    let mut snapshot = Vec::new();
    let mut tracks = Vec::new();
    // body: static path node; v0 paths have no animatable props, so the
    // body cannot ride main (documented v0 limitation).
    snapshot.push(Node {
        id: 1,
        depth: 0,
        kind: NodeKind::Path { path: 0 },
        props: Props::new(),
    });
    // 5 limbs: own x merged with main's ride, own y.
    for (i, (times, xs, ys, eases)) in starfish_limbs().into_iter().enumerate() {
        let id = 2 + i as u16;
        let own_x = Kfs::frames(times, xs, &eases);
        let own_y = Kfs::frames(times, ys, &eases);
        let merged_x = own_x.plus(&main_x);
        let (x0, tx) = merged_x.track(id, Prop::X);
        let (y0, ty) = own_y.track(id, Prop::Y);
        snapshot.push(rect(id, id - 1, x0, y0, 40.0, 40.0, orange));
        tracks.push(tx);
        tracks.push(ty);
    }
    // riders on main: ellipse (SL3), popping shape (SL4), eyeball, eyelid.
    for (i, w) in [(0u16, 60.0f32), (1, 80.0), (2, 24.0), (3, 26.0)] {
        let id = 7 + i;
        let (x0, tx) = main_x.track(id, Prop::X);
        let y = 250.0 + 30.0 * i as f32;
        snapshot.push(rect(id, id - 1, x0, y, w, 30.0, orange));
        tracks.push(tx);
    }
    // SL4 scale pop at 114..126 f (one segment on w and on h).
    let pop = |prop, v1| {
        Kfs::frames(&[114.0, 126.0], &[80.0, v1], &[Easing::EaseInOut])
            .track(8, prop)
            .1
    };
    tracks.push(pop(Prop::W, 120.0));
    tracks.push(pop(Prop::H, 71.99));
    // eye blink: mask slides down then back (identity time remap).
    let blink = Kfs::frames(
        &[26.0, 31.588, 44.0],
        &[-38.188, -22.438, -38.188],
        &[cb(0.45, 0.0, 0.52, 1.0), cb(0.48, 0.0, 0.57, 1.0)],
    );
    tracks.push(blink.track(10, Prop::Y).1);
    let timeline = Timeline {
        duration_s: STARFISH_DURATION_S,
        fps_hint: 60,
        keyframes: vec![Keyframe { t: 0.0, snapshot }],
        tracks,
        ..Timeline::default()
    };
    // body geometry the renderer needs: 10 vertices x (v,in,out) x 2
    // coords x i32 twips = 240 B, the quantized weight of the json path.
    let body: Vec<u8> = (0..240u32).map(|i| (i * 7 % 251) as u8).collect();
    let assets = vec![Asset {
        kind: AssetKind::Path,
        data: body,
    }];
    let bytes = encode(&timeline, &assets, &[]).expect("starfish timeline encodes");
    decode(&bytes).expect("starfish monster decodes back");
    print_optimized("starfish", &timeline, &assets, bytes.len());
    report(&Bench {
        name: "starfish.json (parenting baked, blink as eyelid rect)",
        monster_bytes: bytes.len(),
        duration_s: STARFISH_DURATION_S,
        json_bytes: STARFISH_JSON_BYTES,
        gzip_bytes: STARFISH_GZIP_BYTES,
        nodes: timeline.keyframes[0].snapshot.len(),
        tracks: timeline.tracks.len(),
        segments: timeline.tracks.iter().map(|t| t.segments.len()).sum(),
    });
}
