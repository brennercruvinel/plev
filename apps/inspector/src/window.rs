//! the desktop window: winit (through the engine) and one `WindowHost`
//! around the view. it turns the window's events into the view's handlers
//! and draws a frame only when one of them says the view changed (render
//! on demand; nothing here animates). it opens nothing but the window: no
//! socket, no process, no file besides the session the command line named.

use std::sync::Arc;

use engine::widgets::content::CodeBlock;
use engine::runtime::window::WindowHost;
use engine::winit::application::ApplicationHandler;
use engine::winit::dpi::LogicalSize;
use engine::winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use engine::winit::event_loop::{ActiveEventLoop, EventLoop};
use engine::winit::keyboard::{Key as Pressed, NamedKey};
use engine::winit::window::{Window, WindowId};

use crate::view::{InspectorView, Key};

struct App {
    view: InspectorView,
    title: String,
    host: Option<WindowHost>,
    /// the pointer, in logical pixels.
    pointer: (f32, f32),
}

/// opens the window on `view` and runs until it closes.
pub fn run(view: InspectorView, title: String) -> Result<(), String> {
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = App {
        view,
        title,
        host: None,
        pointer: (0.0, 0.0),
    };
    event_loop.run_app(&mut app).map_err(|e| e.to_string())
}

impl App {
    fn scale(&self) -> f32 {
        self.host.as_ref().map_or(1.0, |h| h.scale_factor() as f32)
    }

    fn redraw_if(&mut self, changed: bool) {
        if changed && let Some(host) = self.host.as_mut() {
            host.request_redraw();
        }
    }

    fn frame(&mut self) {
        let Some(host) = self.host.as_mut() else {
            return;
        };
        host.begin_frame();
        self.view.render(host.compositor_mut());
        let clear = self.view.theme().colors.bg.to_linear_array();
        host.present(clear, "inspector_frame");
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.host.is_some() {
            return;
        }
        let (w, h) = self.view.size();
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(LogicalSize::new(f64::from(w), f64::from(h)));
        let window = match event_loop.create_window(attrs) {
            Ok(window) => Arc::new(window),
            Err(e) => {
                eprintln!("plev-inspector: no window: {e}");
                event_loop.exit();
                return;
            }
        };
        let host = WindowHost::open(window);
        let (lw, lh) = host.logical_size();
        self.view.resize(lw, lh);
        self.host = Some(host);
        self.redraw_if(true);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(host) = self.host.as_mut() {
                    host.resize(size.width, size.height);
                    let (lw, lh) = host.logical_size();
                    self.view.resize(lw, lh);
                }
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(host) = self.host.as_mut() {
                    host.scale_factor_changed(scale_factor);
                    let (lw, lh) = host.logical_size();
                    self.view.resize(lw, lh);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let s = self.scale();
                self.pointer = (position.x as f32 / s, position.y as f32 / s);
                let changed = self.view.pointer_move(self.pointer.0, self.pointer.1);
                self.redraw_if(changed);
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                let (x, y) = self.pointer;
                let changed = match state {
                    ElementState::Pressed => self.view.pointer_down(x, y),
                    ElementState::Released => self.view.pointer_up(x, y),
                };
                self.redraw_if(changed);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.pointer;
                let line = CodeBlock::row_h(self.view.theme());
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => -dy * line,
                    MouseScrollDelta::PixelDelta(p) => -(p.y as f32) / self.scale(),
                };
                let changed = self.view.scroll(x, y, delta);
                self.redraw_if(changed);
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let key = match event.logical_key {
                    Pressed::Named(NamedKey::ArrowLeft) => Some(Key::Previous),
                    Pressed::Named(NamedKey::ArrowRight) => Some(Key::Next),
                    Pressed::Named(NamedKey::Home) => Some(Key::First),
                    Pressed::Named(NamedKey::End) => Some(Key::Last),
                    Pressed::Named(NamedKey::Escape) => Some(Key::Close),
                    _ => None,
                };
                if let Some(key) = key {
                    let changed = self.view.key(key);
                    self.redraw_if(changed);
                }
            }
            WindowEvent::RedrawRequested => self.frame(),
            _ => {}
        }
    }
}
