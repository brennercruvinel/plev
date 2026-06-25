use super::*;

#[test]
fn hit_test_basic() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 10.0, 10.0, 100.0, 100.0, false);

    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(0)));
    assert_eq!(input.hit_test(5.0, 5.0), None);
    assert_eq!(input.hit_test(110.0, 110.0), Some(ViewId(0)));
    assert_eq!(input.hit_test(111.0, 111.0), None);
}

#[test]
fn hit_test_z_order() {
    let mut input = InputState::new();
    let id0 = input.next_view_id();
    let id1 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, false);
    input.register_hit_region(id1, 50.0, 50.0, 100.0, 100.0, false);

    assert_eq!(input.hit_test(75.0, 75.0), Some(ViewId(1)));
    assert_eq!(input.hit_test(25.0, 25.0), Some(ViewId(0)));
}

#[test]
fn hit_test_skips_invisible_layer() {
    let mut input = InputState::new();
    input.set_current_layer(true, 1.0);
    let id0 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, false);

    input.set_current_layer(false, 1.0);
    let id1 = input.next_view_id();
    input.register_hit_region(id1, 0.0, 0.0, 100.0, 100.0, false);

    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(0)));
}

#[test]
fn hit_test_skips_zero_opacity_layer() {
    let mut input = InputState::new();
    input.set_current_layer(true, 1.0);
    let id0 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, false);

    input.set_current_layer(true, 0.0);
    let id1 = input.next_view_id();
    input.register_hit_region(id1, 0.0, 0.0, 100.0, 100.0, false);

    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(0)));
}

#[test]
fn hit_test_respects_visible_layer_on_top() {
    let mut input = InputState::new();
    input.set_current_layer(true, 1.0);
    let id0 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, false);

    input.set_current_layer(true, 0.5);
    let id1 = input.next_view_id();
    input.register_hit_region(id1, 0.0, 0.0, 100.0, 100.0, false);

    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(1)));
}

#[test]
fn hit_test_focusable_skips_invisible_layer() {
    let mut input = InputState::new();
    input.set_current_layer(true, 1.0);
    let id0 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, true);

    input.set_current_layer(false, 1.0);
    let id1 = input.next_view_id();
    input.register_hit_region(id1, 0.0, 0.0, 100.0, 100.0, true);

    assert_eq!(input.hit_test_focusable(50.0, 50.0), Some(ViewId(0)));
}

#[test]
fn set_current_layer_defaults_visible() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);
    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(0)));
}

#[test]
fn begin_frame_resets_layer_state() {
    let mut input = InputState::new();
    input.set_current_layer(false, 0.0);
    input.begin_frame();

    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);
    assert_eq!(input.hit_test(50.0, 50.0), Some(ViewId(0)));
}
