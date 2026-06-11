//! Reactivity probes: per-panel scroll, hit clamping behind panel heads,
//! clipping of scrolled rows and hover reporting.

use super::*;

#[test]
fn probe_scroll_left_panel_moves_file_list() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(many_files(60));
    let mut c = Compositor::new();
    w.render(&mut c); // sets viewport/content heights

    let cx = SIDEBAR_W + 50.0; // inside the left panel
    w.scroll(cx, 120.0);
    assert!(
        w.unassigned.scroll.offset() > 0.0,
        "left panel scroll must move (offset={})",
        w.unassigned.scroll.offset()
    );
}

#[test]
fn probe_scroll_middle_panel_moves_commits() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.stacks.set_stacks(sample_stacks(50));
    let mut c = Compositor::new();
    w.render(&mut c);

    let (left_x, right_x) = w.panel_bounds();
    let cx = (left_x + w.left_w + right_x) / 2.0; // middle panel
    w.scroll(cx, 120.0);
    assert!(
        w.stacks.scroll.offset() > 0.0,
        "stacks scroll must move (offset={})",
        w.stacks.scroll.offset()
    );
}

#[test]
fn probe_scroll_right_panel_moves_diff() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.diff.set_lines(sample_diff(200));
    let mut c = Compositor::new();
    w.render(&mut c);

    let cx = 1280.0 - 50.0; // inside the diff panel
    w.scroll(cx, 120.0);
    assert!(
        w.diff.scroll.offset() > 0.0,
        "diff scroll must move (offset={})",
        w.diff.scroll.offset()
    );
}

#[test]
fn probe_scrolled_left_panel_renders_shifted_rows() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(many_files(60));
    let mut c = Compositor::new();
    w.render(&mut c);
    let y_before = w.unassigned.hit_rects()[10].1;

    w.scroll(SIDEBAR_W + 50.0, 120.0);
    w.render(&mut c);
    let y_after = w.unassigned.hit_rects()[10].1;
    assert!(
        y_after < y_before,
        "rows must shift up after scroll (before={y_before}, after={y_after})"
    );
}

#[test]
fn sidebar_rail_does_not_scroll_panels() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(many_files(60));
    w.stacks.set_stacks(sample_stacks(50));
    w.diff.set_lines(sample_diff(200));
    let mut c = Compositor::new();
    w.render(&mut c);

    assert!(!w.scroll(30.0, 120.0), "the 72px rail must not scroll");
    assert_eq!(w.unassigned.scroll.offset(), 0.0);
    assert_eq!(w.stacks.scroll.offset(), 0.0);
    assert_eq!(w.diff.scroll.offset(), 0.0);
}

#[test]
fn rows_hidden_behind_the_panel_head_are_not_hit() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(many_files(60));
    let mut c = Compositor::new();
    w.render(&mut c);

    // Scroll the file list so early rows slide under the "Changes" head.
    let cx = SIDEBAR_W + 50.0;
    assert!(w.scroll(cx, 300.0));
    w.render(&mut c);

    // Clicking/hovering on the head band must hit nothing — before the fix
    // the hidden rows kept full-size hit rects and swallowed these events.
    let head_y = HEADER_H + 30.0;
    assert_eq!(w.unassigned.hit_test(cx, head_y), None);
    assert!(
        !w.handle_hover(cx, head_y),
        "no hover change on the head band"
    );
    assert!(
        !w.handle_click(cx, head_y),
        "no selection from a hidden row"
    );
}

#[test]
fn commits_hidden_behind_the_stacks_head_are_not_hit() {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.stacks.set_stacks(sample_stacks(50));
    let mut c = Compositor::new();
    w.render(&mut c);

    let (left_x, right_x) = w.panel_bounds();
    let cx = (left_x + w.left_w + right_x) / 2.0;
    assert!(w.scroll(cx, 400.0));
    w.render(&mut c);

    let head_y = HEADER_H + 30.0;
    assert_eq!(w.stacks.hit_test(cx, head_y), None);
    assert!(!w.handle_click(cx, head_y));
}

#[test]
fn scrolled_panels_clip_rows_to_the_list_viewport() {
    use plev::compositor::{LayerId, SceneNode};
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(many_files(60));
    let mut c = Compositor::new();
    w.scroll(SIDEBAR_W + 50.0, 300.0);
    w.render(&mut c);

    // The file list emits a PushClip at the list viewport so scrolled rows
    // cannot paint over the panel head.
    let nodes = c.layer(LayerId::DEFAULT).unwrap().nodes();
    let has_list_clip = nodes.iter().any(|n| {
        matches!(n, SceneNode::PushClip { x, y, .. }
            if *x == SIDEBAR_W && *y > HEADER_H)
    });
    assert!(has_list_clip, "scrolled rows must be clipped to the panel");
}

#[test]
fn probe_hover_over_file_row_reports_change() {
    let (mut w, _c) = ws();
    let (cx, cy) = first_file_center(&w);
    assert!(w.handle_hover(cx, cy), "hover entering a row must report");
    assert!(!w.handle_hover(cx, cy), "same hover must not re-report");
    assert!(
        w.handle_hover(cx, cy + 2000.0),
        "hover leaving a row must report"
    );
}
