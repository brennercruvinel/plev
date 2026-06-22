//! Touch and gesture demo -- 6-state gesture recognizer per touch ID.
//!
//! Run: `cargo run --example touch_demo`
#![allow(dead_code)]

mod gpu;
mod palette;
mod render;
mod state;

use std::sync::Arc;
use std::time::Instant;

use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{WindowAttributes, WindowId};

use crate::state::{State, TouchDemoApp};

impl ApplicationHandler for TouchDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Touch & Gestures")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(900, 650));
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
            WindowEvent::Touch(touch) => {
                self.touch_input.handle_touch(&touch, Instant::now());
                if let Some(ref w) = self.window {
                    w.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                self.touch_input.tick(Instant::now());
                self.process_gestures();
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
    let mut app = TouchDemoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
