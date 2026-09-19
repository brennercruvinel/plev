use super::{PendingAction, UiRequest, WorkspaceView};
use comps::feedback::{ContextMenu, MenuEntry, Modal};
use comps::overlay::{MenuItem, OverlayKind};
use comps::prelude::{Rect, WidgetEvent, rounded_rect, shadow_node};
use engine::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::text::TextMeasurer;
use engine::theme::{Intent, Theme};

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
                    && let Some(path) = self.remove_file_row(file_idx)
                {
                    self.requests.push(UiRequest::Discard { path });
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
                        self.ctx_menu = None;
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

    /// Opens the file context menu at `(x, y)`: the overlay entry for the
    /// manager and the design-system widget that draws and hit-tests it.
    pub(crate) fn open_context_menu(&mut self, x: f32, y: f32, items: Vec<MenuItem>) {
        let entries = items
            .iter()
            .map(|item| {
                let entry = MenuEntry::item(item.id, item.label.clone());
                if item.label.starts_with("Discard") {
                    entry.intent(Intent::Destructive).icon("trash")
                } else if item.label.starts_with("Ignore") {
                    entry.icon("eye")
                } else {
                    entry.icon("check")
                }
            })
            .collect();
        self.ctx_menu = Some(ContextMenu::new(entries));
        self.overlay_mgr
            .push(OverlayKind::ContextMenu { items }, x, y, 0.0, 0.0);
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
        let title = "Discard changes?".to_string();
        let body = format!("Discard all changes to {file_name}? This cannot be undone.");
        let modal = Modal::new(title.clone(), body.clone(), "Discard", "Cancel")
            .intent(Intent::Destructive);
        let dialog = modal.dialog_rect(self.theme(), self.vw, self.vh);
        self.modal = Some(modal);
        self.overlay_mgr.push(
            OverlayKind::Modal {
                title,
                body,
                confirm: "Discard".into(),
                cancel: "Cancel".into(),
            },
            dialog.x,
            dialog.y,
            dialog.w,
            dialog.h,
        );
    }

    /// Closes every overlay and clears the cached interaction state.
    fn dismiss_overlays(&mut self) {
        self.overlay_mgr.pop_all();
        self.ctx_menu = None;
        self.modal = None;
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
                    let menu = self.ctx_menu.get_or_insert_with(|| {
                        ContextMenu::new(
                            items
                                .iter()
                                .map(|i| MenuEntry::item(i.id, i.label.clone()))
                                .collect(),
                        )
                    });
                    let (w, h) = menu.size(theme);
                    menu.render(compositor, layer_id, theme, overlay.x, overlay.y);
                    // Item rows only (separators are not targets).
                    self.ctx_menu_item_rects = menu
                        .entry_rects(overlay.x, overlay.y, theme)
                        .into_iter()
                        .zip(&menu.entries)
                        .filter(|(_, e)| matches!(e, MenuEntry::Item { .. }))
                        .map(|(r, _)| (r.x, r.y, r.w, r.h))
                        .collect();
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
                    let modal = self.modal.get_or_insert_with(|| {
                        Modal::new(title.clone(), body.clone(), confirm.clone(), cancel.clone())
                            .intent(Intent::Destructive)
                    });
                    modal.render(compositor, layer_id, theme, self.vw, self.vh);
                    let dialog = modal.dialog_rect(theme, self.vw, self.vh);
                    let (confirm_rect, cancel_rect) = modal.button_rects(theme, dialog);
                    self.modal_confirm_rect = Some((
                        confirm_rect.x,
                        confirm_rect.y,
                        confirm_rect.w,
                        confirm_rect.h,
                    ));
                    self.modal_cancel_rect =
                        Some((cancel_rect.x, cancel_rect.y, cancel_rect.w, cancel_rect.h));
                }
                OverlayKind::Tooltip { text } => {
                    // Tooltip at the overlay origin: hint shadow, tooltip
                    // body at the tooltip radius, caption-r text-secondary.
                    let style = theme.typography.caption_r();
                    let pad_x = theme.spacing.md;
                    let pad_y = theme.spacing.xs;
                    let (tw, _) = TextMeasurer::measure_styled(text, &style, None);
                    let rect = Rect::new(
                        overlay.x,
                        overlay.y,
                        tw + pad_x * 2.0,
                        style.line_height + pad_y * 2.0,
                    );
                    let radius = theme.shape.tooltip;
                    if let Some(shadow) = shadow_node(rect, radius, &theme.shadows.hint) {
                        compositor.push_to_layer(layer_id, shadow);
                    }
                    compositor.push_to_layer(
                        layer_id,
                        rounded_rect(
                            rect.x,
                            rect.y,
                            rect.w,
                            rect.h,
                            radius,
                            theme.glass.tooltip.0,
                        ),
                    );
                    compositor.push_to_layer(
                        layer_id,
                        SceneNode::Text {
                            key: TextNodeKey::from_style(text, &style, None),
                            x: rect.x + pad_x,
                            y: rect.y + pad_y,
                            color: theme.colors.text_mid.0,
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
        if let Some((rx, ry, rw, rh)) = self.modal_confirm_rect
            && cx >= rx
            && cx <= rx + rw
            && cy >= ry
            && cy <= ry + rh
        {
            self.hover_modal_confirm = true;
        }
        if let Some((rx, ry, rw, rh)) = self.modal_cancel_rect
            && cx >= rx
            && cx <= rx + rw
            && cy >= ry
            && cy <= ry + rh
        {
            self.hover_modal_cancel = true;
        }

        // The widgets keep their own hover for the rendering.
        let event = WidgetEvent::MouseMove { x: cx, y: cy };
        let theme = self.theme().clone();
        let mut widget_changed = false;
        if let Some(menu) = &mut self.ctx_menu
            && let Some(overlay) = self
                .overlay_mgr
                .stack
                .iter()
                .find(|o| matches!(o.kind, OverlayKind::ContextMenu { .. }))
        {
            widget_changed |= menu
                .handle_event(&event, overlay.x, overlay.y, &theme)
                .0
                .changed;
        }
        if let Some(modal) = &mut self.modal {
            widget_changed |= modal
                .handle_event(&event, &theme, self.vw, self.vh)
                .1
                .changed;
        }

        widget_changed
            || old_item != self.hover_overlay_item
            || old_confirm != self.hover_modal_confirm
            || old_cancel != self.hover_modal_cancel
    }
}
