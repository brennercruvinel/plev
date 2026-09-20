//! Workspace chrome state: the commit form and the non-destructive panel
//! width clamp (desired vs effective widths).

use super::*;

#[test]
fn commit_form_submit_queues_commit_and_clears() {
    let (mut w, _c) = ws();
    assert!(!w.submit_commit(), "hidden form must not submit");

    w.toggle_commit_form();
    assert!(w.commit_form.visible);
    assert!(!w.submit_commit(), "empty message must not submit");

    for ch in "fix: real commit".chars() {
        w.commit_form.append_char(ch);
    }
    assert!(w.submit_commit());
    assert!(!w.commit_form.visible);
    assert!(w.commit_form.message().is_empty());
    assert_eq!(
        w.take_requests(),
        vec![UiRequest::Commit {
            message: "fix: real commit".into()
        }]
    );
}

#[test]
fn shrinking_then_growing_window_restores_panel_widths() {
    // Regression: the old clamp scaled `left_w`/`right_w` in place when the
    // window shrank, destroying the user's layout — growing the window back
    // never restored the panels.
    let mut w = WorkspaceView::new(1280.0, 800.0);
    let (left0, right0) = (w.left_w, w.right_w);
    assert!(left0 > 0.0 && right0 > 0.0, "defaults at 1280px");

    // Shrink hard: panels must squeeze to keep the 200px middle column.
    w.resize(700.0, 800.0);
    assert!(w.left_w < left0 && w.right_w < right0, "shrink must clamp");

    // Grow back: the desired widths were never overwritten, so the
    // effective widths return exactly to the user's layout.
    w.resize(1280.0, 800.0);
    assert_eq!((w.left_w, w.right_w), (left0, right0));
}

#[test]
fn dragged_width_survives_shrink_and_grow_cycle() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    // User drags the left panel 40px wider: that is the new desired width.
    w.begin_drag_left(sidebar_w() + w.left_w);
    w.update_drag(sidebar_w() + w.left_w + 40.0);
    w.end_drag();
    let dragged = w.left_w;
    assert_eq!(dragged, 320.0);

    w.resize(640.0, 800.0);
    assert!(w.left_w < dragged);
    w.resize(1280.0, 800.0);
    assert_eq!(w.left_w, dragged, "grow must restore the dragged width");
    assert_eq!(w.right_w, WorkspaceView::new(1280.0, 800.0).right_w);
}

#[test]
fn effective_widths_are_clamped_on_construction() {
    // A window too small for the defaults must start clamped (the desired
    // defaults stay intact for a later grow). 760px keeps the proportional
    // clamp above the column floors, so the middle column keeps its full
    // minimum.
    let defaults = WorkspaceView::new(1280.0, 800.0);
    let mut w = WorkspaceView::new(760.0, 600.0);
    let middle_min = defaults.theme().size.field_min_w + defaults.theme().size.field_min_w / 4.0;
    assert!(
        sidebar_w() + w.left_w + w.right_w + middle_min <= 760.0 + 1e-3,
        "left {} + right {} must leave a {middle_min}px middle at 760px",
        w.left_w,
        w.right_w
    );
    w.resize(1280.0, 800.0);
    assert_eq!((w.left_w, w.right_w), (defaults.left_w, defaults.right_w));
}
