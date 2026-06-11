//! optimizer tests, pipeline side: the full pass pipeline on rich
//! authored and discovered timelines: exact measured reduction,
//! idempotence (optimizing twice equals once), codec round-trip after
//! optimization, player parity within the configured tolerance, pin
//! safety, and the error path. builders and the quantized parity
//! helper come from tests_optimize.

use crate::discover::discover;
use crate::easing::Easing;
use crate::ir::{IrError, Keyframe, Node, Prop, Timeline};
use crate::optimize::{OptimizeCfg, optimize};
use crate::play::AnmPlayer;
use crate::read::decode;
use crate::tests_discover::{assert_reproduced, cfg as discover_cfg};
use crate::tests_optimize::{assert_quantized_parity, col, lin, seg, track};
use crate::tests_write::rect;
use crate::write::encode;
use plev::compositor::SceneNode;

/// 2 px on coordinates, 40/255 per color channel, 0.6 px for RDP.
fn aggressive() -> OptimizeCfg {
    OptimizeCfg {
        static_tol: 40.0,
        rdp_tol: 12.0,
        ..OptimizeCfg::default()
    }
}

fn n_segments(tl: &Timeline) -> usize {
    tl.tracks.iter().map(|t| t.segments.len()).sum()
}

/// Five tracks of known redundancy over two rects: a half-px wiggle,
/// a four-keyframe straight ramp, a redundant color midpoint, collinear
/// pairs around a fixed eased segment, and a constant chain.
fn authored_rich() -> Timeline {
    let ramp = vec![
        lin(30.0, 0.25),
        lin(40.0, 0.25),
        lin(50.0, 0.25),
        lin(60.0, 0.25),
    ];
    let fade = vec![
        col([0.6, 0.4, 0.6, 1.0], 0.5),
        col([1.0, 0.4, 0.6, 1.0], 0.5),
    ];
    let eased = vec![
        lin(220.0, 0.25),
        lin(240.0, 0.25),
        seg(300.0, 0.5, Easing::EaseOutCubic),
        lin(280.0, 0.25),
        lin(260.0, 0.25),
    ];
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect(1, 0, 10.0), rect(2, 1, 200.0)],
        }],
        tracks: vec![
            track(
                1,
                Prop::X,
                0.0,
                vec![lin(10.5, 0.5), lin(9.5, 0.5), lin(10.0, 0.5)],
            ),
            track(1, Prop::Y, 0.0, ramp),
            track(1, Prop::Color, 0.0, fade),
            track(2, Prop::X, 0.0, eased),
            track(2, Prop::Y, 0.5, vec![lin(20.0, 0.5), lin(20.0, 0.5)]),
        ],
        ..Timeline::default()
    }
}

/// The tests_discover_ops pin scenario: id 2 lives twice in one window,
/// so discovery emits twin tracks on (2, X), the dead chain and its pin.
fn pinned_frames() -> Vec<(f32, Vec<Node>)> {
    (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let mut scene = vec![rect(1, 0, 10.0)];
            if (1..=3).contains(&i) {
                scene.push(rect(2, 1, 20.0 + 40.0 * (t - 0.25)));
            }
            if i >= 6 {
                scene.push(rect(2, 1, 60.0));
            }
            (t, scene)
        })
        .collect()
}

/// Every rect prop of `b` stays within the given absolute bounds of
/// `a`, sampled on a 50 ms grid.
fn assert_rects_close(a: &Timeline, b: &Timeline, scalar_tol: f32, color_tol: f32) {
    let mut pa = AnmPlayer::new(a.clone()).unwrap();
    let mut pb = AnmPlayer::new(b.clone()).unwrap();
    let steps = (a.duration_s / 0.05).round() as u32;
    for k in 0..=steps {
        let t = k as f32 * 0.05;
        let (sa, sb) = (pa.scene_at(t), pb.scene_at(t));
        assert_eq!(sa.len(), sb.len(), "node count at t={t}");
        for (na, nb) in sa.iter().zip(&sb) {
            let (
                SceneNode::Rect { x, y, w, h, color },
                SceneNode::Rect {
                    x: x2,
                    y: y2,
                    w: w2,
                    h: h2,
                    color: color2,
                },
            ) = (na, nb)
            else {
                panic!("rects only at t={t}");
            };
            for (va, vb) in [(x, x2), (y, y2), (w, w2), (h, h2)] {
                assert!((va - vb).abs() <= scalar_tol, "{va} vs {vb} at t={t}");
            }
            for (ca, cb) in color.iter().zip(color2) {
                assert!((ca - cb).abs() <= color_tol, "{ca} vs {cb} at t={t}");
            }
        }
    }
}

