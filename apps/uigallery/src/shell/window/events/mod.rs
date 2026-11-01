//! The shell's window events, by window: the main window's chrome and
//! page, or the page's own window (`detached.rs`). Keys go to the view
//! whichever window has the focus; the pointer goes to the page laid out
//! for the window it is in; a resize or a scale change is that window's
//! host's; the close and the redraw are answered per window. The input
//! method's events go to the view as well (`ime.rs`).

use engine::widgets::prelude::WidgetEvent;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::WindowId;

use super::super::App;
use crate::keyboard;

impl App {
    /// One event of one window (`ApplicationHandler::window_event`).
    pub(crate) fn handle_window_event(
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
                // A literal edited in place owns the keyboard (`source`).
                #[cfg(all(
                    debug_assertions,
                    any(target_os = "macos", target_os = "linux", target_os = "windows")
                ))]
                if self.edit_key(&key_event.logical_key) {
                    return;
                }
                // The save keys of the recording, when it records.
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                if self.record_key(&key_event.logical_key) {
                    return;
                }
                // The print keys: `shell.print`, the person's request
                // (task-76 item 9), desktop only.
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                if self.print_key(&key_event.logical_key) {
                    return;
                }
                // The platform's window keys close the window or quit;
                // Escape never does (below).
                #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
                if let Some(action) = window_key(&key_event.logical_key, self.modifiers) {
                    match action {
                        WindowKey::Close if self.is_page_window(id) => self.close_page_window(),
                        WindowKey::Close | WindowKey::Quit => event_loop.exit(),
                    }
                    return;
                }
                match &key_event.logical_key {
                    // Escape closes what is open, and only that: with
                    // nothing open it does nothing (leaving the app is the
                    // platform's key, `window_key`).
                    Key::Named(NamedKey::Escape) => {
                        if self.view.close_top_overlay() {
                            self.changed();
                        }
                    }
                    // The editor owns the keyboard while it is showing.
                    key if self.view.handle_editor_key(key, self.modifiers) => {
                        self.changed();
                    }
                    // Space and the editing keys bridge through keys.rs
                    // (the view stays winit-free).
                    Key::Named(named) => {
                        if keyboard::handle_named(&mut self.view, named, self.modifiers.shift_key())
                        {
                            self.changed();
                        }
                    }
                    // With cmd or ctrl held a letter is never typed: a
                    // focused field's clipboard keys (`view/clipboard.rs`).
                    Key::Character(c) if chord(self.modifiers) => {
                        if self.view.handle_chord(c.as_str(), primary(self.modifiers)) {
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
                // a pointer moved: press areas back to the pointer's
                // minimum (`engine::widgets::hit`)
                engine::widgets::hit::set_touch_input(false);
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

            // a finger (android, iOS): a tap clicks, a drag scrolls
            // a finger: press areas grow to the touch minimum
            WindowEvent::Touch(touch) => {
                engine::widgets::hit::set_touch_input(true);
                self.touched(id, &touch);
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

            WindowEvent::RedrawRequested => {
                self.redraw(event_loop, id);
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.sync_ime(id);
            }

            // The input method: a composition, a commit (`ime.rs`).
            WindowEvent::Ime(ime) => self.ime(&ime),

            // The platform says the window is covered (minimized, or
            // fully behind others), or shows again: its host stops asking
            // for frames, or asks for one new one (`sleep.rs`).
            WindowEvent::Occluded(covered) => self.covered(id, covered),

            // A window of ours took or lost the focus: while none has it,
            // what lands is told to the person by notification (desktop).
            WindowEvent::Focused(on) => {
                #[cfg(not(any(
                    target_arch = "wasm32",
                    target_os = "android",
                    target_os = "ios"
                )))]
                self.focus_changed(id, on);
                #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "ios"))]
                let _ = on;
            }

            // A file dragged over or dropped on a window is the engine's
            // event (it knows which platform events say so); the active
            // page takes it, whichever window it is in (task-76: the
            // Files page).
            other => {
                if let Some(drop) = engine::control::input::FileDrop::from_window_event(&other)
                    && self.view.handle_file_drop(&drop)
                {
                    self.changed();
                }
            }
        }
    }
}

