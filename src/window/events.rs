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
                    // Gesture recognition (tap/double-tap/long-press/drag/
                    // pinch/swipe) for consumers of the recognizer...
                    self.touch_input.handle_touch(&touch, when);
                    // ...AND mouse-equivalent pointer synthesis from the
                    // primary finger, injected into the same `InputState`
                    // path as real mouse input, so hover/click/focus work
                    // on touch screens without widget changes.
                    let synthesized = self.touch_pointer.synthesize(
                        touch.id,
                        touch.phase,
                        touch.location.x as f32,
                        touch.location.y as f32,
                    );
                    for pointer_event in synthesized {
                        self.input_state.handle_synthetic_pointer(pointer_event);
                    }
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
                // `gpu.resize` reconfigures the surface AND resets the
                // projection to physical pixels (clearing `logical_size`).
                // Reapply the logical projection right away, otherwise on
                // HiDPI every resize would leave the scene rendering at
                // physical scale (half-size content on a 2x display).
                let scale_factor = self
                    .window
                    .as_ref()
                    .map(|w| w.scale_factor())
                    .unwrap_or(self.scale_factor);
                if let GpuState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                    let (lw, lh) = logical_projection_size(size.width, size.height, scale_factor);
                    gpu.set_projection(lw, lh);
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

/// Logical size for the projection matrix: physical surface pixels divided
/// by the window scale factor. This is what the engine `App` feeds to
/// [`GpuContext::set_projection`] after every resize, so scenes keep
/// addressing logical pixels on HiDPI displays.
///
/// Pure on purpose: `set_projection` itself needs a live GPU queue, so the
/// resize math is tested here without one (the full `Resized` path requires
/// a window + adapter and is exercised by the apps).
///
/// [`GpuContext::set_projection`]: crate::gpu::GpuContext::set_projection
pub(crate) fn logical_projection_size(
    physical_width: u32,
    physical_height: u32,
    scale_factor: f64,
) -> (f32, f32) {
    // Guard degenerate factors (uninitialized window state); 0 would
    // produce inf/NaN and a broken projection.
    let sf = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    (
        (physical_width as f64 / sf) as f32,
        (physical_height as f64 / sf) as f32,
    )
}

#[cfg(test)]
mod tests {
    use super::logical_projection_size;

    #[test]
    fn logical_projection_divides_by_scale_factor() {
        assert_eq!(logical_projection_size(2048, 1640, 2.0), (1024.0, 820.0));
    }

    #[test]
    fn logical_projection_identity_at_scale_one() {
        assert_eq!(logical_projection_size(1024, 820, 1.0), (1024.0, 820.0));
    }

    #[test]
    fn logical_projection_fractional_scale() {
        let (w, h) = logical_projection_size(1280, 800, 1.25);
        assert!((w - 1024.0).abs() < 0.001);
        assert!((h - 640.0).abs() < 0.001);
    }

    #[test]
    fn logical_projection_guards_degenerate_scale() {
        assert_eq!(logical_projection_size(800, 600, 0.0), (800.0, 600.0));
        assert_eq!(logical_projection_size(800, 600, -2.0), (800.0, 600.0));
        assert_eq!(logical_projection_size(800, 600, f64::NAN), (800.0, 600.0));
    }
}
