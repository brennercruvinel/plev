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
fn tabs_segments_share_width_equally() {
    // HOFF tabs are flex-1: equal segments inside the 4px container pad.
    let tabs = Tabs::new(["I", "Considerably longer label"]);
    let bounds = Rect::new(0.0, 0.0, 800.0, 44.0);
    let rects = tabs.item_rects(bounds);
    assert_eq!(rects[0].w, rects[1].w);
    assert_eq!(rects[0].x, 4.0);
    assert_eq!(rects[0].h, 36.0, "36px segments in a 44px strip");
    assert_eq!(rects[1].x + rects[1].w, 796.0);
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

fn scrolled_state() -> crate::input::scroll::ScrollState {
    let mut s = crate::input::scroll::ScrollState::new();
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
    let mut scroll = crate::input::scroll::ScrollState::new();
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
    // First item row center: PAD_Y(8) + ITEM_H(44)/2.
    let (r, id) = m.handle_event(&down(50.0, 10.0 + 8.0 + 22.0), 10.0, 10.0);
    assert!(r.clicked);
    assert_eq!(id, Some(1));
}

#[test]
fn context_menu_disabled_item_swallows_click_without_id() {
    let mut m = menu();
    // Rows: item(44) item(44) sep(9) item(44) item(44); last item center:
    let y = 10.0 + 8.0 + 44.0 + 44.0 + 9.0 + 44.0 + 22.0;
    let (r, id) = m.handle_event(&down(50.0, y), 10.0, 10.0);
    assert!(r.handled);
    assert!(!r.clicked);
    assert_eq!(id, None);
}

#[test]
fn context_menu_hover_skips_disabled_and_separators() {
    let mut m = menu();
    let sep_y = 10.0 + 8.0 + 44.0 + 44.0 + 4.0;
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

    // Option 1 ("light"): dropdown starts at bounds bottom + gap;
    // PAD_Y(8) + OPTION_H(44) + 22 centers the second option.
    let dd = s.dropdown_rect(bounds);
    let y = dd.y + 8.0 + 44.0 + 22.0;
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

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

use crate::compositor::{LayerId, SceneNode};

fn card_nodes(card: &Card, theme: &Theme) -> Vec<SceneNode> {
    let mut c = Compositor::new();
    c.begin_frame();
    let (w, h) = card.preferred_size();
    card.render(&mut c, Rect::new(0.0, 0.0, w, h), theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

fn sample_cards() -> Vec<Card> {
    vec![
        Card::new(CardVariant::Stat {
            value: "1,632".into(),
            label: "Clicks".into(),
            delta: Some(("+12.4%".into(), true)),
        }),
        Card::new(CardVariant::Profile {
            name: "Artur".into(),
            username: "@artur".into(),
            bio: "Designs dark glass.".into(),
            action: "Follow".into(),
            online: true,
            avatar: None,
        }),
        Card::new(CardVariant::Media {
            title: "4K Video Streaming".into(),
            caption: "Buffer-free playback".into(),
            badge: Some("4K".into()),
            image: None,
        }),
        Card::new(CardVariant::List {
            title: "Expense Tracker".into(),
            rows: vec![
                CardListRow::new("Starter", "$88.00"),
                CardListRow::new("Pro", "$128.00").active(true),
                CardListRow::new("Sync", "70%").progress(0.7),
            ],
        }),
        Card::new(CardVariant::Chart {
            value: "$408.36".into(),
            label: "Last month".into(),
            groups: vec![(0.4, 0.6), (0.5, 0.85), (0.3, 0.5), (0.6, 0.76)],
            highlight: 1,
        }),
        Card::new(CardVariant::Cta {
            title: "Detailed Analytics".into(),
            body: "Track every click with the comparative toolkit.".into(),
            button: "Discover".into(),
        }),
    ]
}

#[test]
fn card_default_width_is_hoff_368() {
    for card in sample_cards() {
        assert_eq!(card.preferred_size().0, 368.0);
        assert!(card.preferred_size().1 > 0.0);
    }
}

#[test]
fn card_deck_shell_is_the_discreet_post_lift() {
    let theme = Theme::hoff();
    let card = &sample_cards()[0]; // Stat: deck shell.
    let nodes = card_nodes(card, &theme);

    // No frost, no drop shadow: the live post card measures
    // `backdrop-filter:none` and `box-shadow:none`. The deck never paints a
    // backdrop blur or an outset (non-inset) shadow.
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::BackdropBlur { .. })),
        "deck shell does not frost (content cards aren't real glass)"
    );
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Shadow { inset: false, .. })),
        "deck shell casts no drop shadow"
    );
    // Edge-light underlay first: a soft white gradient fading downward.
    assert!(
        matches!(nodes[0], SceneNode::GradientRect { color, color2, angle_deg, .. }
            if color[3] > 0.0 && color2[3] == 0.0 && angle_deg == 180.0),
        "edge-light underlay fades to transparent"
    );
    // Surface: the .02 white post-card lift at radius 20 (inset 1.0 → 19.0),
    // translucent so the #303030 page reads through as ~#343434.
    match nodes[1] {
        SceneNode::RoundedRect {
            color,
            corner_radius,
            ..
        } => {
            // White (#f8f8f8) at .02 — a whisper lighter than the page.
            assert!((color[0] - 248.0 / 255.0).abs() < 1e-5);
            assert!((color[3] - 0.02).abs() < 1e-5);
            assert_eq!(corner_radius, theme.radius.lg - 1.0);
        }
        ref other => panic!("expected surface RoundedRect, got {other:?}"),
    }
    // Inset key-light closes the stack.
    assert!(
        nodes
            .iter()
            .any(|n| matches!(n, SceneNode::Shadow { inset: true, color, .. }
                if (color[3] - 0.06).abs() < 1e-5)),
        "deck shell carries the inset key-light glint"
    );
}

