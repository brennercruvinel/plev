use super::{PendingAction, UiRequest, WorkspaceView};
use accord::RequestId;
use engine::drawing::compositor::{Compositor, SceneNode, TextNodeKey};
use engine::drawing::text::TextMeasurer;
use engine::styling::theme::{Intent, Theme};
use engine::widgets::feedback::{ContextMenu, MenuEntry, Modal};
use engine::widgets::overlay::{MenuItem, OverlayKind};
use engine::widgets::prelude::{Rect, WidgetEvent, rounded_rect, shadow_node};

impl WorkspaceView {
    /// Handle a click when overlays are active. Returns true if consumed.
    ///
    /// A menu item or a confirmed modal queues what the person asked for
    /// in `self.requests`; the workbench runs it through the dispatcher,
    /// where the view is updated right away and the status refresh after
    /// git answers reconciles it.
    pub(crate) fn handle_overlay_click(&mut self, cx: f32, cy: f32) -> bool {
        if self.overlay_mgr.is_empty() {
            return false;
        }

        // Check modal buttons first (modal is always on top when present)
        if let Some(confirm_rect) = self.modal_confirm_rect {
            let (rx, ry, rw, rh) = confirm_rect;
            if cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh {
                // Confirmed: ask to discard the file, or confirm the
                // request that waits for a person.
                match self.pending_action.take() {
                    Some(PendingAction::ConfirmDiscard { path }) => {
                        self.requests.push(UiRequest::Discard { path });
                    }
                    Some(PendingAction::ConfirmRequest { request }) => {
                        self.requests.push(UiRequest::Confirm { request });
                    }
                    _ => {}
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
        let hit = self
            .ctx_menu_item_rects
            .iter()
            .position(|&(rx, ry, rw, rh)| cx >= rx && cx <= rx + rw && cy >= ry && cy <= ry + rh);
        if let Some(i) = hit {
            let Some(PendingAction::ContextMenu { path, staged }) = self.pending_action.clone()
            else {
                self.dismiss_overlays();
                return true;
            };
            match i {
                0 => {
                    // Stage or unstage, as the row the menu opened on reads.
                    self.requests.push(if staged {
                        UiRequest::Unstage { path }
                    } else {
                        UiRequest::Stage { path }
                    });
                    self.dismiss_overlays();
                }
                1 => {
                    // Discard -- show confirmation modal
                    self.overlay_mgr.pop_all(); // close context menu first
                    self.ctx_menu = None;
                    self.ctx_menu_item_rects.clear();
                    self.open_discard_modal(path);
                }
                2 => {
                    // Ignore -- append to .gitignore
                    self.requests.push(UiRequest::Ignore { path });
                    self.dismiss_overlays();
                }
                _ => {}
            }
            return true;
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

    /// Opens the discard confirmation modal for `path`: a person confirms
    /// on screen before asking to discard. The discard is asked on
    /// confirm.
    pub(crate) fn open_discard_modal(&mut self, path: String) {
        let file_name = path.rsplit('/').next().unwrap_or(&path).to_string();
        let body = format!("Discard all changes to {file_name}? This cannot be undone.");
        self.open_modal(
            PendingAction::ConfirmDiscard { path },
            "Discard changes?".into(),
            body,
            "Discard",
        );
    }

    /// Opens the modal that asks the person to confirm `request`, which
    /// another actor made and the dispatcher holds until a person answers.
    /// Any menu open closes first. Confirm answers through the dispatcher
    /// (`UiRequest::Confirm`); closing the modal any other way declines.
    pub(crate) fn open_confirm_modal(
        &mut self,
        request: RequestId,
        title: String,
        body: String,
        confirm: &str,
    ) {
        self.overlay_mgr.pop_all();
        self.ctx_menu = None;
        self.ctx_menu_item_rects.clear();
        self.pending_action = None;
        self.open_modal(
            PendingAction::ConfirmRequest { request },
            title,
            body,
            confirm,
        );
    }

    /// Closes the modal that asks about `request`, if it is the one open:
    /// the request was confirmed (the answer is not a decline).
    pub(crate) fn close_confirm_modal(&mut self, request: RequestId) {
        if self.pending_action == Some(PendingAction::ConfirmRequest { request }) {
            self.pending_action = None;
            self.dismiss_overlays();
        }
    }

    /// Whether a modal holds the screen (a confirmation is asked).
    pub fn modal_open(&self) -> bool {
        self.modal.is_some()
    }

    /// The open modal's confirm and cancel buttons as last drawn
    /// (`[x, y, w, h]` in window coordinates), for a host or a test that
    /// points at them.
    pub fn modal_buttons(&self) -> Option<([f32; 4], [f32; 4])> {
        let rect = |(x, y, w, h): (f32, f32, f32, f32)| [x, y, w, h];
        Some((
            rect(self.modal_confirm_rect?),
            rect(self.modal_cancel_rect?),
        ))
    }

    fn open_modal(&mut self, pending: PendingAction, title: String, body: String, confirm: &str) {
        self.pending_action = Some(pending);
        let modal =
            Modal::new(title.clone(), body.clone(), confirm, "Cancel").intent(Intent::Destructive);
        let (vw, vh) = self.viewport();
        let dialog = modal.dialog_rect(self.theme(), vw, vh);
        self.modal = Some(modal);
        self.overlay_mgr.push(
            OverlayKind::Modal {
                title,
                body,
                confirm: confirm.into(),
                cancel: "Cancel".into(),
            },
            dialog.x,
            dialog.y,
            dialog.w,
            dialog.h,
        );
    }

    /// Closes every overlay and clears the cached interaction state. A
    /// request that waited for the person's confirmation is declined.
    pub(super) fn dismiss_overlays(&mut self) {
        if let Some(PendingAction::ConfirmRequest { request }) = self.pending_action.take() {
            self.requests.push(UiRequest::Decline { request });
        }
        self.overlay_mgr.pop_all();
        self.ctx_menu = None;
        self.modal = None;
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;
    }

    /// Draws the toasts over everything, at the bottom-right of the frame,
    /// and binds each one as `notices[<id>]`.
    pub(crate) fn render_toasts(&self, compositor: &mut Compositor, theme: &Theme) {
        let (vw, vh) = self.viewport();
        self.toasts
            .render(compositor, self.overlay_layer, theme, vw, vh);
    }

    /// The toast under `(x, y)` that is not already leaving.
    pub(crate) fn toast_at(&self, x: f32, y: f32) -> Option<u64> {
        let (vw, vh) = self.viewport();
        let rects = self.toasts.visible_rects(self.theme(), vw, vh);
        self.toasts
            .visible()
            .zip(rects)
            .find(|(t, r)| !t.is_closing() && r.contains(x, y))
            .map(|(t, _)| t.id())
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
                    let (vw, vh) = self.viewport();
                    let modal = self.modal.get_or_insert_with(|| {
                        Modal::new(title.clone(), body.clone(), confirm.clone(), cancel.clone())
                            .intent(Intent::Destructive)
                    });
                    modal.render(compositor, layer_id, theme, vw, vh);
                    let dialog = modal.dialog_rect(theme, vw, vh);
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
                // The ide pushes no anchored panel; its owner would draw it.
                OverlayKind::Anchored => {}
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
