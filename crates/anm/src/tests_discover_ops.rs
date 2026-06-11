//! encoder mode B tests, structural side: an intermittent node is
//! discovered as place/remove, a slot changing hands as replace, an id
//! moving depth as remove plus place, and a re-placed id is pinned
//! against the held tail of its dead chain. motion discovery, codec
//! round-trip and the helpers live in tests_discover.

use crate::discover::discover;
use crate::ir::{Node, PlaceNode, RemoveNode, ReplaceNode};
use crate::play::AnmPlayer;
use crate::tests_discover::{assert_reproduced, cfg, seg};
use crate::tests_write::rect;
use plev::compositor::SceneNode;

fn rect_x(scene: &[SceneNode], at: usize) -> f32 {
    match &scene[at] {
        SceneNode::Rect { x, .. } => *x,
        other => panic!("expected rect, got {other:?}"),
    }
}

#[test]
fn intermittent_node_becomes_place_and_remove() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let mut scene = vec![rect(1, 0, 10.0)];
            if (2..=5).contains(&i) {
                scene.push(rect(2, 1, 40.0));
            }
            (t, scene)
        })
        .collect();
    let tl = discover(&frames, &cfg(100.0, 8.0)).unwrap();
    assert_eq!(tl.keyframes.len(), 1);
    assert_eq!(
        tl.places,
        vec![PlaceNode {
            t: 0.5,
            node: rect(2, 1, 40.0)
        }]
    );
    assert_eq!(tl.removes, vec![RemoveNode { t: 1.5, depth: 1 }]);
    assert!(tl.replaces.is_empty() && tl.tracks.is_empty());
    assert_reproduced(&tl, &frames);
    let mut player = AnmPlayer::new(tl).unwrap();
    assert_eq!(player.scene_at(0.25).len(), 1);
    assert_eq!(player.scene_at(0.5).len(), 2);
    assert_eq!(player.scene_at(1.25).len(), 2);
    assert_eq!(player.scene_at(1.5).len(), 1);
}

#[test]
fn slot_handover_becomes_replace() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let node = if i < 4 { rect(2, 1, 20.0) } else { rect(3, 1, 30.0) };
            (t, vec![node])
        })
        .collect();
    let tl = discover(&frames, &cfg(100.0, 8.0)).unwrap();
    assert_eq!(
        tl.replaces,
        vec![ReplaceNode {
            t: 1.0,
            depth: 1,
            node: rect(3, 1, 30.0)
        }]
    );
    assert!(tl.places.is_empty() && tl.removes.is_empty());
    assert_reproduced(&tl, &frames);
}

#[test]
fn depth_move_is_remove_plus_place() {
    let frames: Vec<(f32, Vec<Node>)> = (0..=8)
        .map(|i| {
            let t = i as f32 * 0.25;
            let depth = if i < 4 { 1 } else { 2 };
            (t, vec![rect(2, depth, 20.0)])
        })
        .collect();
    let tl = discover(&frames, &cfg(100.0, 8.0)).unwrap();
    assert_eq!(tl.removes, vec![RemoveNode { t: 1.0, depth: 1 }]);
    assert_eq!(
        tl.places,
        vec![PlaceNode {
            t: 1.0,
            node: rect(2, 2, 20.0)
        }]
    );
    assert!(tl.replaces.is_empty());
    assert_reproduced(&tl, &frames);
}

#[test]
fn replaced_life_is_pinned_against_its_dead_chain() {
    // id 2 lives twice in one window: first life animates x to 40 and
    // dies; the second life re-places it at a constant 60. the dead
    // chain holds 40 forever, so the second life needs a pin track.
    let frames: Vec<(f32, Vec<Node>)> = (0..=8)
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
        .collect();
    let tl = discover(&frames, &cfg(100.0, 100.0)).unwrap();
    assert_eq!(tl.keyframes.len(), 1);
    assert_eq!(tl.places.len(), 2);
    assert_eq!(tl.removes, vec![RemoveNode { t: 1.0, depth: 1 }]);
    assert_eq!(tl.tracks.len(), 2);
    assert_eq!(tl.tracks[0].segments, vec![seg(40.0, 0.5)]);
    assert_eq!(
        (tl.tracks[1].start_t, tl.tracks[1].segments.clone()),
        (1.5, vec![seg(60.0, 0.25)]),
        "second life pins x against the held 40"
    );
    assert_reproduced(&tl, &frames);
    let mut player = AnmPlayer::new(tl).unwrap();
    for t in [1.5, 1.75, 2.0] {
        assert_eq!(rect_x(&player.scene_at(t), 1), 60.0, "no bleed at t={t}");
    }
}
