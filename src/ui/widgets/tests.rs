use super::*;
use crate::compositor::Compositor;
use crate::theme::{Intent, Theme};

const B: Rect = Rect {
    x: 10.0,
    y: 10.0,
    w: 100.0,
    h: 30.0,
};

fn move_to(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseMove { x, y }
}
fn down(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseDown { x, y }
}
fn up(x: f32, y: f32) -> WidgetEvent {
    WidgetEvent::MouseUp { x, y }
}

fn click(mut result: impl FnMut(&WidgetEvent) -> EventResult) -> EventResult {
    let r1 = result(&down(20.0, 20.0));
    let r2 = result(&up(20.0, 20.0));
    r1.merge(r2)
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

#[test]
fn button_click_fires_on_release_inside() {
    let mut b = Button::new("Save");
    assert!(!b.handle_event(&down(20.0, 20.0), B).clicked);
    assert!(b.is_pressed());
    assert!(b.handle_event(&up(20.0, 20.0), B).clicked);
    assert!(!b.is_pressed());
}

#[test]
fn button_press_cancelled_by_release_outside() {
    let mut b = Button::new("Save");
    b.handle_event(&down(20.0, 20.0), B);
    let r = b.handle_event(&up(500.0, 500.0), B);
    assert!(!r.clicked);
    assert!(r.changed, "visual pressed state must clear");
}

#[test]
fn button_disabled_ignores_clicks() {
    let mut b = Button::new("Save").disabled(true);
    let r = click(|e| b.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!b.is_pressed());
}

#[test]
fn button_hover_state_tracks_pointer() {
    let mut b = Button::new("Save");
    assert!(b.handle_event(&move_to(20.0, 20.0), B).changed);
    assert!(b.is_hovered());
    assert!(b.handle_event(&move_to(500.0, 20.0), B).changed);
    assert!(!b.is_hovered());
    // No change -> no redraw request.
    assert_eq!(
        b.handle_event(&move_to(500.0, 20.0), B),
        EventResult::IGNORED
    );
}

#[test]
fn button_preferred_size_scales_with_size_variant() {
    let sm = Button::new("Commit").size(ButtonSize::Sm).preferred_size();
    let lg = Button::new("Commit").size(ButtonSize::Lg).preferred_size();
    assert!(lg.0 > sm.0);
    assert!(lg.1 > sm.1);
    assert_eq!(sm.1, ButtonSize::Sm.height());
}

#[test]
fn button_renders_nodes_for_all_variants() {
    let theme = Theme::dark();
    for variant in [
        ButtonVariant::Solid,
        ButtonVariant::Outline,
        ButtonVariant::Ghost,
        ButtonVariant::Danger,
    ] {
        let b = Button::new("Go")
            .variant(variant)
            .intent(Intent::Destructive);
        let mut c = Compositor::new();
        b.render(&mut c, B, &theme);
        // Smoke check: render must not panic and must emit something.
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

#[test]
fn checkbox_click_toggles_checked() {
    let mut cb = Checkbox::new(false);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(cb.checked);
    let r = click(|e| cb.handle_event(e, B));
    assert!(r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_disabled_ignores_click() {
    let mut cb = Checkbox::new(false).disabled(true);
    let r = click(|e| cb.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!cb.checked);
}

#[test]
fn checkbox_release_outside_does_not_toggle() {
    let mut cb = Checkbox::new(false);
    cb.handle_event(&down(20.0, 20.0), B);
    cb.handle_event(&up(500.0, 500.0), B);
    assert!(!cb.checked);
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

#[test]
fn switch_click_toggles_and_animates_knob() {
    let theme = Theme::dark();
    let mut sw = Switch::new(false).with_motion(&theme.motion);
    assert_eq!(sw.knob_progress(), 0.0);

    let r = click(|e| sw.handle_event(e, B));
    assert!(r.clicked);
    assert!(sw.on);
    assert!(sw.is_animating());

    sw.tick(1.0 / 60.0);
    let mid = sw.knob_progress();
    assert!(mid > 0.0 && mid < 1.0, "knob mid-flight, got {mid}");

    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    assert!(!sw.is_animating());
    assert!((sw.knob_progress() - 1.0).abs() < 1e-3);
}

#[test]
fn switch_disabled_ignores_click() {
    let mut sw = Switch::new(false).disabled(true);
    let r = click(|e| sw.handle_event(e, B));
    assert!(!r.clicked);
    assert!(!sw.on);
    assert!(!sw.is_animating());
}

// ---------------------------------------------------------------------------
// Slider
// ---------------------------------------------------------------------------

#[test]
fn slider_drag_updates_value() {
    let mut s = Slider::new(0.0, 100.0, 0.0);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(100.0, 10.0), bounds);
    assert!(s.is_dragging());
    assert!(s.value() > 25.0 && s.value() < 75.0, "value={}", s.value());

    s.handle_event(&move_to(1000.0, 10.0), bounds);
    assert_eq!(s.value(), 100.0, "drag clamps at max");
    s.handle_event(&up(1000.0, 10.0), bounds);
    assert!(!s.is_dragging());
}

#[test]
fn slider_step_snaps_value() {
    let mut s = Slider::new(0.0, 10.0, 0.0).step(2.0);
    s.set_value(3.2);
    assert_eq!(s.value(), 4.0);
}

#[test]
fn slider_disabled_ignores_drag() {
    let mut s = Slider::new(0.0, 100.0, 50.0).disabled(true);
    let bounds = Rect::new(0.0, 0.0, 200.0, 20.0);
    s.handle_event(&down(180.0, 10.0), bounds);
    assert_eq!(s.value(), 50.0);
}

// ---------------------------------------------------------------------------
// ProgressBar
// ---------------------------------------------------------------------------

#[test]
fn progress_clamps_value() {
    let mut p = ProgressBar::new(1.7);
    assert_eq!(p.value(), 1.0);
    p.set_value(-2.0);
    assert_eq!(p.value(), 0.0);
}

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

#[test]
fn tabs_click_changes_active() {
    let mut tabs = Tabs::new(["Files", "Branches", "History"]);
    let bounds = Rect::new(0.0, 0.0, 400.0, 32.0);
    let rects = tabs.item_rects(bounds);
    assert_eq!(rects.len(), 3);

    let (cx, cy) = rects[2].center();
    let r = tabs.handle_event(&down(cx, cy), bounds);
    assert!(r.clicked);
    assert_eq!(tabs.active, 2);

    // Clicking the active tab again is not a change.
    let r = tabs.handle_event(&down(cx, cy), bounds);
    assert!(!r.clicked);
}

#[test]
fn tabs_widths_follow_label_length() {
    let tabs = Tabs::new(["I", "Considerably longer label"]);
    let rects = tabs.item_rects(Rect::new(0.0, 0.0, 800.0, 32.0));
    assert!(rects[1].w > rects[0].w * 2.0);
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

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
    let above = t.placement(800.0, 600.0);
    assert!(above.y + above.h <= 300.0, "placed above the anchor");

    let t2 = {
        let mut t = Tooltip::new("hint");
        t.set_hover(true, Rect::new(100.0, 2.0, 50.0, 20.0));
        t
    };
    let below = t2.placement(800.0, 600.0);
    assert!(below.y >= 22.0, "no room above -> flips below");
}

// ---------------------------------------------------------------------------
// ToastManager
// ---------------------------------------------------------------------------

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
        tm.tick(1.0 / 60.0);
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
        tm.tick(1.0 / 60.0);
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
        tm.tick(1.0 / 60.0);
    }
    let rect = tm.visible_rects(800.0, 600.0)[0];
    let (cx, cy) = rect.center();
    let r = tm.handle_event(&down(cx, cy), 800.0, 600.0);
    assert!(r.clicked);
    assert!(tm.visible().next().unwrap().is_closing());
}

#[test]
fn toast_render_emits_nodes() {
    let theme = Theme::dark();
    let mut tm = ToastManager::new();
    tm.push("render", Intent::Constructive, &theme);
    for _ in 0..60 {
        tm.tick(1.0 / 60.0);
    }
    let mut c = Compositor::new();
    let layer = c.create_layer(500);
    tm.render(&mut c, layer, &theme, 800.0, 600.0);
}

// ---------------------------------------------------------------------------
// Scrollbar
// ---------------------------------------------------------------------------

fn scrolled_state() -> crate::scroll::ScrollState {
    let mut s = crate::scroll::ScrollState::new();
    s.set_viewport(200.0);
    s.set_content(800.0);
    s
}

#[test]
fn scrollbar_thumb_is_proportional() {
    let sb = Scrollbar::new();
    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    let scroll = scrolled_state();
    let thumb = sb.thumb_rect(bounds, &scroll);
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

    let thumb = sb.thumb_rect(bounds, &scroll);
    let (tx, ty) = thumb.center();
    sb.handle_event(&down(tx, ty), bounds, &mut scroll);
    assert!(sb.is_dragging());

    sb.handle_event(&move_to(tx, ty + 75.0), bounds, &mut scroll);
    assert!(scroll.offset() > 200.0, "offset={}", scroll.offset());

    sb.handle_event(&up(tx, ty + 75.0), bounds, &mut scroll);
    assert!(!sb.is_dragging());
}

#[test]
fn scrollbar_inert_when_content_fits() {
    let mut sb = Scrollbar::new();
    let bounds = Rect::new(0.0, 0.0, 300.0, 200.0);
    let mut scroll = crate::scroll::ScrollState::new();
    scroll.set_viewport(200.0);
    scroll.set_content(100.0);
    let r = sb.handle_event(&down(295.0, 100.0), bounds, &mut scroll);
    assert_eq!(r, EventResult::IGNORED);
}

// ---------------------------------------------------------------------------
// ContextMenu
// ---------------------------------------------------------------------------

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
    let (_, h) = m.size();
    assert!(h > 0.0);
    // First item row center: PAD_Y(5) + ITEM_H/2.
    let (r, id) = m.handle_event(&down(50.0, 10.0 + 5.0 + 14.0), 10.0, 10.0);
    assert!(r.clicked);
    assert_eq!(id, Some(1));
}

#[test]
fn context_menu_disabled_item_swallows_click_without_id() {
    let mut m = menu();
    // Rows: item(28) item(28) sep(9) item(28) item(28); last item center:
    let y = 10.0 + 5.0 + 28.0 + 28.0 + 9.0 + 28.0 + 14.0;
    let (r, id) = m.handle_event(&down(50.0, y), 10.0, 10.0);
    assert!(r.handled);
    assert!(!r.clicked);
    assert_eq!(id, None);
}

#[test]
fn context_menu_hover_skips_disabled_and_separators() {
    let mut m = menu();
    let sep_y = 10.0 + 5.0 + 28.0 + 28.0 + 4.0;
    m.handle_event(&move_to(50.0, sep_y), 10.0, 10.0);
    assert_eq!(m.hovered(), None);
}

// ---------------------------------------------------------------------------
// Modal
// ---------------------------------------------------------------------------

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
    let dialog = m.dialog_rect(vw, vh);
    assert!(dialog.w > 0.0 && dialog.h > 0.0);

    // Click outside the dialog: cancels.
    let (action, r) = m.handle_event(&down(1.0, 1.0), vw, vh);
    assert_eq!(action, ModalAction::Cancel);
    assert!(r.clicked);
}

#[test]
fn modal_swallows_events_inside_dialog() {
    let mut m = Modal::new("T", "B", "Ok", "Cancel");
    let (vw, vh) = (800.0, 600.0);
    let dialog = m.dialog_rect(vw, vh);
    let (cx, cy) = dialog.center();
    let (action, r) = m.handle_event(&down(cx, cy), vw, vh);
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

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

#[test]
fn select_opens_and_picks_option() {
    let mut s = Select::new(["dark", "light", "dracula"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);

    s.handle_event(&down(50.0, 20.0), bounds);
    assert!(s.is_open());

    // Option 1 ("light"): dropdown starts at bounds bottom + gap.
    let dd = s.dropdown_rect(bounds);
    let y = dd.y + 5.0 + 28.0 + 14.0;
    let r = s.handle_event(&down(50.0, y), bounds);
    assert!(r.clicked);
    assert!(!s.is_open());
    assert_eq!(s.selected, 1);
    assert_eq!(s.selected_label(), Some("light"));
}

#[test]
fn select_click_outside_closes_without_change() {
    let mut s = Select::new(["a", "b"], 0);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds);
    assert!(s.is_open());
    let r = s.handle_event(&down(700.0, 500.0), bounds);
    assert!(!s.is_open());
    assert!(!r.clicked);
    assert_eq!(s.selected, 0);
}

#[test]
fn select_disabled_never_opens() {
    let mut s = Select::new(["a"], 0).disabled(true);
    let bounds = Rect::new(10.0, 10.0, 160.0, 30.0);
    s.handle_event(&down(50.0, 20.0), bounds);
    assert!(!s.is_open());
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

fn sample_tree() -> Tree {
    Tree::new(vec![
        TreeNode::branch(
            1,
            "src",
            vec![
                TreeNode::leaf(2, "main.rs"),
                TreeNode::branch(3, "ui", vec![TreeNode::leaf(4, "mod.rs")]),
            ],
        )
        .expanded(true),
        TreeNode::leaf(5, "Cargo.toml"),
    ])
}

#[test]
fn tree_flattens_only_expanded_branches() {
    let tree = sample_tree();
    let rows = tree.visible_rows();
    // src (expanded) -> main.rs, ui (collapsed); Cargo.toml.
    let ids: Vec<u64> = rows.iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 5]);
    assert_eq!(rows[1].depth, 1);
    assert_eq!(rows[3].depth, 0);
}

#[test]
fn tree_click_branch_toggles_expansion() {
    let mut tree = sample_tree();
    let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);
    // Row index 2 is the collapsed "ui" branch.
    let y = 2.0 * tree.row_height() + tree.row_height() / 2.0;
    let r = tree.handle_event(&down(50.0, y), bounds);
    assert!(r.clicked);
    let ids: Vec<u64> = tree.visible_rows().iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 5], "ui expanded, mod.rs visible");

    // Collapse the root: only roots remain.
    let r = tree.handle_event(&down(50.0, tree.row_height() / 2.0), bounds);
    assert!(r.clicked);
    let ids: Vec<u64> = tree.visible_rows().iter().map(|r| r.id).collect();
    assert_eq!(ids, vec![1, 5]);
}

