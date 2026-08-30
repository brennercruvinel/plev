//! Spellbook - MTG collection and deck manager, drawn by plev.
//!
//! GPU-native window: no Tauri, no WebView, no JavaScript. The card index, the
//! collection and the decks come from `spellbook-core` running on a worker
//! thread; the UI thread never blocks on a query.
//!
//! Frames render on demand. Without input the event loop stays idle, which is
//! also why every handler that changes something visible has to say so: under
//! render-on-demand a missed invalidation is a frozen window, not a glitch.

mod art;
mod renderer;
mod view;

use std::sync::Arc;

use engine::animation::FrameClock;
use engine::compositor::Compositor;
use engine::gpu::GpuContext;
use engine::gpu::texture_pool::TexturePool;
use engine::text::TextSystem;
use engine::ui::widgets::WidgetEvent;
use spellbook_core::client::{Command, Event, SpellbookClient};
use view::SpellbookView;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

/// Events injected into the winit loop from the data thread.
enum AppEvent {
    Data(Event),
}

// Ready is ~2.4 KB against 0 for Uninitialized; one instance lives for the
// whole process, so boxing would only add indirection on the render path.
#[allow(clippy::large_enum_variant)]
enum GpuState {
    Uninitialized,
    Ready {
        gpu: GpuContext,
        text_system: TextSystem,
        effects: engine::effects::EffectProcessor,
        texture_pool: TexturePool,
    },
}

struct App {
    window: Option<Arc<Window>>,
    state: GpuState,
    compositor: Compositor,
    view: SpellbookView,
    clock: FrameClock,
    cursor: (f32, f32),
    scale_factor: f64,
    /// The data thread. Dropping it stops the worker and joins it.
    data: SpellbookClient,
}

impl App {
    fn new(data: SpellbookClient) -> Self {
        let mut view = SpellbookView::new(1400.0, 900.0, data.sender());
        view.boot();
        Self {
            window: None,
            state: GpuState::Uninitialized,
            compositor: Compositor::new(),
            view,
            clock: FrameClock::new(),
            cursor: (0.0, 0.0),
            scale_factor: 1.0,
            data,
        }
    }

    /// Mark the scene changed and schedule a frame. Frames are rendered on
    /// demand only: without this the window would sit on a stale image.
    fn invalidate(&mut self) {
        self.compositor.invalidate();
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }

    /// Sync surface, projection and layout to the window's current size.
    fn configure_viewport(&mut self) {
        let Some(window) = self.window.clone() else {
            return;
        };
        let size = window.inner_size();
        let sf = self.scale_factor as f32;
        let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
        if let GpuState::Ready { gpu, .. } = &mut self.state {
            gpu.resize(size.width, size.height);
            gpu.set_projection(lw, lh);
        }
        self.view.resize(lw, lh, sf);
        self.invalidate();
    }
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = WindowAttributes::default()
            .with_title("Spellbook")
            .with_inner_size(winit::dpi::LogicalSize::new(1400u32, 900u32))
            .with_min_inner_size(winit::dpi::LogicalSize::new(900u32, 600u32));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());

        self.window = Some(window.clone());
        self.scale_factor = window.scale_factor();

        let gpu = pollster::block_on(GpuContext::new(window));
        let text_system = TextSystem::new(&gpu.device, &gpu.text_bind_group_layout);
        let effects = engine::effects::EffectProcessor::new(&gpu.device, gpu.surface_format());
        self.state = GpuState::Ready {
            gpu,
            text_system,
            effects,
            texture_pool: TexturePool::new(),
        };
        self.configure_viewport();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::Data(Event::Ready) => log::info!("data thread ready"),
            AppEvent::Data(event) => {
                if self.view.handle_data(&event) {
                    self.invalidate();
                }
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                if key_event.state == ElementState::Pressed
                    && matches!(key_event.logical_key, Key::Named(NamedKey::Escape))
                {
                    event_loop.exit();
                }
            }

            WindowEvent::Resized(size) => {
                let sf = self.scale_factor as f32;
                let (lw, lh) = (size.width as f32 / sf, size.height as f32 / sf);
                if let GpuState::Ready { gpu, .. } = &mut self.state {
                    gpu.resize(size.width, size.height);
                    gpu.set_projection(lw, lh);
                }
                self.view.resize(lw, lh, sf);
                self.invalidate();
            }

            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                // A Resized follows on most platforms, but invalidating here
                // is cheap and guarantees a frame on a DPI change.
                self.scale_factor = scale_factor;
                self.invalidate();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let sf = self.scale_factor as f32;
                let (x, y) = (position.x as f32 / sf, position.y as f32 / sf);
                self.cursor = (x, y);
                if self.view.handle_event(&WidgetEvent::MouseMove { x, y }) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                let (x, y) = self.cursor;
                let ev = match state {
                    ElementState::Pressed => WidgetEvent::MouseDown { x, y },
                    ElementState::Released => WidgetEvent::MouseUp { x, y },
                };
                if self.view.handle_event(&ev) {
                    self.invalidate();
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.cursor;
                let delta = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => -dy * 24.0,
                    MouseScrollDelta::PixelDelta(pos) => -pos.y as f32,
                };
                if self.view.handle_event(&WidgetEvent::Scroll { x, y, delta }) {
                    self.invalidate();
                }
            }

            WindowEvent::RedrawRequested => {
                let GpuState::Ready {
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                } = &mut self.state
                else {
                    return;
                };
                let tick = self.clock.tick();
                let animating = self.view.tick(tick.dt);
                renderer::render_frame(
                    gpu,
                    text_system,
                    effects,
                    texture_pool,
                    &mut self.compositor,
                    &mut self.view,
                );
                // Art the screens asked for while laying out this frame goes
                // to the worker as one batch. 640 logical px covers the card
                // modal; grid tiles draw smaller and downscale fine.
                let rels = self.view.take_art_requests();
                if !rels.is_empty() {
                    let max_edge = (640.0 * self.scale_factor).ceil().clamp(256.0, 1024.0) as u32;
                    self.data.send(Command::LoadArt { rels, max_edge });
                    // The answer lands as an event, which invalidates then.
                }
                if animating {
                    self.compositor.invalidate();
                    if let Some(w) = &self.window {
                        w.request_redraw();
                    }
                }
            }

            _ => {}
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    spellbook_core::init();

    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    // The worker answers on its own thread; the proxy is how a result becomes
    // a frame. Without it a finished query would sit unrendered until the next
    // mouse move, because nothing else wakes an idle render-on-demand loop.
    let proxy: EventLoopProxy<AppEvent> = event_loop.create_proxy();
    let data = SpellbookClient::spawn(move |event| {
        let _ = proxy.send_event(AppEvent::Data(event));
    });

    let mut app = App::new(data);
    event_loop.run_app(&mut app).unwrap();
}
