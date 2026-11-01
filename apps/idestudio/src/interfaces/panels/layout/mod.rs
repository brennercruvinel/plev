mod input;
mod overlays;
mod render;
#[cfg(test)]
mod tests;

use super::commit_form::CommitForm;
use super::diff::DiffView;
use super::header::Header;
use super::sidebar::Sidebar;
use super::stacks::MultiStackView;
use super::unassigned::UnassignedView;
use crate::capability::ids::Ids;
use accord::RequestId;
use engine::drawing::compositor::{Compositor, LayerId};
use engine::styling::theme::{Intent, Theme};
use engine::widgets::feedback::{ContextMenu, Modal, ToastManager};
use engine::widgets::overlay::OverlayManager;
use engine::widgets::prelude::Wake;

/// Three-panel workspace layout — mirrors plev ide's MainViewport.
pub struct WorkspaceView {
    /// EFFECTIVE left panel width used by layout/render — re-derived from
    /// `left_w_desired` on every resize/drag (clamped to available space).
    pub left_w: f32,
    /// EFFECTIVE right panel width (see `left_w`).
    pub right_w: f32,
    /// Width the USER wants for the left panel (default or last drag).
    /// Never overwritten by window shrinking: growing the window back
    /// restores the panel to this width.
    left_w_desired: f32,
    /// Width the user wants for the right panel (see `left_w_desired`).
    right_w_desired: f32,
    pub theme_mode: ThemeMode,
    /// The two themes the toggle switches between.
    themes: [Theme; 2],

    // Chrome
    pub sidebar: Sidebar,
    pub header: Header,

    // Views
    pub unassigned: UnassignedView,
    pub stacks: MultiStackView,
    pub diff: DiffView,
    pub commit_form: CommitForm,

    // Interaction state
    pub dragging_left: bool,
    pub dragging_right: bool,
    pub(crate) drag_start_x: f32,
    pub(crate) drag_start_w: f32,

    // Hover state
    pub hover_left_handle: bool,
    pub hover_right_handle: bool,
    pub hover_unassigned_row: Option<usize>,
    pub hover_stack_commit: Option<(usize, usize)>,
    pub hover_overlay_item: Option<usize>,
    pub hover_modal_confirm: bool,
    pub hover_modal_cancel: bool,

    // Overlay system
    pub overlay_mgr: OverlayManager,
    pub(crate) overlay_layer: LayerId,
    /// The design-system widgets behind the open overlays (built when an
    /// overlay is pushed, dropped when it closes).
    pub(crate) ctx_menu: Option<ContextMenu>,
    pub(crate) modal: Option<Modal>,
    // Cached hit rects for overlay interaction
    pub(crate) ctx_menu_item_rects: Vec<(f32, f32, f32, f32)>,
    pub(crate) modal_confirm_rect: Option<(f32, f32, f32, f32)>,
    pub(crate) modal_cancel_rect: Option<(f32, f32, f32, f32)>,
    /// What the open overlay refers to (context menu target / discard
    /// confirmation). `None` when no overlay-driven action is in flight.
    pub(crate) pending_action: Option<PendingAction>,
    /// What the dispatcher refused and why, as toasts over the frame
    /// (`notices[<id>]`).
    pub(crate) toasts: ToastManager,

    /// What the person asked for since the last drain; the workbench
    /// turns each into a request through its dispatcher (views never
    /// change git state or the projected state themselves).
    pub(crate) requests: Vec<UiRequest>,

    /// Repo name + branch shown in the header (injected by the app).
    pub repo_label: String,
    pub branch_label: String,

    // Frame the workspace lays out in: `vw`/`vh` is its size, `origin`
    // its top-left in window coordinates (zero when it owns the window).
    pub vw: f32,
    pub vh: f32,
    pub origin: (f32, f32),
    /// Embedded in a host that provides its own header (the showcase):
    /// the header is skipped; the rail and the three panels fill the
    /// frame from `origin`.
    pub embedded: bool,

    /// The logical identities the views bind and the workbench projects;
    /// `None` only if the constant ids failed to parse (a test proves they
    /// do not), which leaves the workspace without semantics.
    pub(crate) ids: Option<Ids>,
}

