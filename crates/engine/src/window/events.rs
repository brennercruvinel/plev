//! A window's events, buffered until its next frame and then handed to
//! its `InputState`: the host's side of `Windows::route`.

use web_time::Instant;

use winit::event::WindowEvent;

use super::host::WindowHost;
use crate::input::FileDrop;

pub(crate) enum BufferedEvent {
    CursorMoved(f32, f32),
    CursorLeft,
    MouseInput(winit::event::MouseButton, winit::event::ElementState),
    KeyboardInput(winit::event::KeyEvent),
    MouseWheel(winit::event::MouseScrollDelta),
    ModifiersChanged(winit::event::Modifiers),
    Touch(winit::event::Touch, Instant),
    Ime(winit::event::Ime),
    FileDrop(FileDrop),
}

impl WindowHost {
    /// Buffer an input event and invalidate the compositor so the next
    /// `about_to_wait` schedules a frame to process it. The platform
    /// sends one drop event per file, all before the next frame: a file
    /// drop joins the one buffered just before it when both are the same
    /// phase, so the view gets one `Dropped` with every path.
    fn buffer_event(&mut self, event: BufferedEvent) {
        let event = match (event, self.events.last_mut()) {
            (BufferedEvent::FileDrop(next), Some(BufferedEvent::FileDrop(last))) => {
                last.join(next).err().map(BufferedEvent::FileDrop)
            }
            (event, _) => Some(event),
        };
        if let Some(event) = event {
            self.events.push(event);
        }
        self.compositor.invalidate();
        self.request_redraw();
    }

    /// The events buffered since the last frame, in order, into this
    /// window's input state.
    pub(crate) fn process_buffered_events(&mut self) {
        let events: Vec<BufferedEvent> = std::mem::take(&mut self.events);
        for event in events {
            match event {
                BufferedEvent::CursorMoved(x, y) => {
                    self.input_mut().handle_cursor_moved(x, y);
                }
                BufferedEvent::CursorLeft => {
                    self.input_mut().handle_cursor_left();
                }
                BufferedEvent::MouseInput(button, state) => {
                    self.input_mut().handle_mouse_input(button, state);
                }
                BufferedEvent::KeyboardInput(event) => {
                    self.input_mut().handle_keyboard_input(&event);
                }
                BufferedEvent::MouseWheel(delta) => {
                    self.input_mut().handle_mouse_wheel(delta);
                }
                BufferedEvent::ModifiersChanged(mods) => {
                    self.input_mut().handle_modifiers_changed(&mods);
                }
                BufferedEvent::Touch(touch, when) => {
                    // Gesture recognition (tap/double-tap/long-press/drag/
                    // pinch/swipe) for consumers of the recognizer...
                    self.touch.handle_touch(&touch, when);
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
                        self.input_mut().handle_synthetic_pointer(pointer_event);
                    }
                }
                BufferedEvent::Ime(ref ime) => {
                    let window_height = self.physical_size().1 as f32;
                    self.ime.handle_event(ime, window_height);
                }
                BufferedEvent::FileDrop(drop) => {
                    self.input_mut().handle_file_drop(drop);
                }
            }
        }
    }

    /// What a redraw request does first: the buffered events reach the
    /// input state, in order, then the touch recognizer ticks. The frame
    /// itself is the caller's (`render`, or a scene and `present`). A
    /// driver without a platform (a test over a bare or headless host)
    /// calls it where the platform's `RedrawRequested` would.
    pub fn process_events(&mut self) {
        self.process_buffered_events();
        self.touch.tick(Instant::now());
        for gesture in self.touch.drain_events() {
            log::debug!("Gesture: {gesture:?}");
        }
    }

    /// One window event of this window, everything but the close and the
    /// redraw request (those are `Windows::route`'s answer to its caller):
    /// a resize, a scale change and the platform's word on visibility
    /// take effect now; input, IME and file drops are buffered for the
    /// next frame. `true` when the event was this host's to take.
    pub fn handle_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(size) => {
                self.resize(size.width, size.height);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor_changed(*scale_factor);
            }
            WindowEvent::Occluded(covered) => {
                self.set_occluded(*covered);
            }
            WindowEvent::Ime(ime) => {
                self.buffer_event(BufferedEvent::Ime(ime.clone()));
            }
            WindowEvent::Touch(touch) => {
                self.buffer_event(BufferedEvent::Touch(*touch, Instant::now()));
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
                self.buffer_event(BufferedEvent::MouseInput(*button, *state));
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.buffer_event(BufferedEvent::KeyboardInput(event.clone()));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.buffer_event(BufferedEvent::MouseWheel(*delta));
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.buffer_event(BufferedEvent::ModifiersChanged(*mods));
            }
            WindowEvent::HoveredFile(_)
            | WindowEvent::DroppedFile(_)
            | WindowEvent::HoveredFileCancelled => {
                if let Some(drop) = FileDrop::from_window_event(event) {
                    self.buffer_event(BufferedEvent::FileDrop(drop));
                }
            }
            _ => return false,
        }
        true
    }

    /// Events buffered and not yet processed.
    pub fn pending_events(&self) -> usize {
        self.events.len()
    }
}

/// Logical size for the projection matrix: physical surface pixels divided
/// by the window scale factor. This is what a host feeds to
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
