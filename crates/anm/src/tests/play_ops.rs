//! player tests for structural delta ops: nodes appear, swap and leave
//! at the authored instants, scrubbing in either direction replays only
//! the governing segment's ops, snapshots reset everything, and tracks
//! follow their node through place and remove. codec coverage of the
//! same document lives in tests_ops.

use crate::ir::{Keyframe, PlaceNode, Prop, RemoveNode, Track};
use crate::play::AnmPlayer;
use crate::tests::ops::ops_timeline;
use crate::tests::write::{rect, seg};
use plev::animation::AnimationTick;
use plev::compositor::SceneNode;

fn rect_x(scene: &[SceneNode], at: usize) -> f32 {
    match &scene[at] {
        SceneNode::Rect { x, .. } => *x,
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn place_replace_remove_act_at_the_authored_instants() {
    let mut p = AnmPlayer::new(ops_timeline()).unwrap();
    assert_eq!(p.scene_at(0.4).len(), 1, "placed node visible early");
    let at_place = p.scene_at(0.5);
    assert_eq!(at_place.len(), 2, "place is inclusive at its t");
    assert_eq!(rect_x(&at_place, 1), 40.0);
    assert_eq!(rect_x(&p.scene_at(1.0), 1), 80.0, "replace swaps the slot");
    assert_eq!(p.scene_at(1.4).len(), 2);
    assert_eq!(p.scene_at(1.5).len(), 1, "remove empties the slot at t");
    assert_eq!(p.scene_at(2.0).len(), 1);
}

#[test]
fn scrubbing_backwards_replays_the_segment_not_the_file() {
    let mut p = AnmPlayer::new(ops_timeline()).unwrap();
    assert_eq!(p.scene_at(1.8).len(), 1);
    // back inside the same segment: the op replay is from the keyframe
    // snapshot, so the placed node is alive again.
    let back = p.scene_at(0.6);
    assert_eq!(back.len(), 2);
    assert_eq!(rect_x(&back, 1), 40.0, "first life, not the replacement");
    assert_eq!(p.scene_at(0.0).len(), 1);
}

#[test]
fn tracks_keep_their_windowed_values_alongside_ops() {
    let mut p = AnmPlayer::new(ops_timeline()).unwrap();
    // node 1: x 10 -> 150 linear over [0.25, 0.75]
    assert_eq!(rect_x(&p.scene_at(0.5), 0), 80.0);
    assert_eq!(rect_x(&p.scene_at(1.9), 0), 150.0, "past chain end holds");
}

#[test]
fn track_on_a_placed_node_animates_from_the_placed_value() {
    let mut tl = ops_timeline();
    tl.replaces.clear();
    tl.removes.clear();
    tl.tracks.push(Track {
        node_id: 2,
        prop: Prop::X,
        start_t: 0.5,
        segments: vec![seg(60.0, crate::easing::Easing::Linear)],
    });
    let mut p = AnmPlayer::new(tl).unwrap();
    assert_eq!(rect_x(&p.scene_at(0.5), 1), 40.0, "chain starts at place");
    assert_eq!(rect_x(&p.scene_at(0.75), 1), 50.0, "halfway 40 -> 60");
    assert_eq!(rect_x(&p.scene_at(1.5), 1), 60.0);
}

#[test]
fn removing_a_tracked_node_stops_its_chain_without_panic() {
    let mut tl = ops_timeline();
    tl.removes = vec![RemoveNode { t: 0.75, depth: 0 }];
    let mut p = AnmPlayer::new(tl).unwrap();
    assert_eq!(rect_x(&p.scene_at(0.5), 0), 80.0, "chain runs before");
    let after = p.scene_at(0.75);
    assert_eq!(after.len(), 1, "tracked node removed mid-chain");
    assert_eq!(rect_x(&after, 0), 40.0, "only the placed node remains");
}

#[test]
fn remove_wins_over_place_at_the_same_instant() {
    let mut tl = ops_timeline();
    tl.replaces.clear();
    tl.removes = vec![RemoveNode { t: 0.5, depth: 1 }];
    let mut p = AnmPlayer::new(tl).unwrap();
    assert_eq!(p.scene_at(0.5).len(), 1, "place then remove leaves a hole");
    assert_eq!(p.scene_at(0.6).len(), 1);
}

#[test]
fn next_keyframe_snapshot_resets_structural_ops() {
    let mut tl = ops_timeline();
    tl.replaces.clear();
    tl.removes.clear();
    tl.tracks.clear();
    tl.keyframes.push(Keyframe {
        t: 1.0,
        snapshot: vec![rect(1, 0, 10.0)],
    });
    let mut p = AnmPlayer::new(tl).unwrap();
    assert_eq!(p.scene_at(0.9).len(), 2);
    assert_eq!(p.scene_at(1.2).len(), 1, "snapshot reset dropped the place");
}

#[test]
fn place_into_an_occupied_depth_overwrites_the_slot() {
    let mut tl = ops_timeline();
    tl.replaces.clear();
    tl.removes.clear();
    tl.places = vec![PlaceNode {
        t: 0.5,
        node: rect(9, 0, 70.0),
    }];
    tl.tracks.clear();
    let mut p = AnmPlayer::new(tl).unwrap();
    assert_eq!(rect_x(&p.scene_at(0.4), 0), 10.0);
    let scene = p.scene_at(0.6);
    assert_eq!(scene.len(), 1, "flat map: one node per depth");
    assert_eq!(rect_x(&scene, 0), 70.0);
}

#[test]
fn playback_with_ops_is_deterministic_and_scrub_consistent() {
    let mut played = AnmPlayer::new(ops_timeline()).unwrap();
    played.play();
    for _ in 0..23 {
        played.tick(&AnimationTick {
            dt: 0.083,
            elapsed: 999.0,
        });
        let mut scrubbed = AnmPlayer::new(ops_timeline()).unwrap();
        scrubbed.scrub(played.current_time());
        assert_eq!(played.scene(), scrubbed.scene());
    }
}

#[test]
fn decoded_ops_file_plays_like_the_authored_timeline() {
    let bytes = crate::write::encode(&ops_timeline(), &[], &[]).unwrap();
    let doc = crate::read::decode(&bytes).unwrap();
    let mut authored = AnmPlayer::new(ops_timeline()).unwrap();
    let mut decoded = AnmPlayer::new(doc.timeline).unwrap();
    for i in 0..=20 {
        let t = i as f32 * 0.1;
        assert_eq!(authored.scene_at(t), decoded.scene_at(t), "at t={t}");
    }
}
