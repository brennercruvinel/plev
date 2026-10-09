//! the ring of served revisions: a hit, an eviction, a refresh.

use accord::{Role, Snapshot, SnapshotBuilder};
use socket::{DEFAULT_RING, Ring};

fn snap(text: &str) -> Snapshot {
    let mut b = SnapshotBuilder::new();
    b.node("field", Role::TextInput, "field")
        .state("text", text);
    b.finish().unwrap()
}

#[test]
fn a_served_revision_is_held_until_it_is_the_oldest_past_the_cap() {
    let mut ring = Ring::new(3);
    assert_eq!(ring.cap(), 3);
    assert!(ring.is_empty());
    let a = ring.serve(snap("a"));
    assert_eq!(ring.get(a), Some(&snap("a")));
    let b = ring.serve(snap("b"));
    let c = ring.serve(snap("c"));
    assert_eq!(ring.len(), 3);
    let d = ring.serve(snap("d"));
    assert_eq!(ring.get(a), None, "evicted");
    assert_eq!(ring.revisions().collect::<Vec<_>>(), [b, c, d]);
}

#[test]
fn serving_a_held_revision_again_makes_it_the_newest() {
    let mut ring = Ring::new(2);
    let a = ring.serve(snap("a"));
    let b = ring.serve(snap("b"));
    assert_eq!(ring.serve(snap("a")), a);
    assert_eq!(ring.len(), 2);
    let c = ring.serve(snap("c"));
    assert_eq!(ring.get(b), None, "b was the oldest");
    assert_eq!(ring.revisions().collect::<Vec<_>>(), [a, c]);
}

#[test]
fn a_ring_keeps_at_least_the_last_revision() {
    let mut ring = Ring::new(0);
    assert_eq!(ring.cap(), 1);
    ring.serve(snap("a"));
    let b = ring.serve(snap("b"));
    assert_eq!(ring.revisions().collect::<Vec<_>>(), [b]);
    assert_eq!(Ring::default().cap(), DEFAULT_RING);
}