#[test]
fn card_profile_uses_post_surface_and_hover() {
    let theme = Theme::hoff();
    let mut card = sample_cards().remove(1);
    let (w, h) = card.preferred_size();
    let bounds = Rect::new(0.0, 0.0, w, h);

    let nodes = card_nodes(&card, &theme);
    // Same discreet post shell as the deck: no frost, no drop shadow, a .02
    // white surface (edge-light underlay first, surface second).
    assert!(!matches!(nodes[0], SceneNode::Shadow { inset: false, .. }));
    assert!(
        !nodes
            .iter()
            .any(|n| matches!(n, SceneNode::BackdropBlur { .. })),
        "post card does not frost its backdrop"
    );
    assert!(matches!(nodes[1], SceneNode::RoundedRect { color, .. }
        if (color[3] - 0.02).abs() < 1e-5));

    card.handle_event(&move_to(10.0, 10.0), bounds);
    assert!(card.is_hovered());
    let nodes = card_nodes(&card, &theme);
    assert!(
        matches!(nodes[1], SceneNode::RoundedRect { color, .. }
            if (color[3] - 0.05).abs() < 1e-5),
        "hover raises the surface to .05"
    );
}

#[test]
fn card_chart_highlight_is_gradient_with_cap() {
    let theme = Theme::hoff();
    let card = &sample_cards()[4];
    let nodes = card_nodes(card, &theme);

    let gradients: Vec<_> = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::GradientRect { angle_deg, color, .. }
                if *angle_deg == 180.0 && (color[3] - 0.20).abs() < 1e-5)
        })
        .collect();
    assert_eq!(gradients.len(), 1, "exactly one highlighted bar");

    // The floating 2px cap at 50% white.
    assert!(nodes.iter().any(|n| matches!(n,
        SceneNode::RoundedRect { h, color, .. } if *h == 2.0 && (color[3] - 0.50).abs() < 1e-5)));

    // 4 groups x 2 bars: 7 plain bars + 1 gradient highlight.
    let plain_bars = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { corner_radius, h, .. }
                if *corner_radius == 2.0 && *h > 2.0)
        })
        .count();
    assert_eq!(plain_bars, 7);
}

