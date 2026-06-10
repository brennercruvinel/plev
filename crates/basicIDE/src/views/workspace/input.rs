use super::super::commit_form::CommitFormAction;
use super::{HEADER_H, PendingAction, RESIZE_HANDLE_W, SIDEBAR_W, UiRequest, WorkspaceView};

impl WorkspaceView {
    /// Handle a left click at (cx, cy). Returns true if state changed.
    pub fn handle_click(&mut self, cx: f32, cy: f32) -> bool {
        // Overlays intercept clicks first
        if !self.overlay_mgr.is_empty() {
            return self.handle_overlay_click(cx, cy);
        }
        // Sidebar clicks
        if cx < SIDEBAR_W {
            if let Some(tab) = self.sidebar.hit_test(cx, cy) {
                self.sidebar.active = tab;
                return true;
            }
            return false;
        }

        // Header clicks
        if cy < HEADER_H {
            if self.header.hit_test_theme_btn(cx, cy) {
                self.toggle_theme();
                return true;
            }
            return false;
        }

        // Commit form clicks
        match self.commit_form.hit_test_click(cx, cy) {
            CommitFormAction::Commit => {
                return self.submit_commit();
            }
            CommitFormAction::Cancel => {
                self.commit_form.hide();
                return true;
            }
            CommitFormAction::None => {}
        }

        let (left_x, right_x) = self.panel_bounds();
        let mid_x = left_x + self.left_w + RESIZE_HANDLE_W;

        if cx < left_x + self.left_w {
            // Click in left panel (unassigned files)
            if let Some(idx) = self.unassigned.hit_test(cx, cy) {
                let changed = self.unassigned.select(Some(idx));
                if changed {
                    self.open_file_diff(idx);
                }
                return changed;
            }
        } else if cx > right_x {
            // Click in right panel (diff) — no action for now
        } else if cx > mid_x && cx < right_x - RESIZE_HANDLE_W {
            // Click in middle panel (stacks)
            if let Some((si, ci)) = self.stacks.hit_test(cx, cy) {
                let changed = self.stacks.select(Some((si, ci)));
                if changed {
                    self.open_commit_diff(si, ci);
                }
                return changed;
            }
        }
        false
    }

    /// Handle hover at (cx, cy). Returns true if hover state changed.
    pub fn handle_hover(&mut self, cx: f32, cy: f32) -> bool {
        let (left_x, right_x) = self.panel_bounds();
        let mid_x = left_x + self.left_w + RESIZE_HANDLE_W;

        let old_file = self.hover_unassigned_row;
        let old_commit = self.hover_stack_commit;

        if cx >= left_x && cx < left_x + self.left_w {
            self.hover_unassigned_row = self.unassigned.hit_test(cx, cy);
            self.hover_stack_commit = None;
        } else if cx > mid_x && cx < right_x - RESIZE_HANDLE_W {
            self.hover_unassigned_row = None;
            self.hover_stack_commit = self.stacks.hit_test(cx, cy);
        } else {
            self.hover_unassigned_row = None;
            self.hover_stack_commit = None;
        }

        old_file != self.hover_unassigned_row || old_commit != self.hover_stack_commit
    }

    /// Handle a right-click at (cx, cy). Returns true if an overlay was opened.
    pub fn handle_right_click(&mut self, cx: f32, cy: f32) -> bool {
        use plev::overlay::{MenuItem, OverlayKind};

        // dismiss any existing overlay first
        self.overlay_mgr.pop_all();
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;

        let (left_x, _) = self.panel_bounds();

        // Right-click on left panel (unassigned files)
        if cx >= left_x && cx < left_x + self.left_w {
            if let Some(idx) = self.unassigned.hit_test(cx, cy) {
                let staged = self.unassigned.files[idx].staged;
                let items = vec![
                    if staged {
                        MenuItem::new("Unstage file", 0)
                    } else {
                        MenuItem::new("Stage file", 0)
                    },
                    MenuItem::new("Discard changes", 1),
                    MenuItem::new("Ignore file", 2),
                ];
                // Offset slightly so the menu doesn't sit right under the cursor
                self.overlay_mgr
                    .push(OverlayKind::ContextMenu { items }, cx + 2.0, cy, 0.0, 0.0);
                self.pending_action = Some(PendingAction::ContextMenu { file_idx: idx });
                // Also select the row
                if self.unassigned.select(Some(idx)) {
                    self.open_file_diff(idx);
                }
                return true;
            }
        }
        false
    }

    // -- semantic actions (invoked by the keymap dispatcher and tests) -------

    /// Closes the topmost overlay. Returns true if one was open.
    pub fn close_top_overlay(&mut self) -> bool {
        if self.overlay_mgr.is_empty() {
            return false;
        }
        self.overlay_mgr.pop();
        if self.overlay_mgr.is_empty() {
            self.ctx_menu_item_rects.clear();
            self.modal_confirm_rect = None;
            self.modal_cancel_rect = None;
            self.pending_action = None;
        }
        true
    }

    /// Moves the file selection up and refreshes the diff panel.
    pub fn nav_up(&mut self) -> bool {
        let changed = self.unassigned.select_prev();
        if changed {
            self.show_selected_diff();
        }
        changed
    }

    /// Moves the file selection down and refreshes the diff panel.
    pub fn nav_down(&mut self) -> bool {
        let changed = self.unassigned.select_next();
        if changed {
            self.show_selected_diff();
        }
        changed
    }

    /// Requests the diff of the currently selected file.
    pub fn show_selected_diff(&mut self) -> bool {
        if let Some(idx) = self.unassigned.selected_idx {
            self.open_file_diff(idx);
            true
        } else {
            false
        }
    }

    /// Shows/hides the commit form.
    pub fn toggle_commit_form(&mut self) -> bool {
        if self.commit_form.visible {
            self.commit_form.hide();
        } else {
            self.commit_form.show();
        }
        true
    }

    /// Submits the commit form: queues the real commit and hides the form
    /// (the status/log refresh arrives via git events). No-op while empty.
    pub fn submit_commit(&mut self) -> bool {
        if !self.commit_form.visible || self.commit_form.message.is_empty() {
            return false;
        }
        let message = std::mem::take(&mut self.commit_form.message);
        self.requests.push(UiRequest::Commit { message });
        self.commit_form.hide();
        true
    }

    /// Points the diff panel at file `idx` and queues the real diff fetch.
    pub(crate) fn open_file_diff(&mut self, idx: usize) {
        if let Some(file) = self.unassigned.files.get(idx) {
            self.diff.show_file(&file.path);
            self.requests.push(UiRequest::FileDiff {
                path: file.path.clone(),
            });
        }
    }

    /// Points the diff panel at a commit and queues the real diff fetch.
    pub(crate) fn open_commit_diff(&mut self, si: usize, ci: usize) {
        if let Some(commit) = self.stacks.stacks.get(si).and_then(|s| s.commits.get(ci)) {
            self.diff.show_commit(&commit.message, &commit.sha);
            self.requests.push(UiRequest::CommitDiff {
                sha: commit.sha.clone(),
            });
        }
    }
}
