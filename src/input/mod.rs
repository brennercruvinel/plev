pub mod gesture;
pub mod touch;
mod types;

mod handlers;
mod hit_test;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_hit;

pub use types::*;

pub struct InputState {
    next_id: u64,
    hit_regions: Vec<HitRegion>,
    cursor_position: Option<(f32, f32)>,
    focused_view: Option<ViewId>,
    hovered_view: Option<ViewId>,
    modifiers: ModifierState,
    pending_events: Vec<InputEvent>,
    current_layer_visible: bool,
    current_layer_opacity: f32,
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

impl InputState {
    pub fn new() -> Self {
        Self {
            next_id: 0,
            hit_regions: Vec::new(),
            cursor_position: None,
            focused_view: None,
            hovered_view: None,
            modifiers: ModifierState::default(),
            pending_events: Vec::new(),
            current_layer_visible: true,
            current_layer_opacity: 1.0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.hit_regions.clear();
        self.next_id = 0;
        self.current_layer_visible = true;
        self.current_layer_opacity = 1.0;
    }

    pub fn next_view_id(&mut self) -> ViewId {
        let id = self.next_id;
        self.next_id += 1;
        ViewId(id)
    }

    pub fn set_current_layer(&mut self, visible: bool, opacity: f32) {
        self.current_layer_visible = visible;
        self.current_layer_opacity = opacity;
    }

    pub fn register_hit_region(
        &mut self,
        view_id: ViewId,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        focusable: bool,
    ) {
        self.hit_regions.push(HitRegion {
            view_id,
            x,
            y,
            w,
            h,
            focusable,
            layer_visible: self.current_layer_visible,
            layer_opacity: self.current_layer_opacity,
        });
    }

    pub fn drain_events(&mut self) -> Vec<InputEvent> {
        std::mem::take(&mut self.pending_events)
    }

    pub fn focused_view(&self) -> Option<ViewId> {
        self.focused_view
    }
    pub fn hovered_view(&self) -> Option<ViewId> {
        self.hovered_view
    }
    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.cursor_position
    }
    pub fn hit_regions(&self) -> &[HitRegion] {
        &self.hit_regions
    }

    pub fn set_focused_view(&mut self, view_id: Option<ViewId>) {
        self.focused_view = view_id;
    }
}
