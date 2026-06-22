// App state and gesture processing logic.

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::input::{GestureEvent, Phase, SwipeDirection, TouchInputState};
use plev::text::TextSystem;
use plev::winit::window::Window;

pub(crate) enum State {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct TouchDemoApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: State,
    pub(crate) compositor: Compositor,
    pub(crate) touch_input: TouchInputState,
    pub(crate) rect_x: f32,
    pub(crate) rect_y: f32,
    pub(crate) rect_w: f32,
    pub(crate) rect_h: f32,
    pub(crate) rect_scale: f32,
    pub(crate) rect_color: [f32; 4],
    pub(crate) status_text: String,
}

impl TouchDemoApp {
    pub(crate) fn new() -> Self {
        Self {
            window: None,
            state: State::Uninitialized,
            compositor: Compositor::new(),
            touch_input: TouchInputState::new(),
            rect_x: 80.0,
            rect_y: 60.0,
            rect_w: 200.0,
            rect_h: 150.0,
            rect_scale: 1.0,
            rect_color: [0.2, 0.4, 0.8, 1.0],
            status_text: "Touch the rectangle to interact".to_string(),
        }
    }

    pub(crate) fn process_gestures(&mut self) {
        let events = self.touch_input.drain_events();
        for event in events {
            match event {
                GestureEvent::Tap(tap) => {
                    self.rect_color = [0.2, 0.8, 0.3, 1.0];
                    self.status_text = format!("Tap at ({:.0}, {:.0})", tap.position.x, tap.position.y);
                    log::info!("{}", self.status_text);
                }
                GestureEvent::DoubleTap(dt) => {
                    self.rect_x = 80.0;
                    self.rect_y = 60.0;
                    self.rect_scale = 1.0;
                    self.rect_color = [0.2, 0.4, 0.8, 1.0];
                    self.status_text = format!("Double-tap reset at ({:.0}, {:.0})", dt.position.x, dt.position.y);
                    log::info!("{}", self.status_text);
                }
                GestureEvent::LongPress(lp) => {
                    self.rect_color = [0.9, 0.2, 0.2, 1.0];
                    self.status_text = format!("Long press ({:.0}ms) at ({:.0}, {:.0})",
                        lp.duration.as_millis(), lp.position.x, lp.position.y);
                    log::info!("{}", self.status_text);
                }
                GestureEvent::Drag(drag) => {
                    match drag.phase {
                        Phase::Started => {
                            self.status_text = "Drag started".to_string();
                        }
                        Phase::Changed => {
                            self.rect_x += drag.delta.x as f32;
                            self.rect_y += drag.delta.y as f32;
                            self.status_text = format!("Dragging to ({:.0}, {:.0})", self.rect_x, self.rect_y);
                        }
                        Phase::Ended => {
                            self.status_text = format!("Drag ended at ({:.0}, {:.0})", self.rect_x, self.rect_y);
                        }
                        Phase::Cancelled => {
                            self.status_text = "Drag cancelled".to_string();
                        }
                    }
                }
                GestureEvent::Pinch(pinch) => {
                    match pinch.phase {
                        Phase::Changed => {
                            self.rect_scale = (self.rect_scale * (1.0 + pinch.delta_scale as f32))
                                .clamp(0.25, 4.0);
                            self.status_text = format!("Pinch scale: {:.2}", self.rect_scale);
                        }
                        Phase::Ended => {
                            self.status_text = format!("Pinch ended, scale: {:.2}", self.rect_scale);
                        }
                        _ => {}
                    }
                    log::info!("Pinch scale={:.2}", self.rect_scale);
                }
                GestureEvent::Swipe(swipe) => {
                    self.rect_color = [0.6, 0.2, 0.8, 1.0];
                    let dir = match swipe.direction {
                        SwipeDirection::Up => "Up",
                        SwipeDirection::Down => "Down",
                        SwipeDirection::Left => "Left",
                        SwipeDirection::Right => "Right",
                    };
                    self.status_text = format!("Swipe {} (vel: {:.0} px/s)", dir, swipe.velocity);
                    log::info!("{}", self.status_text);
                }
            }
        }
    }
}
