//! Context menu / modal flow: stage, unstage, discard and dismissal.

use super::*;

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
