//! Editor demo: a window filled by the plev multi-line editor.
//!
//! Run: `cargo run --example editor [file]`
//!
//! Opens `file` when given (created on save if missing), otherwise an
//! embedded ~200-line demo text. `cmd-s` saves, `escape` quits. Multi-cursor
//! via alt+click, word/line selection via double/triple click, IME preedit
//! rendered inline.

mod render;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use engine::compositor::Compositor;
use engine::editor::{EditorTheme, EditorView, MouseEvent};
use engine::gpu::GpuContext;
use engine::layout::ComputedBounds;
use engine::text::TextSystem;
use rope::Document;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

// ---------------------------------------------------------------------------
// GPU state
// ---------------------------------------------------------------------------

// Ready is ~2320 B vs 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
    },
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    editor: EditorView,
    theme: EditorTheme,
    /// Save target (argv); `None` when editing the embedded demo text.
    path: Option<PathBuf>,
    modifiers: ModifiersState,
    cursor_pos: (f32, f32),
    mouse_down: bool,
    scale_factor: f64,
    logical_size: (f32, f32),
    last_tick: Instant,
}

impl App {
    fn new(path: Option<PathBuf>, text: String) -> Self {
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            editor: EditorView::new(Document::load(&text)),
            theme: EditorTheme::default(),
            path,
            modifiers: ModifiersState::empty(),
            cursor_pos: (0.0, 0.0),
            mouse_down: false,
            scale_factor: 1.0,
            logical_size: (0.0, 0.0),
            last_tick: Instant::now(),
        }
    }

    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    fn editor_bounds(&self) -> ComputedBounds {
        ComputedBounds {
            x: 0.0,
            y: 0.0,
            width: self.logical_size.0,
            height: self.logical_size.1,
        }
    }

    fn save(&self) {
        let Some(path) = &self.path else {
            eprintln!("no file argument given; nothing to save");
            return;
        };
        match std::fs::write(path, self.editor.document.to_string()) {
            Ok(()) => println!("saved {}", path.display()),
            Err(e) => eprintln!("save failed: {e}"),
        }
    }

    /// Keep the IME candidate window glued to the caret.
    fn update_ime_area(&self) {
        let Some(window) = &self.window else { return };
        let rect = self.editor.ime_cursor_rect();
        window.set_ime_cursor_area(
            LogicalPosition::new(rect.x as f64, rect.y as f64),
            LogicalSize::new(rect.width as f64, rect.height as f64),
        );
    }

    fn render_frame(&mut self) {
        let GpuState::Ready { gpu, text_system } = &mut self.state else {
            return;
        };

        self.compositor.begin_frame();
        let bounds = ComputedBounds {
            x: 0.0,
            y: 0.0,
            width: self.logical_size.0,
            height: self.logical_size.1,
        };
        self.editor
            .render(&mut self.compositor, bounds, &self.theme);

        render::present_frame(
            gpu,
            text_system,
            &mut self.compositor,
            self.theme.background,
        );
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let title = match &self.path {
            Some(p) => format!("plev editor — {}", p.display()),
            None => "plev editor — demo".to_string(),
        };
        let attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(LogicalSize::new(1100u32, 760u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        window.set_ime_allowed(true);
        self.window = Some(window.clone());

        self.scale_factor = window.scale_factor();
        let gpu = pollster::block_on(GpuContext::new(window.clone()));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        self.state = GpuState::Ready { gpu, text_system };

        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        self.logical_size = (size.width as f32 / sf, size.height as f32 / sf);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.set_projection(self.logical_size.0, self.logical_size.1);
        }
        self.editor.set_bounds(self.editor_bounds());
        self.invalidate();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(mods) => self.modifiers = mods.state(),

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state != ElementState::Pressed {
                    return;
                }
                let primary = self.modifiers.super_key() || self.modifiers.control_key();
                match &key_event.logical_key {
                    Key::Named(NamedKey::Escape) => event_loop.exit(),
                    Key::Character(c) if primary && c.as_str() == "s" => self.save(),
                    key => {
                        if self.editor.handle_key(key, self.modifiers) {
                            self.update_ime_area();
                            self.invalidate();
                        }
                    }
                }
            }

            WindowEvent::Ime(ime) => {
                if self.editor.handle_ime(&ime) {
                    self.update_ime_area();
                    self.invalidate();
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                self.logical_size = (size.width as f32 / sf, size.height as f32 / sf);
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(self.logical_size.0, self.logical_size.1);
                }
                self.editor.set_bounds(self.editor_bounds());
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor;
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                self.cursor_pos = (position.x as f32 / sf, position.y as f32 / sf);
                if self.mouse_down {
                    let (x, y) = self.cursor_pos;
                    if self.editor.handle_mouse(MouseEvent::Drag { x, y }) {
                        self.invalidate();
                    }
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (x, y) = self.cursor_pos;
                let handled = match state {
                    ElementState::Pressed => {
                        self.mouse_down = true;
                        self.editor.handle_mouse(MouseEvent::Down {
                            x,
                            y,
                            alt: self.modifiers.alt_key(),
                            shift: self.modifiers.shift_key(),
                        })
                    }
                    ElementState::Released => {
                        self.mouse_down = false;
                        self.editor.handle_mouse(MouseEvent::Up)
                    }
                };
                if handled {
                    self.update_ime_area();
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, y) => -y * self.editor.config.line_height,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                if self.editor.handle_mouse(MouseEvent::Wheel { dy }) {
                    self.invalidate();
                }
            }

            WindowEvent::RedrawRequested => self.render_frame(),

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Cursor blink clock: wake up at the configured interval and redraw
        // when the caret toggles; everything else renders on demand only.
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f32();
        self.last_tick = now;
        if self.editor.tick(dt) {
            self.invalidate();
        }
        let interval = Duration::from_secs_f32(self.editor.config.cursor_blink_interval);
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + interval));
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    let path = std::env::args().nth(1).map(PathBuf::from);
    let text = match &path {
        Some(p) => std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("could not read {} ({e}); starting empty", p.display());
            String::new()
        }),
        None => demo_text(),
    };

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(path, text);
    event_loop.run_app(&mut app).unwrap();
}

/// ~200 lines of representative text for the no-argument case.
fn demo_text() -> String {
    let mut s = String::new();
    s.push_str("// plev editor demo\n");
    s.push_str("//\n");
    s.push_str("// Try it out:\n");
    s.push_str("//   - click / drag to select, double-click a word, triple-click a line\n");
    s.push_str("//   - alt+click to add cursors, then type\n");
    s.push_str("//   - cmd-c / cmd-x / cmd-v with multiple cursors\n");
    s.push_str("//   - cmd-z / cmd-shift-z, cmd-a, home/end, pageup/pagedown\n");
    s.push_str("//   - an IME (e.g. Japanese) composes inline with an underline\n");
    s.push_str("//   - cmd-s saves when a file argument was given\n");
    s.push('\n');
    for i in 0..38 {
        s.push_str(&format!("/// Block {i}: five lines of sample content.\n"));
        s.push_str(&format!("fn sample_{i}(input: &str) -> usize {{\n"));
        s.push_str(&format!("    let value = input.len() + {i};\n"));
        s.push_str("    value * 2\n");
        s.push_str("}\n");
    }
    s
}
