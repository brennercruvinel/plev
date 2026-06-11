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
    assert!(w.commit_form.message.is_empty());
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
    assert_eq!((left0, right0), (280.0, 340.0), "defaults at 1280px");

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
    w.begin_drag_left(SIDEBAR_W + w.left_w);
    w.update_drag(SIDEBAR_W + w.left_w + 40.0);
    w.end_drag();
    let dragged = w.left_w;
    assert_eq!(dragged, 320.0);

    w.resize(640.0, 800.0);
    assert!(w.left_w < dragged);
    w.resize(1280.0, 800.0);
    assert_eq!(w.left_w, dragged, "grow must restore the dragged width");
    assert_eq!(w.right_w, 340.0);
}

#[test]
fn effective_widths_are_clamped_on_construction() {
    // A window too small for the defaults must start clamped (the desired
    // defaults stay intact for a later grow). 720px keeps the proportional
    // clamp above the LEFT_MIN_W/RIGHT_MIN_W floors, so the middle column
    // keeps its full 200px minimum.
    let mut w = WorkspaceView::new(720.0, 600.0);
    let middle_min = 200.0;
    assert!(
        SIDEBAR_W + w.left_w + w.right_w + middle_min <= 720.0 + 1e-3,
        "left {} + right {} must leave a {middle_min}px middle at 720px",
        w.left_w,
        w.right_w
    );
    w.resize(1280.0, 800.0);
    assert_eq!((w.left_w, w.right_w), (280.0, 340.0));
}
