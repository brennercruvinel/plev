use plev::compositor::{Compositor, SceneNode};
use plev::overlay::OverlayKind;
use crate::actions::{FileAction, ModalAction};
use crate::components::{context_menu, modal};
use crate::theme::Theme;
use super::WorkspaceView;

impl WorkspaceView {
    /// Handle a click when overlays are active. Returns true if consumed.
    pub(crate) fn handle_overlay_click(&mut self, cx: f32, cy: f32) -> bool {
        if self.overlay_mgr.is_empty() {
            return false;
        }

        // Check modal buttons first (modal is always on top when present)
        if let Some(confirm_rect) = self.modal_confirm_rect {
            let (rx, ry, rw, rh) = confirm_rect;
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                // Confirmed -> discard the file
                if let Some(idx) = self.pending_discard_idx.take() {
                    if idx < self.unassigned.files.len() {
                        self.unassigned.files.remove(idx);
                        // Adjust selection
                        if self.unassigned.selected_idx == Some(idx) {
                            self.unassigned.selected_idx = None;
                        } else if let Some(sel) = self.unassigned.selected_idx {
                            if sel > idx {
                                self.unassigned.selected_idx = Some(sel - 1);
                            }
                        }
                        self.diff.clear();
                    }
                }
                self.action_queue.emit(0, ModalAction::Confirmed);
                self.overlay_mgr.pop_all();
                self.modal_confirm_rect = None;
                self.modal_cancel_rect = None;
                return true;
            }
        }
        if let Some(cancel_rect) = self.modal_cancel_rect {
            let (rx, ry, rw, rh) = cancel_rect;
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                self.action_queue.emit(0, ModalAction::Cancelled);
                self.pending_discard_idx = None;
                self.overlay_mgr.pop_all();
                self.modal_confirm_rect = None;
                self.modal_cancel_rect = None;
                return true;
            }
        }

        // Check context menu items
        for (i, &(rx, ry, rw, rh)) in self.ctx_menu_item_rects.iter().enumerate() {
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                let file_idx = self.pending_discard_idx.unwrap_or(0);
                match i {
                    0 => {
                        // Stage -- remove from unassigned
                        if file_idx < self.unassigned.files.len() {
                            let path = self.unassigned.files[file_idx].path.clone();
                            self.action_queue.emit(file_idx as u64, FileAction::Stage(path));
                            self.unassigned.files.remove(file_idx);
                            if self.unassigned.selected_idx == Some(file_idx) {
                                self.unassigned.selected_idx = None;
                                self.diff.clear();
                            }
                        }
                        self.overlay_mgr.pop_all();
                        self.ctx_menu_item_rects.clear();
                        self.pending_discard_idx = None;
                    }
                    1 => {
                        // Discard -- show confirmation modal
                        self.overlay_mgr.pop_all(); // close context menu first
                        self.ctx_menu_item_rects.clear();
                        let file_name = self.unassigned.files.get(file_idx)
                            .map(|f| f.path.rsplit('/').next().unwrap_or(&f.path).to_string())
                            .unwrap_or_default();
                        let (mx, my) = modal::centered_pos(self.vw, self.vh);
                        let (mw, mh) = modal::dimensions();
                        self.overlay_mgr.push(
                            OverlayKind::Modal {
                                title: "Discard changes?".into(),
                                body: format!("Discard all changes to {}? This cannot be undone.", file_name),
                                confirm: "Discard".into(),
                                cancel: "Cancel".into(),
                            },
                            mx, my, mw, mh,
                        );
                    }
                    2 => {
                        // Ignore -- remove from list
                        if file_idx < self.unassigned.files.len() {
                            let path = self.unassigned.files[file_idx].path.clone();
                            self.action_queue.emit(file_idx as u64, FileAction::Ignore(path));
                            self.unassigned.files.remove(file_idx);
                            if self.unassigned.selected_idx == Some(file_idx) {
                                self.unassigned.selected_idx = None;
                                self.diff.clear();
                            }
                        }
                        self.overlay_mgr.pop_all();
                        self.ctx_menu_item_rects.clear();
                        self.pending_discard_idx = None;
                    }
                    _ => {}
                }
                return true;
            }
        }

        // Click outside all overlays -- dismiss
        if self.overlay_mgr.hit_test_outside(cx, cy) {
            self.overlay_mgr.pop_all();
            self.ctx_menu_item_rects.clear();
            self.modal_confirm_rect = None;
            self.modal_cancel_rect = None;
            self.pending_discard_idx = None;
            return true;
        }

        // Click inside an overlay but not on an item -- consume without action
        true
    }

    /// Draw active overlays onto the overlay layer. Must be called after the
    /// main UI render and before `Compositor::resolve`.
    pub(crate) fn render_overlays(&mut self, compositor: &mut Compositor, theme: &Theme) {
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;

        if self.overlay_mgr.is_empty() {
            return;
        }

        log::info!("render_overlays: {} active, layer={:?}", self.overlay_mgr.len(), self.overlay_layer);
        let layer_id = self.overlay_layer;

        for overlay in &self.overlay_mgr.stack.clone() {
            match &overlay.kind {
                OverlayKind::ContextMenu { items } => {
                    let (w, h, item_rects) = context_menu::draw(
                        compositor,
                        layer_id,
                        theme,
                        overlay.x,
                        overlay.y,
                        &items,
                        self.hover_overlay_item,
                    );
                    self.ctx_menu_item_rects = item_rects;
                    // Update bounds if not yet known
                    if overlay.w == 0.0 {
                        self.overlay_mgr.set_bounds(overlay.id, w, h);
                    }
                }
                OverlayKind::Modal { title, body, confirm, cancel } => {
                    let (confirm_rect, cancel_rect) = modal::draw(
                        compositor,
                        layer_id,
                        theme,
                        self.vw,
                        self.vh,
                        overlay.x,
                        overlay.y,
                        &title,
                        &body,
                        &confirm,
                        &cancel,
                        self.hover_modal_confirm,
                        self.hover_modal_cancel,
                    );
                    self.modal_confirm_rect = Some(confirm_rect);
                    self.modal_cancel_rect = Some(cancel_rect);
                }
                OverlayKind::Tooltip { text } => {
                    // Simple tooltip: text in a small rounded rect
                    compositor.push_to_layer(layer_id, SceneNode::RoundedRect {
                        x: overlay.x,
                        y: overlay.y,
                        w: text.len() as f32 * 7.5 + 16.0,
                        h: 26.0,
                        color: theme.bg_3.to_array(),
                        corner_radius: theme.radius_s,
                        border_width: 1.0,
                        border_color: theme.border.to_array(),
                    });
                    compositor.push_to_layer(layer_id, SceneNode::Text {
                        key: plev::compositor::TextNodeKey::new(text, 12.0, 16.0, None),
                        x: overlay.x + 8.0,
                        y: overlay.y + 5.0,
                        color: theme.text_2.to_array(),
                    });
                }
            }
        }
    }

    /// Handle hover for overlay items. Returns true if hover state changed.
    pub fn handle_overlay_hover(&mut self, cx: f32, cy: f32) -> bool {
        let old_item = self.hover_overlay_item;
        let old_confirm = self.hover_modal_confirm;
        let old_cancel = self.hover_modal_cancel;

        self.hover_overlay_item = None;
        self.hover_modal_confirm = false;
        self.hover_modal_cancel = false;

        for (i, &(rx, ry, rw, rh)) in self.ctx_menu_item_rects.iter().enumerate() {
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                self.hover_overlay_item = Some(i);
                break;
            }
        }
        if let Some((rx, ry, rw, rh)) = self.modal_confirm_rect {
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                self.hover_modal_confirm = true;
            }
        }
        if let Some((rx, ry, rw, rh)) = self.modal_cancel_rect {
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                self.hover_modal_cancel = true;
            }
        }

        old_item != self.hover_overlay_item
            || old_confirm != self.hover_modal_confirm
            || old_cancel != self.hover_modal_cancel
    }
}
