use super::super::commit_form::CommitFormAction;
use super::{HEADER_H, RESIZE_HANDLE_W, SIDEBAR_W, WorkspaceView};

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
                // Mock commit: just hide the form
                self.commit_form.hide();
                return true;
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
                    if let Some(file) = self.unassigned.files.get(idx) {
                        self.diff.set_file(&file.path, file.status);
                    }
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
                    if let Some(commit) = self.stacks.stacks.get(si).and_then(|s| s.commits.get(ci))
                    {
                        self.diff.set_commit(&commit.message, &commit.sha);
                    }
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

        log::info!("right-click at ({cx:.0}, {cy:.0})");
        // dismiss any existing overlay first
        self.overlay_mgr.pop_all();
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;

        let (left_x, _) = self.panel_bounds();

        // Right-click on left panel (unassigned files)
        if cx >= left_x && cx < left_x + self.left_w {
            if let Some(idx) = self.unassigned.hit_test(cx, cy) {
                let path = self.unassigned.files[idx].path.clone();
                let items = vec![
                    MenuItem::new("Stage file", 0),
                    MenuItem::new("Discard changes", 1),
                    MenuItem::new("Ignore file", 2),
                ];
                // Offset slightly so the menu doesn't sit right under the cursor
                let id = self.overlay_mgr.push(
                    OverlayKind::ContextMenu { items },
                    cx + 2.0,
                    cy,
                    0.0,
                    0.0,
                );
                log::info!("opened context menu id={:?} for file[{idx}]={path}", id);
                // Store which file this menu is for
                self.pending_discard_idx = Some(idx);
                let _ = id;
                // Also select the row
                self.unassigned.select(Some(idx));
                if let Some(file) = self.unassigned.files.get(idx) {
                    self.diff.set_file(&file.path, file.status);
                }
                let _ = path;
                return true;
            }
        }
        false
    }

    /// Handle key press. Returns true if state changed.
    pub fn handle_key_down(&mut self, key: &winit::keyboard::Key) -> bool {
        use winit::keyboard::{Key, NamedKey};

        // Escape closes the topmost overlay before anything else
        if let Key::Named(NamedKey::Escape) = key {
            if !self.overlay_mgr.is_empty() {
                self.overlay_mgr.pop();
                if self.overlay_mgr.is_empty() {
                    self.ctx_menu_item_rects.clear();
                    self.modal_confirm_rect = None;
                    self.modal_cancel_rect = None;
                    self.pending_discard_idx = None;
                }
                return true;
            }
        }

        // If commit form is active, route text input there
        if self.commit_form.visible {
            match key {
                Key::Named(NamedKey::Escape) => {
                    self.commit_form.hide();
                    return true;
                }
                Key::Named(NamedKey::Backspace) => {
                    self.commit_form.backspace();
                    return true;
                }
                Key::Named(NamedKey::Enter) => {
                    if !self.commit_form.message.is_empty() {
                        self.commit_form.hide();
                        return true;
                    }
                    return false;
                }
                Key::Character(c) => {
                    for ch in c.chars() {
                        self.commit_form.append_char(ch);
                    }
                    return true;
                }
                _ => return false,
            }
        }

        match key {
            Key::Named(NamedKey::ArrowUp) => self.unassigned.select_prev(),
            Key::Named(NamedKey::ArrowDown) => self.unassigned.select_next(),
            Key::Named(NamedKey::Enter) => {
                if let Some(file) = self.unassigned.selected_file() {
                    self.diff.set_file(&file.path, file.status);
                    true
                } else {
                    false
                }
            }
            Key::Character(c) if c == "c" || c == "C" => {
                // 'C' toggles commit mode
                if self.commit_form.visible {
                    self.commit_form.hide();
                } else {
                    self.commit_form.show();
                }
                true
            }
            _ => false,
        }
    }
}
