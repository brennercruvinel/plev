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
