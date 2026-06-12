// App state, GPU state enum, and input handling logic.

use std::sync::Arc;

use plev::animation::FrameClock;
use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::gpu::texture_pool::TexturePool;
use plev::text::TextSystem;
use plev::text_input::TextInput;
use plev::winit::event::ElementState;
use plev::winit::keyboard::{Key, NamedKey};
use plev::winit::window::Window;

pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        pool: TexturePool,
    },
}

pub(crate) struct TextInputApp {
    pub(crate) window: Option<Arc<Window>>,
    pub(crate) state: GpuState,
    pub(crate) compositor: Compositor,
    pub(crate) clock: FrameClock,
    pub(crate) inputs: Vec<TextInput>,
    pub(crate) focus_index: Option<usize>,
    pub(crate) cursor_pos: (f32, f32),
}

impl TextInputApp {
    pub fn new() -> Self {
        let inputs = vec![
            TextInput::new()
                .with_placeholder("Your name...")
                .with_font_size(18.0),
            TextInput::new()
                .with_placeholder("Email address...")
                .with_font_size(18.0),
            TextInput::new()
                .with_placeholder("Write something here...")
                .with_font_size(18.0),
        ];

        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            clock: FrameClock::new(),
            inputs,
            focus_index: None,
            cursor_pos: (0.0, 0.0),
        }
    }

    pub fn focus_input(&mut self, index: usize) {
        for (i, input) in self.inputs.iter_mut().enumerate() {
            if i == index {
                input.focus();
            } else {
                input.unfocus();
            }
        }
        self.focus_index = Some(index);
    }

    pub fn cycle_focus(&mut self) {
        let next = match self.focus_index {
            Some(i) => (i + 1) % self.inputs.len(),
            None => 0,
        };
        self.focus_input(next);
    }

    pub fn handle_key_event(&mut self, event: &plev::winit::event::KeyEvent) {
        if event.state != ElementState::Pressed {
            return;
        }

        match &event.logical_key {
            Key::Named(NamedKey::Tab) => {
                self.cycle_focus();
            }
            Key::Named(NamedKey::Escape) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].unfocus();
                    self.focus_index = None;
                }
            }
            Key::Named(NamedKey::Backspace) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_backspace();
                }
            }
            Key::Named(NamedKey::Delete) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_delete();
                }
            }
            Key::Named(NamedKey::ArrowLeft) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_left();
                }
            }
            Key::Named(NamedKey::ArrowRight) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_right();
                }
            }
            Key::Named(NamedKey::Home) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_home();
                }
            }
            Key::Named(NamedKey::End) => {
                if let Some(idx) = self.focus_index {
                    self.inputs[idx].handle_end();
                }
            }
            Key::Character(c) => {
                if let Some(idx) = self.focus_index {
                    for ch in c.chars() {
                        if !ch.is_control() {
                            self.inputs[idx].handle_char(ch);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn handle_click(&mut self, x: f32, y: f32) {
        let vw = match &self.state {
            GpuState::Ready { gpu, .. } => gpu.surface_config.width as f32,
            _ => 700.0,
        };
        let input_w = (vw - 64.0).min(500.0);
        let input_x = (vw - input_w) / 2.0;
        let start_y = 106.0;
        let card_h = 62.0;
        let spacing = card_h + 16.0;

        let mut clicked_any = false;
        for (i, input) in self.inputs.iter_mut().enumerate() {
            let card_y = start_y + i as f32 * spacing;
            let field_y = card_y + 26.0;
            let ih = input.font_size * 2.0;
            if x >= input_x && x <= input_x + input_w && y >= field_y && y <= field_y + ih {
                let local_x = x - input_x - 16.0;
                input.handle_click(local_x.max(0.0));
                self.focus_index = Some(i);
                clicked_any = true;
            } else {
                input.unfocus();
            }
        }
        if !clicked_any {
            self.focus_index = None;
        }
    }
}
