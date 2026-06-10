use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use winit::keyboard::{Key, NamedKey};

use super::touch::SyntheticPointerEvent;
use super::types::*;

impl super::InputState {
    pub fn handle_cursor_moved(&mut self, x: f32, y: f32) {
        self.cursor_position = Some((x, y));
        let new_hover = self.hit_test(x, y);
        let old_hover = self.hovered_view;

        if new_hover != old_hover {
            if let Some(old_id) = old_hover {
                self.pending_events.push(InputEvent::Hover(HoverEvent {
                    view_id: old_id,
                    position: (x, y),
                    entered: false,
                }));
            }
            if let Some(new_id) = new_hover {
                self.pending_events.push(InputEvent::Hover(HoverEvent {
                    view_id: new_id,
                    position: (x, y),
                    entered: true,
                }));
            }
            self.hovered_view = new_hover;
        }
    }

    pub fn handle_cursor_left(&mut self) {
        if let Some(old_id) = self.hovered_view.take() {
            let pos = self.cursor_position.unwrap_or((0.0, 0.0));
            self.pending_events.push(InputEvent::Hover(HoverEvent {
                view_id: old_id,
                position: pos,
                entered: false,
            }));
        }
        self.cursor_position = None;
    }

    pub fn handle_mouse_input(&mut self, button: MouseButton, state: ElementState) {
        let (x, y) = match self.cursor_position {
            Some(pos) => pos,
            None => return,
        };

        let view_id = match self.hit_test(x, y) {
            Some(id) => id,
            None => {
                if state == ElementState::Pressed {
                    self.focused_view = None;
                }
                return;
            }
        };

        if state == ElementState::Pressed {
            self.focused_view = self.hit_test_focusable(x, y);
        }

        self.pending_events.push(InputEvent::Click(ClickEvent {
            view_id,
            position: (x, y),
            button: button.into(),
            state: state.into(),
            modifiers: self.modifiers,
        }));
    }

    pub fn handle_keyboard_input(&mut self, event: &winit::event::KeyEvent) {
        if event.state == ElementState::Pressed
            && let Key::Named(NamedKey::Escape) = &event.logical_key
        {
            self.focused_view = None;
            return;
        }

        let view_id = match self.focused_view {
            Some(id) => id,
            None => return,
        };

        let key = match &event.logical_key {
            Key::Named(named) => KeyInput::Named(*named),
            Key::Character(s) => KeyInput::Character(s.to_string()),
            _ => return,
        };

        self.pending_events.push(InputEvent::Key(PlevKeyEvent {
            view_id,
            key,
            state: event.state.into(),
            text: event.text.as_ref().map(|s| s.to_string()),
            modifiers: self.modifiers,
            repeat: event.repeat,
        }));
    }

    pub fn handle_mouse_wheel(&mut self, delta: MouseScrollDelta) {
        let (x, y) = match self.cursor_position {
            Some(pos) => pos,
            None => return,
        };

        let view_id = match self.hit_test(x, y) {
            Some(id) => id,
            None => return,
        };

        let (delta_x, delta_y) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x, y),
            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
        };

        self.pending_events.push(InputEvent::Scroll(ScrollEvent {
            view_id,
            position: (x, y),
            delta_x,
            delta_y,
            modifiers: self.modifiers,
        }));
    }

    /// Inject a touch-synthesized pointer event (see
    /// [`TouchPointerSynth`](super::touch::TouchPointerSynth)) into the
    /// exact same handlers real mouse input goes through, so hover, focus,
    /// click and hit-testing behave identically for touch and mouse.
    pub fn handle_synthetic_pointer(&mut self, event: SyntheticPointerEvent) {
        match event {
            SyntheticPointerEvent::CursorMoved { x, y } => self.handle_cursor_moved(x, y),
            SyntheticPointerEvent::PrimaryButtonDown => {
                self.handle_mouse_input(MouseButton::Left, ElementState::Pressed)
            }
            SyntheticPointerEvent::PrimaryButtonUp => {
                self.handle_mouse_input(MouseButton::Left, ElementState::Released)
            }
            SyntheticPointerEvent::CursorLeft => self.handle_cursor_left(),
        }
    }

    pub fn handle_modifiers_changed(&mut self, mods: &winit::event::Modifiers) {
        let state = mods.state();
        self.modifiers = ModifierState {
            shift: state.shift_key(),
            ctrl: state.control_key(),
            alt: state.alt_key(),
            meta: state.super_key(),
        };
    }
}
