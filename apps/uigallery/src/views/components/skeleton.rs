//! Skeleton page: disc, lines and block placeholders. They breathe only
//! while toggled on (a press on the row), so an idle page settles.

use comps::content::{Skeleton, SkeletonShape};
use comps::prelude::{EventResult, Rect, WidgetEvent};
use engine::compositor::Compositor;
use engine::theme::{ControlSize, Theme};

pub struct SkeletonPage {
    skeletons: [Skeleton; 3],
    breathing: bool,
}

impl SkeletonPage {
    pub fn new() -> Self {
        Self {
            skeletons: [
                Skeleton::new(SkeletonShape::Disc),
                Skeleton::new(SkeletonShape::Lines).lines(3),
                Skeleton::new(SkeletonShape::Block),
            ],
            breathing: false,
        }
    }

    fn rects(&self, content: Rect, theme: &Theme) -> [Rect; 3] {
        let gap = theme.spacing.md;
        let disc = theme.control.height(ControlSize::Md);
        let lines_h = self.skeletons[1].preferred_height(theme).unwrap_or(disc);
        let block_w = ((content.w - disc - gap * 2.0) / 2.0).max(0.0);
        [
            Rect::new(content.x, content.y, disc, disc),
            Rect::new(content.x + disc + gap, content.y, block_w, lines_h),
            Rect::new(
                content.x + disc + gap + block_w + gap,
                content.y,
                block_w,
                lines_h,
            ),
        ]
    }

    pub fn content_height(&self, content: Rect, theme: &Theme) -> f32 {
        self.rects(content, theme)
            .iter()
            .map(|r| r.h)
            .fold(0.0, f32::max)
    }

    pub fn handle_event(
        &mut self,
        event: &WidgetEvent,
        content: Rect,
        theme: &Theme,
    ) -> EventResult {
        if let WidgetEvent::MouseDown { x, y } = *event
            && self.rects(content, theme).iter().any(|r| r.contains(x, y))
        {
            self.breathing = !self.breathing;
            return EventResult::changed();
        }
        EventResult::IGNORED
    }

    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.breathing {
            return false;
        }
        // with reduced motion on each stands still and asks for nothing;
        // every one is ticked (no short circuit)
        let mut moved = false;
        for s in &mut self.skeletons {
            moved |= s.tick(dt);
        }
        moved
    }

    pub fn render(&self, c: &mut Compositor, content: Rect, theme: &Theme) {
        for (sk, rect) in self.skeletons.iter().zip(self.rects(content, theme)) {
            sk.render(c, rect, theme);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    /// Started, the page asks for every frame; with the system's "reduce
    /// motion" on (task-75 phase 2a) it asks for none, and asks again once
    /// the preference is off.
    #[test]
    fn the_page_asks_for_no_frame_with_reduced_motion() {
        let mut page = SkeletonPage::new();
        assert!(!page.tick(FRAME), "at rest until the person starts it");
        page.breathing = true;
        assert!(page.tick(FRAME), "with motion");
        engine::animation::set_reduced_motion(true);
        let moved = page.tick(FRAME);
        engine::animation::set_reduced_motion(false);
        assert!(!moved, "reduced: it stands still");
        assert!(page.tick(FRAME), "motion again");
    }
}
