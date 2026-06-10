use super::*;
use crate::views::unassigned_view::{FileEntry, FileStatus};
use plev::compositor::Compositor;
use plev::overlay::OverlayKind;

/// Fixture data injected into the view — tests never touch a real repo;
/// the app feeds the same structs from `git status` via `adapters`.
fn sample_files() -> Vec<FileEntry> {
    let entry = |path: &str, status, staged| FileEntry {
        path: path.into(),
        status,
        staged,
    };
    vec![
        entry("src/compositor.rs", FileStatus::Modified, false),
        entry("src/builder.rs", FileStatus::Modified, false),
        entry("src/scroll.rs", FileStatus::Added, true),
        entry("docs/notes.md", FileStatus::Untracked, false),
        entry("old/gone.rs", FileStatus::Deleted, false),
    ]
}

fn ws() -> (WorkspaceView, Compositor) {
    let mut w = WorkspaceView::new(1280.0, 800.0);
    w.unassigned.set_files(sample_files());
    let mut c = Compositor::new();
    w.render(&mut c);
    (w, c)
}

fn first_file_center(w: &WorkspaceView) -> (f32, f32) {
    let r = &w.unassigned.hit_rects()[0];
    (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0)
}

#[test]
fn right_click_opens_context_menu() {
    let (mut w, _c) = ws();
    let initial_files = w.unassigned.files.len();
    assert!(initial_files > 0);
    assert!(w.overlay_mgr.is_empty());

    let (cx, cy) = first_file_center(&w);
    let changed = w.handle_right_click(cx, cy);

    assert!(changed);
    assert_eq!(w.overlay_mgr.len(), 1);
    assert!(matches!(
        w.overlay_mgr.top().unwrap().kind,
        OverlayKind::ContextMenu { .. }
    ));
}

#[test]
fn overlay_layer_receives_scene_nodes_on_render() {
    let (mut w, mut c) = ws();
    let (cx, cy) = first_file_center(&w);
    w.handle_right_click(cx, cy);

    // re-render with overlay active
    w.render(&mut c);

    let overlay_layer = c.layer(w.overlay_layer).unwrap();
    let node_count = overlay_layer.nodes().len();
    // context menu: 1 background RoundedRect + 3 items * (optional hover rect + text) = >= 4 nodes
    assert!(
        node_count >= 4,
        "overlay layer has {node_count} nodes, expected >= 4"
    );
}

#[test]
fn escape_closes_overlay() {
    let (mut w, _c) = ws();
    let (cx, cy) = first_file_center(&w);
    w.handle_right_click(cx, cy);
    assert!(!w.overlay_mgr.is_empty());

    // Escape is bound to overlay::Close in the keymap; the dispatcher
    // invokes this method.
    let changed = w.close_top_overlay();
    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
}

#[test]
fn click_outside_dismisses_overlay() {
    let (mut w, mut c) = ws();
    let (cx, cy) = first_file_center(&w);
    w.handle_right_click(cx, cy);
    w.render(&mut c); // populate ctx_menu_item_rects

    // click far from the menu
    let changed = w.handle_click(1200.0, 700.0);
    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
}

#[test]
fn stage_via_context_menu_marks_file_and_queues_request() {
    let (mut w, mut c) = ws();
    let initial_count = w.unassigned.files.len();
    let (cx, cy) = first_file_center(&w);

    w.handle_right_click(cx, cy);
    w.render(&mut c); // populate ctx_menu_item_rects
    w.take_requests(); // drop the FileDiff request from row selection

    // click the first item (Stage)
    assert!(!w.ctx_menu_item_rects.is_empty());
    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[0];
    let changed = w.handle_click(ix + iw / 2.0, iy + ih / 2.0);

    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
    // The file moves to staged (dot marker) instead of vanishing; the
    // status refresh after the real `git add` reconciles the list.
    assert_eq!(w.unassigned.files.len(), initial_count);
    assert!(w.unassigned.files[0].staged);
    assert_eq!(
        w.take_requests(),
        vec![UiRequest::Stage {
            path: "src/compositor.rs".into()
        }]
    );
}

#[test]
fn unstage_via_context_menu_on_staged_file() {
    let (mut w, mut c) = ws();
    // file index 2 ("src/scroll.rs") is staged in the fixture
    let r = w.unassigned.hit_rects()[2];
    let (cx, cy) = (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);

    w.handle_right_click(cx, cy);
    w.render(&mut c);
    w.take_requests();

    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[0];
    w.handle_click(ix + iw / 2.0, iy + ih / 2.0);

    assert!(!w.unassigned.files[2].staged);
    assert_eq!(
        w.take_requests(),
        vec![UiRequest::Unstage {
            path: "src/scroll.rs".into()
        }]
    );
}