#[test]
fn tree_click_leaf_selects() {
    let mut tree = sample_tree();
    let bounds = Rect::new(0.0, 0.0, 300.0, 400.0);
    let y = 1.0 * tree.row_height() + tree.row_height() / 2.0;
    let r = tree.handle_event(&down(50.0, y), bounds);
    assert!(r.clicked);
    assert_eq!(tree.selected, Some(2));
}

// ---------------------------------------------------------------------------
// VirtualList
// ---------------------------------------------------------------------------

#[test]
fn virtual_list_visible_range_at_top() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    let range = list.visible_range();
    assert_eq!(range.start, 0);
    // 10 rows fit; +1 partial +2 overscan.
    assert!(range.end >= 11 && range.end <= 13, "end={}", range.end);
}

#[test]
fn virtual_list_visible_range_mid_scroll() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    list.scroll.scroll_to(2400.0); // exactly 100 rows down
    let range = list.visible_range();
    assert!(
        range.start <= 100 && range.start >= 98,
        "start={}",
        range.start
    );
    assert!(range.contains(&100));
    assert!(range.contains(&109));
    assert!(range.end <= 113, "end={}", range.end);
}

#[test]
fn virtual_list_visible_range_clamps_at_bottom() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(100);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    list.scroll.scroll_to(f32::MAX);
    let range = list.visible_range();
    assert_eq!(range.end, 100);
    assert!(range.start < 100);
}

