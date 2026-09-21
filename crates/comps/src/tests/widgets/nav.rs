//! nav widgets: tabs, split pane.

use super::*;

// Tabs

#[test]
fn tabs_click_changes_active() {
    let mut tabs = Tabs::new(["Files", "Branches", "History"]);
    let bounds = Rect::new(0.0, 0.0, 400.0, 32.0);
    let rects = tabs.item_rects(bounds, &hoff());
    assert_eq!(rects.len(), 3);

    let (cx, cy) = rects[2].center();
    let r = tabs.handle_event(&down(cx, cy), bounds, &hoff());
    assert!(r.clicked);
    assert_eq!(tabs.active, 2);

    // Clicking the active tab again is not a change.
    let r = tabs.handle_event(&down(cx, cy), bounds, &hoff());
    assert!(!r.clicked);
}

#[test]
fn tabs_segments_share_width_equally() {
    // HOFF tabs are flex-1: equal segments inside the 4px container pad.
    let tabs = Tabs::new(["I", "Considerably longer label"]);
    let bounds = Rect::new(0.0, 0.0, 800.0, 44.0);
    let rects = tabs.item_rects(bounds, &hoff());
    assert_eq!(rects[0].w, rects[1].w);
    assert_eq!(rects[0].x, 4.0);
    assert_eq!(rects[0].h, 36.0, "36px segments in a 44px strip");
    assert_eq!(rects[1].x + rects[1].w, 796.0);
}

// SplitPane

#[test]
fn split_pane_rects_partition_the_bounds() {
    let sp = SplitPane::new(SplitDirection::Horizontal, 0.25, &hoff());
    let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let first = sp.first_rect(bounds);
    let second = sp.second_rect(bounds);
    // 25% of (1000 - 2px divider) = 249.5; the second pane gets the rest.
    assert!((first.w - 249.5).abs() < 0.01);
    assert!((second.w - 748.5).abs() < 0.01);
    assert!((second.x - (first.x + first.w + 2.0)).abs() < 0.01);

    let v = SplitPane::new(SplitDirection::Vertical, 0.5, &hoff());
    let first = v.first_rect(bounds);
    let second = v.second_rect(bounds);
    assert!((first.h - 299.0).abs() < 0.01);
    assert!((second.y - (first.y + first.h + 2.0)).abs() < 0.01);
}

#[test]
fn split_pane_drag_updates_ratio_and_clamps_pane_minimums() {
    let mut sp = SplitPane::new(SplitDirection::Horizontal, 0.5, &hoff());
    let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
    // Drag starts on the divider (x ≈ 499).
    let r = sp.handle_event(&down(499.0, 300.0), bounds);
    assert!(r.changed);
    assert!(sp.is_dragging());
    sp.handle_event(&move_to(750.0, 300.0), bounds);
    assert!((sp.ratio() - 0.751).abs() < 0.01);
    let r = sp.handle_event(&up(750.0, 300.0), bounds);
    assert!(r.changed);
    assert!(!sp.is_dragging());

    // Dragging past the edge clamps the panes at their px minimums, and
    // the ratio survives to expand again. Grab the divider at its CURRENT
    // position (the first drag moved it).
    let dx = sp.divider_rect(bounds).x + 1.0;
    sp.handle_event(&down(dx, 300.0), bounds);
    sp.handle_event(&move_to(-500.0, 300.0), bounds);
    assert_eq!(sp.ratio(), 0.0);
    assert_eq!(sp.first_rect(bounds).w, sp.min_first);
    sp.handle_event(&up(-500.0, 300.0), bounds);
}

#[test]
fn split_pane_hover_and_render_without_gpu() {
    let theme = Theme::hoff();
    let mut sp = SplitPane::new(SplitDirection::Horizontal, 0.5, &hoff());
    let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
    assert!(sp.handle_event(&move_to(499.0, 300.0), bounds).changed);
    assert!(sp.is_hovered());
    let mut c = Compositor::new();
    sp.render(&mut c, bounds, &theme);
}
