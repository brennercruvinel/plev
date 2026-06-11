use super::dock::*;
use plev::ui::widgets::Rect;

const DT: f32 = 1.0 / 60.0;

fn area() -> Rect {
    Rect::new(0.0, 0.0, 1200.0, 800.0)
}

fn step(dock: &mut DockModel, frames: usize) {
    for _ in 0..frames {
        dock.update(DT);
    }
}

/// Enough frames for every tween and the send flash to finish (2s).
fn settle(dock: &mut DockModel) {
    step(dock, 120);
}

// -- happy path -------------------------------------------------------------

#[test]
fn new_starts_idle_collapsed_and_at_rest() {
    let dock = DockModel::new();
    assert_eq!(dock.state(), DockState::Idle);
    assert_eq!(dock.selected(), None);
    assert!(!dock.is_animating(), "nothing moves before any input");
    assert_eq!(dock.input_alpha(), 0.0);
    assert_eq!(dock.separator_alpha(), 1.0);
    assert_eq!(dock.cursor_alpha(), 0.0);
    for i in 0..AVATARS {
        assert_eq!(dock.avatar_alpha(i), 1.0);
        assert_eq!(dock.avatar_lift(i), 0.0);
    }
    let dock_r = dock.dock_rect(area());
    assert!(dock_r.w < area().w, "collapsed row fits the area");
    assert!(
        dock_r.y + dock_r.h < area().h,
        "bottom-anchored inside the area"
    );
}

#[test]
fn click_expands_then_settles_expanded_with_crossed_opacities() {
    let mut dock = DockModel::new();
    assert!(dock.on_click(1));
    assert_eq!(dock.state(), DockState::Expanding(1));
    assert!(dock.is_animating());
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Expanded(1));
    assert_eq!(dock.selected(), Some(1));
    assert_eq!(dock.input_alpha(), 1.0);
    assert_eq!(dock.separator_alpha(), 0.0);
    assert_eq!(dock.avatar_alpha(1), 1.0, "selected stays visible");
    for i in [0usize, 2, 3] {
        assert_eq!(dock.avatar_alpha(i), 0.0, "others fade out");
    }
    assert!(
        dock.is_animating(),
        "expanded keeps frames for the caret blink"
    );
}

#[test]
fn morph_converges_exactly_to_target_both_ways() {
    let mut dock = DockModel::new();
    let collapsed = dock.width(area());
    dock.on_click(0);
    settle(&mut dock);
    let expanded = dock.width(area());
    assert_eq!(expanded, 480.0, "wide area expands to the max width");
    step(&mut dock, 10);
    assert_eq!(dock.width(area()), expanded, "no crawl after completion");

    dock.on_click(0); // selected again: collapse
    assert_eq!(dock.state(), DockState::Collapsing);
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Idle);
    assert_eq!(dock.width(area()), collapsed);
    assert!(!dock.is_animating(), "fully at rest after collapsing");
}

#[test]
fn hover_lifts_only_that_avatar_and_follows_the_pointer() {
    let mut dock = DockModel::new();
    assert!(dock.on_hover(Some(1)));
    assert_eq!(dock.state(), DockState::Hover(1));
    settle(&mut dock);
    assert!(dock.avatar_lift(1) > 7.9, "hovered avatar fully lifted");
    for i in [0usize, 2, 3] {
        assert_eq!(dock.avatar_lift(i), 0.0);
    }
    let resting = dock.avatar_rect(0, area()).y;
    assert!(
        dock.avatar_rect(1, area()).y < resting,
        "lift raises the rect"
    );

    assert!(dock.on_hover(Some(2)), "pointer slides to the neighbor");
    settle(&mut dock);
    assert_eq!(dock.avatar_lift(1), 0.0);
    assert!(dock.avatar_lift(2) > 7.9);

    assert!(dock.on_hover(None));
    assert_eq!(dock.state(), DockState::Idle);
    settle(&mut dock);
    assert_eq!(dock.avatar_lift(2), 0.0);
}

#[test]
fn send_flashes_then_collapses_to_idle() {
    let mut dock = DockModel::new();
    dock.on_click(2);
    settle(&mut dock);
    assert!(dock.on_send());
    assert_eq!(dock.state(), DockState::Sending(2));
    assert!(dock.flash_alpha() > 0.9, "flash starts hot");
    step(&mut dock, 12);
    let mid = dock.flash_alpha();
    assert!(mid > 0.0 && mid < 1.0, "flash decays over time, got {mid}");
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Idle);
    assert_eq!(dock.flash_alpha(), 0.0);
    assert_eq!(dock.input_alpha(), 0.0);
    for i in 0..AVATARS {
        assert_eq!(dock.avatar_alpha(i), 1.0, "roster returns after send");
    }
}