#[test]
fn virtual_list_renders_only_visible_rows() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(10_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    let theme = Theme::dark();
    let mut c = Compositor::new();
    let mut rendered = Vec::new();
    list.render_with(&mut c, bounds, &theme, |_, index, _, _, _| {
        rendered.push(index);
    });
    assert!(!rendered.is_empty());
    assert!(rendered.len() < 20, "rendered {} of 10000", rendered.len());
}

#[test]
fn virtual_list_scroll_event_moves_and_wakes_scrollbar() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(1_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    let r = list.handle_event(
        &WidgetEvent::Scroll {
            x: 100.0,
            y: 100.0,
            delta: 48.0,
        },
        bounds,
    );
    assert!(r.changed);
    assert_eq!(list.scroll.offset(), 48.0);
    assert!(list.scrollbar.opacity() > 0.0 || list.scrollbar.is_animating());
    assert_eq!(list.visible_range().start, 0);
}

#[test]
fn virtual_list_click_selects_item() {
    let mut list = VirtualList::new(24.0);
    list.set_item_count(1_000);
    let bounds = Rect::new(0.0, 0.0, 300.0, 240.0);
    list.handle_event(&down(100.0, 50.0), bounds);
    assert_eq!(list.selected, Some(2)); // y=50 / 24 = row 2
}

#[test]
fn virtual_list_empty_has_empty_range() {
    let mut list = VirtualList::new(24.0);
    list.set_viewport(Rect::new(0.0, 0.0, 300.0, 240.0));
    assert_eq!(list.visible_range(), 0..0);
}
