//! Event processing and FPS tracking for ShowcaseState.

use crate::input::{InputEvent, InputState, PressState};

use super::ShowcaseState;

impl ShowcaseState {
    pub fn process_events(&mut self, input_state: &mut InputState) {
        for event in input_state.drain_events() {
            match event {
                InputEvent::Click(click) => {
                    if click.state == PressState::Pressed
                        && let Some(btn_id) = self.btn_view_id
                        && click.view_id == btn_id
                    {
                        self.click_count += 1;
                    }
                }
                InputEvent::Hover(hover) => {
                    if let Some(btn_id) = self.btn_view_id
                        && hover.view_id == btn_id
                    {
                        self.btn_hovered = hover.entered;
                    }
                }
                _ => {}
            }
        }
    }

    /// Update FPS counter. Called at the start of each build_scene.
    pub(crate) fn tick_fps(&mut self) {
        self.frame += 1;
        self.fps_frame_count += 1;
        let now = web_time::Instant::now();
        let elapsed = now.duration_since(self.fps_last_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.fps_display = self.fps_frame_count as f32 / elapsed;
            self.fps_frame_count = 0;
            self.fps_last_time = now;
        }
    }
}
