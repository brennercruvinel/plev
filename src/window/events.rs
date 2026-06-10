use web_time::Instant;

use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use super::state::GpuState;
use crate::platform::SafeAreaInsets;

pub(crate) enum BufferedEvent {
    CursorMoved(f32, f32),
    CursorLeft,
    MouseInput(winit::event::MouseButton, winit::event::ElementState),
    KeyboardInput(winit::event::KeyEvent),
    MouseWheel(winit::event::MouseScrollDelta),
    ModifiersChanged(winit::event::Modifiers),
    Touch(winit::event::Touch, Instant),
    Ime(winit::event::Ime),
}

impl super::App {
    /// Buffer an input event and invalidate the compositor so the next
    /// `about_to_wait` schedules a frame to process it.
    fn buffer_event(&mut self, event: BufferedEvent) {
        self.event_buffer.push(event);
        self.compositor.invalidate();
    }

    pub(crate) fn process_buffered_events(&mut self) {
        let events: Vec<BufferedEvent> = self.event_buffer.drain(..).collect();
        for event in events {
            match event {
                BufferedEvent::CursorMoved(x, y) => {
                    self.input_state.handle_cursor_moved(x, y);
                }
                BufferedEvent::CursorLeft => {
                    self.input_state.handle_cursor_left();
                }
                BufferedEvent::MouseInput(button, state) => {
                    self.input_state.handle_mouse_input(button, state);
                }
                BufferedEvent::KeyboardInput(event) => {
                    self.input_state.handle_keyboard_input(&event);
                }
                BufferedEvent::MouseWheel(delta) => {
                    self.input_state.handle_mouse_wheel(delta);
                }
                BufferedEvent::ModifiersChanged(mods) => {
                    self.input_state.handle_modifiers_changed(&mods);
                }
                BufferedEvent::Touch(touch, when) => {
                    self.touch_input.handle_touch(&touch, when);
                }
                BufferedEvent::Ime(ref ime) => {
                    let window_height = if let GpuState::Ready { ref gpu, .. } = self.state {
                        gpu.surface_config.height as f32
                    } else {
                        0.0
                    };
                    self.ime_state.handle_event(ime, window_height);
                }
            }
        }
    }

    pub(crate) fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Forward events to accessibility adapter
        #[cfg(feature = "accessibility")]
        if let Some(ref mut adapter) = self.a11y_adapter
            && let Some(ref window) = self.window
        {
            adapter.process_event(window, &event);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let GpuState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                self.compositor.invalidate();
                if let Some(ref window) = self.window {
                    self.safe_area = SafeAreaInsets::from_window(window);
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
                self.compositor.invalidate();
                log::info!("Scale factor changed: {}", scale_factor);
            }
            WindowEvent::Ime(ime) => {
                self.buffer_event(BufferedEvent::Ime(ime));
            }
            WindowEvent::RedrawRequested => {
                // Drain all buffered events before rendering
                self.process_buffered_events();
                self.touch_input.tick(Instant::now());
                for gesture in self.touch_input.drain_events() {
                    log::debug!("Gesture: {:?}", gesture);
                }
                self.render();
                // Re-schedule only while animating or when new work arrived
                // during the frame; otherwise stay idle until invalidated.
                if let Some(ref window) = self.window
                    && (self.is_animating || self.compositor.needs_render())
                {
                    window.request_redraw();
                }
            }
            WindowEvent::Touch(touch) => {
                self.buffer_event(BufferedEvent::Touch(touch, Instant::now()));
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.buffer_event(BufferedEvent::CursorMoved(
                    position.x as f32,
                    position.y as f32,
                ));
            }
            WindowEvent::CursorLeft { .. } => {
                self.buffer_event(BufferedEvent::CursorLeft);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.buffer_event(BufferedEvent::MouseInput(button, state));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.buffer_event(BufferedEvent::KeyboardInput(event));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.buffer_event(BufferedEvent::MouseWheel(delta));
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.buffer_event(BufferedEvent::ModifiersChanged(mods));
            }
            _ => {}
        }
    }
}