#[test]
fn card_list_marks_active_row_with_strong_border() {
    let theme = Theme::hoff();
    let card = &sample_cards()[3];
    let nodes = card_nodes(card, &theme);

    let strong_rows = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { h, border_color, .. }
                if *h == 58.0 && (border_color[3] - 0.40).abs() < 1e-5)
        })
        .count();
    let soft_rows = nodes
        .iter()
        .filter(|n| {
            matches!(n, SceneNode::RoundedRect { h, border_color, .. }
                if *h == 58.0 && (border_color[3] - 0.05).abs() < 1e-5)
        })
        .count();
    assert_eq!(strong_rows, 1, "one active row");
    assert_eq!(soft_rows, 2, "two resting rows");

    // The progress row carries the 90deg gradient fill.
    assert!(nodes.iter().any(|n| matches!(n,
        SceneNode::GradientRect { angle_deg, h, .. } if *angle_deg == 90.0 && *h == 4.0)));
}

#[test]
fn card_click_reports_activation() {
    let mut card = sample_cards().remove(5);
    let (w, h) = card.preferred_size();
    let bounds = Rect::new(0.0, 0.0, w, h);
    let r = click(|e| card.handle_event(e, bounds));
    assert!(r.clicked);
}

#[test]
fn card_renders_under_every_builtin_theme() {
    for theme in [Theme::hoff(), Theme::dark(), Theme::light()] {
        for card in sample_cards() {
            let nodes = card_nodes(&card, &theme);
            assert!(nodes.len() > 3, "variant emits a scene");
        }
    }
}

// ---------------------------------------------------------------------------
// HOFF widget fixtures
// ---------------------------------------------------------------------------

fn default_nodes(render: impl FnOnce(&mut Compositor, &Theme)) -> Vec<SceneNode> {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    render(&mut c, &theme);
    c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec()
}

#[test]
fn hoff_button_solid_is_graphite_glass_pill() {
    let b = Button::new("Follow");
    let nodes = default_nodes(|c, t| b.render(c, Rect::new(0.0, 0.0, 120.0, 44.0), t));
    // Edge-light underlay first, then the pill surface, then the label.
    assert!(matches!(nodes[0], SceneNode::GradientRect { angle_deg, .. } if angle_deg == 180.0));
    match nodes[1] {
        SceneNode::RoundedRect {
            color,
            corner_radius,
            ..
        } => {
            assert!((color[0] - 40.0 / 255.0).abs() < 1e-5, "graphite fill");
            assert!((color[3] - 0.70).abs() < 1e-5, "rgba(40,40,40,.70)");
            assert_eq!(corner_radius, 20.5, "pill radius (22) minus the border");
        }
        ref other => panic!("expected pill surface, got {other:?}"),
    }
    assert!(matches!(nodes.last(), Some(SceneNode::Text { .. })));
}

