use winit::event::{ElementState, MouseButton, MouseScrollDelta};

use super::*;

#[test]
fn view_id_equality() {
    assert_eq!(ViewId(0), ViewId(0));
    assert_ne!(ViewId(0), ViewId(1));
}

#[test]
fn next_view_id_increments() {
    let mut input = InputState::new();
    assert_eq!(input.next_view_id(), ViewId(0));
    assert_eq!(input.next_view_id(), ViewId(1));
    assert_eq!(input.next_view_id(), ViewId(2));
}

#[test]
fn begin_frame_resets_ids_and_regions() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);
    assert_eq!(input.hit_regions.len(), 1);

    input.begin_frame();
    assert_eq!(input.hit_regions.len(), 0);
    assert_eq!(input.next_view_id(), ViewId(0));
}

#[test]
fn begin_frame_preserves_pending_events() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);
    input.handle_cursor_moved(50.0, 50.0);

    input.begin_frame();
    let events = input.drain_events();
    assert!(!events.is_empty());
}

#[test]
fn hover_enter_leave() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    assert_eq!(input.hovered_view(), Some(ViewId(0)));

    let events = input.drain_events();
    assert_eq!(events.len(), 1);
    let InputEvent::Hover(h) = &events[0] else {
        panic!("Expected HoverEvent, got {:?}", &events[0]);
    };
    assert_eq!(h.view_id, ViewId(0));
    assert!(h.entered);

    input.handle_cursor_moved(200.0, 200.0);
    assert_eq!(input.hovered_view(), None);

    let events = input.drain_events();
    assert_eq!(events.len(), 1);
    let InputEvent::Hover(h) = &events[0] else {
        panic!("Expected HoverEvent, got {:?}", &events[0]);
    };
    assert_eq!(h.view_id, ViewId(0));
    assert!(!h.entered);
}

#[test]
fn hover_transition_between_views() {
    let mut input = InputState::new();
    let id0 = input.next_view_id();
    let id1 = input.next_view_id();
    input.register_hit_region(id0, 0.0, 0.0, 100.0, 100.0, false);
    input.register_hit_region(id1, 200.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    input.drain_events();

    input.handle_cursor_moved(250.0, 50.0);
    let events = input.drain_events();
    assert_eq!(events.len(), 2);
    let InputEvent::Hover(h) = &events[0] else {
        panic!("Expected HoverLeave, got {:?}", &events[0]);
    };
    assert_eq!(h.view_id, ViewId(0));
    assert!(!h.entered);
    let InputEvent::Hover(h) = &events[1] else {
        panic!("Expected HoverEnter, got {:?}", &events[1]);
    };
    assert_eq!(h.view_id, ViewId(1));
    assert!(h.entered);
}

#[test]
fn cursor_left_generates_hover_leave() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    input.drain_events();

    input.handle_cursor_left();
    assert_eq!(input.hovered_view(), None);
    assert_eq!(input.cursor_position(), None);

    let events = input.drain_events();
    assert_eq!(events.len(), 1);
    let InputEvent::Hover(h) = &events[0] else {
        panic!("Expected HoverLeave, got {:?}", &events[0]);
    };
    assert!(!h.entered);
}

#[test]
fn click_generates_event_and_updates_focus() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, true);

    input.handle_cursor_moved(50.0, 50.0);
    input.drain_events();

    input.handle_mouse_input(MouseButton::Left, ElementState::Pressed);
    assert_eq!(input.focused_view(), Some(ViewId(0)));

    let events = input.drain_events();
    assert_eq!(events.len(), 1);
    let InputEvent::Click(c) = &events[0] else {
        panic!("Expected ClickEvent, got {:?}", &events[0]);
    };
    assert_eq!(c.view_id, ViewId(0));
    assert_eq!(c.button, PointerButton::Primary);
    assert!(matches!(c.state, PressState::Pressed));
}

#[test]
fn click_outside_clears_focus() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, true);

    input.handle_cursor_moved(50.0, 50.0);
    input.handle_mouse_input(MouseButton::Left, ElementState::Pressed);
    assert_eq!(input.focused_view(), Some(ViewId(0)));
    input.drain_events();

    input.handle_cursor_moved(200.0, 200.0);
    input.handle_mouse_input(MouseButton::Left, ElementState::Pressed);
    assert_eq!(input.focused_view(), None);
}

#[test]
fn non_focusable_region_does_not_gain_focus() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    input.handle_mouse_input(MouseButton::Left, ElementState::Pressed);
    assert_eq!(input.focused_view(), None);

    let events = input.drain_events();
    assert!(events.iter().any(|e| matches!(e, InputEvent::Click(_))));
}

#[test]
fn scroll_generates_event() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    input.drain_events();

    input.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, -3.0));
    let events = input.drain_events();
    assert_eq!(events.len(), 1);
    let InputEvent::Scroll(s) = &events[0] else {
        panic!("Expected ScrollEvent, got {:?}", &events[0]);
    };
    assert_eq!(s.view_id, ViewId(0));
    assert_eq!(s.delta_y, -3.0);
}

#[test]
fn scroll_outside_region_ignored() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(200.0, 200.0);
    input.drain_events();

    input.handle_mouse_wheel(MouseScrollDelta::LineDelta(0.0, -1.0));
    assert!(input.drain_events().is_empty());
}

#[test]
fn no_events_without_cursor_position() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, true);

    input.handle_mouse_input(MouseButton::Left, ElementState::Pressed);
    assert!(input.drain_events().is_empty());
}

#[test]
fn drain_clears_queue() {
    let mut input = InputState::new();
    let id = input.next_view_id();
    input.register_hit_region(id, 0.0, 0.0, 100.0, 100.0, false);

    input.handle_cursor_moved(50.0, 50.0);
    assert!(!input.drain_events().is_empty());
    assert!(input.drain_events().is_empty());
}
