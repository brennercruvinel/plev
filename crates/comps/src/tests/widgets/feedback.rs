//! feedback widgets: progress, tooltip, toasts, scrollbar, context menu, modal, spinner.

use super::*;

// ProgressBar

#[test]
fn progress_clamps_value() {
    let mut p = ProgressBar::new(1.7);
    assert_eq!(p.value(), 1.0);
    p.set_value(-2.0);
    assert_eq!(p.value(), 0.0);
}

// Tooltip

#[test]
fn tooltip_waits_out_delay_before_showing() {
    let mut t = Tooltip::new("hint").delay(0.3);
    t.set_hover(true, B);
    assert!(!t.is_visible());
    t.tick(0.1);
    assert!(!t.is_visible());
    t.tick(0.25);
    assert!(t.is_visible());
}

#[test]
fn tooltip_hides_on_hover_out() {
    let mut t = Tooltip::new("hint").delay(0.1);
    t.set_hover(true, B);
    t.tick(0.2);
    assert!(t.is_visible());
    let changed = t.set_hover(false, B);
    assert!(changed);
    assert!(!t.is_visible());
}

#[test]
fn tooltip_placement_prefers_above_and_flips_below() {
    let t = {
        let mut t = Tooltip::new("hint");
        t.set_hover(true, Rect::new(100.0, 300.0, 50.0, 20.0));
        t
    };
    let above = t.placement(&hoff(), 800.0, 600.0);
    assert!(above.y + above.h <= 300.0, "placed above the anchor");

    let t2 = {
        let mut t = Tooltip::new("hint");
        t.set_hover(true, Rect::new(100.0, 2.0, 50.0, 20.0));
        t
    };
    let below = t2.placement(&hoff(), 800.0, 600.0);
    assert!(below.y >= 22.0, "no room above -> flips below");
}

// ToastManager

#[test]
fn toast_queue_caps_visible_count() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.max_visible = 2;
    for i in 0..5 {
        tm.push(format!("toast {i}"), Intent::Neutral, &theme);
    }
    assert_eq!(tm.len(), 5);
    assert_eq!(tm.visible_count(), 2);
}

#[test]
fn toast_auto_dismisses_after_duration() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.duration = 0.5;
    tm.push("bye", Intent::Neutral, &theme);

    // Entry + lifetime + exit: generously tick 4 simulated seconds.
    for _ in 0..240 {
        tm.wake(1.0 / 60.0);
    }
    assert!(tm.is_empty(), "toast should have auto-dismissed");
}

#[test]
fn toast_queue_promotes_waiting_toasts() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.max_visible = 1;
    tm.duration = 0.2;
    tm.push("first", Intent::Neutral, &theme);
    tm.push("second", Intent::Neutral, &theme);
    assert_eq!(tm.visible().next().unwrap().message, "first");

    for _ in 0..240 {
        tm.wake(1.0 / 60.0);
    }
    assert!(tm.is_empty(), "both toasts eventually shown and dismissed");
}

#[test]
fn toast_click_dismisses() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.push("clickme", Intent::Destructive, &theme);
    // Let the entry animation settle so the rect is in place.
    for _ in 0..120 {
        tm.wake(1.0 / 60.0);
    }
    let rect = tm.visible_rects(&theme, 800.0, 600.0)[0];
    let (cx, cy) = rect.center();
    let r = tm.handle_event(&down(cx, cy), &theme, 800.0, 600.0);
    assert!(r.clicked);
    assert!(tm.visible().next().unwrap().is_closing());
}

#[test]
fn toast_render_emits_nodes() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.push("render", Intent::Constructive, &theme);
    for _ in 0..60 {
        tm.wake(1.0 / 60.0);
    }
    let mut c = Compositor::new();
    let layer = c.create_layer(500);
    tm.render(&mut c, layer, &theme, 800.0, 600.0);
}

// Scrollbar

fn scrolled_state() -> engine::input::scroll::ScrollState {
    let mut s = engine::input::scroll::ScrollState::new();
    s.set_viewport(200.0);
    s.set_content(800.0);
    s
}

#[test]
fn scrollbar_thumb_is_proportional() {
    let sb = Scrollbar::new();
    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    let scroll = scrolled_state();
    let thumb = sb.thumb_rect(bounds, &scroll, &hoff());
    // viewport/content = 0.25 -> thumb is a quarter of the track.
    assert!((thumb.h - 50.0).abs() < 1.0, "thumb.h={}", thumb.h);
}

#[test]
fn scrollbar_fades_in_on_scroll_and_out_after_idle() {
    let mut sb = Scrollbar::new();
    assert_eq!(sb.opacity(), 0.0);

    sb.notify_scroll();
    for _ in 0..30 {
        sb.tick(1.0 / 60.0);
    }
    assert!(sb.opacity() > 0.5, "visible after scroll: {}", sb.opacity());

    // Stay idle past the hide delay plus fade time.
    for _ in 0..240 {
        sb.tick(1.0 / 60.0);
    }
    assert!(sb.opacity() < 0.05, "faded out: {}", sb.opacity());
}

