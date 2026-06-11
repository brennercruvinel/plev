//! Layer system demo -- per-layer dirty tracking + offscreen composition.
//!
//! Run: `cargo run --example layers_demo`
#![allow(dead_code)]

mod palette;
mod render;
mod ui;

use std::sync::Arc;

use plev::compositor::{Compositor, LayerId};
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::winit::application::ApplicationHandler;
use plev::winit::event::WindowEvent;
use plev::winit::event_loop::{ActiveEventLoop, EventLoop};
use plev::winit::window::{Window, WindowAttributes, WindowId};

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

enum State {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

struct LayersDemoApp {
    window: Option<Arc<Window>>,
    state: State,
    compositor: Compositor,
    bg_layer: LayerId,
    fg_layer: LayerId,
    frame_count: u64,
}

impl LayersDemoApp {
    fn new() -> Self {
        let mut compositor = Compositor::new();
        let bg_layer = compositor.create_layer(-1);
        let fg_layer = compositor.create_layer(1);
        compositor.set_layer_opacity(fg_layer, 0.8);

        Self {
            window: None,
            state: State::Uninitialized,
            compositor,
            bg_layer,
            fg_layer,
            frame_count: 0,
        }
    }
}

impl ApplicationHandler for LayersDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Layer System")
            .with_inner_size(plev::winit::dpi::LogicalSize::new(900, 600));
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
    let mut app = LayersDemoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
