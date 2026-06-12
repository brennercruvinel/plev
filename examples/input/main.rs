//! Input system demo -- hit regions, hover states, event queue.
//!
//! Run: `cargo run --example input`
#![allow(dead_code)]

mod gpu;
mod palette;
mod render;
mod state;

use std::sync::Arc;

use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{WindowAttributes, WindowId};

use crate::state::{InputDemoApp, State};

impl ApplicationHandler for InputDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Input System")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(900, 550));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = State::Ready { gpu, text_system };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let State::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input_state
                    .handle_cursor_moved(position.x as f32, position.y as f32);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::CursorLeft { .. } => {
                self.input_state.handle_cursor_left();
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input_state.handle_mouse_input(button, state);
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                self.input_state.handle_keyboard_input(&event);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input_state.handle_mouse_wheel(delta);
            }
            WindowEvent::ModifiersChanged(ref mods) => {
                self.input_state.handle_modifiers_changed(mods);
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
    let mut app = InputDemoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
