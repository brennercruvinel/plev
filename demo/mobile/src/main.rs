//! Mobile Input -- safe area insets, IME input, lifecycle state.
//!
//! Run: `cargo run --example mobile_input`
#![allow(dead_code)]

mod palette;
mod render;
mod state;
mod ui;

use std::sync::Arc;

use engine::gpu::GpuContext;
use engine::platform::SafeAreaInsets;
use engine::platform::lifecycle::AppState;
use engine::text::TextSystem;
use engine::winit::application::ApplicationHandler;
use engine::winit::event::WindowEvent;
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::window::{WindowAttributes, WindowId};

use crate::state::{MobileInputApp, State};

impl ApplicationHandler for MobileInputApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            self.lifecycle.transition_to(AppState::Active);
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Mobile Input")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.scale_factor = window.scale_factor();
        self.safe_area = SafeAreaInsets::from_window(&window);
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = State::Ready { gpu, text_system };
        self.lifecycle.transition_to(AppState::Active);
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.transition_to(AppState::Suspended);
    }

    fn memory_warning(&mut self, _event_loop: &ActiveEventLoop) {
        self.lifecycle.fire_memory_warning();
        if let State::Ready {
            ref mut text_system,
            ..
        } = self.state
        {
            text_system.purge_caches();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let State::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                if let Some(ref window) = self.window {
                    self.safe_area = SafeAreaInsets::from_window(window);
                    window.request_redraw();
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
            }
            WindowEvent::Ime(ref ime) => {
                let window_height = if let State::Ready { ref gpu, .. } = self.state {
                    gpu.surface_config.height as f32
                } else {
                    0.0
                };
                self.ime_state.handle_event(ime, window_height);
                if let Some(ref window) = self.window {
                    window.request_redraw();
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
    let mut app = MobileInputApp::new();
    event_loop.run_app(&mut app).unwrap();
}
