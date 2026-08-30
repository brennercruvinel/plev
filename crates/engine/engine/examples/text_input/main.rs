//! Text input demo -- editable fields with cursor, blink, and keyboard navigation.
//!
//! Run: `cargo run --example text_input_demo`
#![allow(dead_code)]

mod palette;
mod render;
mod state;

use std::sync::Arc;

use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::winit::application::ApplicationHandler;
use engine::winit::event::{ElementState, WindowEvent};
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::window::{WindowAttributes, WindowId};

use crate::state::{GpuState, TextInputApp};

impl ApplicationHandler for TextInputApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Text Input Demo")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(700, 520));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.state = GpuState::Ready {
            gpu,
            text_system,
            pool,
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let GpuState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key_event(&event);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: engine::winit::event::MouseButton::Left,
                ..
            } => {
                self.handle_click(self.cursor_pos.0, self.cursor_pos.1);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.render();
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = TextInputApp::new();
    event_loop.run_app(&mut app).unwrap();
}