/// cmd or ctrl is held, and not as AltGr (ctrl with alt), which types.
fn chord(modifiers: ModifiersState) -> bool {
    modifiers.super_key() || (modifiers.control_key() && !modifiers.alt_key())
}

/// The platform's primary modifier is held: cmd on macOS, ctrl elsewhere
/// (the print keys' rule, `print.rs`).
fn primary(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.super_key()
    } else {
        modifiers.control_key()
    }
}

/// What a window key asks of the shell, before the page sees the key.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowKey {
    /// The window closes, as its close button does (the main one quits).
    Close,
    /// The app quits.
    Quit,
}

/// The platform's keys for leaving: the primary with W closes the window,
/// with Q quits (on macOS the menu's Quit has cmd-q too; Alt-F4 on
/// Windows arrives as `CloseRequested`). Escape is never one of them: it
/// closes what is open, nothing more.
#[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "ios")))]
fn window_key(key: &Key, modifiers: ModifiersState) -> Option<WindowKey> {
    let Key::Character(c) = key else {
        return None;
    };
    if !primary(modifiers) || modifiers.alt_key() {
        return None;
    }
    match c.as_str() {
        "w" | "W" => Some(WindowKey::Close),
        "q" | "Q" => Some(WindowKey::Quit),
        _ => None,
    }
}

/// The input method (task-74 phase 4).
mod ime;

#[cfg(test)]
mod tests {
    use winit::keyboard::ModifiersState as M;

    use winit::keyboard::{Key, NamedKey};

    use super::{WindowKey, chord, primary, window_key};

    /// cmd or ctrl makes a chord, AltGr (ctrl with alt) does not; the
    /// primary is cmd on macOS and ctrl elsewhere.
    #[test]
    fn a_chord_is_cmd_or_ctrl_and_the_primary_is_the_platforms() {
        assert!(chord(M::SUPER) && chord(M::CONTROL));
        assert!(chord(M::SUPER | M::SHIFT) && chord(M::SUPER | M::ALT));
        assert!(!chord(M::CONTROL | M::ALT), "AltGr types");
        assert!(!chord(M::SHIFT) && !chord(M::ALT) && !chord(M::empty()));
        let (mine, other) = if cfg!(target_os = "macos") {
            (M::SUPER, M::CONTROL)
        } else {
            (M::CONTROL, M::SUPER)
        };
        assert!(primary(mine) && !primary(other));
    }

    /// Escape never leaves the app, with or without a modifier; the
    /// primary with W closes the window and with Q quits; the other
    /// modifier, or a plain W, types.
    #[test]
    fn escape_never_quits_and_leaving_is_the_platforms_key() {
        let esc = Key::Named(NamedKey::Escape);
        let (mine, other) = if cfg!(target_os = "macos") {
            (M::SUPER, M::CONTROL)
        } else {
            (M::CONTROL, M::SUPER)
        };
        for mods in [M::empty(), mine, other, M::SHIFT] {
            assert_eq!(window_key(&esc, mods), None, "Escape with {mods:?}");
        }
        let w = Key::Character("w".into());
        let q = Key::Character("q".into());
        assert_eq!(window_key(&w, mine), Some(WindowKey::Close));
        assert_eq!(window_key(&q, mine), Some(WindowKey::Quit));
        assert_eq!(
            window_key(&Key::Character("W".into()), mine | M::SHIFT),
            Some(WindowKey::Close)
        );
        assert_eq!(window_key(&w, M::empty()), None, "a plain w types");
        assert_eq!(window_key(&w, other), None);
        assert_eq!(window_key(&q, mine | M::ALT), None, "AltGr types");
    }
}
