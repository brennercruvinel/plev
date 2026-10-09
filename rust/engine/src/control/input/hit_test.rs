use super::types::*;

impl super::InputState {
    pub(crate) fn hit_test(&self, x: f32, y: f32) -> Option<ViewId> {
        self.hit_regions
            .iter()
            .rev()
            .find(|r| {
                r.layer_visible
                    && r.layer_opacity > 0.0
                    && x >= r.x
                    && x <= r.x + r.w
                    && y >= r.y
                    && y <= r.y + r.h
            })
            .map(|r| r.view_id)
    }

    pub(crate) fn hit_test_focusable(&self, x: f32, y: f32) -> Option<ViewId> {
        self.hit_regions
            .iter()
            .rev()
            .find(|r| {
                r.focusable
                    && r.layer_visible
                    && r.layer_opacity > 0.0
                    && x >= r.x
                    && x <= r.x + r.w
                    && y >= r.y
                    && y <= r.y + r.h
            })
            .map(|r| r.view_id)
    }
}
