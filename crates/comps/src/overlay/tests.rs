use super::*;

fn ctx_menu(labels: &[(&str, u64)]) -> OverlayKind {
    OverlayKind::ContextMenu {
        items: labels
            .iter()
            .map(|(l, id)| MenuItem::new(*l, *id))
            .collect(),
    }
}

#[test]
fn new_manager_is_empty() {
    let mgr = OverlayManager::new();
    assert!(mgr.is_empty());
    assert_eq!(mgr.len(), 0);
}

#[test]
fn push_increases_len() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[("Stage", 1)]), 10.0, 20.0, 120.0, 80.0);
    assert_eq!(mgr.len(), 1);
}

#[test]
fn push_assigns_base_z_to_first_overlay() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    assert_eq!(mgr.top().unwrap().z_order, OverlayManager::BASE_Z);
}

#[test]
fn z_order_is_monotonically_increasing() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.push(ctx_menu(&[]), 10.0, 10.0, 100.0, 50.0);
    mgr.push(ctx_menu(&[]), 20.0, 20.0, 100.0, 50.0);

    let zs: Vec<i32> = mgr.stack.iter().map(|o| o.z_order).collect();
    assert!(zs[0] < zs[1] && zs[1] < zs[2]);
}

#[test]
fn pop_removes_topmost() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.pop();
    assert_eq!(mgr.len(), 1);
}

#[test]
fn pop_returns_id_of_removed() {
    let mut mgr = OverlayManager::new();
    let _id1 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let id2 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let removed = mgr.pop();
    assert_eq!(removed, Some(id2));
}

#[test]
fn pop_on_empty_returns_none() {
    let mut mgr = OverlayManager::new();
    assert_eq!(mgr.pop(), None);
}

#[test]
fn pop_id_invalid_is_noop() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.pop_id(OverlayId(9999));
    assert_eq!(mgr.len(), 1);
}

#[test]
fn pop_id_removes_correct_entry() {
    let mut mgr = OverlayManager::new();
    let id1 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let _id2 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.pop_id(id1);
    assert_eq!(mgr.len(), 1);
    assert!(mgr.stack.iter().all(|o| o.id != id1));
}

#[test]
fn pop_id_reassigns_z_orders() {
    let mut mgr = OverlayManager::new();
    let id1 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let _id2 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let _id3 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.pop_id(id1);
    let zs: Vec<i32> = mgr.stack.iter().map(|o| o.z_order).collect();
    assert_eq!(zs[0], OverlayManager::BASE_Z);
    assert_eq!(zs[1], OverlayManager::BASE_Z + 1);
}

#[test]
fn pop_all_empties_stack() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.pop_all();
    assert!(mgr.is_empty());
}

#[test]
fn hit_test_point_inside_returns_false() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 10.0, 20.0, 100.0, 80.0);
    assert!(!mgr.hit_test_outside(50.0, 60.0));
}

#[test]
fn hit_test_point_outside_returns_true() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 10.0, 20.0, 100.0, 80.0);
    assert!(mgr.hit_test_outside(5.0, 5.0));
}

#[test]
fn hit_test_empty_stack_always_true() {
    let mgr = OverlayManager::new();
    assert!(mgr.hit_test_outside(100.0, 100.0));
}

#[test]
fn hit_test_zero_bounds_skipped() {
    let mut mgr = OverlayManager::new();
    // bounds unknown -- should not count as a hit
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 0.0, 0.0);
    assert!(mgr.hit_test_outside(0.0, 0.0));
}

#[test]
fn set_bounds_updates_overlay() {
    let mut mgr = OverlayManager::new();
    let id = mgr.push(ctx_menu(&[]), 10.0, 10.0, 0.0, 0.0);
    mgr.set_bounds(id, 120.0, 90.0);
    let o = mgr.top().unwrap();
    assert_eq!(o.w, 120.0);
    assert_eq!(o.h, 90.0);
    // Now point inside is a hit
    assert!(!mgr.hit_test_outside(50.0, 50.0));
}

#[test]
fn ids_are_unique_across_pushes() {
    let mut mgr = OverlayManager::new();
    let id1 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let id2 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    assert_ne!(id1, id2);
}

// ---------------------------------------------------------------------------
// Animated overlays (push_animated / pop_animated / tick)
// ---------------------------------------------------------------------------

fn neutral_motion() -> crate::theme::MotionPhysics {
    crate::theme::MotionPhysics {
        mass: 1.0,
        stiffness: 170.0,
        damping: 26.0,
    }
}

