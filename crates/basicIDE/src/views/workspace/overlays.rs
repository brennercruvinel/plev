use super::{PendingAction, UiRequest, WorkspaceView};
use crate::components::{context_menu, modal};
use crate::theme::Theme;
use plev::compositor::{Compositor, SceneNode};
use plev::overlay::OverlayKind;

impl WorkspaceView {
    /// Handle a click when overlays are active. Returns true if consumed.
    ///
    /// Mutations update the view optimistically (the row flips/disappears
    /// right away) and queue the real git operation in `self.requests`; the
    /// follow-up status refresh reconciles with reality.
    pub(crate) fn handle_overlay_click(&mut self, cx: f32, cy: f32) -> bool {
        if self.overlay_mgr.is_empty() {
            return false;
        }

        // Check modal buttons first (modal is always on top when present)
        if let Some(confirm_rect) = self.modal_confirm_rect {
            let (rx, ry, rw, rh) = confirm_rect;
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                // Confirmed -> discard the file
                if let Some(PendingAction::ConfirmDiscard { file_idx }) = self.pending_action.take()
                {
                    if let Some(path) = self.remove_file_row(file_idx) {
                        self.requests.push(UiRequest::Discard { path });
                    }
                }
                self.dismiss_overlays();
                return true;
            }
        }
        if let Some(cancel_rect) = self.modal_cancel_rect {
            let (rx, ry, rw, rh) = cancel_rect;
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                self.dismiss_overlays();
                return true;
            }
        }

        // Check context menu items
        for (i, &(rx, ry, rw, rh)) in self.ctx_menu_item_rects.iter().enumerate() {
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                let file_idx = self
                    .pending_action
                    .map(PendingAction::file_idx)
                    .unwrap_or(0);
                match i {
                    0 => {
                        // Stage/Unstage -- flip the staged flag in place
                        if let Some(file) = self.unassigned.files.get_mut(file_idx) {
                            file.staged = !file.staged;
                            let path = file.path.clone();
                            self.requests.push(if file.staged {
                                UiRequest::Stage { path }
                            } else {
                                UiRequest::Unstage { path }
                            });
                        }
                        self.dismiss_overlays();
                    }
                    1 => {
                        // Discard -- show confirmation modal
                        self.overlay_mgr.pop_all(); // close context menu first
                        self.ctx_menu_item_rects.clear();
                        self.open_discard_modal(file_idx);
                    }
                    2 => {
                        // Ignore -- remove from list, append to .gitignore
                        if let Some(path) = self.remove_file_row(file_idx) {
                            self.requests.push(UiRequest::Ignore { path });
                        }
                        self.dismiss_overlays();
                    }
                    _ => {}
                }
                return true;
            }
        }

        // Click outside all overlays -- dismiss
        if self.overlay_mgr.hit_test_outside(cx, cy) {
            self.dismiss_overlays();
            return true;
        }

        // Click inside an overlay but not on an item -- consume without action
        true
    }

    /// Opens the discard confirmation modal for file `file_idx` (destructive
    /// operations always confirm). The actual discard happens on confirm.
    pub(crate) fn open_discard_modal(&mut self, file_idx: usize) {
        self.pending_action = Some(PendingAction::ConfirmDiscard { file_idx });
        let file_name = self
            .unassigned
            .files
            .get(file_idx)
            .map(|f| f.path.rsplit('/').next().unwrap_or(&f.path).to_string())
            .unwrap_or_default();
        let (mx, my) = modal::centered_pos(self.vw, self.vh);
        let (mw, mh) = modal::dimensions();
        self.overlay_mgr.push(
            OverlayKind::Modal {
                title: "Discard changes?".into(),
                body: format!(
                    "Discard all changes to {}? This cannot be undone.",
                    file_name
                ),
                confirm: "Discard".into(),
                cancel: "Cancel".into(),
            },
            mx,
            my,
            mw,
            mh,
        );
    }

    /// Closes every overlay and clears the cached interaction state.
    fn dismiss_overlays(&mut self) {
        self.overlay_mgr.pop_all();
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;
        self.pending_action = None;
    }

    /// Optimistically removes a file row (discard/ignore), fixing up the
    /// selection and clearing the diff if it pointed at the removed row.
    /// Returns the removed path.
    fn remove_file_row(&mut self, idx: usize) -> Option<String> {
        if idx >= self.unassigned.files.len() {
            return None;
        }
        let entry = self.unassigned.files.remove(idx);
        match self.unassigned.selected_idx {
            Some(sel) if sel == idx => {
                self.unassigned.selected_idx = None;
                self.diff.clear();
            }
            Some(sel) if sel > idx => self.unassigned.selected_idx = Some(sel - 1),
            _ => {}
        }
        Some(entry.path)
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

        log::info!(
            "render_overlays: {} active, layer={:?}",
            self.overlay_mgr.len(),
            self.overlay_layer
        );
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
                OverlayKind::Modal {
                    title,
                    body,
                    confirm,
                    cancel,
                } => {
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
                    compositor.push_to_layer(
                        layer_id,
                        SceneNode::RoundedRect {
                            x: overlay.x,
                            y: overlay.y,
                            w: text.len() as f32 * 7.5 + 16.0,
                            h: 26.0,
                            color: theme.bg_3.to_array(),
                            corner_radius: theme.radius_s,
                            border_width: 1.0,
                            border_color: theme.border.to_array(),
                        },
                    );
                    compositor.push_to_layer(
                        layer_id,
                        SceneNode::Text {
                            key: plev::compositor::TextNodeKey::new(text, 12.0, 16.0, None),
                            x: overlay.x + 8.0,
                            y: overlay.y + 5.0,
                            color: theme.text_2.to_array(),
                        },
                    );
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