#[test]
fn scrollbar_drag_moves_scroll() {
    let mut sb = Scrollbar::new();
    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    let mut scroll = scrolled_state();
    sb.notify_scroll();
    for _ in 0..60 {
        sb.tick(1.0 / 60.0);
    }

    let thumb = sb.thumb_rect(bounds, &scroll, &hoff());
    let (tx, ty) = thumb.center();
    sb.handle_event(&down(tx, ty), bounds, &mut scroll, &hoff());
    assert!(sb.is_dragging());

    sb.handle_event(&move_to(tx, ty + 75.0), bounds, &mut scroll, &hoff());
    assert!(scroll.offset() > 200.0, "offset={}", scroll.offset());

    sb.handle_event(&up(tx, ty + 75.0), bounds, &mut scroll, &hoff());
    assert!(!sb.is_dragging());
}

#[test]
fn scrollbar_inert_when_content_fits() {
    let mut sb = Scrollbar::new();
    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    let mut scroll = engine::input::scroll::ScrollState::new();
    scroll.set_viewport(200.0);
    scroll.set_content(100.0);
    let r = sb.handle_event(&down(295.0, 100.0), bounds, &mut scroll, &hoff());
    assert_eq!(r, EventResult::IGNORED);
}

// ContextMenu

fn menu() -> ContextMenu {
    ContextMenu::new(vec![
        MenuEntry::item(1, "Stage").icon("plus"),
        MenuEntry::item(2, "Unstage"),
        MenuEntry::Separator,
        MenuEntry::item(3, "Discard").intent(Intent::Destructive),
        MenuEntry::item(4, "Locked").disabled(true),
    ])
}

#[test]
fn context_menu_click_reports_item_id() {
    let mut m = menu();
    let (_, h) = m.size(&hoff());
    assert!(h > 0.0);
    // First item row center: PAD_Y(8) + ITEM_H(44)/2.
    let (r, id) = m.handle_event(&down(50.0, 10.0 + 8.0 + 22.0), 10.0, 10.0, &hoff());
    assert!(r.clicked);
    assert_eq!(id, Some(1));
}

#[test]
fn context_menu_disabled_item_swallows_click_without_id() {
    let mut m = menu();
    // Rows: item(44) item(44) sep(9) item(44) item(44); last item center:
    let y = 10.0 + 8.0 + 44.0 + 44.0 + 9.0 + 44.0 + 22.0;
    let (r, id) = m.handle_event(&down(50.0, y), 10.0, 10.0, &hoff());
    assert!(r.handled);
    assert!(!r.clicked);
    assert_eq!(id, None);
}

#[test]
fn context_menu_hover_skips_disabled_and_separators() {
    let mut m = menu();
    let sep_y = 10.0 + 8.0 + 44.0 + 44.0 + 4.0;
    m.handle_event(&move_to(50.0, sep_y), 10.0, 10.0, &hoff());
    assert_eq!(m.hovered(), None);
}

// Modal

#[test]
fn modal_confirm_and_cancel_buttons_resolve_actions() {
    let mut m = Modal::new(
        "Discard changes?",
        "This cannot be undone.",
        "Discard",
        "Cancel",
    )
    .intent(Intent::Destructive);
    let (vw, vh) = (800.0, 600.0);
    let dialog = m.dialog_rect(&hoff(), vw, vh);
    assert!(dialog.w > 0.0 && dialog.h > 0.0);

    // Click outside the dialog: cancels.
    let (action, r) = m.handle_event(&down(1.0, 1.0), &hoff(), vw, vh);
    assert_eq!(action, ModalAction::Cancel);
    assert!(r.clicked);
}

#[test]
fn modal_swallows_events_inside_dialog() {
    let mut m = Modal::new("T", "B", "Ok", "Cancel");
    let (vw, vh) = (800.0, 600.0);
    let dialog = m.dialog_rect(&hoff(), vw, vh);
    let (cx, cy) = dialog.center();
    let (action, r) = m.handle_event(&down(cx, cy), &hoff(), vw, vh);
    assert_eq!(action, ModalAction::None);
    assert!(r.handled, "modal is blocking");
}

#[test]
fn modal_render_emits_to_layer() {
    let m = Modal::new("T", "B", "Ok", "Cancel");
    let theme = Theme::dark();
    let mut c = Compositor::new();
    let layer = c.create_layer(1000);
    m.render(&mut c, layer, &theme, 800.0, 600.0);
}

// Spinner

#[test]
fn spinner_tick_advances_the_angle_and_always_animates() {
    let mut s = Spinner::new();
    assert_eq!(s.angle(), 0.0);
    assert!(s.tick(0.1));
    let a = s.angle();
    assert!(a > 0.0);
    assert!(s.tick(0.1));
    assert!(s.angle() > a);
    // Wraps after a full turn, never grows unbounded.
    for _ in 0..100 {
        s.tick(1.0);
    }
    assert!(s.angle() < std::f32::consts::TAU);
}

/// With reduced motion on (the thread's), the arc stands still and asks
/// for no frame; off again, it turns.
#[test]
fn spinner_stands_still_with_reduced_motion() {
    let mut s = Spinner::new();
    s.tick(0.1);
    let at = s.angle();
    engine::animation::set_reduced_motion(true);
    let moved = s.tick(0.1);
    let still = s.angle();
    engine::animation::set_reduced_motion(false);
    assert!(!moved, "no frame asked");
    assert_eq!(still, at, "the arc stands where it was");
    assert!(s.tick(0.1) && s.angle() > at, "motion again");
}

#[test]
fn spinner_renders_every_size_without_gpu() {
    let theme = Theme::hoff();
    for size in [SpinnerSize::Sm, SpinnerSize::Md, SpinnerSize::Lg] {
        let mut c = Compositor::new();
        let mut s = Spinner::new().size(size);
        s.tick(0.2);
        s.render(&mut c, B, &theme);
    }
}