#[test]
fn push_animated_starts_hidden_and_animating() {
    let mut mgr = OverlayManager::new();
    mgr.push_animated(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0, &neutral_motion());
    let o = mgr.top().unwrap();
    assert!(o.progress() < 0.05, "progress={}", o.progress());
    assert!((o.scale() - 0.96).abs() < 0.01);
    assert!(mgr.is_animating());
}

#[test]
fn plain_push_is_fully_shown_and_static() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    let o = mgr.top().unwrap();
    assert_eq!(o.progress(), 1.0);
    assert_eq!(o.opacity(), 1.0);
    assert_eq!(o.scale(), 1.0);
    assert!(!mgr.is_animating());
}

#[test]
fn entry_animation_progresses_and_settles_at_one() {
    let mut mgr = OverlayManager::new();
    mgr.push_animated(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0, &neutral_motion());

    mgr.tick(1.0 / 60.0);
    let early = mgr.top().unwrap().progress();
    assert!(early > 0.0 && early < 1.0, "early={early}");

    for _ in 0..300 {
        mgr.tick(1.0 / 60.0);
    }
    let o = mgr.top().unwrap();
    assert!(!mgr.is_animating());
    assert!((o.progress() - 1.0).abs() < 1e-3);
    assert!((o.scale() - 1.0).abs() < 1e-3);
    assert_eq!(mgr.len(), 1, "entry animation must not remove the overlay");
}

#[test]
fn pop_animated_marks_closing_and_removes_when_settled() {
    let mut mgr = OverlayManager::new();
    let id = mgr.push_animated(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0, &neutral_motion());
    for _ in 0..300 {
        mgr.tick(1.0 / 60.0);
    }

    let popped = mgr.pop_animated();
    assert_eq!(popped, Some(id));
    assert_eq!(mgr.len(), 1, "closing overlay stays until settled");
    assert!(mgr.top().unwrap().is_closing());

    for _ in 0..300 {
        mgr.tick(1.0 / 60.0);
    }
    assert!(mgr.is_empty(), "overlay removed after exit animation");
    assert!(!mgr.is_animating());
}

#[test]
fn pop_animated_works_for_overlays_pushed_without_animation() {
    let mut mgr = OverlayManager::new();
    mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    assert!(mgr.pop_animated().is_some());
    assert!(mgr.is_animating());
    for _ in 0..600 {
        mgr.tick(1.0 / 60.0);
    }
    assert!(mgr.is_empty());
}

#[test]
fn pop_animated_twice_closes_two_distinct_overlays() {
    let mut mgr = OverlayManager::new();
    let id1 = mgr.push_animated(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0, &neutral_motion());
    let id2 = mgr.push_animated(ctx_menu(&[]), 10.0, 10.0, 100.0, 50.0, &neutral_motion());
    assert_eq!(mgr.pop_animated(), Some(id2));
    assert_eq!(mgr.pop_animated(), Some(id1));
    assert_eq!(mgr.pop_animated(), None, "everything already closing");
    for _ in 0..600 {
        mgr.tick(1.0 / 60.0);
    }
    assert!(mgr.is_empty());
}

#[test]
fn top_active_skips_closing_overlays() {
    let mut mgr = OverlayManager::new();
    let id1 = mgr.push(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0);
    mgr.push(ctx_menu(&[]), 10.0, 10.0, 100.0, 50.0);
    mgr.pop_animated();
    assert_eq!(mgr.top_active().unwrap().id, id1);
}

#[test]
fn destructive_intent_exit_is_faster_than_informational() {
    use crate::theme::{Intent, Theme};
    let theme = Theme::dark();

    let mut fast = OverlayManager::new();
    fast.push_animated(
        ctx_menu(&[]),
        0.0,
        0.0,
        100.0,
        50.0,
        &theme.intent_motion(Intent::Destructive),
    );
    let mut slow = OverlayManager::new();
    slow.push_animated(
        ctx_menu(&[]),
        0.0,
        0.0,
        100.0,
        50.0,
        &theme.intent_motion(Intent::Informational),
    );

    // Same wall-clock time: destructive (snappier physics) gets further.
    for _ in 0..6 {
        fast.tick(1.0 / 60.0);
        slow.tick(1.0 / 60.0);
    }
    let fp = fast.top().unwrap().progress();
    let sp = slow.top().unwrap().progress();
    assert!(
        fp > sp,
        "destructive {fp} should outpace informational {sp}"
    );
}

#[test]
fn tick_returns_false_once_everything_settled() {
    let mut mgr = OverlayManager::new();
    mgr.push_animated(ctx_menu(&[]), 0.0, 0.0, 100.0, 50.0, &neutral_motion());
    let mut last = true;
    for _ in 0..300 {
        last = mgr.tick(1.0 / 60.0);
    }
    assert!(!last);
}
