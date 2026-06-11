//! Component Counter -- demonstrates the Component<T> + Lifecycle pattern.
//!
//! Run: `cargo run --example counter`
//!
//! The Counter Lifecycle increments a u64 every frame via on_update().
//! Its render() produces SceneNodes for the counter display card.
//! The app adds header, info card, and footer around it.
#![allow(dead_code)]

mod lifecycle;
mod render;

use std::sync::Arc;

use plev::component::Component;
use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

use lifecycle::Counter;

// --- App ------------------------------------------------------------------

// Ready is ~2320 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
pub(crate) enum AppState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

pub(crate) struct CounterApp {
    pub window: Option<Arc<Window>>,
    pub state: AppState,
    pub compositor: Compositor,
    pub counter: Component<Counter>,
}

impl CounterApp {
    fn new() -> Self {
        Self {
            window: None,
            state: AppState::Uninitialized,
            compositor: Compositor::new(),
            counter: Component::new(Counter),
        }
    }
}

impl ApplicationHandler for CounterApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Component Counter")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(900, 500));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = AppState::Ready { gpu, text_system };
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let AppState::Ready { ref mut gpu, .. } = self.state {
                    gpu.resize(size.width, size.height);
                }
                if let Some(ref w) = self.window {
                    w.request_redraw();
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
    let mut app = CounterApp::new();
    event_loop.run_app(&mut app).unwrap();
}
