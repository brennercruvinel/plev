//! The shell's window events, by window: the main window's chrome and
//! page, or the page's own window (`detached.rs`). Keys go to the view
//! whichever window has the focus; the pointer goes to the page laid out
//! for the window it is in; a resize or a scale change is that window's
//! host's; the close and the redraw are answered per window.

use comps::prelude::WidgetEvent;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowId;

use super::App;
use crate::keys;

impl App {
    /// One event of one window (`ApplicationHandler::window_event`).
    pub(super) fn handle_window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
        if Some(id) == self.main
            && let Some(bridge) = self.access.bridge.as_mut()
            && let Some(window) = self.main.and_then(|m| self.windows.get(m)?.window())
        {
            bridge.process_event(window, &event);
        }
        match event {
            // The page's own window closes alone; the main window closing
            // quits.
            WindowEvent::CloseRequested => {
                if self.is_page_window(id) {
                    #[cfg(not(any(
                        target_arch = "wasm32",
                        target_os = "android",
                        target_os = "ios"
                    )))]
                    self.close_page_window();
                } else {
                    event_loop.exit();
                }
            }

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
                self.view.set_modifiers(self.modifiers);
            }

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state != ElementState::Pressed {
                    return;
                }
                // The save keys of the recording, when it records.
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                if self.record_key(&key_event.logical_key) {
                    return;
                }
                match &key_event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        if self.view.close_top_overlay() {
                            self.changed();
                        } else if self.is_page_window(id) {
                            #[cfg(not(any(
                                target_arch = "wasm32",
                                target_os = "android",
                                target_os = "ios"
                            )))]
                            self.close_page_window();
                        } else {
                            event_loop.exit();
                        }
                    }
                    // The editor owns the keyboard while it is showing.
                    key if self.view.handle_editor_key(key, self.modifiers) => {
                        self.changed();
                    }
                    // Space and the editing keys bridge through keys.rs
                    // (the view stays winit-free).
                    Key::Named(named) => {
                        if keys::handle_named(&mut self.view, named) {
                            self.changed();
                        }
                    }
                    Key::Character(c) if self.view.handle_key(c.as_str()) => {
                        self.changed();
                    }
                    _ => {}
                }
                self.detach_requested(event_loop);
            }

            WindowEvent::Resized(size) => {
                if let Some(host) = self.windows.get_mut(id) {
                    host.resize(size.width, size.height);
                }
                self.viewport_changed(id);
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // A Resized follows on most platforms, but laying out here
                // is cheap and guarantees a frame on DPI changes.
                if let Some(host) = self.windows.get_mut(id) {
                    host.scale_factor_changed(scale_factor);
                }
                self.viewport_changed(id);
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.windows.get(id).map_or(1.0, |h| h.scale_factor()) as f32;
                let (x, y) = (position.x as f32 / sf, position.y as f32 / sf);
                self.set_cursor(id, (x, y));
                if self.pointer(id, &WidgetEvent::MouseMove { x, y }) {
                    self.invalidate_in(id);
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                #[cfg(all(
                    debug_assertions,
                    any(target_os = "macos", target_os = "linux", target_os = "windows")
                ))]
                if self.view_source(id, state) {
                    return;
                }
                let (x, y) = self.cursor_in(id);
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.record_pointer(id, state, x, y);
                let ev = match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                };
                if self.pointer(id, &ev) {
                    self.changed();
                }
            }

            // The right click is the main window's (the Git page's menu,
            // the Context Menu page's demo), laid out there.
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } if Some(id) == self.main => {
                let (x, y) = self.cursor_in(id);
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.record_pointer(id, ElementState::Pressed, x, y);
                if self.view.handle_right_click(x, y) {
                    self.changed();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.cursor_in(id);
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => -dy * 24.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                if self.pointer(id, &WidgetEvent::Scroll { x, y, delta }) {
                    self.invalidate_in(id);
                }
            }

            WindowEvent::RedrawRequested => self.redraw(event_loop, id),

            // A file dragged over or dropped on a window is the engine's
            // event (it knows which platform events say so); the active
            // page takes it, whichever window it is in (task-76: the
            // Files page).
            other => {
                if let Some(drop) = engine::input::FileDrop::from_window_event(&other)
                    && self.view.handle_file_drop(&drop)
                {
                    self.changed();
                }
            }
        }
    }
}
