//! Makepad-inspired charts demo -- line, bar, area charts via PathBuilder.
//!
//! Demonstrates: Path tessellation for data visualization, animated data,
//! axis labels, grid lines, and multiple chart types in a dashboard layout.
//!
//! Run: `cargo run --example makepad_charts`
#![allow(dead_code)]

mod charts;
mod palette;
mod render;
mod state;
mod submit;

use std::sync::Arc;

use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::winit::application::ApplicationHandler;
use engine::winit::event::WindowEvent;
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::window::{WindowAttributes, WindowId};

use crate::state::{App, GpuState};

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Data Visualization")
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 700));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.gpu_state = GpuState::Ready {
            gpu,
            text_system,
            pool,
        };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _wid: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let GpuState::Ready { ref mut gpu, .. } = self.gpu_state {
                    gpu.resize(size.width, size.height);
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
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
