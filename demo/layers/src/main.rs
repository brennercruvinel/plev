//! Layer system demo, in two windows -- per-layer dirty tracking +
//! offscreen composition in the first, which animates; a second window
//! with another scene, at rest, which draws nothing while the first
//! animates (render on demand, per window). Close either: the other
//! stays. Close both: the app exits. (task-76 item 2)
//!
//! Run: `cargo run --example layers`
#![allow(dead_code)]

mod palette;
mod render;
mod ui;

use engine::compositor::LayerId;
use engine::window::{Routed, Windows};
use engine::winit::application::ApplicationHandler;
use engine::winit::event::WindowEvent;
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::window::{WindowAttributes, WindowId};

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct LayersDemoApp {
    /// Every window with what it needs (its context, compositor, input
    /// and clock); the app keeps the frame counter the animation reads.
    windows: Windows,
    /// The animated layers demo.
    main: Option<WindowId>,
    /// The other window: a card at rest.
    side: Option<WindowId>,
    /// The main window's layers, made in its compositor (a layer id names
    /// a layer of one compositor).
    bg_layer: LayerId,
    fg_layer: LayerId,
    frame_count: u64,
}

impl LayersDemoApp {
    fn new() -> Self {
        Self {
            windows: Windows::new(),
            main: None,
            side: None,
            bg_layer: LayerId::DEFAULT,
            fg_layer: LayerId::DEFAULT,
            frame_count: 0,
        }
    }
}

impl ApplicationHandler for LayersDemoApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("plev -- Layer System")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(900, 600));
        let main = self.windows.open(event_loop, attrs).unwrap();
        {
            let host = self.windows.get_mut(main).unwrap();
            let c = host.compositor_mut();
            self.bg_layer = c.create_layer(-1);
            self.fg_layer = c.create_layer(1);
            c.set_layer_opacity(self.fg_layer, 0.8);
            // the demo's foreground moves every 60 frames: a frame every
            // frame, for this window only
            host.set_animating(true);
        }
        self.main = Some(main);

        let attrs = WindowAttributes::default()
            .with_title("plev -- Layer System, the other window")
            .with_inner_size(engine::winit::dpi::LogicalSize::new(460, 320));
        let side = self.windows.open(event_loop, attrs).unwrap();
        self.side = Some(side);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match self.windows.route(id, &event) {
            Routed::Redraw => {
                if Some(id) == self.main {
                    render::render_layers(self, id);
                } else {
                    render::render_side(self, id);
                }
                // the animating window asks for its next frame; the other
                // rests until something changes in it
                if let Some(host) = self.windows.get_mut(id)
                    && host.needs_redraw()
                {
                    host.request_redraw();
                }
            }
            Routed::CloseRequested => {
                self.windows.close(id);
                if Some(id) == self.main {
                    self.main = None;
                }
                if Some(id) == self.side {
                    self.side = None;
                }
                if self.windows.should_exit() {
                    event_loop.exit();
                }
            }
            Routed::Handled | Routed::Unknown => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.windows.request_pending_redraws();
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = LayersDemoApp::new();
    event_loop.run_app(&mut app).unwrap();
}