#[test]
fn discard_opens_modal() {
    let (mut w, mut c) = ws();
    let (cx, cy) = first_file_center(&w);

    w.handle_right_click(cx, cy);
    w.render(&mut c);

    // click "Discard changes" (item index 1)
    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[1];
    w.handle_click(ix + iw / 2.0, iy + ih / 2.0);

    // context menu gone, modal pushed
    assert_eq!(w.overlay_mgr.len(), 1);
    assert!(matches!(
        w.overlay_mgr.top().unwrap().kind,
        OverlayKind::Modal { .. }
    ));
}

#[test]
fn modal_confirm_removes_file_and_queues_discard() {
    let (mut w, mut c) = ws();
    let initial_count = w.unassigned.files.len();
    let (cx, cy) = first_file_center(&w);

    // right-click -> context menu
    w.handle_right_click(cx, cy);
    w.render(&mut c);
    w.take_requests();

    // click "Discard changes" -> modal
    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[1];
    w.handle_click(ix + iw / 2.0, iy + ih / 2.0);
    w.render(&mut c); // populate modal rects

    // click confirm button
    let (rx, ry, rw, rh) = w.modal_confirm_rect.unwrap();
    let changed = w.handle_click(rx + rw / 2.0, ry + rh / 2.0);

    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
    assert_eq!(w.unassigned.files.len(), initial_count - 1);
    assert_eq!(
        w.take_requests(),
        vec![UiRequest::Discard {
            path: "src/compositor.rs".into()
        }]
    );
}

#[test]
fn modal_cancel_preserves_file() {
    let (mut w, mut c) = ws();
    let initial_count = w.unassigned.files.len();
    let (cx, cy) = first_file_center(&w);

    w.handle_right_click(cx, cy);
    w.render(&mut c);

    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[1];
    w.handle_click(ix + iw / 2.0, iy + ih / 2.0);
    w.render(&mut c);
    w.take_requests();

    // click cancel button
    let (rx, ry, rw, rh) = w.modal_cancel_rect.unwrap();
    let changed = w.handle_click(rx + rw / 2.0, ry + rh / 2.0);

    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
    assert_eq!(w.unassigned.files.len(), initial_count);
    assert!(w.take_requests().is_empty(), "cancel must not touch git");
}

#[test]
fn overlay_blocks_normal_clicks() {
    let (mut w, mut c) = ws();
    let (cx, cy) = first_file_center(&w);
    w.handle_right_click(cx, cy);
    w.render(&mut c);

    // click on a different file row — should NOT change selection
    // because overlay intercepts
    let old_sel = w.unassigned.selected_idx;
    if w.unassigned.hit_rects().len() > 1 {
        let r = w.unassigned.hit_rects()[1];
        w.handle_click(r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
        // selection unchanged because overlay consumed the click
        // (it was either inside the overlay or outside = dismiss)
    }
    // overlay state should have changed (either item clicked or dismissed)
    // but the underlying file list selection should not have advanced
    assert!(w.overlay_mgr.is_empty() || w.unassigned.selected_idx == old_sel);
}

// -- Reactivity probes: scroll + hover must move state ----------------------

fn many_files(n: usize) -> Vec<FileEntry> {
    (0..n)
        .map(|i| FileEntry {
            path: format!("src/file_{i}.rs"),
            status: FileStatus::Modified,
            staged: false,
        })
        .collect()
}

fn sample_stacks(commits: usize) -> Vec<crate::views::multi_stack_view::Stack> {
    use crate::views::multi_stack_view::{CommitEntry, Stack};
    vec![Stack {
        branch_name: "main".into(),
        is_active: true,
        commits: (0..commits)
            .map(|i| CommitEntry {
                sha: format!("{i:07x}"),
                message: format!("commit {i}"),
                author: "Hoff".into(),
                time_ago: "now".into(),
            })
            .collect(),
    }]
}

fn sample_diff(lines: usize) -> Vec<crate::views::diff_view::DiffLine> {
    use crate::views::diff_view::{DiffLine, DiffLineKind};
    (0..lines)
        .map(|i| DiffLine {
            kind: DiffLineKind::Context,
            line_no_old: Some(i as u32 + 1),
            line_no_new: Some(i as u32 + 1),
            content: format!("line {i}"),
        })
        .collect()
}

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

// ---------------------------------------------------------------------------
// Panel width clamp — non-destructive (desired vs effective widths)
// ---------------------------------------------------------------------------

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
