//! Makepad-inspired SDF shapes showcase.
//! Run: `cargo run --example makepad_sdf_shapes`
#![allow(dead_code)]

mod cards_row1;
mod cards_row2;
mod render;
mod shapes;

use std::sync::Arc;

use plev::animation::{Easing, FrameClock, Tween};
use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

pub(crate) enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        pool: TexturePool,
    },
}

pub(crate) struct App {
    window: Option<Arc<Window>>,
    gpu_state: GpuState,
    compositor: Compositor,
    clock: FrameClock,
    frame: u64,
    pulse: Tween<f32>,
}

impl App {
    fn new() -> Self {
        let mut pulse = Tween::new(0.0_f32, 3.0, Easing::EaseInOutSine);
        pulse.set_target(1.0);
        Self {
            window: None,
            gpu_state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            clock: FrameClock::new(),
            frame: 0,
            pulse,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- SDF Shapes")
            .with_inner_size(winit::dpi::LogicalSize::new(1100, 700));
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
                render::render(self);
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
