//! player tests, all GPU free: determinism (same ticks -> same scenes),
//! scrub == play-until-t, validity windows (the search counter does not
//! grow between boundaries), reactive signals, sub-epsilon skip, and
//! sampling through plev's own ease() + Interpolate.

use crate::easing::Easing;
use crate::ir::{IrError, Keyframe, Node, NodeKind, Prop, Props, Segment, Timeline, Track, Value};
use crate::play::{EPSILON_S, MonsterPlayer};
use engine::animation::{AnimationTick, Interpolate};
use engine::compositor::SceneNode;

fn tick(dt: f32) -> AnimationTick {
    // elapsed is wall-clock data; the player must key off dt alone.
    AnimationTick { dt, elapsed: 999.0 }
}

fn rect(id: u16, depth: u16, x: f32, color: [f32; 4]) -> Node {
    Node {
        id,
        depth,
        kind: NodeKind::Rect,
        props: Props::new()
            .with(Prop::X, Value::Scalar(x))
            .with(Prop::Y, Value::Scalar(20.0))
            .with(Prop::W, Value::Scalar(100.0))
            .with(Prop::H, Value::Scalar(50.0))
            .with(Prop::Color, Value::Color(color)),
    }
}

fn seg(target: Value, easing: Easing, dur_s: f32) -> Segment {
    Segment {
        target,
        easing,
        dur_s,
    }
}

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Two keyframes, eased motion and a color fade in the first window, a
/// second chain owned by the second keyframe.
fn demo() -> Timeline {
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![
            Keyframe {
                t: 0.0,
                snapshot: vec![rect(1, 0, 0.0, RED), rect(2, 1, 200.0, BLUE)],
            },
            Keyframe {
                t: 1.0,
                snapshot: vec![rect(1, 0, 80.0, RED), rect(2, 1, 240.0, BLUE)],
            },
        ],
        tracks: vec![
            Track {
                node_id: 1,
                prop: Prop::X,
                start_t: 0.0,
                segments: vec![
                    seg(Value::Scalar(100.0), Easing::EaseInOut, 0.5),
                    seg(Value::Scalar(60.0), Easing::Linear, 0.5),
                ],
            },
            Track {
                node_id: 1,
                prop: Prop::Color,
                start_t: 0.0,
                segments: vec![seg(Value::Color(BLUE), Easing::Linear, 1.0)],
            },
            Track {
                node_id: 2,
                prop: Prop::X,
                start_t: 1.0,
                segments: vec![seg(Value::Scalar(0.0), Easing::EaseOutCubic, 1.0)],
            },
        ],
        ..Timeline::default()
    }
}

/// Single keyframe, one two-segment chain: the window fixture.
fn two_windows() -> Timeline {
    Timeline {
        duration_s: 2.0,
        fps_hint: 60,
        keyframes: vec![Keyframe {
            t: 0.0,
            snapshot: vec![rect(1, 0, 0.0, RED)],
        }],
        tracks: vec![Track {
            node_id: 1,
            prop: Prop::X,
            start_t: 0.0,
            segments: vec![
                seg(Value::Scalar(100.0), Easing::Linear, 1.0),
                seg(Value::Scalar(0.0), Easing::Linear, 1.0),
            ],
        }],
        ..Timeline::default()
    }
}

fn rect_x(scene: &[SceneNode], at: usize) -> f32 {
    match &scene[at] {
        SceneNode::Rect { x, .. } => *x,
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn new_rejects_invalid_timeline() {
    let mut t = demo();
    t.keyframes.remove(0);
    assert_eq!(
        MonsterPlayer::new(t).err(),
        Some(IrError::MissingOpeningKeyframe)
    );
}

#[test]
fn determinism_same_ticks_same_scenes() {
    let mut a = MonsterPlayer::new(demo()).unwrap();
    let mut b = MonsterPlayer::new(demo()).unwrap();
    a.play();
    b.play();
    let dts = [0.016, 0.033, 0.007, 0.142, 0.4, 0.016, 0.25, 0.6];
    for dt in dts {
        a.tick(&tick(dt));
        b.tick(&tick(dt));
        assert_eq!(a.scene(), b.scene(), "diverged at dt {dt}");
        assert_eq!(a.current_time().to_bits(), b.current_time().to_bits());
    }
}

#[test]
fn scrub_equals_play_until_t() {
    let mut played = MonsterPlayer::new(demo()).unwrap();
    played.play();
    for _ in 0..23 {
        played.tick(&tick(0.029));
        let mut scrubbed = MonsterPlayer::new(demo()).unwrap();
        scrubbed.scrub(played.current_time());
        assert_eq!(played.scene(), scrubbed.scene());
    }
}

#[test]
fn validity_window_no_research_between_boundaries() {
    let mut p = MonsterPlayer::new(two_windows()).unwrap();
    p.play();
    p.tick(&tick(0.01));
    p.scene();
    // first evaluation: one keyframe seek + one chain derivation
    assert_eq!(p.segment_searches(), 2);
    for _ in 0..80 {
        p.tick(&tick(0.01)); // stays inside [0,1): t reaches 0.81
        p.scene();
    }
    assert_eq!(p.segment_searches(), 2, "ticks inside the window searched");
    p.scrub(0.95); // still the same window, even via scrub
    p.scene();
    assert_eq!(p.segment_searches(), 2);
    p.scrub(1.5); // crosses into the second segment: exactly one search
    p.scene();
    assert_eq!(p.segment_searches(), 3);
    p.scrub(1.2); // backwards inside the same window: cache holds
    p.scene();
    assert_eq!(p.segment_searches(), 3);
}

#[test]
fn signals_reflect_play_pause_scrub() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    let (time, playing) = (p.time(), p.playing());
    assert!(!playing.get());
    assert_eq!(time.get(), 0.0);

    p.play();
    assert!(playing.get());
    p.tick(&tick(0.5));
    assert_eq!(time.get(), 0.5);

    p.pause();
    assert!(!playing.get());
    p.tick(&tick(0.5));
    assert_eq!(time.get(), 0.5, "paused player advanced");

    p.scrub(1.25);
    assert_eq!(time.get(), 1.25);
    assert!(!playing.get(), "scrub changed play state");
}

#[test]
fn sub_epsilon_tick_is_skipped() {
    let mut p = MonsterPlayer::new(two_windows()).unwrap();
    p.play();
    p.tick(&tick(0.25));
    let before = p.scene();
    let searches = p.segment_searches();
    for _ in 0..100 {
        p.tick(&tick(EPSILON_S / 2.0));
    }
    assert_eq!(p.time().get(), 0.25);
    assert_eq!(p.scene(), before);
    assert_eq!(p.segment_searches(), searches);
}

#[test]
fn end_clamps_pauses_and_play_replays() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    p.play();
    p.tick(&tick(5.0));
    assert_eq!(p.time().get(), 2.0);
    assert!(!p.playing().get(), "end did not pause");
    p.play();
    assert_eq!(p.time().get(), 0.0, "replay did not rewind");
    assert!(p.playing().get());
}

