use super::{HEADER_H, RESIZE_HANDLE_W, SIDEBAR_W, ThemeMode, WorkspaceView};
use plev::compositor::{Compositor, SceneNode};

impl WorkspaceView {
    /// Full render — clears and rebuilds everything.
    pub fn render(&mut self, compositor: &mut Compositor) {
        self.ensure_overlay_layer(compositor);

        let theme = match self.theme_mode {
            ThemeMode::Dark => &crate::theme::DARK,
            ThemeMode::Light => &crate::theme::LIGHT,
        };
        let vw = self.vw;
        let vh = self.vh;

        compositor.begin_frame();

        // Global frame behind the app — body #444444.
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: vh,
            color: theme.bg_body.to_array(),
        });

        // -- Sidebar --
        self.sidebar.render(compositor, theme, vh, HEADER_H);

        // -- Header --
        self.header.render(
            compositor,
            theme,
            self.theme_mode,
            vw,
            SIDEBAR_W,
            &self.repo_label,
            &self.branch_label,
        );

        let content_y = HEADER_H;
        let content_h = vh - HEADER_H;

        // -- Left panel (Unassigned Changes) --
        let left_x = SIDEBAR_W;
        let mid_x = left_x + self.left_w + RESIZE_HANDLE_W;
        let right_x = vw - self.right_w;

        // Commit form (inline, above file list)
        let commit_form_h =
            self.commit_form
                .render(compositor, theme, left_x, content_y, self.left_w);

        let hover_row = self.hover_unassigned_row;
        self.unassigned.render(
            compositor,
            theme,
            left_x,
            content_y + commit_form_h,
            self.left_w,
            content_h - commit_form_h,
            hover_row,
        );

        // Left resize handle — dark seam at rest, rgba($n2,.25) when grabbed.
        let handle_hov = self.hover_left_handle || self.dragging_left;
        compositor.push(SceneNode::Rect {
            x: left_x + self.left_w,
            y: content_y,
            w: RESIZE_HANDLE_W,
            h: content_h,
            color: if handle_hov {
                theme.field_focus_border.to_array()
            } else {
                theme.bg_body.to_array()
            },
        });

        // -- Middle panel (Stacks) --
        let mid_w = right_x - mid_x;
        let hover_commit = self.hover_stack_commit;
        self.stacks.render(
            compositor,
            theme,
            mid_x,
            content_y,
            mid_w.max(0.0),
            content_h,
            hover_commit,
        );

        // Right resize handle — dark seam at rest, rgba($n2,.25) when grabbed.
        let right_handle_hov = self.hover_right_handle || self.dragging_right;
        compositor.push(SceneNode::Rect {
            x: right_x - RESIZE_HANDLE_W,
            y: content_y,
            w: RESIZE_HANDLE_W,
            h: content_h,
            color: if right_handle_hov {
                theme.field_focus_border.to_array()
            } else {
                theme.bg_body.to_array()
            },
        });

        // -- Right panel (Diff) --
        self.diff.render(
            compositor,
            theme,
            right_x,
            content_y,
            self.right_w,
            content_h,
        );

        // -- Overlays (always last, highest z_order) --
        self.render_overlays(compositor, theme);
    }
}
