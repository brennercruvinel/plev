//! the view drawn into a compositor: the shell (the header with the
//! instant, the list of requests), the frame on screen at the instant fit
//! into its rect with the outline of every control over it, the strip,
//! and the panel. the recorded frame is a picture here: it binds nothing
//! in this compositor, so no recorded control ever answers a hit test as a
//! live one. every color is the theme's or the recording's own.

use engine::widgets::Rect;
use engine::widgets::overlay::OverlayManager;
use engine::widgets::recipe::{focus_ring, rounded_rect_stroke};
use engine::drawing::compositor::{Compositor, LayerId, SceneNode};

use crate::view::InspectorView;

impl InspectorView {
    pub fn render(&mut self, c: &mut Compositor) {
        c.begin_frame();
        let overlay = match self.overlay.filter(|id| c.layer(*id).is_some()) {
            Some(id) => id,
            None => {
                let id = c.create_layer(OverlayManager::BASE_Z);
                self.overlay = Some(id);
                id
            }
        };
        let theme = self.theme.clone();
        let layout = self.layout();
        self.shell.render(c, LayerId::DEFAULT, overlay, &theme);
        self.render_frame(c, layout.frame);
        self.strip.render(c, layout.strip, &theme, self.at());
        if self.panel.is_some() {
            let max = self.block.max_scroll(layout.panel, &theme);
            self.block.scroll = self.block.scroll.clamp(0.0, max);
            self.block.render(c, layout.panel, &theme);
        } else {
            self.empty.render(c, layout.panel, &theme);
        }
        let (vw, vh) = self.size();
        self.tip.render(c, overlay, &theme, vw, vh);
    }

    fn render_frame(&self, c: &mut Compositor, into: Rect) {
        let theme = &self.theme;
        let Some(fit) = self.fit() else {
            self.frame_note(c, into);
            return;
        };
        let frame = fit.frame();
        c.push(SceneNode::PushClip {
            x: frame.x,
            y: frame.y,
            w: frame.w,
            h: frame.h,
        });
        for node in &self.shown.scene {
            if let Some(node) = fit.node(node) {
                c.push(node);
            }
        }
        c.push(SceneNode::PopClip);
        let radius = theme.shape.micro;
        for b in &self.shown.bound {
            let r = fit.rect(b.rect);
            let (color, width) = if self.hover.as_ref() == Some(&b.id) {
                (theme.colors.accent.0, theme.control.edge_width_strong)
            } else {
                (theme.glass.edge.0, theme.control.edge_width)
            };
            c.push(rounded_rect_stroke(
                r.x, r.y, r.w, r.h, radius, color, width,
            ));
        }
        if let Some(r) = self.picked().and_then(|id| self.control_rect(id)) {
            c.push(focus_ring(r, radius, theme));
        }
        c.push(rounded_rect_stroke(
            frame.x,
            frame.y,
            frame.w,
            frame.h,
            0.0,
            theme.glass.edge_soft.0,
            theme.control.edge_width,
        ));
        if self.shown.error.is_some() {
            self.frame_note(c, into);
        }
    }

    /// what stands in the frame's place when there is no frame to show.
    fn frame_note(&self, c: &mut Compositor, into: Rect) {
        let (title, message) = match &self.shown.error {
            Some(e) => ("the frame does not read back", e.clone()),
            None => (
                "no frame at this instant",
                "the session holds no frame at or before it".to_owned(),
            ),
        };
        engine::widgets::content::EmptyState::new(title, message).render(c, into, &self.theme);
    }
}
