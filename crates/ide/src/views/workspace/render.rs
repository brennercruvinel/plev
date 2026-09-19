use super::WorkspaceView;
use engine::compositor::{Compositor, SceneNode};

impl WorkspaceView {
    /// Full render: clears and rebuilds everything.
    pub fn render(&mut self, compositor: &mut Compositor) {
        self.ensure_overlay_layer(compositor);

        let theme = self.theme().clone();
        let vw = self.vw;
        let vh = self.vh;
        let sidebar_w = self.sidebar_w();
        let header_h = self.header_h();
        let handle_w = self.handle_w();

        compositor.begin_frame();

        // Page canvas behind the app.
        compositor.push(SceneNode::Rect {
            x: 0.0,
            y: 0.0,
            w: vw,
            h: vh,
            color: theme.colors.bg.0,
        });

        // The header spans the window; the rail sits under it.
        self.header.render(
            compositor,
            &theme,
            self.theme_mode,
            vw,
            sidebar_w,
            &self.repo_label,
            &self.branch_label,
        );
        self.sidebar.render(compositor, &theme, vw, vh);

        let content_y = header_h;
        let content_h = vh - header_h;

        // -- Left panel (Unassigned Changes) --
        let left_x = sidebar_w;
        let mid_x = left_x + self.left_w + handle_w;
        let right_x = vw - self.right_w;

        // Commit form (inline, above file list)
        let commit_form_h =
            self.commit_form
                .render(compositor, &theme, left_x, content_y, self.left_w);

        let hover_row = self.hover_unassigned_row;
        self.unassigned.render(
            compositor,
            &theme,
            left_x,
            content_y + commit_form_h,
            self.left_w,
            content_h - commit_form_h,
            hover_row,
        );

        // Left resize handle: page seam at rest, the field focus tone
        // when grabbed.
        let handle_hov = self.hover_left_handle || self.dragging_left;
        compositor.push(SceneNode::Rect {
            x: left_x + self.left_w,
            y: content_y,
            w: handle_w,
            h: content_h,
            color: if handle_hov {
                theme.glass.field_focus_border.0
            } else {
                theme.colors.bg.0
            },
        });

        // -- Middle panel (Stacks) --
        let mid_w = right_x - mid_x;
        let hover_commit = self.hover_stack_commit;
        self.stacks.render(
            compositor,
            &theme,
            mid_x,
            content_y,
            mid_w.max(0.0),
            content_h,
            hover_commit,
        );

        // Right resize handle.
        let right_handle_hov = self.hover_right_handle || self.dragging_right;
        compositor.push(SceneNode::Rect {
            x: right_x - handle_w,
            y: content_y,
            w: handle_w,
            h: content_h,
            color: if right_handle_hov {
                theme.glass.field_focus_border.0
            } else {
                theme.colors.bg.0
            },
        });

        // -- Right panel (Diff) --
        self.diff.render(
            compositor,
            &theme,
            right_x,
            content_y,
            self.right_w,
            content_h,
        );

        // -- Overlays (always last, highest z_order) --
        self.render_overlays(compositor, &theme);
    }
}
