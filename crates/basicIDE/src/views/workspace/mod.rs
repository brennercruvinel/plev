mod input;
mod overlays;
mod render;
#[cfg(test)]
mod tests;

use super::commit_form::CommitForm;
use super::diff_view::DiffView;
use super::header::Header;
use super::multi_stack_view::MultiStackView;
use super::sidebar::SIDEBAR_W;
use super::sidebar::Sidebar;
use super::unassigned_view::UnassignedView;
use plev::compositor::{Compositor, LayerId};
use plev::dispatch::ActionQueue;
use plev::overlay::OverlayManager;

pub(crate) const HEADER_H: f32 = super::header::HEADER_H;
pub(crate) const RESIZE_HANDLE_W: f32 = 4.0;
const LEFT_MIN_W: f32 = 200.0;
const LEFT_DEFAULT_W: f32 = 280.0;
const RIGHT_DEFAULT_W: f32 = 340.0;
const RIGHT_MIN_W: f32 = 220.0;

/// Three-panel workspace layout — mirrors basicIDE's MainViewport.
pub struct WorkspaceView {
    pub left_w: f32,
    pub right_w: f32,
    pub theme_mode: ThemeMode,

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
    pub action_queue: ActionQueue,
    pub(crate) overlay_layer: LayerId,
    // Cached hit rects for overlay interaction
    pub(crate) ctx_menu_item_rects: Vec<(f32, f32, f32, f32)>,
    pub(crate) modal_confirm_rect: Option<(f32, f32, f32, f32)>,
    pub(crate) modal_cancel_rect: Option<(f32, f32, f32, f32)>,
    // Index of file pending discard confirmation
    pub(crate) pending_discard_idx: Option<usize>,

    // Window size
    pub vw: f32,
    pub vh: f32,
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
        Self {
            left_w: LEFT_DEFAULT_W,
            right_w: RIGHT_DEFAULT_W,
            theme_mode: ThemeMode::Dark,
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
            action_queue: ActionQueue::new(),
            overlay_layer: LayerId::DEFAULT, // replaced on first render
            ctx_menu_item_rects: Vec::new(),
            modal_confirm_rect: None,
            modal_cancel_rect: None,
            pending_discard_idx: None,
            vw,
            vh,
        }
    }

    /// Create the overlay layer once after the Compositor is available.
    /// Called from `render()` on the first frame.
    pub(crate) fn ensure_overlay_layer(&mut self, compositor: &mut Compositor) {
        if self.overlay_layer == LayerId::DEFAULT {
            self.overlay_layer = compositor.create_layer(OverlayManager::BASE_Z);
        }
    }

    pub fn theme(&self) -> &'static crate::theme::Theme {
        match self.theme_mode {
            ThemeMode::Dark => &crate::theme::DARK,
            ThemeMode::Light => &crate::theme::LIGHT,
        }
    }

    pub fn toggle_theme(&mut self) {
        self.theme_mode = match self.theme_mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    /// Update window dimensions.
    pub fn resize(&mut self, vw: f32, vh: f32) {
        self.vw = vw;
        self.vh = vh;
        self.clamp_panel_widths();
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

    /// Update panel widths based on cursor drag.
    pub fn update_drag(&mut self, cursor_x: f32) {
        let delta = cursor_x - self.drag_start_x;
        if self.dragging_left {
            self.left_w = (self.drag_start_w + delta).max(LEFT_MIN_W);
            self.clamp_panel_widths();
        } else if self.dragging_right {
            self.right_w = (self.drag_start_w - delta).max(RIGHT_MIN_W);
            self.clamp_panel_widths();
        }
    }

    fn clamp_panel_widths(&mut self) {
        let middle_min = 200.0;
        let usable_w = self.vw - SIDEBAR_W;
        let available = usable_w - middle_min;
        if self.left_w + self.right_w > available {
            let ratio = available / (self.left_w + self.right_w);
            self.left_w *= ratio;
            self.right_w *= ratio;
        }
        self.left_w = self.left_w.max(LEFT_MIN_W);
        self.right_w = self.right_w.max(RIGHT_MIN_W);
    }

    /// Handle mouse scroll for the hovered panel.
    pub fn scroll(&mut self, cursor_x: f32, delta: f32) {
        let (left_x, right_x) = self.panel_bounds();
        if cursor_x < left_x + self.left_w {
            self.unassigned.scroll.scroll_by(delta);
        } else if cursor_x > right_x {
            self.diff.scroll.scroll_by(delta);
        } else {
            self.stacks.scroll.scroll_by(delta);
        }
    }

    /// Returns (left_panel_x, right_panel_x) accounting for sidebar.
    pub(crate) fn panel_bounds(&self) -> (f32, f32) {
        let left_x = SIDEBAR_W;
        let right_x = self.vw - self.right_w;
        (left_x, right_x)
    }

    /// Hit-test resize handles. Returns Some(Side) if cursor is on a handle.
    pub fn hit_test_handle(&self, cursor_x: f32) -> Option<Side> {
        let left_handle_x = SIDEBAR_W + self.left_w;
        let right_handle_x = self.vw - self.right_w - RESIZE_HANDLE_W / 2.0;

        if (cursor_x - left_handle_x).abs() < RESIZE_HANDLE_W * 2.0 {
            Some(Side::Left)
        } else if (cursor_x - right_handle_x).abs() < RESIZE_HANDLE_W * 2.0 {
            Some(Side::Right)
        } else {
            None
        }
    }
}
