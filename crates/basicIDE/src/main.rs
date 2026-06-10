//! basicIDE UI — plev native port
//!
//! GPU-native 3-panel workspace UI (no Tauri, no WebView, no JavaScript).
//! Built on plev: wgpu + winit, single Rust binary.
//!
//! Run: `cargo run -p basicIDE`

// Catálogo de componentes (avatar, badge, checkbox, tabs, separator,
// panel_header) ainda não está todo conectado às views — silencia os
// dead_code até o port completar.
#![allow(dead_code)]

mod actions;
mod components;
mod renderer;
mod theme;
mod views;

use std::sync::Arc;

use plev::compositor::Compositor;
use plev::gpu::GpuContext;
use plev::text::TextSystem;
use plev::texture_pool::TexturePool;
use views::workspace::{Side, WorkspaceView};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{CursorIcon, Window, WindowAttributes, WindowId};

// ---------------------------------------------------------------------------
// GPU state
// ---------------------------------------------------------------------------

enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        _pool: TexturePool,
    },
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    workspace: WorkspaceView,
    cursor_pos: (f32, f32),
    scale_factor: f64,
}

impl App {
    fn new() -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            workspace: WorkspaceView::new(1280.0, 800.0),
            cursor_pos: (0.0, 0.0),
            scale_factor: 1.0,
        }
    }

    fn request_redraw(&self) {
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title("basicIDE — Plev native")
            .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 800u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        self.scale_factor = window.scale_factor();
        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let pool = TexturePool::new();
        self.state = GpuState::Ready {
            gpu,
            text_system,
            _pool: pool,
        };

        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let lw = size.width as f32 / sf;
        let lh = size.height as f32 / sf;
        self.workspace.resize(lw, lh);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.set_projection(lw, lh);
        }
        self.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == ElementState::Pressed {
                    if let Key::Named(NamedKey::Escape) = key_event.logical_key {
                        // Workspace handles Escape first (closes overlays).
                        // Only exit the app when no overlays are active.
                        if self.workspace.overlay_mgr.is_empty() {
                            event_loop.exit();
                        }
                    }
                    if let Key::Character(c) = &key_event.logical_key {
                        if c == "t" || c == "T" {
                            self.workspace.toggle_theme();
                            self.request_redraw();
                        }
                    }
                    // File list navigation + overlay Escape handling
                    if self.workspace.handle_key_down(&key_event.logical_key) {
                        self.request_redraw();
                    }
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                let lw = size.width as f32 / sf;
                let lh = size.height as f32 / sf;
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(lw, lh);
                }
                self.workspace.resize(lw, lh);
                self.request_redraw();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let cx = position.x as f32 / sf;
                let cy = position.y as f32 / sf;
                self.cursor_pos = (cx, cy);

                if self.workspace.dragging_left || self.workspace.dragging_right {
                    self.workspace.update_drag(cx);
                    self.request_redraw();
                }

                let old_lh = self.workspace.hover_left_handle;
                let old_rh = self.workspace.hover_right_handle;
                match self.workspace.hit_test_handle(cx) {
                    Some(Side::Left) => {
                        self.workspace.hover_left_handle = true;
                        self.workspace.hover_right_handle = false;
                    }
                    Some(Side::Right) => {
                        self.workspace.hover_right_handle = true;
                        self.workspace.hover_left_handle = false;
                    }
                    None => {
                        self.workspace.hover_left_handle = false;
                        self.workspace.hover_right_handle = false;
                    }
                }

                let on_handle = self.workspace.hover_left_handle
                    || self.workspace.hover_right_handle
                    || self.workspace.dragging_left
                    || self.workspace.dragging_right;
                if let Some(w) = &self.window {
                    w.set_cursor(if on_handle {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::Default
                    });
                }

                let handle_changed = old_lh != self.workspace.hover_left_handle
                    || old_rh != self.workspace.hover_right_handle;
                let hover_changed = if !self.workspace.overlay_mgr.is_empty() {
                    self.workspace.handle_overlay_hover(cx, cy)
                } else {
                    self.workspace.handle_hover(cx, cy)
                };
                if handle_changed || hover_changed {
                    self.request_redraw();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (cx, cy) = self.cursor_pos;
                match state {
                    ElementState::Pressed => match self.workspace.hit_test_handle(cx) {
                        Some(Side::Left) => self.workspace.begin_drag_left(cx),
                        Some(Side::Right) => self.workspace.begin_drag_right(cx),
                        None => {
                            if self.workspace.handle_click(cx, cy) {
                                self.request_redraw();
                            }
                        }
                    },
                    ElementState::Released => {
                        self.workspace.end_drag();
                        self.request_redraw();
                    }
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state: ElementState::Pressed,
                ..
            } => {
                let (cx, cy) = self.cursor_pos;
                if self.workspace.handle_right_click(cx, cy) {
                    self.request_redraw();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (cx, _) = self.cursor_pos;
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(_x, y) => -y * 20.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                self.workspace.scroll(cx, scroll_delta);
                self.request_redraw();
            }

            WindowEvent::RedrawRequested => {
                let GpuState::Ready {
                    gpu, text_system, ..
                } = &mut self.state
                else {
                    return;
                };
                renderer::render_frame(gpu, text_system, &mut self.compositor, &mut self.workspace);
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {}
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