#[test]
fn default_pipeline_reduces_the_authored_redundancy_exactly() {
    let tl = authored_rich();
    let opt = optimize(&tl, &OptimizeCfg::default()).unwrap();
    assert_eq!(
        (n_segments(&tl), n_segments(&opt)),
        (16, 8),
        "measured reduction"
    );
    assert_eq!(opt.tracks.len(), 4, "the constant chain is gone");
    assert_eq!(
        opt.tracks[0].segments.len(),
        3,
        "the wiggle is over half a step"
    );
    assert_eq!(opt.tracks[1].segments, vec![lin(60.0, 1.0)]);
    assert_eq!(opt.tracks[2].segments, vec![col([1.0, 0.4, 0.6, 1.0], 1.0)]);
    assert_eq!(
        opt.tracks[3].segments,
        vec![
            lin(240.0, 0.5),
            seg(300.0, 0.5, Easing::EaseOutCubic),
            lin(260.0, 0.5)
        ]
    );
    assert_quantized_parity(&tl, &opt);
}

#[test]
fn aggressive_optimization_stays_within_its_tolerance() {
    let tl = authored_rich();
    let opt = optimize(&tl, &aggressive()).unwrap();
    assert_eq!(opt.tracks.len(), 3, "wiggle and constant chain collapsed");
    assert_eq!(n_segments(&opt), 5);
    assert_rects_close(&tl, &opt, 2.0 + 1e-3, 40.0 / 255.0 + 1e-3);
}

#[test]
fn optimizing_twice_equals_optimizing_once() {
    let rich = authored_rich();
    let pin = discover(&pinned_frames(), &discover_cfg(100.0, 100.0)).unwrap();
    let cadenced = discover(&pinned_frames(), &discover_cfg(1.0, 100.0)).unwrap();
    for cfg in [OptimizeCfg::default(), aggressive()] {
        for tl in [&rich, &pin, &cadenced] {
            let once = optimize(tl, &cfg).unwrap();
            let twice = optimize(&once, &cfg).unwrap();
            assert_eq!(twice, once, "fixpoint under {cfg:?}");
        }
    }
}

#[test]
fn optimized_timelines_round_trip_the_codec() {
    let pin = discover(&pinned_frames(), &discover_cfg(100.0, 100.0)).unwrap();
    for tl in [authored_rich(), pin] {
        let opt = optimize(&tl, &OptimizeCfg::default()).unwrap();
        let doc = decode(&encode(&opt, &[], &[]).unwrap()).unwrap();
        assert_eq!(doc.timeline, opt);
    }
}

#[test]
fn pinned_twins_survive_collapse_and_playback() {
    let frames = pinned_frames();
    let tl = discover(&frames, &discover_cfg(100.0, 100.0)).unwrap();
    // a tolerance loose enough to call everything static: the twin
    // rule alone protects the pin and the dead chain it overrides.
    let blunt = OptimizeCfg {
        static_tol: 1000.0,
        ..OptimizeCfg::default()
    };
    let opt = optimize(&tl, &blunt).unwrap();
    assert_eq!(opt.tracks, tl.tracks, "twins skip the collapse");
    assert_reproduced(&opt, &frames);
    let mut player = AnmPlayer::new(opt).unwrap();
    for t in [1.5, 1.75, 2.0] {
        let scene = player.scene_at(t);
        let SceneNode::Rect { x, .. } = scene[1] else {
            panic!("rect at depth 1");
        };
        assert_eq!(x, 60.0, "no bleed at t={t}");
    }
}

#[test]
fn malformed_timeline_is_a_typed_error() {
    assert_eq!(
        optimize(&Timeline::default(), &OptimizeCfg::default()),
        Err(IrError::MissingOpeningKeyframe)
    );
}

#[test]
fn disabled_passes_leave_the_timeline_untouched() {
    let off = OptimizeCfg {
        collapse_static: false,
        reduce_rdp: false,
        fuse_collinear: false,
        ..OptimizeCfg::default()
    };
    let tl = authored_rich();
    assert_eq!(optimize(&tl, &off).unwrap(), tl);
}
