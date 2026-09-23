//! Pointer and key handling of the workspace. What only moves the person
//! around (hover, the list cursor, the context menu, the overlays, focus
//! and the caret) happens here; what acts on a capability (a diff, stage,
//! discard, the commit form, a commit) is queued as a [`UiRequest`] for
//! the workbench to ask through its dispatcher.

use super::super::commit_form::CommitFormAction;
use super::{PendingAction, UiRequest, WorkspaceView};
use comps::prelude::WidgetEvent;

impl WorkspaceView {
    /// Handle a left click at (cx, cy). Returns true if state changed.
    pub(crate) fn handle_click(&mut self, cx: f32, cy: f32) -> bool {
        // Overlays intercept clicks first
        if !self.overlay_mgr.is_empty() {
            return self.handle_overlay_click(cx, cy);
        }
        let sidebar_w = self.sidebar_w();
        let header_h = self.header_h();
        let handle_w = self.handle_w();
        // The chrome is laid out from the frame origin.
        let (lx, ly) = (cx - self.origin.0, cy - self.origin.1);

        // Sidebar clicks: a press and a release on the same link.
        if lx < sidebar_w && ly >= header_h {
            let theme = self.theme().clone();
            self.sidebar.handle_event(
                &WidgetEvent::MouseDown { x: cx, y: cy },
                &theme,
                self.vw,
                self.vh,
            );
            let (_, tab) = self.sidebar.handle_event(
                &WidgetEvent::MouseUp { x: cx, y: cy },
                &theme,
                self.vw,
                self.vh,
            );
            return tab.is_some();
        }

        // Header clicks
        if ly < header_h {
            if self.header.hit_test_theme_btn(cx, cy) {
                self.toggle_theme();
                return true;
            }
            return false;
        }

        // Commit form clicks
        match self.commit_form.hit_test_click(cx, cy) {
            CommitFormAction::Commit => return self.submit_commit(),
            CommitFormAction::Cancel => return self.cancel_commit_form(),
            CommitFormAction::None => {}
        }

        // The commit field takes focus and caret placement.
        if self.commit_form.visible {
            let theme = self.theme().clone();
            let r = self
                .commit_form
                .handle_event(&WidgetEvent::MouseDown { x: cx, y: cy }, &theme);
            if r.clicked {
                return true;
            }
        }

        let (left_x, right_x) = self.panel_bounds();
        let mid_x = left_x + self.left_w + handle_w;

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
            // Click in right panel (diff): no action for now
        } else if cx > mid_x && cx < right_x - handle_w {
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
        let handle_w = self.handle_w();
        let mid_x = left_x + self.left_w + handle_w;

        let old_file = self.hover_unassigned_row;
        let old_commit = self.hover_stack_commit;

        if cx >= left_x && cx < left_x + self.left_w {
            self.hover_unassigned_row = self.unassigned.hit_test(cx, cy);
            self.hover_stack_commit = None;
        } else if cx > mid_x && cx < right_x - handle_w {
            self.hover_unassigned_row = None;
            self.hover_stack_commit = self.stacks.hit_test(cx, cy);
        } else {
            self.hover_unassigned_row = None;
            self.hover_stack_commit = None;
        }

        // The retained chrome widgets track their own hover.
        let theme = self.theme().clone();
        let event = WidgetEvent::MouseMove { x: cx, y: cy };
        let mut widget_changed = self.header.handle_event(&event).changed;
        widget_changed |= self.commit_form.handle_event(&event, &theme).changed;
        widget_changed |= self
            .sidebar
            .handle_event(&event, &theme, self.vw, self.vh)
            .0
            .changed;

        widget_changed
            || old_file != self.hover_unassigned_row
            || old_commit != self.hover_stack_commit
    }