/// Overlay-driven interaction state: which file the open context menu or
/// discard-confirmation modal is about, by path (a refresh may reorder
/// the rows while the overlay is open).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PendingAction {
    /// The menu of a row: its path and whether the row is the staged one.
    ContextMenu {
        path: String,
        staged: bool,
    },
    ConfirmDiscard {
        path: String,
    },
    /// Another actor's request that waits for a person to confirm it.
    ConfirmRequest {
        request: RequestId,
    },
}

/// What a person asked for through the workspace (a click, a key, a menu
/// item), named without ids. The views queue it; the workbench observes
/// the workspace at that instant and asks its dispatcher as the person,
/// so the effect happens once, in `execute`, whoever asked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiRequest {
    ShowFileDiff {
        path: String,
    },
    ShowCommitDiff {
        sha: String,
    },
    Stage {
        path: String,
    },
    Unstage {
        path: String,
    },
    Discard {
        path: String,
    },
    Ignore {
        path: String,
    },
    OpenForm,
    CancelForm,
    Submit,
    /// A click on a toast.
    DismissNotice {
        id: u64,
    },
    /// The person confirmed a request waiting for it (the modal's confirm).
    Confirm {
        request: RequestId,
    },
    /// The person closed that modal without confirming.
    Decline {
        request: RequestId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Side {
    Left,
    Right,
}

impl WorkspaceView {
    pub fn new(vw: f32, vh: f32) -> Self {
        // Overlay layer is created lazily on first render via `ensure_overlay_layer`.
        // Use a sentinel LayerId so the default layer is never accidentally used.
        let (left_default, right_default) = Self::panel_defaults(&Theme::hoff());
        let mut view = Self {
            left_w: left_default,
            right_w: right_default,
            left_w_desired: left_default,
            right_w_desired: right_default,
            theme_mode: ThemeMode::Dark,
            themes: [Theme::hoff(), Theme::hoff_light()],
            sidebar: Sidebar::new(),
            header: Header::new(),
            unassigned: UnassignedView::new(),
            stacks: MultiStackView::new(),
            diff: DiffView::new(),
            commit_form: CommitForm::new(),
            dragging_left: false,
            dragging_right: false,
            drag_start_x: 0.0,
            drag_start_w: 0.0,
            hover_left_handle: false,
            hover_right_handle: false,
            hover_unassigned_row: None,
            hover_stack_commit: None,
            hover_overlay_item: None,
            hover_modal_confirm: false,
            hover_modal_cancel: false,
            overlay_mgr: OverlayManager::new(),
            overlay_layer: LayerId::DEFAULT, // replaced on first render
            ctx_menu: None,
            modal: None,
            ctx_menu_item_rects: Vec::new(),
            modal_confirm_rect: None,
            modal_cancel_rect: None,
            pending_action: None,
            toasts: ToastManager::new(),
            requests: Vec::new(),
            repo_label: String::new(),
            branch_label: String::new(),
            vw,
            vh,
            origin: (0.0, 0.0),
            embedded: false,
            ids: Ids::new()
                .inspect_err(|e| log::error!("ide: the workspace ids do not parse: {e}"))
                .ok(),
        };
        if let Some(ids) = &view.ids {
            view.unassigned
                .set_sem(ids.changes.clone(), ids.file.clone());
            view.commit_form
                .set_sem(ids.form.clone(), ids.message.clone(), ids.submit.clone());
            view.diff.set_sem(ids.diff.clone());
            view.stacks.set_sem(ids.stacks.clone(), ids.commit.clone());
            view.toasts = ToastManager::new().sem(ids.notices.clone());
        }
        view.apply_panel_widths();
        view
    }

    /// Whether a commit can be asked for now: the form is open, the
    /// message is not blank and something is staged. The Commit button
    /// draws from this and `commit.submit` projects it.
    pub fn can_submit(&self) -> bool {
        self.commit_form.visible
            && !self.commit_form.message().trim().is_empty()
            && self.unassigned.files.iter().any(|f| f.staged)
    }

    /// Embed inside a host's content rect: no header, rail and panels
    /// laid out from `origin`. Input arrives in window coordinates and
    /// is resolved against the same origin.
    pub fn set_frame(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.embedded = true;
        self.origin = (x, y);
        self.sidebar.set_origin(x, y);
        self.resize(w, h);
    }

    /// Bottom-right corner of the frame in window coordinates: the
    /// viewport overlays center in and cover (dimming the host too).
    pub(crate) fn viewport(&self) -> (f32, f32) {
        (self.origin.0 + self.vw, self.origin.1 + self.vh)
    }

    /// Takes what the person asked for since the last drain.
    pub(crate) fn take_requests(&mut self) -> Vec<UiRequest> {
        std::mem::take(&mut self.requests)
    }

    /// Create the overlay layer once after the Compositor is available.
    /// Called from `render()` on the first frame.
    pub(crate) fn ensure_overlay_layer(&mut self, compositor: &mut Compositor) {
        if self.overlay_layer == LayerId::DEFAULT {
            self.overlay_layer = compositor.create_layer(OverlayManager::BASE_Z);
        }
    }

    pub fn theme(&self) -> &Theme {
        match self.theme_mode {
            ThemeMode::Dark => &self.themes[0],
            ThemeMode::Light => &self.themes[1],
        }
    }

    /// Sidebar rail width for the current theme.
    pub(crate) fn sidebar_w(&self) -> f32 {
        self.sidebar.width(self.theme())
    }

    /// Header height for the current theme.
    pub(crate) fn header_h(&self) -> f32 {
        if self.embedded {
            return 0.0;
        }
        Header::height(self.theme())
    }

    /// Resize handle width: the divider hit width.
    pub(crate) fn handle_w(&self) -> f32 {
        self.theme().control.divider_hit
    }

    /// Shows `message` as a toast at the bottom-right of the frame (what
    /// the dispatcher refused, a confirmation left unanswered).
    pub(crate) fn notify(&mut self, message: impl Into<String>, intent: Intent) {
        let theme = self.theme().clone();
        self.toasts.push(message, intent, &theme);
    }

    /// Advance the widgets that animate (scrollbar fades, the commit
    /// field caret, the toasts) and merge what they ask of the loop: a
    /// frame on every turn while something moves; otherwise the earliest
    /// deadline (the caret's next flip, the instant a toast dismisses
    /// itself); nothing when all rest.
    pub fn wake(&mut self, dt: f32) -> Wake {
        self.unassigned
            .wake(dt)
            .merge(self.stacks.wake(dt))
            .merge(self.diff.wake(dt))
            .merge(self.commit_form.wake(dt))
            .merge(self.toasts.wake(dt))
    }

    /// Draws with a host's theme in the slot of `mode`: a page that
    /// embeds the workspace hands it its own, so the workspace follows
    /// the page around it (a light one in the light slot).
    pub fn use_host_theme(&mut self, theme: Theme, mode: ThemeMode) {
        let slot = match mode {
            ThemeMode::Dark => 0,
            ThemeMode::Light => 1,
        };
        self.themes[slot] = theme;
        self.theme_mode = mode;
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    /// Update window dimensions. Effective panel widths are re-derived
    /// from the user-desired ones, so shrinking the window squeezes the
    /// panels and growing it back restores them.
    pub fn resize(&mut self, vw: f32, vh: f32) {
        self.vw = vw;
        self.vh = vh;
        self.apply_panel_widths();
    }

    /// Begin resizing the left panel. `cursor_x` is current cursor X.
    pub fn begin_drag_left(&mut self, cursor_x: f32) {
        self.dragging_left = true;
        self.drag_start_x = cursor_x;
        self.drag_start_w = self.left_w;
    }

    /// Begin resizing the right panel. `cursor_x` is current cursor X.
    pub fn begin_drag_right(&mut self, cursor_x: f32) {
        self.dragging_right = true;
        self.drag_start_x = cursor_x;
        self.drag_start_w = self.right_w;
    }

    pub fn end_drag(&mut self) {
        self.dragging_left = false;
        self.dragging_right = false;
    }

    /// Update panel widths based on cursor drag. Dragging expresses user
    /// intent: it rewrites the DESIRED width; the effective width is then
    /// re-derived (clamped) like on any resize.
    pub fn update_drag(&mut self, cursor_x: f32) {
        let delta = cursor_x - self.drag_start_x;
        if self.dragging_left {
            let (left_min, _, _) = self.panel_minimums();
            self.left_w_desired = (self.drag_start_w + delta).max(left_min);
            self.apply_panel_widths();
        } else if self.dragging_right {
            let (_, right_min, _) = self.panel_minimums();
            self.right_w_desired = (self.drag_start_w - delta).max(right_min);
            self.apply_panel_widths();
        }
    }

    /// Derive the EFFECTIVE panel widths from the desired ones:
    /// `effective = clamp(desired, available space)`, computed fresh each
    /// time. The desired widths are never touched here — the old code
    /// shrank `left_w`/`right_w` in place on every window shrink and could
    /// never grow them back (destructive loss of the user's layout).
    fn apply_panel_widths(&mut self) {
        let (left_min, right_min, middle_min) = self.panel_minimums();
        let usable_w = self.vw - self.sidebar_w();
        let available = usable_w - middle_min;
        let mut left = self.left_w_desired;
        let mut right = self.right_w_desired;
        if left + right > available {
            let ratio = available / (left + right);
            left *= ratio;
            right *= ratio;
        }
        self.left_w = left.max(left_min);
        self.right_w = right.max(right_min);
    }

    /// Narrowest each column may get: the changes column and the stacks
    /// feed fit a card, the diff column fits a code line.
    fn panel_minimums(&self) -> (f32, f32, f32) {
        let size = &self.theme().size;
        (
            size.field_min_w + size.field_min_w / 4.0,
            size.card_min_w - size.field_min_w / 4.0,
            size.field_min_w + size.field_min_w / 4.0,
        )
    }

    /// Default column widths: the sidebar width for the changes column,
    /// the card width for the diff column.
    fn panel_defaults(theme: &Theme) -> (f32, f32) {
        (
            theme.size.sidebar_w + theme.spacing.xxl,
            theme.size.card_w - theme.spacing.xxl,
        )
    }

    /// Handle mouse scroll for the hovered panel. Returns `true` when an
    /// offset actually moved (callers may invalidate unconditionally; the
    /// return value keeps the routing testable).
    pub fn scroll(&mut self, cursor_x: f32, delta: f32) -> bool {
        let (left_x, right_x) = self.panel_bounds();
        if cursor_x < left_x {
            // The sidebar rail does not scroll any panel.
            return false;
        }
        let (target, moved) = if cursor_x < left_x + self.left_w {
            let old = self.unassigned.scroll.offset();
            self.unassigned.scroll.scroll_by(delta);
            (0, self.unassigned.scroll.offset() != old)
        } else if cursor_x > right_x {
            let old = self.diff.scroll.offset();
            self.diff.scroll.scroll_by(delta);
            (2, self.diff.scroll.offset() != old)
        } else {
            let old = self.stacks.scroll.offset();
            self.stacks.scroll.scroll_by(delta);
            (1, self.stacks.scroll.offset() != old)
        };
        // The scrollbar fades in on activity.
        match target {
            0 => self.unassigned.notify_scroll(),
            1 => self.stacks.notify_scroll(),
            _ => self.diff.notify_scroll(),
        }
        moved
    }

    /// Returns (left_panel_x, right_panel_x) in window coordinates,
    /// accounting for the sidebar and the frame origin.
    pub(crate) fn panel_bounds(&self) -> (f32, f32) {
        let left_x = self.origin.0 + self.sidebar_w();
        let right_x = self.origin.0 + self.vw - self.right_w;
        (left_x, right_x)
    }

    /// Hit-test resize handles. Returns Some(Side) if cursor is on a handle.
    pub fn hit_test_handle(&self, cursor_x: f32) -> Option<Side> {
        let handle_w = self.handle_w();
        let (left_x, right_x) = self.panel_bounds();
        let left_handle_x = left_x + self.left_w;
        let right_handle_x = right_x - handle_w / 2.0;

        if (cursor_x - left_handle_x).abs() < handle_w * 2.0 {
            Some(Side::Left)
        } else if (cursor_x - right_handle_x).abs() < handle_w * 2.0 {
            Some(Side::Right)
        } else {
            None
        }
    }
}