#[test]
fn cursor_blinks_only_while_the_panel_is_visible() {
    let mut dock = DockModel::new();
    for _ in 0..60 {
        dock.update(DT);
        assert_eq!(dock.cursor_alpha(), 0.0, "no caret while collapsed");
    }
    dock.on_click(0);
    settle(&mut dock);
    let (mut seen_on, mut seen_off) = (false, false);
    for _ in 0..90 {
        // 1.5s: covers a full blink period
        dock.update(DT);
        match dock.cursor_alpha() {
            a if a > 0.9 => seen_on = true,
            0.0 => seen_off = true,
            _ => {}
        }
    }
    assert!(
        seen_on && seen_off,
        "caret must toggle (on={seen_on}, off={seen_off})"
    );
}

// -- mid-morph retargeting --------------------------------------------------

#[test]
fn cancel_mid_morph_retargets_without_a_width_jump() {
    let mut dock = DockModel::new();
    let collapsed = dock.width(area());
    dock.on_click(1);
    step(&mut dock, 6); // partway through the open morph
    let before = dock.width(area());
    assert!(before > collapsed, "morph is in flight");

    assert!(dock.on_click(1), "clicking the selected avatar cancels");
    assert_eq!(dock.state(), DockState::Collapsing);
    let after = dock.width(area());
    assert!(
        (before - after).abs() < 1e-3,
        "no jump: {before} -> {after}"
    );
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Idle);
    assert_eq!(dock.width(area()), collapsed);
}

#[test]
fn reselect_mid_morph_keeps_expanding_continuously() {
    let mut dock = DockModel::new();
    dock.on_click(1);
    step(&mut dock, 6);
    let before = dock.width(area());
    assert!(dock.on_click(3), "re-aim at another avatar mid-flight");
    assert_eq!(dock.state(), DockState::Expanding(3));
    assert!(
        (dock.width(area()) - before).abs() < 1e-3,
        "width is continuous"
    );
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Expanded(3));
    assert_eq!(dock.avatar_alpha(3), 1.0);
    assert_eq!(dock.avatar_alpha(1), 0.0);
}

#[test]
fn reopen_while_collapsing_morphs_back_without_a_jump() {
    let mut dock = DockModel::new();
    dock.on_click(0);
    settle(&mut dock);
    dock.on_click(0); // collapse...
    step(&mut dock, 6);
    let before = dock.width(area());
    assert!(dock.on_click(2), "...and reopen mid-collapse");
    assert_eq!(dock.state(), DockState::Expanding(2));
    assert!((dock.width(area()) - before).abs() < 1e-3);
    settle(&mut dock);
    assert_eq!(dock.state(), DockState::Expanded(2));
}

// -- error paths and edges --------------------------------------------------

#[test]
fn invalid_input_is_ignored_in_every_state() {
    let mut dock = DockModel::new();
    assert!(!dock.on_click(AVATARS), "out-of-range click");
    assert!(!dock.on_hover(Some(AVATARS)), "out-of-range hover");
    assert!(!dock.on_send(), "send with nothing selected");
    assert!(!dock.on_hover(None), "hover-leave while already idle");
    assert_eq!(dock.state(), DockState::Idle);

    dock.on_click(1);
    settle(&mut dock);
    assert!(
        !dock.on_hover(Some(2)),
        "hover does not drive expanded states"
    );
    assert_eq!(dock.state(), DockState::Expanded(1));

    dock.on_send();
    assert!(!dock.on_click(1), "clicks ignored while sending");
    assert!(!dock.on_send(), "send is not re-entrant");
    assert_eq!(dock.state(), DockState::Sending(1));
}

#[test]
fn narrow_area_clamps_both_widths_to_the_available_space() {
    let mut dock = DockModel::new();
    let narrow = Rect::new(0.0, 0.0, 200.0, 400.0);
    assert_eq!(dock.width(narrow), 200.0, "collapsed clamps to the area");
    dock.on_click(0);
    settle(&mut dock);
    assert_eq!(dock.width(narrow), 200.0, "expanded cannot overflow either");
    assert_eq!(
        dock.width(area()),
        480.0,
        "same state, wide area: max width"
    );
}

#[test]
fn selected_avatar_slides_to_the_front_slot() {
    let mut dock = DockModel::new();
    dock.on_click(2);
    settle(&mut dock);
    let dock_r = dock.dock_rect(area());
    let av = dock.avatar_rect(2, area());
    assert!(
        (av.x - (dock_r.x + 10.0)).abs() < 1e-3,
        "selected sits at the front pad, got {} vs dock {}",
        av.x,
        dock_r.x
    );
}