    /// Handle a right-click at (cx, cy). Returns true if an overlay was opened.
    pub(crate) fn handle_right_click(&mut self, cx: f32, cy: f32) -> bool {
        use comps::overlay::MenuItem;

        // dismiss any existing overlay first
        self.overlay_mgr.pop_all();
        self.ctx_menu = None;
        self.modal = None;
        self.ctx_menu_item_rects.clear();
        self.modal_confirm_rect = None;
        self.modal_cancel_rect = None;

        let (left_x, _) = self.panel_bounds();

        // Right-click on left panel (unassigned files)
        if cx >= left_x
            && cx < left_x + self.left_w
            && let Some(idx) = self.unassigned.hit_test(cx, cy)
            && let Some(file) = self.unassigned.files.get(idx)
        {
            let (path, staged) = (file.path.clone(), file.staged);
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
            let nudge = self.theme().spacing.xs / 2.0;
            self.open_context_menu(cx + nudge, cy, items);
            self.pending_action = Some(PendingAction::ContextMenu { path, staged });
            // Also select the row
            if self.unassigned.select(Some(idx)) {
                self.open_file_diff(idx);
            }
            return true;
        }
        false
    }

    // -- keys and commands (the keymap, the host's key routing, tests) -------

    /// Closes the topmost overlay. Returns true if one was open.
    pub(crate) fn close_top_overlay(&mut self) -> bool {
        if self.overlay_mgr.is_empty() {
            return false;
        }
        self.overlay_mgr.pop();
        if self.overlay_mgr.is_empty() {
            self.ctx_menu = None;
            self.modal = None;
            self.ctx_menu_item_rects.clear();
            self.modal_confirm_rect = None;
            self.modal_cancel_rect = None;
            self.pending_action = None;
        }
        true
    }

    /// Moves the file cursor up and asks for that file's diff.
    pub(crate) fn nav_up(&mut self) -> bool {
        let changed = self.unassigned.select_prev();
        if changed {
            self.show_selected_diff();
        }
        changed
    }

    /// Moves the file cursor down and asks for that file's diff.
    pub(crate) fn nav_down(&mut self) -> bool {
        let changed = self.unassigned.select_next();
        if changed {
            self.show_selected_diff();
        }
        changed
    }

    /// Asks for the diff of the file under the cursor.
    pub(crate) fn show_selected_diff(&mut self) -> bool {
        if let Some(idx) = self.unassigned.selected_idx {
            self.open_file_diff(idx);
            true
        } else {
            false
        }
    }

    /// Asks to stage the file under the cursor.
    pub(crate) fn stage_selected(&mut self) -> bool {
        let Some(path) = self.unassigned.selected_path() else {
            return false;
        };
        let path = path.to_owned();
        self.requests.push(UiRequest::Stage { path });
        true
    }

    /// Asks the person to confirm before discarding the file under the
    /// cursor.
    pub(crate) fn discard_selected(&mut self) -> bool {
        let Some(path) = self.unassigned.selected_path() else {
            return false;
        };
        let path = path.to_owned();
        self.open_discard_modal(path);
        true
    }

    /// Asks to open the commit form, or to cancel it when open.
    pub(crate) fn toggle_commit_form(&mut self) -> bool {
        self.requests.push(if self.commit_form.visible {
            UiRequest::CancelForm
        } else {
            UiRequest::OpenForm
        });
        true
    }

    /// Asks to cancel the commit form (clearing its message).
    pub(crate) fn cancel_commit_form(&mut self) -> bool {
        self.requests.push(UiRequest::CancelForm);
        true
    }

    /// Asks for a commit of the staged files with the form's message. The
    /// dispatcher decides: a closed form, a blank message or an empty
    /// index refuse it.
    pub(crate) fn submit_commit(&mut self) -> bool {
        self.requests.push(UiRequest::Submit);
        true
    }

    /// Asks for the diff of file row `idx`.
    pub(crate) fn open_file_diff(&mut self, idx: usize) {
        if let Some(file) = self.unassigned.files.get(idx) {
            self.requests.push(UiRequest::ShowFileDiff {
                path: file.path.clone(),
            });
        }
    }

    /// Asks for the diff of commit `ci` of stack `si`.
    pub(crate) fn open_commit_diff(&mut self, si: usize, ci: usize) {
        if let Some(commit) = self.stacks.stacks.get(si).and_then(|s| s.commits.get(ci)) {
            self.requests.push(UiRequest::ShowCommitDiff {
                sha: commit.sha.clone(),
            });
        }
    }
}