#[test]
fn hoff_switch_track_and_knob_match_spec() {
    let theme = Theme::hoff();
    let mut c = Compositor::new();
    c.begin_frame();
    let sw = Switch::new(false);
    sw.render(&mut c, Rect::new(0.0, 0.0, 44.0, 24.0), &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();

    match nodes[0] {
        SceneNode::RoundedRect {
            w,
            h,
            corner_radius,
            color,
            ..
        } => {
            assert_eq!((w, h), (44.0, 24.0));
            assert_eq!(corner_radius, 12.0);
            assert!((color[3] - 0.05).abs() < 1e-5, "off track rgba($n2,.05)");
        }
        ref other => panic!("expected track, got {other:?}"),
    }
    match nodes[1] {
        SceneNode::GradientRect { x, y, w, h, .. } => {
            assert_eq!((w, h), (16.0, 16.0), "16px knob");
            assert_eq!((x, y), (4.0, 4.0), "knob rests at (4,4)");
        }
        ref other => panic!("expected knob, got {other:?}"),
    }
}

#[test]
fn hoff_switch_on_knob_is_white_gradient() {
    let theme = Theme::hoff();
    let mut sw = Switch::new(true);
    // Settle the spring at the on position.
    for _ in 0..300 {
        sw.tick(1.0 / 60.0);
    }
    let mut c = Compositor::new();
    c.begin_frame();
    sw.render(&mut c, Rect::new(0.0, 0.0, 44.0, 24.0), &theme);
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes().to_vec();
    match nodes[1] {
        SceneNode::GradientRect {
            x, color, color2, ..
        } => {
            assert!((x - 24.0).abs() < 0.1, "knob travelled 20px");
            assert!((color[3] - 0.90).abs() < 0.01, "top stop .90");
            assert!((color2[3] - 0.30).abs() < 0.01, "bottom stop .30");
        }
        ref other => panic!("expected gradient knob, got {other:?}"),
    }
}

#[test]
fn hoff_tooltip_is_solid_262626() {
    let theme = Theme::hoff();
    let mut tip = Tooltip::new("hint").delay(0.0);
    tip.set_hover(true, Rect::new(100.0, 100.0, 50.0, 20.0));
    tip.tick(0.1);
    let mut c = Compositor::new();
    c.begin_frame();
    let layer = c.create_layer(500);
    tip.render(&mut c, layer, &theme, 800.0, 600.0);
    let nodes = c.layer(layer).unwrap().nodes().to_vec();
    assert!(matches!(nodes[0], SceneNode::Shadow { .. }));
    assert!(
        matches!(nodes[1], SceneNode::RoundedRect { color, corner_radius, .. }
        if (color[0] - 0x26 as f32 / 255.0).abs() < 1e-5 && corner_radius == 8.0)
    );
}

// ---------------------------------------------------------------------------
// Chip
// ---------------------------------------------------------------------------

#[test]
fn chip_preferred_size_uses_real_measurement() {
    let chip = Chip::new("ann");
    let (w, h) = chip.preferred_size();
    assert_eq!(h, CHIP_H);
    let (tw, _) = crate::text::TextMeasurer::measure_styled(
        "ann",
        &crate::theme::TypographyScale::hoff().caption_sm(),
        None,
    );
    assert!((w - (tw + 24.0)).abs() < 1.0, "width is text + padding");
}

#[test]
fn chip_is_static_by_default_and_clicks_when_interactive() {
    let mut chip = Chip::new("tag");
    let r = click(|e| chip.handle_event(e, B));
    assert_eq!(r, EventResult::IGNORED, "static chips swallow nothing");

    let mut chip = Chip::new("tag").interactive(true);
    let r = click(|e| chip.handle_event(e, B));
    assert!(r.clicked);
    // Press cancelled by releasing outside.
    chip.handle_event(&down(20.0, 20.0), B);
    let r = chip.handle_event(&up(500.0, 500.0), B);
    assert!(!r.clicked);
    assert!(r.changed);
}

#[test]
fn chip_renders_every_intent_and_variant_without_gpu() {
    let theme = Theme::hoff();
    for intent in [
        Intent::Neutral,
        Intent::Constructive,
        Intent::Destructive,
        Intent::Informational,
    ] {
        for selected in [false, true] {
            let mut c = Compositor::new();
            Chip::new("capability")
                .intent(intent)
                .selected(selected)
                .render(&mut c, B, &theme);
        }
    }
}

// ---------------------------------------------------------------------------
// EmptyState
// ---------------------------------------------------------------------------

#[test]
fn empty_state_centers_its_stack_and_routes_cta_clicks() {
    let mut es = EmptyState::new("No database", "Open a .nest file to explore it.")
        .icon("folder-open")
        .cta(Button::new("Open"));
    let bounds = Rect::new(0.0, 0.0, 800.0, 600.0);
    // The CTA is centered horizontally; find it by probing the center.
    let cta = es.cta.as_ref().unwrap().preferred_size();
    let cx = bounds.x + (bounds.w - cta.0) / 2.0 + 2.0;
    // Probe vertically: scan for the cta's y band.
    let mut fired = false;
    for y in (0..600).step_by(4) {
        let r = click(|e| es.handle_event(e, bounds));
        let _ = (cx, y);
        if r.clicked {
            fired = true;
            break;
        }
        let r1 = es.handle_event(&down(cx, y as f32), bounds);
        let r2 = es.handle_event(&up(cx, y as f32), bounds);
        if r1.merge(r2).clicked {
            fired = true;
            break;
        }
    }
    assert!(fired, "the CTA is clickable somewhere in the stack");
}

#[test]
fn empty_state_without_cta_is_inert() {
    let mut es = EmptyState::new("Title", "Message");
    let r = click(|e| es.handle_event(e, B));
    assert_eq!(r, EventResult::IGNORED);
}

#[test]
fn empty_state_renders_narrow_and_wide_without_gpu() {
    let theme = Theme::hoff();
    for w in [320.0, 1600.0] {
        let mut c = Compositor::new();
        EmptyState::new(
            "A fairly long empty-state title",
            "A long message that must wrap against the available width rather than run edge to edge in the panel it is given.",
        )
        .icon("file")
        .cta(Button::new("Do the thing"))
        .render(&mut c, Rect::new(0.0, 0.0, w, 600.0), &theme);
    }
}

// ---------------------------------------------------------------------------
// Spinner
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// SplitPane
// ---------------------------------------------------------------------------

#[test]
fn split_pane_rects_partition_the_bounds() {
    let sp = SplitPane::new(SplitDirection::Horizontal, 0.25);
    let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
    let first = sp.first_rect(bounds);
    let second = sp.second_rect(bounds);
    // 25% of (1000 - 2px divider) = 249.5; the second pane gets the rest.
    assert!((first.w - 249.5).abs() < 0.01);
    assert!((second.w - 748.5).abs() < 0.01);
    assert!((second.x - (first.x + first.w + 2.0)).abs() < 0.01);

    let v = SplitPane::new(SplitDirection::Vertical, 0.5);
    let first = v.first_rect(bounds);
    let second = v.second_rect(bounds);
    assert!((first.h - 299.0).abs() < 0.01);
    assert!((second.y - (first.y + first.h + 2.0)).abs() < 0.01);
}

#[test]
fn split_pane_drag_updates_ratio_and_clamps_pane_minimums() {
    let mut sp = SplitPane::new(SplitDirection::Horizontal, 0.5);
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
    let mut sp = SplitPane::new(SplitDirection::Horizontal, 0.5);
    let bounds = Rect::new(0.0, 0.0, 1000.0, 600.0);
    assert!(sp.handle_event(&move_to(499.0, 300.0), bounds).changed);
    assert!(sp.is_hovered());
    let mut c = Compositor::new();
    sp.render(&mut c, bounds, &theme);
}

// ---------------------------------------------------------------------------
// IconButton
// ---------------------------------------------------------------------------

#[test]
fn icon_button_click_contract_matches_button() {
    let mut b = IconButton::new("copy");
    assert!(!b.handle_event(&down(20.0, 20.0), B).clicked);
    assert!(b.is_pressed());
    assert!(b.handle_event(&up(20.0, 20.0), B).clicked);

    let mut b = IconButton::new("copy").disabled(true);
    let r = click(|e| b.handle_event(e, B));
    assert!(!r.clicked);
}

#[test]
fn icon_button_is_square_and_renders_all_variants() {
    let (w, h) = IconButton::new("x").preferred_size();
    assert_eq!(w, h);
    let theme = Theme::hoff();
    for variant in [
        ButtonVariant::Solid,
        ButtonVariant::Outline,
        ButtonVariant::Ghost,
        ButtonVariant::Danger,
    ] {
        let mut c = Compositor::new();
        IconButton::new("trash")
            .variant(variant)
            .intent(Intent::Destructive)
            .render(&mut c, B, &theme);
    }
}
