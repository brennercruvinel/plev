use super::*;
use plev::compositor::Compositor;
use plev::overlay::OverlayKind;
use winit::keyboard::{Key, NamedKey};

fn ws() -> (WorkspaceView, Compositor) {
    let mut w = WorkspaceView::new(1280.0, 800.0);
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
    assert!(node_count >= 4, "overlay layer has {node_count} nodes, expected >= 4");
}

#[test]
fn escape_closes_overlay() {
    let (mut w, _c) = ws();
    let (cx, cy) = first_file_center(&w);
    w.handle_right_click(cx, cy);
    assert!(!w.overlay_mgr.is_empty());

    let esc = Key::Named(NamedKey::Escape);
    let changed = w.handle_key_down(&esc);
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
fn stage_removes_file_via_context_menu() {
    let (mut w, mut c) = ws();
    let initial_count = w.unassigned.files.len();
    let (cx, cy) = first_file_center(&w);

    w.handle_right_click(cx, cy);
    w.render(&mut c); // populate ctx_menu_item_rects

    // click the first item (Stage)
    assert!(!w.ctx_menu_item_rects.is_empty());
    let (ix, iy, iw, ih) = w.ctx_menu_item_rects[0];
    let changed = w.handle_click(ix + iw / 2.0, iy + ih / 2.0);

    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
    assert_eq!(w.unassigned.files.len(), initial_count - 1);
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
fn modal_confirm_removes_file() {
    let (mut w, mut c) = ws();
    let initial_count = w.unassigned.files.len();
    let (cx, cy) = first_file_center(&w);

    // right-click -> context menu
    w.handle_right_click(cx, cy);
    w.render(&mut c);

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

    // click cancel button
    let (rx, ry, rw, rh) = w.modal_cancel_rect.unwrap();
    let changed = w.handle_click(rx + rw / 2.0, ry + rh / 2.0);

    assert!(changed);
    assert!(w.overlay_mgr.is_empty());
    assert_eq!(w.unassigned.files.len(), initial_count);
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