#[test]
fn scrub_clamps_and_ignores_non_finite() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    p.scrub(-5.0);
    assert_eq!(p.time().get(), 0.0);
    p.scrub(99.0);
    assert_eq!(p.time().get(), 2.0);
    p.scrub(f32::NAN);
    assert_eq!(p.time().get(), 2.0);
}

#[test]
fn sampling_matches_plev_ease_and_interpolate() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    // first segment of node 1: 0 -> 100 over 0.5s, EaseInOut
    let expected = 0.0f32.lerp(
        &100.0,
        engine::animation::ease(0.25 / 0.5, engine::animation::Easing::EaseInOut),
    );
    assert_eq!(rect_x(&p.scene_at(0.25), 0), expected);
}

#[test]
fn second_segment_starts_from_first_target() {
    let mut p = MonsterPlayer::new(two_windows()).unwrap();
    // chain 0 -> 100 over [0,1), -> 0 over [1,2): halfway back is 50
    assert_eq!(rect_x(&p.scene_at(1.5), 0), 50.0);
    // past the chain end the last target holds
    assert_eq!(rect_x(&p.scene_at(2.0), 0), 0.0);
}

#[test]
fn color_track_lerps_rgba() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    match &p.scene_at(0.5)[0] {
        SceneNode::Rect { color, .. } => assert_eq!(*color, [0.5, 0.0, 0.5, 1.0]),
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn hold_easing_keeps_from_value_until_segment_end() {
    let mut t = two_windows();
    t.tracks[0].segments = vec![seg(Value::Scalar(100.0), Easing::Hold, 1.0)];
    let mut p = MonsterPlayer::new(t).unwrap();
    assert_eq!(rect_x(&p.scene_at(0.9), 0), 0.0);
    assert_eq!(rect_x(&p.scene_at(1.0), 0), 100.0);
}

#[test]
fn next_keyframe_snapshot_resets_earlier_tracks() {
    let mut p = MonsterPlayer::new(demo()).unwrap();
    // at t=1.5 the second keyframe governs: node 1 sits at its
    // snapshot x (80), not at the end of the first window's chain (60)
    let scene = p.scene_at(1.5);
    assert_eq!(rect_x(&scene, 0), 80.0);
    // and node 2's own chain (240 -> 0, EaseOutCubic) is halfway in
    let expected = 240.0f32.lerp(
        &0.0,
        engine::animation::ease(0.5, engine::animation::Easing::EaseOutCubic),
    );
    assert_eq!(rect_x(&scene, 1), expected);
}

#[test]
fn track_before_start_keeps_snapshot_value() {
    let mut t = two_windows();
    t.tracks[0].start_t = 0.5;
    t.tracks[0].segments = vec![seg(Value::Scalar(100.0), Easing::Linear, 1.0)];
    let mut p = MonsterPlayer::new(t).unwrap();
    assert_eq!(rect_x(&p.scene_at(0.2), 0), 0.0);
    assert_eq!(rect_x(&p.scene_at(1.0), 0), 50.0);
}

#[test]
fn decoded_file_plays_like_the_authored_timeline() {
    // grid-aligned values round-trip the codec exactly, so the decoded
    // player must produce bit-identical scenes (codec -> player seam)
    let bytes = crate::write::encode(&demo(), &[], &[]).unwrap();
    let doc = crate::read::decode(&bytes).unwrap();
    let mut authored = MonsterPlayer::new(demo()).unwrap();
    let mut decoded = MonsterPlayer::new(doc.timeline).unwrap();
    for i in 0..=20 {
        let t = i as f32 * 0.1;
        assert_eq!(authored.scene_at(t), decoded.scene_at(t), "at t={t}");
    }
}
